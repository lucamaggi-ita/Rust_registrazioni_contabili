//! Adattatore verso `rust_xlsxwriter` (fase 3C.4.1, decisione D18).
//!
//! # Che cosa fa, e che cosa non fa
//!
//! Traduce il modello dell'export in fogli e celle della libreria e
//! restituisce i **byte** del pacchetto. Non apre file: l'IO sta in
//! `export_io.rs`, e questa separazione e' cio' che rende provabile il
//! pacchetto senza toccare il disco.
//!
//! Non contiene XML ne' ZIP scritti a mano: D18 ha scelto la libreria, e
//! correggere a mano i byte che produce e' fuori perimetro.
//!
//! # L'unica eccezione di `f64` (D26(a), `EXP-53`)
//!
//! `Worksheet::write_number` prende `impl Into<f64>`: la libreria non ha un
//! percorso numerico che eviti la conversione. L'eccezione vive in
//! [`escrever_numero_de_comodo`] e **soltanto** la'. Ogni importo autorevole
//! passa invece per `write_string` con il testo esatto di
//! `Decimal::to_string()`, scala compresa.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_xlsxwriter::{Workbook, Worksheet, XlsxError};

use crate::export::{Celula, ModeloExport};

/// Errore dell'adattatore. Separato da `ErroExport` perche' nasce dalla
/// libreria, non dal modello.
#[derive(Debug)]
pub enum ErroXlsx {
    /// La libreria ha rifiutato una scrittura.
    Biblioteca(XlsxError),
    /// Un decimale eleggibile che non si converte in `f64`: non accade nel
    /// dominio delle 15 cifre significative, e **non** si sostituisce con uno
    /// zero.
    NumeroNaoConvertivel {
        folha: &'static str,
        linha: usize,
        coluna: usize,
        valor: String,
    },
    /// Oltre i limiti di riga o colonna del formato.
    ForaDosLimites {
        folha: &'static str,
        linhas: usize,
        colunas: usize,
    },
}

impl std::fmt::Display for ErroXlsx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroXlsx::Biblioteca(e) => write!(f, "rust_xlsxwriter: {e}"),
            ErroXlsx::NumeroNaoConvertivel {
                folha,
                linha,
                coluna,
                valor,
            } => write!(
                f,
                "decimal {valor} em {folha} linha {linha} coluna {coluna} nao \
                 se converte: nenhum zero substituto e' escrito"
            ),
            ErroXlsx::ForaDosLimites {
                folha,
                linhas,
                colunas,
            } => write!(
                f,
                "folha {folha} fora dos limites do formato: {linhas} linhas, \
                 {colunas} colunas"
            ),
        }
    }
}

impl std::error::Error for ErroXlsx {}

impl From<XlsxError> for ErroXlsx {
    fn from(erro: XlsxError) -> Self {
        ErroXlsx::Biblioteca(erro)
    }
}

/// Costruisce il pacchetto XLSX e restituisce i **byte**.
///
/// Non apre file: `save_to_buffer` scrive su un `Cursor` in memoria
/// (`workbook.rs:1402`, misurato da `TX-38`). Il testo passa alla libreria
/// **come e'**: nessuna trasformazione nostra, perche' la codifica dentro il
/// pacchetto e' sua (`EXP-17`, condizione di D22).
pub fn pacote(modelo: &ModeloExport) -> Result<Vec<u8>, ErroXlsx> {
    let mut livro = Workbook::new();
    for folha_modelo in &modelo.folhas {
        let folha = livro.add_worksheet();
        folha.set_name(folha_modelo.nome)?;

        let colunas =
            u16::try_from(folha_modelo.cabecalho.len()).map_err(|_| ErroXlsx::ForaDosLimites {
                folha: folha_modelo.nome,
                linhas: folha_modelo.linhas.len(),
                colunas: folha_modelo.cabecalho.len(),
            })?;
        for (j, nome) in folha_modelo.cabecalho.iter().enumerate() {
            folha.write_string(0, j as u16, *nome)?;
        }
        let _ = colunas;

        for (i, linha) in folha_modelo.linhas.iter().enumerate() {
            let riga = u32::try_from(i + 1).map_err(|_| ErroXlsx::ForaDosLimites {
                folha: folha_modelo.nome,
                linhas: folha_modelo.linhas.len(),
                colunas: folha_modelo.cabecalho.len(),
            })?;
            for (j, celula) in linha.iter().enumerate() {
                let coluna = j as u16;
                match celula {
                    // `write_string`: nessun `f64`, e un testo che inizia con
                    // `=` resta testo (misurato da `TX-38`, proprieta' 2).
                    Celula::Texto(t) => {
                        folha.write_string(riga, coluna, t.as_str())?;
                    }
                    // La conversione vive dentro la funzione autorizzata: qui
                    // non compare nessun `f64` (EXP-53, inventario di §11.2).
                    Celula::NumeroDeComodo(d) => {
                        escrever_numero_de_comodo(folha, riga, coluna, *d, folha_modelo.nome)?;
                    }
                    // Una cella vuota **non si scrive**: un'assenza non e' uno
                    // zero e non e' una stringa vuota.
                    Celula::Vazia => {}
                }
            }
        }
    }
    Ok(livro.save_to_buffer()?)
}

/// Scrive la cella numerica **di comodo**, non autorevole.
///
/// # Perche' esiste una conversione a `f64` qui
///
/// Decisione **D26(a)** del 2026-10-23, riscontro `R5-01`: la libreria scelta
/// da D18 non espone un percorso numerico senza `f64`, e la colonna sommabile
/// di D20 richiede una cella numerica. L'eccezione e' circoscritta a questa
/// funzione (`EXP-53`), e le sue cinque condizioni sono:
///
/// 1. la colonna **testuale** autorevole resta `Decimal::to_string()` esatto;
/// 2. nessun `f64` nei calcoli, nell'archivio, nelle impronte o nel testo
///    autorevole;
/// 3. eleggibilita' a 15 cifre significative, assenza dichiarata con stato e
///    motivo, **mai** uno zero sostitutivo;
/// 4. il valore riletto si confronta nel dominio ammesso; la **scala** non e'
///    promessa in questa cella;
/// 5. nessuna correzione manuale di XML o ZIP per aggirare l'API.
///
/// Il chiamante ha gia' deciso l'eleggibilita': questa funzione non la valuta
/// e non inventa un valore quando la cella non va scritta.
///
/// Prende un [`Decimal`] e **non** un `f64`: la conversione deve stare qui
/// dentro, non nel chiamante, altrimenti `EXP-53` sarebbe violato dalla riga
/// che converte. L'inventario di §11.2 trova quindi `f64` in questa funzione e
/// in nessun'altra parte dei tre moduli, escluse le fixture dei test.
pub(crate) fn escrever_numero_de_comodo(
    folha: &mut Worksheet,
    linha: u32,
    coluna: u16,
    valor: Decimal,
    nome_da_folha: &'static str,
) -> Result<(), ErroXlsx> {
    let Some(numero) = valor.to_f64() else {
        return Err(ErroXlsx::NumeroNaoConvertivel {
            folha: nome_da_folha,
            linha: linha as usize + 1,
            coluna: coluna as usize,
            valor: valor.to_string(),
        });
    };
    folha.write_number(linha, coluna, numero)?;
    Ok(())
}

#[cfg(test)]
mod tx38 {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use std::path::PathBuf;

    /// I testi ostili della prova, in un posto solo: il confronto del lettore
    /// indipendente usa gli stessi valori.
    pub(crate) const TEXTOS: &[(&str, &str)] = &[
        ("formula_igual", "=1+1 SUCO \"MARCA\" A\u{00E7}ucar"),
        ("mais", "+55 11"),
        ("menos", "-7"),
        ("arroba", "@utente"),
        ("zeros", "00012345678901"),
        ("quebra", "linha1\nlinha2"),
        ("tab", "a\tb"),
        ("nfc", "A\u{00E7}ucar"),
        ("nfd", "Ac\u{0327}ucar"),
        ("escape_literal", "_x0041_"),
        ("escape_protegido", "_x005F_x0041_"),
        ("monetario_texto", "10.0000"),
        ("monetario_escala_zero", "0.00"),
        ("monetario_negativo", "-7.50"),
        ("monetario_longo", "79228162514264337593543950335"),
    ];

    /// Gli importi scritti **come numero**, con il testo autorevole accanto.
    /// Il testo e' la verita'; il numero e' la cella di comodo di D20.
    // I testi autorevoli e il decimale che la cella di comodo riceve: anche
    // la fixture passa per `Decimal`, non per `f64` (EXP-53).
    pub(crate) const NUMEROS: &[&str] = &["10.0000", "0.00", "-7.50", "0.01", "12345678901234.56"];

    fn pacote_minimo() -> Result<Vec<u8>, ErroXlsx> {
        let mut livro = Workbook::new();
        let folha = livro.add_worksheet();
        folha.set_name("Prova")?;

        // Riga 0: intestazioni. Righe 1..: un testo ostile per riga.
        folha.write_string(0, 0, "caso")?;
        folha.write_string(0, 1, "texto")?;
        for (i, (caso, texto)) in TEXTOS.iter().enumerate() {
            let linha = u32::try_from(i).expect("poche righe") + 1;
            folha.write_string(linha, 0, *caso)?;
            folha.write_string(linha, 1, *texto)?;
        }

        // Secondo foglio: testo autorevole e cella numerica di comodo.
        let numeros = livro.add_worksheet();
        numeros.set_name("Numeros")?;
        numeros.write_string(0, 0, "valor")?;
        numeros.write_string(0, 1, "valor_numero")?;
        for (i, texto) in NUMEROS.iter().enumerate() {
            let linha = u32::try_from(i).expect("poche righe") + 1;
            numeros.write_string(linha, 0, *texto)?;
            let valor = Decimal::from_str_exact(texto).expect("decimale esatto");
            escrever_numero_de_comodo(numeros, linha, 1, valor, "Numeros")?;
        }

        Ok(livro.save_to_buffer()?)
    }

    /// Dove la prova deposita il workbook per il lettore indipendente.
    /// Fuori dal repository: area temporanea del sistema.
    pub(crate) fn caminho_da_prova() -> PathBuf {
        std::env::temp_dir()
            .join("fantozzi_3c41")
            .join("tx38_minimo.xlsx")
    }

    /// TX-38, proprieta' 1: il pacchetto si produce **in memoria**.
    ///
    /// La libreria scrive con un `Cursor` su un `Vec<u8>`
    /// (`workbook.rs:1402`), quindi nessun file nasce dalla scrittura. Il test
    /// lo verifica anche dall'esterno: la directory di lavoro non cambia.
    #[test]
    fn tx38_pacote_em_memoria_sem_tocar_o_disco() {
        let antes = std::fs::read_dir(".").unwrap().count();
        let bytes = pacote_minimo().expect("il pacchetto si costruisce");
        let depois = std::fs::read_dir(".").unwrap().count();

        assert_eq!(
            antes, depois,
            "nessun file creato nella directory di lavoro"
        );
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[..2], b"PK", "e' un contenitore ZIP");
        // Con `zip/deflate` il pacchetto e' compresso: non e' `stored` come la
        // sonda §5.1 della specifica, e il rapporto lo dichiara.
        assert!(bytes.len() > 1000, "un pacchetto XLSX minimo ha piu' parti");
    }

    /// Deposita il workbook perche' la prova indipendente lo rilegga.
    ///
    /// Non e' un'asserzione sul contenuto: quello lo verifica
    /// `script/releitura_export.py`, perche' da Rust non si vede il **tipo**
    /// effettivo della cella. `cargo test` non dipende da Python: questo test
    /// scrive il file e finisce.
    #[test]
    fn tx38_deposita_o_workbook_para_o_leitor_independente() {
        let caminho = caminho_da_prova();
        std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
        let bytes = pacote_minimo().expect("il pacchetto si costruisce");
        std::fs::write(&caminho, &bytes).expect("scrittura della prova");
        assert!(caminho.exists());
        println!("TX38_WORKBOOK {}", caminho.display());
    }
}

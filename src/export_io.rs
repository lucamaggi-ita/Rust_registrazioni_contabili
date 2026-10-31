//! Scrittura del file derivato (fase 3C.4.1, §8 della specifica).
//!
//! Attua il protocollo con le decisioni **D24** (lucchetto cooperativo) e
//! **D25(b)** (`Recusar` con controllo di esistenza, senza rivendicazione del
//! nome finale), più l'esclusione del sorgente di `EXP-42`/`EXP-48`.
//!
//! # I fatti su cui si appoggia
//!
//! **[RIUSATO]** da §3.6 della specifica di persistenza, misurati su Windows:
//! `fs::rename` sostituisce una destinazione esistente; `create_new` ripetuto
//! fallisce con `AlreadyExists` ed è il lucchetto cooperativo; `sync_all` su
//! un handle di **sola lettura** è negato, quindi la copia del precedente va
//! aperta in scrittura prima di sincronizzarla.
//!
//! # Che cosa questo modulo non fa
//!
//! Non costruisce il pacchetto, non sceglie percorsi, non legge l'orologio.
//! Destinazione, etichetta, politica e metadati del lucchetto arrivano dal
//! chiamante. Non apre l'archivio: `EXP-24`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::arquivo_io::MetadadosLucchetto;
use crate::impressao;

/// Che fare se la destinazione esiste già (`EXP-23`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeExistir {
    /// Rifiuta. **D25(b)**: controllo di esistenza, non rivendicazione del
    /// nome finale, quindi nessuna destinazione di zero byte può restare.
    Recusar,
    /// Sostituisce, conservando il precedente al passo 5.
    Substituir,
}

/// L'archivio da proteggere: la destinazione non può coincidere con il suo
/// file né cadere nella sua famiglia di nomi (`EXP-42`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FonteProtegida {
    pub diretorio: PathBuf,
    pub nome: String,
}

/// Esito della scrittura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escrita {
    pub caminho: PathBuf,
    pub bytes: usize,
    pub sha256: String,
    /// `Some` quando `Substituir` ha conservato un file precedente.
    pub anterior: Option<PathBuf>,
    /// `true` se il commit è avvenuto e il rilascio del lucchetto è fallito:
    /// la scrittura **è valida**.
    pub lucchetto_residuo: bool,
}

/// Errore dello strato IO dell'export.
#[derive(Debug)]
pub enum ErroExportIo {
    /// `EXP-42`: la destinazione è il file dell'archivio, o cade nella sua
    /// famiglia di nomi. Nessun file è stato toccato.
    DestinoCoincideComFonte {
        destino: PathBuf,
        nome_da_fonte: String,
        o_que: &'static str,
    },
    /// `Recusar` e la destinazione esiste.
    DestinoJaExistente { caminho: PathBuf },
    /// Il lucchetto è di qualcun altro.
    ExportOcupado {
        lucchetto: PathBuf,
        conteudo_declarado: String,
    },
    /// Un residuo di un tentativo precedente: non si tronca, si segnala.
    ResiduoPresente { caminho: PathBuf },
    /// Una copia conservata non si sovrascrive.
    AnteriorJaPresente { caminho: PathBuf },
    /// Il pacchetto è vuoto: non si scrive un file che nessun lettore apre.
    PacoteVazio,
    Io {
        operacao: &'static str,
        caminho: PathBuf,
        erro: std::io::Error,
    },
}

impl std::fmt::Display for ErroExportIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroExportIo::DestinoCoincideComFonte {
                destino,
                nome_da_fonte,
                o_que,
            } => write!(
                f,
                "destino {} coincide com {o_que} do arquivo autoritativo \
                 ({nome_da_fonte}): nenhum arquivo foi tocado",
                destino.display()
            ),
            ErroExportIo::DestinoJaExistente { caminho } => write!(
                f,
                "destino ja' existente: {}. Com Recusar nada e' sobrescrito",
                caminho.display()
            ),
            ErroExportIo::ExportOcupado {
                lucchetto,
                conteudo_declarado,
            } => write!(
                f,
                "export ocupado, lucchetto {}: {conteudo_declarado}",
                lucchetto.display()
            ),
            ErroExportIo::ResiduoPresente { caminho } => write!(
                f,
                "residuo presente: {}. Nao e' truncado",
                caminho.display()
            ),
            ErroExportIo::AnteriorJaPresente { caminho } => write!(
                f,
                "copia anterior ja' presente: {}. Nenhuma copia conservada e' \
                 sobrescrita",
                caminho.display()
            ),
            ErroExportIo::PacoteVazio => {
                write!(f, "pacote vazio: nada a publicar")
            }
            ErroExportIo::Io {
                operacao,
                caminho,
                erro,
            } => write!(
                f,
                "falha de io em {operacao} sobre {}: {erro}",
                caminho.display()
            ),
        }
    }
}

impl std::error::Error for ErroExportIo {}

fn io(operacao: &'static str, caminho: &Path, erro: std::io::Error) -> ErroExportIo {
    ErroExportIo::Io {
        operacao,
        caminho: caminho.to_path_buf(),
        erro,
    }
}

// ---------------------------------------------------------------------------
// EXP-48: la chiave di confronto dei nomi
// ---------------------------------------------------------------------------

/// Chiave di confronto di un nome di file.
///
/// `EXP-48`, residuo `R3C4-03`: la canonicalizzazione **non** riconduce il
/// nome di un file che non esiste ancora, perché quel nome arriva dal
/// chiamante. Il confronto lessicale lascerebbe passare `ARQUIVO.JSON.LOCK`:
/// misurato in §5.5 della specifica, e la scrittura finirebbe in
/// `arquivo.json.lock`.
///
/// La chiave non riscrive il nome originale: serve solo al confronto, e ciò
/// che si memorizza e si stampa è il nome come il chiamante l'ha dato.
///
/// Il confronto sbaglia per **eccesso**: NTFS può marcare una directory come
/// sensibile alle maiuscole, e rifiutare una destinazione che non collide è un
/// fastidio, mentre lasciarne passare una che collide è una perdita.
fn chave_de_nome(nome: &str) -> String {
    nome.to_lowercase()
}

/// Directory in cui il destino verrà scritto.
///
/// `R3C41-03`: `Path::new("relativo.xlsx").parent()` è `Some("")`, **non**
/// `None`. Un `unwrap_or` non interviene su un parent vuoto, e
/// `canonicalize("")` falliva con `NotFound`: un nome semplice, che è un
/// percorso legittimo, veniva rifiutato per un errore di IO inventato.
///
/// Un parent vuoto significa «la directory corrente», e qui si dice così.
/// Nessun vincolo nuovo di soli percorsi assoluti: il chiamante resta libero.
fn diretorio_do_destino(destino: &Path) -> &Path {
    match destino.parent() {
        Some(pai) if !pai.as_os_str().is_empty() => pai,
        _ => Path::new("."),
    }
}

/// `EXP-42`: rifiuta prima di toccare qualunque file.
///
/// Due confronti, e servono entrambi.
///
/// 1. **Per identità**, se il destino esiste già: `fs::canonicalize` risolve
///    l'alias 8.3 al nome lungo, quindi `ARQUIV~1.JSO` e
///    `arquivo_autoritativo.json` si riconoscono come lo stesso file.
///    `R3C41-01`: il solo confronto lessicale li vedeva diversi, e con
///    `Substituir` l'archivio veniva sostituito dall'export, con esito
///    dichiarato riuscito.
/// 2. **Per chiave lessicale**, per i nomi che **non esistono ancora**: la
///    canonicalizzazione non può risolvere un nome che il chiamante ha appena
///    inventato, e `EXP-48` resta la difesa su quel caso.
///
/// Nessuna promessa sugli hard link: un hard link è un nome proprio, e
/// `canonicalize` lo risolve a sé stesso. Resta dichiarato fuori dalla
/// garanzia, come già scritto in §9.6 della specifica.
fn verificar_exclusao(destino: &Path, fonte: &FonteProtegida) -> Result<(), ErroExportIo> {
    let erro = |o_que: &'static str| ErroExportIo::DestinoCoincideComFonte {
        destino: destino.to_path_buf(),
        nome_da_fonte: fonte.nome.clone(),
        o_que,
    };
    let chave_fonte = chave_de_nome(&fonte.nome);
    let prefixo_da_familia = format!("{chave_fonte}.");

    let fonte_canonica = match fs::canonicalize(&fonte.diretorio) {
        Ok(c) => c,
        // Se la directory dell'archivio non esiste, non c'e' nulla da
        // proteggere: non si inventa una collisione.
        Err(_) => return Ok(()),
    };

    // 1. Confronto per identita', sui soli percorsi che esistono.
    if let Ok(destino_canonico) = fs::canonicalize(destino) {
        if let Ok(arquivo_canonico) = fs::canonicalize(fonte.diretorio.join(&fonte.nome))
            && destino_canonico == arquivo_canonico
        {
            return Err(erro("o proprio arquivo"));
        }
        // La famiglia si confronta sul nome **risolto**: e' quello che il
        // filesystem considera il nome del file, non l'alias con cui e' stato
        // chiesto.
        let nella_fonte = destino_canonico.parent() == Some(fonte_canonica.as_path());
        if nella_fonte && let Some(nome) = destino_canonico.file_name().and_then(|n| n.to_str()) {
            let chave = chave_de_nome(nome);
            if chave == chave_fonte {
                return Err(erro("o proprio arquivo"));
            }
            if chave.starts_with(&prefixo_da_familia) {
                return Err(erro("a familia de nomes"));
            }
        }
    }

    // 2. Confronto lessicale, per i nomi che non esistono ancora.
    let pai_destino = diretorio_do_destino(destino);
    let pai_canonico = fs::canonicalize(pai_destino)
        .map_err(|e| io("canonicalizar o diretorio do destino", pai_destino, e))?;
    if pai_canonico != fonte_canonica {
        return Ok(());
    }
    let Some(nome_destino) = destino.file_name().and_then(|n| n.to_str()) else {
        return Ok(());
    };
    let chave = chave_de_nome(nome_destino);
    if chave == chave_fonte {
        return Err(erro("o proprio arquivo"));
    }
    // La famiglia si riconosce per **prefisso**: un nome futuro della stessa
    // famiglia resta protetto senza che nessuno si ricordi di aggiungerlo.
    if chave.starts_with(&prefixo_da_familia) {
        return Err(erro("a familia de nomes"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Lucchetto cooperativo (D24, EXP-44)
// ---------------------------------------------------------------------------

/// Contenuto del lucchetto dell'export.
///
/// `arquivo_io::MetadadosLucchetto::texto` è privato a quel modulo, che questo
/// lotto non può modificare: qui si scrive una forma propria, dichiarata, per
/// un file che ha uno scopo diverso. Il tipo dei metadati è **riusato**.
fn texto_do_lucchetto(m: &MetadadosLucchetto) -> String {
    format!(
        "export\noperador={}\ninstante={}\noffset_segundos={}\nmotivo={}\n",
        m.operador,
        m.instante.naive_local().format("%Y-%m-%dT%H:%M:%S%.9f"),
        m.instante.offset().local_minus_utc(),
        m.motivo
    )
}

struct Trava {
    caminho: PathBuf,
    liberada: bool,
}

impl Trava {
    fn tomar(destino: &Path, m: &MetadadosLucchetto) -> Result<Self, ErroExportIo> {
        let caminho = caminho_derivado(destino, "export.lock");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&caminho)
        {
            Ok(mut arquivo) => {
                arquivo
                    .write_all(texto_do_lucchetto(m).as_bytes())
                    .map_err(|e| io("escrever lucchetto", &caminho, e))?;
                let _ = arquivo.sync_all();
                Ok(Trava {
                    caminho,
                    liberada: false,
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let conteudo =
                    fs::read_to_string(&caminho).unwrap_or_else(|_| "(ilegivel)".to_string());
                Err(ErroExportIo::ExportOcupado {
                    lucchetto: caminho,
                    conteudo_declarado: conteudo,
                })
            }
            Err(e) => Err(io("criar lucchetto", &caminho, e)),
        }
    }

    /// Rilascio esplicito: l'esito è un dato dell'esito, non un dettaglio.
    fn liberar(mut self) -> bool {
        self.liberada = true;
        fs::remove_file(&self.caminho).is_ok()
    }
}

impl Drop for Trava {
    fn drop(&mut self) {
        if !self.liberada {
            // Percorso d'errore: rilascio di cortesia. Dopo il commit il
            // rilascio è esplicito e il suo esito entra in `Escrita`.
            let _ = fs::remove_file(&self.caminho);
        }
    }
}

/// `<destino>.<suffisso>`, accanto alla destinazione.
fn caminho_derivado(destino: &Path, suffixo: &str) -> PathBuf {
    let nome = destino
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    destino.with_file_name(format!("{nome}.{suffixo}"))
}

/// Forza su disco un file già scritto.
///
/// L'handle va aperto in **scrittura**: su Windows `sync_all` su un handle di
/// sola lettura è negato con `PermissionDenied`. **[RIUSATO]** dal lotto
/// 3C.3.b.1, dove la misura è costata un difetto.
fn sincronizar(operacao: &'static str, caminho: &Path) -> Result<(), ErroExportIo> {
    OpenOptions::new()
        .write(true)
        .open(caminho)
        .map_err(|e| io(operacao, caminho, e))?
        .sync_all()
        .map_err(|e| io(operacao, caminho, e))
}

// ---------------------------------------------------------------------------
// Il protocollo
// ---------------------------------------------------------------------------

/// Scrive il pacchetto nella destinazione, con un solo punto di commit.
///
/// Passi 0–8 di §8 della specifica. Il passo 3 **controlla** l'esistenza e non
/// la crea (D25(b)): nessuna destinazione di zero byte può restare.
///
/// `etiqueta` è del chiamante (`EXP-45`): questo modulo non legge l'orologio e
/// non inventa un nome per il temporaneo.
pub fn escrever(
    destino: &Path,
    pacote: &[u8],
    se_existir: SeExistir,
    fonte: &FonteProtegida,
    etiqueta: &str,
    lucchetto: &MetadadosLucchetto,
) -> Result<Escrita, ErroExportIo> {
    // Passo 0: esclusione del sorgente, prima di qualunque file.
    verificar_exclusao(destino, fonte)?;

    // Passo 1: il pacchetto deve esistere.
    if pacote.is_empty() {
        return Err(ErroExportIo::PacoteVazio);
    }

    // Passo 2: lucchetto cooperativo.
    let trava = Trava::tomar(destino, lucchetto)?;

    // Passo 3: con `Recusar`, si **controlla** l'esistenza.
    let existia = destino.exists();
    if existia && se_existir == SeExistir::Recusar {
        return Err(ErroExportIo::DestinoJaExistente {
            caminho: destino.to_path_buf(),
        });
    }

    // Passo 4: temporaneo, senza troncare un residuo.
    let temporario = caminho_derivado(destino, &format!("novo.{etiqueta}"));
    if temporario.exists() {
        return Err(ErroExportIo::ResiduoPresente {
            caminho: temporario,
        });
    }
    let mut arquivo = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporario)
        .map_err(|e| io("criar temporario", &temporario, e))?;
    arquivo
        .write_all(pacote)
        .map_err(|e| io("escrever temporario", &temporario, e))?;
    arquivo
        .sync_all()
        .map_err(|e| io("sincronizar temporario", &temporario, e))?;
    drop(arquivo);

    // Passo 5: il precedente, mai sovrascritto.
    let anterior = if existia && se_existir == SeExistir::Substituir {
        let caminho = caminho_derivado(destino, &format!("anterior.{etiqueta}"));
        if caminho.exists() {
            return Err(ErroExportIo::AnteriorJaPresente { caminho });
        }
        fs::copy(destino, &caminho).map_err(|e| io("copiar anterior", &caminho, e))?;
        sincronizar("sincronizar anterior", &caminho)?;
        Some(caminho)
    } else {
        None
    };

    // Passo 6: PUNTO DI COMMIT.
    fs::rename(&temporario, destino).map_err(|e| io("publicar", destino, e))?;

    // Passo 7: rilascio. Un fallimento qui non annulla la scrittura.
    let liberado = trava.liberar();

    // Passo 8: esito.
    Ok(Escrita {
        caminho: destino.to_path_buf(),
        bytes: pacote.len(),
        sha256: impressao::sha256_hex(pacote),
        anterior,
        lucchetto_residuo: !liberado,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arquivo::testes_comuns as arq;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQUENCIA: AtomicU32 = AtomicU32::new(0);

    /// Directory isolata, una per chiamata, sotto la temporanea di sistema.
    /// Non tocca nessuna cartella del progetto.
    fn arena(nome: &str) -> PathBuf {
        let unica = format!(
            "{nome}_{}_{}",
            std::process::id(),
            SEQUENCIA.fetch_add(1, Ordering::SeqCst)
        );
        let diretorio = std::env::temp_dir().join("fantozzi_3c41_io").join(unica);
        if diretorio.exists() {
            fs::remove_dir_all(&diretorio).unwrap();
        }
        fs::create_dir_all(&diretorio).unwrap();
        diretorio
    }

    /// Un archivio finto accanto alla destinazione, da proteggere.
    fn fonte_com_arquivo(diretorio: &Path) -> FonteProtegida {
        let fonte = FonteProtegida {
            diretorio: diretorio.to_path_buf(),
            nome: "arquivo.json".to_string(),
        };
        fs::write(
            diretorio.join(&fonte.nome),
            b"{\"formato\":\"FANTOZZI-ARQUIVO/v1\"}",
        )
        .unwrap();
        fonte
    }

    fn lucchetto(motivo: &str) -> MetadadosLucchetto {
        MetadadosLucchetto {
            operador: "Jos\u{00E9}".to_string(),
            instante: arq::instante(9, -3 * 3600),
            motivo: motivo.to_string(),
        }
    }

    fn pacote() -> Vec<u8> {
        b"PK\x03\x04 pacchetto finto per il protocollo".to_vec()
    }

    fn impronta(caminho: &Path) -> String {
        impressao::sha256_hex(&fs::read(caminho).unwrap())
    }

    // =================================================================
    // TX-16: destinazione assente, Recusar, Substituir
    // =================================================================

    #[test]
    fn tx16_destino_ausente_recusar_e_substituir() {
        let dir = arena("tx16");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");

        // Destinazione assente: si crea.
        let escrita = escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("primo"),
        )
        .unwrap();
        assert_eq!(escrita.caminho, destino);
        assert_eq!(escrita.bytes, pacote().len());
        assert_eq!(escrita.sha256, impressao::sha256_hex(&pacote()));
        assert!(escrita.anterior.is_none());
        assert!(!escrita.lucchetto_residuo);
        assert_eq!(fs::read(&destino).unwrap(), pacote());
        assert!(!dir.join("export.xlsx.export.lock").exists());
        assert!(!dir.join("export.xlsx.novo.a").exists());

        // `Recusar` con destinazione esistente: rifiuta e non tocca niente.
        let antes = impronta(&destino);
        match escrever(
            &destino,
            b"altri byte",
            SeExistir::Recusar,
            &fonte,
            "b",
            &lucchetto("secondo"),
        )
        .unwrap_err()
        {
            ErroExportIo::DestinoJaExistente { caminho } => assert_eq!(caminho, destino),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(impronta(&destino), antes, "destinazione invariata");
        assert!(
            !dir.join("export.xlsx.export.lock").exists(),
            "lucchetto rilasciato"
        );

        // `Substituir`: sostituisce e conserva il precedente.
        let escrita = escrever(
            &destino,
            b"nuovo contenuto",
            SeExistir::Substituir,
            &fonte,
            "c",
            &lucchetto("terzo"),
        )
        .unwrap();
        assert_eq!(fs::read(&destino).unwrap(), b"nuovo contenuto");
        let anterior = escrita.anterior.expect("il precedente e' conservato");
        assert_eq!(anterior, dir.join("export.xlsx.anterior.c"));
        assert_eq!(impressao::sha256_hex(&fs::read(&anterior).unwrap()), antes);
    }

    #[test]
    fn tx16_substituir_non_sovrascrive_una_copia_conservata() {
        let dir = arena("tx16b");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");
        escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("p"),
        )
        .unwrap();

        let anterior = dir.join("export.xlsx.anterior.a");
        fs::write(&anterior, b"copia di un'altra esecuzione").unwrap();
        match escrever(
            &destino,
            b"nuovo",
            SeExistir::Substituir,
            &fonte,
            "a",
            &lucchetto("s"),
        )
        .unwrap_err()
        {
            ErroExportIo::AnteriorJaPresente { caminho } => assert_eq!(caminho, anterior),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read(&anterior).unwrap(),
            b"copia di un'altra esecuzione",
            "la copia conservata resta"
        );
        assert_eq!(
            fs::read(&destino).unwrap(),
            pacote(),
            "destinazione intatta"
        );
    }

    // =================================================================
    // TX-14, TX-15: errori prima del commit
    // =================================================================

    #[test]
    fn tx15_residuo_presente_e_nominato_e_non_cancellato() {
        let dir = arena("tx15");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");
        escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("p"),
        )
        .unwrap();
        let antes = impronta(&destino);

        let residuo = dir.join("export.xlsx.novo.b");
        fs::write(&residuo, b"residuo di un tentativo precedente").unwrap();
        match escrever(
            &destino,
            b"nuovo",
            SeExistir::Substituir,
            &fonte,
            "b",
            &lucchetto("r"),
        )
        .unwrap_err()
        {
            ErroExportIo::ResiduoPresente { caminho } => assert_eq!(caminho, residuo),
            outro => panic!("erro inesperado: {outro}"),
        }
        assert_eq!(
            fs::read(&residuo).unwrap(),
            b"residuo di un tentativo precedente",
            "non troncato ne' cancellato"
        );
        assert_eq!(impronta(&destino), antes, "destinazione invariata");
        assert!(!dir.join("export.xlsx.export.lock").exists());
    }

    #[test]
    fn tx14_pacote_vazio_e_recusado_prima_di_toccare_qualcosa() {
        let dir = arena("tx14");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");
        match escrever(
            &destino,
            &[],
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("v"),
        )
        .unwrap_err()
        {
            ErroExportIo::PacoteVazio => {}
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(!destino.exists(), "niente creato");
        assert!(!dir.join("export.xlsx.export.lock").exists());
    }

    /// `EXP-49` decaduto con D25(b): dopo un errore **nessuna** destinazione
    /// di zero byte resta, perche' il passo 3 controlla e non rivendica.
    #[test]
    fn tx34b_nessuna_destinazione_di_zero_byte_dopo_un_errore() {
        let dir = arena("tx34b");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");

        // Residuo del temporaneo: l'errore cade al passo 4.
        fs::write(dir.join("export.xlsx.novo.a"), b"residuo").unwrap();
        let esito = escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("r"),
        );
        assert!(esito.is_err());
        assert!(
            !destino.exists(),
            "con D25(b) la destinazione non viene rivendicata, quindi non resta vuota"
        );
        assert!(!dir.join("export.xlsx.export.lock").exists());
    }

    // =================================================================
    // TX-17: l'archivio non viene toccato
    // =================================================================

    #[test]
    fn tx17_o_arquivo_fonte_fica_identico() {
        let dir = arena("tx17");
        let fonte = fonte_com_arquivo(&dir);
        let caminho_fonte = dir.join(&fonte.nome);
        let antes = impronta(&caminho_fonte);

        let destino = dir.join("export.xlsx");
        escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("p"),
        )
        .unwrap();
        escrever(
            &destino,
            b"ancora",
            SeExistir::Substituir,
            &fonte,
            "b",
            &lucchetto("s"),
        )
        .unwrap();
        assert_eq!(impronta(&caminho_fonte), antes, "l'archivio e' identico");
    }

    // =================================================================
    // TX-33: esclusione del sorgente e della sua famiglia (EXP-42, EXP-48)
    // =================================================================

    #[test]
    fn tx33_destino_che_coincide_con_la_fonte_e_recusado() {
        let dir = arena("tx33");
        let fonte = fonte_com_arquivo(&dir);
        let caminho_fonte = dir.join(&fonte.nome);
        let antes = impronta(&caminho_fonte);

        // Il file stesso, e le varianti di maiuscole su un nome **esistente**.
        for nome in ["arquivo.json", "ARQUIVO.JSON", "Arquivo.Json"] {
            let destino = dir.join(nome);
            match escrever(
                &destino,
                &pacote(),
                SeExistir::Substituir,
                &fonte,
                "a",
                &lucchetto("x"),
            )
            .unwrap_err()
            {
                ErroExportIo::DestinoCoincideComFonte { o_que, .. } => {
                    assert_eq!(o_que, "o proprio arquivo", "{nome}")
                }
                outro => panic!("{nome}: erro inesperado {outro}"),
            }
        }

        // La **famiglia**, con destinazioni inizialmente **assenti** e
        // maiuscole diverse: e' il residuo `R3C4-03`.
        for nome in [
            "arquivo.json.lock",
            "ARQUIVO.JSON.LOCK",
            "arquivo.json.GER1",
            "ARQUIVO.JSON.novo.2",
            "arquivo.json.CORRUPTO7",
            "ARQUIVO.JSON.recuperado.1",
            "arquivo.json.suffisso-futuro",
        ] {
            let destino = dir.join(nome);
            assert!(!destino.exists(), "{nome} non esiste prima della prova");
            match escrever(
                &destino,
                &pacote(),
                SeExistir::Recusar,
                &fonte,
                "a",
                &lucchetto("x"),
            )
            .unwrap_err()
            {
                ErroExportIo::DestinoCoincideComFonte {
                    o_que, destino: d, ..
                } => {
                    assert_eq!(o_que, "a familia de nomes", "{nome}");
                    // Il messaggio porta il nome **come l'ha dato il
                    // chiamante**, non la chiave di confronto.
                    assert_eq!(d.file_name().unwrap().to_string_lossy(), nome);
                }
                outro => panic!("{nome}: erro inesperado {outro}"),
            }
            assert!(!destino.exists(), "{nome} non e' stato creato");
        }

        // Nessun file toccato: ne' il sorgente, ne' il lucchetto, ne' temporanei.
        assert_eq!(impronta(&caminho_fonte), antes);
        let entradas: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entradas, vec!["arquivo.json".to_string()], "{entradas:?}");

        // Un nome che **non** e' della famiglia passa.
        let destino = dir.join("arquivo.xlsx");
        escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("ok"),
        )
        .unwrap();
        assert!(destino.exists());
    }

    #[test]
    fn tx33_percorsi_equivalenti_sono_ricondotti() {
        let dir = arena("tx33b");
        let fonte = fonte_com_arquivo(&dir);
        // `dir/sub/../arquivo.json` e' lo stesso file.
        fs::create_dir_all(dir.join("sub")).unwrap();
        let destino = dir.join("sub").join("..").join("arquivo.json");
        match escrever(
            &destino,
            &pacote(),
            SeExistir::Substituir,
            &fonte,
            "a",
            &lucchetto("x"),
        )
        .unwrap_err()
        {
            ErroExportIo::DestinoCoincideComFonte { .. } => {}
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    // =================================================================
    // TX-34: concorrenza fra esecuzioni cooperanti (D24)
    // =================================================================

    #[test]
    fn tx34_lucchetto_tenuto_da_un_altro_da_export_ocupado() {
        let dir = arena("tx34");
        let fonte = fonte_com_arquivo(&dir);
        let destino = dir.join("export.xlsx");

        let metadados = lucchetto("manutencao in corso");
        let trava = Trava::tomar(&destino, &metadados).unwrap();
        assert!(dir.join("export.xlsx.export.lock").exists());

        // Chi arriva durante riceve `ExportOcupado`, non `DestinoJaExistente`.
        match escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("secondo"),
        )
        .unwrap_err()
        {
            ErroExportIo::ExportOcupado {
                conteudo_declarado, ..
            } => {
                assert!(conteudo_declarado.contains("manutencao in corso"));
                assert!(conteudo_declarado.contains("Jos\u{00E9}"));
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(!destino.exists(), "niente pubblicato");

        // Rilasciato il lucchetto, il primo passa.
        assert!(trava.liberar());
        escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "a",
            &lucchetto("ok"),
        )
        .unwrap();

        // E chi arriva **dopo** il rilascio, con `Recusar`, trova la
        // destinazione esistente: l'altro esito dei due intrecci.
        match escrever(
            &destino,
            &pacote(),
            SeExistir::Recusar,
            &fonte,
            "b",
            &lucchetto("terzo"),
        )
        .unwrap_err()
        {
            ErroExportIo::DestinoJaExistente { .. } => {}
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn chave_de_nome_non_riscrive_il_nome_originale() {
        // La chiave serve al confronto; il nome resta quello del chiamante.
        assert_eq!(chave_de_nome("ARQUIVO.JSON"), "arquivo.json");
        assert_eq!(chave_de_nome("arquivo.json"), "arquivo.json");
        assert_ne!(chave_de_nome("arquivo.json"), "ARQUIVO.JSON");
    }

    // =================================================================
    // Regressioni dell'audit del 2026-10-30
    // =================================================================

    /// Nome breve 8.3 di un file, chiesto a `cmd`.
    ///
    /// Rust non espone `GetShortPathNameW`, e questo lotto non autorizza una
    /// dipendenza per ottenerlo. `for %I in (...) do @echo %~sI` e' la via
    /// documentata di `cmd`. Se il volume non genera alias, il nome tornato
    /// coincide con quello lungo e il test lo dichiara **NON_ESEGUITO**
    /// invece di simulare un successo.
    #[cfg(windows)]
    fn nome_breve(caminho: &Path) -> Option<PathBuf> {
        use std::os::windows::process::CommandExt;
        // `raw_arg`: la citazione automatica di `Command` non è quella che
        // `cmd` si aspetta, e il primo tentativo ha stampato un percorso con
        // le virgolette dentro. Qui la riga di comando si scrive per intero.
        let saida = std::process::Command::new("cmd")
            .raw_arg(format!(
                "/C for %I in (\"{}\") do @echo %~sI",
                caminho.display()
            ))
            .output()
            .ok()?;
        let texto = String::from_utf8_lossy(&saida.stdout).trim().to_string();
        if texto.is_empty() {
            return None;
        }
        let breve = PathBuf::from(texto);
        if breve == caminho { None } else { Some(breve) }
    }

    /// Fuori da Windows non esistono alias 8.3, e il caso si dichiara
    /// `NON_ESEGUITO` invece di essere simulato.
    #[cfg(not(windows))]
    fn nome_breve(_caminho: &Path) -> Option<PathBuf> {
        None
    }

    /// `R3C41-01`: il confronto lessicale del nome lasciava passare l'alias
    /// 8.3 della fonte, che identifica **lo stesso file** con un nome
    /// diverso. Con `Substituir` l'archivio veniva sostituito da un export
    /// dichiarato riuscito.
    #[test]
    fn r1_alias_8_3_da_fonte_e_da_sua_familia_e_recusado() {
        let dir = arena("r1_alias");
        // Un nome lungo, perche' un nome di otto caratteri non genera alias.
        let fonte = FonteProtegida {
            diretorio: dir.clone(),
            nome: "arquivo_autoritativo_longo.json".to_string(),
        };
        let caminho_fonte = dir.join(&fonte.nome);
        fs::write(&caminho_fonte, b"ORIGINAL_ARCHIVE").unwrap();
        // E un membro della famiglia, anch'esso con nome lungo.
        let geracao = dir.join(format!("{}.geracao_conservada.1", fonte.nome));
        fs::write(&geracao, b"GERACAO_CONSERVADA").unwrap();

        let (Some(alias_fonte), Some(alias_geracao)) =
            (nome_breve(&caminho_fonte), nome_breve(&geracao))
        else {
            println!(
                "NON_ESEGUITO: questo volume non genera alias 8.3, quindi il \
                 caso di R3C41-01 non e' riproducibile qui. Non viene \
                 simulato."
            );
            return;
        };
        println!("alias della fonte:    {}", alias_fonte.display());
        println!("alias della famiglia: {}", alias_geracao.display());

        for (alias, originale, o_que) in [
            (&alias_fonte, &caminho_fonte, "o proprio arquivo"),
            (&geracao, &geracao, "a familia de nomes"),
            (&alias_geracao, &geracao, "a familia de nomes"),
        ] {
            let antes = fs::read(originale).unwrap();
            let erro = escrever(
                alias,
                &pacote(),
                SeExistir::Substituir,
                &fonte,
                "a",
                &lucchetto("x"),
            )
            .unwrap_err();
            match erro {
                ErroExportIo::DestinoCoincideComFonte { o_que: lido, .. } => {
                    assert_eq!(lido, o_que, "{}", alias.display())
                }
                outro => panic!("{}: erro inesperado {outro}", alias.display()),
            }
            assert_eq!(
                fs::read(originale).unwrap(),
                antes,
                "{} invariato byte per byte",
                originale.display()
            );
        }

        // Nessuna mutazione: niente lucchetto, niente temporaneo, niente copia.
        let mut entradas: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        entradas.sort();
        assert_eq!(
            entradas,
            vec![
                "arquivo_autoritativo_longo.json".to_string(),
                "arquivo_autoritativo_longo.json.geracao_conservada.1".to_string(),
            ],
            "{entradas:?}"
        );

        // Un nome breve di un file **estraneo** alla famiglia resta ammesso.
        let estraneo = dir.join("relatorio_para_o_contador.xlsx");
        fs::write(&estraneo, b"vecchio").unwrap();
        if let Some(alias_estraneo) = nome_breve(&estraneo) {
            escrever(
                &alias_estraneo,
                &pacote(),
                SeExistir::Substituir,
                &fonte,
                "b",
                &lucchetto("ok"),
            )
            .expect("un file estraneo non e' protetto");
        }
    }

    /// `R3C41-03`: `Path::new("relativo.xlsx").parent()` e' `Some("")`, non
    /// `None`, quindi il fallback su `.` non interveniva e la
    /// canonicalizzazione falliva. Un nome semplice e' un percorso legittimo.
    #[test]
    fn r3_percorso_relativo_semplice_e_utilizzabile() {
        let dir = arena("r3_relativo");
        let fonte = fonte_com_arquivo(&dir);
        // Il test non cambia la directory di lavoro del processo: userebbe
        // uno stato globale condiviso con gli altri test. Verifica invece il
        // caso direttamente sul predicato dei percorsi.
        assert_eq!(
            Path::new("relativo.xlsx").parent(),
            Some(Path::new("")),
            "e' il fatto da cui nasce il riscontro"
        );
        // Con il parent vuoto trattato come directory corrente, un nome
        // semplice non e' nella famiglia della fonte e passa il controllo.
        verificar_exclusao(Path::new("relativo.xlsx"), &fonte)
            .expect("un nome semplice si canonicalizza sulla directory corrente");
        verificar_exclusao(Path::new("./esplicito.xlsx"), &fonte)
            .expect("e la forma esplicita si comporta allo stesso modo");

        // E la protezione resta: una fonte raggiunta da un percorso relativo
        // viene comunque riconosciuta.
        let fonte_relativa = FonteProtegida {
            diretorio: PathBuf::from("."),
            nome: "relativo.xlsx".to_string(),
        };
        match verificar_exclusao(Path::new("relativo.xlsx"), &fonte_relativa).unwrap_err() {
            ErroExportIo::DestinoCoincideComFonte { .. } => {}
            outro => panic!("erro inesperado: {outro}"),
        }
    }
}

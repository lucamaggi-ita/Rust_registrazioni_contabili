//! Modello dell'export derivato (fase 3C.4.1).
//!
//! Attua i contratti `EXP-01…EXP-58` della specifica di fase 3C.4, con le
//! decisioni D18–D26, `EXP-29` e la divergenza nota del lettore approvate il
//! 2026-10-23.
//!
//! # Che cosa produce, e che cosa non e'
//!
//! Un `ModeloExport`: tredici fogli di celle, nell'ordine canonico che
//! `prelivro` e `revisao` hanno gia' fissato. E' un **derivato** (D7(a)):
//! l'archivio resta la fonte autorevole, e nulla di cio' che accade al file
//! torna indietro. Il foglio `limites` lo dichiara nel file stesso.
//!
//! # Purezza
//!
//! Questo modulo non apre file, non legge l'orologio, non carica il piano dei
//! conti e non conosce percorsi: tutto arriva come parametro, `gerado_em`
//! compreso. L'IO sta in [`crate::export_io`], il pacchetto in
//! [`crate::xlsx`].
//!
//! # Nessun `f64` qui
//!
//! `EXP-07` ed `EXP-53`: l'unica conversione a `f64` del lotto vive in
//! `xlsx.rs`, nella funzione che scrive la cella numerica di comodo. Qui una
//! cella numerica porta un [`Decimal`], e la conversione avviene al confine
//! della scrittura.

use std::fmt;

use chrono::{DateTime, FixedOffset, NaiveDate};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::alocacao::{Componente, Reconciliacao};
use crate::arquivo::{Arquivo, IdentidadePlano};
use crate::modelos::{Dado, StatusDado};
use crate::nfe::Aviso;
use crate::prelivro::{DocumentoPreLivro, Exclusao, Indicadores, LinhaPreLivro, PreLivro};
use crate::revisao::{
    DecisaoAplicada, EventoRevisao, LinhaRevisada, OperacaoRevisao, VistaRevisada,
};
use crate::{impressao, revisao};

/// Identificatore del formato. Entra nel foglio `proveniencia`.
pub const FORMATO: &str = "FANTOZZI-EXPORT/v1";
/// Versione dello schema. `EXP-41`: identificativo, quindi **testo**.
pub const VERSAO_SCHEMA: u32 = 1;

/// Limiti del formato, controllati (`EXP-35`).
pub const LIMITE_CARACTERES_CELULA: usize = 32_767;
pub const LIMITE_LINHAS_FOLHA: usize = 1_048_576;
pub const LIMITE_COLUNAS_FOLHA: usize = 16_384;

/// Cifre significative oltre le quali la cella numerica di comodo non si
/// scrive (`EXP-38`): e' la precisione documentata di Excel, e la stessa
/// frontiera che `f64` garantisce in round-trip.
pub const CIFRAS_SIGNIFICATIVAS_MAXIMAS: u32 = 15;

/// Separatore delle celle concatenate di comodo (§6.3.1).
const SEPARADOR_LISTA: &str = " | ";

// ---------------------------------------------------------------------------
// Celle e fogli
// ---------------------------------------------------------------------------

/// Una cella del modello.
///
/// `NumeroDeComodo` porta un [`Decimal`], **non** un `f64`: la conversione e'
/// l'eccezione di D26(a) e vive soltanto in [`crate::xlsx`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Celula {
    Texto(String),
    NumeroDeComodo(Decimal),
    /// Cella vuota: un'assenza, **non** uno zero.
    Vazia,
}

impl Celula {
    fn texto(valor: impl Into<String>) -> Self {
        Celula::Texto(valor.into())
    }

    /// `SIM`/`NAO`: un booleano resta leggibile e non diventa `TRUE`/`1`.
    fn booleano(valor: bool) -> Self {
        Celula::Texto(if valor { "SIM" } else { "NAO" }.to_string())
    }

    /// Data come testo ISO: una cella data di Excel non conserva il dominio
    /// che serve (§7.3).
    fn data(valor: NaiveDate) -> Self {
        Celula::Texto(valor.format("%Y-%m-%d").to_string())
    }

    fn data_opcional(valor: Option<NaiveDate>) -> Self {
        match valor {
            Some(d) => Celula::data(d),
            None => Celula::Vazia,
        }
    }

    /// Un `Decimal` autorevole: testo esatto, scala compresa.
    fn decimal(valor: Decimal) -> Self {
        Celula::Texto(valor.to_string())
    }

    fn contagem(valor: usize) -> Self {
        Celula::NumeroDeComodo(Decimal::from(valor))
    }

    fn lista(valores: &[String]) -> Self {
        if valores.is_empty() {
            Celula::Vazia
        } else {
            Celula::Texto(valores.join(SEPARADOR_LISTA))
        }
    }

    fn texto_opcional(valor: Option<&str>) -> Self {
        match valor {
            Some(v) => Celula::texto(v),
            None => Celula::Vazia,
        }
    }
}

/// Un foglio: nome, intestazioni fisse e righe in ordine canonico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folha {
    pub nome: &'static str,
    pub cabecalho: &'static [&'static str],
    pub linhas: Vec<Vec<Celula>>,
}

impl Folha {
    fn nova(nome: &'static str, cabecalho: &'static [&'static str]) -> Self {
        Folha {
            nome,
            cabecalho,
            linhas: Vec::new(),
        }
    }
}

/// Il modello dell'export: tredici fogli, nell'ordine dello schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeloExport {
    pub folhas: Vec<Folha>,
}

impl ModeloExport {
    /// Un foglio per nome, per i test e per i chiamanti che ispezionano.
    pub fn folha(&self, nome: &str) -> Option<&Folha> {
        self.folhas.iter().find(|f| f.nome == nome)
    }
}

// ---------------------------------------------------------------------------
// Provenienza
// ---------------------------------------------------------------------------

/// Tutto cio' che il foglio `proveniencia` dichiara, dal chiamante.
///
/// `EXP-32`: **due** identita' del piano, quella archiviata nell'esecuzione e
/// quella del piano effettivamente usato. `plano_coincide` confronta
/// **soltanto** versione e percorso: un piano puo' cambiare conservandoli, e
/// il foglio `limites` lo dichiara.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proveniencia {
    pub arquivo_caminho: String,
    pub arquivo_revisao: u64,
    pub arquivo_sha256: String,
    pub plano_arquivado: IdentidadePlano,
    pub plano_usado_versao: String,
    pub plano_usado_caminho: String,
    pub gerado_em: DateTime<FixedOffset>,
    /// Divergenze del **solo lotto esportato** (`EXP-30`).
    pub divergencias: Vec<crate::arquivo::Divergencia>,
    pub eventos_no_arquivo: usize,
}

impl Proveniencia {
    /// `SIM`, `PARCIALE` o `NAO`, sui due soli metadati confrontabili.
    fn coincidencia_do_plano(&self) -> &'static str {
        let versao = self.plano_arquivado.versao == self.plano_usado_versao;
        let caminho = self.plano_arquivado.caminho_declarado.as_deref()
            == Some(self.plano_usado_caminho.as_str());
        match (versao, caminho) {
            (true, true) => "SIM",
            (true, false) => "PARCIALE",
            _ => "NAO",
        }
    }
}

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore del modello. Nessun caso produce un export parziale.
#[derive(Debug)]
pub enum ErroExport {
    /// `EXP-33`: un carattere che XML 1.0 non puo' rappresentare.
    TextoNaoRepresentavel {
        folha: &'static str,
        linha: usize,
        coluna: &'static str,
        codepoint: u32,
        posicao: usize,
    },
    /// `EXP-34`: oltre il limite di cella. **Nessun troncamento**.
    TextoTroppoLungo {
        folha: &'static str,
        linha: usize,
        coluna: &'static str,
        caracteres: usize,
        limite: usize,
    },
    /// `EXP-35`: oltre un limite del formato.
    LimiteDoFormato {
        folha: &'static str,
        o_que: &'static str,
        encontrado: usize,
        limite: usize,
    },
    /// Il lotto richiesto non esiste nell'archivio.
    LoteAusente {
        pedido: String,
        disponiveis: Vec<String>,
    },
    /// Un'etichetta di enum non si e' serializzata: non accade per gli enum
    /// unitari del progetto, e non si sostituisce con una stringa vuota.
    EtiquetaNaoSerializavel { tipo: &'static str },
}

impl fmt::Display for ErroExport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroExport::TextoNaoRepresentavel {
                folha,
                linha,
                coluna,
                codepoint,
                posicao,
            } => write!(
                f,
                "texto nao representavel em {folha}, linha {linha}, coluna \
                 {coluna}: U+{codepoint:04X} na posicao {posicao}. XML 1.0 nao \
                 o representa em nenhuma forma, e nada e' truncado ou \
                 substituido"
            ),
            ErroExport::TextoTroppoLungo {
                folha,
                linha,
                coluna,
                caracteres,
                limite,
            } => write!(
                f,
                "texto de {caracteres} caracteres em {folha}, linha {linha}, \
                 coluna {coluna}: o limite da celula e' {limite}. Nenhum \
                 truncamento e' aplicado"
            ),
            ErroExport::LimiteDoFormato {
                folha,
                o_que,
                encontrado,
                limite,
            } => write!(
                f,
                "limite do formato em {folha}: {o_que} {encontrado}, limite \
                 {limite}"
            ),
            ErroExport::LoteAusente {
                pedido,
                disponiveis,
            } => write!(
                f,
                "lote {pedido:?} ausente do arquivo. Disponiveis: {}",
                if disponiveis.is_empty() {
                    "nenhum".to_string()
                } else {
                    disponiveis.join(", ")
                }
            ),
            ErroExport::EtiquetaNaoSerializavel { tipo } => {
                write!(f, "etiqueta de {tipo} nao serializavel")
            }
        }
    }
}

impl std::error::Error for ErroExport {}

// ---------------------------------------------------------------------------
// Etichette: una sola verita', quella di serde
// ---------------------------------------------------------------------------

/// Etichetta di un enum, dalla sua rappresentazione `serde`.
///
/// E' la stessa dell'archivio: `arquivo.rs` fissa le etichette con
/// `rotulo_de`, e usare serde qui evita una seconda tabella che potrebbe
/// divergere. Gli enum del progetto hanno varianti unitarie, quindi
/// l'errore non accade; non viene comunque sostituito da una stringa vuota.
fn etiqueta<T: Serialize>(valor: &T, tipo: &'static str) -> Result<String, ErroExport> {
    match serde_json::to_value(valor) {
        Ok(serde_json::Value::String(s)) => Ok(s),
        Ok(outro) => Ok(outro.to_string()),
        Err(_) => Err(ErroExport::EtiquetaNaoSerializavel { tipo }),
    }
}

// ---------------------------------------------------------------------------
// La cella numerica di comodo (D20, D26(a), EXP-37/38/39)
// ---------------------------------------------------------------------------

/// Cifre significative di un decimale, contate sulla mantissa normalizzata.
fn cifras_significativas(valor: Decimal) -> u32 {
    let mantissa = valor.normalize().mantissa().unsigned_abs();
    if mantissa == 0 {
        return 1;
    }
    let mut cifras = 0;
    let mut resto = mantissa;
    while resto > 0 {
        resto /= 10;
        cifras += 1;
    }
    cifras
}

/// Le tre celle della colonna numerica di comodo: valore, stato, motivo.
///
/// `EXP-37`: la **colonna** c'e' sempre, la **cella** puo' mancare.
/// `EXP-38`: eleggibile se il decimale ha al massimo
/// [`CIFRAS_SIGNIFICATIVAS_MAXIMAS`] cifre significative. Mai uno zero
/// sostitutivo.
fn numero_de_comodo(valor: Decimal) -> [Celula; 3] {
    let cifras = cifras_significativas(valor);
    if cifras <= CIFRAS_SIGNIFICATIVAS_MAXIMAS {
        [
            Celula::NumeroDeComodo(valor),
            Celula::texto("OK"),
            Celula::Vazia,
        ]
    } else {
        [
            Celula::Vazia,
            Celula::texto("NAO_REPRESENTAVEL"),
            Celula::texto(format!(
                "{cifras} cifras significativas, acima do limite de \
                 {CIFRAS_SIGNIFICATIVAS_MAXIMAS} da celula numerica"
            )),
        ]
    }
}

/// Le quattro celle di un `Dado<T>`: valore, fonte, stato, motivo (`EXP-16`).
fn dado<T>(valor: &Dado<T>, formatar: impl Fn(&T) -> Celula) -> [Celula; 4] {
    [
        match valor.valor() {
            Some(v) => formatar(v),
            None => Celula::Vazia,
        },
        Celula::texto(valor.fonte()),
        Celula::texto(etiqueta_status(valor.status())),
        Celula::texto_opcional(valor.motivo()),
    ]
}

/// `StatusDado` ha un `rotulo()` pubblico: si usa quello.
fn etiqueta_status(status: StatusDado) -> &'static str {
    status.rotulo()
}

// ---------------------------------------------------------------------------
// Validazione al confine: rappresentabilita' e limiti
// ---------------------------------------------------------------------------

/// `EXP-33`: i caratteri che XML 1.0 non rappresenta in nessuna forma.
///
/// La produzione `Char` di XML 1.0 e':
///
/// ```text
/// Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
/// ```
///
/// Quindi fuori repertorio sono: i controlli C0 tranne tabulazione, LF e CR;
/// i surrogati, che una `char` di Rust non puo' contenere; e **U+FFFE** e
/// **U+FFFF**. Nemmeno `&#0;` rappresenta un controllo vietato: misurato in
/// §5.4 della specifica con due parser indipendenti.
///
/// Riscontro `R3C41-02` del 2026-10-30: la prima versione dimenticava
/// `U+FFFE` e `U+FFFF`, che una `String` di Rust **puo'** contenere, a
/// differenza dei surrogati isolati. Il caso era quindi raggiungibile, e il
/// modello non eseguiva il rifiuto che il contratto gli attribuisce.
///
/// Tutto il piano 1 e oltre resta ammesso, non-caratteri compresi: `U+1FFFE`
/// e' dentro `[#x10000-#x10FFFF]`, e vietarlo sarebbe un eccesso che XML 1.0
/// non chiede. `r2_unicode_valido_resta_ammesso` lo fissa.
fn nao_representavel(c: char) -> bool {
    matches!(
        c,
        '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}'
    )
}

/// Valida una cella di testo: rappresentabilita' (`EXP-18`, `EXP-33`) e
/// lunghezza (`EXP-34`). Non trasforma niente.
fn validar_texto(
    texto: &str,
    folha: &'static str,
    linha: usize,
    coluna: &'static str,
) -> Result<(), ErroExport> {
    for (posicao, c) in texto.char_indices() {
        if nao_representavel(c) {
            return Err(ErroExport::TextoNaoRepresentavel {
                folha,
                linha,
                coluna,
                codepoint: c as u32,
                posicao,
            });
        }
    }
    let caracteres = texto.chars().count();
    if caracteres > LIMITE_CARACTERES_CELULA {
        return Err(ErroExport::TextoTroppoLungo {
            folha,
            linha,
            coluna,
            caracteres,
            limite: LIMITE_CARACTERES_CELULA,
        });
    }
    Ok(())
}

/// Valida un foglio intero: ogni cella di testo, piu' i limiti di righe e
/// colonne (`EXP-35`).
fn validar_folha(folha: &Folha) -> Result<(), ErroExport> {
    if folha.linhas.len() + 1 > LIMITE_LINHAS_FOLHA {
        return Err(ErroExport::LimiteDoFormato {
            folha: folha.nome,
            o_que: "linhas",
            encontrado: folha.linhas.len() + 1,
            limite: LIMITE_LINHAS_FOLHA,
        });
    }
    if folha.cabecalho.len() > LIMITE_COLUNAS_FOLHA {
        return Err(ErroExport::LimiteDoFormato {
            folha: folha.nome,
            o_que: "colunas",
            encontrado: folha.cabecalho.len(),
            limite: LIMITE_COLUNAS_FOLHA,
        });
    }
    for (i, linha) in folha.linhas.iter().enumerate() {
        if linha.len() != folha.cabecalho.len() {
            return Err(ErroExport::LimiteDoFormato {
                folha: folha.nome,
                o_que: "celulas na linha",
                encontrado: linha.len(),
                limite: folha.cabecalho.len(),
            });
        }
        for (j, celula) in linha.iter().enumerate() {
            if let Celula::Texto(t) = celula {
                validar_texto(t, folha.nome, i + 2, folha.cabecalho[j])?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// L'approvazione attiva (EXP-46, EXP-47)
// ---------------------------------------------------------------------------

/// Chi apre l'approvazione attiva, e le conferme successive.
struct Aprovacao<'a> {
    /// La **prima** `Aprovar` dopo l'ultima `Revogar` efficace.
    abre: Option<&'a EventoRevisao>,
    /// Le `Aprovar` applicate **dopo** quella attiva: ripetizioni, non
    /// approvazioni (`APROVACAO_REPETIDA` di `revisao.rs`).
    confirmacoes: Vec<&'a EventoRevisao>,
}

/// `EXP-46`: una seconda approvazione «fica na trilha e nao substitui a
/// primeira». Scorre `decisoes`, che e' gia' in ordine canonico.
fn aprovacao_ativa(decisoes: &[DecisaoAplicada]) -> Aprovacao<'_> {
    let mut aprovacao = Aprovacao {
        abre: None,
        confirmacoes: Vec::new(),
    };
    for decisao in decisoes {
        match decisao.evento.operacao {
            OperacaoRevisao::Aprovar => {
                if aprovacao.abre.is_none() {
                    aprovacao.abre = Some(&decisao.evento);
                } else {
                    aprovacao.confirmacoes.push(&decisao.evento);
                }
            }
            OperacaoRevisao::Revogar => {
                aprovacao.abre = None;
                aprovacao.confirmacoes.clear();
            }
            OperacaoRevisao::Anotar => {}
        }
    }
    aprovacao
}

/// Le tre celle di un istante: locale e offset in secondi (`EXP-15`).
fn instante(valor: DateTime<FixedOffset>) -> [Celula; 2] {
    [
        Celula::texto(
            valor
                .naive_local()
                .format("%Y-%m-%dT%H:%M:%S%.9f")
                .to_string(),
        ),
        Celula::NumeroDeComodo(Decimal::from(valor.offset().local_minus_utc())),
    ]
}

// ---------------------------------------------------------------------------
// Sotto-archivio del lotto (EXP-30)
// ---------------------------------------------------------------------------

/// Archivio ridotto al lotto scelto, per restringere lo scope del controllo
/// di presentazione senza modificare `arquivo`.
///
/// `EXP-30`: `arquivo::verificar_apresentacao` visita **tutti** gli snapshot e
/// puo' restituire `ContaFora` per un conto che appartiene a un altro lotto.
/// Si restringe l'insieme, non la funzione.
pub fn sub_arquivo_do_lote(arquivo: &Arquivo, lote: &str) -> Result<Arquivo, ErroExport> {
    let execucao = arquivo.execucoes.iter().find(|e| e.lote.id == lote);
    let Some(execucao) = execucao else {
        return Err(ErroExport::LoteAusente {
            pedido: lote.to_string(),
            disponiveis: arquivo
                .execucoes
                .iter()
                .map(|e| e.lote.id.clone())
                .collect(),
        });
    };
    Ok(Arquivo {
        execucoes: vec![execucao.clone()],
        snapshots: arquivo
            .snapshots
            .iter()
            .filter(|s| s.lote == lote)
            .cloned()
            .collect(),
        eventos: arquivo.eventos.clone(),
    })
}

// ---------------------------------------------------------------------------
// Intestazioni: l'ordine delle colonne e' il contratto (EXP-01)
// ---------------------------------------------------------------------------

const C_PROVENIENCIA: &[&str] = &[
    "tipo_linha",
    "formato",
    "versao_schema",
    "gerado_em",
    "arquivo_caminho",
    "arquivo_revisao",
    "arquivo_sha256",
    "lote_id",
    "lote_gerado_em",
    "lote_descricao",
    "plano_arquivado_versao",
    "plano_arquivado_sha256",
    "plano_arquivado_caminho",
    "plano_usado_versao",
    "plano_usado_caminho",
    "plano_coincide",
    "gramatica_conteudo",
    "gramatica_apresentacao",
    "gramatica_evento",
    "eventos_no_arquivo",
    "eventos_aplicados",
    "eventos_nao_aplicados",
    "documentos",
    "linhas",
    "apresentacao_divergente",
    "div_chave_origem",
    "div_impressao_salva",
    "div_impressao_atual",
];

const C_DIAGNOSTICO: &[&str] = &[
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
    "numero",
    "serie",
    "data_documento",
    "competencia",
    "cnpj_empresa",
    "nome_empresa",
    "empresa_identificada",
    "fornecedor_ou_favorecido",
    "cnpj_cpf_favorecido",
    "documento",
    "tipo_documento",
    "conta_nome",
    "conta_nome_plano",
    "grupo_conta",
    "tipo_lancamento",
    "descricao_itens",
    "valor",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
    "status_tecnico",
    "estado_revisao",
    "particao",
    "no_extrato_aprovado",
    "no_extrato_revisado",
    "stato_documento",
    "motivos",
    "alertas",
    "observacoes",
    "centro_custo",
    "dre_linha_id",
    "dre_linha_nome",
    "data_pagamento",
    "data_pagamento_fonte",
    "data_pagamento_status",
    "data_pagamento_motivo",
    "arquivo_origem",
    "hash_registro",
    "impressao_conteudo",
    "impressao_apresentacao",
    "decisoes_aplicadas",
    "anotacoes_aplicadas",
    "resumo_cnpj_empresa",
    "resumo_competencia",
    "resumo_conta_id",
];

const C_RATE: &[&str] = &[
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
    "indice",
    "numero",
    "vencimento",
    "valor",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
];

const C_DOCUMENTOS: &[&str] = &[
    "lote",
    "chave_origem",
    "numero",
    "serie",
    "data_documento",
    "competencia",
    "cnpj_empresa",
    "nome_empresa",
    "empresa_identificada",
    "arquivo_origem",
    "linhas",
    "stato_documento",
    "valor_total_nfe",
    "valor_total_nfe_numero",
    "valor_total_nfe_numero_status",
    "valor_total_nfe_numero_motivo",
    "soma_lancamentos",
    "soma_lancamentos_numero",
    "soma_lancamentos_numero_status",
    "soma_lancamentos_numero_motivo",
    "esito_total",
    "esito_rate",
    "esito_copertura",
    "esito_componenti",
    "soma_rate_documento",
    "soma_rate_documento_fonte",
    "soma_rate_documento_status",
    "soma_rate_documento_motivo",
    "soma_rate_documento_numero",
    "soma_rate_documento_numero_status",
    "soma_rate_documento_numero_motivo",
    "soma_rate_allocate",
    "soma_rate_allocate_fonte",
    "soma_rate_allocate_status",
    "soma_rate_allocate_motivo",
    "soma_rate_allocate_numero",
    "soma_rate_allocate_numero_status",
    "soma_rate_allocate_numero_motivo",
    "articoli_allocati",
    "articoli_attesi",
    "articoli_attesi_fonte",
    "articoli_attesi_status",
    "articoli_attesi_motivo",
    "articoli_scartati_qtd",
    "avisos_documento_qtd",
    "avisos_leitura_qtd",
    "componenti_qtd",
    "campi_rinviati_qtd",
];

const C_COMPONENTES: &[&str] = &[
    "lote",
    "chave_origem",
    "indice",
    "nome",
    "ruolo",
    "usada_na_verificacao",
    "valor",
    "valor_fonte",
    "valor_status",
    "valor_motivo",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
];

const C_CAMPOS_RINVIATI: &[&str] = &[
    "lote",
    "chave_origem",
    "indice",
    "campo",
    "status",
    "fonte",
    "motivo",
];

const C_RESUMOS: &[&str] = &[
    "cnpj_empresa",
    "competencia",
    "conta_id",
    "conta_nome",
    "grupo_conta",
    "empresa_identificada",
    "qtd_aprovado",
    "qtd_ok",
    "qtd_revisar",
    "qtd_total",
    "valor_aprovado",
    "valor_ok",
    "valor_revisar",
    "valor_total",
    "valor_aprovado_numero",
    "valor_ok_numero",
    "valor_revisar_numero",
    "valor_total_numero",
    "valor_numero_status",
    "valor_numero_motivo",
    "documentos_ok",
    "documentos_parcial",
    "documentos_erro",
    "documentos_marcados",
    "nomes_divergentes",
];

const C_INDICADORES: &[&str] = &[
    "nome",
    "valor",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
    "tipo",
];

const C_EXTRATO: &[&str] = &[
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
    "data_documento",
    "competencia",
    "cnpj_empresa",
    "nome_empresa",
    "fornecedor_ou_favorecido",
    "documento",
    "conta_nome_plano",
    "grupo_conta",
    "descricao_itens",
    "valor",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
    "estado_revisao",
    "aprovado_por",
    "aprovado_em",
    "aprovado_offset_segundos",
    "confirmacoes",
    "ultima_confirmacao_por",
    "ultima_confirmacao_em",
    "ultima_confirmacao_offset_segundos",
];

const C_EXCLUSOES: &[&str] = &[
    "estrato",
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
    "particao",
    "stato_documento",
    "valor",
    "valor_numero",
    "valor_numero_status",
    "valor_numero_motivo",
    "motivo",
];

const C_DECISOES: &[&str] = &[
    "aplicado",
    "motivo_nao_aplicacao",
    "detalhe",
    "alvo_chave_origem",
    "alvo_origem",
    "alvo_conta_id",
    "alvo_ordinal",
    "alvo_impressao_conteudo",
    "alvo_impressao_conteudo_fonte",
    "alvo_impressao_conteudo_status",
    "alvo_impressao_conteudo_motivo",
    "operacao",
    "autor",
    "instante_local",
    "instante_offset_segundos",
    "motivo",
    "nota",
    "sequencia",
    "vista",
    "vista_fonte",
    "vista_status",
    "vista_motivo",
    "repeticoes",
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
];

const C_AVISOS: &[&str] = &[
    "fonte",
    "lote",
    "chave_origem",
    "origem",
    "conta_id",
    "ordinal",
    "resumo_cnpj_empresa",
    "resumo_competencia",
    "resumo_conta_id",
    "indice",
    "codigo",
    "detalhe",
];

const C_LIMITES: &[&str] = &["codigo", "descricao", "stato"];

// ---------------------------------------------------------------------------
// Costruzione dei tredici fogli
// ---------------------------------------------------------------------------

fn folha_proveniencia(vista: &VistaRevisada, p: &Proveniencia) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("proveniencia", C_PROVENIENCIA);
    let lote = vista.pre_livro.lote.as_ref();
    let aplicados = vista.trilha.len();
    let nao_aplicados = vista.eventos_nao_aplicados.len();

    let mut metadato = vec![
        Celula::texto("METADATO"),
        Celula::texto(FORMATO),
        // EXP-41: identificativi come testo.
        Celula::texto(VERSAO_SCHEMA.to_string()),
    ];
    metadato.push(Celula::texto(
        p.gerado_em.format("%Y-%m-%dT%H:%M:%S%.9f%:z").to_string(),
    ));
    metadato.push(Celula::texto(&p.arquivo_caminho));
    metadato.push(Celula::texto(p.arquivo_revisao.to_string()));
    metadato.push(Celula::texto(&p.arquivo_sha256));
    match lote {
        Some(l) => {
            metadato.push(Celula::texto(&l.id));
            metadato.push(Celula::data(l.gerado_em));
            metadato.push(Celula::texto(&l.descricao));
        }
        None => {
            metadato.push(Celula::Vazia);
            metadato.push(Celula::Vazia);
            metadato.push(Celula::Vazia);
        }
    }
    metadato.push(Celula::texto(&p.plano_arquivado.versao));
    metadato.push(Celula::texto_opcional(
        p.plano_arquivado.sha256_arquivo.as_deref(),
    ));
    metadato.push(Celula::texto_opcional(
        p.plano_arquivado.caminho_declarado.as_deref(),
    ));
    metadato.push(Celula::texto(&p.plano_usado_versao));
    metadato.push(Celula::texto(&p.plano_usado_caminho));
    metadato.push(Celula::texto(p.coincidencia_do_plano()));
    metadato.push(Celula::texto(impressao::VERSAO_CONTEUDO));
    metadato.push(Celula::texto(impressao::VERSAO_APRESENTACAO));
    metadato.push(Celula::texto(revisao::VERSAO_EVENTO_EXATO));
    metadato.push(Celula::contagem(p.eventos_no_arquivo));
    metadato.push(Celula::contagem(aplicados));
    metadato.push(Celula::contagem(nao_aplicados));
    metadato.push(Celula::contagem(vista.pre_livro.indicadores.documentos));
    metadato.push(Celula::contagem(vista.pre_livro.indicadores.linhas));
    metadato.push(Celula::booleano(!p.divergencias.is_empty()));
    metadato.push(Celula::Vazia);
    metadato.push(Celula::Vazia);
    metadato.push(Celula::Vazia);
    folha.linhas.push(metadato);

    for d in &p.divergencias {
        let mut linha = vec![Celula::texto("DIVERGENCIA")];
        linha.extend(std::iter::repeat_n(Celula::Vazia, C_PROVENIENCIA.len() - 4));
        linha.push(Celula::texto(&d.chave_origem));
        linha.push(Celula::texto(&d.salva));
        linha.push(Celula::texto(&d.atual));
        folha.linhas.push(linha);
    }
    Ok(folha)
}

/// La riga della vista che corrisponde a una riga del pre-libro.
fn revisada_de<'a>(vista: &'a VistaRevisada, linha: &LinhaPreLivro) -> Option<&'a LinhaRevisada> {
    vista
        .linhas
        .iter()
        .find(|r| r.identidade == linha.identidade)
}

fn documento_de<'a>(
    pre_livro: &'a PreLivro,
    lote: &str,
    chave: &str,
) -> Option<&'a DocumentoPreLivro> {
    pre_livro
        .documentos
        .iter()
        .find(|d| d.lote == lote && d.chave_origem == chave)
}

fn folha_diagnostico(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("diagnostico", C_DIAGNOSTICO);
    for linha in &vista.pre_livro.linhas {
        let l = &linha.lancamento;
        let revisada = revisada_de(vista, linha);
        let documento = documento_de(
            &vista.pre_livro,
            &linha.identidade.lote,
            &linha.identidade.chave_origem,
        );
        let versao = vista
            .versoes
            .iter()
            .find(|v| v.chave_origem == linha.identidade.chave_origem);

        let mut celulas = vec![
            Celula::texto(&linha.identidade.lote),
            Celula::texto(&linha.identidade.chave_origem),
            Celula::texto(etiqueta(&linha.identidade.origem, "OrigemLancamento")?),
            Celula::texto(&linha.identidade.conta_id),
            Celula::contagem(linha.identidade.ordinal as usize),
            Celula::texto_opcional(documento.map(|d| d.numero.as_str())),
            Celula::texto_opcional(documento.map(|d| d.serie.as_str())),
            Celula::data(l.data_documento),
            Celula::texto(l.competencia.to_string()),
            Celula::texto(&l.cnpj_empresa),
            Celula::texto(&l.nome_empresa),
            Celula::booleano(linha.empresa_identificada),
            Celula::texto(&l.fornecedor_ou_favorecido),
            Celula::texto(&l.cnpj_cpf_favorecido),
            Celula::texto(&l.documento),
            Celula::texto(&l.tipo_documento),
            Celula::texto(&l.conta_nome),
            Celula::texto(&linha.conta_nome_plano),
            Celula::texto(&l.grupo_conta),
            Celula::texto(etiqueta(&l.tipo_lancamento, "TipoConta")?),
            Celula::texto(&l.descricao_itens),
            Celula::decimal(l.valor),
        ];
        celulas.extend(numero_de_comodo(l.valor));
        celulas.push(Celula::texto(etiqueta(&l.status_revisao, "StatusRevisao")?));
        celulas.push(match revisada {
            Some(r) => Celula::texto(r.estado.rotulo()),
            None => Celula::Vazia,
        });
        celulas.push(Celula::texto(linha.particao.rotulo()));
        celulas.push(Celula::booleano(linha.no_extrato_aprovado));
        celulas.push(match revisada {
            Some(r) => Celula::booleano(r.no_extrato_revisado),
            None => Celula::Vazia,
        });
        celulas.push(Celula::texto(etiqueta(
            &linha.stato_documento,
            "StatoDocumento",
        )?));
        celulas.push(Celula::lista(&linha.motivos));
        celulas.push(Celula::texto(&l.alertas));
        celulas.push(Celula::texto(&l.observacoes));
        celulas.push(Celula::texto(&l.centro_custo));
        celulas.push(Celula::texto(&l.dre_linha_id));
        celulas.push(Celula::texto(&l.dre_linha_nome));
        celulas.extend(dado(&l.data_pagamento, |d| Celula::data(*d)));
        celulas.push(Celula::texto(&l.arquivo_origem));
        celulas.push(Celula::texto(&l.hash_registro));
        celulas.push(Celula::texto_opcional(
            versao.map(|v| v.impressao_conteudo.as_str()),
        ));
        celulas.push(Celula::texto_opcional(
            versao.map(|v| v.impressao_apresentacao.as_str()),
        ));
        celulas.push(Celula::contagem(
            revisada.map(|r| r.decisoes.len()).unwrap_or(0),
        ));
        celulas.push(Celula::contagem(
            revisada.map(|r| r.anotacoes.len()).unwrap_or(0),
        ));
        celulas.push(Celula::texto(&linha.chave_resumo.cnpj_empresa));
        celulas.push(Celula::texto(linha.chave_resumo.competencia.to_string()));
        celulas.push(Celula::texto(&linha.chave_resumo.conta_id));
        folha.linhas.push(celulas);
    }
    Ok(folha)
}

fn folha_rate(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("rate", C_RATE);
    for linha in &vista.pre_livro.linhas {
        for (indice, rata) in linha.lancamento.duplicatas.iter().enumerate() {
            let mut celulas = vec![
                Celula::texto(&linha.identidade.lote),
                Celula::texto(&linha.identidade.chave_origem),
                Celula::texto(etiqueta(&linha.identidade.origem, "OrigemLancamento")?),
                Celula::texto(&linha.identidade.conta_id),
                Celula::contagem(linha.identidade.ordinal as usize),
                Celula::contagem(indice),
                Celula::texto(&rata.numero),
                Celula::data_opcional(rata.vencimento),
                Celula::decimal(rata.valor),
            ];
            celulas.extend(numero_de_comodo(rata.valor));
            folha.linhas.push(celulas);
        }
    }
    Ok(folha)
}

fn celulas_reconciliacao(r: &Reconciliacao) -> Result<Vec<Celula>, ErroExport> {
    let mut c = vec![Celula::decimal(r.valor_total_nfe)];
    c.extend(numero_de_comodo(r.valor_total_nfe));
    c.push(Celula::decimal(r.soma_lancamentos));
    c.extend(numero_de_comodo(r.soma_lancamentos));
    c.push(Celula::texto(etiqueta(&r.esito_total, "EsitoTotal")?));
    c.push(Celula::texto(etiqueta(&r.esito_rate, "EsitoRate")?));
    c.push(Celula::texto(etiqueta(
        &r.esito_copertura,
        "EsitoCopertura",
    )?));
    c.push(Celula::texto(etiqueta(
        &r.esito_componenti,
        "EsitoComponenti",
    )?));
    c.extend(dado(&r.soma_rate_documento, |v| Celula::decimal(*v)));
    c.extend(match r.soma_rate_documento.valor() {
        Some(v) => numero_de_comodo(*v),
        None => [Celula::Vazia, Celula::texto("N.A."), Celula::Vazia],
    });
    c.extend(dado(&r.soma_rate_allocate, |v| Celula::decimal(*v)));
    c.extend(match r.soma_rate_allocate.valor() {
        Some(v) => numero_de_comodo(*v),
        None => [Celula::Vazia, Celula::texto("N.A."), Celula::Vazia],
    });
    c.push(Celula::contagem(r.articoli_allocati));
    c.extend(dado(&r.articoli_attesi, |v| Celula::contagem(*v)));
    c.push(Celula::contagem(r.articoli_scartati.len()));
    c.push(Celula::contagem(r.avisos_documento.len()));
    Ok(c)
}

fn folha_documentos(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("documentos", C_DOCUMENTOS);
    for d in &vista.pre_livro.documentos {
        let mut celulas = vec![
            Celula::texto(&d.lote),
            Celula::texto(&d.chave_origem),
            Celula::texto(&d.numero),
            Celula::texto(&d.serie),
            Celula::data(d.data_documento),
            Celula::texto(d.competencia.to_string()),
            Celula::texto(&d.cnpj_empresa),
            Celula::texto(&d.nome_empresa),
            Celula::booleano(d.empresa_identificada),
            Celula::texto(&d.arquivo_origem),
            Celula::contagem(d.linhas),
            Celula::texto(etiqueta(&d.stato_documento, "StatoDocumento")?),
        ];
        celulas.extend(celulas_reconciliacao(&d.reconciliacao)?);
        celulas.push(Celula::contagem(d.avisos_leitura.len()));
        celulas.push(Celula::contagem(d.reconciliacao.componenti.len()));
        celulas.push(Celula::contagem(d.reconciliacao.campi_rinviati.len()));
        folha.linhas.push(celulas);
    }
    Ok(folha)
}

fn folha_componentes(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("reconciliacao_componentes", C_COMPONENTES);
    for d in &vista.pre_livro.documentos {
        for (indice, c) in d.reconciliacao.componenti.iter().enumerate() {
            folha.linhas.push(celulas_componente(d, indice, c)?);
        }
    }
    Ok(folha)
}

fn celulas_componente(
    d: &DocumentoPreLivro,
    indice: usize,
    c: &Componente,
) -> Result<Vec<Celula>, ErroExport> {
    let mut celulas = vec![
        Celula::texto(&d.lote),
        Celula::texto(&d.chave_origem),
        Celula::contagem(indice),
        Celula::texto(&c.nome),
        Celula::texto(etiqueta(&c.ruolo, "RuoloComponente")?),
        Celula::booleano(c.usada_na_verificacao),
    ];
    celulas.extend(dado(&c.valor, |v| Celula::decimal(*v)));
    celulas.extend(match c.valor.valor() {
        Some(v) => numero_de_comodo(*v),
        None => [Celula::Vazia, Celula::texto("N.A."), Celula::Vazia],
    });
    Ok(celulas)
}

fn folha_campos_rinviati(vista: &VistaRevisada) -> Folha {
    let mut folha = Folha::nova("reconciliacao_campos_rinviati", C_CAMPOS_RINVIATI);
    for d in &vista.pre_livro.documentos {
        for (indice, campo) in d.reconciliacao.campi_rinviati.iter().enumerate() {
            folha.linhas.push(vec![
                Celula::texto(&d.lote),
                Celula::texto(&d.chave_origem),
                Celula::contagem(indice),
                Celula::texto(&campo.campo),
                Celula::texto(&campo.status),
                Celula::texto(&campo.fonte),
                Celula::texto(&campo.motivo),
            ]);
        }
    }
    folha
}

fn folha_resumos(vista: &VistaRevisada) -> Folha {
    let mut folha = Folha::nova("resumos", C_RESUMOS);
    for r in &vista.pre_livro.resumos {
        // L'eleggibilita' si valuta sui quattro valori insieme: una riga di
        // totali non deve invitare a sommare cio' che non e' sommabile.
        let eleggibili = [r.valor_aprovado, r.valor_ok, r.valor_revisar, r.valor_total]
            .iter()
            .all(|v| cifras_significativas(*v) <= CIFRAS_SIGNIFICATIVAS_MAXIMAS);
        let numero = |v: Decimal| {
            if eleggibili {
                Celula::NumeroDeComodo(v)
            } else {
                Celula::Vazia
            }
        };
        folha.linhas.push(vec![
            Celula::texto(&r.chave.cnpj_empresa),
            Celula::texto(r.chave.competencia.to_string()),
            Celula::texto(&r.chave.conta_id),
            Celula::texto(&r.conta_nome),
            Celula::texto(&r.grupo_conta),
            Celula::booleano(r.empresa_identificada),
            Celula::contagem(r.qtd_aprovado),
            Celula::contagem(r.qtd_ok),
            Celula::contagem(r.qtd_revisar),
            Celula::contagem(r.qtd_total),
            Celula::decimal(r.valor_aprovado),
            Celula::decimal(r.valor_ok),
            Celula::decimal(r.valor_revisar),
            Celula::decimal(r.valor_total),
            numero(r.valor_aprovado),
            numero(r.valor_ok),
            numero(r.valor_revisar),
            numero(r.valor_total),
            Celula::texto(if eleggibili {
                "OK"
            } else {
                "NAO_REPRESENTAVEL"
            }),
            if eleggibili {
                Celula::Vazia
            } else {
                Celula::texto(
                    "um dos quatro valores excede as cifras significativas da \
                     celula numerica: nenhum dos quatro e' escrito",
                )
            },
            Celula::contagem(r.documentos_ok),
            Celula::contagem(r.documentos_parcial),
            Celula::contagem(r.documentos_erro),
            Celula::lista(&r.documentos_marcados),
            Celula::lista(&r.nomes_divergentes),
        ]);
    }
    folha
}

fn folha_indicadores(i: &Indicadores) -> Folha {
    let mut folha = Folha::nova("indicadores", C_INDICADORES);
    let contagens: [(&str, usize); 10] = [
        ("documentos", i.documentos),
        ("documentos_ok", i.documentos_ok),
        ("documentos_parcial", i.documentos_parcial),
        ("documentos_erro", i.documentos_erro),
        ("documentos_sem_linhas", i.documentos_sem_linhas),
        ("linhas", i.linhas),
        ("linhas_aprovado", i.linhas_aprovado),
        ("linhas_ok", i.linhas_ok),
        ("linhas_revisar", i.linhas_revisar),
        ("linhas_excluidas_do_extrato", i.linhas_excluidas_do_extrato),
    ];
    let monetarios: [(&str, Decimal); 5] = [
        ("valor_total", i.valor_total),
        ("valor_aprovado", i.valor_aprovado),
        ("valor_ok", i.valor_ok),
        ("valor_revisar", i.valor_revisar),
        ("valor_extrato_aprovado", i.valor_extrato_aprovado),
    ];
    // L'ordine e' quello dei campi della struttura, non alfabetico.
    for (nome, valor) in &contagens[..9] {
        empilha_indicador(&mut folha, nome, Decimal::from(*valor), "CONTAGEM");
    }
    for (nome, valor) in &monetarios {
        empilha_indicador(&mut folha, nome, *valor, "MONETARIO");
    }
    let (nome, valor) = contagens[9];
    empilha_indicador(&mut folha, nome, Decimal::from(valor), "CONTAGEM");
    folha
}

fn empilha_indicador(folha: &mut Folha, nome: &str, valor: Decimal, tipo: &str) {
    let mut celulas = vec![Celula::texto(nome), Celula::decimal(valor)];
    celulas.extend(numero_de_comodo(valor));
    celulas.push(Celula::texto(tipo));
    folha.linhas.push(celulas);
}

fn folha_extrato(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("extrato_revisado", C_EXTRATO);
    for revisada in vista.extrato_revisado() {
        let Some(linha) = vista
            .pre_livro
            .linhas
            .iter()
            .find(|l| l.identidade == revisada.identidade)
        else {
            continue;
        };
        let l = &linha.lancamento;
        let aprovacao = aprovacao_ativa(&revisada.decisoes);
        let mut celulas = vec![
            Celula::texto(&linha.identidade.lote),
            Celula::texto(&linha.identidade.chave_origem),
            Celula::texto(etiqueta(&linha.identidade.origem, "OrigemLancamento")?),
            Celula::texto(&linha.identidade.conta_id),
            Celula::contagem(linha.identidade.ordinal as usize),
            Celula::data(l.data_documento),
            Celula::texto(l.competencia.to_string()),
            Celula::texto(&l.cnpj_empresa),
            Celula::texto(&l.nome_empresa),
            Celula::texto(&l.fornecedor_ou_favorecido),
            Celula::texto(&l.documento),
            Celula::texto(&linha.conta_nome_plano),
            Celula::texto(&l.grupo_conta),
            Celula::texto(&l.descricao_itens),
            Celula::decimal(l.valor),
        ];
        celulas.extend(numero_de_comodo(l.valor));
        celulas.push(Celula::texto(revisada.estado.rotulo()));
        match aprovacao.abre {
            Some(e) => {
                celulas.push(Celula::texto(&e.autor));
                celulas.extend(instante(e.instante));
            }
            None => {
                celulas.push(Celula::Vazia);
                celulas.push(Celula::Vazia);
                celulas.push(Celula::Vazia);
            }
        }
        celulas.push(Celula::contagem(aprovacao.confirmacoes.len()));
        match aprovacao.confirmacoes.last() {
            Some(e) => {
                celulas.push(Celula::texto(&e.autor));
                celulas.extend(instante(e.instante));
            }
            None => {
                celulas.push(Celula::Vazia);
                celulas.push(Celula::Vazia);
                celulas.push(Celula::Vazia);
            }
        }
        folha.linhas.push(celulas);
    }
    Ok(folha)
}

fn folha_exclusoes(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("exclusoes", C_EXCLUSOES);
    for (estrato, lista) in [
        ("APROVADO", &vista.pre_livro.exclusoes_extrato),
        ("REVISADO", &vista.exclusoes_extrato_revisado),
    ] {
        for e in lista.iter() {
            folha.linhas.push(celulas_exclusao(estrato, e)?);
        }
    }
    Ok(folha)
}

fn celulas_exclusao(estrato: &str, e: &Exclusao) -> Result<Vec<Celula>, ErroExport> {
    let mut celulas = vec![
        Celula::texto(estrato),
        Celula::texto(&e.identidade.lote),
        Celula::texto(&e.identidade.chave_origem),
        Celula::texto(etiqueta(&e.identidade.origem, "OrigemLancamento")?),
        Celula::texto(&e.identidade.conta_id),
        Celula::contagem(e.identidade.ordinal as usize),
        Celula::texto(e.particao.rotulo()),
        Celula::texto(etiqueta(&e.stato_documento, "StatoDocumento")?),
        Celula::decimal(e.valor),
    ];
    celulas.extend(numero_de_comodo(e.valor));
    celulas.push(Celula::texto(&e.motivo));
    Ok(celulas)
}

fn folha_decisoes(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("decisoes", C_DECISOES);
    for decisao in &vista.trilha {
        let identidade = vista
            .linhas
            .iter()
            .find(|r| {
                r.chave.chave_origem == decisao.evento.alvo.chave_origem
                    && r.chave.conta_id == decisao.evento.alvo.conta_id
                    && r.chave.ordinal == decisao.evento.alvo.ordinal
            })
            .map(|r| &r.identidade);
        folha.linhas.push(celulas_decisao(
            true,
            None,
            "",
            &decisao.evento,
            decisao.repeticoes,
            identidade,
        )?);
    }
    for nao in &vista.eventos_nao_aplicados {
        folha.linhas.push(celulas_decisao(
            false,
            Some(nao.motivo.rotulo()),
            &nao.detalhe,
            &nao.evento,
            0,
            None,
        )?);
    }
    Ok(folha)
}

fn celulas_decisao(
    aplicado: bool,
    motivo_nao: Option<&str>,
    detalhe: &str,
    e: &EventoRevisao,
    repeticoes: usize,
    identidade: Option<&crate::prelivro::IdentidadeLinha>,
) -> Result<Vec<Celula>, ErroExport> {
    let mut celulas = vec![
        Celula::booleano(aplicado),
        Celula::texto_opcional(motivo_nao),
        if detalhe.is_empty() {
            Celula::Vazia
        } else {
            Celula::texto(detalhe)
        },
        Celula::texto(&e.alvo.chave_origem),
        Celula::texto(etiqueta(&e.alvo.origem, "OrigemLancamento")?),
        Celula::texto(&e.alvo.conta_id),
        Celula::contagem(e.alvo.ordinal as usize),
    ];
    // EXP-16: il `Dado` del bersaglio ha quattro colonne, non una.
    celulas.extend(dado(&e.alvo.impressao_conteudo, |v| Celula::texto(v)));
    celulas.push(Celula::texto(e.operacao.rotulo()));
    celulas.push(Celula::texto(&e.autor));
    celulas.extend(instante(e.instante));
    celulas.push(Celula::texto(&e.motivo));
    celulas.push(Celula::texto_opcional(e.nota.as_deref()));
    celulas.push(match e.sequencia {
        Some(s) => Celula::texto(s.to_string()),
        None => Celula::Vazia,
    });
    celulas.extend(dado(&e.vista, |v| Celula::texto(v)));
    celulas.push(Celula::contagem(repeticoes));
    match identidade {
        Some(i) => {
            celulas.push(Celula::texto(&i.lote));
            celulas.push(Celula::texto(&i.chave_origem));
            celulas.push(Celula::texto(etiqueta(&i.origem, "OrigemLancamento")?));
            celulas.push(Celula::texto(&i.conta_id));
            celulas.push(Celula::contagem(i.ordinal as usize));
        }
        None => celulas.extend(std::iter::repeat_n(Celula::Vazia, 5)),
    }
    Ok(celulas)
}

fn folha_avisos(vista: &VistaRevisada) -> Result<Folha, ErroExport> {
    let mut folha = Folha::nova("avisos", C_AVISOS);
    let vazio_linha = || std::iter::repeat_n(Celula::Vazia, 5);
    let vazio_resumo = || std::iter::repeat_n(Celula::Vazia, 3);

    let mut senza_chiave = |fonte: &str, indice: usize, a: &Aviso| {
        let mut celulas = vec![Celula::texto(fonte)];
        celulas.extend(vazio_linha());
        celulas.extend(vazio_resumo());
        celulas.push(Celula::contagem(indice));
        celulas.push(Celula::texto(&a.codigo));
        celulas.push(Celula::texto(&a.detalhe));
        folha.linhas.push(celulas);
    };
    for (i, a) in vista.avisos.iter().enumerate() {
        senza_chiave("VISTA", i, a);
    }
    for (i, a) in vista.pre_livro.avisos.iter().enumerate() {
        senza_chiave("PRE_LIVRO", i, a);
    }

    for d in &vista.pre_livro.documentos {
        for (fonte, lista) in [
            ("LEITURA", &d.avisos_leitura),
            ("DOCUMENTO", &d.reconciliacao.avisos_documento),
            ("ARTIGO_DESCARTADO", &d.reconciliacao.articoli_scartati),
        ] {
            for (i, a) in lista.iter().enumerate() {
                let mut celulas = vec![
                    Celula::texto(fonte),
                    Celula::texto(&d.lote),
                    Celula::texto(&d.chave_origem),
                    Celula::Vazia,
                    Celula::Vazia,
                    Celula::Vazia,
                ];
                celulas.extend(vazio_resumo());
                celulas.push(Celula::contagem(i));
                celulas.push(Celula::texto(&a.codigo));
                celulas.push(Celula::texto(&a.detalhe));
                folha.linhas.push(celulas);
            }
        }
    }

    // LINHA_MOTIVO: la CHIAVE_LINHA completa, non `(lote, chave_origem)`.
    for linha in &vista.pre_livro.linhas {
        for (i, motivo) in linha.motivos.iter().enumerate() {
            let mut celulas = vec![
                Celula::texto("LINHA_MOTIVO"),
                Celula::texto(&linha.identidade.lote),
                Celula::texto(&linha.identidade.chave_origem),
                Celula::texto(etiqueta(&linha.identidade.origem, "OrigemLancamento")?),
                Celula::texto(&linha.identidade.conta_id),
                Celula::contagem(linha.identidade.ordinal as usize),
            ];
            celulas.extend(vazio_resumo());
            celulas.push(Celula::contagem(i));
            celulas.push(Celula::texto("LINHA_MOTIVO"));
            celulas.push(Celula::texto(motivo));
            folha.linhas.push(celulas);
        }
    }

    // RESUMO_*: la CHIAVE_RESUMO.
    for r in &vista.pre_livro.resumos {
        for (fonte, lista) in [
            ("RESUMO_DOCUMENTO_MARCADO", &r.documentos_marcados),
            ("RESUMO_NOME_DIVERGENTE", &r.nomes_divergentes),
        ] {
            for (i, valor) in lista.iter().enumerate() {
                let mut celulas = vec![Celula::texto(fonte)];
                celulas.extend(vazio_linha());
                celulas.push(Celula::texto(&r.chave.cnpj_empresa));
                celulas.push(Celula::texto(r.chave.competencia.to_string()));
                celulas.push(Celula::texto(&r.chave.conta_id));
                celulas.push(Celula::contagem(i));
                celulas.push(Celula::texto(fonte));
                celulas.push(Celula::texto(valor));
                folha.linhas.push(celulas);
            }
        }
    }
    Ok(folha)
}

/// I limiti dichiarati nel file stesso (§6.3.14).
///
/// `ESCAPE_LEITOR` e' la divergenza nota del lettore, decisa il 2026-10-23
/// (strada (a), `EXP-58`): il limite vive nel workbook, non solo nella
/// specifica.
fn folha_limites() -> Folha {
    let mut folha = Folha::nova("limites", C_LIMITES);
    let voci: [(&str, &str, &str); 9] = [
        (
            "DERIVADO_SEM_RETORNO",
            "Este arquivo e' um derivado do arquivo autoritativo. Alterar o \
             workbook nao e' uma decisao humana e nao modifica o arquivo: nao \
             existe caminho de retorno.",
            "DICHIARATO",
        ),
        (
            "AUTOR_NAO_AUTENTICADO",
            "O autor de uma decisao e' uma cadeia declarada, nao uma \
             identidade verificada.",
            "DICHIARATO",
        ),
        (
            "NUMERO_NAO_AUTORITATIVO",
            "As colunas terminadas em _numero sao de comodidade e nao sao \
             autoritativas: a escala nao e' preservada nelas. O valor exato \
             esta' sempre na coluna textual.",
            "DICHIARATO",
        ),
        (
            "ESCAPE_LEITOR",
            "Um texto que contem uma sequencia semelhante a _xHHHH_ ja' \
             protegida pode ser mostrado de forma diferente: medido com \
             openpyxl 3.1.5, a entrada _x005F_x0041_ foi relida _x0041_. O XML \
             produzido e' correto; a perda vem do decodificador do leitor.",
            "DIVERGENZA_NOTA",
        ),
        (
            "EXCEL_NAO_ATESTADO",
            "A compatibilidade com Microsoft Excel nao e' atestada: as provas \
             TX-18 e TX-19 nao foram executadas.",
            "NON_VERIFICATO",
        ),
        (
            "SEM_SEQUENCIA_HISTORICA",
            "Nao existe uma sequencia historica dos documentos: existe a \
             impressao do conteudo, nao um numero de versao.",
            "DICHIARATO",
        ),
        (
            "DURABILIDADE_ARQUIVO",
            "A durabilidade do arquivo autoritativo depois de uma perda de \
             alimentacao nao e' verificada.",
            "NON_VERIFICATO",
        ),
        (
            "EXCLUSAO_FONTE",
            "A exclusao entre fonte e destino nao cobre hard links nem o mesmo \
             volume montado duas vezes.",
            "NON_VERIFICATO",
        ),
        (
            "RECUSAR_NAO_ATOMICO",
            "A recusa de um destino existente nao e' atomica contra quem \
             ignora o lucchetto cooperativo; a conservacao do anterior depende \
             dele.",
            "DICHIARATO",
        ),
    ];
    for (codigo, descricao, stato) in voci {
        folha.linhas.push(vec![
            Celula::texto(codigo),
            Celula::texto(descricao),
            Celula::texto(stato),
        ]);
    }
    folha
}

// ---------------------------------------------------------------------------
// Costruzione del modello
// ---------------------------------------------------------------------------

/// Costruisce il modello dell'export dalla vista revisionata.
///
/// Non calcola niente di contabile: riversa cio' che `prelivro` e `revisao`
/// hanno gia' composto, nell'ordine che hanno gia' fissato. L'unica aritmetica
/// e' il conteggio delle cifre significative per decidere se una cella
/// numerica di comodo si scrive.
pub fn modelo(
    vista: &VistaRevisada,
    proveniencia: &Proveniencia,
) -> Result<ModeloExport, ErroExport> {
    let folhas = vec![
        folha_proveniencia(vista, proveniencia)?,
        folha_diagnostico(vista)?,
        folha_rate(vista)?,
        folha_documentos(vista)?,
        folha_componentes(vista)?,
        folha_campos_rinviati(vista),
        folha_resumos(vista),
        folha_indicadores(&vista.pre_livro.indicadores),
        folha_extrato(vista)?,
        folha_exclusoes(vista)?,
        folha_decisoes(vista)?,
        folha_avisos(vista)?,
        folha_limites(),
    ];
    for folha in &folhas {
        validar_folha(folha)?;
    }
    Ok(ModeloExport { folhas })
}

/// Costruttori condivisi dai test di questo modulo e di
/// [`crate::export_io`]. Riusano le fixture dell'archivio: un lotto sintetico
/// non si costruisce due volte.
#[cfg(test)]
pub(crate) mod testes_comuns {
    use super::*;
    use crate::arquivo::testes_comuns as arq;
    use crate::prelivro::ContextoDocumento;
    use crate::regras::PlanoContas;
    use crate::revisao::{EventoRevisao, VistaRevisada};

    pub(crate) fn proveniencia() -> Proveniencia {
        Proveniencia {
            arquivo_caminho: "C:/dados/arquivo.json".to_string(),
            arquivo_revisao: 7,
            arquivo_sha256: "a".repeat(64),
            plano_arquivado: arq::identidade_plano(),
            plano_usado_versao: "1.0".to_string(),
            plano_usado_caminho: "config/plano_contas_gerencial.yaml".to_string(),
            gerado_em: arq::instante(9, -3 * 3600),
            divergencias: Vec::new(),
            eventos_no_arquivo: 2,
        }
    }

    pub(crate) fn documentos() -> Vec<ContextoDocumento> {
        vec![arq::um_documento(), arq::sem_linhas()]
    }

    pub(crate) fn plano() -> PlanoContas {
        arq::plano()
    }

    /// Vista con i documenti e gli eventi dati.
    pub(crate) fn vista(
        documentos: &[ContextoDocumento],
        eventos: &[EventoRevisao],
    ) -> VistaRevisada {
        crate::revisao::aplicar_revisao(documentos, &plano(), eventos).expect("la vista si compone")
    }

    /// Vista con un'approvazione umana su `2.01.004` del primo documento.
    pub(crate) fn vista_aprovada() -> VistaRevisada {
        let docs = documentos();
        let eventos = vec![arq::evento(
            &docs[0],
            "2.01.004",
            0,
            crate::revisao::OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        )];
        vista(&docs, &eventos)
    }
}

#[cfg(test)]
mod tests {
    use super::testes_comuns::*;
    use super::*;
    use crate::arquivo::testes_comuns as arq;
    use crate::revisao::OperacaoRevisao;

    fn celula_texto(folha: &Folha, linha: usize, coluna: &str) -> String {
        let j = folha
            .cabecalho
            .iter()
            .position(|c| *c == coluna)
            .unwrap_or_else(|| panic!("colonna {coluna} in {}", folha.nome));
        match &folha.linhas[linha][j] {
            Celula::Texto(t) => t.clone(),
            Celula::NumeroDeComodo(d) => d.to_string(),
            Celula::Vazia => String::new(),
        }
    }

    fn celula(folha: &Folha, linha: usize, coluna: &str) -> Celula {
        let j = folha
            .cabecalho
            .iter()
            .position(|c| *c == coluna)
            .unwrap_or_else(|| panic!("colonna {coluna} in {}", folha.nome));
        folha.linhas[linha][j].clone()
    }

    fn folha_de<'a>(m: &'a ModeloExport, nome: &str) -> &'a Folha {
        m.folha(nome).unwrap_or_else(|| panic!("foglio {nome}"))
    }

    // =================================================================
    // TX-01: lo schema e' il contratto
    // =================================================================

    #[test]
    fn tx01_treze_folhas_com_as_colunas_fixadas_uma_a_uma() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let nomes: Vec<&str> = modelo.folhas.iter().map(|f| f.nome).collect();
        assert_eq!(
            nomes,
            vec![
                "proveniencia",
                "diagnostico",
                "rate",
                "documentos",
                "reconciliacao_componentes",
                "reconciliacao_campos_rinviati",
                "resumos",
                "indicadores",
                "extrato_revisado",
                "exclusoes",
                "decisoes",
                "avisos",
                "limites",
            ],
            "tredici fogli, in quest'ordine"
        );

        // Le intestazioni, fissate qui invece di essere contate: una colonna
        // in piu' senza cambio di versione deve far fallire il test.
        assert_eq!(
            folha_de(&modelo, "limites").cabecalho,
            ["codigo", "descricao", "stato"]
        );
        assert_eq!(
            folha_de(&modelo, "indicadores").cabecalho,
            [
                "nome",
                "valor",
                "valor_numero",
                "valor_numero_status",
                "valor_numero_motivo",
                "tipo"
            ]
        );
        assert_eq!(
            folha_de(&modelo, "rate").cabecalho,
            [
                "lote",
                "chave_origem",
                "origem",
                "conta_id",
                "ordinal",
                "indice",
                "numero",
                "vencimento",
                "valor",
                "valor_numero",
                "valor_numero_status",
                "valor_numero_motivo"
            ]
        );
        assert_eq!(folha_de(&modelo, "diagnostico").cabecalho.len(), 50);
        assert_eq!(folha_de(&modelo, "decisoes").cabecalho.len(), 28);
        assert_eq!(folha_de(&modelo, "avisos").cabecalho.len(), 12);

        // Ogni riga ha esattamente le celle delle sue colonne.
        for folha in &modelo.folhas {
            for (i, linha) in folha.linhas.iter().enumerate() {
                assert_eq!(
                    linha.len(),
                    folha.cabecalho.len(),
                    "{}, riga {i}",
                    folha.nome
                );
            }
        }

        let prov = folha_de(&modelo, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "formato"), FORMATO);
        assert_eq!(celula_texto(prov, 0, "versao_schema"), "1");
    }

    // =================================================================
    // TX-02, TX-03: documenti senza righe, righe con lo stesso payload
    // =================================================================

    #[test]
    fn tx02_documento_sem_linhas_compare_em_documentos_e_non_in_diagnostico() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let documentos = folha_de(&modelo, "documentos");
        let diagnostico = folha_de(&modelo, "diagnostico");

        let senza = (0..documentos.linhas.len())
            .find(|i| celula_texto(documentos, *i, "chave_origem") == arq::CHAVE2)
            .expect("il documento senza righe compare in `documentos`");
        assert_eq!(celula_texto(documentos, senza, "linhas"), "0");
        // La riconciliazione resta intera: i quattro esiti ci sono.
        for esito in [
            "esito_total",
            "esito_rate",
            "esito_copertura",
            "esito_componenti",
        ] {
            assert!(
                !celula_texto(documentos, senza, esito).is_empty(),
                "{esito}"
            );
        }
        assert!(
            (0..diagnostico.linhas.len())
                .all(|i| celula_texto(diagnostico, i, "chave_origem") != arq::CHAVE2),
            "non ha righe, quindi non compare in `diagnostico`"
        );
    }

    #[test]
    fn tx03_duas_linhas_com_o_mesmo_payload_restam_duas() {
        let primeira = arq::lancamento(arq::CHAVE, "2.01.004", "10.00");
        let segunda = primeira.clone();
        let documento = arq::contexto(arq::CHAVE, "lote-2026-05-A", vec![primeira, segunda]);
        let docs = vec![documento];
        let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
        let diagnostico = folha_de(&modelo, "diagnostico");
        assert_eq!(diagnostico.linhas.len(), 2, "nessuno scarto");
        assert_ne!(
            celula_texto(diagnostico, 0, "ordinal"),
            celula_texto(diagnostico, 1, "ordinal"),
            "le distingue l'ordinale"
        );
    }

    // =================================================================
    // TX-04: importi, cella numerica, eleggibilita'
    // =================================================================

    #[test]
    fn tx04_texto_exato_e_celula_numerica_secondo_d26a() {
        for bruto in [
            // Il pre-libro della 3C.2 rifiuta una scala monetaria oltre i due
            // decimali dopo `normalize`: la quantita a quattro decimali vive
            // in un componente, non in `Lancamento::valor`.
            // `1234567890123.45` ha esattamente 15 cifre significative: e il
            // confine dell'eleggibilita', dal lato che passa.
            "0",
            "0.00",
            "0.01",
            "-7.50",
            "10.0000",
            "1234567890123.45",
        ] {
            let mut documento = arq::um_documento();
            documento.alocacao.lancamentos[0].valor = arq::dec(bruto);
            let docs = vec![documento];
            let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
            let d = folha_de(&modelo, "diagnostico");

            // Il testo autorevole: **esatto**, scala compresa.
            assert_eq!(celula_texto(d, 0, "valor"), bruto, "testo di {bruto}");
            // La cella numerica: per **valore**, e porta il Decimal.
            assert_eq!(celula_texto(d, 0, "valor_numero_status"), "OK");
            match celula(d, 0, "valor_numero") {
                Celula::NumeroDeComodo(v) => {
                    assert_eq!(v, arq::dec(bruto), "valore di {bruto}")
                }
                outra => panic!("{bruto}: cella inattesa {outra:?}"),
            }
        }
    }

    #[test]
    fn tx04_celula_numerica_assente_oltre_quindici_cifre() {
        // `12345678901234.56` ha **sedici** cifre significative: la regola
        // morde su un importo realistico, non solo su `Decimal::MAX`.
        for bruto in [
            "12345678901234.56",
            "79228162514264337593543950335",
            "-79228162514264337593543950335",
        ] {
            let mut documento = arq::um_documento();
            documento.alocacao.lancamentos[0].valor = arq::dec(bruto);
            let docs = vec![documento];
            let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
            let d = folha_de(&modelo, "diagnostico");

            assert_eq!(celula_texto(d, 0, "valor"), bruto, "il testo resta esatto");
            assert_eq!(celula(d, 0, "valor_numero"), Celula::Vazia);
            assert_eq!(
                celula_texto(d, 0, "valor_numero_status"),
                "NAO_REPRESENTAVEL"
            );
            assert!(
                celula_texto(d, 0, "valor_numero_motivo").contains("cifras"),
                "il motivo riporta le cifre"
            );
        }
    }

    #[test]
    fn tx04_nessuno_zero_tecnico_nella_cella_numerica() {
        // Una cella numerica con un valore qualunque deve essere impossibile:
        // il valore e' quello del testo, e questo test lo confronta.
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let d = folha_de(&modelo, "diagnostico");
        for i in 0..d.linhas.len() {
            if let Celula::NumeroDeComodo(v) = celula(d, i, "valor_numero") {
                assert_eq!(
                    v.to_string(),
                    celula_texto(d, i, "valor"),
                    "riga {i}: il numero e' lo stesso decimale del testo"
                );
            }
        }
    }

    #[test]
    fn tx05_quantidade_a_quatro_decimais_non_viene_arrotondata() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let c = folha_de(&modelo, "reconciliacao_componentes");
        let quantita = (0..c.linhas.len())
            .find(|i| celula_texto(c, *i, "nome") == "quantidade_media")
            .expect("il componente informativo compare");
        assert_eq!(celula_texto(c, quantita, "valor"), "1.2345");
    }

    // =================================================================
    // TX-06, TX-25: riepiloghi e indicatori copiati
    // =================================================================

    #[test]
    fn tx06_resumos_e_indicadores_coincidono_con_la_vista() {
        let vista = vista_aprovada();
        let modelo = modelo(&vista, &proveniencia()).unwrap();
        let resumos = folha_de(&modelo, "resumos");
        assert_eq!(resumos.linhas.len(), vista.pre_livro.resumos.len());
        for (i, r) in vista.pre_livro.resumos.iter().enumerate() {
            assert_eq!(celula_texto(resumos, i, "conta_id"), r.chave.conta_id);
            assert_eq!(
                celula_texto(resumos, i, "valor_total"),
                r.valor_total.to_string(),
                "nessuna somma ricalcolata"
            );
            assert_eq!(
                celula_texto(resumos, i, "qtd_total"),
                r.qtd_total.to_string()
            );
        }

        let indicadores = folha_de(&modelo, "indicadores");
        assert_eq!(indicadores.linhas.len(), 15, "quindici indicatori");
        let per_nome = |nome: &str| -> String {
            let i = (0..indicadores.linhas.len())
                .find(|i| celula_texto(indicadores, *i, "nome") == nome)
                .unwrap_or_else(|| panic!("indicatore {nome}"));
            celula_texto(indicadores, i, "valor")
        };
        let i = &vista.pre_livro.indicadores;
        assert_eq!(per_nome("documentos"), i.documentos.to_string());
        assert_eq!(per_nome("linhas"), i.linhas.to_string());
        assert_eq!(per_nome("valor_total"), i.valor_total.to_string());
        assert_eq!(
            per_nome("valor_extrato_aprovado"),
            i.valor_extrato_aprovado.to_string()
        );
    }

    #[test]
    fn tx25_documentos_marcados_compaiono_nel_resumo_e_in_avisos() {
        let vista = vista_aprovada();
        let modelo = modelo(&vista, &proveniencia()).unwrap();
        let resumos = folha_de(&modelo, "resumos");
        // I documenti della fixture sono `PARCIAL`.
        let marcados: Vec<String> = (0..resumos.linhas.len())
            .map(|i| celula_texto(resumos, i, "documentos_marcados"))
            .collect();
        assert!(
            marcados.iter().any(|m| !m.is_empty()),
            "almeno un riepilogo ha documenti marcati: {marcados:?}"
        );
        assert!(
            (0..resumos.linhas.len())
                .any(|i| celula_texto(resumos, i, "documentos_parcial") != "0")
        );

        let avisos = folha_de(&modelo, "avisos");
        assert!(
            (0..avisos.linhas.len())
                .any(|i| celula_texto(avisos, i, "fonte") == "RESUMO_DOCUMENTO_MARCADO"),
            "le stesse marcature sono interrogabili in `avisos`"
        );
    }

    // =================================================================
    // TX-07: liste annidate nei loro fogli
    // =================================================================

    #[test]
    fn tx07_rate_componenti_e_campi_hanno_il_loro_foglio_con_la_chiave() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();

        let rate = folha_de(&modelo, "rate");
        assert_eq!(rate.linhas.len(), 1, "una rata nel documento con righe");
        assert_eq!(celula_texto(rate, 0, "chave_origem"), arq::CHAVE);
        assert_eq!(celula_texto(rate, 0, "conta_id"), "2.01.004");
        assert_eq!(celula_texto(rate, 0, "indice"), "0");
        assert_eq!(celula_texto(rate, 0, "numero"), "001");
        assert_eq!(celula_texto(rate, 0, "vencimento"), "2026-05-30");

        let componentes = folha_de(&modelo, "reconciliacao_componentes");
        assert_eq!(componentes.linhas.len(), 4, "due componenti per documento");
        let campos = folha_de(&modelo, "reconciliacao_campos_rinviati");
        assert_eq!(campos.linhas.len(), 2, "un campo rinviato per documento");
        assert_eq!(celula_texto(campos, 0, "campo"), "centro_custo");
        // `CampoRinviato::status` e' un testo libero della 3B, non uno
        // `StatusDado`: si esporta come e'.
        assert_eq!(celula_texto(campos, 0, "status"), "NAO_DISPONIVEL");
    }

    #[test]
    fn tx07_rata_senza_vencimento_lascia_la_cella_vuota() {
        let mut documento = arq::um_documento();
        documento.alocacao.lancamentos[0].duplicatas[0].vencimento = None;
        let docs = vec![documento];
        let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
        let rate = folha_de(&modelo, "rate");
        assert_eq!(celula(rate, 0, "vencimento"), Celula::Vazia);
    }

    #[test]
    fn tx07_documento_senza_rate_non_produce_righe() {
        let mut documento = arq::um_documento();
        documento.alocacao.lancamentos[0].duplicatas.clear();
        let docs = vec![documento];
        let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
        assert!(
            folha_de(&modelo, "rate").linhas.is_empty(),
            "nessuna riga fittizia"
        );
    }

    // =================================================================
    // TX-08, TX-11, TX-13: identificativi, tempo, Dado
    // =================================================================

    #[test]
    fn tx08_identificadores_restam_testo_com_zeros() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let d = folha_de(&modelo, "diagnostico");
        assert_eq!(celula_texto(d, 0, "cnpj_cpf_favorecido"), "00012345678901");
        assert_eq!(celula_texto(d, 0, "chave_origem"), arq::CHAVE);
        assert_eq!(celula_texto(d, 0, "conta_id"), "2.01.004");
        for coluna in ["cnpj_cpf_favorecido", "chave_origem", "conta_id"] {
            assert!(
                matches!(celula(d, 0, coluna), Celula::Texto(_)),
                "{coluna} e' testo"
            );
        }
    }

    #[test]
    fn tx11_datas_iso_e_istanti_in_due_colonne() {
        let docs = testes_comuns::documentos();
        let mut evento = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            1, // offset di **un secondo**
        );
        evento.instante += chrono::Duration::nanoseconds(1);
        let modelo = modelo(&vista(&docs, &[evento]), &proveniencia()).unwrap();

        let d = folha_de(&modelo, "diagnostico");
        assert_eq!(celula_texto(d, 0, "data_documento"), "2026-05-05");
        assert_eq!(celula_texto(d, 0, "competencia"), "2026-05");
        // Il `Dado` assente: cella vuota con stato e motivo.
        assert_eq!(celula(d, 0, "data_pagamento"), Celula::Vazia);
        assert_eq!(
            celula_texto(d, 0, "data_pagamento_status"),
            "NAO_DISPONIVEL"
        );
        assert!(!celula_texto(d, 0, "data_pagamento_motivo").is_empty());

        let decisoes = folha_de(&modelo, "decisoes");
        assert_eq!(celula_texto(decisoes, 0, "instante_offset_segundos"), "1");
        assert!(
            celula_texto(decisoes, 0, "instante_local").ends_with(".000000001"),
            "il nanosecondo sopravvive: {}",
            celula_texto(decisoes, 0, "instante_local")
        );
    }

    #[test]
    fn tx13_dado_presente_assente_e_zero_restam_distinti() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let documentos = folha_de(&modelo, "documentos");
        // `soma_rate_documento` e' presente nella fixture, `articoli_attesi` no.
        assert!(!celula_texto(documentos, 0, "soma_rate_documento").is_empty());
        assert_eq!(
            celula_texto(documentos, 0, "soma_rate_documento_status"),
            "OK"
        );
        assert_eq!(celula(documentos, 0, "articoli_attesi"), Celula::Vazia);
        assert_eq!(
            celula_texto(documentos, 0, "articoli_attesi_status"),
            "NAO_DISPONIVEL"
        );
        assert!(!celula_texto(documentos, 0, "articoli_attesi_motivo").is_empty());
        // Uno zero reale resta uno zero.
        let senza = (0..documentos.linhas.len())
            .find(|i| celula_texto(documentos, *i, "chave_origem") == arq::CHAVE2)
            .unwrap();
        assert_eq!(celula_texto(documentos, senza, "valor_total_nfe"), "0.00");
    }

    // =================================================================
    // TX-21, TX-24: stato della revisione e attribuzione
    // =================================================================

    #[test]
    fn tx21_aprovado_por_e_quem_apre_lapprovazione_attiva() {
        let docs = testes_comuns::documentos();
        let alice = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        let mut bob = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            13,
            -3 * 3600,
        );
        bob.autor = "Bob".to_string();
        let mut carla = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Revogar,
            14,
            -3 * 3600,
        );
        carla.autor = "Carla".to_string();
        let mut dan = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            15,
            -3 * 3600,
        );
        dan.autor = "Dan".to_string();
        let mut eve = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            16,
            -3 * 3600,
        );
        eve.autor = "Eve".to_string();

        // Due approvazioni di persone diverse: la seconda non sostituisce.
        for ordem in [
            vec![alice.clone(), bob.clone()],
            vec![bob.clone(), alice.clone()],
        ] {
            let m = modelo(&vista(&docs, &ordem), &proveniencia()).unwrap();
            let e = folha_de(&m, "extrato_revisado");
            assert_eq!(e.linhas.len(), 1);
            assert_eq!(celula_texto(e, 0, "aprovado_por"), alice.autor);
            assert_eq!(celula_texto(e, 0, "confirmacoes"), "1");
            assert_eq!(celula_texto(e, 0, "ultima_confirmacao_por"), "Bob");
        }

        // Approva, ripete, revoca, riapprova, ripete.
        let catena = vec![alice, bob, carla, dan.clone(), eve];
        let m = modelo(&vista(&docs, &catena), &proveniencia()).unwrap();
        let e = folha_de(&m, "extrato_revisado");
        assert_eq!(celula_texto(e, 0, "aprovado_por"), "Dan");
        assert_eq!(
            celula_texto(e, 0, "aprovado_em"),
            dan.instante
                .naive_local()
                .format("%Y-%m-%dT%H:%M:%S%.9f")
                .to_string()
        );
        assert_eq!(celula_texto(e, 0, "confirmacoes"), "1");
        assert_eq!(celula_texto(e, 0, "ultima_confirmacao_por"), "Eve");
    }

    #[test]
    fn tx21_estratto_contiene_solo_lapprovazione_umana_attiva() {
        let docs = testes_comuns::documentos();
        // Nessun evento: nessuna riga nell'estratto revisionato.
        let sem_eventos = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
        assert!(folha_de(&sem_eventos, "extrato_revisado").linhas.is_empty());

        // Approvata, poi revocata: fuori dall'estratto, e in `exclusoes`.
        let aprovar = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        let revogar = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Revogar,
            13,
            -3 * 3600,
        );
        let m = modelo(&vista(&docs, &[aprovar, revogar]), &proveniencia()).unwrap();
        let e = folha_de(&m, "extrato_revisado");
        assert!(e.linhas.is_empty(), "una revoca la esclude");
        let d = folha_de(&m, "diagnostico");
        assert_eq!(celula_texto(d, 0, "estado_revisao"), "APROVACAO_REVOGADA");
    }

    // =================================================================
    // TX-22: tutti gli eventi, senza scarti silenziosi (D21)
    // =================================================================

    #[test]
    fn tx22_eventi_di_altri_lotti_compaiono_come_alvo_ausente() {
        let docs = testes_comuns::documentos();
        let mio = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        // Un evento che riguarda un documento assente da questo lotto.
        let mut altrove = mio.clone();
        altrove.alvo.chave_origem = "29260512345678000195550010009999912345678901".to_string();

        let vista = vista(&docs, &[mio, altrove]);
        let modelo = modelo(&vista, &proveniencia()).unwrap();
        let decisoes = folha_de(&modelo, "decisoes");
        assert_eq!(decisoes.linhas.len(), 2, "nessuno scartato in silenzio");

        let aplicados: Vec<usize> = (0..decisoes.linhas.len())
            .filter(|i| celula_texto(decisoes, *i, "aplicado") == "SIM")
            .collect();
        let nao: Vec<usize> = (0..decisoes.linhas.len())
            .filter(|i| celula_texto(decisoes, *i, "aplicado") == "NAO")
            .collect();
        assert_eq!(aplicados.len(), 1);
        assert_eq!(nao.len(), 1);
        assert_eq!(
            celula_texto(decisoes, nao[0], "motivo_nao_aplicacao"),
            "ALVO_AUSENTE"
        );
        // L'evento non applicato non ha una riga a cui legarsi.
        assert_eq!(celula(decisoes, nao[0], "lote"), Celula::Vazia);
        assert_ne!(celula(decisoes, aplicados[0], "lote"), Celula::Vazia);

        // I conteggi di `proveniencia` tornano con le righe.
        let prov = folha_de(&modelo, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "eventos_aplicados"), "1");
        assert_eq!(celula_texto(prov, 0, "eventos_nao_aplicados"), "1");
    }

    // =================================================================
    // TX-35: la regressione del riscontro R3C4-01
    // =================================================================

    #[test]
    fn tx35_o_dado_do_alvo_tem_quatro_colunas() {
        let docs = testes_comuns::documentos();
        let impressao = crate::impressao::impressao_conteudo(&docs[0]);
        let mut sem = arq::evento(
            &docs[0],
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        sem.alvo.impressao_conteudo = crate::modelos::Dado::novo(
            Some(impressao.clone()),
            "revisao:conteudo",
            crate::modelos::StatusDado::Ok,
            None,
        )
        .unwrap();
        let mut com = sem.clone();
        com.alvo.impressao_conteudo = crate::modelos::Dado::novo(
            Some(impressao),
            "revisao:conteudo",
            crate::modelos::StatusDado::Ok,
            Some("motivo dichiarato".to_string()),
        )
        .unwrap();

        let modelo_sem = modelo(&vista(&docs, &[sem]), &proveniencia()).unwrap();
        let modelo_com = modelo(&vista(&docs, &[com]), &proveniencia()).unwrap();
        let riga = |m: &ModeloExport| folha_de(m, "decisoes").linhas[0].clone();
        assert_ne!(
            riga(&modelo_sem),
            riga(&modelo_com),
            "le due righe si distinguono"
        );
        assert_eq!(
            celula(
                folha_de(&modelo_sem, "decisoes"),
                0,
                "alvo_impressao_conteudo_motivo"
            ),
            Celula::Vazia
        );
        assert_eq!(
            celula_texto(
                folha_de(&modelo_com, "decisoes"),
                0,
                "alvo_impressao_conteudo_motivo"
            ),
            "motivo dichiarato"
        );
    }

    // =================================================================
    // TX-29, TX-30: rappresentabilita' e limiti
    // =================================================================

    #[test]
    fn tx29_carattere_non_rappresentabile_e_un_errore_nominato() {
        for (controllo, codepoint) in [('\u{0}', 0u32), ('\u{1}', 1), ('\u{1F}', 31), ('\u{B}', 11)]
        {
            let docs = testes_comuns::documentos();
            let mut evento = arq::evento(
                &docs[0],
                "2.01.004",
                0,
                OperacaoRevisao::Anotar,
                12,
                -3 * 3600,
            );
            evento.autor = format!("Jos{controllo}e");
            let vista = vista(&docs, &[evento]);
            match modelo(&vista, &proveniencia()).unwrap_err() {
                ErroExport::TextoNaoRepresentavel {
                    folha,
                    coluna,
                    codepoint: lido,
                    ..
                } => {
                    assert_eq!(folha, "decisoes");
                    assert_eq!(coluna, "autor");
                    assert_eq!(lido, codepoint);
                }
                outro => panic!("U+{codepoint:04X}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tx28_tab_lf_e_cr_sobrevivem() {
        let mut documento = arq::um_documento();
        documento.alocacao.lancamentos[0].observacoes = "a\tb\nc\rd".to_string();
        let docs = vec![documento];
        let modelo = modelo(&vista(&docs, &[]), &proveniencia()).unwrap();
        let d = folha_de(&modelo, "diagnostico");
        assert_eq!(celula_texto(d, 0, "observacoes"), "a\tb\nc\rd");
    }

    #[test]
    fn tx30_testo_al_limite_passa_e_oltre_il_limite_e_un_errore() {
        for (caracteres, deve_passar) in [
            (LIMITE_CARACTERES_CELULA, true),
            (LIMITE_CARACTERES_CELULA + 1, false),
        ] {
            let docs = testes_comuns::documentos();
            let mut evento = arq::evento(
                &docs[0],
                "2.01.004",
                0,
                OperacaoRevisao::Anotar,
                12,
                -3 * 3600,
            );
            evento.nota = Some("n".repeat(caracteres));
            let vista = vista(&docs, &[evento]);
            let esito = modelo(&vista, &proveniencia());
            if deve_passar {
                assert!(esito.is_ok(), "{caracteres} caratteri devono passare");
            } else {
                match esito.unwrap_err() {
                    ErroExport::TextoTroppoLungo {
                        coluna,
                        caracteres: lido,
                        limite,
                        ..
                    } => {
                        assert_eq!(coluna, "nota");
                        assert_eq!(lido, caracteres);
                        assert_eq!(limite, LIMITE_CARACTERES_CELULA);
                    }
                    outro => panic!("erro inesperado: {outro}"),
                }
            }
        }
    }

    // =================================================================
    // TX-32, TX-36: identificativi lunghi e identita' del piano
    // =================================================================

    #[test]
    fn tx32_revisao_do_arquivo_e_testo_esatto() {
        let mut p = proveniencia();
        p.arquivo_revisao = 9_007_199_254_740_993; // oltre le 15 cifre
        let modelo = modelo(&vista_aprovada(), &p).unwrap();
        let prov = folha_de(&modelo, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "arquivo_revisao"), "9007199254740993");
        assert!(
            matches!(celula(prov, 0, "arquivo_revisao"), Celula::Texto(_)),
            "e' testo, non un numero"
        );
    }

    #[test]
    fn tx36_due_identita_del_piano_e_la_loro_coincidenza() {
        let mut p = proveniencia();
        // Versione e percorso coincidono con quelli archiviati.
        let m = modelo(&vista_aprovada(), &p).unwrap();
        let prov = folha_de(&m, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "plano_arquivado_versao"), "1.0");
        assert_eq!(celula_texto(prov, 0, "plano_usado_versao"), "1.0");
        assert_eq!(celula_texto(prov, 0, "plano_coincide"), "SIM");

        // Solo la versione cambia.
        p.plano_usado_versao = "9.9".to_string();
        let m = modelo(&vista_aprovada(), &p).unwrap();
        let prov = folha_de(&m, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "plano_coincide"), "NAO");

        // Versione uguale, percorso diverso.
        let mut p = proveniencia();
        p.plano_usado_caminho = "outro/plano.yaml".to_string();
        let m = modelo(&vista_aprovada(), &p).unwrap();
        let prov = folha_de(&m, "proveniencia");
        assert_eq!(celula_texto(prov, 0, "plano_coincide"), "PARCIALE");
    }

    #[test]
    fn tx23_divergenza_di_apresentacao_non_blocca_e_ha_la_sua_riga() {
        let mut p = proveniencia();
        p.divergencias = vec![crate::arquivo::Divergencia {
            chave_origem: arq::CHAVE.to_string(),
            salva: "a".repeat(64),
            atual: "b".repeat(64),
        }];
        let modelo = modelo(&vista_aprovada(), &p).unwrap();
        let prov = folha_de(&modelo, "proveniencia");
        assert_eq!(prov.linhas.len(), 2, "metadato piu' una divergenza");
        assert_eq!(celula_texto(prov, 0, "apresentacao_divergente"), "SIM");
        assert_eq!(celula_texto(prov, 1, "tipo_linha"), "DIVERGENCIA");
        assert_eq!(celula_texto(prov, 1, "div_chave_origem"), arq::CHAVE);
        // Le colonne del metadato sono vuote nella riga di divergenza.
        assert_eq!(celula(prov, 1, "formato"), Celula::Vazia);
    }

    // =================================================================
    // Foglio dei limiti e sotto-archivio
    // =================================================================

    #[test]
    fn limites_dichiara_la_divergenza_nota_del_lettore() {
        let modelo = modelo(&vista_aprovada(), &proveniencia()).unwrap();
        let limites = folha_de(&modelo, "limites");
        let codici: Vec<String> = (0..limites.linhas.len())
            .map(|i| celula_texto(limites, i, "codigo"))
            .collect();
        for esperado in [
            "DERIVADO_SEM_RETORNO",
            "AUTOR_NAO_AUTENTICADO",
            "NUMERO_NAO_AUTORITATIVO",
            "ESCAPE_LEITOR",
            "EXCEL_NAO_ATESTADO",
            "SEM_SEQUENCIA_HISTORICA",
            "DURABILIDADE_ARQUIVO",
            "EXCLUSAO_FONTE",
            "RECUSAR_NAO_ATOMICO",
        ] {
            assert!(codici.contains(&esperado.to_string()), "manca {esperado}");
        }
        let i = codici.iter().position(|c| c == "ESCAPE_LEITOR").unwrap();
        assert_eq!(celula_texto(limites, i, "stato"), "DIVERGENZA_NOTA");
        assert!(
            celula_texto(limites, i, "descricao").contains("openpyxl 3.1.5"),
            "il limite nomina il lettore e la versione misurata"
        );
        let j = codici
            .iter()
            .position(|c| c == "EXCEL_NAO_ATESTADO")
            .unwrap();
        assert_eq!(celula_texto(limites, j, "stato"), "NON_VERIFICATO");
    }

    #[test]
    fn tx26_sub_arquivo_de_un_lote_inesistente_e_nomeado() {
        let arquivo = arq::arquivo_valido();
        match sub_arquivo_do_lote(&arquivo, "lote-que-nao-existe").unwrap_err() {
            ErroExport::LoteAusente {
                pedido,
                disponiveis,
            } => {
                assert_eq!(pedido, "lote-que-nao-existe");
                assert_eq!(disponiveis, vec!["lote-2026-05-A".to_string()]);
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn tx36_sub_arquivo_restringe_lo_scope_del_controllo_di_presentazione() {
        let arquivo = arq::arquivo_valido();
        let sub = sub_arquivo_do_lote(&arquivo, "lote-2026-05-A").unwrap();
        assert_eq!(sub.execucoes.len(), 1);
        assert_eq!(sub.snapshots.len(), 2, "i soli snapshot del lotto");
        assert_eq!(
            sub.eventos.len(),
            arquivo.eventos.len(),
            "gli eventi non si filtrano: D21"
        );
        // Il controllo di presentazione sul sotto-archivio riesce.
        let divergencias =
            crate::arquivo::verificar_apresentacao(&sub, &testes_comuns::plano()).unwrap();
        assert!(divergencias.is_empty());
    }

    #[test]
    fn cifras_significativas_conta_sulla_mantissa_normalizzata() {
        for (bruto, esperado) in [
            ("0", 1u32),
            ("0.00", 1),
            ("10.0000", 2),
            ("0.01", 1),
            ("-7.50", 2),
            ("12345678901234.56", 16),
            ("79228162514264337593543950335", 29),
        ] {
            assert_eq!(
                cifras_significativas(arq::dec(bruto)),
                esperado,
                "cifre di {bruto}"
            );
        }
    }

    // =================================================================
    // Regressioni dell'audit del 2026-10-30
    // =================================================================

    /// `R3C41-02`: il predicato ignorava U+FFFE e U+FFFF, che XML 1.0 non
    /// ammette fra i caratteri. Sono valori possibili in una `String` Rust, a
    /// differenza dei surrogati isolati, quindi il caso e' raggiungibile.
    #[test]
    fn r2_u_fffe_e_u_ffff_non_sono_rappresentabili() {
        for (controllo, codepoint) in [('\u{FFFE}', 0xFFFEu32), ('\u{FFFF}', 0xFFFF)] {
            let mut documento = arq::um_documento();
            documento.alocacao.lancamentos[0].descricao_itens = format!("a{controllo}b");
            let docs = vec![documento];
            let vista = vista(&docs, &[]);
            match modelo(&vista, &proveniencia()).unwrap_err() {
                ErroExport::TextoNaoRepresentavel {
                    folha,
                    coluna,
                    codepoint: lido,
                    posicao,
                    ..
                } => {
                    assert_eq!(folha, "diagnostico");
                    assert_eq!(coluna, "descricao_itens");
                    assert_eq!(lido, codepoint);
                    assert_eq!(posicao, 1, "la posizione e' quella del carattere");
                }
                outro => panic!("U+{codepoint:04X}: erro inesperado {outro}"),
            }
        }
    }

    /// Il rovescio: il predicato non deve vietare Unicode valido per eccesso.
    #[test]
    fn r2_unicode_valido_resta_ammesso() {
        for (nome, texto) in [
            ("tab_lf_cr", "a\tb\nc\rd"),
            ("fuori_dal_bmp", "clave de sol \u{1D11E}"),
            ("confine_fffd", "sostituzione \u{FFFD}"),
            ("confine_e000", "uso privato \u{E000}"),
            ("confine_d7ff", "limite \u{D7FF}"),
            ("non_carattere_fuori_bmp", "ammesso da XML 1.0 \u{1FFFE}"),
        ] {
            let mut documento = arq::um_documento();
            documento.alocacao.lancamentos[0].observacoes = texto.to_string();
            let docs = vec![documento];
            let vista = vista(&docs, &[]);
            let m = modelo(&vista, &proveniencia()).unwrap_or_else(|e| panic!("{nome}: {e}"));
            let d = folha_de(&m, "diagnostico");
            assert_eq!(celula_texto(d, 0, "observacoes"), texto, "{nome}");
        }
    }
}

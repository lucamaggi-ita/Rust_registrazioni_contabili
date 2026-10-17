//! Pre-libro gerenciale in memoria (fase 3C.2).
//!
//! Trasforma le allocazioni della fase 3B in un pre-libro: righe partizionate,
//! documenti conservati **con la loro riconciliazione**, riepiloghi per azienda,
//! competenza e conto, e le esclusioni motivate dell'estratto approvato.
//!
//! [`construir_pre_livro`] e' **pura**: non scrive su disco, non legge
//! workbook, non consulta archivi, non esegue transizioni di revisione e non
//! cancella avvisi.
//!
//! Contratti applicati qui (identificativi della specifica di fase 3C):
//!
//! - **F3C-01**: l'ingresso e' `AlocacaoNfe` piu' il contesto tipizzato, non un
//!   workbook.
//! - **F3C-02/07** e invarianti I1/I2: ogni riga ricevuta finisce in
//!   **esattamente una** partizione; nessuno scarto implicito.
//! - **F3C-03**: partizioni con precedenza dichiarata e **tutte** le cause
//!   conservate in `motivos`.
//! - **F3C-08/09** (decisione D13): riepiloghi per
//!   `(cnpj_empresa, competencia, conta_id)`; il nome del conto viene dal
//!   [`PlanoContas`] ricevuto come parametro, mai dalla riga; un `conta_id`
//!   assente dal piano e' un errore esplicito; i nomi di riga divergenti
//!   restano conservati e segnalati.
//! - **F3C-11** (decisione D10): diagnostico e riepiloghi conservano tutto,
//!   **marcato**: ogni riepilogo dice da quanti documenti `OK`, `PARCIAL` e
//!   `ERRO` arriva, e nomina quelli non `OK`. L'estratto approvato contiene
//!   solo righe `APROVADO` di documenti non `ERRO`, e ogni esclusione e'
//!   elencata con identita', valore, stato e motivo.
//! - **F3C-12**: la [`Reconciliacao`] del documento e' conservata integra, non
//!   appiattita in colonne di riga.
//! - **F3C-25** e invariante I7: ordinamento deterministico, indifferente
//!   all'ordine d'ingresso. Vale anche per i **contenitori aggregati**: avvisi
//!   del lotto e nomi di conto divergenti hanno un ordine canonico, non quello
//!   di arrivo.
//! - **F3C-26**: tutte le somme passano per [`crate::dinheiro`], in centesimi
//!   interi; un aggregato non rappresentabile e' un errore, non un troncamento,
//!   e **non** si produce un pre-libro parziale.
//! - **F3C-27** e invariante I8: ogni documento del lotto compare con la
//!   propria identita' **anche senza righe**.
//!
//! # Coerenza fra contesto e allocazione
//!
//! `ContextoDocumento` porta l'identita' del documento e, separatamente, il
//! risultato della 3B. I tipi non garantiscono che i due oggetti parlino della
//! stessa NF-e: un abbinamento sbagliato attribuirebbe il valore di un
//! documento all'azienda e al mese di un altro. Riscontro R3C2-01 dell'audit
//! del 2026-10-17. Per questo i campi ripetuti vengono **confrontati** prima di
//! aggregare, e un disaccordo e' [`ErroPreLivro::DocumentoIncoerente`]:
//!
//! | Campo confrontato | Contesto | Registrazione |
//! |---|---|---|
//! | chiave NF-e | `chave_origem` | `Lancamento::chave_origem` |
//! | numero del documento | `numero` | `Lancamento::documento` |
//! | data del documento | `data_documento` | `Lancamento::data_documento` |
//! | competenza | `competencia` | `Lancamento::competencia` |
//! | azienda | `cnpj_empresa` | `Lancamento::cnpj_empresa` |
//!
//! Non vengono confrontati i campi che non identificano il documento
//! (`serie`, `nome_empresa`, `arquivo_origem`): restano del contesto. Le righe
//! **non** vengono mai riscritte per adattarle al contesto, e un documento
//! senza registrazioni usa il contesto della NF-e senza inventare una riga.
//!
//! Fuori da questo blocco, per contratto: ingressi grezzi (importi illeggibili,
//! stati sconosciuti), archivio, esportazione, import dello storico, CLI e
//! transizioni di revisione umana. Le partizioni `INCOMPLETO` e
//! `NAO_RECONHECIDO` della specifica nascono con gli adattatori della 3C.4:
//! qui non sono rappresentabili, perche' `Lancamento::valor` e' un `Decimal`
//! sempre valido e `StatusRevisao` e' un enum chiuso.

use std::collections::BTreeMap;
use std::fmt;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::alocacao::{AlocacaoNfe, Reconciliacao, StatoDocumento};
use crate::dinheiro::{self, ErroDinheiro};
use crate::modelos::{Competencia, Lancamento, OrigemLancamento, StatusRevisao};
use crate::nfe::Aviso;
use crate::regras::PlanoContas;

/// Etichetta della partizione delle aziende non identificate.
///
/// Non e' un CNPJ e non puo' collidere con uno: le righe senza azienda non
/// vengono mai assimilate a un'azienda identificata.
pub const EMPRESA_NAO_IDENTIFICADA: &str = "EMPRESA_NAO_IDENTIFICADA";

/// Codici di avviso prodotti da questo modulo.
pub mod avisos {
    /// Lo stesso `conta_id` arriva con un nome diverso da quello del piano.
    pub const NOME_DE_CONTA_DIVERGENTE: &str = "NOME_DE_CONTA_DIVERGENTE";
    /// Il documento non ha prodotto registrazioni, ma resta nel pre-libro.
    pub const DOCUMENTO_SEM_LINHAS: &str = "DOCUMENTO_SEM_LINHAS";
    /// Il documento non identifica l'azienda acquirente.
    pub const EMPRESA_SEM_IDENTIFICACAO: &str = "EMPRESA_SEM_IDENTIFICACAO";
}

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore che impedisce di produrre il pre-libro.
///
/// Nessuno di questi casi produce un pre-libro parziale: o il lotto e'
/// calcolabile interamente, o si restituisce l'errore.
#[derive(Debug)]
pub enum ErroPreLivro {
    /// La chiave del documento e' vuota: senza di essa la riga non e'
    /// identificabile (C22).
    ChaveDocumentoVazia { arquivo_origem: String },
    /// Due documenti con la stessa chiave nello stesso lotto: in 3C.2 non
    /// esiste ancora il versionamento che li distinguerebbe.
    DocumentoDuplicadoNoLote { chave_origem: String },
    /// I documenti dichiarano identificatori di lotto diversi.
    LotesDivergentes {
        esperado: String,
        encontrado: String,
        chave_origem: String,
    },
    /// Stesso identificatore di lotto, metadati diversi: conflitto esplicito,
    /// non si sceglie il primo ne' si scarta un valore in silenzio.
    MetadadosDoLoteDivergentes {
        lote: String,
        campo: &'static str,
        esperado: String,
        encontrado: String,
        chave_origem: String,
    },
    /// Il contesto e la registrazione non parlano dello stesso documento.
    DocumentoIncoerente {
        chave_origem: String,
        campo: &'static str,
        no_contexto: String,
        no_lancamento: String,
    },
    /// Il conto della registrazione non esiste nel piano ricevuto.
    ContaNaoEncontradaNoPlano {
        chave_origem: String,
        conta_id: String,
    },
    /// Somma o conversione non rappresentabile: nessun troncamento.
    CapacidadeNumericaExcedida {
        contexto: String,
        operacao: &'static str,
        detalhe: String,
    },
    /// Un importo in ingresso non e' un multiplo esatto di 0.01.
    EscalaMonetariaInvalida {
        contexto: String,
        campo: String,
        valor: Decimal,
    },
}

impl fmt::Display for ErroPreLivro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroPreLivro::ChaveDocumentoVazia { arquivo_origem } => write!(
                f,
                "documento sem chave de origem ({arquivo_origem}): \
                 sem ela as linhas nao sao identificaveis"
            ),
            ErroPreLivro::DocumentoDuplicadoNoLote { chave_origem } => write!(
                f,
                "documento repetido no mesmo lote: {chave_origem}. \
                 O versionamento de documentos pertence a um bloco posterior"
            ),
            ErroPreLivro::LotesDivergentes {
                esperado,
                encontrado,
                chave_origem,
            } => write!(
                f,
                "lotes divergentes no mesmo pre-livro: esperado {esperado}, \
                 encontrado {encontrado} em {chave_origem}"
            ),
            ErroPreLivro::MetadadosDoLoteDivergentes {
                lote,
                campo,
                esperado,
                encontrado,
                chave_origem,
            } => write!(
                f,
                "lote {lote}: metadados divergentes em {campo} \
                 (esperado {esperado}, encontrado {encontrado} em {chave_origem}): \
                 nenhum valor e' escolhido nem descartado em silencio"
            ),
            ErroPreLivro::DocumentoIncoerente {
                chave_origem,
                campo,
                no_contexto,
                no_lancamento,
            } => write!(
                f,
                "documento {chave_origem}: contexto e lancamento discordam em {campo} \
                 (contexto {no_contexto}, lancamento {no_lancamento}): \
                 o valor nao e' agregado e nenhuma linha e' reescrita"
            ),
            ErroPreLivro::ContaNaoEncontradaNoPlano {
                chave_origem,
                conta_id,
            } => write!(
                f,
                "conta {conta_id} do documento {chave_origem} nao existe no plano de contas: \
                 nenhum nome e' inventado nem recuperado da ordem de entrada"
            ),
            ErroPreLivro::CapacidadeNumericaExcedida {
                contexto,
                operacao,
                detalhe,
            } => write!(
                f,
                "{contexto}: capacidade numerica excedida em {operacao}: {detalhe}. \
                 Nenhum centavo foi descartado e nenhum pre-livro parcial foi produzido"
            ),
            ErroPreLivro::EscalaMonetariaInvalida {
                contexto,
                campo,
                valor,
            } => write!(
                f,
                "{contexto}: valor monetario fora do dominio em {campo}: {valor} \
                 nao e' multiplo exato de 0.01"
            ),
        }
    }
}

impl std::error::Error for ErroPreLivro {}

fn de_erro_dinheiro(erro: ErroDinheiro, contexto: &str, campo: &str) -> ErroPreLivro {
    match erro {
        ErroDinheiro::EscalaInvalida { valor } => ErroPreLivro::EscalaMonetariaInvalida {
            contexto: contexto.to_string(),
            campo: campo.to_string(),
            valor,
        },
        ErroDinheiro::CapacidadeExcedida { operacao, detalhe } => {
            ErroPreLivro::CapacidadeNumericaExcedida {
                contexto: contexto.to_string(),
                operacao,
                detalhe,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Ingresso
// ---------------------------------------------------------------------------

/// Identita' dell'esecuzione che ha prodotto i documenti.
///
/// Dichiarata da chi invoca: non viene dedotta dall'ambiente, dall'utente del
/// sistema operativo o dall'orologio. Tutti i documenti di un pre-libro devono
/// portarne una **identica**, metadati compresi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentidadeLote {
    pub id: String,
    pub gerado_em: NaiveDate,
    pub descricao: String,
}

/// Un documento allocato, con l'identita' che la 3B **non** porta.
///
/// `Reconciliacao` non contiene chiave, azienda ne' competenza, e un documento
/// senza registrazioni non ha nemmeno una riga da cui dedurle: per questo
/// l'identita' arriva dal contesto, costruito da chi ha letto la NF-e. I campi
/// ripetuti fra contesto e registrazioni vengono confrontati, vedi la nota di
/// modulo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextoDocumento {
    pub chave_origem: String,
    pub numero: String,
    pub serie: String,
    pub data_documento: NaiveDate,
    pub competencia: Competencia,
    /// Vuoto = azienda non identificata: partizione dedicata, mai unita a
    /// un'azienda vera.
    pub cnpj_empresa: String,
    pub nome_empresa: String,
    pub arquivo_origem: String,
    pub alocacao: AlocacaoNfe,
    /// Avvisi della fase 2, conservati come sono.
    pub avisos_leitura: Vec<Aviso>,
    pub lote: IdentidadeLote,
}

// ---------------------------------------------------------------------------
// Uscita
// ---------------------------------------------------------------------------

/// Partizione della riga nel pre-libro.
///
/// Precedenza dichiarata della specifica:
/// `INCOMPLETO > NAO_RECONHECIDO > REVISAR > APROVADO > OK`. In 3C.2 solo le
/// ultime tre sono raggiungibili: le prime due richiedono un ingresso grezzo,
/// che questo blocco non accetta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Particao {
    Revisar,
    Aprovado,
    Ok,
}

impl Particao {
    pub const fn rotulo(self) -> &'static str {
        match self {
            Particao::Revisar => "REVISAR",
            Particao::Aprovado => "APROVADO",
            Particao::Ok => "OK",
        }
    }
}

/// Identita' di una riga del pre-libro.
///
/// Scope: la tupla `(origem, conta_id, ordinal)` e' unica **dentro un
/// documento**; con `lote` e `chave_origem` diventa unica nel lotto. La
/// `versao` del documento, necessaria per lo storico, nasce con il blocco che
/// introduce l'archivio: qui non esiste e non viene finta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentidadeLinha {
    pub lote: String,
    pub chave_origem: String,
    pub origem: OrigemLancamento,
    pub conta_id: String,
    pub ordinal: u32,
}

/// Riga del pre-libro: la registrazione, la sua partizione e il suo contesto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinhaPreLivro {
    pub identidade: IdentidadeLinha,
    /// La registrazione della 3B, **invariata**: nessun avviso cancellato,
    /// nessuno stato riscritto.
    pub lancamento: Lancamento,
    pub particao: Particao,
    /// Tutte le cause che hanno portato alla partizione, nessuna perduta.
    pub motivos: Vec<String>,
    /// Chiave del riepilogo a cui questa riga contribuisce: una sola fonte per
    /// azienda, competenza e conto.
    pub chave_resumo: ResumoChave,
    /// Nome del conto secondo il piano, che e' la fonte autorevole.
    pub conta_nome_plano: String,
    pub empresa_identificada: bool,
    /// Stato del documento a cui la riga appartiene: viaggia con la riga.
    pub stato_documento: StatoDocumento,
    /// D10(a): solo righe `APROVADO` di documenti non `ERRO`.
    pub no_extrato_aprovado: bool,
}

/// Documento del lotto, con la riconciliazione conservata integra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentoPreLivro {
    pub lote: String,
    pub chave_origem: String,
    pub numero: String,
    pub serie: String,
    pub data_documento: NaiveDate,
    pub competencia: Competencia,
    pub cnpj_empresa: String,
    pub nome_empresa: String,
    pub arquivo_origem: String,
    pub empresa_identificada: bool,
    pub stato_documento: StatoDocumento,
    pub linhas: usize,
    /// Quattro esiti, componenti, campi rinviati e avvisi del documento.
    pub reconciliacao: Reconciliacao,
    pub avisos_leitura: Vec<Aviso>,
}

/// Chiave di aggregazione: azienda, competenza, conto (decisione D13).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResumoChave {
    /// CNPJ dell'azienda, oppure [`EMPRESA_NAO_IDENTIFICADA`].
    pub cnpj_empresa: String,
    pub competencia: Competencia,
    pub conta_id: String,
}

/// Riepilogo di una chiave, con i conteggi e i valori per partizione.
///
/// Porta anche lo **stato dei documenti** che lo alimentano: senza questo, un
/// gruppo nutrito da un documento `PARCIAL` o `ERRO` sarebbe indistinguibile da
/// uno sano, che e' il riscontro R3C2-02 dell'audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resumo {
    pub chave: ResumoChave,
    /// Nome dal piano dei conti, non dalla riga.
    pub conta_nome: String,
    pub grupo_conta: String,
    pub empresa_identificada: bool,
    pub qtd_aprovado: usize,
    pub qtd_ok: usize,
    pub qtd_revisar: usize,
    pub qtd_total: usize,
    pub valor_aprovado: Decimal,
    pub valor_ok: Decimal,
    pub valor_revisar: Decimal,
    pub valor_total: Decimal,
    /// Documenti distinti che alimentano il gruppo, per stato.
    pub documentos_ok: usize,
    pub documentos_parcial: usize,
    pub documentos_erro: usize,
    /// Chiavi dei documenti **non** `OK` che alimentano il gruppo, ordinate.
    /// Nessun gruppo viene marcato per un problema di un documento estraneo.
    pub documentos_marcados: Vec<String>,
    /// Nomi di conto visti nelle righe e diversi da quello del piano, ordinati.
    pub nomes_divergentes: Vec<String>,
}

/// Riga esclusa dall'estratto approvato, con il motivo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exclusao {
    pub identidade: IdentidadeLinha,
    pub valor: Decimal,
    pub particao: Particao,
    pub stato_documento: StatoDocumento,
    pub motivo: String,
}

/// Indicatori del lotto. Conteggi e somme, nessuna percentuale inventata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Indicadores {
    pub documentos: usize,
    pub documentos_ok: usize,
    pub documentos_parcial: usize,
    pub documentos_erro: usize,
    pub documentos_sem_linhas: usize,
    pub linhas: usize,
    pub linhas_aprovado: usize,
    pub linhas_ok: usize,
    pub linhas_revisar: usize,
    pub valor_total: Decimal,
    pub valor_aprovado: Decimal,
    pub valor_ok: Decimal,
    pub valor_revisar: Decimal,
    pub valor_extrato_aprovado: Decimal,
    pub linhas_excluidas_do_extrato: usize,
}

/// Pre-libro di un lotto: righe, documenti, riepiloghi, indicatori, esclusioni.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreLivro {
    /// `None` solo per un lotto vuoto: nessuna identita' viene inventata.
    pub lote: Option<IdentidadeLote>,
    pub linhas: Vec<LinhaPreLivro>,
    pub documentos: Vec<DocumentoPreLivro>,
    pub resumos: Vec<Resumo>,
    pub indicadores: Indicadores,
    pub exclusoes_extrato: Vec<Exclusao>,
    /// Avvisi prodotti dal pre-libro, piu' quelli di lettura propagati, in
    /// ordine canonico per codice e dettaglio: l'ordine d'ingresso non deve
    /// cambiare il risultato (I7).
    pub avisos: Vec<Aviso>,
}

impl PreLivro {
    /// Vista dell'estratto approvato (D10a): solo righe `APROVADO` di documenti
    /// non `ERRO`. E' una **vista in memoria**, non una contabilizzazione.
    pub fn extrato_aprovado(&self) -> Vec<&LinhaPreLivro> {
        self.linhas
            .iter()
            .filter(|linha| linha.no_extrato_aprovado)
            .collect()
    }

    /// Righe in una partizione, nell'ordine deterministico del pre-libro.
    pub fn linhas_na_particao(&self, particao: Particao) -> Vec<&LinhaPreLivro> {
        self.linhas
            .iter()
            .filter(|linha| linha.particao == particao)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Costruzione
// ---------------------------------------------------------------------------

/// Accumulatore interno di un riepilogo, in centesimi interi.
struct AcumuladorResumo {
    conta_nome: String,
    grupo_conta: String,
    empresa_identificada: bool,
    qtd_aprovado: usize,
    qtd_ok: usize,
    qtd_revisar: usize,
    cent_aprovado: i128,
    cent_ok: i128,
    cent_revisar: i128,
    /// Documenti distinti che alimentano il gruppo, con il loro stato.
    documentos: BTreeMap<String, StatoDocumento>,
    nomes_divergentes: Vec<String>,
}

/// Costruisce il pre-libro di un lotto di documenti allocati.
///
/// Il `plano` e' la fonte autorevole dei nomi dei conti: una funzione pura non
/// puo' leggerlo altrove, quindi lo riceve come parametro. Chi invoca lo carica
/// e lo valida fuori, con la stessa validazione degli altri comandi.
pub fn construir_pre_livro(
    documentos: &[ContextoDocumento],
    plano: &PlanoContas,
) -> Result<PreLivro, ErroPreLivro> {
    if documentos.is_empty() {
        return Ok(PreLivro {
            lote: None,
            linhas: Vec::new(),
            documentos: Vec::new(),
            resumos: Vec::new(),
            indicadores: Indicadores {
                documentos: 0,
                documentos_ok: 0,
                documentos_parcial: 0,
                documentos_erro: 0,
                documentos_sem_linhas: 0,
                linhas: 0,
                linhas_aprovado: 0,
                linhas_ok: 0,
                linhas_revisar: 0,
                valor_total: Decimal::ZERO,
                valor_aprovado: Decimal::ZERO,
                valor_ok: Decimal::ZERO,
                valor_revisar: Decimal::ZERO,
                valor_extrato_aprovado: Decimal::ZERO,
                linhas_excluidas_do_extrato: 0,
            },
            exclusoes_extrato: Vec::new(),
            avisos: Vec::new(),
        });
    }

    let lote = documentos[0].lote.clone();
    let mut chaves_vistas: Vec<&str> = Vec::new();
    let mut linhas: Vec<LinhaPreLivro> = Vec::new();
    let mut docs: Vec<DocumentoPreLivro> = Vec::new();
    let mut avisos: Vec<Aviso> = Vec::new();
    let mut resumos: BTreeMap<ResumoChave, AcumuladorResumo> = BTreeMap::new();

    for contexto in documentos {
        if contexto.chave_origem.trim().is_empty() {
            return Err(ErroPreLivro::ChaveDocumentoVazia {
                arquivo_origem: contexto.arquivo_origem.clone(),
            });
        }
        verificar_lote(&lote, contexto)?;
        if chaves_vistas.contains(&contexto.chave_origem.as_str()) {
            return Err(ErroPreLivro::DocumentoDuplicadoNoLote {
                chave_origem: contexto.chave_origem.clone(),
            });
        }
        chaves_vistas.push(&contexto.chave_origem);

        let empresa_identificada = !contexto.cnpj_empresa.trim().is_empty();
        let chave_empresa = if empresa_identificada {
            contexto.cnpj_empresa.trim().to_string()
        } else {
            EMPRESA_NAO_IDENTIFICADA.to_string()
        };
        if !empresa_identificada {
            avisos.push(aviso(
                avisos::EMPRESA_SEM_IDENTIFICACAO,
                format!(
                    "documento {}: empresa sem CNPJ; agrupada em {EMPRESA_NAO_IDENTIFICADA}",
                    contexto.chave_origem
                ),
            ));
        }

        let reconciliacao = &contexto.alocacao.reconciliacao;
        let stato_documento = reconciliacao.stato_documento;

        if contexto.alocacao.lancamentos.is_empty() {
            avisos.push(aviso(
                avisos::DOCUMENTO_SEM_LINHAS,
                format!(
                    "documento {} sem lancamentos: conservado no pre-livro com estado {:?}",
                    contexto.chave_origem, stato_documento
                ),
            ));
        }

        for (indice, lancamento) in contexto.alocacao.lancamentos.iter().enumerate() {
            verificar_coerencia(contexto, lancamento)?;

            let conta = plano.por_id(&lancamento.conta_id).ok_or_else(|| {
                ErroPreLivro::ContaNaoEncontradaNoPlano {
                    chave_origem: contexto.chave_origem.clone(),
                    conta_id: lancamento.conta_id.clone(),
                }
            })?;

            let mut motivos = motivos_da_linha(lancamento);
            let nome_divergente = conta.nome != lancamento.conta_nome;
            if nome_divergente {
                let detalhe = format!(
                    "conta {}: plano diz {:?}, lancamento diz {:?}",
                    lancamento.conta_id, conta.nome, lancamento.conta_nome
                );
                let aviso_divergencia = aviso(avisos::NOME_DE_CONTA_DIVERGENTE, detalhe);
                if !avisos.contains(&aviso_divergencia) {
                    avisos.push(aviso_divergencia);
                }
                motivos.push(avisos::NOME_DE_CONTA_DIVERGENTE.to_string());
            }

            let particao = particao_de(lancamento.status_revisao);
            let no_extrato_aprovado =
                particao == Particao::Aprovado && stato_documento != StatoDocumento::Erro;

            let contexto_erro = format!("documento {}", contexto.chave_origem);
            let campo = format!("lancamento {}/valor", lancamento.conta_id);
            let cent = dinheiro::em_centesimos(lancamento.valor, &campo)
                .map_err(|e| de_erro_dinheiro(e, &contexto_erro, &campo))?;

            let chave = ResumoChave {
                cnpj_empresa: chave_empresa.clone(),
                competencia: contexto.competencia,
                conta_id: lancamento.conta_id.clone(),
            };
            let acumulador = resumos
                .entry(chave.clone())
                .or_insert_with(|| AcumuladorResumo {
                    conta_nome: conta.nome.clone(),
                    grupo_conta: conta.grupo.clone(),
                    empresa_identificada,
                    qtd_aprovado: 0,
                    qtd_ok: 0,
                    qtd_revisar: 0,
                    cent_aprovado: 0,
                    cent_ok: 0,
                    cent_revisar: 0,
                    documentos: BTreeMap::new(),
                    nomes_divergentes: Vec::new(),
                });
            match particao {
                Particao::Aprovado => {
                    acumulador.qtd_aprovado += 1;
                    acumulador.cent_aprovado = somar(
                        acumulador.cent_aprovado,
                        cent,
                        "soma do resumo aprovado",
                        &contexto_erro,
                    )?;
                }
                Particao::Ok => {
                    acumulador.qtd_ok += 1;
                    acumulador.cent_ok = somar(
                        acumulador.cent_ok,
                        cent,
                        "soma do resumo ok",
                        &contexto_erro,
                    )?;
                }
                Particao::Revisar => {
                    acumulador.qtd_revisar += 1;
                    acumulador.cent_revisar = somar(
                        acumulador.cent_revisar,
                        cent,
                        "soma do resumo revisar",
                        &contexto_erro,
                    )?;
                }
            }
            // R3C2-02: il gruppo registra da quali documenti arriva e in quale
            // stato, cosi' un gruppo nutrito da un PARCIAL non si confonde con
            // uno sano, e un problema altrui non lo marca.
            acumulador
                .documentos
                .insert(contexto.chave_origem.clone(), stato_documento);
            if nome_divergente
                && !acumulador
                    .nomes_divergentes
                    .contains(&lancamento.conta_nome)
            {
                acumulador
                    .nomes_divergentes
                    .push(lancamento.conta_nome.clone());
            }

            linhas.push(LinhaPreLivro {
                identidade: IdentidadeLinha {
                    lote: contexto.lote.id.clone(),
                    chave_origem: contexto.chave_origem.clone(),
                    origem: lancamento.origem,
                    conta_id: lancamento.conta_id.clone(),
                    ordinal: indice as u32,
                },
                lancamento: lancamento.clone(),
                particao,
                motivos,
                chave_resumo: chave,
                conta_nome_plano: conta.nome.clone(),
                empresa_identificada,
                stato_documento,
                no_extrato_aprovado,
            });
        }

        for a in &contexto.avisos_leitura {
            if !avisos.contains(a) {
                avisos.push(a.clone());
            }
        }

        docs.push(DocumentoPreLivro {
            lote: contexto.lote.id.clone(),
            chave_origem: contexto.chave_origem.clone(),
            numero: contexto.numero.clone(),
            serie: contexto.serie.clone(),
            data_documento: contexto.data_documento,
            competencia: contexto.competencia,
            cnpj_empresa: contexto.cnpj_empresa.clone(),
            nome_empresa: contexto.nome_empresa.clone(),
            arquivo_origem: contexto.arquivo_origem.clone(),
            empresa_identificada,
            stato_documento,
            linhas: contexto.alocacao.lancamentos.len(),
            reconciliacao: reconciliacao.clone(),
            avisos_leitura: contexto.avisos_leitura.clone(),
        });
    }

    // I7/F3C-25: ordinamento deterministico, indipendente dall'ordine
    // d'ingresso. La chiave di riga e' unica, quindi l'ordine e' totale.
    linhas.sort_by_key(chave_ordenacao);
    docs.sort_by(|a, b| {
        (
            chave_empresa_de(&a.cnpj_empresa),
            a.competencia,
            a.chave_origem.clone(),
        )
            .cmp(&(
                chave_empresa_de(&b.cnpj_empresa),
                b.competencia,
                b.chave_origem.clone(),
            ))
    });
    // Anche i contenitori aggregati hanno un ordine canonico: l'ordine di
    // arrivo degli avvisi non e' un'informazione del pre-libro (R3C2-03).
    avisos.sort_by(|a, b| (&a.codigo, &a.detalhe).cmp(&(&b.codigo, &b.detalhe)));

    let exclusoes = exclusoes_do_extrato(&linhas);
    let resumos_ordenados = finalizar_resumos(resumos)?;
    let indicadores = calcular_indicadores(&linhas, &docs, &exclusoes)?;

    Ok(PreLivro {
        lote: Some(lote),
        linhas,
        documentos: docs,
        resumos: resumos_ordenados,
        indicadores,
        exclusoes_extrato: exclusoes,
        avisos,
    })
}

/// Tutti i documenti di un pre-libro appartengono alla **stessa** esecuzione,
/// metadati compresi: con lo stesso `id` ma metadati diversi si dichiara il
/// conflitto invece di tenere quelli del primo documento (R3C2-03).
fn verificar_lote(lote: &IdentidadeLote, contexto: &ContextoDocumento) -> Result<(), ErroPreLivro> {
    if contexto.lote.id != lote.id {
        return Err(ErroPreLivro::LotesDivergentes {
            esperado: lote.id.clone(),
            encontrado: contexto.lote.id.clone(),
            chave_origem: contexto.chave_origem.clone(),
        });
    }
    let divergencia = |campo: &'static str, esperado: String, encontrado: String| {
        ErroPreLivro::MetadadosDoLoteDivergentes {
            lote: lote.id.clone(),
            campo,
            esperado,
            encontrado,
            chave_origem: contexto.chave_origem.clone(),
        }
    };
    if contexto.lote.gerado_em != lote.gerado_em {
        return Err(divergencia(
            "gerado_em",
            lote.gerado_em.to_string(),
            contexto.lote.gerado_em.to_string(),
        ));
    }
    if contexto.lote.descricao != lote.descricao {
        return Err(divergencia(
            "descricao",
            lote.descricao.clone(),
            contexto.lote.descricao.clone(),
        ));
    }
    Ok(())
}

/// R3C2-01: il contesto e la registrazione devono parlare dello stesso
/// documento. I campi confrontati sono quelli che identificano il documento;
/// l'elenco e la motivazione stanno nella nota di modulo.
fn verificar_coerencia(
    contexto: &ContextoDocumento,
    lancamento: &Lancamento,
) -> Result<(), ErroPreLivro> {
    let discordancia = |campo: &'static str, no_contexto: String, no_lancamento: String| {
        ErroPreLivro::DocumentoIncoerente {
            chave_origem: contexto.chave_origem.clone(),
            campo,
            no_contexto,
            no_lancamento,
        }
    };
    if contexto.chave_origem != lancamento.chave_origem {
        return Err(discordancia(
            "chave_origem",
            contexto.chave_origem.clone(),
            lancamento.chave_origem.clone(),
        ));
    }
    if contexto.numero != lancamento.documento {
        return Err(discordancia(
            "documento",
            contexto.numero.clone(),
            lancamento.documento.clone(),
        ));
    }
    if contexto.data_documento != lancamento.data_documento {
        return Err(discordancia(
            "data_documento",
            contexto.data_documento.to_string(),
            lancamento.data_documento.to_string(),
        ));
    }
    if contexto.competencia != lancamento.competencia {
        return Err(discordancia(
            "competencia",
            contexto.competencia.to_string(),
            lancamento.competencia.to_string(),
        ));
    }
    if contexto.cnpj_empresa.trim() != lancamento.cnpj_empresa.trim() {
        return Err(discordancia(
            "cnpj_empresa",
            contexto.cnpj_empresa.clone(),
            lancamento.cnpj_empresa.clone(),
        ));
    }
    Ok(())
}

fn chave_ordenacao(linha: &LinhaPreLivro) -> (String, Competencia, String, String, u32) {
    (
        linha.chave_resumo.cnpj_empresa.clone(),
        linha.chave_resumo.competencia,
        linha.chave_resumo.conta_id.clone(),
        linha.identidade.chave_origem.clone(),
        linha.identidade.ordinal,
    )
}

fn chave_empresa_de(cnpj: &str) -> String {
    if cnpj.trim().is_empty() {
        EMPRESA_NAO_IDENTIFICADA.to_string()
    } else {
        cnpj.trim().to_string()
    }
}

fn aviso(codigo: &str, detalhe: impl Into<String>) -> Aviso {
    Aviso {
        codigo: codigo.to_string(),
        detalhe: detalhe.into(),
    }
}

fn particao_de(status: StatusRevisao) -> Particao {
    // Precedenza della specifica: REVISAR prima di APROVADO prima di OK.
    match status {
        StatusRevisao::Revisar => Particao::Revisar,
        StatusRevisao::Aprovado => Particao::Aprovado,
        StatusRevisao::Ok => Particao::Ok,
    }
}

/// Tutte le cause della partizione: gli avvisi della riga, nessuno perduto.
fn motivos_da_linha(lancamento: &Lancamento) -> Vec<String> {
    lancamento
        .alertas
        .split(';')
        .map(|m| m.trim())
        .filter(|m| !m.is_empty())
        .map(|m| m.to_string())
        .collect()
}

fn somar(
    acumulado: i128,
    valor: i128,
    operacao: &'static str,
    contexto: &str,
) -> Result<i128, ErroPreLivro> {
    dinheiro::somar_centesimos([acumulado, valor], operacao)
        .map_err(|e| de_erro_dinheiro(e, contexto, operacao))
}

/// D10(a): ogni riga fuori dall'estratto approvato e' elencata con il motivo.
fn exclusoes_do_extrato(linhas: &[LinhaPreLivro]) -> Vec<Exclusao> {
    linhas
        .iter()
        .filter(|linha| !linha.no_extrato_aprovado)
        .map(|linha| {
            let motivo = if linha.stato_documento == StatoDocumento::Erro {
                format!(
                    "documento com estado ERRO: nao entra no extrato aprovado (linha {})",
                    linha.particao.rotulo()
                )
            } else {
                format!(
                    "linha em {}: somente linhas APROVADO entram no extrato",
                    linha.particao.rotulo()
                )
            };
            Exclusao {
                identidade: linha.identidade.clone(),
                valor: linha.lancamento.valor,
                particao: linha.particao,
                stato_documento: linha.stato_documento,
                motivo,
            }
        })
        .collect()
}

fn finalizar_resumos(
    resumos: BTreeMap<ResumoChave, AcumuladorResumo>,
) -> Result<Vec<Resumo>, ErroPreLivro> {
    let mut saida = Vec::with_capacity(resumos.len());
    for (chave, acc) in resumos {
        let contexto = format!(
            "resumo {}/{}/{}",
            chave.cnpj_empresa, chave.competencia, chave.conta_id
        );
        let cent_total = dinheiro::somar_centesimos(
            [acc.cent_aprovado, acc.cent_ok, acc.cent_revisar],
            "soma do resumo total",
        )
        .map_err(|e| de_erro_dinheiro(e, &contexto, "soma do resumo total"))?;

        let documentos_ok = acc
            .documentos
            .values()
            .filter(|s| **s == StatoDocumento::Ok)
            .count();
        let documentos_parcial = acc
            .documentos
            .values()
            .filter(|s| **s == StatoDocumento::Parcial)
            .count();
        let documentos_erro = acc
            .documentos
            .values()
            .filter(|s| **s == StatoDocumento::Erro)
            .count();
        // `BTreeMap` da' le chiavi in ordine: la marcatura non dipende
        // dall'ordine d'ingresso.
        let documentos_marcados: Vec<String> = acc
            .documentos
            .iter()
            .filter(|(_, stato)| **stato != StatoDocumento::Ok)
            .map(|(chave, _)| chave.clone())
            .collect();
        let mut nomes_divergentes = acc.nomes_divergentes;
        nomes_divergentes.sort();

        saida.push(Resumo {
            conta_nome: acc.conta_nome,
            grupo_conta: acc.grupo_conta,
            empresa_identificada: acc.empresa_identificada,
            qtd_total: acc.qtd_aprovado + acc.qtd_ok + acc.qtd_revisar,
            qtd_aprovado: acc.qtd_aprovado,
            qtd_ok: acc.qtd_ok,
            qtd_revisar: acc.qtd_revisar,
            valor_aprovado: decimal_de(acc.cent_aprovado, "valor aprovado do resumo", &contexto)?,
            valor_ok: decimal_de(acc.cent_ok, "valor ok do resumo", &contexto)?,
            valor_revisar: decimal_de(acc.cent_revisar, "valor revisar do resumo", &contexto)?,
            valor_total: decimal_de(cent_total, "valor total do resumo", &contexto)?,
            documentos_ok,
            documentos_parcial,
            documentos_erro,
            documentos_marcados,
            nomes_divergentes,
            chave,
        });
    }
    Ok(saida)
}

fn decimal_de(
    centesimos: i128,
    operacao: &'static str,
    contexto: &str,
) -> Result<Decimal, ErroPreLivro> {
    dinheiro::de_centesimos(centesimos, operacao)
        .map_err(|e| de_erro_dinheiro(e, contexto, operacao))
}

fn calcular_indicadores(
    linhas: &[LinhaPreLivro],
    docs: &[DocumentoPreLivro],
    exclusoes: &[Exclusao],
) -> Result<Indicadores, ErroPreLivro> {
    let contexto = "indicadores do lote";
    let mut cent_aprovado: i128 = 0;
    let mut cent_ok: i128 = 0;
    let mut cent_revisar: i128 = 0;
    let mut cent_extrato: i128 = 0;

    for linha in linhas {
        let campo = format!("linha {}/valor", linha.identidade.conta_id);
        let cent = dinheiro::em_centesimos(linha.lancamento.valor, &campo)
            .map_err(|e| de_erro_dinheiro(e, contexto, &campo))?;
        match linha.particao {
            Particao::Aprovado => {
                cent_aprovado = somar(cent_aprovado, cent, "soma aprovado do lote", contexto)?
            }
            Particao::Ok => cent_ok = somar(cent_ok, cent, "soma ok do lote", contexto)?,
            Particao::Revisar => {
                cent_revisar = somar(cent_revisar, cent, "soma revisar do lote", contexto)?
            }
        }
        if linha.no_extrato_aprovado {
            cent_extrato = somar(cent_extrato, cent, "soma do extrato aprovado", contexto)?;
        }
    }
    let cent_total =
        dinheiro::somar_centesimos([cent_aprovado, cent_ok, cent_revisar], "soma total do lote")
            .map_err(|e| de_erro_dinheiro(e, contexto, "soma total do lote"))?;

    Ok(Indicadores {
        documentos: docs.len(),
        documentos_ok: docs
            .iter()
            .filter(|d| d.stato_documento == StatoDocumento::Ok)
            .count(),
        documentos_parcial: docs
            .iter()
            .filter(|d| d.stato_documento == StatoDocumento::Parcial)
            .count(),
        documentos_erro: docs
            .iter()
            .filter(|d| d.stato_documento == StatoDocumento::Erro)
            .count(),
        documentos_sem_linhas: docs.iter().filter(|d| d.linhas == 0).count(),
        linhas: linhas.len(),
        linhas_aprovado: linhas
            .iter()
            .filter(|l| l.particao == Particao::Aprovado)
            .count(),
        linhas_ok: linhas.iter().filter(|l| l.particao == Particao::Ok).count(),
        linhas_revisar: linhas
            .iter()
            .filter(|l| l.particao == Particao::Revisar)
            .count(),
        valor_total: decimal_de(cent_total, "valor total do lote", contexto)?,
        valor_aprovado: decimal_de(cent_aprovado, "valor aprovado do lote", contexto)?,
        valor_ok: decimal_de(cent_ok, "valor ok do lote", contexto)?,
        valor_revisar: decimal_de(cent_revisar, "valor revisar do lote", contexto)?,
        valor_extrato_aprovado: decimal_de(cent_extrato, "valor do extrato aprovado", contexto)?,
        linhas_excluidas_do_extrato: exclusoes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alocacao::{
        Componente, EsitoComponenti, EsitoCopertura, EsitoRate, EsitoTotal, RuoloComponente,
        alocar_nfe, avisos as avisos_3b,
    };
    use crate::classificacao::Classificador;
    use crate::modelos::{Dado, OrigemLancamento, StatusDado, TipoConta};
    use crate::nfe::ler_nfe;
    use crate::regras::RegrasClassificacao;
    use std::path::PathBuf;

    fn raiz() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn plano() -> PlanoContas {
        PlanoContas::carregar(raiz().join("config/plano_contas_gerencial.yaml")).unwrap()
    }

    fn dec(bruto: &str) -> Decimal {
        Decimal::from_str_exact(bruto).unwrap()
    }

    fn competencia() -> Competencia {
        Competencia::new(2026, 5).unwrap()
    }

    fn lote() -> IdentidadeLote {
        IdentidadeLote {
            id: "lote-2026-05-A".to_string(),
            gerado_em: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            descricao: "teste".to_string(),
        }
    }

    /// Riconciliazione sintetica dichiarata: i 15 campi, senza passare dalla
    /// fase 2. Non sostituisce il campione canonico, usato in `T-01`.
    fn reconciliacao(stato: StatoDocumento, total: &str, soma: &str) -> Reconciliacao {
        let presente = |v: Decimal| Dado::presente(v, "teste://componente").unwrap();
        Reconciliacao {
            valor_total_nfe: dec(total),
            soma_lancamentos: dec(soma),
            esito_total: if total == soma {
                EsitoTotal::Ok
            } else {
                EsitoTotal::Erro
            },
            soma_rate_documento: Dado::ausente(
                "nfe:cobr/dup/vDup",
                StatusDado::NaoAplicavel,
                "sintetica sem duplicatas",
            )
            .unwrap(),
            soma_rate_allocate: Dado::ausente(
                "nfe:cobr/dup/vDup",
                StatusDado::NaoAplicavel,
                "sintetica sem duplicatas",
            )
            .unwrap(),
            esito_rate: EsitoRate::NaoAplicavel,
            articoli_allocati: 1,
            articoli_attesi: Dado::ausente(
                "nfe:det",
                StatusDado::NaoDisponivel,
                "a fase 2 nao expoe o numero de <det> lidos",
            )
            .unwrap(),
            articoli_scartati: Vec::new(),
            esito_copertura: EsitoCopertura::Ok,
            componenti: vec![Componente {
                nome: "vProd".to_string(),
                valor: presente(dec(total)),
                ruolo: RuoloComponente::Base,
                usada_na_verificacao: true,
            }],
            esito_componenti: EsitoComponenti::Ok,
            campi_rinviati: Vec::new(),
            avisos_documento: Vec::new(),
            stato_documento: stato,
        }
    }

    fn lancamento(
        conta_id: &str,
        conta_nome: &str,
        valor: &str,
        status: StatusRevisao,
        alertas: &str,
        cnpj_empresa: &str,
    ) -> Lancamento {
        Lancamento {
            data_documento: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
            competencia: competencia(),
            cnpj_empresa: cnpj_empresa.to_string(),
            nome_empresa: "EMPRESA".to_string(),
            fornecedor_ou_favorecido: "FORNECEDOR".to_string(),
            cnpj_cpf_favorecido: "12345678000195".to_string(),
            documento: "12566".to_string(),
            tipo_documento: "XML_NFE".to_string(),
            conta_id: conta_id.to_string(),
            conta_nome: conta_nome.to_string(),
            grupo_conta: "Grupo".to_string(),
            descricao_itens: "ITEM (10.00)".to_string(),
            valor: dec(valor),
            duplicatas: Vec::new(),
            origem: OrigemLancamento::XmlNfe,
            arquivo_origem: "teste://doc.xml".to_string(),
            chave_origem: "CHAVE-1".to_string(),
            hash_registro: "hash".to_string(),
            status_revisao: status,
            alertas: alertas.to_string(),
            observacoes: String::new(),
            centro_custo: String::new(),
            tipo_lancamento: TipoConta::Custo,
            dre_linha_id: String::new(),
            dre_linha_nome: String::new(),
            data_pagamento: Dado::ausente(
                "nfe:cobr/dup/dVenc",
                StatusDado::NaoDisponivel,
                "a NF-e declara vencimentos, nao pagamentos",
            )
            .unwrap(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn contexto(
        chave: &str,
        cnpj_empresa: &str,
        stato: StatoDocumento,
        total: &str,
        lancamentos: Vec<Lancamento>,
    ) -> ContextoDocumento {
        let soma: Decimal = lancamentos
            .iter()
            .fold(Decimal::ZERO, |acc, l| acc + l.valor);
        // Contesto e registrazioni vengono dalla stessa NF-e: l'helper
        // riproduce quella coerenza, come fa il chiamante reale. I test che
        // provano l'incoerenza la rompono **dopo**, a campo singolo.
        let lancamentos: Vec<Lancamento> = lancamentos
            .into_iter()
            .map(|mut l| {
                l.chave_origem = chave.to_string();
                l.documento = "12566".to_string();
                l.data_documento = NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
                l.competencia = competencia();
                l.cnpj_empresa = cnpj_empresa.to_string();
                l
            })
            .collect();
        ContextoDocumento {
            chave_origem: chave.to_string(),
            numero: "12566".to_string(),
            serie: "001".to_string(),
            data_documento: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
            competencia: competencia(),
            cnpj_empresa: cnpj_empresa.to_string(),
            nome_empresa: "EMPRESA".to_string(),
            arquivo_origem: format!("teste://{chave}.xml"),
            alocacao: AlocacaoNfe {
                lancamentos,
                reconciliacao: reconciliacao(stato, total, &soma.to_string()),
            },
            avisos_leitura: Vec::new(),
            lote: lote(),
        }
    }

    // -----------------------------------------------------------------
    // T-01: campione canonico, attraverso la pipeline reale della 3B
    // -----------------------------------------------------------------

    #[test]
    fn t01_campione_canonico_tres_linhas_e_rate_conservadas() {
        let importada = ler_nfe(raiz().join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml"))
            .expect("campione leggibile");
        let classificador = Classificador::compilar(
            &RegrasClassificacao::carregar(
                raiz().join("config/regras_classificacao_produtos.yaml"),
            )
            .unwrap(),
        );
        let plano = plano();
        let alocacao = alocar_nfe(&importada, &classificador, &plano).unwrap();

        let ctx = ContextoDocumento {
            chave_origem: importada.nfe.chave.clone(),
            numero: importada.nfe.numero.clone(),
            serie: importada.nfe.serie.clone(),
            data_documento: importada.nfe.data_emissao,
            competencia: Competencia::from_data(importada.nfe.data_emissao).unwrap(),
            cnpj_empresa: importada.nfe.cnpj_destinatario.clone(),
            nome_empresa: importada.nfe.nome_destinatario.clone(),
            arquivo_origem: importada.nfe.arquivo_origem.clone(),
            alocacao,
            avisos_leitura: importada.avisos.clone(),
            lote: lote(),
        };

        let pre = construir_pre_livro(&[ctx], &plano).unwrap();

        assert_eq!(pre.linhas.len(), 3);
        assert_eq!(pre.indicadores.valor_total, dec("35.00"));
        assert_eq!(pre.indicadores.linhas_ok, 3);
        assert_eq!(pre.indicadores.linhas_revisar, 0);
        assert_eq!(pre.documentos.len(), 1);
        assert_eq!(pre.documentos[0].stato_documento, StatoDocumento::Ok);
        // Le rate restano sulle righe, non vengono riassunte via.
        for linha in &pre.linhas {
            assert_eq!(linha.lancamento.duplicatas.len(), 2);
            let soma: Decimal = linha
                .lancamento
                .duplicatas
                .iter()
                .fold(Decimal::ZERO, |a, d| a + d.valor);
            assert_eq!(soma, linha.lancamento.valor);
        }
        // Tre conti distinti, un riepilogo per conto, una sola azienda.
        assert_eq!(pre.resumos.len(), 3);
        assert!(pre.resumos.iter().all(|r| r.empresa_identificada));
    }

    // -----------------------------------------------------------------
    // T-02…T-05, T-09: partizioni, stato del documento, riconciliazione
    // -----------------------------------------------------------------

    #[test]
    fn t02_linha_revisar_conserva_conta_e_avisos() {
        let l = lancamento(
            "13.01.001",
            "Revisar - Produto Nao Classificado",
            "12.00",
            StatusRevisao::Revisar,
            avisos_3b::NAO_CLASSIFICADO,
            "55746722000181",
        );
        let pre = construir_pre_livro(
            &[contexto(
                "CH-1",
                "55746722000181",
                StatoDocumento::Parcial,
                "12.00",
                vec![l],
            )],
            &plano(),
        )
        .unwrap();

        let linha = &pre.linhas[0];
        assert_eq!(linha.particao, Particao::Revisar);
        assert_eq!(linha.lancamento.conta_id, "13.01.001");
        assert_eq!(linha.motivos, vec![avisos_3b::NAO_CLASSIFICADO.to_string()]);
        assert_eq!(linha.lancamento.status_revisao, StatusRevisao::Revisar);
    }

    #[test]
    fn t03_aprovado_distinto_de_ok_e_de_revisar() {
        let ctx = contexto(
            "CH-2",
            "55746722000181",
            StatoDocumento::Ok,
            "60.00",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                    "55746722000181",
                ),
                lancamento(
                    "2.01.002",
                    "Vinhos e Espumantes",
                    "20.00",
                    StatusRevisao::Aprovado,
                    "",
                    "55746722000181",
                ),
                lancamento(
                    "2.04.001",
                    "Produtos de Limpeza e Higiene",
                    "30.00",
                    StatusRevisao::Revisar,
                    avisos_3b::CLASSIFICACAO_AMBIGUA,
                    "55746722000181",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.indicadores.linhas_ok, 1);
        assert_eq!(pre.indicadores.linhas_aprovado, 1);
        assert_eq!(pre.indicadores.linhas_revisar, 1);
        assert_eq!(pre.indicadores.valor_aprovado, dec("20.00"));
        assert_eq!(pre.indicadores.valor_ok, dec("10.00"));
        assert_eq!(pre.indicadores.valor_revisar, dec("30.00"));
        // Solo la riga APROVADO entra nell'estratto.
        assert_eq!(pre.extrato_aprovado().len(), 1);
        assert_eq!(pre.indicadores.valor_extrato_aprovado, dec("20.00"));
    }

    #[test]
    fn t04_documento_parcial_com_linhas_ok_resta_marcato() {
        let ctx = contexto(
            "CH-3",
            "55746722000181",
            StatoDocumento::Parcial,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            )],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.linhas[0].particao, Particao::Ok);
        assert_eq!(pre.linhas[0].stato_documento, StatoDocumento::Parcial);
        assert_eq!(pre.indicadores.documentos_parcial, 1);
    }

    #[test]
    fn t05_documento_erro_nao_entra_no_extrato_aprovado() {
        let ctx = contexto(
            "CH-4",
            "55746722000181",
            StatoDocumento::Erro,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Aprovado,
                "",
                "55746722000181",
            )],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        // La riga e' APROVADO, ma il documento e' ERRO.
        assert_eq!(pre.linhas[0].particao, Particao::Aprovado);
        assert!(!pre.linhas[0].no_extrato_aprovado);
        assert!(pre.extrato_aprovado().is_empty());
        assert_eq!(pre.exclusoes_extrato.len(), 1);
        assert!(
            pre.exclusoes_extrato[0].motivo.contains("ERRO"),
            "{}",
            pre.exclusoes_extrato[0].motivo
        );
        // Ma resta nel diagnostico e nei riepiloghi.
        assert_eq!(pre.linhas.len(), 1);
        assert_eq!(pre.resumos[0].valor_total, dec("10.00"));
    }

    #[test]
    fn t06_documento_sem_linhas_resta_com_a_sua_identidade() {
        let ctx = contexto(
            "CH-5",
            "55746722000181",
            StatoDocumento::Erro,
            "35.00",
            Vec::new(),
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert!(pre.linhas.is_empty());
        assert_eq!(pre.documentos.len(), 1);
        let doc = &pre.documentos[0];
        assert_eq!(doc.chave_origem, "CH-5");
        assert_eq!(doc.cnpj_empresa, "55746722000181");
        assert_eq!(doc.competencia, competencia());
        assert_eq!(doc.stato_documento, StatoDocumento::Erro);
        assert_eq!(doc.linhas, 0);
        assert_eq!(pre.indicadores.documentos_sem_linhas, 1);
        assert!(
            pre.avisos
                .iter()
                .any(|a| a.codigo == avisos::DOCUMENTO_SEM_LINHAS)
        );
    }

    #[test]
    fn t07_data_pagamento_ausente_e_vencimento_sao_distintos() {
        let importada =
            ler_nfe(raiz().join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml")).unwrap();
        let classificador = Classificador::compilar(
            &RegrasClassificacao::carregar(
                raiz().join("config/regras_classificacao_produtos.yaml"),
            )
            .unwrap(),
        );
        let plano = plano();
        let alocacao = alocar_nfe(&importada, &classificador, &plano).unwrap();
        let ctx = ContextoDocumento {
            chave_origem: importada.nfe.chave.clone(),
            numero: importada.nfe.numero.clone(),
            serie: importada.nfe.serie.clone(),
            data_documento: importada.nfe.data_emissao,
            competencia: Competencia::from_data(importada.nfe.data_emissao).unwrap(),
            cnpj_empresa: importada.nfe.cnpj_destinatario.clone(),
            nome_empresa: importada.nfe.nome_destinatario.clone(),
            arquivo_origem: importada.nfe.arquivo_origem.clone(),
            alocacao,
            avisos_leitura: importada.avisos.clone(),
            lote: lote(),
        };
        let pre = construir_pre_livro(&[ctx], &plano).unwrap();

        let linha = &pre.linhas[0];
        assert_eq!(linha.lancamento.data_pagamento.valor(), None);
        assert_eq!(
            linha.lancamento.data_pagamento.status(),
            StatusDado::NaoDisponivel
        );
        assert!(linha.lancamento.duplicatas[0].vencimento.is_some());
    }

    #[test]
    fn t08_payload_repetido_da_duas_linhas_com_identidades_distintas() {
        let mesma = |n: &str| {
            lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                n,
            )
        };
        let ctx = contexto(
            "CH-6",
            "55746722000181",
            StatoDocumento::Ok,
            "20.00",
            vec![mesma("55746722000181"), mesma("55746722000181")],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.linhas.len(), 2, "nessuna riga scartata");
        assert_ne!(pre.linhas[0].identidade, pre.linhas[1].identidade);
        assert_eq!(pre.linhas[0].identidade.ordinal, 0);
        assert_eq!(pre.linhas[1].identidade.ordinal, 1);
        // I5: le due righe non vengono fuse nel riepilogo.
        assert_eq!(pre.resumos.len(), 1);
        assert_eq!(pre.resumos[0].qtd_total, 2);
        assert_eq!(pre.resumos[0].valor_total, dec("20.00"));
    }

    #[test]
    fn t09_reconciliacao_do_documento_e_conservada() {
        let ctx = contexto(
            "CH-7",
            "55746722000181",
            StatoDocumento::Parcial,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            )],
        );
        let reconciliacao_original = ctx.alocacao.reconciliacao.clone();
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.documentos[0].reconciliacao, reconciliacao_original);
        assert_eq!(
            pre.documentos[0].reconciliacao.esito_copertura,
            EsitoCopertura::Ok
        );
        assert_eq!(
            pre.documentos[0].reconciliacao.articoli_attesi.status(),
            StatusDado::NaoDisponivel
        );
    }

    // -----------------------------------------------------------------
    // T-10…T-13: aggregazioni (D13)
    // -----------------------------------------------------------------

    #[test]
    fn t10_duas_empresas_no_mesmo_mes_dao_resumos_distintos() {
        let a = contexto(
            "CH-A",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let b = contexto(
            "CH-B",
            "22222222222222",
            StatoDocumento::Ok,
            "20.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "20.00",
                StatusRevisao::Ok,
                "",
                "22222222222222",
            )],
        );
        let pre = construir_pre_livro(&[a, b], &plano()).unwrap();

        assert_eq!(pre.resumos.len(), 2, "le aziende non si fondono");
        let valori: Vec<Decimal> = pre.resumos.iter().map(|r| r.valor_total).collect();
        assert!(valori.contains(&dec("10.00")) && valori.contains(&dec("20.00")));
        assert_eq!(pre.indicadores.valor_total, dec("30.00"));
    }

    #[test]
    fn t11_empresa_nao_identificada_fica_em_particao_propria() {
        let identificada = contexto(
            "CH-C",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let anonima = contexto(
            "CH-D",
            "",
            StatoDocumento::Ok,
            "30.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "30.00",
                StatusRevisao::Ok,
                "",
                "",
            )],
        );
        let pre = construir_pre_livro(&[identificada, anonima], &plano()).unwrap();

        assert_eq!(pre.resumos.len(), 2);
        let anonimo = pre
            .resumos
            .iter()
            .find(|r| r.chave.cnpj_empresa == EMPRESA_NAO_IDENTIFICADA)
            .expect("partizione dell'azienda non identificata");
        assert!(!anonimo.empresa_identificada);
        assert_eq!(anonimo.valor_total, dec("30.00"));
        assert!(
            pre.avisos
                .iter()
                .any(|a| a.codigo == avisos::EMPRESA_SEM_IDENTIFICACAO)
        );
    }

    #[test]
    fn t12_nome_do_conto_vem_do_plano_nao_da_linha() {
        let ctx = contexto(
            "CH-E",
            "11111111111111",
            StatoDocumento::Ok,
            "30.00",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.01.004",
                    "REFRIGERANTES",
                    "20.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        // Un solo gruppo, nome dal piano.
        assert_eq!(pre.resumos.len(), 1);
        assert_eq!(pre.resumos[0].conta_nome, "Refrigerantes");
        assert_eq!(pre.resumos[0].valor_total, dec("30.00"));
        assert_eq!(pre.resumos[0].nomes_divergentes, vec!["REFRIGERANTES"]);
        assert!(
            pre.avisos
                .iter()
                .any(|a| a.codigo == avisos::NOME_DE_CONTA_DIVERGENTE)
        );
        // La riga conserva il proprio nome: niente correzione silenziosa.
        let divergente = pre
            .linhas
            .iter()
            .find(|l| l.lancamento.conta_nome == "REFRIGERANTES")
            .unwrap();
        assert_eq!(divergente.conta_nome_plano, "Refrigerantes");
        assert!(
            divergente
                .motivos
                .contains(&avisos::NOME_DE_CONTA_DIVERGENTE.to_string())
        );
    }

    #[test]
    fn t13_resumos_com_negativos_e_zeros_fecham() {
        let ctx = contexto(
            "CH-F",
            "11111111111111",
            StatoDocumento::Ok,
            "0.00",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.05.002",
                    "Descontos Obtidos sobre Compras",
                    "-10.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.04.001",
                    "Produtos de Limpeza e Higiene",
                    "0.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.resumos.len(), 3);
        let por_conta = |id: &str| {
            pre.resumos
                .iter()
                .find(|r| r.chave.conta_id == id)
                .unwrap()
                .valor_total
        };
        assert_eq!(por_conta("2.01.004"), dec("10.00"));
        assert_eq!(por_conta("2.05.002"), dec("-10.00"));
        assert_eq!(por_conta("2.04.001"), dec("0.00"));
        assert_eq!(pre.indicadores.valor_total, dec("0.00"));
    }

    #[test]
    fn t14_particao_unica_com_mais_de_um_motivo() {
        let l = lancamento(
            "13.01.001",
            "Revisar - Produto Nao Classificado",
            "12.00",
            StatusRevisao::Revisar,
            &format!(
                "{}; {}",
                avisos_3b::NAO_CLASSIFICADO,
                avisos_3b::VALOR_INCONSISTENTE
            ),
            "11111111111111",
        );
        let pre = construir_pre_livro(
            &[contexto(
                "CH-G",
                "11111111111111",
                StatoDocumento::Erro,
                "12.00",
                vec![l],
            )],
            &plano(),
        )
        .unwrap();

        let linha = &pre.linhas[0];
        assert_eq!(linha.particao, Particao::Revisar);
        assert_eq!(linha.motivos.len(), 2, "nessuna causa perduta");
        assert!(
            linha
                .motivos
                .contains(&avisos_3b::NAO_CLASSIFICADO.to_string())
        );
        assert!(
            linha
                .motivos
                .contains(&avisos_3b::VALOR_INCONSISTENTE.to_string())
        );
        // I1: una sola partizione.
        assert_eq!(pre.indicadores.linhas, 1);
        assert_eq!(
            pre.indicadores.linhas_aprovado
                + pre.indicadores.linhas_ok
                + pre.indicadores.linhas_revisar,
            1
        );
    }

    // -----------------------------------------------------------------
    // T-15…T-20: invarianti e casi limite
    // -----------------------------------------------------------------

    #[test]
    fn t15_conservacao_e_contagens_fecham() {
        let ctx = contexto(
            "CH-H",
            "11111111111111",
            StatoDocumento::Parcial,
            "60.00",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.01.002",
                    "Vinhos e Espumantes",
                    "20.00",
                    StatusRevisao::Aprovado,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.04.001",
                    "Produtos de Limpeza e Higiene",
                    "30.00",
                    StatusRevisao::Revisar,
                    avisos_3b::ITEM_SEM_DESCRICAO,
                    "11111111111111",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        // I1 e I2.
        assert_eq!(pre.linhas.len(), 3);
        assert_eq!(
            pre.indicadores.linhas_aprovado
                + pre.indicadores.linhas_ok
                + pre.indicadores.linhas_revisar,
            pre.indicadores.linhas
        );
        // I3: le somme per partizione chiudono sul totale.
        assert_eq!(
            pre.indicadores.valor_aprovado
                + pre.indicadores.valor_ok
                + pre.indicadores.valor_revisar,
            pre.indicadores.valor_total
        );
        // Ogni riga ha esattamente una partizione: il conteggio per partizione
        // riproduce l'insieme.
        assert_eq!(pre.linhas_na_particao(Particao::Ok).len(), 1);
        assert_eq!(pre.linhas_na_particao(Particao::Aprovado).len(), 1);
        assert_eq!(pre.linhas_na_particao(Particao::Revisar).len(), 1);
    }

    #[test]
    fn t16_somas_por_chave_em_centavos() {
        let ctx = contexto(
            "CH-I",
            "11111111111111",
            StatoDocumento::Ok,
            "0.03",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "0.01",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "0.01",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "0.01",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();
        assert_eq!(pre.resumos.len(), 1);
        assert_eq!(pre.resumos[0].valor_total, dec("0.03"));
        assert_eq!(pre.resumos[0].valor_ok, dec("0.03"));
        assert_eq!(pre.indicadores.valor_total, dec("0.03"));
    }

    #[test]
    fn t17_nenhuma_fusao_por_hash() {
        let mut l1 = lancamento(
            "2.01.004",
            "Refrigerantes",
            "10.00",
            StatusRevisao::Ok,
            "",
            "11111111111111",
        );
        let mut l2 = l1.clone();
        l1.hash_registro = "mesmo-hash".to_string();
        l2.hash_registro = "mesmo-hash".to_string();
        let pre = construir_pre_livro(
            &[contexto(
                "CH-J",
                "11111111111111",
                StatoDocumento::Ok,
                "20.00",
                vec![l1, l2],
            )],
            &plano(),
        )
        .unwrap();

        assert_eq!(pre.linhas.len(), 2);
        assert_eq!(pre.resumos[0].qtd_total, 2);
        assert_eq!(pre.resumos[0].valor_total, dec("20.00"));
    }

    #[test]
    fn t18_exclusoes_elencam_identidade_valor_e_motivo() {
        let ctx = contexto(
            "CH-K",
            "11111111111111",
            StatoDocumento::Ok,
            "30.00",
            vec![
                lancamento(
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                    "11111111111111",
                ),
                lancamento(
                    "2.01.002",
                    "Vinhos e Espumantes",
                    "20.00",
                    StatusRevisao::Aprovado,
                    "",
                    "11111111111111",
                ),
            ],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();

        assert_eq!(pre.exclusoes_extrato.len(), 1);
        let exclusao = &pre.exclusoes_extrato[0];
        assert_eq!(exclusao.particao, Particao::Ok);
        assert_eq!(exclusao.valor, dec("10.00"));
        assert_eq!(exclusao.identidade.conta_id, "2.01.004");
        assert!(!exclusao.motivo.is_empty());
        assert_eq!(pre.indicadores.linhas_excluidas_do_extrato, 1);
    }

    #[test]
    fn t19_permutar_a_ordem_de_entrada_nao_muda_o_resultado() {
        let a = contexto(
            "CH-L",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let b = contexto(
            "CH-M",
            "22222222222222",
            StatoDocumento::Ok,
            "20.00",
            vec![lancamento(
                "2.01.002",
                "Vinhos e Espumantes",
                "20.00",
                StatusRevisao::Ok,
                "",
                "22222222222222",
            )],
        );
        let plano = plano();
        let direto = construir_pre_livro(&[a.clone(), b.clone()], &plano).unwrap();
        let invertido = construir_pre_livro(&[b, a], &plano).unwrap();

        assert_eq!(direto.linhas, invertido.linhas);
        assert_eq!(direto.resumos, invertido.resumos);
        assert_eq!(direto.documentos, invertido.documentos);
        assert_eq!(direto.indicadores, invertido.indicadores);
        assert_eq!(direto.exclusoes_extrato, invertido.exclusoes_extrato);
    }

    #[test]
    fn t20_lote_vazio_e_valido_e_nao_inventa_identidade() {
        let pre = construir_pre_livro(&[], &plano()).unwrap();
        assert!(pre.lote.is_none(), "nessuna identita' di lotto inventata");
        assert!(pre.linhas.is_empty());
        assert!(pre.documentos.is_empty());
        assert!(pre.resumos.is_empty());
        assert_eq!(pre.indicadores.documentos, 0);
        assert_eq!(pre.indicadores.valor_total, Decimal::ZERO);
    }

    // -----------------------------------------------------------------
    // T-21…T-23: capacita' numerica e risoluzione del piano
    // -----------------------------------------------------------------

    /// T-21: due documenti con importi legittimi la cui somma non e'
    /// rappresentabile. Nessun pre-libro parziale, nessun troncamento.
    #[test]
    fn t21_soma_de_varios_documentos_que_excede_decimal_e_erro() {
        let maximo = "79228162514264337593543950335";
        let a = contexto(
            "CH-N",
            "11111111111111",
            StatoDocumento::Ok,
            maximo,
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                maximo,
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let b = contexto(
            "CH-O",
            "11111111111111",
            StatoDocumento::Ok,
            "0.01",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "0.01",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let erro = construir_pre_livro(&[a, b], &plano()).unwrap_err();
        match &erro {
            ErroPreLivro::CapacidadeNumericaExcedida { operacao, .. } => {
                assert!(
                    operacao.contains("resumo") || operacao.contains("lote"),
                    "{operacao}"
                );
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(
            erro.to_string().contains("nenhum pre-livro parcial"),
            "{erro}"
        );
    }

    #[test]
    fn t22_nome_do_conto_resolvido_pelo_plano() {
        let ctx = contexto(
            "CH-P",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.04.001",
                "nome qualquer",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();
        assert_eq!(pre.resumos[0].conta_nome, "Produtos de Limpeza e Higiene");
        assert_eq!(pre.resumos[0].grupo_conta, "Materiais de Limpeza e Higiene");
        assert_eq!(
            pre.linhas[0].conta_nome_plano,
            "Produtos de Limpeza e Higiene"
        );
    }

    #[test]
    fn t23_conto_ausente_do_plano_e_erro_nomeado() {
        let ctx = contexto(
            "CH-Q",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "9.99.999",
                "Conta inexistente",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let erro = construir_pre_livro(&[ctx], &plano()).unwrap_err();
        match &erro {
            ErroPreLivro::ContaNaoEncontradaNoPlano {
                chave_origem,
                conta_id,
            } => {
                assert_eq!(chave_origem, "CH-Q");
                assert_eq!(conta_id, "9.99.999");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(
            erro.to_string().contains("nenhum nome e' inventado"),
            "{erro}"
        );
    }

    // -----------------------------------------------------------------
    // Identita' del lotto e del documento
    // -----------------------------------------------------------------

    #[test]
    fn chave_de_documento_vazia_e_recusada() {
        let ctx = contexto(
            "",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let erro = construir_pre_livro(&[ctx], &plano()).unwrap_err();
        assert!(
            matches!(erro, ErroPreLivro::ChaveDocumentoVazia { .. }),
            "{erro}"
        );
    }

    #[test]
    fn documento_repetido_no_lote_e_recusado() {
        let um = contexto(
            "CH-R",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let outro = um.clone();
        let erro = construir_pre_livro(&[um, outro], &plano()).unwrap_err();
        assert!(
            matches!(erro, ErroPreLivro::DocumentoDuplicadoNoLote { .. }),
            "{erro}"
        );
    }

    #[test]
    fn lotes_divergentes_sao_recusados() {
        let um = contexto(
            "CH-S",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        let mut outro = contexto(
            "CH-T",
            "11111111111111",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        outro.lote.id = "lote-diverso".to_string();
        let erro = construir_pre_livro(&[um, outro], &plano()).unwrap_err();
        assert!(
            matches!(erro, ErroPreLivro::LotesDivergentes { .. }),
            "{erro}"
        );
    }

    #[test]
    fn avisos_de_leitura_sao_propagados_sem_duplicati() {
        let mut ctx = contexto(
            "CH-U",
            "11111111111111",
            StatoDocumento::Parcial,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "11111111111111",
            )],
        );
        ctx.avisos_leitura = vec![
            Aviso {
                codigo: "DET_SEM_PROD".to_string(),
                detalhe: "det 2 sem bloco prod".to_string(),
            },
            Aviso {
                codigo: "DET_SEM_PROD".to_string(),
                detalhe: "det 2 sem bloco prod".to_string(),
            },
        ];
        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();
        assert_eq!(
            pre.avisos
                .iter()
                .filter(|a| a.codigo == "DET_SEM_PROD")
                .count(),
            1
        );
        assert_eq!(
            pre.documentos[0].avisos_leitura.len(),
            2,
            "il documento conserva i suoi"
        );
    }

    // -----------------------------------------------------------------
    // R3C2-01: il contesto e l'allocazione devono parlare dello stesso
    // documento. Prima l'abbinamento sbagliato passava in silenzio e il
    // valore finiva nel riepilogo di un'altra azienda e di un altro mese.
    // -----------------------------------------------------------------

    #[test]
    fn r1_chave_discordante_entre_contexto_e_lancamento_e_erro() {
        let mut ctx = contexto(
            "CHAVE-CONTEXTO",
            "55746722000181",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            )],
        );
        // La registrazione dichiara un'altra NF-e.
        ctx.alocacao.lancamentos[0].chave_origem = "CHAVE-OUTRA".to_string();
        let erro = construir_pre_livro(&[ctx], &plano()).unwrap_err();
        match &erro {
            ErroPreLivro::DocumentoIncoerente {
                campo,
                no_contexto,
                no_lancamento,
                ..
            } => {
                assert_eq!(*campo, "chave_origem");
                assert_eq!(no_contexto, "CHAVE-CONTEXTO");
                assert_eq!(no_lancamento, "CHAVE-OUTRA");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn r1_empresa_discordante_entre_contexto_e_lancamento_e_erro() {
        let mut ctx = contexto(
            "CHAVE-1",
            "00000000000001",
            StatoDocumento::Ok,
            "35.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "35.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            )],
        );
        // La registrazione dichiara un'altra azienda.
        ctx.alocacao.lancamentos[0].cnpj_empresa = "55746722000181".to_string();
        let erro = construir_pre_livro(&[ctx], &plano()).unwrap_err();
        match &erro {
            ErroPreLivro::DocumentoIncoerente {
                campo,
                no_contexto,
                no_lancamento,
                ..
            } => {
                assert_eq!(*campo, "cnpj_empresa");
                assert_eq!(no_contexto, "00000000000001");
                assert_eq!(no_lancamento, "55746722000181");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        // Il valore non e' mai stato attribuito all'azienda del contesto.
        assert!(erro.to_string().contains("00000000000001"), "{erro}");
    }

    #[test]
    fn r1_competencia_discordante_entre_contexto_e_lancamento_e_erro() {
        let mut ctx = contexto(
            "CHAVE-2",
            "55746722000181",
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            )],
        );
        ctx.alocacao.lancamentos[0].chave_origem = "CHAVE-2".to_string();
        ctx.competencia = Competencia::new(2026, 6).unwrap();
        let erro = construir_pre_livro(&[ctx], &plano()).unwrap_err();
        match &erro {
            ErroPreLivro::DocumentoIncoerente { campo, .. } => {
                assert_eq!(*campo, "competencia");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    /// Controllo positivo: un contesto coerente non viene disturbato, e le
    /// registrazioni restano **inalterate**.
    #[test]
    fn r1_contexto_coerente_e_aceito_sem_tocar_os_lancamentos() {
        let l = lancamento(
            "2.01.004",
            "Refrigerantes",
            "10.00",
            StatusRevisao::Ok,
            "",
            "55746722000181",
        );
        let mut ctx = contexto(
            "CHAVE-OK",
            "55746722000181",
            StatoDocumento::Ok,
            "10.00",
            vec![l.clone()],
        );
        ctx.alocacao.lancamentos[0].chave_origem = "CHAVE-OK".to_string();
        let original = ctx.alocacao.lancamentos[0].clone();

        let pre = construir_pre_livro(&[ctx], &plano()).unwrap();
        assert_eq!(pre.linhas[0].lancamento, original, "nessuna riscrittura");
        assert_eq!(pre.resumos[0].chave.cnpj_empresa, "55746722000181");
        assert_eq!(pre.resumos[0].chave.competencia, competencia());
    }

    // -----------------------------------------------------------------
    // R3C2-02: il riepilogo deve dire da quali documenti arriva, quando
    // quei documenti sono PARCIAL o ERRO (D10a, F3C-11).
    // -----------------------------------------------------------------

    #[test]
    fn r2_resumo_marca_os_documentos_parcial_que_o_alimentam() {
        let linha_ok = |chave: &str| {
            let mut l = lancamento(
                "2.01.004",
                "Refrigerantes",
                "35.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            );
            l.chave_origem = chave.to_string();
            l
        };
        let sao = contexto(
            "CH-SAO",
            "55746722000181",
            StatoDocumento::Ok,
            "35.00",
            vec![linha_ok("CH-SAO")],
        );
        let parcial = contexto(
            "CH-PARCIAL",
            "55746722000181",
            StatoDocumento::Parcial,
            "35.00",
            vec![linha_ok("CH-PARCIAL")],
        );

        let plano = plano();
        let so_sao = construir_pre_livro(std::slice::from_ref(&sao), &plano).unwrap();
        let com_parcial = construir_pre_livro(&[sao, parcial], &plano).unwrap();

        // Le righe restano OK in entrambi i casi: la marcatura e' del documento.
        assert_eq!(so_sao.resumos[0].qtd_ok, 1);
        assert_eq!(com_parcial.resumos[0].qtd_ok, 2);

        // Prima della correzione i due riepiloghi erano indistinguibili.
        assert_eq!(so_sao.resumos[0].documentos_parcial, 0);
        assert_eq!(so_sao.resumos[0].documentos_erro, 0);
        assert_eq!(com_parcial.resumos[0].documentos_parcial, 1);
        assert_eq!(com_parcial.resumos[0].documentos_ok, 1);
        assert_eq!(
            com_parcial.resumos[0].documentos_marcados,
            vec!["CH-PARCIAL".to_string()]
        );
        assert_ne!(so_sao.resumos[0], com_parcial.resumos[0]);
    }

    #[test]
    fn r2_problema_de_um_grupo_nao_marca_grupo_alheio() {
        let mut l_limpo = lancamento(
            "2.01.004",
            "Refrigerantes",
            "10.00",
            StatusRevisao::Ok,
            "",
            "55746722000181",
        );
        l_limpo.chave_origem = "CH-LIMPO".to_string();
        let mut l_erro = lancamento(
            "2.04.001",
            "Produtos de Limpeza e Higiene",
            "20.00",
            StatusRevisao::Ok,
            "",
            "55746722000181",
        );
        l_erro.chave_origem = "CH-ERRO".to_string();

        let limpo = contexto(
            "CH-LIMPO",
            "55746722000181",
            StatoDocumento::Ok,
            "10.00",
            vec![l_limpo],
        );
        let com_erro = contexto(
            "CH-ERRO",
            "55746722000181",
            StatoDocumento::Erro,
            "20.00",
            vec![l_erro],
        );
        let pre = construir_pre_livro(&[limpo, com_erro], &plano()).unwrap();

        let resumo_limpo = pre
            .resumos
            .iter()
            .find(|r| r.chave.conta_id == "2.01.004")
            .unwrap();
        let resumo_erro = pre
            .resumos
            .iter()
            .find(|r| r.chave.conta_id == "2.04.001")
            .unwrap();

        assert_eq!(
            resumo_limpo.documentos_erro, 0,
            "gruppo estraneo non marcato"
        );
        assert!(resumo_limpo.documentos_marcados.is_empty());
        assert_eq!(resumo_erro.documentos_erro, 1);
        assert_eq!(resumo_erro.documentos_marcados, vec!["CH-ERRO".to_string()]);
        // L'esclusione dall'estratto del documento ERRO resta.
        assert!(pre.extrato_aprovado().is_empty());
    }

    // -----------------------------------------------------------------
    // R3C2-03: nessuna dipendenza dall'ordine d'ingresso, nemmeno nei
    // contenitori aggregati e nei metadati del lotto (I7, F3C-25).
    // -----------------------------------------------------------------

    #[test]
    fn r3_permutacao_com_avisos_e_nomes_divergentes_e_identica() {
        let fazer = |chave: &str, nome_conta: &str, aviso_codigo: &str| {
            let mut l = lancamento(
                "2.01.004",
                nome_conta,
                "10.00",
                StatusRevisao::Revisar,
                aviso_codigo,
                "55746722000181",
            );
            l.chave_origem = chave.to_string();
            let mut ctx = contexto(
                chave,
                "55746722000181",
                StatoDocumento::Parcial,
                "10.00",
                vec![l],
            );
            ctx.avisos_leitura = vec![Aviso {
                codigo: aviso_codigo.to_string(),
                detalhe: format!("detalhe de {chave}"),
            }];
            ctx
        };
        let a = fazer("CH-AA", "Refrigerante A", "DET_SEM_PROD");
        let b = fazer("CH-BB", "Refrigerante B", "SOMA_ITENS_DIVERGE");

        let plano = plano();
        let direto = construir_pre_livro(&[a.clone(), b.clone()], &plano).unwrap();
        let invertido = construir_pre_livro(&[b, a], &plano).unwrap();

        assert_eq!(
            direto.resumos, invertido.resumos,
            "nomi divergenti ordinati"
        );
        assert_eq!(direto.avisos, invertido.avisos, "avvisi ordinati");
        assert_eq!(direto, invertido, "l'intero pre-libro e' identico");
    }

    #[test]
    fn r3_metadados_do_lote_discordantes_sao_conflito_nos_dois_sentidos() {
        let fazer = |chave: &str, dia: u32, descricao: &str| {
            let mut l = lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            );
            l.chave_origem = chave.to_string();
            let mut ctx = contexto(
                chave,
                "55746722000181",
                StatoDocumento::Ok,
                "10.00",
                vec![l],
            );
            ctx.lote = IdentidadeLote {
                id: "lote-2026-05-A".to_string(),
                gerado_em: NaiveDate::from_ymd_opt(2026, 5, dia).unwrap(),
                descricao: descricao.to_string(),
            };
            ctx
        };
        let a = fazer("CH-CC", 30, "primeira descricao");
        let b = fazer("CH-DD", 31, "segunda descricao");

        let plano = plano();
        for (um, outro) in [(a.clone(), b.clone()), (b, a)] {
            let erro = construir_pre_livro(&[um, outro], &plano).unwrap_err();
            match &erro {
                ErroPreLivro::MetadadosDoLoteDivergentes { campo, .. } => {
                    assert!(
                        *campo == "gerado_em" || *campo == "descricao",
                        "campo inesperado: {campo}"
                    );
                }
                outro => panic!("erro inesperado: {outro}"),
            }
        }
    }

    #[test]
    fn r3_lote_inteiramente_identico_e_aceito() {
        let fazer = |chave: &str| {
            let mut l = lancamento(
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
                "55746722000181",
            );
            l.chave_origem = chave.to_string();
            contexto(
                chave,
                "55746722000181",
                StatoDocumento::Ok,
                "10.00",
                vec![l],
            )
        };
        let pre = construir_pre_livro(&[fazer("CH-EE"), fazer("CH-FF")], &plano()).unwrap();
        assert_eq!(pre.lote.as_ref().unwrap().id, "lote-2026-05-A");
        assert_eq!(pre.indicadores.valor_total, dec("20.00"));
    }
}

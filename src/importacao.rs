//! Composizione dell'importazione (fase 3C.5.a): che cosa contiene
//! l'esecuzione, e che esito ha ogni documento.
//!
//! # Purezza
//!
//! Nessun IO, nessun orologio, nessun percorso letto dal disco. I documenti
//! arrivano già letti dalla fase 2, l'archivio corrente arriva come
//! parametro, e il piano come gli altri comandi lo ricevono. Leggere la
//! cartella, pubblicare e stampare sono della CLI.
//!
//! # I tre esiti di un documento (D29)
//!
//! | Esito | Quando | Che cosa fa la CLI |
//! |---|---|---|
//! | [`EsitoDocumento::Novo`] | la chiave non è in nessuno snapshot | entra nell'esecuzione |
//! | [`EsitoDocumento::JaArquivadoIdentico`] | chiave già archiviata, **impronta di contenuto uguale** | entra nell'esecuzione: è riuso fra esecuzioni, non un duplicato nel risultato |
//! | [`EsitoDocumento::ConteudoDiverso`] | chiave già archiviata, impronta diversa | rifiuto dell'intera esecuzione (D33) |
//!
//! Il terzo esito dice **«contenuto allocato diverso»** e non «la fattura è
//! stata corretta»: [`crate::impressao::texto_canonico`] include `conta_id`,
//! `conta_nome`, `grupo_conta` e `alertas`, cioè gli **effetti delle regole**.
//! Una differenza può venire dalla fonte **o** dalla configurazione, e
//! l'archivio registra l'identità del piano ma non quella delle regole:
//! distinguerle non è possibile qui, quindi non si attribuisce una causa.
//!
//! # Che cosa questo modulo **non** decide
//!
//! Non deduplica fra esecuzioni e non sceglie quali documenti sopravvivono:
//! D34 dichiara che il risultato copre la cartella **al momento del
//! rilancio**. Una fattura togliata dalla cartella resta nello storico
//! dell'archivio e non entra nell'esecuzione nuova, perché non è stata letta.

use std::fmt;

use crate::alocacao::{ErroAlocacao, StatoDocumento, alocar_nfe};
use crate::arquivo::Arquivo;
use crate::classificacao::Classificador;
use crate::impressao;
use crate::modelos::{Competencia, ErroCompetencia};
use crate::nfe::NfeImportada;
use crate::prelivro::{ContextoDocumento, IdentidadeLote};
use crate::regras::PlanoContas;

// ---------------------------------------------------------------------------
// Codici d'uscita (CLI-01)
// ---------------------------------------------------------------------------

/// Codici d'uscita di `arquivar-nfe`.
///
/// Esiti distinti e determinati: il riassunto del comando dice sempre se
/// l'archivio corrente è stato pubblicato o no.
pub mod codigos_arquivar {
    /// Esecuzione pubblicata, tutti i documenti con stato tecnico `OK`.
    pub const OK: u8 = 0;
    /// Configurazione bocciata, argomenti incoerenti, cartella senza XML.
    pub const CONFIGURACAO: u8 = 1;
    /// Un XML non elaborabile: archivio corrente **non** pubblicato.
    pub const NAO_PROCESSAVEL: u8 = 2;
    /// Due XML con la stessa chiave nella stessa esecuzione.
    pub const DUPLICADA_NA_EXECUCAO: u8 = 3;
    /// Chiave già archiviata con contenuto allocato diverso (D33).
    pub const CONTEUDO_DIVERSO: u8 = 4;
    /// Pubblicata con riserva: almeno un documento `PARCIAL` o `ERRO` (D30).
    pub const ARQUIVADO_COM_RESERVA: u8 = 5;
    /// Rifiuto di IO **prima** del commit.
    pub const IO: u8 = 6;
    /// `--recusar-nao-ok` con almeno un documento non `OK`: non pubblicato.
    pub const RECUSADO_NAO_OK: u8 = 7;
}

/// Codici d'uscita di `exportar-diagnostico`.
pub mod codigos_exportar {
    /// File derivato pubblicato.
    pub const OK: u8 = 0;
    /// Configurazione, argomenti, o lotto inesistente nell'archivio.
    pub const CONFIGURACAO: u8 = 1;
    /// Destinazione già esistente con `--se-existir recusar`.
    pub const DESTINO_EXISTENTE: u8 = 3;
    /// Destinazione che coincide con l'archivio o con la sua famiglia.
    pub const DESTINO_E_FONTE: u8 = 4;
    /// Altro rifiuto di IO prima del commit.
    pub const IO: u8 = 6;
}

// ---------------------------------------------------------------------------
// Ingresso e uscita
// ---------------------------------------------------------------------------

/// Un documento raccolto: il percorso **dichiarato** da chi ha letto, e il
/// contesto costruito.
///
/// Il percorso serve soltanto ai messaggi: nominare i due file di una chiave
/// ripetuta è la differenza fra un rifiuto utile e un rifiuto muto.
#[derive(Debug, Clone)]
pub struct DocumentoColhido {
    pub arquivo: String,
    pub contexto: ContextoDocumento,
}

/// Esito di un documento rispetto all'archivio corrente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EsitoDocumento {
    Novo,
    JaArquivadoIdentico {
        lote: String,
    },
    ConteudoDiverso {
        lote: String,
        salva: String,
        atual: String,
    },
}

impl EsitoDocumento {
    pub const fn rotulo(&self) -> &'static str {
        match self {
            EsitoDocumento::Novo => "NOVO",
            EsitoDocumento::JaArquivadoIdentico { .. } => "JA_ARQUIVADO_IDENTICO",
            EsitoDocumento::ConteudoDiverso { .. } => "CONTEUDO_ALOCADO_DIVERSO",
        }
    }
}

/// Una riga del rapporto: un documento, il suo esito e il suo stato tecnico.
#[derive(Debug, Clone)]
pub struct LinhaRelatorio {
    pub arquivo: String,
    pub chave_origem: String,
    pub esito: EsitoDocumento,
    /// Stato tecnico del documento, dalla riconciliazione della 3B.
    pub stato_documento: StatoDocumento,
}

/// Che cosa l'esecuzione conterrebbe, documento per documento.
#[derive(Debug, Clone)]
pub struct Relatorio {
    pub linhas: Vec<LinhaRelatorio>,
}

impl Relatorio {
    pub fn novos(&self) -> usize {
        self.linhas
            .iter()
            .filter(|l| l.esito == EsitoDocumento::Novo)
            .count()
    }

    pub fn ja_arquivados(&self) -> usize {
        self.linhas
            .iter()
            .filter(|l| matches!(l.esito, EsitoDocumento::JaArquivadoIdentico { .. }))
            .count()
    }

    /// I documenti con contenuto allocato diverso: ognuno blocca l'esecuzione.
    pub fn conteudos_diversos(&self) -> Vec<&LinhaRelatorio> {
        self.linhas
            .iter()
            .filter(|l| matches!(l.esito, EsitoDocumento::ConteudoDiverso { .. }))
            .collect()
    }

    /// I documenti il cui stato tecnico non è `OK` (D30).
    pub fn nao_ok(&self) -> Vec<&LinhaRelatorio> {
        self.linhas
            .iter()
            .filter(|l| l.stato_documento != StatoDocumento::Ok)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore della composizione. Ogni variante nomina il file che l'ha causato.
#[derive(Debug)]
pub enum ErroImportacao {
    /// La fase 3B ha rifiutato il documento.
    Alocacao { arquivo: String, erro: ErroAlocacao },
    /// La data di emissione non dà una competenza valida.
    Competencia {
        arquivo: String,
        erro: ErroCompetencia,
    },
    /// La classificazione ha rifiutato una descrizione.
    Classificacao { arquivo: String, detalhe: String },
    /// Due file della **stessa** esecuzione portano la stessa chiave: un
    /// risultato non può contare una fattura due volte.
    DuplicadaNaExecucao {
        chave_origem: String,
        primeiro: String,
        segundo: String,
    },
}

impl fmt::Display for ErroImportacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroImportacao::Alocacao { arquivo, erro } => {
                write!(f, "{arquivo}: {erro}")
            }
            ErroImportacao::Competencia { arquivo, erro } => {
                write!(f, "{arquivo}: competencia invalida: {erro}")
            }
            ErroImportacao::Classificacao { arquivo, detalhe } => {
                write!(f, "{arquivo}: classificacao recusada: {detalhe}")
            }
            ErroImportacao::DuplicadaNaExecucao {
                chave_origem,
                primeiro,
                segundo,
            } => write!(
                f,
                "chave {chave_origem} repetida na mesma execucao: {primeiro} e {segundo}. \
                 Um resultado nao pode contar uma fatura duas vezes"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Composizione
// ---------------------------------------------------------------------------

/// Costruisce il contesto di un documento da una NF-e letta.
///
/// I cinque campi che il pre-libro confronta fra contesto e registrazioni
/// (`chave_origem`, `documento`, `data_documento`, `competencia`,
/// `cnpj_empresa`) vengono dalla **stessa** fonte che
/// [`alocar_nfe`] usa per le righe: l'azienda è il destinatario della NF-e,
/// il fornitore è l'emittente. Prenderli da due posti diversi farebbe
/// scattare `DocumentoIncoerente`.
pub fn contexto_de(
    importada: &NfeImportada,
    arquivo: &str,
    classificador: &Classificador,
    plano: &PlanoContas,
    lote: &IdentidadeLote,
) -> Result<ContextoDocumento, ErroImportacao> {
    let nfe = &importada.nfe;
    let competencia =
        Competencia::from_data(nfe.data_emissao).map_err(|erro| ErroImportacao::Competencia {
            arquivo: arquivo.to_string(),
            erro,
        })?;
    let alocacao =
        alocar_nfe(importada, classificador, plano).map_err(|erro| ErroImportacao::Alocacao {
            arquivo: arquivo.to_string(),
            erro,
        })?;

    Ok(ContextoDocumento {
        chave_origem: nfe.chave.clone(),
        numero: nfe.numero.clone(),
        serie: nfe.serie.clone(),
        data_documento: nfe.data_emissao,
        competencia,
        cnpj_empresa: nfe.cnpj_destinatario.clone(),
        nome_empresa: nfe.nome_destinatario.clone(),
        arquivo_origem: nfe.arquivo_origem.clone(),
        alocacao,
        avisos_leitura: importada.avisos.clone(),
        lote: lote.clone(),
    })
}

/// Classifica i documenti raccolti contro l'archivio corrente.
///
/// `corrente` è `None` alla prima esecuzione: non esiste niente con cui
/// confrontare, e ogni documento è nuovo.
///
/// Il confronto usa l'**ultimo** snapshot archiviato con quella chiave, cioè
/// l'esecuzione più recente che la contiene: è lo stato con cui il rilancio si
/// confronta. L'impronta di contenuto non include il lotto né
/// `arquivo_origem`, quindi lo stesso documento riletto da un'altra cartella
/// resta identico.
pub fn classificar(
    colhidos: &[DocumentoColhido],
    corrente: Option<&Arquivo>,
) -> Result<Relatorio, ErroImportacao> {
    let mut vistos: Vec<(&str, &str)> = Vec::with_capacity(colhidos.len());
    let mut linhas: Vec<LinhaRelatorio> = Vec::with_capacity(colhidos.len());

    for colhido in colhidos {
        let chave = colhido.contexto.chave_origem.as_str();
        if let Some((_, primeiro)) = vistos.iter().find(|(c, _)| *c == chave) {
            return Err(ErroImportacao::DuplicadaNaExecucao {
                chave_origem: chave.to_string(),
                primeiro: (*primeiro).to_string(),
                segundo: colhido.arquivo.clone(),
            });
        }
        vistos.push((chave, colhido.arquivo.as_str()));

        let atual = impressao::impressao_conteudo(&colhido.contexto);
        let esito = match corrente
            .and_then(|a| a.snapshots.iter().rev().find(|s| s.chave_origem == chave))
        {
            None => EsitoDocumento::Novo,
            Some(snapshot) if snapshot.impressao_conteudo == atual => {
                EsitoDocumento::JaArquivadoIdentico {
                    lote: snapshot.lote.clone(),
                }
            }
            Some(snapshot) => EsitoDocumento::ConteudoDiverso {
                lote: snapshot.lote.clone(),
                salva: snapshot.impressao_conteudo.clone(),
                atual,
            },
        };

        linhas.push(LinhaRelatorio {
            arquivo: colhido.arquivo.clone(),
            chave_origem: chave.to_string(),
            esito,
            stato_documento: colhido.contexto.alocacao.reconciliacao.stato_documento,
        });
    }

    Ok(Relatorio { linhas })
}

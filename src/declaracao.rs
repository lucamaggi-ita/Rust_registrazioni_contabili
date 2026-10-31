//! Dichiarazione di revisione dalla CLI (fase 3C.5.b).
//!
//! Costruisce il **candidato**, lo identifica esattamente, e dice **prima**
//! di scrivere se si applica. Non apre file, non legge l'orologio e non
//! pubblica niente: l'IO sta nella CLI.
//!
//! # Perché il controllo precede la scrittura
//!
//! [`crate::revisao::aplicar_revisao`] non tollera un evento malformato né una
//! coppia incompatibile: autore o motivo vuoti, nota mancante per `ANOTAR`,
//! bersaglio incompleto o ambiguo, e due decisioni incompatibili alla stessa
//! coordinata temporale sono **errori** che fanno fallire l'intera
//! applicazione. L'archivio è append-only e non si corregge (`D16(a)`), quindi
//! un evento così, una volta archiviato, renderebbe impossibile ogni export
//! futuro di quell'archivio.
//!
//! # Come si riconosce il candidato
//!
//! Per [`crate::revisao::texto_exato_evento`], la codifica **iniettiva** dei
//! dati dichiarati, istante e offset compresi. Non per `ChaveEvento`, che il
//! modello dichiara non unica, non per un indice, e non per l'impronta del
//! payload.
//!
//! La ripetizione si cerca fra gli eventi **già archiviati**, prima di
//! applicare. Cercarla nella traccia dopo l'applicazione troverebbe il
//! rappresentante vecchio: in caso di duplicato esatto il rappresentante resta
//! applicato e la ripetizione finisce fra i non applicati, quindi la traccia
//! conterrebbe un evento con lo stesso testo esatto **comunque**, e la
//! dichiarazione nuova passerebbe per applicata.
//!
//! # Che cosa questo modulo non fa
//!
//! Non pretende che `eventos_nao_aplicados` sia vuoto: un evento che riguarda
//! un documento presente solo in un altro lotto diventa `ALVO_AUSENTE` nella
//! vista del lotto scelto, ed è normale. E non giudica l'esito dall'effetto
//! sullo stato della riga: `ANOTAR`, una conferma e un evento dichiarato
//! retroattivamente possono applicarsi **senza** cambiare lo stato corrente.

use std::fmt;

use chrono::{DateTime, FixedOffset};

use crate::alocacao::StatoDocumento;
use crate::modelos::{Dado, ErroDado, OrigemLancamento, StatusDado, StatusRevisao};
use crate::prelivro::{ContextoDocumento, Particao};
use crate::regras::PlanoContas;
use crate::revisao::{
    ChaveRevisao, ErroRevisao, EstadoRevisao, EventoRevisao, MotivoNaoAplicacao, OperacaoRevisao,
    VistaRevisada, aplicar_revisao, texto_exato_evento,
};

/// Codici d'uscita dei due comandi della 3C.5.b (`CLI-01`).
///
/// Una categoria per **famiglia**, non un codice per variante.
pub mod codigos_revisao {
    /// Candidato applicato e pubblicato, o consultazione riuscita.
    pub const OK: u8 = 0;
    /// Configurazione, argomenti, lotto o riga inesistenti.
    pub const CONFIGURACAO: u8 = 1;
    /// La barriera di `revisao.rs` rifiuta: forma, bersaglio ambiguo, o
    /// decisioni incompatibili.
    pub const BARREIRA: u8 = 2;
    /// Il candidato non sarebbe applicato: il motivo è uno dei cinque.
    pub const NAO_APLICADO: u8 = 3;
    /// Rifiuto di IO prima del commit, con i residui nominati.
    pub const IO: u8 = 6;
}

// ---------------------------------------------------------------------------
// Ingresso: ciò che la persona dichiara
// ---------------------------------------------------------------------------

/// Il bersaglio, completo come `ChaveRevisao` lo richiede.
///
/// `origem` fa parte del bersaglio e viene dalla riga **selezionata**: non ha
/// un valore predefinito arbitrario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alvo {
    pub chave_origem: String,
    pub conta_id: String,
    pub ordinal: u32,
    pub origem: OrigemLancamento,
}

/// I campi dichiarati da chi decide. Nessuno viene dedotto dall'ambiente.
#[derive(Debug, Clone)]
pub struct Declaracao {
    pub alvo: Alvo,
    pub operacao: OperacaoRevisao,
    pub autor: String,
    pub motivo: String,
    pub nota: Option<String>,
    /// L'impronta di contenuto che la persona **ha esaminato**. Non viene
    /// sostituita con quella della riga corrente: è ciò che rende possibile
    /// `VERSAO_SUPERADA`, e quindi la provenienza della decisione.
    pub impressao_conteudo: String,
    /// L'impronta di presentazione. `None` = assenza **dichiarata**.
    pub vista: Option<String>,
    pub instante: DateTime<FixedOffset>,
    pub sequencia: Option<u64>,
}

/// Costruisce il candidato dai campi dichiarati.
///
/// La forma viene comunque validata da `aplicar_revisao`, che è l'unico
/// giudice: qui si costruisce, non si duplica quella validazione.
pub fn candidato(d: &Declaracao) -> Result<EventoRevisao, ErroDado> {
    let vista = match &d.vista {
        Some(valor) => Dado::presente(valor.clone(), "cli:--vista")?,
        None => Dado::ausente(
            "cli:--vista",
            StatusDado::NaoDisponivel,
            "nao declarada na linha de comando",
        )?,
    };
    Ok(EventoRevisao {
        alvo: ChaveRevisao {
            chave_origem: d.alvo.chave_origem.clone(),
            impressao_conteudo: Dado::presente(
                d.impressao_conteudo.clone(),
                "cli:--impressao-conteudo",
            )?,
            origem: d.alvo.origem,
            conta_id: d.alvo.conta_id.clone(),
            ordinal: d.alvo.ordinal,
        },
        operacao: d.operacao,
        autor: d.autor.clone(),
        instante: d.instante,
        motivo: d.motivo.clone(),
        nota: d.nota.clone(),
        sequencia: d.sequencia,
        vista,
    })
}

// ---------------------------------------------------------------------------
// Uscita: l'esito del candidato
// ---------------------------------------------------------------------------

/// Che cosa succederebbe al candidato.
#[derive(Debug, Clone)]
pub enum EsitoCandidato {
    /// Applicato. Lo stato della riga può essere cambiato **o no**: un'
    /// annotazione si applica senza decidere lo stato.
    Aplicado {
        /// `None` se la riga non esiste in questa esecuzione.
        estado_antes: Option<EstadoRevisao>,
        estado_depois: Option<EstadoRevisao>,
    },
    /// Non applicato, con uno dei cinque motivi del modello.
    NaoAplicado {
        motivo: MotivoNaoAplicacao,
        detalhe: String,
    },
    /// Ripetizione esatta di un evento **già archiviato**: stessi dati
    /// dichiarati, istante e offset compresi.
    JaArquivado,
}

/// Perché non si è potuto nemmeno classificare il candidato.
#[derive(Debug)]
pub enum ErroClassificacao {
    /// Gli eventi **già archiviati** non si applicano nemmeno da soli: il
    /// problema precede questa dichiarazione, e va detto, perché non è
    /// dell'operatore.
    ArquivoJaNaoAplica(ErroRevisao),
    /// Con il candidato la barriera rifiuta: forma, bersaglio ambiguo, o
    /// decisioni incompatibili.
    Barreira(ErroRevisao),
}

impl fmt::Display for ErroClassificacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroClassificacao::ArquivoJaNaoAplica(erro) => write!(
                f,
                "os eventos ja' arquivados nao se aplicam nem sem esta declaracao: {erro}. \
                 O problema precede este comando"
            ),
            ErroClassificacao::Barreira(erro) => write!(f, "{erro}"),
        }
    }
}

/// `true` se un evento con **lo stesso testo esatto** è già archiviato.
pub fn ja_arquivado(candidato: &EventoRevisao, arquivados: &[EventoRevisao]) -> bool {
    let exato = texto_exato_evento(candidato);
    arquivados.iter().any(|e| texto_exato_evento(e) == exato)
}

/// Classifica l'esito del **solo** candidato, applicandolo insieme a tutta la
/// storia degli eventi dell'archivio.
pub fn classificar(
    contextos: &[ContextoDocumento],
    plano: &PlanoContas,
    arquivados: &[EventoRevisao],
    candidato: &EventoRevisao,
) -> Result<EsitoCandidato, ErroClassificacao> {
    if ja_arquivado(candidato, arquivados) {
        return Ok(EsitoCandidato::JaArquivado);
    }

    // Prima: la vista con la **sola** storia. Se già questa non si applica, il
    // problema non è della dichiarazione nuova.
    let antes = aplicar_revisao(contextos, plano, arquivados)
        .map_err(ErroClassificacao::ArquivoJaNaoAplica)?;

    let mut tudo: Vec<EventoRevisao> = arquivados.to_vec();
    tudo.push(candidato.clone());
    let depois = aplicar_revisao(contextos, plano, &tudo).map_err(ErroClassificacao::Barreira)?;

    let exato = texto_exato_evento(candidato);
    if let Some(nao) = depois
        .eventos_nao_aplicados
        .iter()
        .find(|n| texto_exato_evento(&n.evento) == exato)
    {
        return Ok(EsitoCandidato::NaoAplicado {
            motivo: nao.motivo,
            detalhe: nao.detalhe.clone(),
        });
    }

    // Applicato: il candidato è nella traccia. Lo stato della riga si riporta
    // prima e dopo, e può essere identico.
    Ok(EsitoCandidato::Aplicado {
        estado_antes: estado_do_alvo(&antes, &candidato.alvo),
        estado_depois: estado_do_alvo(&depois, &candidato.alvo),
    })
}

/// Lo stato della riga bersaglio in una vista, se quella riga c'è.
///
/// Il confronto usa l'identità della riga — chiave, conto, ordinale, origine —
/// e **non** l'impronta dichiarata, che è il dato da verificare, non la chiave
/// di ricerca.
fn estado_do_alvo(vista: &VistaRevisada, alvo: &ChaveRevisao) -> Option<EstadoRevisao> {
    vista
        .linhas
        .iter()
        .find(|l| {
            l.identidade.chave_origem == alvo.chave_origem
                && l.identidade.conta_id == alvo.conta_id
                && l.identidade.ordinal == alvo.ordinal
                && l.identidade.origem == alvo.origem
        })
        .map(|l| l.estado)
}

// ---------------------------------------------------------------------------
// Consultazione
// ---------------------------------------------------------------------------

/// Una riga come la consultazione la mostra: i cinque stati **separati**, e le
/// due impronte che un evento deve citare.
///
/// `R3C5B-01`: la partizione da sola non prova l'esistenza di una decisione
/// umana attiva. Una riga con stato tecnico ereditato `APROVADO` ma senza un
/// evento che lo giustifichi è `APROVADO_SEM_PROVENIENCIA` (`D9.4`) e può
/// avere partizione `APROVADO`: dedurre l'una dall'altra la nasconderebbe
/// proprio a chi deve sanarla.
#[derive(Debug, Clone)]
pub struct LinhaConsultada {
    pub chave_origem: String,
    pub conta_id: String,
    /// Il nome **autorevole**, dal piano usato ora: è quello con cui l'export
    /// mostra la riga e con cui si calcola l'impronta di presentazione
    /// corrente. Mostrare lo storico accanto a quell'impronta farebbe vedere
    /// all'operatore due cose di due momenti diversi (`R3C5B-I02`).
    pub conta_nome_plano: String,
    /// Il nome conservato nella registrazione, cioè quello del momento in cui
    /// il documento è stato allocato. Si mostra **etichettato**, e solo se
    /// differisce da quello del piano corrente.
    pub conta_nome_historico: String,
    pub ordinal: u32,
    pub origem: OrigemLancamento,
    pub valor: String,
    pub status_tecnico: StatusRevisao,
    pub estado: EstadoRevisao,
    pub particao: Particao,
    /// Tipizzato: l'etichetta la decide chi stampa, con lo stesso
    /// strumento dell'export, invece di duplicare qui i nomi che
    /// `serde` gia' possiede.
    pub stato_documento: StatoDocumento,
    pub no_extrato_revisado: bool,
    /// Il motivo per cui la riga **non** entra nell'estratto revisionato, se
    /// la vista ne dichiara uno: un'approvazione su un documento `ERRO` è
    /// ammessa da `D9.2`, e non va presentata come inclusione.
    pub motivo_exclusao: Option<String>,
    /// L'impronta che un evento deve citare, dalla **vista** e non dallo
    /// snapshot storico.
    pub impressao_conteudo: Option<String>,
    pub impressao_apresentacao: Option<String>,
    pub decisoes_aplicadas: usize,
    pub anotacoes: usize,
}

impl LinhaConsultada {
    /// `true` se **non** esiste un'approvazione umana attiva su questa riga.
    pub fn sem_aprovacao_humana(&self) -> bool {
        self.estado != EstadoRevisao::AprovadoPorPessoa
    }
}

/// Le righe di una vista, nell'ordine in cui la vista le porta.
pub fn consultar(vista: &VistaRevisada) -> Vec<LinhaConsultada> {
    vista
        .linhas
        .iter()
        .map(|linha| {
            let do_prelivro = vista
                .pre_livro
                .linhas
                .iter()
                .find(|p| p.identidade == linha.identidade);
            let versao = vista
                .versoes
                .iter()
                .find(|v| v.chave_origem == linha.identidade.chave_origem);
            let exclusao = vista
                .exclusoes_extrato_revisado
                .iter()
                .find(|e| e.identidade == linha.identidade)
                .map(|e| e.motivo.clone())
                .or_else(|| {
                    vista
                        .pre_livro
                        .exclusoes_extrato
                        .iter()
                        .find(|e| e.identidade == linha.identidade)
                        .map(|e| e.motivo.clone())
                });
            LinhaConsultada {
                chave_origem: linha.identidade.chave_origem.clone(),
                conta_id: linha.identidade.conta_id.clone(),
                conta_nome_plano: do_prelivro
                    .map(|p| p.conta_nome_plano.clone())
                    .unwrap_or_default(),
                conta_nome_historico: do_prelivro
                    .map(|p| p.lancamento.conta_nome.clone())
                    .unwrap_or_default(),
                ordinal: linha.identidade.ordinal,
                origem: linha.identidade.origem,
                valor: do_prelivro
                    .map(|p| p.lancamento.valor.to_string())
                    .unwrap_or_default(),
                status_tecnico: linha.status_tecnico,
                estado: linha.estado,
                particao: do_prelivro.map(|p| p.particao).unwrap_or(Particao::Revisar),
                stato_documento: do_prelivro
                    .map(|p| p.stato_documento)
                    .unwrap_or(StatoDocumento::Erro),
                no_extrato_revisado: linha.no_extrato_revisado,
                motivo_exclusao: exclusao,
                impressao_conteudo: linha.chave.impressao_conteudo.valor().cloned(),
                impressao_apresentacao: versao.map(|v| v.impressao_apresentacao.clone()),
                decisoes_aplicadas: linha.decisoes.len(),
                anotacoes: linha.anotacoes.len(),
            }
        })
        .collect()
}

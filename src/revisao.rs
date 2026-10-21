//! Revisione umana tracciata, in memoria (fase 3C.3.a).
//!
//! Attua i contratti F3C-04, F3C-05, F3C-13, F3C-22, F3C-23 e F3C-28…F3C-46
//! della specifica di fase 3C.3, con le decisioni D9.1–D9.5 e D8.1/D8.5
//! approvate il 2026-10-21.
//!
//! [`aplicar_revisao`] e' **pura**: non legge l'orologio, non consulta
//! l'utente del sistema operativo, non tocca il filesystem e non crea nessuno
//! storico. Autore, istante e motivo arrivano dal chiamante.
//!
//! # Tre nozioni separate (F3C-29)
//!
//! | Nozione | Dove vive | Chi la cambia |
//! |---|---|---|
//! | **esito tecnico** della riga | [`LinhaRevisada::status_tecnico`], copiato dalla 3B | solo una nuova allocazione |
//! | **decisione umana** | [`LinhaRevisada::decisoes`] e [`VistaRevisada::trilha`] | un [`EventoRevisao`] dichiarato |
//! | **stato della vista** | [`LinhaRevisada::estado`] | nessuno direttamente: e' derivato dalle due precedenti |
//!
//! La fase 3B produce soltanto `OK` o `REVISAR`: un `APROVADO` in ingresso non
//! puo' venire da lei, quindi senza un evento che lo giustifichi diventa
//! [`EstadoRevisao::AprovadoSemProveniencia`] e **non** entra nell'estratto
//! revisionato (D9.4, F3C-42).
//!
//! # Identita' esatta e ordine canonico: due cose diverse (F3C-44, F3C-45)
//!
//! - **identita' esatta**: [`texto_exato_evento`], codifica **iniettiva** dei
//!   dati dichiarati, senza normalizzazione, con l'istante e il suo offset.
//!   E' l'**unico** criterio che autorizza il collasso di una ripetizione.
//! - **ordine canonico**: [`ChaveCanonica`], sei livelli, totale sugli eventi
//!   superstiti perche' il sesto livello e' il testo esatto. Non usa mai la
//!   posizione nell'array d'ingresso, nemmeno per scegliere quale errore
//!   riportare.
//!
//! Confondere le due relazioni significa scartare una decisione valida: due
//! `Aprovar` su righe diverse, oppure la stessa riga approvata con l'autore
//! scritto in due forme Unicode, o con lo stesso istante dichiarato in due
//! fusi, **non** sono ripetizioni.
//!
//! # Fuori perimetro
//!
//! Persistenza, archivio, esportazione, import dello storico e CLI: la vista e
//! gli eventi vivono in memoria, e la consultabilita' della traccia **non** e'
//! conservazione su disco. Correggere un conto, un importo, un pagamento o una
//! regola YAML sono operazioni separate, non raggiungibili da qui.

use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, FixedOffset, Utc};
use serde::Serialize;

use crate::alocacao::StatoDocumento;
use crate::impressao::{self};
use crate::modelos::{Dado, OrigemLancamento, StatusDado, StatusRevisao};
use crate::nfe::Aviso;
use crate::prelivro::{self, ContextoDocumento, ErroPreLivro, Exclusao, IdentidadeLinha, PreLivro};
use crate::regras::PlanoContas;

/// Versione dello schema della codifica esatta degli eventi.
pub const VERSAO_EVENTO_EXATO: &str = "EVENTO-EXATO/v1";

/// Codici di avviso prodotti da questo modulo.
pub mod avisos {
    /// La riga era gia' approvata da una persona: la seconda approvazione
    /// resta in traccia e non sostituisce la prima.
    pub const APROVACAO_REPETIDA: &str = "APROVACAO_REPETIDA";
    /// Un'approvazione e' stata revocata: lo stato torna a quello tecnico.
    pub const APROVACAO_REVOGADA: &str = "APROVACAO_REVOGADA";
    /// Approvazione registrata su un documento `ERRO`: resta fuori
    /// dall'estratto (D9.2, D10a).
    pub const APROVACAO_EM_DOCUMENTO_ERRO: &str = "APROVACAO_EM_DOCUMENTO_ERRO";
    /// Riga `APROVADO` senza evento di provenienza (D9.4).
    pub const APROVACAO_SEM_PROVENIENCIA: &str = "APROVACAO_SEM_PROVENIENCIA";
    /// Il piano mostra nomi diversi da quelli esaminati: le decisioni
    /// restano, il cambiamento viene segnalato (D8.3).
    pub const APRESENTACAO_MUDADA: &str = "APRESENTACAO_MUDADA";
}

// ---------------------------------------------------------------------------
// Ingresso
// ---------------------------------------------------------------------------

/// Bersaglio di una decisione: **quale contenuto** e quale riga.
///
/// Non contiene il lotto, ed e' la differenza decisiva rispetto a
/// [`IdentidadeLinha`]: una rigenerazione con lo stesso contenuto ritrova la
/// decisione, perche' la corrispondenza non passa dall'identificativo
/// dell'esecuzione (D8.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChaveRevisao {
    pub chave_origem: String,
    /// Impronta di contenuto esaminata. Assente = provenienza non dichiarata:
    /// l'evento non viene applicato come revisione verificata.
    pub impressao_conteudo: Dado<String>,
    pub origem: OrigemLancamento,
    pub conta_id: String,
    pub ordinal: u32,
}

/// Le tre sole operazioni di questo blocco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperacaoRevisao {
    Aprovar,
    Revogar,
    Anotar,
}

impl OperacaoRevisao {
    pub const fn rotulo(self) -> &'static str {
        match self {
            OperacaoRevisao::Aprovar => "APROVAR",
            OperacaoRevisao::Revogar => "REVOGAR",
            OperacaoRevisao::Anotar => "ANOTAR",
        }
    }

    /// Due operazioni sono incompatibili se decidono lo stato in modi opposti.
    /// `Anotar` non decide lo stato, quindi convive con tutto.
    const fn incompativel_com(self, outra: OperacaoRevisao) -> bool {
        matches!(
            (self, outra),
            (OperacaoRevisao::Aprovar, OperacaoRevisao::Revogar)
                | (OperacaoRevisao::Revogar, OperacaoRevisao::Aprovar)
        )
    }
}

/// Una decisione umana dichiarata.
///
/// Nessun campo viene dedotto dall'ambiente: `autor` e' una **dichiarazione**
/// di chi opera, non l'identita' del sistema operativo, e `instante` arriva
/// dal chiamante con il proprio scostamento da UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventoRevisao {
    pub alvo: ChaveRevisao,
    pub operacao: OperacaoRevisao,
    /// Non vuoto. Attribuisce una dichiarazione, **non** autentica nessuno.
    pub autor: String,
    /// Istante dichiarato, con fuso esplicito.
    pub instante: DateTime<FixedOffset>,
    /// Non vuoto.
    pub motivo: String,
    /// Obbligatoria per [`OperacaoRevisao::Anotar`], ammessa altrove.
    pub nota: Option<String>,
    /// Ordine dichiarato a parita' di istante. Un'assenza non e' uno zero.
    pub sequencia: Option<u64>,
    /// Impronta di presentazione che la persona aveva davanti.
    pub vista: Dado<String>,
}

/// Come si nomina un evento in un messaggio, senza usare la sua posizione.
///
/// Porta le forme **dichiarate**: autore non normalizzato e istante con il suo
/// offset. Non e' richiesta unica: la determinatezza dei messaggi viene dalla
/// scelta del primo evento in **ordine canonico**, non dall'unicita' di questa
/// chiave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChaveEvento {
    pub chave_origem: String,
    pub conta_id: String,
    pub ordinal: u32,
    pub operacao: OperacaoRevisao,
    pub autor: String,
    pub instante: DateTime<FixedOffset>,
}

impl fmt::Display for ChaveEvento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} em {}/{}#{} por {:?} as {}",
            self.operacao.rotulo(),
            self.chave_origem,
            self.conta_id,
            self.ordinal,
            self.autor,
            self.instante.to_rfc3339()
        )
    }
}

impl EventoRevisao {
    /// Restituita in `Box` perche' compare solo negli errori: tenere
    /// l'enum piccolo evita un `Result` sbilanciato.
    fn chave(&self) -> Box<ChaveEvento> {
        Box::new(self.chave_interna())
    }

    fn chave_interna(&self) -> ChaveEvento {
        ChaveEvento {
            chave_origem: self.alvo.chave_origem.clone(),
            conta_id: self.alvo.conta_id.clone(),
            ordinal: self.alvo.ordinal,
            operacao: self.operacao,
            autor: self.autor.clone(),
            instante: self.instante,
        }
    }
}

// ---------------------------------------------------------------------------
// Codifica esatta: l'unica identita'
// ---------------------------------------------------------------------------

/// `str_exato`: lunghezza in byte dei **byte originali**, senza NFC.
fn str_exato(texto: &str) -> String {
    format!("{}:{}", texto.len(), texto)
}

fn presenca_exata(valor: Option<&str>) -> String {
    match valor {
        None => "AUSENTE".to_string(),
        Some(v) => format!("PRESENTE:{}", str_exato(v)),
    }
}

/// `dado_esatto_string`: conserva **tutti** i dati tipizzati di un
/// [`Dado<String>`] — stato, fonte, presenza e valore, presenza e motivo — con
/// tag di presenza e lunghezze esplicite, cosi' `None` non puo' coincidere con
/// `Some`.
///
/// Non e' la produzione `dado` dell'impronta tecnica, che normalizza in NFC ed
/// **esclude** il motivo: riusarla qui perderebbe informazione e farebbe
/// sembrare identici due eventi diversi.
fn dado_esatto_string(valor: &Dado<String>) -> String {
    format!(
        "DADOX:{}:{}:{}:{}",
        valor.status().rotulo(),
        str_exato(valor.fonte()),
        presenca_exata(valor.valor().map(|v| v.as_str())),
        presenca_exata(valor.motivo())
    )
}

/// Istante dichiarato, in forma **iniettiva**: orario locale con i
/// nanosecondi, piu' l'offset in **secondi**.
///
/// Riscontro R3C3A-02 dell'audit del 2026-10-21: `to_rfc3339_opts` rappresenta
/// l'offset ai **minuti** — `SecondsFormat::Nanos` riguarda la frazione
/// dell'orario, non la precisione dell'offset — mentre `FixedOffset` ammette
/// offset con i secondi. Due dichiarazioni con offset di 1 e 2 secondi
/// producevano lo stesso testo e diventavano una falsa `Repeticao`. Qui
/// l'offset viene scritto come numero intero di secondi, quindi nessuna
/// dichiarazione rappresentabile dal tipo pubblico va perduta.
fn instante_exato(instante: DateTime<FixedOffset>) -> String {
    format!(
        "{}OFF{}",
        instante.naive_local().format("%Y-%m-%dT%H:%M:%S%.9f"),
        instante.offset().local_minus_utc()
    )
}

fn sequencia_exata(sequencia: Option<u64>) -> String {
    match sequencia {
        None => "SEM_SEQUENCIA".to_string(),
        Some(n) => n.to_string(),
    }
}

/// Codifica esatta e **iniettiva** dei dati dichiarati di un evento.
///
/// Ogni campo di [`EventoRevisao`] compare, con tag di presenza dove e'
/// opzionale e lunghezza esplicita dove e' testo. **Ogni campo aggiunto in
/// futuro all'evento va aggiunto anche qui** (F3C-46): altrimenti due eventi
/// diversi tornerebbero a sembrare identici.
///
/// Esposta perche' un test deve poter confrontare i byte, non solo l'esito.
pub fn texto_exato_evento(evento: &EventoRevisao) -> String {
    let mut saida = String::with_capacity(512);
    saida.push_str(VERSAO_EVENTO_EXATO);
    saida.push('\n');
    saida.push_str(&format!("ACH={}\n", str_exato(&evento.alvo.chave_origem)));
    saida.push_str(&format!(
        "AIM={}\n",
        dado_esatto_string(&evento.alvo.impressao_conteudo)
    ));
    saida.push_str(&format!("AOG={}\n", rotulo_origem(evento.alvo.origem)));
    saida.push_str(&format!("ACI={}\n", str_exato(&evento.alvo.conta_id)));
    saida.push_str(&format!("AOR={}\n", evento.alvo.ordinal));
    saida.push_str(&format!("EOP={}\n", evento.operacao.rotulo()));
    saida.push_str(&format!("EAU={}\n", str_exato(&evento.autor)));
    saida.push_str(&format!(
        "EIN={}\n",
        str_exato(&instante_exato(evento.instante))
    ));
    saida.push_str(&format!("ESQ={}\n", sequencia_exata(evento.sequencia)));
    saida.push_str(&format!("EMO={}\n", str_exato(&evento.motivo)));
    saida.push_str(&format!("ENO={}\n", presenca_exata(evento.nota.as_deref())));
    saida.push_str(&format!("EVI={}\n", dado_esatto_string(&evento.vista)));
    saida
}

fn rotulo_origem(origem: OrigemLancamento) -> &'static str {
    match origem {
        OrigemLancamento::XmlNfe => "XML_NFE",
        OrigemLancamento::Csv => "CSV",
        OrigemLancamento::Manual => "MANUAL",
    }
}

// ---------------------------------------------------------------------------
// Ordine canonico
// ---------------------------------------------------------------------------

/// Chiave di ordinamento canonico a sei livelli (F3C-44).
///
/// `Ord` derivato confronta i campi nell'ordine di dichiarazione, e le
/// `String` di Rust si confrontano byte per byte sull'UTF-8: e' esattamente il
/// confronto ordinale richiesto. La specifica proponeva una firma che
/// restituiva una stringa unica; una chiave strutturata realizza lo stesso
/// ordine in modo verificabile campo per campo.
///
/// **Non** e' un criterio di identita': due eventi distinti possono coincidere
/// fino al livello 5. Il sesto livello, il testo esatto, li separa sempre.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChaveCanonica {
    // 1. istante assoluto: due fusi diversi dello stesso momento sono
    //    contemporanei, ed e' giusto cosi'.
    instante: DateTime<Utc>,
    // 2. le sequenze dichiarate precedono l'assenza, e si ordinano per valore.
    //    Un'assenza non e' uno zero e non scalza un dato dichiarato.
    sem_sequencia: bool,
    sequencia: u64,
    // 3. bersaglio completo, con l'ordinale confrontato come numero.
    chave_origem: String,
    impressao_alvo: String,
    origem: &'static str,
    conta_id: String,
    ordinal: u32,
    // 4. operazione.
    operacao: &'static str,
    // 5. contenuto semantico, in NFC. `None` precede `Some`.
    autor_nfc: String,
    motivo_nfc: String,
    nota_nfc: Option<String>,
    vista_semantica: String,
    // 6. testo esatto: rende l'ordine totale sugli eventi superstiti.
    texto_exato: String,
}

/// Chiave canonica di un evento.
pub fn chave_canonica(evento: &EventoRevisao) -> ChaveCanonica {
    ChaveCanonica {
        instante: evento.instante.with_timezone(&Utc),
        sem_sequencia: evento.sequencia.is_none(),
        sequencia: evento.sequencia.unwrap_or(0),
        chave_origem: evento.alvo.chave_origem.clone(),
        impressao_alvo: dado_esatto_string(&evento.alvo.impressao_conteudo),
        origem: rotulo_origem(evento.alvo.origem),
        conta_id: evento.alvo.conta_id.clone(),
        ordinal: evento.alvo.ordinal,
        operacao: evento.operacao.rotulo(),
        autor_nfc: impressao::nfc(&evento.autor),
        motivo_nfc: impressao::nfc(&evento.motivo),
        nota_nfc: evento.nota.as_deref().map(impressao::nfc),
        // Livello semantico: la produzione tecnica, che normalizza ed esclude
        // il motivo. Il motivo distingue al livello 6, non qui.
        vista_semantica: impressao::dado(&evento.vista, |v| impressao::nfc(v)),
        texto_exato: texto_exato_evento(evento),
    }
}

// ---------------------------------------------------------------------------
// Uscita
// ---------------------------------------------------------------------------

/// Stato della **vista** di una riga: esito tecnico e decisione umana
/// combinati, senza confonderli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EstadoRevisao {
    /// Esito tecnico `OK` della 3B, nessuna decisione umana.
    TecnicoOk,
    /// Esito tecnico `REVISAR` della 3B, nessuna decisione umana.
    TecnicoRevisar,
    /// Una persona ha approvato, con evento in traccia.
    AprovadoPorPessoa,
    /// Un'approvazione e' stata revocata: lo stato tecnico e' ripristinato e
    /// l'approvazione precedente resta in traccia (D9.3).
    AprovacaoRevogada,
    /// `APROVADO` in ingresso senza evento che lo giustifichi (D9.4).
    AprovadoSemProveniencia,
}

impl EstadoRevisao {
    pub const fn rotulo(self) -> &'static str {
        match self {
            EstadoRevisao::TecnicoOk => "TECNICO_OK",
            EstadoRevisao::TecnicoRevisar => "TECNICO_REVISAR",
            EstadoRevisao::AprovadoPorPessoa => "APROVADO_POR_PESSOA",
            EstadoRevisao::AprovacaoRevogada => "APROVACAO_REVOGADA",
            EstadoRevisao::AprovadoSemProveniencia => "APROVADO_SEM_PROVENIENCIA",
        }
    }
}

/// Un evento applicato. Porta **l'evento**, non la sua posizione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisaoAplicada {
    pub evento: EventoRevisao,
    /// Quante volte lo **stesso** evento e' arrivato, secondo l'identita'
    /// esatta: `0` se unico. Due dichiarazioni diverse che si somigliano non
    /// si contano qui: restano due decisioni.
    pub repeticoes: usize,
}

/// Perche' un evento non e' stato applicato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MotivoNaoAplicacao {
    /// Il documento o la riga non esistono in questo lotto.
    AlvoAusente,
    /// L'impronta citata non e' quella della versione corrente.
    VersaoSuperada,
    /// Il bersaglio non dichiara l'impronta esaminata.
    ProvenienciaAusente,
    /// Revoca senza un'approvazione applicata da revocare.
    NadaARevogar,
    /// Ripetizione esatta di un evento gia' applicato.
    Repeticao,
}

impl MotivoNaoAplicacao {
    pub const fn rotulo(self) -> &'static str {
        match self {
            MotivoNaoAplicacao::AlvoAusente => "ALVO_AUSENTE",
            MotivoNaoAplicacao::VersaoSuperada => "VERSAO_SUPERADA",
            MotivoNaoAplicacao::ProvenienciaAusente => "PROVENIENCIA_AUSENTE",
            MotivoNaoAplicacao::NadaARevogar => "NADA_A_REVOGAR",
            MotivoNaoAplicacao::Repeticao => "REPETICAO",
        }
    }
}

/// Un evento non applicato, **conservato e consultabile**: documenta che
/// qualcuno ha deciso qualcosa, anche se quella decisione non si applica a
/// questo lotto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventoNaoAplicado {
    pub evento: EventoRevisao,
    pub motivo: MotivoNaoAplicacao,
    pub detalhe: String,
}

/// Perche' nasce una nuova versione di un documento. Rinviato: senza archivio
/// non e' calcolabile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MotivoNovaVersao {
    Reimportacao,
    Correcao,
    RegrasMudadas,
    Manual,
}

/// Versione di un documento: l'impronta e' calcolabile, la **sequenza** no.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VersaoDocumento {
    pub chave_origem: String,
    pub impressao_conteudo: String,
    pub impressao_apresentacao: String,
    /// D8.5: `NAO_DISPONIVEL` finche' non esiste un archivio. Nessun numero
    /// inventato consultando l'ambiente.
    pub sequencia: Dado<u32>,
    pub motivo: Dado<MotivoNovaVersao>,
}

/// Una riga del pre-libro con la sua storia di revisione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinhaRevisada {
    pub chave: ChaveRevisao,
    /// Identita' nel lotto, dalla 3C.2: conserva la provenienza
    /// dell'esecuzione.
    pub identidade: IdentidadeLinha,
    /// L'esito della 3B, **mai** riscritto.
    pub status_tecnico: StatusRevisao,
    pub estado: EstadoRevisao,
    /// Approvazioni e revoche applicate, in ordine canonico.
    pub decisoes: Vec<DecisaoAplicada>,
    /// Annotazioni applicate, in ordine canonico.
    pub anotacoes: Vec<DecisaoAplicada>,
    pub no_extrato_revisado: bool,
}

/// Vista revisionata di un lotto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VistaRevisada {
    /// Pre-libro **proiettato**: riepiloghi, partizioni ed esclusioni
    /// coerenti con le decisioni applicate. Costruito da
    /// [`prelivro::construir_pre_livro`], senza duplicare nessuna aritmetica.
    pub pre_livro: PreLivro,
    pub linhas: Vec<LinhaRevisada>,
    pub versoes: Vec<VersaoDocumento>,
    /// Storia completa in ordine canonico: niente viene cancellato.
    pub trilha: Vec<DecisaoAplicada>,
    pub eventos_nao_aplicados: Vec<EventoNaoAplicado>,
    /// Righe che l'estratto **revisionato** esclude e che l'estratto della
    /// 3C.2 conterrebbe: la differenza fra i due insiemi, con il motivo.
    pub exclusoes_extrato_revisado: Vec<Exclusao>,
    pub avisos: Vec<Aviso>,
}

impl VistaRevisada {
    /// Estratto revisionato: piu' severo di
    /// [`PreLivro::extrato_aprovado`], perche' esclude anche le righe
    /// `APROVADO` senza provenienza (D9.4).
    ///
    /// E' sempre un sottoinsieme dell'estratto del pre-libro proiettato, e
    /// ogni differenza compare in [`Self::exclusoes_extrato_revisado`].
    pub fn extrato_revisado(&self) -> Vec<&LinhaRevisada> {
        self.linhas
            .iter()
            .filter(|linha| linha.no_extrato_revisado)
            .collect()
    }

    /// Righe in un dato stato della vista, nell'ordine deterministico del
    /// pre-libro.
    pub fn linhas_no_estado(&self, estado: EstadoRevisao) -> Vec<&LinhaRevisada> {
        self.linhas
            .iter()
            .filter(|linha| linha.estado == estado)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore che impedisce di produrre la vista. Nessun risultato parziale.
#[derive(Debug)]
pub enum ErroRevisao {
    /// Autore vuoto: non esiste un autore predefinito.
    AutorNaoDeclarado { evento: Box<ChaveEvento> },
    /// Motivo vuoto: non esiste un motivo predefinito.
    MotivoNaoDeclarado { evento: Box<ChaveEvento> },
    /// `Anotar` senza nota.
    NotaNaoDeclarada { evento: Box<ChaveEvento> },
    /// Bersaglio incompleto: senza chiave o senza conto non e' un bersaglio.
    AlvoIncompleto {
        evento: Box<ChaveEvento>,
        campo: &'static str,
    },
    /// Più di una riga corrisponde al bersaglio: conflitto dichiarato, non si
    /// sceglie la prima.
    AlvoAmbiguo {
        evento: Box<ChaveEvento>,
        chave_origem: String,
        encontrados: usize,
    },
    /// Due decisioni incompatibili sullo stesso bersaglio e alla stessa
    /// coordinata temporale: non sono ordinabili, e inventare un ordine
    /// sarebbe peggio che fermarsi.
    DecisoesIncompativeis {
        primeiro: Box<ChaveEvento>,
        segundo: Box<ChaveEvento>,
        detalhe: String,
    },
    /// L'ingresso non e' un pre-libro valido: l'errore della 3C.2 viene
    /// propagato, non riscritto.
    PreLivro(ErroPreLivro),
}

impl fmt::Display for ErroRevisao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroRevisao::AutorNaoDeclarado { evento } => write!(
                f,
                "evento sem autor declarado ({evento}): \
                 nenhum autor padrao e' inventado pelo sistema"
            ),
            ErroRevisao::MotivoNaoDeclarado { evento } => write!(
                f,
                "evento sem motivo declarado ({evento}): \
                 nenhum motivo padrao e' inventado pelo sistema"
            ),
            ErroRevisao::NotaNaoDeclarada { evento } => write!(
                f,
                "anotacao sem nota ({evento}): \
                 uma anotacao vazia nao e' uma informacao"
            ),
            ErroRevisao::AlvoIncompleto { evento, campo } => write!(
                f,
                "alvo incompleto em {campo} ({evento}): \
                 sem ele a decisao nao identifica uma linha"
            ),
            ErroRevisao::AlvoAmbiguo {
                evento,
                chave_origem,
                encontrados,
            } => write!(
                f,
                "alvo ambiguo em {chave_origem}: {encontrados} linhas correspondem \
                 ({evento}). Nenhuma e' escolhida"
            ),
            ErroRevisao::DecisoesIncompativeis {
                primeiro,
                segundo,
                detalhe,
            } => write!(
                f,
                "decisoes incompativeis no mesmo alvo: {primeiro} e {segundo} ({detalhe}). \
                 Nenhuma ordem e' deduzida da posicao no array"
            ),
            ErroRevisao::PreLivro(erro) => write!(f, "pre-livro invalido: {erro}"),
        }
    }
}

impl std::error::Error for ErroRevisao {}

// ---------------------------------------------------------------------------
// Applicazione
// ---------------------------------------------------------------------------

/// Evento superstite al collasso, con il numero di ripetizioni esatte.
struct EventoUnico<'a> {
    evento: &'a EventoRevisao,
    repeticoes: usize,
}

/// Coordinate di una riga nel lotto ricevuto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PosicaoLinha {
    documento: usize,
    lancamento: usize,
}

/// Stato di lavoro di una riga mentre si applicano gli eventi.
struct EstadoLinha<'a> {
    status_tecnico: StatusRevisao,
    estado: EstadoRevisao,
    decisoes: Vec<DecisaoAplicada>,
    anotacoes: Vec<DecisaoAplicada>,
    eventos: Vec<&'a EventoRevisao>,
}

/// Applica gli eventi di revisione a un lotto e costruisce la vista.
///
/// Quattro passate, nell'ordine (§5.3 della specifica):
///
/// 0. **ordina** con la chiave canonica completa e collassa le ripetizioni con
///    dati dichiarati **identici**. Da qui in avanti la posizione nell'array
///    non esiste piu', nemmeno per scegliere quale errore riportare.
/// 1. **valida** la forma di ogni evento. Il primo errore in ordine canonico
///    interrompe: nessuna vista viene prodotta.
/// 2. **abbina** i bersagli alle righe, usando le impronte dei documenti
///    **originali**, e dichiara conflitti e ambiguita'.
/// 3. **proietta** una copia dei contesti con il solo `status_revisao`
///    aggiornato, ricostruisce il pre-libro e assembla la vista.
///
/// Gli ingressi sono immutabili: in errore il chiamante resta esattamente come
/// era, perche' non esiste nessuno stato intermedio osservabile.
pub fn aplicar_revisao(
    documentos: &[ContextoDocumento],
    plano: &PlanoContas,
    eventos: &[EventoRevisao],
) -> Result<VistaRevisada, ErroRevisao> {
    // --- passata 0: ordine canonico e collasso delle ripetizioni esatte ----
    let mut ordenados: Vec<&EventoRevisao> = eventos.iter().collect();
    ordenados.sort_by_cached_key(|e| chave_canonica(e));

    let mut unicos: Vec<EventoUnico> = Vec::with_capacity(ordenados.len());
    let mut repetidos: Vec<&EventoRevisao> = Vec::new();
    for evento in ordenados {
        let texto = texto_exato_evento(evento);
        match unicos
            .last_mut()
            .filter(|u| texto_exato_evento(u.evento) == texto)
        {
            Some(anterior) => {
                anterior.repeticoes += 1;
                repetidos.push(evento);
            }
            None => unicos.push(EventoUnico {
                evento,
                repeticoes: 0,
            }),
        }
    }

    // --- passata 1: validazione della forma ---------------------------------
    for unico in &unicos {
        validar(unico.evento)?;
    }

    // --- passata 2: il lotto deve essere un pre-libro valido ---------------
    // Chiamare la 3C.2 sugli originali e' anche la validazione dell'ingresso:
    // un abbinamento non puo' rendere valido cio' che `construir_pre_livro`
    // rifiuta.
    let base = prelivro::construir_pre_livro(documentos, plano).map_err(ErroRevisao::PreLivro)?;

    // Impronte dei documenti **originali**, prima di qualunque proiezione:
    // approvare non deve cambiare l'impronta dell'oggetto approvato.
    let mut impressoes: Vec<(String, String)> = Vec::with_capacity(documentos.len());
    for contexto in documentos {
        let conteudo = impressao::impressao_conteudo(contexto);
        let contas = impressao::contas_mostradas(contexto, plano).map_err(|erro| {
            ErroRevisao::PreLivro(ErroPreLivro::ContaNaoEncontradaNoPlano {
                chave_origem: erro.chave_origem,
                conta_id: erro.conta_id,
            })
        })?;
        let apresentacao = impressao::impressao_apresentacao(&conteudo, &contas);
        impressoes.push((conteudo, apresentacao));
    }

    // Indice dei bersagli: (chiave, impronta, origem, conta, ordinal).
    let mut estados: BTreeMap<PosicaoLinha, EstadoLinha> = BTreeMap::new();
    for (d, contexto) in documentos.iter().enumerate() {
        for (l, lancamento) in contexto.alocacao.lancamentos.iter().enumerate() {
            let estado = match lancamento.status_revisao {
                StatusRevisao::Ok => EstadoRevisao::TecnicoOk,
                StatusRevisao::Revisar => EstadoRevisao::TecnicoRevisar,
                // La 3B non produce `APROVADO`: senza un evento che lo
                // giustifichi la provenienza non esiste (D9.4).
                StatusRevisao::Aprovado => EstadoRevisao::AprovadoSemProveniencia,
            };
            estados.insert(
                PosicaoLinha {
                    documento: d,
                    lancamento: l,
                },
                EstadoLinha {
                    status_tecnico: lancamento.status_revisao,
                    estado,
                    decisoes: Vec::new(),
                    anotacoes: Vec::new(),
                    eventos: Vec::new(),
                },
            );
        }
    }

    // --- passata 2b: abbinamento -------------------------------------------
    let mut nao_aplicados: Vec<EventoNaoAplicado> = Vec::new();
    let mut aplicaveis: Vec<(PosicaoLinha, &EventoUnico)> = Vec::new();

    for unico in &unicos {
        let evento = unico.evento;
        let Some(impressao_citada) = evento.alvo.impressao_conteudo.valor() else {
            nao_aplicados.push(EventoNaoAplicado {
                evento: evento.clone(),
                motivo: MotivoNaoAplicacao::ProvenienciaAusente,
                detalhe: format!(
                    "o alvo nao declara a impressao examinada (status {})",
                    evento.alvo.impressao_conteudo.status().rotulo()
                ),
            });
            continue;
        };

        let mut correspondencias: Vec<PosicaoLinha> = Vec::new();
        let mut documento_existe = false;
        let mut impressao_atual: Option<&str> = None;
        for (d, contexto) in documentos.iter().enumerate() {
            if contexto.chave_origem != evento.alvo.chave_origem {
                continue;
            }
            documento_existe = true;
            impressao_atual = Some(&impressoes[d].0);
            if impressoes[d].0 != *impressao_citada {
                continue;
            }
            for (l, lancamento) in contexto.alocacao.lancamentos.iter().enumerate() {
                if lancamento.origem == evento.alvo.origem
                    && lancamento.conta_id == evento.alvo.conta_id
                    && l as u32 == evento.alvo.ordinal
                {
                    correspondencias.push(PosicaoLinha {
                        documento: d,
                        lancamento: l,
                    });
                }
            }
        }

        match correspondencias.len() {
            1 => aplicaveis.push((correspondencias[0], unico)),
            0 => {
                let (motivo, detalhe) = if !documento_existe {
                    (
                        MotivoNaoAplicacao::AlvoAusente,
                        format!("documento {} ausente deste lote", evento.alvo.chave_origem),
                    )
                } else if impressao_atual != Some(impressao_citada.as_str()) {
                    (
                        MotivoNaoAplicacao::VersaoSuperada,
                        format!(
                            "a impressao citada {} nao e' a da versao corrente {}",
                            impressao_citada,
                            impressao_atual.unwrap_or("-")
                        ),
                    )
                } else {
                    (
                        MotivoNaoAplicacao::AlvoAusente,
                        format!(
                            "linha {}/{}#{} ausente nesta versao",
                            evento.alvo.chave_origem, evento.alvo.conta_id, evento.alvo.ordinal
                        ),
                    )
                };
                nao_aplicados.push(EventoNaoAplicado {
                    evento: evento.clone(),
                    motivo,
                    detalhe,
                });
            }
            encontrados => {
                return Err(ErroRevisao::AlvoAmbiguo {
                    evento: evento.chave(),
                    chave_origem: evento.alvo.chave_origem.clone(),
                    encontrados,
                });
            }
        }
    }

    for (posicao, unico) in &aplicaveis {
        if let Some(estado) = estados.get_mut(posicao) {
            estado.eventos.push(unico.evento);
        }
    }

    // Conflitti: per **bersaglio**, e solo fra operazioni incompatibili.
    //
    // Riscontro R3C3A-03: la scelta del conflitto da riportare deve seguire
    // l'ordine canonico degli **eventi**. Prima attraversava
    // `estados.values()`, ordinato per `PosicaoLinha`, che contiene l'indice
    // del documento ricevuto: permutare i documenti cambiava quale conflitto
    // veniva nominato, proprio dove F3C-44 esclude la posizione d'ingresso.
    let mut conflitos: Vec<(&EventoRevisao, &EventoRevisao)> = Vec::new();
    for estado in estados.values() {
        conflitos.extend(conflitos_do_alvo(&estado.eventos));
    }
    conflitos.sort_by_cached_key(|(primeiro, segundo)| {
        (chave_canonica(primeiro), chave_canonica(segundo))
    });
    if let Some((primeiro, segundo)) = conflitos.first() {
        return Err(ErroRevisao::DecisoesIncompativeis {
            primeiro: primeiro.chave(),
            segundo: segundo.chave(),
            detalhe: format!(
                "mesmo instante absoluto e sequencias {:?} e {:?}",
                primeiro.sequencia, segundo.sequencia
            ),
        });
    }

    // --- passata 2c: applicazione degli eventi ------------------------------
    let mut avisos: Vec<Aviso> = Vec::new();
    for (posicao, unico) in &aplicaveis {
        let evento = unico.evento;
        let estado = estados.get_mut(posicao).expect("posizione indicizzata");
        let decisao = DecisaoAplicada {
            evento: evento.clone(),
            repeticoes: unico.repeticoes,
        };
        match evento.operacao {
            OperacaoRevisao::Anotar => estado.anotacoes.push(decisao),
            OperacaoRevisao::Aprovar => {
                if estado.estado == EstadoRevisao::AprovadoPorPessoa {
                    avisos.push(aviso(
                        avisos::APROVACAO_REPETIDA,
                        format!(
                            "linha {}/{}#{} ja' estava aprovada: a segunda aprovacao fica \
                             na trilha e nao substitui a primeira",
                            evento.alvo.chave_origem, evento.alvo.conta_id, evento.alvo.ordinal
                        ),
                    ));
                } else {
                    estado.estado = EstadoRevisao::AprovadoPorPessoa;
                }
                estado.decisoes.push(decisao);
            }
            OperacaoRevisao::Revogar => {
                if estado.estado == EstadoRevisao::AprovadoPorPessoa {
                    estado.estado = EstadoRevisao::AprovacaoRevogada;
                    estado.decisoes.push(decisao);
                    avisos.push(aviso(
                        avisos::APROVACAO_REVOGADA,
                        format!(
                            "linha {}/{}#{}: aprovacao revogada, estado tecnico {} \
                             restaurado; a aprovacao anterior permanece na trilha",
                            evento.alvo.chave_origem,
                            evento.alvo.conta_id,
                            evento.alvo.ordinal,
                            impressao::status_revisao(estado.status_tecnico)
                        ),
                    ));
                } else {
                    nao_aplicados.push(EventoNaoAplicado {
                        evento: evento.clone(),
                        motivo: MotivoNaoAplicacao::NadaARevogar,
                        detalhe: format!(
                            "linha {}/{}#{} no estado {}: nenhuma aprovacao aplicada a revogar",
                            evento.alvo.chave_origem,
                            evento.alvo.conta_id,
                            evento.alvo.ordinal,
                            estado.estado.rotulo()
                        ),
                    });
                }
            }
        }
    }

    // Le ripetizioni esatte restano consultabili, oltre che contate.
    for evento in repetidos {
        nao_aplicados.push(EventoNaoAplicado {
            evento: evento.clone(),
            motivo: MotivoNaoAplicacao::Repeticao,
            detalhe: "dados declarados identicos a um evento ja' aplicado".to_string(),
        });
    }

    // --- passata 3: proiezione ---------------------------------------------
    // L'unico campo che la copia scrive e' `status_revisao`.
    let mut mudou = false;
    let mut projetados: Vec<ContextoDocumento> = documentos.to_vec();
    for (posicao, estado) in &estados {
        let novo = status_projetado(estado.estado, estado.status_tecnico);
        let linha = &mut projetados[posicao.documento].alocacao.lancamentos[posicao.lancamento];
        if linha.status_revisao != novo {
            linha.status_revisao = novo;
            mudou = true;
        }
    }
    let pre_livro = if mudou {
        prelivro::construir_pre_livro(&projetados, plano).map_err(ErroRevisao::PreLivro)?
    } else {
        base
    };

    // --- assemblaggio della vista ------------------------------------------
    let mut posicao_por_identidade: BTreeMap<(String, String, u32, &'static str), PosicaoLinha> =
        BTreeMap::new();
    for (d, contexto) in documentos.iter().enumerate() {
        for (l, lancamento) in contexto.alocacao.lancamentos.iter().enumerate() {
            posicao_por_identidade.insert(
                (
                    contexto.chave_origem.clone(),
                    lancamento.conta_id.clone(),
                    l as u32,
                    rotulo_origem(lancamento.origem),
                ),
                PosicaoLinha {
                    documento: d,
                    lancamento: l,
                },
            );
        }
    }

    let mut linhas: Vec<LinhaRevisada> = Vec::with_capacity(pre_livro.linhas.len());
    let mut exclusoes: Vec<Exclusao> = Vec::new();
    for linha in &pre_livro.linhas {
        let chave = (
            linha.identidade.chave_origem.clone(),
            linha.identidade.conta_id.clone(),
            linha.identidade.ordinal,
            rotulo_origem(linha.identidade.origem),
        );
        let posicao = *posicao_por_identidade
            .get(&chave)
            .expect("ogni riga del pre-libro viene da una riga ricevuta");
        let estado = &estados[&posicao];
        let indice_documento = posicao.documento;

        let sem_proveniencia = estado.estado == EstadoRevisao::AprovadoSemProveniencia;
        // R3C3A-01: entrare nell'estratto revisionato richiede
        // un'approvazione umana **attualmente attiva**, non la semplice
        // assenza di `APROVADO_SEM_PROVENIENCIA`. Dopo la revoca di una
        // sanatoria lo stato tecnico ripristinato e' `APROVADO`, che da solo
        // riabilitava l'estratto senza nessuna decisione attiva.
        let aprovacao_ativa = estado.estado == EstadoRevisao::AprovadoPorPessoa;
        let no_extrato_revisado = linha.no_extrato_aprovado && aprovacao_ativa;
        if linha.no_extrato_aprovado && !no_extrato_revisado {
            let motivo = match estado.estado {
                EstadoRevisao::AprovacaoRevogada => format!(
                    "linha {}: nenhuma aprovacao humana ativa. O APROVADO que resta e' o \
                     dado tecnico de origem, sem evento de proveniencia, logo nao entra \
                     no extrato revisado",
                    EstadoRevisao::AprovacaoRevogada.rotulo()
                ),
                outro => format!(
                    "linha {outro:?} sem evento de proveniencia ({}): nao e' uma revisao \
                     verificada, logo nao entra no extrato revisado",
                    outro.rotulo()
                ),
            };
            exclusoes.push(Exclusao {
                identidade: linha.identidade.clone(),
                valor: linha.lancamento.valor,
                particao: linha.particao,
                stato_documento: linha.stato_documento,
                motivo,
            });
        }
        if sem_proveniencia {
            avisos.push(aviso(
                avisos::APROVACAO_SEM_PROVENIENCIA,
                format!(
                    "linha {}/{}#{} chegou APROVADO sem evento declarado: dado original \
                     conservado, fora do extrato revisado",
                    linha.identidade.chave_origem,
                    linha.identidade.conta_id,
                    linha.identidade.ordinal
                ),
            ));
        }
        if estado.estado == EstadoRevisao::AprovadoPorPessoa
            && linha.stato_documento == StatoDocumento::Erro
        {
            avisos.push(aviso(
                avisos::APROVACAO_EM_DOCUMENTO_ERRO,
                format!(
                    "aprovacao registrada em documento com estado ERRO ({}): \
                     conservada na trilha e fora do extrato",
                    linha.identidade.chave_origem
                ),
            ));
        }
        // D8.3: il piano mostra oggi nomi diversi da quelli esaminati.
        for decisao in estado.decisoes.iter().chain(estado.anotacoes.iter()) {
            if let Some(vista) = decisao.evento.vista.valor()
                && *vista != impressoes[indice_documento].1
            {
                avisos.push(aviso(
                    avisos::APRESENTACAO_MUDADA,
                    format!(
                        "documento {}: a apresentacao examinada {} nao e' a atual {}; \
                         as decisoes permanecem",
                        linha.identidade.chave_origem, vista, impressoes[indice_documento].1
                    ),
                ));
            }
        }

        linhas.push(LinhaRevisada {
            chave: ChaveRevisao {
                chave_origem: linha.identidade.chave_origem.clone(),
                impressao_conteudo: Dado::presente(
                    impressoes[indice_documento].0.clone(),
                    "revisao:impressao_conteudo",
                )
                .expect("fonte non vuota e valore presente"),
                origem: linha.identidade.origem,
                conta_id: linha.identidade.conta_id.clone(),
                ordinal: linha.identidade.ordinal,
            },
            identidade: linha.identidade.clone(),
            status_tecnico: estado.status_tecnico,
            estado: estado.estado,
            decisoes: estado.decisoes.clone(),
            anotacoes: estado.anotacoes.clone(),
            no_extrato_revisado,
        });
    }

    // Versioni: una per documento, con l'impronta calcolata sugli originali.
    let mut versoes: Vec<VersaoDocumento> = Vec::with_capacity(documentos.len());
    for (d, contexto) in documentos.iter().enumerate() {
        versoes.push(VersaoDocumento {
            chave_origem: contexto.chave_origem.clone(),
            impressao_conteudo: impressoes[d].0.clone(),
            impressao_apresentacao: impressoes[d].1.clone(),
            sequencia: Dado::ausente(
                "arquivo autorevole ausente (3C.4)",
                StatusDado::NaoDisponivel,
                "a sequencia historica exige um arquivo que este bloco nao cria",
            )
            .expect("fonte e motivo non vuoti"),
            motivo: Dado::ausente(
                "arquivo autorevole ausente (3C.4)",
                StatusDado::NaoDisponivel,
                "sem historico nao se sabe por que esta versao nasceu",
            )
            .expect("fonte e motivo non vuoti"),
        });
    }
    versoes.sort_by(|a, b| a.chave_origem.cmp(&b.chave_origem));

    // Traccia globale in ordine canonico: una sola regola d'ordine, e le
    // collezioni di riga ne sono sottosequenze.
    let mut trilha: Vec<DecisaoAplicada> = linhas
        .iter()
        .flat_map(|l| l.decisoes.iter().chain(l.anotacoes.iter()).cloned())
        .collect();
    trilha.sort_by_cached_key(|d| chave_canonica(&d.evento));

    nao_aplicados.sort_by_cached_key(|e| chave_canonica(&e.evento));
    ordenar_avisos(&mut avisos);
    exclusoes.sort_by_key(chave_exclusao);

    Ok(VistaRevisada {
        pre_livro,
        linhas,
        versoes,
        trilha,
        eventos_nao_aplicados: nao_aplicados,
        exclusoes_extrato_revisado: exclusoes,
        avisos,
    })
}

fn chave_exclusao(exclusao: &Exclusao) -> (String, String, u32) {
    (
        exclusao.identidade.chave_origem.clone(),
        exclusao.identidade.conta_id.clone(),
        exclusao.identidade.ordinal,
    )
}

/// D9.3: la revoca riporta allo stato **tecnico**, non a `REVISAR` d'ufficio.
fn status_projetado(estado: EstadoRevisao, tecnico: StatusRevisao) -> StatusRevisao {
    match estado {
        EstadoRevisao::AprovadoPorPessoa => StatusRevisao::Aprovado,
        EstadoRevisao::AprovacaoRevogada
        | EstadoRevisao::TecnicoOk
        | EstadoRevisao::TecnicoRevisar
        | EstadoRevisao::AprovadoSemProveniencia => tecnico,
    }
}

/// Validazione della **forma** di un evento (D9.5, F3C-32).
///
/// Nessun valore di default somiglia a un'approvazione valida: non esiste un
/// autore predefinito, non esiste un motivo predefinito.
fn validar(evento: &EventoRevisao) -> Result<(), ErroRevisao> {
    if evento.autor.trim().is_empty() {
        return Err(ErroRevisao::AutorNaoDeclarado {
            evento: evento.chave(),
        });
    }
    if evento.motivo.trim().is_empty() {
        return Err(ErroRevisao::MotivoNaoDeclarado {
            evento: evento.chave(),
        });
    }
    if evento.operacao == OperacaoRevisao::Anotar
        && evento
            .nota
            .as_deref()
            .map(|n| n.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(ErroRevisao::NotaNaoDeclarada {
            evento: evento.chave(),
        });
    }
    if evento.alvo.chave_origem.trim().is_empty() {
        return Err(ErroRevisao::AlvoIncompleto {
            evento: evento.chave(),
            campo: "chave_origem",
        });
    }
    if evento.alvo.conta_id.trim().is_empty() {
        return Err(ErroRevisao::AlvoIncompleto {
            evento: evento.chave(),
            campo: "conta_id",
        });
    }
    Ok(())
}

/// Conflitti fra operazioni incompatibili **sullo stesso bersaglio**.
///
/// A parita' di istante assoluto l'ordine e' deciso solo se **entrambi** gli
/// eventi dichiarano una sequenza e le due sequenze sono diverse. Il livello 2
/// della chiave canonica ordina sempre, ma ordinare non autorizza a dedurre
/// l'intenzione umana: per le operazioni incompatibili serve una coordinata
/// esplicita su entrambe.
/// Restituisce **tutte** le coppie in conflitto di un bersaglio, invece di
/// fermarsi alla prima: la scelta di quale riportare e' del chiamante, che la
/// fa in ordine canonico (R3C3A-03).
fn conflitos_do_alvo<'a>(
    eventos: &[&'a EventoRevisao],
) -> Vec<(&'a EventoRevisao, &'a EventoRevisao)> {
    let mut encontrados = Vec::new();
    for (i, primeiro) in eventos.iter().enumerate() {
        for segundo in eventos.iter().skip(i + 1) {
            if !primeiro.operacao.incompativel_com(segundo.operacao) {
                continue;
            }
            if primeiro.instante.with_timezone(&Utc) != segundo.instante.with_timezone(&Utc) {
                continue;
            }
            let ordenavel = match (primeiro.sequencia, segundo.sequencia) {
                (Some(a), Some(b)) => a != b,
                _ => false,
            };
            if !ordenavel {
                encontrados.push((*primeiro, *segundo));
            }
        }
    }
    encontrados
}

fn aviso(codigo: &str, detalhe: impl Into<String>) -> Aviso {
    Aviso {
        codigo: codigo.to_string(),
        detalhe: detalhe.into(),
    }
}

/// F3C-43 anche per gli avvisi della vista: chiavi normalizzate prima del
/// confronto ordinale.
fn ordenar_avisos(avisos: &mut [Aviso]) {
    avisos.sort_by_cached_key(|a| (impressao::nfc(&a.codigo), impressao::nfc(&a.detalhe)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alocacao::{
        AlocacaoNfe, CampoRinviato, Componente, EsitoComponenti, EsitoCopertura, EsitoRate,
        EsitoTotal, Reconciliacao, RuoloComponente,
    };
    use crate::modelos::{Competencia, Lancamento, TipoConta};
    use crate::prelivro::{IdentidadeLote, Particao};
    use chrono::{NaiveDate, TimeZone};
    use rust_decimal::Decimal;
    use std::path::Path;
    use std::str::FromStr;

    const CHAVE: &str = "29260512345678000195550010001256612345678901";
    const CHAVE2: &str = "29260512345678000195550010001256712345678901";
    const CNPJ: &str = "55746722000181";

    fn plano() -> PlanoContas {
        PlanoContas::carregar(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("config/plano_contas_gerencial.yaml"),
        )
        .unwrap()
    }

    fn dec(bruto: &str) -> Decimal {
        Decimal::from_str(bruto).unwrap()
    }

    fn data(dia: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, dia).unwrap()
    }

    fn instante(hora: u32, minuto: u32, offset_horas: i32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(offset_horas * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 21, hora, minuto, 0)
            .unwrap()
    }

    fn rinviato(campo: &str) -> CampoRinviato {
        CampoRinviato {
            campo: campo.to_string(),
            status: "NAO_DISPONIVEL".to_string(),
            fonte: "fase 3B".to_string(),
            motivo: "campo non popolato in questo blocco".to_string(),
        }
    }

    fn reconciliacao(total: &str, stato: StatoDocumento) -> Reconciliacao {
        Reconciliacao {
            valor_total_nfe: dec(total),
            soma_lancamentos: dec(total),
            esito_total: if stato == StatoDocumento::Erro {
                EsitoTotal::Erro
            } else {
                EsitoTotal::Ok
            },
            soma_rate_documento: Dado::ausente(
                "nfe:cobr/dup/vDup",
                StatusDado::NaoAplicavel,
                "NF-e sem duplicatas",
            )
            .unwrap(),
            soma_rate_allocate: Dado::ausente(
                "nfe:cobr/dup/vDup",
                StatusDado::NaoAplicavel,
                "NF-e sem duplicatas",
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
            esito_copertura: if stato == StatoDocumento::Parcial {
                EsitoCopertura::Parcial
            } else {
                EsitoCopertura::Ok
            },
            componenti: vec![Componente {
                nome: "vProd".to_string(),
                valor: Dado::presente(dec(total), "nfe:total/ICMSTot/vProd").unwrap(),
                ruolo: RuoloComponente::Base,
                usada_na_verificacao: true,
            }],
            esito_componenti: EsitoComponenti::Ok,
            campi_rinviati: vec![rinviato("centro_custo"), rinviato("observacoes")],
            avisos_documento: Vec::new(),
            stato_documento: stato,
        }
    }

    fn lancamento(
        chave: &str,
        conta_id: &str,
        conta_nome: &str,
        valor: &str,
        status: StatusRevisao,
        alertas: &str,
    ) -> Lancamento {
        Lancamento {
            data_documento: data(5),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: CNPJ.to_string(),
            nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
            fornecedor_ou_favorecido: "DISTRIBUIDORA OESA LTDA".to_string(),
            cnpj_cpf_favorecido: "12345678000195".to_string(),
            documento: "12566".to_string(),
            tipo_documento: "NFE".to_string(),
            conta_id: conta_id.to_string(),
            conta_nome: conta_nome.to_string(),
            grupo_conta: "Custos de Mercadorias e Insumos".to_string(),
            descricao_itens: format!("ITEM DE {conta_id} ({valor})"),
            valor: dec(valor),
            duplicatas: Vec::new(),
            origem: OrigemLancamento::XmlNfe,
            arquivo_origem: "samples/nfe.xml".to_string(),
            chave_origem: chave.to_string(),
            hash_registro: format!("hash de {conta_id}"),
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

    fn contexto(
        chave: &str,
        stato: StatoDocumento,
        total: &str,
        lancamentos: Vec<Lancamento>,
    ) -> ContextoDocumento {
        ContextoDocumento {
            chave_origem: chave.to_string(),
            numero: "12566".to_string(),
            serie: "1".to_string(),
            data_documento: data(5),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: CNPJ.to_string(),
            nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
            arquivo_origem: "samples/nfe.xml".to_string(),
            alocacao: AlocacaoNfe {
                lancamentos,
                reconciliacao: reconciliacao(total, stato),
            },
            avisos_leitura: Vec::new(),
            lote: IdentidadeLote {
                id: "lote-2026-05-A".to_string(),
                gerado_em: data(31),
                descricao: "esecuzione di prova".to_string(),
            },
        }
    }

    /// Documento con una riga sola, nello stato tecnico richiesto.
    fn um_documento(status: StatusRevisao, alertas: &str) -> ContextoDocumento {
        contexto(
            CHAVE,
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                CHAVE,
                "2.01.004",
                "Refrigerantes",
                "10.00",
                status,
                alertas,
            )],
        )
    }

    /// Documento con due righe su conti diversi.
    fn duas_linhas(chave: &str) -> ContextoDocumento {
        contexto(
            chave,
            StatoDocumento::Ok,
            "30.00",
            vec![
                lancamento(
                    chave,
                    "2.01.004",
                    "Refrigerantes",
                    "10.00",
                    StatusRevisao::Ok,
                    "",
                ),
                lancamento(
                    chave,
                    "2.01.002",
                    "Vinhos e Espumantes",
                    "20.00",
                    StatusRevisao::Ok,
                    "",
                ),
            ],
        )
    }

    /// Bersaglio costruito dall'impronta **reale** del documento.
    fn alvo(doc: &ContextoDocumento, conta_id: &str, ordinal: u32) -> ChaveRevisao {
        ChaveRevisao {
            chave_origem: doc.chave_origem.clone(),
            impressao_conteudo: Dado::presente(
                impressao::impressao_conteudo(doc),
                "revisao:conteudo",
            )
            .unwrap(),
            origem: OrigemLancamento::XmlNfe,
            conta_id: conta_id.to_string(),
            ordinal,
        }
    }

    fn evento(
        alvo: ChaveRevisao,
        operacao: OperacaoRevisao,
        autor: &str,
        instante: DateTime<FixedOffset>,
        motivo: &str,
    ) -> EventoRevisao {
        EventoRevisao {
            alvo,
            operacao,
            autor: autor.to_string(),
            instante,
            motivo: motivo.to_string(),
            nota: None,
            sequencia: None,
            vista: Dado::ausente(
                "revisao:vista",
                StatusDado::NaoDisponivel,
                "apresentacao examinada nao declarada neste teste",
            )
            .unwrap(),
        }
    }

    fn aprovar(doc: &ContextoDocumento) -> EventoRevisao {
        evento(
            alvo(doc, "2.01.004", 0),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "visto e aceito",
        )
    }

    // =================================================================
    // T3-01, T3-02, T3-03: approvare non riscrive niente
    // =================================================================

    #[test]
    fn t3_01_aprovacao_explicita_de_linha_ok() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[aprovar(&doc)]).unwrap();

        assert_eq!(vista.linhas.len(), 1);
        let linha = &vista.linhas[0];
        assert_eq!(linha.estado, EstadoRevisao::AprovadoPorPessoa);
        assert_eq!(linha.status_tecnico, StatusRevisao::Ok, "esito tecnico 3B");
        assert_eq!(linha.decisoes.len(), 1);
        let decisao = &linha.decisoes[0];
        assert_eq!(decisao.evento.autor, "luca");
        assert_eq!(decisao.evento.motivo, "visto e aceito");
        assert_eq!(decisao.evento.instante, instante(12, 0, -3));
        assert_eq!(
            decisao.evento.alvo.impressao_conteudo.valor().unwrap(),
            &impressao::impressao_conteudo(&doc),
            "la decisione cita l'impronta esaminata"
        );
        assert_eq!(vista.trilha.len(), 1);
        assert_eq!(vista.extrato_revisado().len(), 1);
        assert_eq!(vista.pre_livro.linhas[0].particao, Particao::Aprovado);
        assert!(vista.eventos_nao_aplicados.is_empty());
    }

    #[test]
    fn t3_01_sem_eventos_nenhuma_linha_e_aprovada() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let vista = aplicar_revisao(&[doc], &plano(), &[]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::TecnicoOk);
        assert!(vista.extrato_revisado().is_empty());
        assert!(vista.trilha.is_empty());
    }

    #[test]
    fn t3_02_aprovar_nao_apaga_avisos_nem_esitos() {
        let doc = um_documento(
            StatusRevisao::Revisar,
            "CLASSIFICACAO_AMBIGUA; DOCUMENTO_SEM_NUMERO",
        );
        let antes = doc.clone();
        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[aprovar(&doc)]).unwrap();

        let linha = &vista.pre_livro.linhas[0];
        assert_eq!(
            linha.lancamento.alertas,
            "CLASSIFICACAO_AMBIGUA; DOCUMENTO_SEM_NUMERO"
        );
        assert_eq!(
            linha.motivos,
            vec![
                "CLASSIFICACAO_AMBIGUA".to_string(),
                "DOCUMENTO_SEM_NUMERO".to_string()
            ]
        );
        assert_eq!(
            vista.pre_livro.documentos[0].reconciliacao, antes.alocacao.reconciliacao,
            "la riconciliazione resta integra"
        );
        assert_eq!(vista.linhas[0].status_tecnico, StatusRevisao::Revisar);
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
    }

    #[test]
    fn t3_03_projecao_muda_so_o_status_revisao() {
        let doc = um_documento(StatusRevisao::Revisar, "CLASSIFICACAO_AMBIGUA");
        let original = doc.alocacao.lancamentos[0].clone();
        let entrada = vec![doc.clone()];
        let vista = aplicar_revisao(&entrada, &plano(), &[aprovar(&doc)]).unwrap();

        let projetado = &vista.pre_livro.linhas[0].lancamento;
        assert_eq!(projetado.status_revisao, StatusRevisao::Aprovado);
        assert_ne!(original.status_revisao, projetado.status_revisao);

        // Gli altri 25 campi coincidono: l'uguaglianza con il solo
        // `status_revisao` allineato lo dimostra campo per campo.
        let mut esperado = original.clone();
        esperado.status_revisao = projetado.status_revisao;
        assert_eq!(&esperado, projetado);
        assert_eq!(projetado.observacoes, "", "le note vivono negli eventi");

        // Gli ingressi sono immutabili.
        assert_eq!(entrada[0].alocacao.lancamentos[0], original);
    }

    // =================================================================
    // T3-04: revoca
    // =================================================================

    #[test]
    fn t3_04_revogacao_restaura_o_estado_tecnico_e_conserva_a_trilha() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let revogacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Revogar,
            "luca",
            instante(15, 30, -3),
            "documento ainda em discussao",
        );
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovar(&doc), revogacao],
        )
        .unwrap();

        let linha = &vista.linhas[0];
        assert_eq!(linha.estado, EstadoRevisao::AprovacaoRevogada);
        assert_eq!(
            linha.decisoes.len(),
            2,
            "l'approvazione precedente resta in traccia"
        );
        assert_eq!(
            linha.decisoes[0].evento.operacao,
            OperacaoRevisao::Aprovar,
            "ordine canonico: prima l'istante piu' antico"
        );
        assert_eq!(linha.decisoes[1].evento.operacao, OperacaoRevisao::Revogar);
        // D9.3: torna allo stato tecnico, non a REVISAR d'ufficio.
        assert_eq!(
            vista.pre_livro.linhas[0].lancamento.status_revisao,
            StatusRevisao::Ok
        );
        assert_eq!(vista.pre_livro.linhas[0].particao, Particao::Ok);
        assert!(vista.extrato_revisado().is_empty());
        assert!(
            vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APROVACAO_REVOGADA),
            "{:?}",
            vista.avisos
        );
    }

    #[test]
    fn t3_04_revogacao_de_linha_revisar_volta_a_revisar() {
        let doc = um_documento(StatusRevisao::Revisar, "CLASSIFICACAO_AMBIGUA");
        let revogacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Revogar,
            "luca",
            instante(15, 30, -3),
            "ritorno in revisione",
        );
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovar(&doc), revogacao],
        )
        .unwrap();
        assert_eq!(
            vista.pre_livro.linhas[0].lancamento.status_revisao,
            StatusRevisao::Revisar
        );
        assert_eq!(vista.pre_livro.linhas[0].particao, Particao::Revisar);
    }

    #[test]
    fn t3_06_revogar_sem_aprovacao_nao_se_aplica() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let revogacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Revogar,
            "luca",
            instante(12, 0, -3),
            "nulla da revocare",
        );
        let vista = aplicar_revisao(&[doc], &plano(), &[revogacao]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::TecnicoOk);
        assert_eq!(vista.eventos_nao_aplicados.len(), 1);
        assert_eq!(
            vista.eventos_nao_aplicados[0].motivo,
            MotivoNaoAplicacao::NadaARevogar
        );
    }

    // =================================================================
    // T3-05, T3-18, T3-21: eventi malformati
    // =================================================================

    #[test]
    fn t3_05_eventos_malformados_sao_erros_nomeados() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let base = aprovar(&doc);

        let mut sem_autor = base.clone();
        sem_autor.autor = "   ".to_string();
        let mut sem_motivo = base.clone();
        sem_motivo.motivo = String::new();
        let mut anotar_sem_nota = base.clone();
        anotar_sem_nota.operacao = OperacaoRevisao::Anotar;
        let mut sem_chave = base.clone();
        sem_chave.alvo.chave_origem = String::new();
        let mut sem_conta = base.clone();
        sem_conta.alvo.conta_id = "  ".to_string();

        let casos: Vec<(&str, EventoRevisao)> = vec![
            ("autor", sem_autor),
            ("motivo", sem_motivo),
            ("nota", anotar_sem_nota),
            ("chave", sem_chave),
            ("conta", sem_conta),
        ];
        for (nome, evento) in casos {
            let erro =
                aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[evento]).unwrap_err();
            match (nome, &erro) {
                ("autor", ErroRevisao::AutorNaoDeclarado { evento }) => {
                    assert_eq!(evento.chave_origem, CHAVE);
                }
                ("motivo", ErroRevisao::MotivoNaoDeclarado { .. }) => {}
                ("nota", ErroRevisao::NotaNaoDeclarada { .. }) => {}
                ("chave", ErroRevisao::AlvoIncompleto { campo, .. }) => {
                    assert_eq!(*campo, "chave_origem")
                }
                ("conta", ErroRevisao::AlvoIncompleto { campo, .. }) => {
                    assert_eq!(*campo, "conta_id")
                }
                (nome, outro) => panic!("caso {nome}: erro inesperado {outro}"),
            }
            // Il messaggio nomina l'evento, non la sua posizione nell'array.
            let texto = erro.to_string();
            assert!(
                texto.contains("APROVAR") || texto.contains("ANOTAR"),
                "{texto}"
            );
        }
    }

    #[test]
    fn t3_18_um_evento_malformado_impede_toda_a_vista() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let valido = aprovar(&doc);
        let mut malformado = aprovar(&doc);
        malformado.autor = String::new();
        malformado.instante = instante(18, 0, -3);
        let erro = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[valido.clone(), malformado, valido],
        )
        .unwrap_err();
        assert!(matches!(erro, ErroRevisao::AutorNaoDeclarado { .. }));
    }

    #[test]
    fn t3_21_erro_escolhido_em_ordem_canonica_nao_por_posicao() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut sem_autor = aprovar(&doc);
        sem_autor.autor = String::new();
        sem_autor.motivo = "zzz ultimo in ordine semantico".to_string();
        let mut sem_motivo = aprovar(&doc);
        sem_motivo.motivo = String::new();
        sem_motivo.autor = "aaa primo in ordine semantico".to_string();

        let direto = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[sem_autor.clone(), sem_motivo.clone()],
        )
        .unwrap_err();
        let invertido = aplicar_revisao(&[doc], &plano(), &[sem_motivo, sem_autor]).unwrap_err();
        assert_eq!(
            direto.to_string(),
            invertido.to_string(),
            "lo stesso errore nelle due permutazioni"
        );
    }

    // =================================================================
    // T3-06, T3-08: bersagli assenti e versioni superate
    // =================================================================

    #[test]
    fn t3_06_alvo_ausente_nao_toca_nenhuma_linha() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut ordinal_inexistente = aprovar(&doc);
        ordinal_inexistente.alvo.ordinal = 7;
        let mut documento_inexistente = aprovar(&doc);
        documento_inexistente.alvo.chave_origem = "CHAVE-QUE-NAO-EXISTE".to_string();
        documento_inexistente.instante = instante(13, 0, -3);

        let vista = aplicar_revisao(
            &[doc],
            &plano(),
            &[ordinal_inexistente, documento_inexistente],
        )
        .unwrap();
        assert_eq!(vista.eventos_nao_aplicados.len(), 2);
        for nao_aplicado in &vista.eventos_nao_aplicados {
            assert_eq!(nao_aplicado.motivo, MotivoNaoAplicacao::AlvoAusente);
        }
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::TecnicoOk);
        assert!(vista.linhas[0].decisoes.is_empty());
    }

    #[test]
    fn t3_08_versao_superada_nao_herda_a_decisao() {
        let antigo = um_documento(StatusRevisao::Ok, "");
        let aprovacao = aprovar(&antigo);

        // Correzione a parita' di chiave, conto e totale: cambia la
        // descrizione, quindi cambia l'impronta.
        let mut corrigido = antigo.clone();
        corrigido.alocacao.lancamentos[0].descricao_itens =
            "ITEM DE 2.01.004 2UN (10.00)".to_string();
        assert_ne!(
            impressao::impressao_conteudo(&antigo),
            impressao::impressao_conteudo(&corrigido)
        );

        let vista = aplicar_revisao(&[corrigido], &plano(), &[aprovacao]).unwrap();
        assert_eq!(vista.eventos_nao_aplicados.len(), 1);
        assert_eq!(
            vista.eventos_nao_aplicados[0].motivo,
            MotivoNaoAplicacao::VersaoSuperada
        );
        // L'evento resta consultabile con l'impronta che cita.
        assert!(
            vista.eventos_nao_aplicados[0]
                .evento
                .alvo
                .impressao_conteudo
                .valor()
                .is_some()
        );
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::TecnicoOk);
    }

    #[test]
    fn t3_06_proveniencia_ausente_nao_e_revisao_verificada() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut sem_impressao = aprovar(&doc);
        sem_impressao.alvo.impressao_conteudo = Dado::ausente(
            "revisao:conteudo",
            StatusDado::NaoDisponivel,
            "evento importado sem impressao",
        )
        .unwrap();
        let vista = aplicar_revisao(&[doc], &plano(), &[sem_impressao]).unwrap();
        assert_eq!(
            vista.eventos_nao_aplicados[0].motivo,
            MotivoNaoAplicacao::ProvenienciaAusente
        );
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::TecnicoOk);
    }

    // =================================================================
    // T3-07: due righe con lo stesso hash C23
    // =================================================================

    #[test]
    fn t3_07_duas_linhas_com_o_mesmo_hash_sao_alvos_distintos() {
        let mut primeira = lancamento(
            CHAVE,
            "2.05.001",
            "Fretes, Seguros e Outras Despesas sobre Compras",
            "10.00",
            StatusRevisao::Ok,
            "",
        );
        let mut segunda = primeira.clone();
        // C23 dipende da (origem, chave, conta_id, valor): identico.
        primeira.hash_registro = "mesmo-payload".to_string();
        segunda.hash_registro = "mesmo-payload".to_string();
        assert_eq!(primeira.hash_registro, segunda.hash_registro);

        let doc = contexto(CHAVE, StatoDocumento::Ok, "20.00", vec![primeira, segunda]);
        // Approva **solo** la seconda, con ordinale 1.
        let aprovacao = evento(
            alvo(&doc, "2.05.001", 1),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "apenas a segunda linha",
        );
        let vista = aplicar_revisao(&[doc], &plano(), &[aprovacao]).unwrap();

        assert_eq!(vista.linhas.len(), 2, "nessuno scarto per hash uguale");
        let por_ordinal = |o: u32| {
            vista
                .linhas
                .iter()
                .find(|l| l.identidade.ordinal == o)
                .unwrap()
        };
        assert_eq!(por_ordinal(0).estado, EstadoRevisao::TecnicoOk);
        assert_eq!(por_ordinal(1).estado, EstadoRevisao::AprovadoPorPessoa);
        assert!(vista.eventos_nao_aplicados.is_empty());
    }

    // =================================================================
    // T3-10, T3-11, T3-11b: rigenerazione e permutazioni
    // =================================================================

    #[test]
    fn t3_10_mesmo_conteudo_em_outro_lote_reencontra_o_alvo() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let aprovacao = aprovar(&doc);

        let mut outro_lote = doc.clone();
        outro_lote.lote = IdentidadeLote {
            id: "lote-2026-06-B".to_string(),
            gerado_em: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            descricao: "rigenerazione".to_string(),
        };
        assert_eq!(
            impressao::impressao_conteudo(&doc),
            impressao::impressao_conteudo(&outro_lote),
            "il lotto non entra nell'impronta"
        );

        let vista = aplicar_revisao(&[outro_lote], &plano(), &[aprovacao]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
        assert_eq!(
            vista.linhas[0].identidade.lote, "lote-2026-06-B",
            "l'identita' di riga porta il lotto nuovo"
        );
        assert!(vista.eventos_nao_aplicados.is_empty());
    }

    #[test]
    fn t3_11_permutacao_dos_documentos_e_indiferente() {
        let primeiro = um_documento(StatusRevisao::Ok, "");
        let mut segundo = primeiro.clone();
        segundo.chave_origem = CHAVE2.to_string();
        segundo.alocacao.lancamentos[0].chave_origem = CHAVE2.to_string();

        let eventos = vec![aprovar(&primeiro), aprovar(&segundo)];
        let direto =
            aplicar_revisao(&[primeiro.clone(), segundo.clone()], &plano(), &eventos).unwrap();
        let invertido = aplicar_revisao(&[segundo, primeiro], &plano(), &eventos).unwrap();
        assert_eq!(direto, invertido, "l'intera vista e' identica");
    }

    #[test]
    fn t3_11b_permutacao_dos_artigos_e_um_caso_distinto() {
        // Permutare gli articoli dell'XML cambia gli `ordinal`, quindi cambia
        // l'impronta: e' una versione diversa, non "gli stessi bersagli".
        let a = lancamento(
            CHAVE,
            "2.01.004",
            "Refrigerantes",
            "10.00",
            StatusRevisao::Ok,
            "",
        );
        let b = lancamento(
            CHAVE,
            "2.01.002",
            "Vinhos e Espumantes",
            "20.00",
            StatusRevisao::Ok,
            "",
        );
        let direto = contexto(
            CHAVE,
            StatoDocumento::Ok,
            "30.00",
            vec![a.clone(), b.clone()],
        );
        let invertido = contexto(CHAVE, StatoDocumento::Ok, "30.00", vec![b, a]);
        assert_ne!(
            impressao::impressao_conteudo(&direto),
            impressao::impressao_conteudo(&invertido),
            "l'ordine degli articoli e' contenuto, non presentazione"
        );

        let aprovacao = evento(
            alvo(&direto, "2.01.004", 0),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "aprovacao da primeira versao",
        );
        let vista = aplicar_revisao(&[invertido], &plano(), &[aprovacao]).unwrap();
        assert_eq!(
            vista.eventos_nao_aplicados[0].motivo,
            MotivoNaoAplicacao::VersaoSuperada
        );
    }

    // =================================================================
    // T3-14, T3-17, T3-22: identita' esatta, ordine e conflitti
    // =================================================================

    #[test]
    fn t3_14_repeticao_exata_nao_e_aplicada_duas_vezes() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let aprovacao = aprovar(&doc);
        let vista = aplicar_revisao(
            &[doc],
            &plano(),
            &[aprovacao.clone(), aprovacao.clone(), aprovacao],
        )
        .unwrap();

        assert_eq!(vista.linhas[0].decisoes.len(), 1, "una sola applicazione");
        assert_eq!(vista.linhas[0].decisoes[0].repeticoes, 2);
        assert_eq!(
            vista.eventos_nao_aplicados.len(),
            2,
            "le ripetizioni restano consultabili"
        );
        for nao_aplicado in &vista.eventos_nao_aplicados {
            assert_eq!(nao_aplicado.motivo, MotivoNaoAplicacao::Repeticao);
        }
        assert!(
            !vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APROVACAO_REPETIDA),
            "una ripetizione esatta non e' una seconda approvazione"
        );
    }

    #[test]
    fn t3_14_decisoes_incompativeis_no_mesmo_instante_sao_conflito() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let aprovacao = aprovar(&doc);
        let mut revogacao = aprovacao.clone();
        revogacao.operacao = OperacaoRevisao::Revogar;
        revogacao.motivo = "revogo no mesmo instante".to_string();

        // Nessuna sequenza: non ordinabili.
        let erro = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovacao.clone(), revogacao.clone()],
        )
        .unwrap_err();
        assert!(matches!(erro, ErroRevisao::DecisoesIncompativeis { .. }));

        // Stessa sequenza dichiarata: ancora non ordinabili.
        let mut com_mesma = aprovacao.clone();
        com_mesma.sequencia = Some(1);
        let mut revogacao_mesma = revogacao.clone();
        revogacao_mesma.sequencia = Some(1);
        assert!(matches!(
            aplicar_revisao(
                std::slice::from_ref(&doc),
                &plano(),
                &[com_mesma, revogacao_mesma]
            )
            .unwrap_err(),
            ErroRevisao::DecisoesIncompativeis { .. }
        ));

        // Una sola sequenza dichiarata: conflitto, non deduzione.
        let mut com_uma = aprovacao.clone();
        com_uma.sequencia = Some(1);
        assert!(matches!(
            aplicar_revisao(
                std::slice::from_ref(&doc),
                &plano(),
                &[com_uma, revogacao.clone()]
            )
            .unwrap_err(),
            ErroRevisao::DecisoesIncompativeis { .. }
        ));

        // Sequenze dichiarate e diverse: la sequenza decide, nessun errore.
        let mut aprovacao_1 = aprovacao;
        aprovacao_1.sequencia = Some(1);
        let mut revogacao_2 = revogacao;
        revogacao_2.sequencia = Some(2);
        let vista = aplicar_revisao(&[doc], &plano(), &[revogacao_2, aprovacao_1]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovacaoRevogada);
    }

    #[test]
    fn t3_14_eventos_contemporaneos_em_linhas_diferentes_nao_sao_conflito() {
        let doc = duas_linhas(CHAVE);
        let aprovar_primeira = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "primeira",
        );
        let revogar_segunda = evento(
            alvo(&doc, "2.01.002", 1),
            OperacaoRevisao::Revogar,
            "luca",
            instante(12, 0, -3),
            "segunda, nada a revogar",
        );
        let vista =
            aplicar_revisao(&[doc], &plano(), &[aprovar_primeira, revogar_segunda]).unwrap();
        assert_eq!(vista.linhas.len(), 2);
        assert_eq!(
            vista.eventos_nao_aplicados[0].motivo,
            MotivoNaoAplicacao::NadaARevogar
        );
    }

    #[test]
    fn t3_17_duas_anotacoes_no_mesmo_instante_sem_sequencia() {
        // Controesempio E1: la vista, traccia compresa, non deve dipendere
        // dall'ordine dell'array.
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut nota_a = aprovar(&doc);
        nota_a.operacao = OperacaoRevisao::Anotar;
        nota_a.nota = Some("nota A".to_string());
        let mut nota_b = nota_a.clone();
        nota_b.nota = Some("nota B".to_string());

        let direto = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[nota_a.clone(), nota_b.clone()],
        )
        .unwrap();
        let invertido = aplicar_revisao(&[doc], &plano(), &[nota_b, nota_a]).unwrap();
        assert_eq!(direto, invertido, "l'intera vista, traccia compresa");
        assert_eq!(direto.linhas[0].anotacoes.len(), 2);
        assert_eq!(
            direto.linhas[0].anotacoes[0].evento.nota.as_deref(),
            Some("nota A")
        );
        assert_eq!(direto.trilha.len(), 2);
    }

    #[test]
    fn t3_22_c1_alvos_distintos_nao_sao_repeticao() {
        let doc = duas_linhas(CHAVE);
        // Tutto uguale tranne il bersaglio: autore, istante e motivo identici.
        let primeira = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "visto e aceito",
        );
        let segunda = evento(
            alvo(&doc, "2.01.002", 1),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(12, 0, -3),
            "visto e aceito",
        );
        assert_ne!(
            texto_exato_evento(&primeira),
            texto_exato_evento(&segunda),
            "C1: i dati dichiarati sono diversi"
        );

        let vista = aplicar_revisao(&[doc], &plano(), &[primeira, segunda]).unwrap();
        assert_eq!(
            vista.extrato_revisado().len(),
            2,
            "C1: entrambe le righe approvate, nessuna falsa Repeticao"
        );
        assert!(vista.eventos_nao_aplicados.is_empty());
    }

    #[test]
    fn t3_22_c2_c3_c4_c5_sao_declaracoes_distintas() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let base = aprovar(&doc);

        // C2: autore in NFC e in NFD.
        let mut c2_nfc = base.clone();
        c2_nfc.autor = "Jos\u{00E9}".to_string();
        let mut c2_nfd = base.clone();
        c2_nfd.autor = "Jose\u{0301}".to_string();
        // C3: stesso istante assoluto, offset dichiarati diversi.
        let mut c3_a = base.clone();
        c3_a.instante = instante(12, 0, -3);
        let mut c3_b = base.clone();
        c3_b.instante = instante(15, 0, 0);
        assert_eq!(
            c3_a.instante.with_timezone(&Utc),
            c3_b.instante.with_timezone(&Utc),
            "C3: lo stesso momento, dichiarato in due fusi"
        );
        // C4: il Dado della vista differisce **solo** nel motivo.
        let mut c4_a = base.clone();
        c4_a.vista =
            Dado::novo(Some("v".repeat(64)), "revisao:vista", StatusDado::Ok, None).unwrap();
        let mut c4_b = base.clone();
        c4_b.vista = Dado::novo(
            Some("v".repeat(64)),
            "revisao:vista",
            StatusDado::Ok,
            Some("examinado junto ao resumo".to_string()),
        )
        .unwrap();
        // C5: nota in NFC e in NFD.
        let mut c5_nfc = base.clone();
        c5_nfc.operacao = OperacaoRevisao::Anotar;
        c5_nfc.nota = Some("a\u{00E7}ucar".to_string());
        let mut c5_nfd = c5_nfc.clone();
        c5_nfd.nota = Some("ac\u{0327}ucar".to_string());

        for (caso, a, b) in [
            ("C2", c2_nfc, c2_nfd),
            ("C3", c3_a, c3_b),
            ("C4", c4_a, c4_b),
            ("C5", c5_nfc, c5_nfd),
        ] {
            assert_ne!(
                texto_exato_evento(&a),
                texto_exato_evento(&b),
                "{caso}: i dati dichiarati sono diversi"
            );
            let direto = aplicar_revisao(
                std::slice::from_ref(&doc),
                &plano(),
                &[a.clone(), b.clone()],
            )
            .unwrap();
            let invertido = aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[b, a]).unwrap();
            assert_eq!(direto, invertido, "{caso}: ordine canonico");
            assert!(
                direto
                    .eventos_nao_aplicados
                    .iter()
                    .all(|e| e.motivo != MotivoNaoAplicacao::Repeticao),
                "{caso}: nessuna falsa Repeticao"
            );
            let aplicados = direto.linhas[0].decisoes.len() + direto.linhas[0].anotacoes.len();
            assert_eq!(aplicados, 2, "{caso}: entrambe conservate");
        }
    }

    #[test]
    fn t3_22_codificacao_exata_e_injetiva_nos_campos_declarados() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let base = aprovar(&doc);
        // Copia con dati identici: stesso testo.
        assert_eq!(texto_exato_evento(&base), texto_exato_evento(&base.clone()));

        let mut variacoes: Vec<(&str, EventoRevisao)> = Vec::new();
        let mut v = base.clone();
        v.alvo.chave_origem = CHAVE2.to_string();
        variacoes.push(("chave_origem", v));
        let mut v = base.clone();
        v.alvo.conta_id = "2.01.002".to_string();
        variacoes.push(("conta_id", v));
        let mut v = base.clone();
        v.alvo.ordinal = 1;
        variacoes.push(("ordinal", v));
        let mut v = base.clone();
        v.alvo.origem = OrigemLancamento::Manual;
        variacoes.push(("origem", v));
        let mut v = base.clone();
        v.operacao = OperacaoRevisao::Revogar;
        variacoes.push(("operacao", v));
        let mut v = base.clone();
        v.autor = "outra pessoa".to_string();
        variacoes.push(("autor", v));
        let mut v = base.clone();
        v.instante = instante(13, 0, -3);
        variacoes.push(("instante", v));
        let mut v = base.clone();
        v.motivo = "outro motivo".to_string();
        variacoes.push(("motivo", v));
        let mut v = base.clone();
        v.nota = Some("uma nota".to_string());
        variacoes.push(("nota presente contra ausente", v));
        let mut v = base.clone();
        v.sequencia = Some(0);
        variacoes.push(("sequencia 0 contra ausente", v));
        let mut v = base.clone();
        v.vista = Dado::presente("v".repeat(64), "revisao:vista").unwrap();
        variacoes.push(("vista presente contra ausente", v));
        let mut v = base.clone();
        v.alvo.impressao_conteudo =
            Dado::presente("outra impressao".to_string(), "revisao:conteudo").unwrap();
        variacoes.push(("impressao do alvo", v));

        let texto_base = texto_exato_evento(&base);
        for (campo, variacao) in variacoes {
            assert_ne!(
                texto_exato_evento(&variacao),
                texto_base,
                "il campo {campo} deve entrare nella codifica esatta"
            );
        }
    }

    #[test]
    fn texto_exato_nao_normaliza_e_conserva_o_offset() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut com_acento = aprovar(&doc);
        com_acento.autor = "Jose\u{0301}".to_string();
        let texto = texto_exato_evento(&com_acento);
        assert!(
            texto.contains("EAU=6:Jose\u{0301}"),
            "byte originali, lunghezza 6: {texto}"
        );
        // L'offset dichiarato e' conservato **in secondi** (R3C3A-02):
        // -03:00 sono -10800 secondi, e la forma non arrotonda ai minuti.
        assert!(
            texto.contains("EIN=38:2026-10-21T12:00:00.000000000OFF-10800"),
            "orario locale, nanosecondi e offset in secondi: {texto}"
        );
        assert!(texto.starts_with("EVENTO-EXATO/v1\n"));
        assert!(texto.contains("ENO=AUSENTE"));
        assert!(texto.contains("ESQ=SEM_SEQUENCIA"));
        assert!(texto.contains("DADOX:"));
    }

    // =================================================================
    // T3-15: annotazioni
    // =================================================================

    #[test]
    fn t3_15_anotacao_conservada_e_aprovacao_nao_muda_a_impressao() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let impressao_antes = impressao::impressao_conteudo(&doc);

        let mut anotacao = aprovar(&doc);
        anotacao.operacao = OperacaoRevisao::Anotar;
        anotacao.nota = Some("conferido com o fornecedor".to_string());
        anotacao.instante = instante(10, 0, -3);
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[anotacao, aprovar(&doc)],
        )
        .unwrap();

        assert_eq!(vista.linhas[0].anotacoes.len(), 1);
        assert_eq!(
            vista.linhas[0].anotacoes[0].evento.nota.as_deref(),
            Some("conferido com o fornecedor")
        );
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
        // `observacoes` non viene sovrascritto: le note vivono negli eventi.
        assert_eq!(vista.pre_livro.linhas[0].lancamento.observacoes, "");
        // Approvare non cambia l'impronta dell'oggetto approvato.
        assert_eq!(
            vista.versoes[0].impressao_conteudo, impressao_antes,
            "l'impronta e' calcolata sui documenti originali"
        );
    }

    // =================================================================
    // T3-16, T3-19: senza righe, ERRO, APROVADO senza provenienza
    // =================================================================

    #[test]
    fn t3_16_documento_sem_linhas_e_aprovado_sem_proveniencia() {
        let sem_linhas = contexto(CHAVE, StatoDocumento::Parcial, "0.00", Vec::new());
        let importado = contexto(
            CHAVE2,
            StatoDocumento::Ok,
            "10.00",
            vec![lancamento(
                CHAVE2,
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Aprovado,
                "",
            )],
        );

        let vista = aplicar_revisao(&[sem_linhas, importado], &plano(), &[]).unwrap();

        // I8: il documento senza righe compare con la sua identita'.
        assert_eq!(vista.pre_livro.documentos.len(), 2);
        assert!(
            vista
                .pre_livro
                .documentos
                .iter()
                .any(|d| d.chave_origem == CHAVE && d.linhas == 0)
        );
        // D9.4: APROVADO senza evento non e' una revisione verificata.
        let linha = &vista.linhas[0];
        assert_eq!(linha.estado, EstadoRevisao::AprovadoSemProveniencia);
        assert_eq!(
            linha.status_tecnico,
            StatusRevisao::Aprovado,
            "il dato originale resta leggibile"
        );
        assert!(!linha.no_extrato_revisado);
        assert!(
            vista.extrato_revisado().is_empty(),
            "fuori dall'estratto revisionato"
        );
        // La 3C.2 invece la includerebbe: la differenza e' dichiarata.
        assert_eq!(vista.pre_livro.extrato_aprovado().len(), 1);
        assert_eq!(vista.exclusoes_extrato_revisado.len(), 1);
        assert!(
            vista.exclusoes_extrato_revisado[0]
                .motivo
                .contains("APROVADO_SEM_PROVENIENCIA")
        );
        assert!(
            vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APROVACAO_SEM_PROVENIENCIA)
        );
        // D8.5: la sequenza storica non e' calcolabile.
        for versao in &vista.versoes {
            assert_eq!(versao.sequencia.status(), StatusDado::NaoDisponivel);
            assert!(versao.sequencia.valor().is_none());
            assert!(versao.motivo.valor().is_none());
            assert_eq!(versao.impressao_conteudo.len(), 64);
            assert_eq!(versao.impressao_apresentacao.len(), 64);
        }
    }

    #[test]
    fn t3_16_sanatoria_explicita_de_aprovado_sem_proveniencia() {
        let doc = um_documento(StatusRevisao::Aprovado, "");
        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[aprovar(&doc)]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
        assert!(vista.linhas[0].no_extrato_revisado);
        assert!(vista.exclusoes_extrato_revisado.is_empty());
    }

    #[test]
    fn t3_19_aprovacao_em_documento_erro_e_registrada_mas_nao_entra() {
        let doc = contexto(
            CHAVE,
            StatoDocumento::Erro,
            "10.00",
            vec![lancamento(
                CHAVE,
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Ok,
                "",
            )],
        );
        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[aprovar(&doc)]).unwrap();

        let linha = &vista.linhas[0];
        assert_eq!(
            linha.estado,
            EstadoRevisao::AprovadoPorPessoa,
            "D9.2: la decisione viene registrata"
        );
        assert_eq!(linha.decisoes.len(), 1);
        assert!(!linha.no_extrato_revisado, "D10a: resta fuori");
        assert!(vista.extrato_revisado().is_empty());
        // L'esclusione e la sua causa restano interrogabili nel pre-libro.
        assert_eq!(vista.pre_livro.exclusoes_extrato.len(), 1);
        assert!(vista.pre_livro.exclusoes_extrato[0].motivo.contains("ERRO"));
        assert!(
            vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APROVACAO_EM_DOCUMENTO_ERRO)
        );
        // Nessun avviso tecnico cancellato per approvare.
        assert_eq!(
            vista.pre_livro.documentos[0].reconciliacao.stato_documento,
            StatoDocumento::Erro
        );
    }

    // =================================================================
    // D8.3 e avvisi
    // =================================================================

    #[test]
    fn apresentacao_mudada_e_sinalizada_e_as_decisoes_permanecem() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let mut com_vista_antiga = aprovar(&doc);
        com_vista_antiga.vista = Dado::presente(
            "impressao de apresentacao antiga".to_string(),
            "revisao:vista",
        )
        .unwrap();

        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[com_vista_antiga]).unwrap();
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
        assert!(
            vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APRESENTACAO_MUDADA),
            "{:?}",
            vista.avisos
        );

        // Con la presentazione corrente nessun avviso.
        let conteudo = impressao::impressao_conteudo(&doc);
        let contas = impressao::contas_mostradas(&doc, &plano()).unwrap();
        let mut atual = aprovar(&doc);
        atual.vista = Dado::presente(
            impressao::impressao_apresentacao(&conteudo, &contas),
            "revisao:vista",
        )
        .unwrap();
        let sem_aviso = aplicar_revisao(&[doc], &plano(), &[atual]).unwrap();
        assert!(
            !sem_aviso
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APRESENTACAO_MUDADA)
        );
    }

    #[test]
    fn aprovacao_repetida_com_declaracoes_diferentes_avisa_e_conserva() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let primeira = aprovar(&doc);
        let mut segunda = primeira.clone();
        segunda.autor = "outra pessoa".to_string();
        segunda.instante = instante(16, 0, -3);

        let vista = aplicar_revisao(&[doc], &plano(), &[primeira, segunda]).unwrap();
        assert_eq!(vista.linhas[0].decisoes.len(), 2);
        assert_eq!(vista.linhas[0].estado, EstadoRevisao::AprovadoPorPessoa);
        assert!(
            vista
                .avisos
                .iter()
                .any(|a| a.codigo == avisos::APROVACAO_REPETIDA)
        );
    }

    #[test]
    fn avisos_da_vista_ficam_em_ordem_canonica() {
        let doc = contexto(
            CHAVE,
            StatoDocumento::Erro,
            "10.00",
            vec![lancamento(
                CHAVE,
                "2.01.004",
                "Refrigerantes",
                "10.00",
                StatusRevisao::Aprovado,
                "",
            )],
        );
        let vista =
            aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[aprovar(&doc)]).unwrap();
        let mut ordenados = vista.avisos.clone();
        ordenados.sort_by(|a, b| (&a.codigo, &a.detalhe).cmp(&(&b.codigo, &b.detalhe)));
        assert_eq!(vista.avisos, ordenados);
    }

    // =================================================================
    // Limiti dichiarati e propagazione degli errori della 3C.2
    // =================================================================

    #[test]
    fn alvo_ambiguo_e_interceptado_antes_pela_validacao_da_3c2() {
        // `AlvoAmbiguo` richiederebbe due righe con la stessa
        // (chave_origem, impressao, origem, conta_id, ordinal). Dentro un
        // documento l'ordinale e' l'indice del vettore, quindi e' unico; due
        // documenti con la stessa chiave vengono rifiutati **prima**, dalla
        // validazione della 3C.2. Questo test registra quale controllo
        // intercetta il caso, invece di forzare un tipo incoerente.
        let doc = um_documento(StatusRevisao::Ok, "");
        let erro = aplicar_revisao(&[doc.clone(), doc], &plano(), &[]).unwrap_err();
        match erro {
            ErroRevisao::PreLivro(ErroPreLivro::DocumentoDuplicadoNoLote { chave_origem }) => {
                assert_eq!(chave_origem, CHAVE);
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn erro_do_pre_livro_e_propagado_sem_reescrita() {
        let mut doc = um_documento(StatusRevisao::Ok, "");
        // Incoerenza fra contesto e registrazione: la 3C.2 la rifiuta.
        doc.alocacao.lancamentos[0].chave_origem = "OUTRA-CHAVE".to_string();
        let erro = aplicar_revisao(&[doc], &plano(), &[]).unwrap_err();
        let texto = erro.to_string();
        assert!(texto.starts_with("pre-livro invalido:"), "{texto}");
        assert!(texto.contains("chave_origem"), "{texto}");
    }

    #[test]
    fn conta_fora_do_plano_e_rejeitada() {
        let mut doc = um_documento(StatusRevisao::Ok, "");
        doc.alocacao.lancamentos[0].conta_id = "9.99.999".to_string();
        let erro = aplicar_revisao(&[doc], &plano(), &[]).unwrap_err();
        assert!(matches!(
            erro,
            ErroRevisao::PreLivro(ErroPreLivro::ContaNaoEncontradaNoPlano { .. })
        ));
    }

    #[test]
    fn lote_vazio_produz_vista_vazia_sem_inventar_identidade() {
        let vista = aplicar_revisao(&[], &plano(), &[]).unwrap();
        assert!(vista.linhas.is_empty());
        assert!(vista.versoes.is_empty());
        assert!(vista.pre_livro.lote.is_none());
    }

    // =================================================================
    // Riscontri dell'audit del 2026-10-21 (R3C3A-01/02/03)
    // =================================================================

    /// R3C3A-01: entrare nell'estratto revisionato richiede un'approvazione
    /// umana **attualmente attiva**, non soltanto l'assenza dello stato
    /// `APROVADO_SEM_PROVENIENCIA`.
    ///
    /// Il caso nasce da un ingresso ammesso da D9.4: una riga che arriva
    /// `APROVADO` senza evento. Dopo la sanatoria e la sua revoca, lo stato
    /// torna a quello tecnico — che qui e' `APROVADO` — e la riga rientrava
    /// nell'estratto pur senza nessuna approvazione attiva.
    #[test]
    fn r1_revogacao_da_sanatoria_tira_a_linha_do_extrato_revisado() {
        let doc = um_documento(StatusRevisao::Aprovado, "");
        let aprovacao = aprovar(&doc);
        let revogacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Revogar,
            "luca",
            instante(15, 30, -3),
            "revogo a sanatoria",
        );

        // 1. Senza eventi la riga e' fuori: nessuna provenienza.
        let sem_eventos = aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[]).unwrap();
        assert!(sem_eventos.extrato_revisado().is_empty());

        // 2. Sanatoria esplicita, poi revoca.
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovacao.clone(), revogacao.clone()],
        )
        .unwrap();
        let linha = &vista.linhas[0];
        assert_eq!(linha.estado, EstadoRevisao::AprovacaoRevogada);
        assert!(
            !linha.no_extrato_revisado,
            "senza approvazione attiva la riga non puo' stare nell'estratto"
        );
        assert!(vista.extrato_revisado().is_empty());
        // La differenza rispetto all'estratto della 3C.2 e' dichiarata.
        assert_eq!(vista.pre_livro.extrato_aprovado().len(), 1);
        assert_eq!(vista.exclusoes_extrato_revisado.len(), 1);
        assert!(
            vista.exclusoes_extrato_revisado[0]
                .motivo
                .contains("APROVACAO_REVOGADA"),
            "{}",
            vista.exclusoes_extrato_revisado[0].motivo
        );
        // Dato originale e decisioni conservati.
        assert_eq!(linha.status_tecnico, StatusRevisao::Aprovado);
        assert_eq!(linha.decisoes.len(), 2);
        assert_eq!(
            vista.pre_livro.linhas[0].lancamento.status_revisao,
            StatusRevisao::Aprovado,
            "la proiezione ripristina lo stato tecnico, non inventa OK o REVISAR"
        );

        // 3. Ri-approvazione: la riga rientra, perche' l'approvazione e'
        //    di nuovo attiva.
        let nova_aprovacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Aprovar,
            "luca",
            instante(18, 0, -3),
            "reaprovo depois de conferir",
        );
        let reaprovada = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovacao, revogacao, nova_aprovacao],
        )
        .unwrap();
        assert_eq!(
            reaprovada.linhas[0].estado,
            EstadoRevisao::AprovadoPorPessoa
        );
        assert_eq!(reaprovada.extrato_revisado().len(), 1);
        assert!(reaprovada.exclusoes_extrato_revisado.is_empty());
        assert_eq!(reaprovada.linhas[0].decisoes.len(), 3);
    }

    /// R3C3A-01, controllo ordinario: su una riga tecnicamente `OK` la revoca
    /// lascia l'estratto vuoto, e questo funzionava gia'.
    #[test]
    fn r1_revogacao_de_linha_ok_continua_fora_do_extrato() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let revogacao = evento(
            alvo(&doc, "2.01.004", 0),
            OperacaoRevisao::Revogar,
            "luca",
            instante(15, 30, -3),
            "revogo",
        );
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[aprovar(&doc), revogacao],
        )
        .unwrap();
        assert!(vista.extrato_revisado().is_empty());
        assert!(
            vista.exclusoes_extrato_revisado.is_empty(),
            "qui la 3C.2 non includerebbe la riga: nessuna differenza da dichiarare"
        );
    }

    /// R3C3A-02: `FixedOffset` ammette offset con **secondi**, e la codifica
    /// esatta deve conservarli. Due dichiarazioni con offset di 1 e 2 secondi
    /// sono due istanti assoluti diversi: fonderle in una `Repeticao`
    /// perderebbe una dichiarazione reale.
    #[test]
    fn r2_offset_declarado_com_segundos_nao_e_repeticao() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let com_offset = |segundos: i32| {
            let mut e = aprovar(&doc);
            e.instante = FixedOffset::east_opt(segundos)
                .unwrap()
                .with_ymd_and_hms(2026, 10, 21, 12, 0, 0)
                .unwrap();
            e
        };
        let a = com_offset(1);
        let b = com_offset(2);
        assert_ne!(a.instante, b.instante, "offset dichiarati diversi");
        assert_ne!(
            a.instante.with_timezone(&Utc),
            b.instante.with_timezone(&Utc),
            "anche gli istanti assoluti sono diversi"
        );
        assert_ne!(
            texto_exato_evento(&a),
            texto_exato_evento(&b),
            "la codifica esatta deve conservare i secondi dell'offset"
        );

        let direto = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[a.clone(), b.clone()],
        )
        .unwrap();
        let invertido = aplicar_revisao(std::slice::from_ref(&doc), &plano(), &[b, a]).unwrap();
        assert_eq!(direto, invertido, "ordine canonico indifferente");
        assert_eq!(
            direto.linhas[0].decisoes.len(),
            2,
            "due dichiarazioni, due decisioni"
        );
        assert!(
            direto
                .eventos_nao_aplicados
                .iter()
                .all(|e| e.motivo != MotivoNaoAplicacao::Repeticao),
            "nessuna falsa Repeticao"
        );
        assert!(
            direto
                .avisos
                .iter()
                .any(|x| x.codigo == avisos::APROVACAO_REPETIDA)
        );
    }

    /// R3C3A-02, controllo: i nanosecondi dell'orario restano distinti, e due
    /// eventi davvero identici restano una ripetizione.
    #[test]
    fn r2_nanossegundos_e_repeticao_exata_continuam_corretos() {
        let doc = um_documento(StatusRevisao::Ok, "");
        let base = aprovar(&doc);
        let mut com_nanos = base.clone();
        com_nanos.instante = FixedOffset::east_opt(-3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 21, 12, 0, 0)
            .unwrap()
            + chrono::Duration::nanoseconds(1);
        assert_ne!(texto_exato_evento(&base), texto_exato_evento(&com_nanos));

        // Due eventi con dati identici restano una ripetizione.
        let vista = aplicar_revisao(
            std::slice::from_ref(&doc),
            &plano(),
            &[base.clone(), base.clone()],
        )
        .unwrap();
        assert_eq!(vista.linhas[0].decisoes.len(), 1);
        assert_eq!(vista.linhas[0].decisoes[0].repeticoes, 1);
    }

    /// R3C3A-03: la scelta del conflitto da riportare non deve dipendere
    /// dall'ordine dei **documenti** in ingresso. L'ordine canonico degli
    /// eventi e' l'unico criterio ammesso (F3C-44).
    #[test]
    fn r3_escolha_do_conflito_nao_depende_da_ordem_dos_documentos() {
        let primeiro = um_documento(StatusRevisao::Ok, "");
        let mut segundo = primeiro.clone();
        segundo.chave_origem = CHAVE2.to_string();
        segundo.alocacao.lancamentos[0].chave_origem = CHAVE2.to_string();

        let conflito = |doc: &ContextoDocumento, hora: u32| {
            let aprovacao = evento(
                alvo(doc, "2.01.004", 0),
                OperacaoRevisao::Aprovar,
                "luca",
                instante(hora, 0, -3),
                "aprovo",
            );
            let mut revogacao = aprovacao.clone();
            revogacao.operacao = OperacaoRevisao::Revogar;
            revogacao.motivo = "revogo no mesmo instante".to_string();
            vec![aprovacao, revogacao]
        };
        let mut eventos = conflito(&primeiro, 12);
        eventos.extend(conflito(&segundo, 13));

        let direto =
            aplicar_revisao(&[primeiro.clone(), segundo.clone()], &plano(), &eventos).unwrap_err();
        let invertido = aplicar_revisao(&[segundo, primeiro], &plano(), &eventos).unwrap_err();
        assert_eq!(
            direto.to_string(),
            invertido.to_string(),
            "lo stesso conflitto nelle due permutazioni dei documenti"
        );
        // Il conflitto scelto e' il primo in ordine canonico: l'istante piu'
        // antico, cioe' quello delle 12:00 sul primo documento.
        match &direto {
            ErroRevisao::DecisoesIncompativeis {
                primeiro, segundo, ..
            } => {
                assert_eq!(primeiro.chave_origem, CHAVE);
                assert_eq!(segundo.chave_origem, CHAVE);
                assert_eq!(primeiro.instante, instante(12, 0, -3));
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    /// R3C3A-03, controllo: anche permutando **eventi e documenti** insieme
    /// l'errore resta lo stesso.
    #[test]
    fn r3_conflito_estavel_permutando_eventos_e_documentos() {
        let primeiro = um_documento(StatusRevisao::Ok, "");
        let mut segundo = primeiro.clone();
        segundo.chave_origem = CHAVE2.to_string();
        segundo.alocacao.lancamentos[0].chave_origem = CHAVE2.to_string();

        let par = |doc: &ContextoDocumento, hora: u32| {
            let aprovacao = evento(
                alvo(doc, "2.01.004", 0),
                OperacaoRevisao::Aprovar,
                "luca",
                instante(hora, 0, -3),
                "aprovo",
            );
            let mut revogacao = aprovacao.clone();
            revogacao.operacao = OperacaoRevisao::Revogar;
            revogacao.motivo = "revogo no mesmo instante".to_string();
            (aprovacao, revogacao)
        };
        let (a1, r1) = par(&primeiro, 12);
        let (a2, r2) = par(&segundo, 13);

        let direto = aplicar_revisao(
            &[primeiro.clone(), segundo.clone()],
            &plano(),
            &[a1.clone(), r1.clone(), a2.clone(), r2.clone()],
        )
        .unwrap_err();
        let invertido =
            aplicar_revisao(&[segundo, primeiro], &plano(), &[r2, a2, r1, a1]).unwrap_err();
        assert_eq!(direto.to_string(), invertido.to_string());
    }
}

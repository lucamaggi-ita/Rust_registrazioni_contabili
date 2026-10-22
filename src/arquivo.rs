//! Archivio autorevole: schema, codec esatto e validazione (fase 3C.3.b.1).
//!
//! Attua i contratti `PER-01…PER-24` della specifica di fase 3C.3.b, con le
//! decisioni D7(a), D14(a), D15(a), D16(a) e D17(a) approvate il 2026-10-22.
//!
//! # Che cosa conserva, e che cosa no
//!
//! D7(a): l'archivio e' la **fonte autorevole**. Conserva i
//! [`ContextoDocumento`] **prima** della proiezione umana, le
//! **dichiarazioni** degli eventi, i metadati delle esecuzioni e l'identita'
//! del piano che ognuna ha usato. **Non** conserva la vista revisionata, i
//! riepiloghi, le partizioni ne' l'estratto: sono derivabili, e salvarli
//! creerebbe una seconda verita'.
//!
//! # Perche' esiste un DTO
//!
//! I tipi del modello derivano `Deserialize`, ma quel percorso **non** da' le
//! garanzie che l'archivio richiede (riscontro R3C3B-01):
//!
//! - un campo sconosciuto, anche annidato, viene **accettato e scartato**;
//! - un **numero** JSON al posto di una stringa monetaria viene accettato, con
//!   la scala schiacciata;
//! - una stringa oltre la precisione rappresentabile viene **arrotondata**:
//!   `"0.00000000000000000000000000001"` diventa zero.
//!
//! Quindi la lettura passa per tre passaggi con responsabilita' separate:
//!
//! 1. **forma**: DTO annidati, tutti con `deny_unknown_fields`. Qui cadono i
//!    campi sconosciuti e le chiavi duplicate. Non si passa per un albero
//!    `serde_json::Value`, che le duplicazioni le collasserebbe prima di
//!    qualunque controllo;
//! 2. **rappresentazione**: ogni `Decimal` e' un testo convertito con
//!    [`Decimal::from_str_exact`] **e** riprodotto con `to_string()` per
//!    confrontarlo con l'originale. Serve la coppia: il primo rifiuta la
//!    perdita di cifre e `3.5e1`, il secondo rifiuta `+35.00`;
//! 3. **invarianti**: la costruzione passa per `Dado::novo`,
//!    `Competencia::try_from` e gli enum chiusi.
//!
//! # Purezza
//!
//! Questo modulo non apre file, non legge l'orologio, non carica il piano dei
//! conti e non conosce percorsi: tutto arriva come parametro. L'IO sta in
//! [`crate::arquivo_io`].

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::alocacao::{
    AlocacaoNfe, CampoRinviato, Componente, EsitoComponenti, EsitoCopertura, EsitoRate, EsitoTotal,
    Reconciliacao, RuoloComponente, StatoDocumento,
};
use crate::impressao;
use crate::modelos::{
    Competencia, Dado, Duplicata, Lancamento, OrigemLancamento, StatusDado, StatusRevisao,
    TipoConta,
};
use crate::nfe::Aviso;
use crate::prelivro::{ContextoDocumento, IdentidadeLote};
use crate::regras::PlanoContas;
use crate::revisao::{ChaveRevisao, EventoRevisao, OperacaoRevisao};

/// Identificatore del formato. Entra nei byte e viene verificato in lettura.
pub const FORMATO: &str = "FANTOZZI-ARQUIVO/v1";
/// Versione dello schema di **questo file**. Non e' la sequenza storica di un
/// documento (D17): quella resta `NAO_DISPONIVEL`.
pub const VERSAO_SCHEMA: u32 = 1;

/// Marcatore interno: un valore monetario non testuale. Serve a classificare
/// l'errore senza dipendere dalla formulazione di `serde`.
const MARCA_NAO_TEXTUAL: &str = "[MONETARIO_NAO_TEXTUAL]";

// ---------------------------------------------------------------------------
// Modello in memoria
// ---------------------------------------------------------------------------

/// Identita' del piano dei conti usato da un'esecuzione.
///
/// L'archivio **non** carica il piano: lo registra per permettere al chiamante
/// di accorgersi che quello corrente e' un altro (D8.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentidadePlano {
    pub versao: String,
    pub sha256_arquivo: Option<String>,
    /// Informativo: dato d'ambiente, mai usato per caricare niente.
    pub caminho_declarado: Option<String>,
}

/// Un'esecuzione archiviata, con il **suo** piano.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execucao {
    pub lote: IdentidadeLote,
    pub plano: IdentidadePlano,
    /// Chiavi dei documenti, ordinate.
    pub documentos: Vec<String>,
}

/// Snapshot di un documento, **prima** della proiezione umana.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub lote: String,
    pub chave_origem: String,
    pub impressao_conteudo: String,
    pub impressao_apresentacao: Option<String>,
    pub contexto: ContextoDocumento,
}

/// Una dichiarazione archiviata, con il digest opzionale del suo testo esatto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoArquivado {
    pub evento: EventoRevisao,
    /// `None` ammesso (§7.4): rileva un'alterazione, non autentica nessuno.
    pub impressao_evento: Option<String>,
}

/// L'archivio in memoria.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Arquivo {
    pub execucoes: Vec<Execucao>,
    pub snapshots: Vec<Snapshot>,
    pub eventos: Vec<EventoArquivado>,
}

/// Una presentazione cambiata rispetto a quella archiviata (D8.3).
///
/// **Non** e' un errore: le decisioni restano, il cambiamento si segnala.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergencia {
    pub chave_origem: String,
    pub salva: String,
    pub atual: String,
}

impl Arquivo {
    pub fn novo() -> Self {
        Self::default()
    }

    /// Aggiunge un'esecuzione con i suoi snapshot, calcolando le impronte dai
    /// contesti **originali**.
    ///
    /// Il `plano` serve soltanto all'impronta di presentazione, e arriva come
    /// parametro: questo modulo non lo carica.
    pub fn acrescentar_lote(
        &mut self,
        contextos: &[ContextoDocumento],
        plano: &PlanoContas,
        identidade_plano: IdentidadePlano,
    ) -> Result<(), ErroArquivo> {
        if contextos.is_empty() {
            return Err(ErroArquivo::LoteVazio);
        }
        let lote = contextos[0].lote.clone();
        let mut documentos: Vec<String> = Vec::with_capacity(contextos.len());
        let mut novos: Vec<Snapshot> = Vec::with_capacity(contextos.len());
        for contexto in contextos {
            if contexto.lote != lote {
                return Err(ErroArquivo::SnapshotIncoerente {
                    chave_origem: contexto.chave_origem.clone(),
                    campo: "lote",
                });
            }
            if self
                .snapshots
                .iter()
                .any(|s| s.lote == lote.id && s.chave_origem == contexto.chave_origem)
            {
                return Err(ErroArquivo::SnapshotConflitante {
                    lote: lote.id.clone(),
                    chave_origem: contexto.chave_origem.clone(),
                });
            }
            let conteudo = impressao::impressao_conteudo(contexto);
            let contas = impressao::contas_mostradas(contexto, plano).map_err(|e| {
                ErroArquivo::ContaFora {
                    chave_origem: e.chave_origem,
                    conta_id: e.conta_id,
                }
            })?;
            novos.push(Snapshot {
                lote: lote.id.clone(),
                chave_origem: contexto.chave_origem.clone(),
                impressao_apresentacao: Some(impressao::impressao_apresentacao(&conteudo, &contas)),
                impressao_conteudo: conteudo,
                contexto: contexto.clone(),
            });
            documentos.push(contexto.chave_origem.clone());
        }
        if self.execucoes.iter().any(|e| e.lote.id == lote.id) {
            return Err(ErroArquivo::ExecucaoConflitante { lote: lote.id });
        }
        documentos.sort();
        self.execucoes.push(Execucao {
            lote,
            plano: identidade_plano,
            documentos,
        });
        self.snapshots.extend(novos);
        Ok(())
    }

    /// Aggiunge dichiarazioni. **Non** deduplica: due dichiarazioni identiche
    /// restano due record, e PER-17 vieta di ridurne la molteplicita'.
    pub fn acrescentar_eventos(&mut self, eventos: &[EventoRevisao]) {
        for evento in eventos {
            let digest =
                impressao::sha256_hex(crate::revisao::texto_exato_evento(evento).as_bytes());
            self.eventos.push(EventoArquivado {
                evento: evento.clone(),
                impressao_evento: Some(digest),
            });
        }
    }

    /// Contesti originali di un lotto, nell'ordine in cui sono archiviati.
    pub fn contextos_do_lote(&self, lote: &str) -> Vec<ContextoDocumento> {
        self.snapshots
            .iter()
            .filter(|s| s.lote == lote)
            .map(|s| s.contexto.clone())
            .collect()
    }

    /// Tutte le dichiarazioni, nell'ordine in cui sono archiviate.
    pub fn eventos(&self) -> Vec<EventoRevisao> {
        self.eventos.iter().map(|e| e.evento.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Come `serde` ha rifiutato il testo. Derivata dal messaggio del parser: la
/// **garanzia** e' il rifiuto, la classe e' diagnostica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClasseJson {
    CampoDesconhecido,
    ChaveDuplicada,
    MonetarioNaoTextual,
    Truncado,
    Outro,
}

impl ClasseJson {
    pub const fn rotulo(self) -> &'static str {
        match self {
            ClasseJson::CampoDesconhecido => "CAMPO_DESCONHECIDO",
            ClasseJson::ChaveDuplicada => "CHAVE_DUPLICADA",
            ClasseJson::MonetarioNaoTextual => "MONETARIO_NAO_TEXTUAL",
            ClasseJson::Truncado => "JSON_TRUNCADO",
            ClasseJson::Outro => "JSON_INVALIDO",
        }
    }

    fn de_mensagem(mensagem: &str) -> Self {
        if mensagem.contains(MARCA_NAO_TEXTUAL) {
            ClasseJson::MonetarioNaoTextual
        } else if mensagem.contains("unknown field") {
            ClasseJson::CampoDesconhecido
        } else if mensagem.contains("duplicate field") {
            ClasseJson::ChaveDuplicada
        } else if mensagem.contains("EOF while parsing") {
            ClasseJson::Truncado
        } else {
            ClasseJson::Outro
        }
    }
}

/// Errore del codec o della validazione. Nessun caso produce un archivio
/// parziale: o si legge tutto, o si restituisce l'errore.
#[derive(Debug)]
pub enum ErroArquivo {
    FormatoDesconhecido {
        encontrado: String,
    },
    SchemaMaisNovo {
        encontrado: u32,
        suportado: u32,
    },
    GramaticaDesconhecida {
        campo: &'static str,
        encontrada: String,
        suportada: String,
    },
    /// Rifiuto di `serde`, con la classe diagnostica e la posizione.
    Json {
        classe: ClasseJson,
        linha: usize,
        coluna: usize,
        detalhe: String,
    },
    /// I byte non sono UTF-8, quindi non possono essere un archivio.
    ///
    /// E' un errore di **contenuto**, non di IO: il file si e' letto, e cio'
    /// che si e' letto non e' interpretabile (riscontro R3C3B1-02).
    NaoUtf8 {
        posicao: usize,
    },
    /// Un decimale non conservabile senza perdita.
    DecimalNaoExato {
        caminho: String,
        texto: String,
        motivo: &'static str,
    },
    /// Un validatore del modello ha rifiutato il dato.
    Modelo {
        caminho: String,
        detalhe: String,
    },
    ImpressaoDivergente {
        o_que: &'static str,
        chave_origem: String,
        salva: String,
        recalculada: String,
    },
    ReferenciaRompida {
        de: String,
        para: String,
    },
    SnapshotIncoerente {
        chave_origem: String,
        campo: &'static str,
    },
    SnapshotConflitante {
        lote: String,
        chave_origem: String,
    },
    ExecucaoConflitante {
        lote: String,
    },
    ContaFora {
        chave_origem: String,
        conta_id: String,
    },
    /// L'accrescimento perderebbe storia.
    HistoriaTruncada {
        o_que: String,
    },
    LoteVazio,
}

impl fmt::Display for ErroArquivo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroArquivo::FormatoDesconhecido { encontrado } => {
                write!(f, "formato desconhecido {encontrado:?}: esperado {FORMATO}")
            }
            ErroArquivo::SchemaMaisNovo {
                encontrado,
                suportado,
            } => write!(
                f,
                "versao_schema {encontrado} e' mais nova que a suportada {suportado}: \
                 nenhuma leitura e' tentada"
            ),
            ErroArquivo::GramaticaDesconhecida {
                campo,
                encontrada,
                suportada,
            } => write!(
                f,
                "gramatica {campo} desconhecida: arquivo diz {encontrada:?}, \
                 este codigo usa {suportada:?}"
            ),
            ErroArquivo::Json {
                classe,
                linha,
                coluna,
                detalhe,
            } => write!(
                f,
                "json rejeitado ({}) na linha {linha} coluna {coluna}: {detalhe}",
                classe.rotulo()
            ),
            ErroArquivo::NaoUtf8 { posicao } => write!(
                f,
                "os bytes nao sao UTF-8 valido a partir da posicao {posicao}: \
                 nenhuma conversao com perda e' tentada para fabricar um texto"
            ),
            ErroArquivo::DecimalNaoExato {
                caminho,
                texto,
                motivo,
            } => write!(
                f,
                "decimal nao exato em {caminho}: {texto:?} ({motivo}). \
                 Nenhum arredondamento corretivo e' aplicado"
            ),
            ErroArquivo::Modelo { caminho, detalhe } => {
                write!(f, "dado invalido em {caminho}: {detalhe}")
            }
            ErroArquivo::ImpressaoDivergente {
                o_que,
                chave_origem,
                salva,
                recalculada,
            } => write!(
                f,
                "impressao de {o_que} divergente em {chave_origem}: \
                 salva {salva}, recalculada {recalculada}. O hash nao e' atualizado \
                 para aceitar um conteudo alterado"
            ),
            ErroArquivo::ReferenciaRompida { de, para } => {
                write!(f, "referencia rompida: {de} aponta para {para}, ausente")
            }
            ErroArquivo::SnapshotIncoerente {
                chave_origem,
                campo,
            } => write!(
                f,
                "snapshot incoerente em {chave_origem}: o campo {campo} nao coincide \
                 com o contexto conservado"
            ),
            ErroArquivo::SnapshotConflitante { lote, chave_origem } => write!(
                f,
                "dois conteudos para ({lote}, {chave_origem}): nao e' uma historia, \
                 e' um conflito"
            ),
            ErroArquivo::ExecucaoConflitante { lote } => {
                write!(f, "execucao {lote} ja' presente no arquivo")
            }
            ErroArquivo::ContaFora {
                chave_origem,
                conta_id,
            } => write!(
                f,
                "conta {conta_id} do documento {chave_origem} ausente do plano recebido"
            ),
            ErroArquivo::HistoriaTruncada { o_que } => write!(
                f,
                "a publicacao perderia historia: {o_que}. O arquivo e' logicamente \
                 append-only"
            ),
            ErroArquivo::LoteVazio => write!(f, "lote sem documentos: nada a arquivar"),
        }
    }
}

impl std::error::Error for ErroArquivo {}

fn de_serde(erro: serde_json::Error) -> ErroArquivo {
    let detalhe = erro.to_string();
    ErroArquivo::Json {
        classe: ClasseJson::de_mensagem(&detalhe),
        linha: erro.line(),
        coluna: erro.column(),
        detalhe,
    }
}

// ---------------------------------------------------------------------------
// Decimali: testo, e solo testo
// ---------------------------------------------------------------------------

/// Un decimale come **testo**. Un numero JSON viene rifiutato qui, prima di
/// qualunque conversione (PER-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TextoDecimal(String);

impl TextoDecimal {
    fn de(valor: Decimal) -> Self {
        TextoDecimal(valor.to_string())
    }

    /// PER-20: conversione senza perdita, piu' confronto del testo riprodotto.
    fn exato(&self, caminho: &str) -> Result<Decimal, ErroArquivo> {
        let valor = Decimal::from_str_exact(&self.0).map_err(|_| ErroArquivo::DecimalNaoExato {
            caminho: caminho.to_string(),
            texto: self.0.clone(),
            motivo: "nao representavel sem perda",
        })?;
        if valor.to_string() != self.0 {
            return Err(ErroArquivo::DecimalNaoExato {
                caminho: caminho.to_string(),
                texto: self.0.clone(),
                motivo: "forma nao canonica",
            });
        }
        Ok(valor)
    }
}

impl<'de> Deserialize<'de> for TextoDecimal {
    fn deserialize<D: Deserializer<'de>>(deserializador: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = TextoDecimal;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "um decimal como texto")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<TextoDecimal, E> {
                Ok(TextoDecimal(v.to_string()))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<TextoDecimal, E> {
                Err(E::custom(format!("{MARCA_NAO_TEXTUAL} numero {v}")))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<TextoDecimal, E> {
                Err(E::custom(format!("{MARCA_NAO_TEXTUAL} numero {v}")))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<TextoDecimal, E> {
                Err(E::custom(format!("{MARCA_NAO_TEXTUAL} numero {v}")))
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<TextoDecimal, E> {
                Err(E::custom(format!("{MARCA_NAO_TEXTUAL} booleano {v}")))
            }

            fn visit_unit<E: de::Error>(self) -> Result<TextoDecimal, E> {
                Err(E::custom(format!("{MARCA_NAO_TEXTUAL} null")))
            }
        }
        deserializador.deserialize_any(V)
    }
}

// ---------------------------------------------------------------------------
// DTO: la forma, con deny_unknown_fields su ogni struct
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
struct DadoDto<T> {
    #[serde(default = "sem_valor")]
    valor: Option<T>,
    fonte: String,
    status: StatusDado,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    motivo: Option<String>,
}

fn sem_valor<T>() -> Option<T> {
    None
}

impl<T> DadoDto<T> {
    fn de<U>(dado: &Dado<U>, mapear: impl Fn(&U) -> T) -> Self {
        DadoDto {
            valor: dado.valor().map(mapear),
            fonte: dado.fonte().to_string(),
            status: dado.status(),
            motivo: dado.motivo().map(|m| m.to_string()),
        }
    }

    fn para<U>(
        self,
        caminho: &str,
        converter: impl Fn(T, &str) -> Result<U, ErroArquivo>,
    ) -> Result<Dado<U>, ErroArquivo> {
        let valor = match self.valor {
            Some(v) => Some(converter(v, caminho)?),
            None => None,
        };
        Dado::novo(valor, self.fonte, self.status, self.motivo).map_err(|e| ErroArquivo::Modelo {
            caminho: caminho.to_string(),
            detalhe: e.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AvisoDto {
    codigo: String,
    detalhe: String,
}

impl AvisoDto {
    fn de(aviso: &Aviso) -> Self {
        AvisoDto {
            codigo: aviso.codigo.clone(),
            detalhe: aviso.detalhe.clone(),
        }
    }
    fn para(self) -> Aviso {
        Aviso {
            codigo: self.codigo,
            detalhe: self.detalhe,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoteDto {
    id: String,
    gerado_em: NaiveDate,
    descricao: String,
}

impl LoteDto {
    fn de(lote: &IdentidadeLote) -> Self {
        LoteDto {
            id: lote.id.clone(),
            gerado_em: lote.gerado_em,
            descricao: lote.descricao.clone(),
        }
    }
    fn para(self) -> IdentidadeLote {
        IdentidadeLote {
            id: self.id,
            gerado_em: self.gerado_em,
            descricao: self.descricao,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DuplicataDto {
    numero: String,
    vencimento: Option<NaiveDate>,
    valor: TextoDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponenteDto {
    nome: String,
    valor: DadoDto<TextoDecimal>,
    ruolo: RuoloComponente,
    usada_na_verificacao: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CampoRinviatoDto {
    campo: String,
    status: String,
    fonte: String,
    motivo: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconciliacaoDto {
    valor_total_nfe: TextoDecimal,
    soma_lancamentos: TextoDecimal,
    esito_total: EsitoTotal,
    soma_rate_documento: DadoDto<TextoDecimal>,
    soma_rate_allocate: DadoDto<TextoDecimal>,
    esito_rate: EsitoRate,
    articoli_allocati: usize,
    articoli_attesi: DadoDto<usize>,
    articoli_scartati: Vec<AvisoDto>,
    esito_copertura: EsitoCopertura,
    componenti: Vec<ComponenteDto>,
    esito_componenti: EsitoComponenti,
    campi_rinviati: Vec<CampoRinviatoDto>,
    avisos_documento: Vec<AvisoDto>,
    stato_documento: StatoDocumento,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LancamentoDto {
    data_documento: NaiveDate,
    competencia: Competencia,
    cnpj_empresa: String,
    nome_empresa: String,
    fornecedor_ou_favorecido: String,
    cnpj_cpf_favorecido: String,
    documento: String,
    tipo_documento: String,
    conta_id: String,
    conta_nome: String,
    grupo_conta: String,
    descricao_itens: String,
    valor: TextoDecimal,
    duplicatas: Vec<DuplicataDto>,
    origem: OrigemLancamento,
    arquivo_origem: String,
    chave_origem: String,
    hash_registro: String,
    status_revisao: StatusRevisao,
    alertas: String,
    observacoes: String,
    centro_custo: String,
    tipo_lancamento: TipoConta,
    dre_linha_id: String,
    dre_linha_nome: String,
    data_pagamento: DadoDto<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlocacaoDto {
    lancamentos: Vec<LancamentoDto>,
    reconciliacao: ReconciliacaoDto,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextoDto {
    chave_origem: String,
    numero: String,
    serie: String,
    data_documento: NaiveDate,
    competencia: Competencia,
    cnpj_empresa: String,
    nome_empresa: String,
    arquivo_origem: String,
    alocacao: AlocacaoDto,
    avisos_leitura: Vec<AvisoDto>,
    lote: LoteDto,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanoDto {
    versao: String,
    sha256_arquivo: Option<String>,
    caminho_declarado: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecucaoDto {
    lote: LoteDto,
    plano: PlanoDto,
    documentos: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotDto {
    lote: String,
    chave_origem: String,
    impressao_conteudo: String,
    impressao_apresentacao: Option<String>,
    contexto: ContextoDto,
}

/// Istante dichiarato, in due campi: §3.4 della specifica dimostra che la
/// stringa RFC3339 **perde i secondi dell'offset**.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstanteDto {
    local: String,
    offset_segundos: i32,
}

const FORMATO_LOCAL: &str = "%Y-%m-%dT%H:%M:%S%.9f";

impl InstanteDto {
    fn de(instante: DateTime<FixedOffset>) -> Self {
        InstanteDto {
            local: instante.naive_local().format(FORMATO_LOCAL).to_string(),
            offset_segundos: instante.offset().local_minus_utc(),
        }
    }

    fn para(self, caminho: &str) -> Result<DateTime<FixedOffset>, ErroArquivo> {
        let erro = |detalhe: String| ErroArquivo::Modelo {
            caminho: caminho.to_string(),
            detalhe,
        };
        let local = NaiveDateTime::parse_from_str(&self.local, FORMATO_LOCAL)
            .map_err(|e| erro(format!("instante local {:?}: {e}", self.local)))?;
        let offset = FixedOffset::east_opt(self.offset_segundos).ok_or_else(|| {
            erro(format!(
                "offset {} segundos fora do intervalo admitido",
                self.offset_segundos
            ))
        })?;
        local
            .and_local_timezone(offset)
            .single()
            .ok_or_else(|| erro("instante local ambiguo no offset declarado".to_string()))
    }
}

/// Operazione nel DTO.
///
/// `OperacaoRevisao` deriva soltanto `Serialize` (`revisao.rs:103`) e quel
/// modulo e' auditato e non si tocca: qui serve un enum locale con le
/// **stesse** etichette. Il test `etiquetas_dos_enums_do_arquivo` confronta
/// ogni variante con `OperacaoRevisao::rotulo()`, cosi' una deriva fallisce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OperacaoDto {
    Aprovar,
    Revogar,
    Anotar,
}

impl OperacaoDto {
    fn de(operacao: OperacaoRevisao) -> Self {
        match operacao {
            OperacaoRevisao::Aprovar => OperacaoDto::Aprovar,
            OperacaoRevisao::Revogar => OperacaoDto::Revogar,
            OperacaoRevisao::Anotar => OperacaoDto::Anotar,
        }
    }

    fn para(self) -> OperacaoRevisao {
        match self {
            OperacaoDto::Aprovar => OperacaoRevisao::Aprovar,
            OperacaoDto::Revogar => OperacaoRevisao::Revogar,
            OperacaoDto::Anotar => OperacaoRevisao::Anotar,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlvoDto {
    chave_origem: String,
    impressao_conteudo: DadoDto<String>,
    origem: OrigemLancamento,
    conta_id: String,
    ordinal: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventoDto {
    alvo: AlvoDto,
    operacao: OperacaoDto,
    autor: String,
    instante: InstanteDto,
    motivo: String,
    nota: Option<String>,
    sequencia: Option<u64>,
    vista: DadoDto<String>,
    impressao_evento: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GramaticasDto {
    impressao_conteudo: String,
    impressao_apresentacao: String,
    evento_exato: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArquivoDto {
    formato: String,
    versao_schema: u32,
    revisao_arquivo: u64,
    gramaticas: GramaticasDto,
    execucoes: Vec<ExecucaoDto>,
    snapshots: Vec<SnapshotDto>,
    eventos: Vec<EventoDto>,
}

// ---------------------------------------------------------------------------
// Conversioni: modello -> DTO
// ---------------------------------------------------------------------------

fn dto_de_duplicata(d: &Duplicata) -> DuplicataDto {
    DuplicataDto {
        numero: d.numero.clone(),
        vencimento: d.vencimento,
        valor: TextoDecimal::de(d.valor),
    }
}

fn dto_de_componente(c: &Componente) -> ComponenteDto {
    ComponenteDto {
        nome: c.nome.clone(),
        valor: DadoDto::de(&c.valor, |v| TextoDecimal::de(*v)),
        ruolo: c.ruolo,
        usada_na_verificacao: c.usada_na_verificacao,
    }
}

fn dto_de_reconciliacao(r: &Reconciliacao) -> ReconciliacaoDto {
    ReconciliacaoDto {
        valor_total_nfe: TextoDecimal::de(r.valor_total_nfe),
        soma_lancamentos: TextoDecimal::de(r.soma_lancamentos),
        esito_total: r.esito_total,
        soma_rate_documento: DadoDto::de(&r.soma_rate_documento, |v| TextoDecimal::de(*v)),
        soma_rate_allocate: DadoDto::de(&r.soma_rate_allocate, |v| TextoDecimal::de(*v)),
        esito_rate: r.esito_rate,
        articoli_allocati: r.articoli_allocati,
        articoli_attesi: DadoDto::de(&r.articoli_attesi, |v| *v),
        articoli_scartati: r.articoli_scartati.iter().map(AvisoDto::de).collect(),
        esito_copertura: r.esito_copertura,
        componenti: r.componenti.iter().map(dto_de_componente).collect(),
        esito_componenti: r.esito_componenti,
        campi_rinviati: r
            .campi_rinviati
            .iter()
            .map(|c| CampoRinviatoDto {
                campo: c.campo.clone(),
                status: c.status.clone(),
                fonte: c.fonte.clone(),
                motivo: c.motivo.clone(),
            })
            .collect(),
        avisos_documento: r.avisos_documento.iter().map(AvisoDto::de).collect(),
        stato_documento: r.stato_documento,
    }
}

fn dto_de_lancamento(l: &Lancamento) -> LancamentoDto {
    LancamentoDto {
        data_documento: l.data_documento,
        competencia: l.competencia,
        cnpj_empresa: l.cnpj_empresa.clone(),
        nome_empresa: l.nome_empresa.clone(),
        fornecedor_ou_favorecido: l.fornecedor_ou_favorecido.clone(),
        cnpj_cpf_favorecido: l.cnpj_cpf_favorecido.clone(),
        documento: l.documento.clone(),
        tipo_documento: l.tipo_documento.clone(),
        conta_id: l.conta_id.clone(),
        conta_nome: l.conta_nome.clone(),
        grupo_conta: l.grupo_conta.clone(),
        descricao_itens: l.descricao_itens.clone(),
        valor: TextoDecimal::de(l.valor),
        duplicatas: l.duplicatas.iter().map(dto_de_duplicata).collect(),
        origem: l.origem,
        arquivo_origem: l.arquivo_origem.clone(),
        chave_origem: l.chave_origem.clone(),
        hash_registro: l.hash_registro.clone(),
        status_revisao: l.status_revisao,
        alertas: l.alertas.clone(),
        observacoes: l.observacoes.clone(),
        centro_custo: l.centro_custo.clone(),
        tipo_lancamento: l.tipo_lancamento,
        dre_linha_id: l.dre_linha_id.clone(),
        dre_linha_nome: l.dre_linha_nome.clone(),
        data_pagamento: DadoDto::de(&l.data_pagamento, |d| *d),
    }
}

fn dto_de_contexto(c: &ContextoDocumento) -> ContextoDto {
    ContextoDto {
        chave_origem: c.chave_origem.clone(),
        numero: c.numero.clone(),
        serie: c.serie.clone(),
        data_documento: c.data_documento,
        competencia: c.competencia,
        cnpj_empresa: c.cnpj_empresa.clone(),
        nome_empresa: c.nome_empresa.clone(),
        arquivo_origem: c.arquivo_origem.clone(),
        alocacao: AlocacaoDto {
            lancamentos: c
                .alocacao
                .lancamentos
                .iter()
                .map(dto_de_lancamento)
                .collect(),
            reconciliacao: dto_de_reconciliacao(&c.alocacao.reconciliacao),
        },
        avisos_leitura: c.avisos_leitura.iter().map(AvisoDto::de).collect(),
        lote: LoteDto::de(&c.lote),
    }
}

fn dto_de_evento(e: &EventoArquivado) -> EventoDto {
    EventoDto {
        alvo: AlvoDto {
            chave_origem: e.evento.alvo.chave_origem.clone(),
            impressao_conteudo: DadoDto::de(&e.evento.alvo.impressao_conteudo, |v| v.clone()),
            origem: e.evento.alvo.origem,
            conta_id: e.evento.alvo.conta_id.clone(),
            ordinal: e.evento.alvo.ordinal,
        },
        operacao: OperacaoDto::de(e.evento.operacao),
        autor: e.evento.autor.clone(),
        instante: InstanteDto::de(e.evento.instante),
        motivo: e.evento.motivo.clone(),
        nota: e.evento.nota.clone(),
        sequencia: e.evento.sequencia,
        vista: DadoDto::de(&e.evento.vista, |v| v.clone()),
        impressao_evento: e.impressao_evento.clone(),
    }
}

/// L'ordine dei documenti di un'esecuzione **non** e' un dato: si ordina sia
/// in scrittura sia nel confronto, cosi' una permutazione non sembra una
/// modifica.
fn dto_de_execucao(execucao: &Execucao) -> ExecucaoDto {
    let mut documentos = execucao.documentos.clone();
    documentos.sort();
    ExecucaoDto {
        lote: LoteDto::de(&execucao.lote),
        plano: PlanoDto {
            versao: execucao.plano.versao.clone(),
            sha256_arquivo: execucao.plano.sha256_arquivo.clone(),
            caminho_declarado: execucao.plano.caminho_declarado.clone(),
        },
        documentos,
    }
}

fn dto_de_snapshot(snapshot: &Snapshot) -> SnapshotDto {
    SnapshotDto {
        lote: snapshot.lote.clone(),
        chave_origem: snapshot.chave_origem.clone(),
        impressao_conteudo: snapshot.impressao_conteudo.clone(),
        impressao_apresentacao: snapshot.impressao_apresentacao.clone(),
        contexto: dto_de_contexto(&snapshot.contexto),
    }
}

// ---------------------------------------------------------------------------
// Conversioni: DTO -> modello, con il percorso negli errori
// ---------------------------------------------------------------------------

fn texto_exato(valor: TextoDecimal, caminho: &str) -> Result<Decimal, ErroArquivo> {
    valor.exato(caminho)
}

fn identico<T>(valor: T, _caminho: &str) -> Result<T, ErroArquivo> {
    Ok(valor)
}

fn para_duplicata(d: DuplicataDto, caminho: &str) -> Result<Duplicata, ErroArquivo> {
    Ok(Duplicata {
        valor: d.valor.exato(&format!("{caminho}.valor"))?,
        numero: d.numero,
        vencimento: d.vencimento,
    })
}

fn para_componente(c: ComponenteDto, caminho: &str) -> Result<Componente, ErroArquivo> {
    Ok(Componente {
        valor: c.valor.para(&format!("{caminho}.valor"), texto_exato)?,
        nome: c.nome,
        ruolo: c.ruolo,
        usada_na_verificacao: c.usada_na_verificacao,
    })
}

fn para_reconciliacao(r: ReconciliacaoDto, caminho: &str) -> Result<Reconciliacao, ErroArquivo> {
    let mut componenti = Vec::with_capacity(r.componenti.len());
    for (i, c) in r.componenti.into_iter().enumerate() {
        componenti.push(para_componente(c, &format!("{caminho}.componenti[{i}]"))?);
    }
    Ok(Reconciliacao {
        valor_total_nfe: r
            .valor_total_nfe
            .exato(&format!("{caminho}.valor_total_nfe"))?,
        soma_lancamentos: r
            .soma_lancamentos
            .exato(&format!("{caminho}.soma_lancamentos"))?,
        esito_total: r.esito_total,
        soma_rate_documento: r
            .soma_rate_documento
            .para(&format!("{caminho}.soma_rate_documento"), texto_exato)?,
        soma_rate_allocate: r
            .soma_rate_allocate
            .para(&format!("{caminho}.soma_rate_allocate"), texto_exato)?,
        esito_rate: r.esito_rate,
        articoli_allocati: r.articoli_allocati,
        articoli_attesi: r
            .articoli_attesi
            .para(&format!("{caminho}.articoli_attesi"), identico)?,
        articoli_scartati: r
            .articoli_scartati
            .into_iter()
            .map(AvisoDto::para)
            .collect(),
        esito_copertura: r.esito_copertura,
        componenti,
        esito_componenti: r.esito_componenti,
        campi_rinviati: r
            .campi_rinviati
            .into_iter()
            .map(|c| CampoRinviato {
                campo: c.campo,
                status: c.status,
                fonte: c.fonte,
                motivo: c.motivo,
            })
            .collect(),
        avisos_documento: r.avisos_documento.into_iter().map(AvisoDto::para).collect(),
        stato_documento: r.stato_documento,
    })
}

fn para_lancamento(l: LancamentoDto, caminho: &str) -> Result<Lancamento, ErroArquivo> {
    let mut duplicatas = Vec::with_capacity(l.duplicatas.len());
    for (i, d) in l.duplicatas.into_iter().enumerate() {
        duplicatas.push(para_duplicata(d, &format!("{caminho}.duplicatas[{i}]"))?);
    }
    Ok(Lancamento {
        valor: l.valor.exato(&format!("{caminho}.valor"))?,
        data_pagamento: l
            .data_pagamento
            .para(&format!("{caminho}.data_pagamento"), identico)?,
        data_documento: l.data_documento,
        competencia: l.competencia,
        cnpj_empresa: l.cnpj_empresa,
        nome_empresa: l.nome_empresa,
        fornecedor_ou_favorecido: l.fornecedor_ou_favorecido,
        cnpj_cpf_favorecido: l.cnpj_cpf_favorecido,
        documento: l.documento,
        tipo_documento: l.tipo_documento,
        conta_id: l.conta_id,
        conta_nome: l.conta_nome,
        grupo_conta: l.grupo_conta,
        descricao_itens: l.descricao_itens,
        duplicatas,
        origem: l.origem,
        arquivo_origem: l.arquivo_origem,
        chave_origem: l.chave_origem,
        hash_registro: l.hash_registro,
        status_revisao: l.status_revisao,
        alertas: l.alertas,
        observacoes: l.observacoes,
        centro_custo: l.centro_custo,
        tipo_lancamento: l.tipo_lancamento,
        dre_linha_id: l.dre_linha_id,
        dre_linha_nome: l.dre_linha_nome,
    })
}

fn para_contexto(c: ContextoDto, caminho: &str) -> Result<ContextoDocumento, ErroArquivo> {
    let mut lancamentos = Vec::with_capacity(c.alocacao.lancamentos.len());
    for (i, l) in c.alocacao.lancamentos.into_iter().enumerate() {
        lancamentos.push(para_lancamento(
            l,
            &format!("{caminho}.alocacao.lancamentos[{i}]"),
        )?);
    }
    Ok(ContextoDocumento {
        chave_origem: c.chave_origem,
        numero: c.numero,
        serie: c.serie,
        data_documento: c.data_documento,
        competencia: c.competencia,
        cnpj_empresa: c.cnpj_empresa,
        nome_empresa: c.nome_empresa,
        arquivo_origem: c.arquivo_origem,
        alocacao: AlocacaoNfe {
            lancamentos,
            reconciliacao: para_reconciliacao(
                c.alocacao.reconciliacao,
                &format!("{caminho}.alocacao.reconciliacao"),
            )?,
        },
        avisos_leitura: c.avisos_leitura.into_iter().map(AvisoDto::para).collect(),
        lote: c.lote.para(),
    })
}

fn para_evento(e: EventoDto, caminho: &str) -> Result<EventoArquivado, ErroArquivo> {
    let instante = e.instante.para(&format!("{caminho}.instante"))?;
    Ok(EventoArquivado {
        evento: EventoRevisao {
            alvo: ChaveRevisao {
                impressao_conteudo: e
                    .alvo
                    .impressao_conteudo
                    .para(&format!("{caminho}.alvo.impressao_conteudo"), identico)?,
                chave_origem: e.alvo.chave_origem,
                origem: e.alvo.origem,
                conta_id: e.alvo.conta_id,
                ordinal: e.alvo.ordinal,
            },
            operacao: e.operacao.para(),
            autor: e.autor,
            instante,
            motivo: e.motivo,
            nota: e.nota,
            sequencia: e.sequencia,
            vista: e.vista.para(&format!("{caminho}.vista"), identico)?,
        },
        impressao_evento: e.impressao_evento,
    })
}

// ---------------------------------------------------------------------------
// Ordine fisico (§8.4)
// ---------------------------------------------------------------------------

/// Chiave di ordine fisico degli eventi. L'ultimo livello e' il **record
/// archivistico serializzato**: copre anche i campi che la dichiarazione umana
/// non ha, come `impressao_evento` (riscontro R3C3B-04).
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ChaveFisica {
    instante: DateTime<Utc>,
    offset_segundos: i32,
    chave_origem: String,
    conta_id: String,
    ordinal: u32,
    operacao: &'static str,
    texto_exato: String,
    record: String,
}

/// Codifica esatta di un record, quella che finisce sul disco.
///
/// Usata per l'ordine fisico degli eventi e per il confronto
/// dell'accrescimento: e' generica perche' snapshot ed esecuzioni hanno
/// bisogno della stessa garanzia (riscontro R3C3B1-01).
fn texto_record<T: Serialize>(dto: &T) -> Result<String, ErroArquivo> {
    serde_json::to_string(dto).map_err(de_serde)
}

fn chave_fisica(evento: &EventoArquivado, dto: &EventoDto) -> Result<ChaveFisica, ErroArquivo> {
    Ok(ChaveFisica {
        instante: evento.evento.instante.with_timezone(&Utc),
        offset_segundos: evento.evento.instante.offset().local_minus_utc(),
        chave_origem: evento.evento.alvo.chave_origem.clone(),
        conta_id: evento.evento.alvo.conta_id.clone(),
        ordinal: evento.evento.alvo.ordinal,
        operacao: evento.evento.operacao.rotulo(),
        texto_exato: crate::revisao::texto_exato_evento(&evento.evento),
        record: texto_record(dto)?,
    })
}

// ---------------------------------------------------------------------------
// Codec
// ---------------------------------------------------------------------------

/// Serializza l'archivio con l'ordine fisico di §8.4.
///
/// `revisao` e' un parametro: la assegna il pubblicatore sotto lucchetto
/// (§9.1 passo 4), non questo modulo.
pub fn serializar(arquivo: &Arquivo, revisao: u64) -> Result<String, ErroArquivo> {
    let mut execucoes: Vec<ExecucaoDto> = arquivo.execucoes.iter().map(dto_de_execucao).collect();
    execucoes.sort_by(|a, b| a.lote.id.cmp(&b.lote.id));

    let mut snapshots: Vec<SnapshotDto> = arquivo.snapshots.iter().map(dto_de_snapshot).collect();
    snapshots.sort_by(|a, b| (&a.lote, &a.chave_origem).cmp(&(&b.lote, &b.chave_origem)));

    // Ogni record si serializza in modo indipendente, poi si ordina, poi si
    // uniscono: nessuna circolarita'.
    let mut eventos: Vec<(ChaveFisica, EventoDto)> = Vec::with_capacity(arquivo.eventos.len());
    for evento in &arquivo.eventos {
        let dto = dto_de_evento(evento);
        let chave = chave_fisica(evento, &dto)?;
        eventos.push((chave, dto));
    }
    eventos.sort_by(|a, b| a.0.cmp(&b.0));

    let dto = ArquivoDto {
        formato: FORMATO.to_string(),
        versao_schema: VERSAO_SCHEMA,
        revisao_arquivo: revisao,
        gramaticas: GramaticasDto {
            impressao_conteudo: impressao::VERSAO_CONTEUDO.to_string(),
            impressao_apresentacao: impressao::VERSAO_APRESENTACAO.to_string(),
            evento_exato: crate::revisao::VERSAO_EVENTO_EXATO.to_string(),
        },
        execucoes,
        snapshots,
        eventos: eventos.into_iter().map(|(_, dto)| dto).collect(),
    };
    let mut texto = serde_json::to_string_pretty(&dto).map_err(de_serde)?;
    texto.push('\n');
    Ok(texto)
}

/// Come [`desserializar`], a partire dai **byte**.
///
/// Esiste perche' lo strato IO deve poter distinguere «non ho potuto leggere
/// il file» da «ho letto byte che non sono un archivio» (riscontro
/// R3C3B1-02). Nessuna conversione con perdita: byte non UTF-8 sono
/// [`ErroArquivo::NaoUtf8`], non un testo inventato.
pub fn desserializar_bytes(bytes: &[u8]) -> Result<(Arquivo, u64), ErroArquivo> {
    let texto = std::str::from_utf8(bytes).map_err(|e| ErroArquivo::NaoUtf8 {
        posicao: e.valid_up_to(),
    })?;
    desserializar(texto)
}

/// Legge l'archivio con i tre passaggi di §7.3.
///
/// Restituisce anche la `revisao_arquivo` dichiarata dal file: e' un metadato
/// del file, non dell'archivio in memoria.
pub fn desserializar(texto: &str) -> Result<(Arquivo, u64), ErroArquivo> {
    let dto: ArquivoDto = serde_json::from_str(texto).map_err(de_serde)?;

    if dto.formato != FORMATO {
        return Err(ErroArquivo::FormatoDesconhecido {
            encontrado: dto.formato,
        });
    }
    if dto.versao_schema != VERSAO_SCHEMA {
        return Err(ErroArquivo::SchemaMaisNovo {
            encontrado: dto.versao_schema,
            suportado: VERSAO_SCHEMA,
        });
    }
    for (campo, encontrada, suportada) in [
        (
            "impressao_conteudo",
            &dto.gramaticas.impressao_conteudo,
            impressao::VERSAO_CONTEUDO,
        ),
        (
            "impressao_apresentacao",
            &dto.gramaticas.impressao_apresentacao,
            impressao::VERSAO_APRESENTACAO,
        ),
        (
            "evento_exato",
            &dto.gramaticas.evento_exato,
            crate::revisao::VERSAO_EVENTO_EXATO,
        ),
    ] {
        if encontrada != suportada {
            return Err(ErroArquivo::GramaticaDesconhecida {
                campo,
                encontrada: encontrada.clone(),
                suportada: suportada.to_string(),
            });
        }
    }

    let execucoes = dto
        .execucoes
        .into_iter()
        .map(|e| Execucao {
            lote: e.lote.para(),
            plano: IdentidadePlano {
                versao: e.plano.versao,
                sha256_arquivo: e.plano.sha256_arquivo,
                caminho_declarado: e.plano.caminho_declarado,
            },
            documentos: e.documentos,
        })
        .collect();

    let mut snapshots = Vec::with_capacity(dto.snapshots.len());
    for (i, s) in dto.snapshots.into_iter().enumerate() {
        let caminho = format!("snapshots[{i}]");
        snapshots.push(Snapshot {
            contexto: para_contexto(s.contexto, &format!("{caminho}.contexto"))?,
            lote: s.lote,
            chave_origem: s.chave_origem,
            impressao_conteudo: s.impressao_conteudo,
            impressao_apresentacao: s.impressao_apresentacao,
        });
    }

    let mut eventos = Vec::with_capacity(dto.eventos.len());
    for (i, e) in dto.eventos.into_iter().enumerate() {
        eventos.push(para_evento(e, &format!("eventos[{i}]"))?);
    }

    Ok((
        Arquivo {
            execucoes,
            snapshots,
            eventos,
        },
        dto.revisao_arquivo,
    ))
}

// ---------------------------------------------------------------------------
// Verifiche
// ---------------------------------------------------------------------------

/// Verifica che **non** richiede il piano: riferimenti, coerenza
/// snapshot/contesto e impronte ricalcolate (PER-13, PER-14).
///
/// Riscontro R3C3B1-03 del 2026-10-22: l'unicita' del lotto era imposta solo
/// da `Arquivo::acrescentar_lote`, e il modello e' pubblico mentre il lettore
/// non passa per quel costruttore. Due esecuzioni con lo stesso `lote.id`
/// significano due identita' del piano per lo stesso lotto, e rendono ambiguo
/// il riferimento di ogni snapshot: qui vengono rifiutate, con lo stesso
/// errore del costruttore. **Non** si deduplica scegliendo la prima o
/// l'ultima: sarebbe una scelta inventata su dati contabili.
pub fn verificar_estrutura(arquivo: &Arquivo) -> Result<(), ErroArquivo> {
    let mut lotes: BTreeMap<&str, ()> = BTreeMap::new();
    for execucao in &arquivo.execucoes {
        if lotes.insert(execucao.lote.id.as_str(), ()).is_some() {
            return Err(ErroArquivo::ExecucaoConflitante {
                lote: execucao.lote.id.clone(),
            });
        }
    }

    let mut vistos: BTreeMap<(&str, &str), ()> = BTreeMap::new();
    for snapshot in &arquivo.snapshots {
        if snapshot.lote != snapshot.contexto.lote.id {
            return Err(ErroArquivo::SnapshotIncoerente {
                chave_origem: snapshot.chave_origem.clone(),
                campo: "lote",
            });
        }
        if snapshot.chave_origem != snapshot.contexto.chave_origem {
            return Err(ErroArquivo::SnapshotIncoerente {
                chave_origem: snapshot.chave_origem.clone(),
                campo: "chave_origem",
            });
        }
        if vistos
            .insert((&snapshot.lote, &snapshot.chave_origem), ())
            .is_some()
        {
            return Err(ErroArquivo::SnapshotConflitante {
                lote: snapshot.lote.clone(),
                chave_origem: snapshot.chave_origem.clone(),
            });
        }
        let recalculada = impressao::impressao_conteudo(&snapshot.contexto);
        if recalculada != snapshot.impressao_conteudo {
            return Err(ErroArquivo::ImpressaoDivergente {
                o_que: "conteudo",
                chave_origem: snapshot.chave_origem.clone(),
                salva: snapshot.impressao_conteudo.clone(),
                recalculada,
            });
        }
        if !arquivo.execucoes.iter().any(|e| e.lote.id == snapshot.lote) {
            return Err(ErroArquivo::ReferenciaRompida {
                de: format!("snapshot {}", snapshot.chave_origem),
                para: format!("execucao {}", snapshot.lote),
            });
        }
    }

    for execucao in &arquivo.execucoes {
        for documento in &execucao.documentos {
            let encontrados = arquivo
                .snapshots
                .iter()
                .filter(|s| s.lote == execucao.lote.id && &s.chave_origem == documento)
                .count();
            if encontrados != 1 {
                return Err(ErroArquivo::ReferenciaRompida {
                    de: format!("execucao {}", execucao.lote.id),
                    para: format!("snapshot {documento} ({encontrados} encontrados)"),
                });
            }
        }
    }

    for evento in &arquivo.eventos {
        if let Some(salva) = &evento.impressao_evento {
            let recalculada = impressao::sha256_hex(
                crate::revisao::texto_exato_evento(&evento.evento).as_bytes(),
            );
            if &recalculada != salva {
                return Err(ErroArquivo::ImpressaoDivergente {
                    o_que: "evento",
                    chave_origem: evento.evento.alvo.chave_origem.clone(),
                    salva: salva.clone(),
                    recalculada,
                });
            }
        }
    }
    Ok(())
}

/// Confronto della **presentazione**, che richiede il piano corrente.
///
/// Una divergenza **non** e' un errore: e' `APRESENTACAO_MUDADA` secondo
/// D8.3, e le decisioni restano.
pub fn verificar_apresentacao(
    arquivo: &Arquivo,
    plano: &PlanoContas,
) -> Result<Vec<Divergencia>, ErroArquivo> {
    let mut divergencias = Vec::new();
    for snapshot in &arquivo.snapshots {
        let Some(salva) = &snapshot.impressao_apresentacao else {
            continue;
        };
        let contas = impressao::contas_mostradas(&snapshot.contexto, plano).map_err(|e| {
            ErroArquivo::ContaFora {
                chave_origem: e.chave_origem,
                conta_id: e.conta_id,
            }
        })?;
        let atual = impressao::impressao_apresentacao(&snapshot.impressao_conteudo, &contas);
        if &atual != salva {
            divergencias.push(Divergencia {
                chave_origem: snapshot.chave_origem.clone(),
                salva: salva.clone(),
                atual,
            });
        }
    }
    Ok(divergencias)
}

/// Append-only (PER-17): confronta il **file che si sta sostituendo** con il
/// candidato.
///
/// Gli eventi si confrontano come **multinsieme**: una base con due
/// dichiarazioni identiche non puo' diventare un file con una sola.
///
/// # Perche' il confronto non usa `PartialEq`
///
/// Riscontro R3C3B1-01 del 2026-10-22: `PartialEq` di `Decimal` e'
/// l'uguaglianza **numerica**, quindi `10.00 == 10.0000`. L'impronta canonica
/// non puo' coprire il buco, perche' normalizza la scala prima di confrontare:
/// erano due controlli ciechi sulla stessa cosa. La scala invece e' un dato
/// originale che il codec conserva, e cambiarla e' una modifica dello storico.
///
/// Quindi i record protetti si confrontano sulla loro **codifica esatta**, la
/// stessa che finisce sul disco. Non si confronta il file intero, perche'
/// revisione e ordine fisico hanno regole proprie e cambiano legittimamente.
pub fn verificar_acrescimo(corrente: &Arquivo, candidato: &Arquivo) -> Result<(), ErroArquivo> {
    for execucao in &corrente.execucoes {
        let Some(novo) = candidato
            .execucoes
            .iter()
            .find(|e| e.lote.id == execucao.lote.id)
        else {
            return Err(ErroArquivo::HistoriaTruncada {
                o_que: format!("execucao {} ausente do candidato", execucao.lote.id),
            });
        };
        if texto_record(&dto_de_execucao(novo))? != texto_record(&dto_de_execucao(execucao))? {
            return Err(ErroArquivo::HistoriaTruncada {
                o_que: format!("execucao {} alterada no candidato", execucao.lote.id),
            });
        }
    }

    for snapshot in &corrente.snapshots {
        let Some(novo) = candidato
            .snapshots
            .iter()
            .find(|s| s.lote == snapshot.lote && s.chave_origem == snapshot.chave_origem)
        else {
            return Err(ErroArquivo::HistoriaTruncada {
                o_que: format!(
                    "snapshot ({}, {}) ausente do candidato",
                    snapshot.lote, snapshot.chave_origem
                ),
            });
        };
        if texto_record(&dto_de_snapshot(novo))? != texto_record(&dto_de_snapshot(snapshot))? {
            return Err(ErroArquivo::HistoriaTruncada {
                o_que: format!(
                    "snapshot ({}, {}) alterado no candidato",
                    snapshot.lote, snapshot.chave_origem
                ),
            });
        }
    }

    let contar = |arquivo: &Arquivo| -> Result<BTreeMap<String, usize>, ErroArquivo> {
        let mut contagem: BTreeMap<String, usize> = BTreeMap::new();
        for evento in &arquivo.eventos {
            let texto = texto_record(&dto_de_evento(evento))?;
            *contagem.entry(texto).or_insert(0) += 1;
        }
        Ok(contagem)
    };
    let antes = contar(corrente)?;
    let depois = contar(candidato)?;
    for (texto, quantas) in antes {
        let agora = depois.get(&texto).copied().unwrap_or(0);
        if agora < quantas {
            return Err(ErroArquivo::HistoriaTruncada {
                o_que: format!(
                    "declaracao presente {quantas} vez(es) no corrente e {agora} no candidato: \
                     {}",
                    resumo(&texto)
                ),
            });
        }
    }
    Ok(())
}

/// Abbrevia un testo per la diagnostica, **su un confine di carattere**.
///
/// Riscontro R3C3B1-04 del 2026-10-22: il limite e' un numero di byte e puo'
/// cadere dentro un carattere UTF-8. I record portano stringhe dichiarate, che
/// possono essere Unicode qualunque: tagliare per indice faceva andare in
/// panico la funzione che doveva **riportare** l'errore. Qui si prende il piu'
/// grande confine non oltre il limite.
fn resumo(texto: &str) -> String {
    const LIMITE: usize = 120;
    if texto.len() <= LIMITE {
        return texto.to_string();
    }
    let corte = texto
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= LIMITE)
        .last()
        .unwrap_or(0);
    format!("{}…", &texto[..corte])
}

/// SHA-256 esadecimale dei byte, per il token di base dello strato IO.
///
/// Lavora sui byte perche' l'impronta deve esistere anche per un file che non
/// si riesce a interpretare: e' cio' che rende confrontabile uno stato
/// corrotto (riscontro R3C3B1-02).
pub(crate) fn sha256_dos_bytes(bytes: &[u8]) -> String {
    impressao::sha256_hex(bytes)
}

/// Etichette degli enum usate nell'archivio, per il test che le fissa.
#[cfg(test)]
pub(crate) fn rotulo_de<T: Serialize>(valor: &T) -> String {
    serde_json::to_string(valor).unwrap()
}

/// Costruisce un `Decimal` da testo con la regola esatta di PER-20.
///
/// Esposta perche' lo strato IO e i test ne hanno bisogno senza duplicare la
/// regola.
pub fn decimal_exato(texto: &str) -> Result<Decimal, ErroArquivo> {
    TextoDecimal(texto.to_string()).exato("decimal")
}

/// Usata dai test del determinismo: il testo di un singolo record.
#[cfg(test)]
pub(crate) fn texto_do_record(evento: &EventoArquivado) -> String {
    texto_record(&dto_de_evento(evento)).unwrap()
}

impl FromStr for IdentidadePlano {
    type Err = ErroArquivo;

    /// Comodita' per i test e per i chiamanti che dichiarano solo la versione.
    fn from_str(versao: &str) -> Result<Self, Self::Err> {
        Ok(IdentidadePlano {
            versao: versao.to_string(),
            sha256_arquivo: None,
            caminho_declarado: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Costruttori sintetici condivisi
// ---------------------------------------------------------------------------

/// Costruttori usati dai test di questo modulo e da quelli di
/// [`crate::arquivo_io`]. Dati dichiarati nel codice: nessuna fixture esterna,
/// nessun archivio reale.
#[cfg(test)]
pub(crate) mod testes_comuns {
    use super::*;
    use crate::alocacao::EsitoComponenti;
    use chrono::TimeZone;
    use std::path::Path;

    pub(crate) const CHAVE: &str = "29260512345678000195550010001256612345678901";
    pub(crate) const CHAVE2: &str = "29260512345678000195550010001256712345678901";
    pub(crate) const CNPJ: &str = "55746722000181";

    pub(crate) fn plano() -> PlanoContas {
        PlanoContas::carregar(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("config/plano_contas_gerencial.yaml"),
        )
        .unwrap()
    }

    /// Costruisce un decimale **senza** arrotondamenti: se il letterale di un
    /// test non e' rappresentabile, il test fallisce qui e non piu' tardi.
    pub(crate) fn dec(bruto: &str) -> Decimal {
        Decimal::from_str_exact(bruto).unwrap()
    }

    pub(crate) fn data(dia: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, dia).unwrap()
    }

    pub(crate) fn instante(hora: u32, offset_segundos: i32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(offset_segundos)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 21, hora, 0, 0)
            .unwrap()
    }

    pub(crate) fn aviso(codigo: &str, detalhe: &str) -> Aviso {
        Aviso {
            codigo: codigo.to_string(),
            detalhe: detalhe.to_string(),
        }
    }

    fn nao_disponivel<T>(fonte: &str, motivo: &str) -> Dado<T> {
        Dado::ausente(fonte, StatusDado::NaoDisponivel, motivo).unwrap()
    }

    pub(crate) fn lancamento(chave: &str, conta_id: &str, valor: &str) -> Lancamento {
        Lancamento {
            data_documento: data(5),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: CNPJ.to_string(),
            nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
            fornecedor_ou_favorecido: "DISTRIBUIDORA OESA LTDA".to_string(),
            // Zeri iniziali: un CPF non e' un numero.
            cnpj_cpf_favorecido: "00012345678901".to_string(),
            documento: "12566".to_string(),
            tipo_documento: "NFE".to_string(),
            conta_id: conta_id.to_string(),
            conta_nome: "Refrigerantes".to_string(),
            grupo_conta: "Custos de Mercadorias e Insumos".to_string(),
            // Testo ostile: formula di foglio, virgolette, NFC.
            descricao_itens: format!("=1+1 SUCO \"MARCA\" A\u{00E7}ucar ({valor})"),
            valor: dec(valor),
            duplicatas: vec![Duplicata {
                numero: "001".to_string(),
                vencimento: Some(data(30)),
                valor: dec(valor),
            }],
            origem: OrigemLancamento::XmlNfe,
            arquivo_origem: "samples/nfe.xml".to_string(),
            chave_origem: chave.to_string(),
            hash_registro: "mesmo-payload".to_string(),
            status_revisao: StatusRevisao::Ok,
            alertas: String::new(),
            observacoes: String::new(),
            centro_custo: String::new(),
            tipo_lancamento: TipoConta::Custo,
            dre_linha_id: String::new(),
            dre_linha_nome: String::new(),
            data_pagamento: nao_disponivel(
                "nfe:cobr/dup/dVenc",
                "a NF-e declara vencimentos, nao pagamentos",
            ),
        }
    }

    pub(crate) fn reconciliacao(total: &str) -> Reconciliacao {
        Reconciliacao {
            valor_total_nfe: dec(total),
            soma_lancamentos: dec(total),
            esito_total: EsitoTotal::Ok,
            soma_rate_documento: Dado::presente(dec(total), "nfe:cobr/dup/vDup").unwrap(),
            soma_rate_allocate: Dado::presente(dec(total), "nfe:cobr/dup/vDup").unwrap(),
            esito_rate: EsitoRate::Ok,
            articoli_allocati: 1,
            articoli_attesi: nao_disponivel("nfe:det", "a fase 2 nao expoe o numero de <det>"),
            articoli_scartati: vec![aviso("DET_SEM_PROD", "item 2 ignorado")],
            esito_copertura: EsitoCopertura::Parcial,
            componenti: vec![
                Componente {
                    nome: "vProd".to_string(),
                    valor: Dado::presente(dec(total), "nfe:total/ICMSTot/vProd").unwrap(),
                    ruolo: RuoloComponente::Base,
                    usada_na_verificacao: true,
                },
                Componente {
                    nome: "quantidade_media".to_string(),
                    // Quattro decimali: nessuna scala 2 imposta ai non monetari.
                    valor: Dado::presente(dec("1.2345"), "sintetico:quantidade").unwrap(),
                    ruolo: RuoloComponente::Informativa,
                    usada_na_verificacao: false,
                },
            ],
            esito_componenti: EsitoComponenti::Ok,
            campi_rinviati: vec![CampoRinviato {
                campo: "centro_custo".to_string(),
                status: "NAO_DISPONIVEL".to_string(),
                fonte: "fase 3B".to_string(),
                motivo: "campo nao populado neste bloco".to_string(),
            }],
            avisos_documento: vec![aviso(
                "AGREGADO_PARCIAL",
                "total/ICMSTot: tributos ausentes",
            )],
            stato_documento: StatoDocumento::Parcial,
        }
    }

    pub(crate) fn contexto(
        chave: &str,
        lote: &str,
        lancamentos: Vec<Lancamento>,
    ) -> ContextoDocumento {
        let total = if lancamentos.is_empty() {
            "0.00"
        } else {
            "10.00"
        };
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
                reconciliacao: reconciliacao(total),
            },
            // Lasciato in NFD di proposito: il record non normalizza.
            avisos_leitura: vec![aviso("SEM_PRODUTOS", "Ac\u{0327}ucar em NFD")],
            lote: IdentidadeLote {
                id: lote.to_string(),
                gerado_em: data(31),
                descricao: "fechamento de maio".to_string(),
            },
        }
    }

    pub(crate) fn um_documento() -> ContextoDocumento {
        contexto(
            CHAVE,
            "lote-2026-05-A",
            vec![lancamento(CHAVE, "2.01.004", "10.00")],
        )
    }

    /// Documento senza righe: non e' un documento vuoto, e' un documento con
    /// zero allocazioni e una riconciliazione che lo spiega.
    pub(crate) fn sem_linhas() -> ContextoDocumento {
        contexto(CHAVE2, "lote-2026-05-A", Vec::new())
    }

    pub(crate) fn identidade_plano() -> IdentidadePlano {
        IdentidadePlano {
            versao: "1.0".to_string(),
            sha256_arquivo: Some("c".repeat(64)),
            caminho_declarado: Some("config/plano_contas_gerencial.yaml".to_string()),
        }
    }

    pub(crate) fn evento(
        contexto: &ContextoDocumento,
        conta_id: &str,
        ordinal: u32,
        operacao: OperacaoRevisao,
        hora: u32,
        offset: i32,
    ) -> EventoRevisao {
        EventoRevisao {
            alvo: ChaveRevisao {
                chave_origem: contexto.chave_origem.clone(),
                impressao_conteudo: Dado::presente(
                    impressao::impressao_conteudo(contexto),
                    "revisao:conteudo",
                )
                .unwrap(),
                origem: OrigemLancamento::XmlNfe,
                conta_id: conta_id.to_string(),
                ordinal,
            },
            operacao,
            autor: "Jos\u{00E9}".to_string(),
            instante: instante(hora, offset),
            motivo: "visto e aceito".to_string(),
            nota: match operacao {
                // `Anotar` esige la nota nella fase 3C.3.a.
                OperacaoRevisao::Anotar => Some("nota do revisor".to_string()),
                _ => None,
            },
            sequencia: None,
            vista: Dado::presente("v".repeat(64), "revisao:vista").unwrap(),
        }
    }

    /// Archivio con un'esecuzione di due documenti e due dichiarazioni.
    pub(crate) fn arquivo_valido() -> Arquivo {
        let plano = plano();
        let mut arquivo = Arquivo::novo();
        let documentos = vec![um_documento(), sem_linhas()];
        arquivo
            .acrescentar_lote(&documentos, &plano, identidade_plano())
            .unwrap();
        let eventos = vec![
            evento(
                &documentos[0],
                "2.01.004",
                0,
                OperacaoRevisao::Aprovar,
                12,
                -3 * 3600,
            ),
            evento(
                &documentos[0],
                "2.01.004",
                0,
                OperacaoRevisao::Revogar,
                15,
                -3 * 3600,
            ),
        ];
        arquivo.acrescentar_eventos(&eventos);
        arquivo
    }
}

#[cfg(test)]
mod tests {
    use super::testes_comuns::*;
    use super::*;
    use crate::revisao::texto_exato_evento;
    use serde_json::{Value, json};

    fn ida_volta(arquivo: &Arquivo) -> Arquivo {
        let texto = serializar(arquivo, 7).unwrap();
        let (relido, revisao) = desserializar(&texto).unwrap();
        assert_eq!(revisao, 7, "la revisione e' un metadato del file");
        relido
    }

    /// Serializza, altera l'albero JSON, riscrive il testo. Usata per i casi
    /// malformati: altera un campo preciso senza dipendere dal rientro.
    fn alterado(arquivo: &Arquivo, mudar: impl FnOnce(&mut Value)) -> String {
        let mut arvore: Value = serde_json::from_str(&serializar(arquivo, 1).unwrap()).unwrap();
        mudar(&mut arvore);
        serde_json::to_string_pretty(&arvore).unwrap()
    }

    fn com_reconciliacao(arquivo: &Arquivo, mudar: impl FnOnce(&mut Value)) -> String {
        alterado(arquivo, |v| {
            mudar(&mut v["snapshots"][0]["contexto"]["alocacao"]["reconciliacao"])
        })
    }

    fn com_valor_da_linha(arquivo: &Arquivo, valor: Value) -> String {
        alterado(arquivo, |v| {
            v["snapshots"][0]["contexto"]["alocacao"]["lancamentos"][0]["valor"] = valor;
        })
    }

    // =================================================================
    // TP-02, TP-21: conservazione integrale
    // =================================================================

    #[test]
    fn tp21_paridade_dto_modelo_no_round_trip() {
        let arquivo = arquivo_valido();
        let relido = ida_volta(&arquivo);
        assert_eq!(
            relido, arquivo,
            "un campo che il DTO non conoscesse si perderebbe qui"
        );
    }

    #[test]
    fn tp02_documento_sem_linhas_conserva_identidade_e_reconciliacao() {
        let relido = ida_volta(&arquivo_valido());
        let sem = relido
            .snapshots
            .iter()
            .find(|s| s.chave_origem == CHAVE2)
            .expect("il documento senza righe compare nell'archivio");
        assert!(sem.contexto.alocacao.lancamentos.is_empty());
        assert_eq!(
            sem.contexto.alocacao.reconciliacao,
            sem_linhas().alocacao.reconciliacao,
            "la riconciliazione del documento senza righe resta intera"
        );
        assert_eq!(sem.contexto.avisos_leitura.len(), 1);
        verificar_estrutura(&relido).unwrap();
    }

    #[test]
    fn tp02_execucao_conserva_o_proprio_plano() {
        let relido = ida_volta(&arquivo_valido());
        assert_eq!(relido.execucoes.len(), 1);
        assert_eq!(relido.execucoes[0].plano, identidade_plano());
        assert_eq!(relido.execucoes[0].documentos.len(), 2);
    }

    // =================================================================
    // TP-03, TP-04: assenze e decimali
    // =================================================================

    #[test]
    fn tp03_ausencia_zero_e_vazio_ficam_distintos() {
        let arquivo = arquivo_valido();
        let texto = serializar(&arquivo, 1).unwrap();
        // Un'assenza si scrive `null` esplicito con il suo stato e motivo:
        // non diventa zero e non sparisce.
        assert!(texto.contains("\"valor\": null"), "{texto}");
        assert!(texto.contains("\"status\": \"NAO_DISPONIVEL\""));
        assert!(texto.contains("\"motivo\": \"a fase 2 nao expoe o numero de <det>\""));

        let relido = ida_volta(&arquivo);
        let rec = &relido.snapshots[0].contexto.alocacao.reconciliacao;
        assert!(rec.articoli_attesi.valor().is_none());
        assert_eq!(rec.articoli_attesi.status(), StatusDado::NaoDisponivel);
        assert_eq!(
            rec.articoli_attesi.motivo(),
            Some("a fase 2 nao expoe o numero de <det>")
        );
        assert_eq!(rec.articoli_allocati, 1, "un conteggio presente resta 1");
        assert!(rec.soma_rate_documento.valor().is_some());
        // Lista vuota e lista assente non si confondono: la prima e' un dato.
        assert_eq!(relido.snapshots[0].contexto.alocacao.lancamentos.len(), 1);
    }

    #[test]
    fn tp03_dado_incoerente_e_rejeitado_pelo_validador_do_modelo() {
        // Stato `OK` senza valore: `Dado::novo` lo rifiuta, e il codec non
        // inventa un valore per farlo passare.
        let caso = com_reconciliacao(&arquivo_valido(), |rec| {
            rec["articoli_attesi"] = json!({
                "valor": Value::Null,
                "fonte": "nfe:det",
                "status": "OK"
            });
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::Modelo { caminho, detalhe } => {
                assert!(
                    caminho.ends_with("reconciliacao.articoli_attesi"),
                    "{caminho}"
                );
                assert!(!detalhe.is_empty());
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn tp04_decimais_validos_conservam_valor_e_escala() {
        for bruto in [
            "0",
            "0.00",
            "0.01",
            "-7.50",
            "10.0000",
            "1.2345",
            "12345678901234.56",
            "79228162514264337593543950335",
            "-79228162514264337593543950335",
        ] {
            let mut documento = um_documento();
            documento.alocacao.lancamentos[0].valor = dec(bruto);
            documento.alocacao.lancamentos[0].duplicatas[0].valor = dec(bruto);
            documento.alocacao.reconciliacao.valor_total_nfe = dec(bruto);
            let mut arquivo = Arquivo::novo();
            arquivo
                .acrescentar_lote(&[documento], &plano(), identidade_plano())
                .unwrap();
            let relido = ida_volta(&arquivo);
            let linha = &relido.snapshots[0].contexto.alocacao.lancamentos[0];
            assert_eq!(linha.valor, dec(bruto), "valor {bruto}");
            assert_eq!(linha.valor.to_string(), bruto, "escala de {bruto}");
            assert_eq!(linha.duplicatas[0].valor.to_string(), bruto);
            assert_eq!(
                relido.snapshots[0]
                    .contexto
                    .alocacao
                    .reconciliacao
                    .valor_total_nfe
                    .to_string(),
                bruto
            );
        }
    }

    #[test]
    fn tp04_numero_json_no_lugar_de_texto_e_rejeitado() {
        let arquivo = arquivo_valido();
        for nao_texto in [json!(10), json!(10.0), json!(true), Value::Null] {
            let caso = com_valor_da_linha(&arquivo, nao_texto.clone());
            match desserializar(&caso).unwrap_err() {
                ErroArquivo::Json { classe, .. } => assert_eq!(
                    classe,
                    ClasseJson::MonetarioNaoTextual,
                    "valor {nao_texto} classificado"
                ),
                outro => panic!("{nao_texto}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tp04_decimais_hostis_sao_rejeitados_com_motivo() {
        let arquivo = arquivo_valido();
        // Motivi **misurati**, non supposti. I primi cadono sulla conversione;
        // gli ultimi tre la superano e cadono sul confronto del testo: e' la
        // prova che PER-20 ha bisogno di entrambi i controlli.
        for (bruto, motivo_esperado) in [
            (
                "0.00000000000000000000000000001",
                "nao representavel sem perda",
            ),
            (
                "35.000000000000000000000000000001",
                "nao representavel sem perda",
            ),
            ("3.5e1", "nao representavel sem perda"),
            ("35.00 ", "nao representavel sem perda"),
            (" 35.00", "nao representavel sem perda"),
            ("35,00", "nao representavel sem perda"),
            ("", "nao representavel sem perda"),
            ("+35.00", "forma nao canonica"),
            ("35.", "forma nao canonica"),
            (".5", "forma nao canonica"),
            ("-0.00", "forma nao canonica"),
        ] {
            let caso = com_valor_da_linha(&arquivo, json!(bruto));
            match desserializar(&caso).unwrap_err() {
                ErroArquivo::DecimalNaoExato {
                    caminho,
                    texto,
                    motivo,
                } => {
                    assert_eq!(texto, bruto);
                    assert_eq!(motivo, motivo_esperado, "motivo de {bruto:?}");
                    assert!(
                        caminho.ends_with("lancamentos[0].valor"),
                        "caminho {caminho}"
                    );
                }
                outro => panic!("{bruto:?}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tp04_decimal_dentro_de_dado_e_de_duplicata_tambem_e_exato() {
        let arquivo = arquivo_valido();
        let perdido = "1.23450000000000000000000000000001";

        // Dentro di un `Dado` monetario.
        let caso = com_reconciliacao(&arquivo, |rec| {
            rec["componenti"][1]["valor"]["valor"] = json!(perdido);
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::DecimalNaoExato { caminho, .. } => {
                assert!(caminho.ends_with("componenti[1].valor"), "{caminho}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Dentro una rata.
        let caso = alterado(&arquivo, |v| {
            v["snapshots"][0]["contexto"]["alocacao"]["lancamentos"][0]["duplicatas"][0]["valor"] =
                json!(perdido);
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::DecimalNaoExato { caminho, .. } => {
                assert!(caminho.ends_with("duplicatas[0].valor"), "{caminho}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Un numero JSON dentro un `Dado` e' lo stesso rifiuto.
        let caso = com_reconciliacao(&arquivo, |rec| {
            rec["soma_rate_documento"]["valor"] = json!(10);
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::Json { classe, .. } => {
                assert_eq!(classe, ClasseJson::MonetarioNaoTextual)
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    // =================================================================
    // TP-05: testi ostili, Unicode, zeri iniziali
    // =================================================================

    #[test]
    fn tp05_textos_hostis_e_unicode_sobrevivem_sem_normalizacao() {
        let relido = ida_volta(&arquivo_valido());
        let linha = &relido.snapshots[0].contexto.alocacao.lancamentos[0];
        assert!(linha.descricao_itens.starts_with("=1+1 SUCO \"MARCA\""));
        assert!(linha.descricao_itens.contains('\u{00E7}'), "NFC conservato");
        assert_eq!(linha.cnpj_cpf_favorecido, "00012345678901", "zeri iniziali");
        assert_eq!(linha.hash_registro, "mesmo-payload");
        // L'avviso era in NFD: il record **non** normalizza i testi.
        let aviso = &relido.snapshots[0].contexto.avisos_leitura[0];
        assert!(
            aviso.detalhe.contains('\u{0327}'),
            "NFD conservato: {aviso:?}"
        );
        assert_ne!(aviso.detalhe, "A\u{00E7}ucar em NFD");
    }

    // =================================================================
    // TP-06: tempo
    // =================================================================

    #[test]
    fn tp06_offset_com_segundos_e_nanossegundos_sobrevivem() {
        let documento = um_documento();
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(
                std::slice::from_ref(&documento),
                &plano(),
                identidade_plano(),
            )
            .unwrap();
        let mut um = evento(&documento, "2.01.004", 0, OperacaoRevisao::Aprovar, 12, 1);
        um.instante += chrono::Duration::nanoseconds(1);
        let mut dois = evento(&documento, "2.01.004", 0, OperacaoRevisao::Aprovar, 12, 2);
        dois.motivo = "outro motivo".to_string();
        arquivo.acrescentar_eventos(&[um.clone(), dois.clone()]);

        let relido = ida_volta(&arquivo);
        assert_eq!(relido.eventos.len(), 2);
        let por_offset = |segundos: i32| -> &EventoRevisao {
            &relido
                .eventos
                .iter()
                .find(|e| e.evento.instante.offset().local_minus_utc() == segundos)
                .unwrap_or_else(|| panic!("nenhum evento com offset {segundos}"))
                .evento
        };
        let relido_um = por_offset(1);
        let relido_dois = por_offset(2);
        assert_eq!(relido_um.instante, um.instante, "offset de 1 segundo");
        assert_eq!(relido_dois.instante, dois.instante, "offset de 2 segundos");
        assert_eq!(
            relido_um.instante.timestamp_subsec_nanos(),
            1,
            "il nanosecondo sopravvive"
        );
        // Il testo esatto e' identico prima e dopo la lettura.
        assert_eq!(texto_exato_evento(relido_um), texto_exato_evento(&um));
        assert_eq!(texto_exato_evento(relido_dois), texto_exato_evento(&dois));
        assert_ne!(texto_exato_evento(&um), texto_exato_evento(&dois));
        verificar_estrutura(&relido).unwrap();
    }

    #[test]
    fn tp06_mesmo_instante_absoluto_com_offsets_diferentes_fica_distinto() {
        let documento = um_documento();
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(
                std::slice::from_ref(&documento),
                &plano(),
                identidade_plano(),
            )
            .unwrap();
        // 12:00-03:00 e 15:00+00:00 sono lo stesso momento, in due fusi dichiarati.
        let a = evento(
            &documento,
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        let b = evento(&documento, "2.01.004", 0, OperacaoRevisao::Aprovar, 15, 0);
        assert_eq!(a.instante, b.instante, "stesso istante assoluto");
        arquivo.acrescentar_eventos(&[a, b]);
        let relido = ida_volta(&arquivo);
        assert_eq!(relido.eventos.len(), 2, "il fuso dichiarato distingue");
        let offsets: Vec<i32> = relido
            .eventos
            .iter()
            .map(|e| e.evento.instante.offset().local_minus_utc())
            .collect();
        assert!(
            offsets.contains(&(-3 * 3600)) && offsets.contains(&0),
            "{offsets:?}"
        );
    }

    #[test]
    fn tp06_offset_impossivel_e_rejeitado_na_leitura() {
        let caso = alterado(&arquivo_valido(), |v| {
            v["eventos"][0]["instante"]["offset_segundos"] = json!(100_000);
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::Modelo { caminho, detalhe } => {
                assert!(caminho.ends_with("instante"), "{caminho}");
                assert!(detalhe.contains("offset"), "{detalhe}");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    // =================================================================
    // TP-07, TP-09, TP-10: storia
    // =================================================================

    #[test]
    fn tp07_duas_linhas_com_o_mesmo_payload_sobrevivem() {
        let primeira = lancamento(CHAVE, "2.01.004", "10.00");
        let segunda = primeira.clone();
        assert_eq!(primeira, segunda, "payload identico, hash C23 compreso");
        let documento = contexto(CHAVE, "lote-2026-05-A", vec![primeira, segunda]);
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(&[documento], &plano(), identidade_plano())
            .unwrap();
        let relido = ida_volta(&arquivo);
        assert_eq!(
            relido.snapshots[0].contexto.alocacao.lancamentos.len(),
            2,
            "nessuno scarto per payload uguale"
        );
    }

    #[test]
    fn tp09_mesmo_conteudo_em_outro_lote_da_dois_snapshots() {
        let plano = plano();
        let primeiro = um_documento();
        let mut segundo = primeiro.clone();
        segundo.lote.id = "lote-2026-06-B".to_string();
        assert_eq!(
            impressao::impressao_conteudo(&primeiro),
            impressao::impressao_conteudo(&segundo),
            "il lotto non entra nell'impronta di contenuto"
        );
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&primeiro), &plano, identidade_plano())
            .unwrap();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&segundo), &plano, identidade_plano())
            .unwrap();
        let relido = ida_volta(&arquivo);
        assert_eq!(
            relido.snapshots.len(),
            2,
            "nessuna deduplicazione per impronta uguale"
        );
        assert_eq!(relido.execucoes.len(), 2);
        assert_eq!(
            relido.snapshots[0].impressao_conteudo,
            relido.snapshots[1].impressao_conteudo
        );
        verificar_estrutura(&relido).unwrap();
    }

    #[test]
    fn tp09_conteudo_corrigido_a_total_invariado_e_outro_snapshot() {
        let plano = plano();
        let original = um_documento();
        let mut corrigido = original.clone();
        corrigido.lote.id = "lote-2026-06-B".to_string();
        corrigido.alocacao.lancamentos[0].descricao_itens =
            "descricao corrigida a mao (10.00)".to_string();
        assert_eq!(
            corrigido.alocacao.reconciliacao.valor_total_nfe,
            original.alocacao.reconciliacao.valor_total_nfe,
            "il totale non cambia, quindi C23 non basterebbe"
        );
        assert_ne!(
            impressao::impressao_conteudo(&original),
            impressao::impressao_conteudo(&corrigido),
            "la correzione e' visibile nell'impronta"
        );
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&original), &plano, identidade_plano())
            .unwrap();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&corrigido), &plano, identidade_plano())
            .unwrap();
        let relido = ida_volta(&arquivo);
        assert_eq!(relido.snapshots.len(), 2);
        assert_ne!(
            relido.snapshots[0].impressao_conteudo,
            relido.snapshots[1].impressao_conteudo
        );
    }

    #[test]
    fn tp10_duas_declaracoes_identicas_nao_se_reduzem_a_uma() {
        let documento = um_documento();
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(
                std::slice::from_ref(&documento),
                &plano(),
                identidade_plano(),
            )
            .unwrap();
        let repetido = evento(
            &documento,
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        arquivo.acrescentar_eventos(&[repetido.clone(), repetido]);
        let relido = ida_volta(&arquivo);
        assert_eq!(
            relido.eventos.len(),
            2,
            "il disco conserva le occorrenze; la vista le dichiara Repeticao"
        );
    }

    // =================================================================
    // TP-11, TP-12: file malformati e alterati
    // =================================================================

    #[test]
    fn tp11_formato_schema_e_gramatica_tem_erro_proprio() {
        let arquivo = arquivo_valido();

        let caso = alterado(&arquivo, |v| v["versao_schema"] = json!(2));
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::SchemaMaisNovo {
                encontrado,
                suportado,
            } => {
                assert_eq!((encontrado, suportado), (2, VERSAO_SCHEMA));
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        let caso = alterado(&arquivo, |v| v["formato"] = json!("OUTRO-FORMATO/v1"));
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::FormatoDesconhecido { encontrado } => {
                assert_eq!(encontrado, "OUTRO-FORMATO/v1")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        for campo in [
            "impressao_conteudo",
            "impressao_apresentacao",
            "evento_exato",
        ] {
            let caso = alterado(&arquivo, |v| {
                v["gramaticas"][campo] = json!("GRAMATICA/v99")
            });
            match desserializar(&caso).unwrap_err() {
                ErroArquivo::GramaticaDesconhecida {
                    campo: nome,
                    encontrada,
                    ..
                } => {
                    assert_eq!(nome, campo);
                    assert_eq!(encontrada, "GRAMATICA/v99");
                }
                outro => panic!("{campo}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tp11_campo_desconhecido_aninhado_e_rejeitado() {
        let arquivo = arquivo_valido();
        // In tre profondita' diverse: radice, snapshot, riconciliazione.
        let casos = [
            alterado(&arquivo, |v| {
                v.as_object_mut().unwrap().insert("extra".into(), json!(1));
            }),
            alterado(&arquivo, |v| {
                v["snapshots"][0]
                    .as_object_mut()
                    .unwrap()
                    .insert("extra".into(), json!(1));
            }),
            com_reconciliacao(&arquivo, |rec| {
                rec.as_object_mut()
                    .unwrap()
                    .insert("campo_novo".into(), json!(42));
            }),
        ];
        for (i, caso) in casos.iter().enumerate() {
            match desserializar(caso).unwrap_err() {
                ErroArquivo::Json { classe, .. } => {
                    assert_eq!(classe, ClasseJson::CampoDesconhecido, "caso {i}")
                }
                outro => panic!("caso {i}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tp11_chave_duplicada_aninhada_e_rejeitada() {
        // Qui serve il testo: un albero `Value` collasserebbe la duplicazione
        // prima di qualunque controllo (riscontro R3C3B-01).
        let texto = serializar(&arquivo_valido(), 1).unwrap();
        for (alvo, duplicado) in [
            (
                "\"stato_documento\": \"PARCIAL\"",
                "\"stato_documento\": \"PARCIAL\", \"stato_documento\": \"OK\"",
            ),
            (
                "\"versao_schema\": 1",
                "\"versao_schema\": 1, \"versao_schema\": 1",
            ),
        ] {
            assert!(texto.contains(alvo), "ancora ausente: {alvo}");
            let caso = texto.replacen(alvo, duplicado, 1);
            match desserializar(&caso).unwrap_err() {
                ErroArquivo::Json { classe, .. } => {
                    assert_eq!(classe, ClasseJson::ChaveDuplicada, "alvo {alvo}")
                }
                outro => panic!("{alvo}: erro inesperado {outro}"),
            }
        }
    }

    #[test]
    fn tp11_enum_fora_do_dominio_e_campo_obrigatorio_ausente() {
        let arquivo = arquivo_valido();

        let caso = com_reconciliacao(&arquivo, |rec| rec["esito_total"] = json!("TALVEZ"));
        assert!(matches!(
            desserializar(&caso).unwrap_err(),
            ErroArquivo::Json { .. }
        ));

        let caso = alterado(&arquivo, |v| {
            v["eventos"][0]["operacao"] = json!("APAGAR");
        });
        assert!(matches!(
            desserializar(&caso).unwrap_err(),
            ErroArquivo::Json { .. }
        ));

        let caso = com_reconciliacao(&arquivo, |rec| {
            rec.as_object_mut().unwrap().remove("esito_total");
        });
        match desserializar(&caso).unwrap_err() {
            ErroArquivo::Json { detalhe, .. } => {
                assert!(detalhe.contains("missing field"), "{detalhe}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn tp11_json_truncado_e_rejeitado() {
        let texto = serializar(&arquivo_valido(), 1).unwrap();
        let metade: String = texto.chars().take(texto.chars().count() / 2).collect();
        match desserializar(&metade).unwrap_err() {
            ErroArquivo::Json { classe, .. } => assert_eq!(classe, ClasseJson::Truncado),
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn tp11_referencia_rompida_e_nomeada() {
        let mut arquivo = arquivo_valido();
        arquivo.execucoes[0]
            .documentos
            .push("CHAVE-INEXISTENTE".to_string());
        match verificar_estrutura(&arquivo).unwrap_err() {
            ErroArquivo::ReferenciaRompida { para, .. } => {
                assert!(para.contains("CHAVE-INEXISTENTE"), "{para}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Snapshot il cui lotto non ha esecuzione.
        let mut arquivo = arquivo_valido();
        arquivo.execucoes.clear();
        assert!(matches!(
            verificar_estrutura(&arquivo).unwrap_err(),
            ErroArquivo::ReferenciaRompida { .. }
        ));
    }

    #[test]
    fn tp12_impressao_ou_conteudo_alterados_sao_rejeitados() {
        // Impronta cambiata lasciando il contenuto.
        let mut arquivo = arquivo_valido();
        arquivo.snapshots[0].impressao_conteudo = "0".repeat(64);
        match verificar_estrutura(&arquivo).unwrap_err() {
            ErroArquivo::ImpressaoDivergente {
                o_que,
                chave_origem,
                ..
            } => {
                assert_eq!(o_que, "conteudo");
                assert_eq!(chave_origem, CHAVE);
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Contenuto cambiato lasciando l'impronta: nessun aggiornamento del
        // digest per far passare il dato alterato.
        let mut arquivo = arquivo_valido();
        arquivo.snapshots[0].contexto.alocacao.lancamentos[0].valor = dec("99.00");
        assert!(matches!(
            verificar_estrutura(&arquivo).unwrap_err(),
            ErroArquivo::ImpressaoDivergente {
                o_que: "conteudo",
                ..
            }
        ));

        // Snapshot incoerente col contesto che contiene.
        for campo in ["lote", "chave_origem"] {
            let mut arquivo = arquivo_valido();
            if campo == "lote" {
                arquivo.snapshots[0].lote = "lote-outro".to_string();
            } else {
                arquivo.snapshots[0].chave_origem = "outra-chave".to_string();
            }
            match verificar_estrutura(&arquivo).unwrap_err() {
                ErroArquivo::SnapshotIncoerente { campo: lido, .. } => {
                    assert_eq!(lido, campo)
                }
                outro => panic!("{campo}: erro inesperado {outro}"),
            }
        }

        // Digest della dichiarazione alterato.
        let mut arquivo = arquivo_valido();
        arquivo.eventos[0].impressao_evento = Some("f".repeat(64));
        match verificar_estrutura(&arquivo).unwrap_err() {
            ErroArquivo::ImpressaoDivergente { o_que, .. } => assert_eq!(o_que, "evento"),
            outro => panic!("erro inesperado: {outro}"),
        }

        // Dichiarazione senza digest: ammessa, non verificabile (§7.4).
        let mut arquivo = arquivo_valido();
        arquivo.eventos[0].impressao_evento = None;
        verificar_estrutura(&arquivo).unwrap();
    }

    #[test]
    fn tp12_dois_conteudos_para_o_mesmo_par_sao_conflito() {
        let mut arquivo = arquivo_valido();
        let duplicado = arquivo.snapshots[0].clone();
        arquivo.snapshots.push(duplicado);
        match verificar_estrutura(&arquivo).unwrap_err() {
            ErroArquivo::SnapshotConflitante { chave_origem, .. } => {
                assert_eq!(chave_origem, CHAVE)
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    // =================================================================
    // TP-16: accrescimento (append-only logico)
    // =================================================================

    #[test]
    fn tp16_acrescimo_recusa_perdas_de_historia() {
        let corrente = arquivo_valido();

        // Candidato che omette una dichiarazione.
        let mut sem_evento = corrente.clone();
        sem_evento.eventos.pop();
        match verificar_acrescimo(&corrente, &sem_evento).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => {
                assert!(o_que.contains("declaracao"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Candidato che omette uno snapshot.
        let mut sem_snapshot = corrente.clone();
        sem_snapshot.snapshots.pop();
        match verificar_acrescimo(&corrente, &sem_snapshot).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => {
                assert!(o_que.contains("ausente"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Candidato che altera uno snapshot.
        let mut com_mao_humana = corrente.clone();
        com_mao_humana.snapshots[0].contexto.alocacao.lancamentos[0].observacoes =
            "corrigido a mao".to_string();
        match verificar_acrescimo(&corrente, &com_mao_humana).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => {
                assert!(o_que.contains("alterado"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Candidato che cambia il piano dichiarato di un'esecuzione.
        let mut outro_plano = corrente.clone();
        outro_plano.execucoes[0].plano.versao = "2.0".to_string();
        match verificar_acrescimo(&corrente, &outro_plano).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => {
                assert!(o_que.contains("alterada"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Candidato che non contiene piu' l'esecuzione.
        let mut sem_execucao = corrente.clone();
        sem_execucao.execucoes.clear();
        assert!(matches!(
            verificar_acrescimo(&corrente, &sem_execucao).unwrap_err(),
            ErroArquivo::HistoriaTruncada { .. }
        ));

        // Aggiungere e' sempre ammesso.
        let mut cresciuto = corrente.clone();
        cresciuto.acrescentar_eventos(&[evento(
            &um_documento(),
            "2.01.004",
            0,
            OperacaoRevisao::Anotar,
            18,
            -3 * 3600,
        )]);
        verificar_acrescimo(&corrente, &cresciuto).unwrap();
        // E un candidato identico non e' una perdita.
        verificar_acrescimo(&corrente, &corrente.clone()).unwrap();
    }

    #[test]
    fn tp16_acrescimo_compara_declaracoes_como_multiconjunto() {
        let documento = um_documento();
        let mut base = Arquivo::novo();
        base.acrescentar_lote(
            std::slice::from_ref(&documento),
            &plano(),
            identidade_plano(),
        )
        .unwrap();
        let repetido = evento(
            &documento,
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        base.acrescentar_eventos(&[repetido.clone(), repetido]);

        // Due occorrenze non possono diventare una: un insieme lo permetterebbe.
        let mut menos = base.clone();
        menos.eventos.pop();
        assert_eq!(menos.eventos.len(), 1);
        match verificar_acrescimo(&base, &menos).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => assert!(
                o_que.contains("2 vez(es)") && o_que.contains("1 no candidato"),
                "{o_que}"
            ),
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    // =================================================================
    // TP-17, TP-22: determinismo e ordine fisico
    // =================================================================

    #[test]
    fn tp22_duas_serializacoes_da_mesma_revisao_sao_identicas() {
        let arquivo = arquivo_valido();
        let primeira = serializar(&arquivo, 3).unwrap();
        let segunda = serializar(&arquivo, 3).unwrap();
        assert_eq!(primeira, segunda);
        assert_eq!(
            sha256_dos_bytes(primeira.as_bytes()),
            sha256_dos_bytes(segunda.as_bytes()),
            "stesso token di base"
        );
    }

    #[test]
    fn tp17_permutar_eventos_e_snapshots_nao_muda_os_bytes() {
        let arquivo = arquivo_valido();
        let mut permutado = arquivo.clone();
        permutado.eventos.reverse();
        permutado.snapshots.reverse();
        permutado.execucoes.reverse();
        assert_eq!(
            serializar(&arquivo, 3).unwrap(),
            serializar(&permutado, 3).unwrap(),
            "l'ordine fisico e' canonico, non quello di arrivo"
        );
    }

    #[test]
    fn tp17_dois_records_da_mesma_declaracao_com_digest_ausente_ou_presente() {
        // Riscontro R3C3B-04: `impressao_evento` non e' un campo della
        // dichiarazione, quindi non entra nel suo testo esatto. L'ultimo
        // livello dell'ordine deve essere il **record** serializzato.
        let documento = um_documento();
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(
                std::slice::from_ref(&documento),
                &plano(),
                identidade_plano(),
            )
            .unwrap();
        let declaracao = evento(
            &documento,
            "2.01.004",
            0,
            OperacaoRevisao::Aprovar,
            12,
            -3 * 3600,
        );
        let digest = impressao::sha256_hex(texto_exato_evento(&declaracao).as_bytes());
        let sem = EventoArquivado {
            evento: declaracao.clone(),
            impressao_evento: None,
        };
        let com = EventoArquivado {
            evento: declaracao,
            impressao_evento: Some(digest),
        };
        assert_eq!(
            texto_exato_evento(&sem.evento),
            texto_exato_evento(&com.evento),
            "la dichiarazione e' la stessa"
        );
        assert_ne!(
            texto_do_record(&sem),
            texto_do_record(&com),
            "i record archivistici sono diversi"
        );

        let mut direto = arquivo.clone();
        direto.eventos = vec![sem.clone(), com.clone()];
        let mut invertido = arquivo;
        invertido.eventos = vec![com, sem];
        let texto = serializar(&direto, 1).unwrap();
        assert_eq!(
            texto,
            serializar(&invertido, 1).unwrap(),
            "stessi byte nelle due permutazioni"
        );
        let (relido, _) = desserializar(&texto).unwrap();
        assert_eq!(relido.eventos.len(), 2, "entrambi i record conservati");
        assert!(relido.eventos.iter().any(|e| e.impressao_evento.is_none()));
        assert!(relido.eventos.iter().any(|e| e.impressao_evento.is_some()));
    }

    #[test]
    fn tp22_arrays_de_ordem_memorizada_nao_sao_reordenados() {
        let mut documento = um_documento();
        documento.alocacao.reconciliacao.componenti.reverse();
        let esperado: Vec<String> = documento
            .alocacao
            .reconciliacao
            .componenti
            .iter()
            .map(|c| c.nome.clone())
            .collect();
        assert_eq!(esperado, vec!["quantidade_media", "vProd"]);
        let mut arquivo = Arquivo::novo();
        arquivo
            .acrescentar_lote(&[documento], &plano(), identidade_plano())
            .unwrap();
        let relido = ida_volta(&arquivo);
        let nomes: Vec<String> = relido.snapshots[0]
            .contexto
            .alocacao
            .reconciliacao
            .componenti
            .iter()
            .map(|c| c.nome.clone())
            .collect();
        assert_eq!(nomes, esperado, "l'ordine memorizzato e' un dato");
    }

    // =================================================================
    // TP-18: presentazione
    // =================================================================

    #[test]
    fn tp18_apresentacao_mudada_e_divergencia_nao_erro() {
        let arquivo = arquivo_valido();
        assert!(
            verificar_apresentacao(&arquivo, &plano())
                .unwrap()
                .is_empty(),
            "col piano con cui e' stata scritta non c'e' divergenza"
        );
        let mut mudado = arquivo;
        mudado.snapshots[0].impressao_apresentacao = Some("a".repeat(64));
        let divergencias = verificar_apresentacao(&mudado, &plano()).unwrap();
        assert_eq!(divergencias.len(), 1);
        assert_eq!(divergencias[0].chave_origem, CHAVE);
        assert_eq!(divergencias[0].salva, "a".repeat(64));
        assert_ne!(divergencias[0].atual, divergencias[0].salva);
        // La struttura resta valida: la presentazione non invalida le decisioni.
        verificar_estrutura(&mudado).unwrap();
    }

    #[test]
    fn tp18_apresentacao_ausente_nao_inventa_divergencia() {
        let mut arquivo = arquivo_valido();
        arquivo.snapshots[0].impressao_apresentacao = None;
        assert!(
            verificar_apresentacao(&arquivo, &plano())
                .unwrap()
                .is_empty()
        );
    }

    // =================================================================
    // Etichette, forma del file e costruzione
    // =================================================================

    #[test]
    fn etiquetas_dos_enums_do_arquivo() {
        // L'archivio usa la rappresentazione serde dei tipi del modello: una
        // sola verita' per le etichette. Qui vengono fissate una per una.
        assert_eq!(rotulo_de(&OrigemLancamento::XmlNfe), "\"XML_NFE\"");
        assert_eq!(rotulo_de(&StatusRevisao::Ok), "\"OK\"");
        assert_eq!(rotulo_de(&StatusRevisao::Revisar), "\"REVISAR\"");
        assert_eq!(rotulo_de(&StatusRevisao::Aprovado), "\"APROVADO\"");
        assert_eq!(rotulo_de(&StatusDado::NaoDisponivel), "\"NAO_DISPONIVEL\"");
        assert_eq!(rotulo_de(&StatusDado::NaoAplicavel), "\"N.A.\"");
        assert_eq!(rotulo_de(&StatoDocumento::Parcial), "\"PARCIAL\"");
        assert_eq!(rotulo_de(&RuoloComponente::Informativa), "\"INFORMATIVA\"");
        assert_eq!(rotulo_de(&EsitoTotal::Ok), "\"OK\"");
        // Differenza tecnica da dichiarare: `TipoConta` e' `snake_case`.
        assert_eq!(rotulo_de(&TipoConta::Custo), "\"custo\"");
        assert_eq!(
            rotulo_de(&Competencia::new(2026, 5).unwrap()),
            "\"2026-05\""
        );

        // L'operazione ha un DTO locale, perche' `OperacaoRevisao` non deriva
        // `Deserialize` e quel modulo e' auditato. Le etichette devono
        // combaciare: una deriva cambierebbe il formato in silenzio.
        for (dto, operacao) in [
            (OperacaoDto::Aprovar, OperacaoRevisao::Aprovar),
            (OperacaoDto::Revogar, OperacaoRevisao::Revogar),
            (OperacaoDto::Anotar, OperacaoRevisao::Anotar),
        ] {
            assert_eq!(
                rotulo_de(&dto),
                format!("\"{}\"", operacao.rotulo()),
                "etichetta di {operacao:?}"
            );
            assert_eq!(rotulo_de(&dto), rotulo_de(&operacao), "serde concorda");
            assert_eq!(dto.para(), operacao, "il DTO torna all'operazione giusta");
            assert_eq!(OperacaoDto::de(operacao), dto);
        }
    }

    #[test]
    fn o_arquivo_e_utf8_com_lf_e_sem_bom() {
        let texto = serializar(&arquivo_valido(), 1).unwrap();
        assert!(!texto.starts_with('\u{feff}'), "nessun BOM");
        assert!(!texto.contains('\r'), "nessun CRLF");
        assert!(texto.ends_with("}\n"), "una sola LF finale");
        assert!(texto.contains(FORMATO));
        assert!(texto.contains("\"revisao_arquivo\": 1"));
    }

    #[test]
    fn decimal_exato_e_a_mesma_regra_do_codec() {
        assert_eq!(decimal_exato("10.0000").unwrap().to_string(), "10.0000");
        assert_eq!(decimal_exato("0").unwrap().to_string(), "0");
        assert!(decimal_exato("3.5e1").is_err());
        assert!(decimal_exato("+35.00").is_err());
        assert!(decimal_exato("").is_err());
    }

    #[test]
    fn lote_vazio_e_conflitos_sao_nomeados() {
        let plano = plano();
        let mut arquivo = Arquivo::novo();
        assert!(matches!(
            arquivo
                .acrescentar_lote(&[], &plano, identidade_plano())
                .unwrap_err(),
            ErroArquivo::LoteVazio
        ));

        let documento = um_documento();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&documento), &plano, identidade_plano())
            .unwrap();

        // Stesso documento nello stesso lotto: conflitto, non sovrascrittura.
        match arquivo
            .acrescentar_lote(std::slice::from_ref(&documento), &plano, identidade_plano())
            .unwrap_err()
        {
            ErroArquivo::SnapshotConflitante { chave_origem, lote } => {
                assert_eq!(chave_origem, CHAVE);
                assert_eq!(lote, "lote-2026-05-A");
            }
            outro => panic!("erro inesperado: {outro}"),
        }

        // Lotto con documenti di esecuzioni diverse.
        let mut outro = um_documento();
        outro.chave_origem = CHAVE2.to_string();
        outro.lote.id = "lote-2026-06-B".to_string();
        let mut vazio = Arquivo::novo();
        match vazio
            .acrescentar_lote(&[um_documento(), outro], &plano, identidade_plano())
            .unwrap_err()
        {
            ErroArquivo::SnapshotIncoerente { campo, .. } => assert_eq!(campo, "lote"),
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    #[test]
    fn conta_fora_do_plano_e_nomeada_sem_inventar_conta() {
        let mut documento = um_documento();
        documento.alocacao.lancamentos[0].conta_id = "9.99.999".to_string();
        let mut arquivo = Arquivo::novo();
        match arquivo
            .acrescentar_lote(&[documento], &plano(), identidade_plano())
            .unwrap_err()
        {
            ErroArquivo::ContaFora {
                conta_id,
                chave_origem,
            } => {
                assert_eq!(conta_id, "9.99.999");
                assert_eq!(chave_origem, CHAVE);
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(arquivo.snapshots.is_empty(), "nessuno snapshot parziale");
        assert!(arquivo.execucoes.is_empty(), "nessuna esecuzione parziale");
    }

    #[test]
    fn identidade_plano_minima_vem_de_from_str() {
        let identidade: IdentidadePlano = "1.0".parse().unwrap();
        assert_eq!(identidade.versao, "1.0");
        // Un'assenza resta un'assenza: nessuna stringa vuota al posto del hash.
        assert!(identidade.sha256_arquivo.is_none());
        assert!(identidade.caminho_declarado.is_none());
    }

    // =================================================================
    // Regressioni dell'audit del 2026-10-22 (R3C3B1-01, 03, 04)
    // =================================================================

    /// R3C3B1-01: `10.00` e `10.0000` sono **lo stesso numero** per
    /// `PartialEq` di `Decimal`, e l'impronta canonica normalizza la scala
    /// prima di confrontare. Nessuno dei due controlli puo' quindi accorgersi
    /// che la rappresentazione originale e' cambiata: il confronto
    /// dell'accrescimento deve guardare la **codifica esatta**.
    #[test]
    fn r1_acrescimo_recusa_mudanca_de_escala_numa_linha() {
        let corrente = arquivo_valido();
        let mut candidato = corrente.clone();
        candidato.snapshots[0].contexto.alocacao.lancamentos[0].valor = dec("10.0000");

        // Le tre premesse del riscontro, fissate per non perderle di vista.
        assert_eq!(
            corrente.snapshots[0].contexto.alocacao.lancamentos[0].valor,
            candidato.snapshots[0].contexto.alocacao.lancamentos[0].valor,
            "PartialEq di Decimal ignora la scala"
        );
        verificar_estrutura(&candidato)
            .expect("l'impronta normalizza la scala, quindi resta valida");
        assert_ne!(
            serializar(&corrente, 2).unwrap(),
            serializar(&candidato, 2).unwrap(),
            "ma i byte conservati sono diversi: la scala e' un dato"
        );

        match verificar_acrescimo(&corrente, &candidato).unwrap_err() {
            ErroArquivo::HistoriaTruncada { o_que } => {
                assert!(o_que.contains("alterado"), "{o_que}")
            }
            outro => panic!("erro inesperado: {outro}"),
        }
    }

    /// Lo stesso, ma dentro un `Dado` monetario e dentro una rata: il
    /// confronto non deve dipendere dalla profondita' del campo.
    #[test]
    fn r1_acrescimo_recusa_mudanca_de_escala_aninhada() {
        let corrente = arquivo_valido();

        let mut na_rate = corrente.clone();
        na_rate.snapshots[0]
            .contexto
            .alocacao
            .reconciliacao
            .soma_rate_documento = Dado::presente(dec("10.0000"), "nfe:cobr/dup/vDup").unwrap();
        assert!(matches!(
            verificar_acrescimo(&corrente, &na_rate).unwrap_err(),
            ErroArquivo::HistoriaTruncada { .. }
        ));

        let mut no_componente = corrente.clone();
        no_componente.snapshots[0]
            .contexto
            .alocacao
            .reconciliacao
            .componenti[0]
            .valor = Dado::presente(dec("10.000"), "nfe:total/ICMSTot/vProd").unwrap();
        assert!(matches!(
            verificar_acrescimo(&corrente, &no_componente).unwrap_err(),
            ErroArquivo::HistoriaTruncada { .. }
        ));

        let mut na_duplicata = corrente.clone();
        na_duplicata.snapshots[0].contexto.alocacao.lancamentos[0].duplicatas[0].valor =
            dec("10.0");
        assert!(matches!(
            verificar_acrescimo(&corrente, &na_duplicata).unwrap_err(),
            ErroArquivo::HistoriaTruncada { .. }
        ));

        let mut no_total = corrente.clone();
        no_total.snapshots[0]
            .contexto
            .alocacao
            .reconciliacao
            .valor_total_nfe = dec("10.000000");
        assert!(matches!(
            verificar_acrescimo(&corrente, &no_total).unwrap_err(),
            ErroArquivo::HistoriaTruncada { .. }
        ));
    }

    /// Il controllo piu' severo non deve rifiutare un candidato **davvero**
    /// identico, nemmeno dopo un giro su disco.
    #[test]
    fn r1_candidato_identico_depois_do_round_trip_continua_publicavel() {
        let corrente = arquivo_valido();
        let relido = ida_volta(&corrente);
        verificar_acrescimo(&corrente, &relido).unwrap();

        // E un accrescimento legittimo sopra il rileto resta ammesso.
        let mut cresciuto = relido;
        cresciuto.acrescentar_eventos(&[evento(
            &um_documento(),
            "2.01.004",
            0,
            OperacaoRevisao::Anotar,
            18,
            -3 * 3600,
        )]);
        verificar_acrescimo(&corrente, &cresciuto).unwrap();
    }

    /// R3C3B1-03: `acrescentar_lote` rifiuta un lotto gia' presente, ma il
    /// modello e' pubblico e il lettore non passa per quel costruttore. Il
    /// validatore condiviso deve imporre la stessa unicita'.
    #[test]
    fn r3_execucao_duplicada_e_rejeitada_pelo_validador() {
        // Duplicato identico.
        let mut arquivo = arquivo_valido();
        let copia = arquivo.execucoes[0].clone();
        arquivo.execucoes.push(copia);
        match verificar_estrutura(&arquivo).unwrap_err() {
            ErroArquivo::ExecucaoConflitante { lote } => assert_eq!(lote, "lote-2026-05-A"),
            outro => panic!("erro inesperado: {outro}"),
        }

        // Duplicato con un piano **diverso**: due identita' incompatibili per
        // lo stesso lotto. Nelle due permutazioni, perche' il difetto non si
        // risolve scegliendo il primo o l'ultimo.
        for invertido in [false, true] {
            let mut arquivo = arquivo_valido();
            let mut outra = arquivo.execucoes[0].clone();
            outra.plano.versao = "2.0".to_string();
            arquivo.execucoes.push(outra);
            if invertido {
                arquivo.execucoes.reverse();
            }
            assert!(
                matches!(
                    verificar_estrutura(&arquivo).unwrap_err(),
                    ErroArquivo::ExecucaoConflitante { .. }
                ),
                "permutazione invertida: {invertido}"
            );
            // E lo stesso rifiuto arriva leggendo da JSON, non solo dal modello.
            let texto = serializar(&arquivo, 1).unwrap();
            let (relido, _) = desserializar(&texto).expect("il codec non giudica la struttura");
            assert_eq!(relido.execucoes.len(), 2);
            assert!(matches!(
                verificar_estrutura(&relido).unwrap_err(),
                ErroArquivo::ExecucaoConflitante { .. }
            ));
        }

        // Due lotti **distinti**, anche con piani diversi, restano ammessi.
        let plano = plano();
        let mut arquivo = Arquivo::novo();
        let primeiro = um_documento();
        let mut segundo = primeiro.clone();
        segundo.lote.id = "lote-2026-06-B".to_string();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&primeiro), &plano, identidade_plano())
            .unwrap();
        let mut outro_plano = identidade_plano();
        outro_plano.versao = "2.0".to_string();
        arquivo
            .acrescentar_lote(std::slice::from_ref(&segundo), &plano, outro_plano)
            .unwrap();
        verificar_estrutura(&arquivo).unwrap();
    }

    /// R3C3B1-04: la diagnostica abbreviava il testo del record a 120 **byte**,
    /// che possono cadere dentro un carattere. La funzione che deve riportare
    /// `HistoriaTruncada` non puo' interrompere l'esecuzione.
    #[test]
    fn r4_diagnostico_de_historia_truncada_nao_corta_caractere_a_meio() {
        // Caratteri di due, tre e quattro byte, con prefissi ASCII che
        // spostano il limite dentro il carattere.
        for recheio in ['\u{00E9}', '\u{20AC}', '\u{1D11E}'] {
            for prefixo in 0..6 {
                let documento = um_documento();
                let mut base = Arquivo::novo();
                base.acrescentar_lote(
                    std::slice::from_ref(&documento),
                    &plano(),
                    identidade_plano(),
                )
                .unwrap();
                let candidato = base.clone();

                let mut declaracao = evento(
                    &documento,
                    "2.01.004",
                    0,
                    OperacaoRevisao::Aprovar,
                    12,
                    -3 * 3600,
                );
                declaracao.alvo.chave_origem =
                    format!("{}{}", "a".repeat(prefixo), recheio.to_string().repeat(60));
                base.acrescentar_eventos(&[declaracao]);

                verificar_estrutura(&base).expect("l'archivio conserva anche chiavi cosi'");
                match verificar_acrescimo(&base, &candidato).unwrap_err() {
                    ErroArquivo::HistoriaTruncada { o_que } => {
                        assert!(
                            o_que.contains("declaracao"),
                            "prefisso {prefixo}, carattere {recheio:?}: {o_que}"
                        );
                        // La diagnostica resta consultabile, non vuota.
                        assert!(o_que.len() > 60);
                    }
                    outro => panic!("erro inesperado: {outro}"),
                }
            }
        }
    }

    #[test]
    fn r4_resumo_abrevia_em_confins_de_caractere() {
        // Direttamente sulla funzione: il limite non deve mai cadere dentro
        // un carattere, qualunque sia la sua lunghezza in byte.
        for recheio in ['\u{00E9}', '\u{20AC}', '\u{1D11E}'] {
            for prefixo in 0..6 {
                let texto = format!("{}{}", "a".repeat(prefixo), recheio.to_string().repeat(60));
                let abreviado = resumo(&texto);
                assert!(
                    texto.starts_with(abreviado.trim_end_matches('\u{2026}')),
                    "l'abbreviazione e' un prefisso del testo"
                );
                assert!(abreviado.len() <= 120 + '\u{2026}'.len_utf8() + 4);
            }
        }
        // Un testo corto non viene toccato.
        assert_eq!(resumo("curto"), "curto");
        // Un testo lungo viene abbreviato e lo dichiara.
        let longo = "a".repeat(200);
        assert!(resumo(&longo).ends_with('\u{2026}'));
    }
}

#[cfg(test)]
mod regressao_nao_utf8 {
    use super::testes_comuns::*;
    use super::*;

    /// R3C3B1-02: byte non UTF-8 non sono un archivio, e il rifiuto deve
    /// essere un errore di **contenuto** nominato, non un errore di IO.
    #[test]
    fn r2_desserializar_bytes_nomeia_o_nao_utf8() {
        match desserializar_bytes(&[0xFF, 0xFE, 0x80]).unwrap_err() {
            ErroArquivo::NaoUtf8 { posicao } => assert_eq!(posicao, 0),
            outro => panic!("erro inesperado: {outro}"),
        }
        // La posizione e quella del primo byte invalido, non zero per comodita.
        let mut quase = b"{\"formato\": \"x\"}".to_vec();
        quase.push(0xFF);
        match desserializar_bytes(&quase).unwrap_err() {
            ErroArquivo::NaoUtf8 { posicao } => assert_eq!(posicao, quase.len() - 1),
            outro => panic!("erro inesperado: {outro}"),
        }
        // Un UTF-8 valido ma non JSON resta un errore di JSON.
        assert!(matches!(
            desserializar_bytes(b"nao e json").unwrap_err(),
            ErroArquivo::Json { .. }
        ));
        // E i byte di un archivio valido danno lo stesso risultato del testo.
        let texto = serializar(&arquivo_valido(), 4).unwrap();
        assert_eq!(
            desserializar_bytes(texto.as_bytes()).unwrap(),
            desserializar(&texto).unwrap()
        );
    }

    #[test]
    fn r2_impressao_dos_bytes_existe_mesmo_sem_texto() {
        let texto = serializar(&arquivo_valido(), 1).unwrap();
        assert_eq!(sha256_dos_bytes(texto.as_bytes()).len(), 64);
        // Byte illeggibili hanno comunque un impronta, e due sequenze diverse
        // ne hanno due diverse: e cio che rende confrontabile uno stato corrotto.
        let uma = sha256_dos_bytes(&[0xFF, 0xFE, 0x80]);
        let outra = sha256_dos_bytes(&[0xFF, 0xFE, 0x81]);
        assert_eq!(uma.len(), 64);
        assert_ne!(uma, outra);
    }
}

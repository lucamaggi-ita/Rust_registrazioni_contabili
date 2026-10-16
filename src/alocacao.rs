//! Allocazione di una NF-e letta in registrazioni contabili (fase 3B).
//!
//! Porting di `src/fantozzi/allocation.py` del repo Python canonico
//! (`@5b70f2b`), con le divergenze **approvate** il 2026-10-16 (decisioni
//! D1-D6 della specifica di fase 3A).
//!
//! La funzione d'ingresso [`alocar_nfe`] e' **pura**: non scrive su disco, non
//! tocca database e non deduplica niente in modo persistente. Produce le
//! registrazioni e una [`Reconciliacao`] con quattro esiti indipendenti.
//!
//! # Aritmetica
//!
//! Tutti i calcoli monetari avvengono in **centesimi interi** (`i128`), non in
//! `Decimal`. Riscontro R3B-01 dell'audit del 2026-10-16: con importi estremi
//! ma legittimi `Decimal` va in panico sul prodotto del riparto e **perde un
//! centesimo in silenzio** nella somma, `checked_add` compreso. In centesimi
//! interi la somma e il prodotto sono esatti; quando il risultato non e'
//! rappresentabile si restituisce [`ErroAlocacao::CapacidadeNumericaExcedida`],
//! che nomina operazione e valori, invece di un risultato parziale.
//!
//! Contratti applicati qui (identificativi della specifica di fase 3A):
//!
//! - **F3A-01/02/03/04**: una registrazione per (documento, conto), nell'ordine
//!   di prima comparsa del conto fra gli articoli; valore = somma dei `vProd`
//!   del gruppo; descrizione `"{desc} ({valor})"` con **due decimali sempre**,
//!   unita da `"; "`.
//! - **F3A-05/06/07**: differenza di totale su `2.05.001` o `2.05.002`, con la
//!   verifica di spiegazione; una componente non `Ok` **non** spiega niente
//!   (D2).
//! - **F3A-08/09/10**: riparto delle rate sul denominatore `vNF`, con i resti
//!   nell'ultima riga e nell'ultima colonna, e le tre invarianti.
//! - **F3A-11**: un riparto che non chiude manda a revisione tutte le
//!   registrazioni (D3).
//! - **F3A-13a/b**: non classificato va al conto di revisione; ambiguo
//!   **conserva** il conto vincente.
//! - **F3A-15/27**: hash a quattro campi come il Python; payload ripetuto nello
//!   stesso documento e' un avviso, mai uno scarto (D4).
//! - **F3A-17/18/19**: `data_pagamento`, linee DRE, centro di costo e
//!   osservazioni restano dichiarati in [`Reconciliacao::campi_rinviati`].
//! - **F3A-21**: arrotondamento `ROUND_HALF_UP` esplicito, non il Bankers di
//!   `round_dp`; in centesimi interi e' il resto della divisione a decidere.
//! - **F3A-23a/b**: chiave vuota blocca; numero e identita' vuote vanno a
//!   revisione (D5).
//! - **F3A-24**: articolo senza descrizione conservato con avviso, nessuna
//!   descrizione inventata.
//! - **F3A-26**: dominio monetario in centesimi sugli ingressi (D6).
//! - **F3A-28**: quattro esiti separati e una sola regola di aggregazione.

use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _;

use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::classificacao::{Classificador, ErroClassificacao};
use crate::modelos::{
    Competencia, Confianca, Dado, Duplicata, ErroCompetencia, ErroDado, Lancamento, Nfe,
    OrigemLancamento, ProdutoNfe, ResultadoClassificacao, StatusDado, StatusRevisao,
};
use crate::nfe::{Aviso, NfeImportada};
use crate::regras::PlanoContas;

/// Origine e tipo di documento di ogni registrazione prodotta da questo modulo.
pub const ORIGEM_XML_NFE: &str = "XML_NFE";
/// Conto della differenza positiva di totale (costi accessori).
pub const CONTA_ACESSORIOS: &str = "2.05.001";
/// Conto della differenza negativa di totale (sconti ottenuti).
pub const CONTA_DESCONTOS: &str = "2.05.002";

/// Codici di avviso prodotti da questo modulo. Stabili e citabili nei test.
pub mod avisos {
    pub const VALOR_INCONSISTENTE: &str = "VALOR_INCONSISTENTE";
    pub const TOTAL_NAO_RECONCILIADO: &str = "TOTAL_NAO_RECONCILIADO";
    pub const AGREGADO_PARCIAL: &str = "AGREGADO_PARCIAL";
    pub const NAO_CLASSIFICADO: &str = "NAO_CLASSIFICADO";
    pub const CLASSIFICACAO_AMBIGUA: &str = "CLASSIFICACAO_AMBIGUA";
    pub const ITEM_SEM_DESCRICAO: &str = "ITEM_SEM_DESCRICAO";
    pub const DOCUMENTO_SEM_NUMERO: &str = "DOCUMENTO_SEM_NUMERO";
    pub const FORNECEDOR_SEM_IDENTIDADE: &str = "FORNECEDOR_SEM_IDENTIDADE";
    pub const EMPRESA_SEM_IDENTIDADE: &str = "EMPRESA_SEM_IDENTIDADE";
    pub const DOCUMENTO_SEM_ITENS: &str = "DOCUMENTO_SEM_ITENS";
    pub const HASH_DUPLICADO_NO_DOCUMENTO: &str = "HASH_DUPLICADO_NO_DOCUMENTO";
}

/// Avvisi del parser che attestano articoli scartati in lettura.
const AVISOS_DE_SCARTO: [&str; 2] = ["DET_SEM_PROD", "SOMA_ITENS_DIVERGE"];

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore che impedisce di produrre le registrazioni di una NF-e.
#[derive(Debug)]
pub enum ErroAlocacao {
    /// D5/F3A-23a: senza chiave la riga non e' identificabile.
    IdentidadeInsuficiente { origem: String, campo: &'static str },
    /// D1/F3A-09: denominatore zero fuori dal caso banale.
    RipartoImpossivel { origem: String, motivo: String },
    /// D6/F3A-26: importo che non e' un multiplo esatto di 0.01.
    EscalaMonetariaInvalida {
        origem: String,
        campo: String,
        valor: Decimal,
    },
    /// R3B-01: l'operazione e' esatta in centesimi interi ma il risultato non
    /// e' rappresentabile, o eccede `i128`. Nessun risultato parziale, nessun
    /// arrotondamento correttivo, nessun tetto di importo inventato.
    CapacidadeNumericaExcedida {
        origem: String,
        operacao: &'static str,
        detalhe: String,
    },
    /// F3A-20: conto di aggiustamento assente dal piano dei conti.
    ContaDeAjusteAusente { origem: String, conta_id: String },
    /// La classificazione cita un conto che il piano non contiene.
    ClassificacaoFalhou {
        origem: String,
        erro: ErroClassificacao,
    },
    /// Competenza fuori dall'intervallo dichiarato (C8).
    CompetenciaInvalida {
        origem: String,
        erro: ErroCompetencia,
    },
    /// Dato incoerente costruito qui: e' un bug di questo modulo, non un
    /// difetto della NF-e, e non resta silenzioso.
    ModeloIncoerente { origem: String, erro: ErroDado },
}

impl fmt::Display for ErroAlocacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroAlocacao::IdentidadeInsuficiente { origem, campo } => write!(
                f,
                "{origem}: {campo} ausente ou vazia: sem ela o lancamento nao e' identificavel"
            ),
            ErroAlocacao::RipartoImpossivel { origem, motivo } => {
                write!(f, "{origem}: rateio das duplicatas nao definido: {motivo}")
            }
            ErroAlocacao::EscalaMonetariaInvalida {
                origem,
                campo,
                valor,
            } => write!(
                f,
                "{origem}: valor monetario fora do dominio em {campo}: {valor} \
                 nao e' multiplo exato de 0.01 (nenhum arredondamento e' aplicado)"
            ),
            ErroAlocacao::CapacidadeNumericaExcedida {
                origem,
                operacao,
                detalhe,
            } => write!(
                f,
                "{origem}: capacidade numerica excedida em {operacao}: {detalhe}. \
                 Nenhum centavo foi descartado e nenhum resultado parcial foi produzido"
            ),
            ErroAlocacao::ContaDeAjusteAusente { origem, conta_id } => write!(
                f,
                "{origem}: conta de ajuste {conta_id} ausente no plano de contas"
            ),
            ErroAlocacao::ClassificacaoFalhou { origem, erro } => {
                write!(f, "{origem}: {erro}")
            }
            ErroAlocacao::CompetenciaInvalida { origem, erro } => {
                write!(f, "{origem}: competencia invalida: {erro}")
            }
            ErroAlocacao::ModeloIncoerente { origem, erro } => {
                write!(f, "{origem}: dado incoerente montado pelo alocador: {erro}")
            }
        }
    }
}

impl std::error::Error for ErroAlocacao {}

// ---------------------------------------------------------------------------
// Riconciliazione: quattro esiti indipendenti
// ---------------------------------------------------------------------------

/// Esito aritmetico del totale: somma delle registrazioni contro `vNF`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EsitoTotal {
    Ok,
    Erro,
}

/// Esito del riparto delle rate, indipendente dal totale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EsitoRate {
    Ok,
    Erro,
    /// Il documento non porta rate: non e' un difetto.
    #[serde(rename = "N.A.")]
    NaoAplicavel,
    /// Riparto non definito (D1, caso 3): non si afferma che sia corretto.
    NaoVerificado,
}

/// Esito della copertura degli articoli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EsitoCopertura {
    Ok,
    Parcial,
}

/// Esito delle componenti usate nelle verifiche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EsitoComponenti {
    Ok,
    Parcial,
}

/// Stato aggregato del documento: `ERRO` > `PARCIAL` > `OK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatoDocumento {
    Ok,
    Parcial,
    Erro,
}

/// Ruolo di una componente nel totale della NF-e.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuoloComponente {
    /// Base imponibile dei prodotti.
    Base,
    /// Si somma al totale.
    Acessorio,
    /// Si sottrae dal totale.
    Deducao,
    /// Informativa: non entra nella differenza di totale.
    Informativa,
}

/// Componente del totale con la sua carta d'identita'.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Componente {
    pub nome: String,
    pub valor: Dado<Decimal>,
    pub ruolo: RuoloComponente,
    /// `true` se questa componente e' stata usata in una verifica di questo
    /// documento: solo allora la sua incompletezza declassa il risultato.
    pub usada_na_verificacao: bool,
}

/// Campo di [`Lancamento`] che la fase 3B non popola, reso interrogabile.
///
/// Serve al criterio "nessun vuoto senza stato": i quattro campi `String`
/// legacy restano vuoti perche' il modello non cambia in questo blocco, ma
/// l'assenza non e' nascosta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampoRinviato {
    pub campo: String,
    pub status: String,
    pub fonte: String,
    pub motivo: String,
}

/// Riconciliazione del documento: quattro esiti e una regola di aggregazione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reconciliacao {
    pub valor_total_nfe: Decimal,
    pub soma_lancamentos: Decimal,
    pub esito_total: EsitoTotal,
    pub soma_rate_documento: Dado<Decimal>,
    pub soma_rate_allocate: Dado<Decimal>,
    pub esito_rate: EsitoRate,
    pub articoli_allocati: usize,
    /// Sempre assente: la fase 2 non espone il numero di `<det>` letti.
    pub articoli_attesi: Dado<usize>,
    pub articoli_scartati: Vec<Aviso>,
    pub esito_copertura: EsitoCopertura,
    pub componenti: Vec<Componente>,
    pub esito_componenti: EsitoComponenti,
    pub campi_rinviati: Vec<CampoRinviato>,
    pub avisos_documento: Vec<Aviso>,
    pub stato_documento: StatoDocumento,
}

impl Reconciliacao {
    /// F3A-28: unica regola di aggregazione, con precedenza dichiarata.
    fn agregar(
        esito_total: EsitoTotal,
        esito_rate: EsitoRate,
        esito_copertura: EsitoCopertura,
        esito_componenti: EsitoComponenti,
    ) -> StatoDocumento {
        if esito_total == EsitoTotal::Erro || esito_rate == EsitoRate::Erro {
            return StatoDocumento::Erro;
        }
        if esito_copertura == EsitoCopertura::Parcial
            || esito_componenti == EsitoComponenti::Parcial
            || esito_rate == EsitoRate::NaoVerificado
        {
            return StatoDocumento::Parcial;
        }
        StatoDocumento::Ok
    }
}

/// Risultato dell'allocazione di una NF-e.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlocacaoNfe {
    pub lancamentos: Vec<Lancamento>,
    pub reconciliacao: Reconciliacao,
}

// ---------------------------------------------------------------------------
// Aritmetica monetaria: centesimi interi
// ---------------------------------------------------------------------------

/// F3A-21: arrotondamento monetario a 2 decimali, `ROUND_HALF_UP`.
///
/// `Decimal::round_dp` usa "Bankers Rounding": usarlo divergerebbe dal Python,
/// che quantizza con `ROUND_HALF_UP`. Resta per i valori che arrivano come
/// `Decimal` e per i confronti nei test; l'aritmetica vera sta in centesimi.
fn centavos(valor: Decimal) -> Decimal {
    valor.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// F3A-26/D6: un importo e' nel dominio se il suo **valore** e' un multiplo
/// esatto di 0.01. `10.0000` passa, `0.005` no: gli zeri finali non contano.
fn no_dominio(valor: Decimal) -> bool {
    valor.normalize().scale() <= 2
}

fn exigir_dominio(
    valor: Decimal,
    campo: impl Into<String>,
    origem: &str,
) -> Result<(), ErroAlocacao> {
    if no_dominio(valor) {
        return Ok(());
    }
    Err(ErroAlocacao::EscalaMonetariaInvalida {
        origem: origem.to_string(),
        campo: campo.into(),
        valor,
    })
}

fn capacidade(origem: &str, operacao: &'static str, detalhe: impl Into<String>) -> ErroAlocacao {
    ErroAlocacao::CapacidadeNumericaExcedida {
        origem: origem.to_string(),
        operacao,
        detalhe: detalhe.into(),
    }
}

/// Converte un importo nel suo numero **esatto** di centesimi.
///
/// Richiede che il valore sia nel dominio (D6). La conversione passa da
/// mantissa e scala, senza moltiplicare in `Decimal`: moltiplicare per 100 un
/// importo vicino al massimo rappresentabile andrebbe in overflow.
fn em_centesimos(valor: Decimal, campo: &str, origem: &str) -> Result<i128, ErroAlocacao> {
    exigir_dominio(valor, campo, origem)?;
    let normalizado = valor.normalize();
    let fator = 10i128.pow(2 - normalizado.scale());
    normalizado.mantissa().checked_mul(fator).ok_or_else(|| {
        capacidade(
            origem,
            "conversao em centavos",
            format!("{campo} = {valor}"),
        )
    })
}

/// Riporta i centesimi a `Decimal`, o dichiara l'incapacita'.
///
/// Si prova prima la scala 2; se la mantissa non ci sta, si **riducono gli
/// zeri finali** abbassando la scala, fino a 0. Riscontro R3B-01 residuo B
/// dell'audit del 2026-10-16: imporre la scala 2 restringeva artificialmente
/// il dominio di `Decimal`, perche' un importo intero grande e' rappresentabile
/// a scala 0 anche quando `mantissa x 100` non ci sta piu'.
///
/// Nessuna cifra diversa da zero viene mai eliminata e non si arrotonda: se il
/// centesimo non e' divisibile, il valore **non** e' rappresentabile e la
/// funzione restituisce l'errore.
fn de_centesimos(
    centesimos: i128,
    operacao: &'static str,
    origem: &str,
) -> Result<Decimal, ErroAlocacao> {
    let mut mantissa = centesimos;
    let mut escala: u32 = 2;
    loop {
        if let Ok(valor) = Decimal::try_from_i128_with_scale(mantissa, escala) {
            return Ok(valor);
        }
        // Solo zeri finali: il valore resta identico.
        if escala == 0 || mantissa % 10 != 0 {
            break;
        }
        mantissa /= 10;
        escala -= 1;
    }
    Err(capacidade(
        origem,
        operacao,
        format!(
            "{centesimos} centavos nao sao representaveis em Decimal nem reduzindo \
             zeros finais, e nenhum centavo pode ser descartado"
        ),
    ))
}

fn somar_centesimos(
    valores: impl IntoIterator<Item = i128>,
    operacao: &'static str,
    origem: &str,
) -> Result<i128, ErroAlocacao> {
    let mut acumulado: i128 = 0;
    for valor in valores {
        acumulado = acumulado
            .checked_add(valor)
            .ok_or_else(|| capacidade(origem, operacao, format!("{acumulado} + {valor}")))?;
    }
    Ok(acumulado)
}

/// Divisione intera con arrotondamento `ROUND_HALF_UP` (meta' lontano da zero).
///
/// Il resto decide: nessun passaggio in virgola mobile, nessun raddoppio del
/// numeratore che potrebbe eccedere `i128`.
///
/// Divisione e resto passano per `checked_div`/`checked_rem`: riscontro R3B-01
/// residuo A dell'audit del 2026-10-16, `i128::MIN / -1` non e' rappresentabile
/// e con gli operatori ordinari andava in **panico**. L'incapacita' numerica
/// deve essere un errore tipizzato, mai un panico.
fn dividir_meio_para_cima(
    numerador: i128,
    denominador: i128,
    origem: &str,
) -> Result<i128, ErroAlocacao> {
    if denominador == 0 {
        return Err(capacidade(
            origem,
            "divisao do rateio",
            "denominador zero nao chega a esta funcao",
        ));
    }
    let excedido = || {
        capacidade(
            origem,
            "divisao do rateio",
            format!("{numerador} / {denominador} nao e' representavel em i128"),
        )
    };
    let quociente = numerador.checked_div(denominador).ok_or_else(excedido)?;
    let resto = numerador.checked_rem(denominador).ok_or_else(excedido)?;
    if resto == 0 {
        return Ok(quociente);
    }
    let dobro_do_resto = resto
        .checked_abs()
        .and_then(|r| r.checked_mul(2))
        .ok_or_else(|| capacidade(origem, "divisao do rateio", format!("resto {resto}")))?;
    if dobro_do_resto < denominador.abs() {
        return Ok(quociente);
    }
    let positivo = (numerador >= 0) == (denominador >= 0);
    let ajuste = if positivo { 1 } else { -1 };
    quociente.checked_add(ajuste).ok_or_else(|| {
        capacidade(
            origem,
            "divisao do rateio",
            format!("{quociente} + {ajuste}"),
        )
    })
}

fn aviso(codigo: &str, detalhe: impl Into<String>) -> Aviso {
    Aviso {
        codigo: codigo.to_string(),
        detalhe: detalhe.into(),
    }
}

/// Somma di `Decimal` usata **solo nei test**, dove gli importi sono piccoli
/// e verificati: l'aritmetica di produzione sta in centesimi interi, vedi
/// [`somar_centesimos`].
#[cfg(test)]
fn soma(valores: impl IntoIterator<Item = Decimal>) -> Decimal {
    valores.into_iter().fold(Decimal::ZERO, |acc, v| acc + v)
}

// ---------------------------------------------------------------------------
// Gruppi per conto
// ---------------------------------------------------------------------------

/// Gruppo di articoli che finiscono sullo stesso conto.
struct Grupo {
    conta_id: String,
    conta_nome: String,
    grupo_conta: String,
    tipo_lancamento: crate::modelos::TipoConta,
    descricoes: Vec<String>,
    /// Valore in centesimi: la somma non perde mai un centesimo.
    valor: i128,
    avisos: Vec<String>,
    todas_classificacoes_ok: bool,
}

/// F3A-01/02: raggruppa nell'ordine di prima comparsa del conto.
fn agrupar_por_conta(
    nfe: &Nfe,
    classificador: &Classificador,
    plano: &PlanoContas,
    origem: &str,
) -> Result<Vec<Grupo>, ErroAlocacao> {
    let mut grupos: Vec<Grupo> = Vec::new();

    for produto in &nfe.produtos {
        let classificacao = classificador
            .classificar(&produto.descricao, plano)
            .map_err(|erro| ErroAlocacao::ClassificacaoFalhou {
                origem: origem.to_string(),
                erro,
            })?;

        let campo = format!("det[{}]/prod/vProd", produto.n_item);
        let valor_item = em_centesimos(produto.valor_total, &campo, origem)?;

        let posicao = grupos
            .iter()
            .position(|g| g.conta_id == classificacao.conta.id);
        let indice = match posicao {
            Some(i) => i,
            None => {
                grupos.push(Grupo {
                    conta_id: classificacao.conta.id.clone(),
                    conta_nome: classificacao.conta.nome.clone(),
                    grupo_conta: classificacao.conta.grupo.clone(),
                    tipo_lancamento: classificacao.conta.tipo,
                    descricoes: Vec::new(),
                    valor: 0,
                    avisos: Vec::new(),
                    todas_classificacoes_ok: true,
                });
                grupos.len() - 1
            }
        };

        let grupo = &mut grupos[indice];
        grupo
            .descricoes
            .push(descrever_item(produto, &mut grupo.avisos));
        grupo.valor = grupo.valor.checked_add(valor_item).ok_or_else(|| {
            capacidade(
                origem,
                "soma dos itens do grupo",
                format!("conta {}: {} + {valor_item}", grupo.conta_id, grupo.valor),
            )
        })?;
        if classificacao.confianca != Confianca::Ok {
            grupo.todas_classificacoes_ok = false;
        }
        for a in avisos_da_classificacao(&classificacao) {
            if !grupo.avisos.contains(&a) {
                grupo.avisos.push(a);
            }
        }
    }

    Ok(grupos)
}

/// F3A-04/24: descrizione dell'articolo nella riga.
///
/// L'importo esce **sempre con due decimali**, come il `f"{...:.2f}"` del
/// Python: riscontro R3B-02 dell'audit, perche' D6 ammette `10` e `10.0` come
/// importi legittimi e la rappresentazione non deve dipendere dalla scala
/// scritta nell'XML.
///
/// Un articolo senza descrizione **resta** nella registrazione con il suo
/// importo: il testo esce `" (12.00)"` come nel Python, e l'assenza diventa un
/// avviso. Nessuna descrizione viene inventata.
fn descrever_item(produto: &ProdutoNfe, avisos_grupo: &mut Vec<String>) -> String {
    if produto.descricao.trim().is_empty() {
        let codigo = avisos::ITEM_SEM_DESCRICAO.to_string();
        if !avisos_grupo.contains(&codigo) {
            avisos_grupo.push(codigo);
        }
    }
    format!(
        "{} ({:.2})",
        produto.descricao,
        centavos(produto.valor_total)
    )
}

/// F3A-13a/b: il non classificato va al conto di revisione scelto dal
/// classificatore; l'ambiguo **conserva** il conto vincente e porta l'avviso.
fn avisos_da_classificacao(classificacao: &ResultadoClassificacao) -> Vec<String> {
    let mut lista: Vec<String> = classificacao.alertas.clone();
    match classificacao.confianca {
        Confianca::NaoClassificado => lista.push(avisos::NAO_CLASSIFICADO.to_string()),
        Confianca::Ambiguo => lista.push(avisos::CLASSIFICACAO_AMBIGUA.to_string()),
        Confianca::Ok => {}
    }
    lista
}

// ---------------------------------------------------------------------------
// Riparto delle rate
// ---------------------------------------------------------------------------

/// F3A-08: matrice rate x registrazioni in centesimi, con i resti nell'ultima
/// riga e nell'ultima colonna. Porting di `_build_allocation_matrix`, con il
/// prodotto esatto in `i128` invece del prodotto `Decimal` che va in panico.
fn construir_matriz_centesimos(
    rate: &[i128],
    lancamentos: &[i128],
    total: i128,
    origem: &str,
) -> Result<Vec<Vec<i128>>, ErroAlocacao> {
    let n_righe = rate.len();
    let n_colonne = lancamentos.len();
    let mut matriz = vec![vec![0i128; n_colonne]; n_righe];

    for r in 0..n_righe {
        for c in 0..n_colonne {
            let ultima_riga = r == n_righe - 1;
            let ultima_colonna = c == n_colonne - 1;

            let valor = if !ultima_riga && !ultima_colonna {
                let produto = rate[r].checked_mul(lancamentos[c]).ok_or_else(|| {
                    capacidade(
                        origem,
                        "produto do rateio",
                        format!("{} centavos x {} centavos", rate[r], lancamentos[c]),
                    )
                })?;
                dividir_meio_para_cima(produto, total, origem)?
            } else if !ultima_colonna {
                let acima = somar_centesimos(
                    (0..r).map(|k| matriz[k][c]),
                    "resto da coluna do rateio",
                    origem,
                )?;
                lancamentos[c].checked_sub(acima).ok_or_else(|| {
                    capacidade(
                        origem,
                        "resto da coluna do rateio",
                        format!("{} - {acima}", lancamentos[c]),
                    )
                })?
            } else {
                let esquerda = somar_centesimos(
                    (0..c).map(|k| matriz[r][k]),
                    "resto da linha do rateio",
                    origem,
                )?;
                rate[r].checked_sub(esquerda).ok_or_else(|| {
                    capacidade(
                        origem,
                        "resto da linha do rateio",
                        format!("{} - {esquerda}", rate[r]),
                    )
                })?
            };
            matriz[r][c] = valor;
        }
    }

    Ok(matriz)
}

/// Versione a `Decimal` della matrice, usata **solo nei test** per leggere le
/// celle in forma monetaria; il riparto di produzione lavora in centesimi.
#[cfg(test)]
fn construir_matriz(
    rate: &[Decimal],
    lancamentos: &[Decimal],
    total: Decimal,
) -> Result<Vec<Vec<Decimal>>, ErroAlocacao> {
    let origem = "teste://matriz";
    let rate_cent: Vec<i128> = rate
        .iter()
        .map(|v| em_centesimos(*v, "rata", origem))
        .collect::<Result<_, _>>()?;
    let lanc_cent: Vec<i128> = lancamentos
        .iter()
        .map(|v| em_centesimos(*v, "lancamento", origem))
        .collect::<Result<_, _>>()?;
    let total_cent = em_centesimos(total, "total", origem)?;
    let matriz = construir_matriz_centesimos(&rate_cent, &lanc_cent, total_cent, origem)?;
    matriz
        .into_iter()
        .map(|linha| {
            linha
                .into_iter()
                .map(|c| de_centesimos(c, "celula do rateio", origem))
                .collect::<Result<Vec<Decimal>, _>>()
        })
        .collect()
}

/// Esito del riparto, con gli avvisi che ne derivano.
struct ResultadoRate {
    esito: EsitoRate,
    avisos: Vec<Aviso>,
    soma_documento: Dado<Decimal>,
    soma_allocate: Dado<Decimal>,
}

/// F3A-09/10 e D1: riparto, o rifiuto quando non e' definito.
fn repartir_rate(
    nfe: &Nfe,
    lancamentos: &mut [Lancamento],
    lancamentos_cent: &[i128],
    total_cent: i128,
    origem: &str,
) -> Result<ResultadoRate, ErroAlocacao> {
    let fonte_rate = "nfe:cobr/dup/vDup";
    let incoerente = |erro: ErroDado| ErroAlocacao::ModeloIncoerente {
        origem: origem.to_string(),
        erro,
    };

    if nfe.duplicatas.is_empty() {
        let ausente = |motivo: &str| {
            Dado::ausente(fonte_rate, StatusDado::NaoAplicavel, motivo).map_err(incoerente)
        };
        return Ok(ResultadoRate {
            esito: EsitoRate::NaoAplicavel,
            avisos: Vec::new(),
            soma_documento: ausente("NF-e sem duplicatas")?,
            soma_allocate: ausente("NF-e sem duplicatas")?,
        });
    }

    let mut rate_cent: Vec<i128> = Vec::with_capacity(nfe.duplicatas.len());
    for duplicata in &nfe.duplicatas {
        let campo = format!("cobr/dup[{}]/vDup", duplicata.numero);
        rate_cent.push(em_centesimos(duplicata.valor, &campo, origem)?);
    }
    let soma_rate_documento = somar_centesimos(
        rate_cent.iter().copied(),
        "soma das duplicatas do documento",
        origem,
    )?;

    let mut avisos_rate = Vec::new();
    if soma_rate_documento != total_cent {
        avisos_rate.push(aviso(
            avisos::VALOR_INCONSISTENTE,
            format!(
                "soma das duplicatas {} difere do total da NF-e {}",
                de_centesimos(soma_rate_documento, "soma das duplicatas", origem)?,
                de_centesimos(total_cent, "total da NF-e", origem)?
            ),
        ));
    }

    // D1: con denominatore zero il rateio proporzionale non e' definito. Si
    // accetta solo il caso banale in cui tutto e' realmente zero.
    if total_cent == 0 {
        let tudo_zero =
            lancamentos_cent.iter().all(|v| *v == 0) && rate_cent.iter().all(|v| *v == 0);
        if !tudo_zero {
            let soma_lancamentos = somar_centesimos(
                lancamentos_cent.iter().copied(),
                "soma dos lancamentos",
                origem,
            )?;
            return Err(ErroAlocacao::RipartoImpossivel {
                origem: origem.to_string(),
                motivo: format!(
                    "total da NF-e e' 0.00 mas ha lancamentos ou duplicatas nao nulos \
                     (soma dos lancamentos {}, soma das duplicatas {}): \
                     nenhum valor declarado e' reescrito como zero",
                    de_centesimos(soma_lancamentos, "soma dos lancamentos", origem)?,
                    de_centesimos(soma_rate_documento, "soma das duplicatas", origem)?
                ),
            });
        }
        let zero = de_centesimos(0, "rateio trivial", origem)?;
        for lancamento in lancamentos.iter_mut() {
            lancamento.duplicatas = nfe
                .duplicatas
                .iter()
                .map(|d| Duplicata {
                    numero: d.numero.clone(),
                    vencimento: d.vencimento,
                    valor: zero,
                })
                .collect();
        }
        return Ok(ResultadoRate {
            esito: EsitoRate::Ok,
            avisos: avisos_rate,
            soma_documento: Dado::presente(zero, fonte_rate).map_err(incoerente)?,
            soma_allocate: Dado::presente(zero, fonte_rate).map_err(incoerente)?,
        });
    }

    let matriz = construir_matriz_centesimos(&rate_cent, lancamentos_cent, total_cent, origem)?;

    for (c, lancamento) in lancamentos.iter_mut().enumerate() {
        let mut duplicatas = Vec::with_capacity(nfe.duplicatas.len());
        for (r, d) in nfe.duplicatas.iter().enumerate() {
            duplicatas.push(Duplicata {
                numero: d.numero.clone(),
                vencimento: d.vencimento,
                valor: de_centesimos(matriz[r][c], "celula do rateio", origem)?,
            });
        }
        lancamento.duplicatas = duplicatas;

        let allocato = somar_centesimos(
            (0..matriz.len()).map(|r| matriz[r][c]),
            "soma das duplicatas do lancamento",
            origem,
        )?;
        if allocato != lancamentos_cent[c] {
            avisos_rate.push(aviso(
                avisos::VALOR_INCONSISTENTE,
                format!(
                    "lancamento {}: duplicatas somam {}, valor {}",
                    lancamento.conta_id,
                    de_centesimos(allocato, "soma alocada", origem)?,
                    lancamento.valor
                ),
            ));
        }
    }

    for (r, duplicata) in nfe.duplicatas.iter().enumerate() {
        let allocato =
            somar_centesimos(matriz[r].iter().copied(), "soma da linha do rateio", origem)?;
        if allocato != rate_cent[r] {
            avisos_rate.push(aviso(
                avisos::VALOR_INCONSISTENTE,
                format!(
                    "duplicata {}: alocado {}, valor {}",
                    duplicata.numero,
                    de_centesimos(allocato, "soma alocada", origem)?,
                    de_centesimos(rate_cent[r], "valor da duplicata", origem)?
                ),
            ));
        }
    }

    let soma_allocate = somar_centesimos(
        matriz.iter().flat_map(|linha| linha.iter().copied()),
        "soma total alocada",
        origem,
    )?;
    let esito = if avisos_rate.is_empty() {
        EsitoRate::Ok
    } else {
        EsitoRate::Erro
    };

    Ok(ResultadoRate {
        esito,
        avisos: deduplicar(avisos_rate),
        soma_documento: Dado::presente(
            de_centesimos(soma_rate_documento, "soma das duplicatas", origem)?,
            fonte_rate,
        )
        .map_err(incoerente)?,
        soma_allocate: Dado::presente(
            de_centesimos(soma_allocate, "soma total alocada", origem)?,
            fonte_rate,
        )
        .map_err(incoerente)?,
    })
}

fn deduplicar(avisos: Vec<Aviso>) -> Vec<Aviso> {
    let mut vistos: Vec<Aviso> = Vec::new();
    for a in avisos {
        if !vistos.contains(&a) {
            vistos.push(a);
        }
    }
    vistos
}

// ---------------------------------------------------------------------------
// Hash
// ---------------------------------------------------------------------------

/// F3A-15: payload di quattro campi, identico al Python.
fn payload_hash(lancamento: &Lancamento) -> String {
    format!(
        "{}|{}|{}|{:.2}",
        ORIGEM_XML_NFE, lancamento.chave_origem, lancamento.conta_id, lancamento.valor
    )
}

fn sha256_hex(payload: &str) -> String {
    let digest = Sha256::digest(payload.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

// ---------------------------------------------------------------------------
// Ingresso
// ---------------------------------------------------------------------------

/// Alloca una NF-e letta in registrazioni, con riconciliazione.
///
/// Funzione pura: non scrive niente e non consulta nessun archivio.
pub fn alocar_nfe(
    importada: &NfeImportada,
    classificador: &Classificador,
    plano: &PlanoContas,
) -> Result<AlocacaoNfe, ErroAlocacao> {
    let nfe = &importada.nfe;
    let origem = if nfe.arquivo_origem.trim().is_empty() {
        nfe.chave.clone()
    } else {
        nfe.arquivo_origem.clone()
    };

    // D5/F3A-23a: la chiave entra nel payload dell'hash. Senza di lei la riga
    // non e' identificabile, quindi non si produce niente.
    if nfe.chave.trim().is_empty() {
        return Err(ErroAlocacao::IdentidadeInsuficiente {
            origem,
            campo: "chave",
        });
    }

    // D6/F3A-26: dominio monetario sugli ingressi, prima di qualunque calcolo.
    // La conversione in centesimi esige il dominio e lo verifica di nuovo.
    let total_cent = em_centesimos(nfe.valor_total, "total/ICMSTot/vNF", &origem)?;
    let produtos_declarados_cent =
        em_centesimos(nfe.valor_produtos, "total/ICMSTot/vProd", &origem)?;
    let frete_cent = em_centesimos(nfe.valor_frete, "total/ICMSTot/vFrete", &origem)?;
    let seguro_cent = em_centesimos(nfe.valor_seguro, "total/ICMSTot/vSeg", &origem)?;
    let desconto_cent = em_centesimos(nfe.valor_desconto, "total/ICMSTot/vDesc", &origem)?;
    let outras_cent = em_centesimos(nfe.valor_outras_despesas, "total/ICMSTot/vOutro", &origem)?;
    // Un Dado assente salta il controllo: non viene azzerato per superarlo.
    let tributos_cent = match nfe.valor_tributos_inclusos.valor() {
        Some(valor) => Some(em_centesimos(
            *valor,
            "total/ICMSTot (tributos inclusos)",
            &origem,
        )?),
        None => None,
    };
    let _ = produtos_declarados_cent;

    let competencia = Competencia::from_data(nfe.data_emissao).map_err(|erro| {
        ErroAlocacao::CompetenciaInvalida {
            origem: origem.clone(),
            erro,
        }
    })?;

    let grupos = agrupar_por_conta(nfe, classificador, plano, &origem)?;

    let mut avisos_documento: Vec<Aviso> = importada.avisos.clone();
    let mut lancamentos: Vec<Lancamento> = Vec::new();
    let mut lancamentos_cent: Vec<i128> = Vec::new();
    let mut esito_componenti = EsitoComponenti::Ok;
    let mut tributos_usados = false;

    // F3A-01: nessun gruppo, nessuna registrazione, nemmeno di differenza
    // (il Python ritorna una lista vuota prima di calcolarla).
    if !grupos.is_empty() {
        for grupo in &grupos {
            lancamentos.push(montar_lancamento(
                nfe,
                competencia,
                grupo.conta_id.clone(),
                grupo.conta_nome.clone(),
                grupo.grupo_conta.clone(),
                grupo.tipo_lancamento,
                grupo.descricoes.join("; "),
                de_centesimos(grupo.valor, "valor do grupo", &origem)?,
                grupo.avisos.clone(),
                grupo.todas_classificacoes_ok && grupo.avisos.is_empty(),
                &origem,
            )?);
            lancamentos_cent.push(grupo.valor);
        }

        // F3A-05/06/07: differenza di totale.
        let soma_produtos_cent =
            somar_centesimos(grupos.iter().map(|g| g.valor), "soma dos produtos", &origem)?;
        let diferenca_cent = total_cent.checked_sub(soma_produtos_cent).ok_or_else(|| {
            capacidade(
                &origem,
                "diferenca de total",
                format!("{total_cent} - {soma_produtos_cent}"),
            )
        })?;

        if diferenca_cent != 0 {
            let (conta_id, descricao) = if diferenca_cent > 0 {
                (
                    CONTA_ACESSORIOS,
                    "Custos acessorios identificados no total da NF-e",
                )
            } else {
                (
                    CONTA_DESCONTOS,
                    "Descontos obtidos identificados no total da NF-e",
                )
            };
            let conta =
                plano
                    .por_id(conta_id)
                    .ok_or_else(|| ErroAlocacao::ContaDeAjusteAusente {
                        origem: origem.clone(),
                        conta_id: conta_id.to_string(),
                    })?;

            let mut avisos_diferenca: Vec<String> = Vec::new();
            let explicada = if diferenca_cent > 0 {
                tributos_usados = true;
                // D2/F3A-07: una componente non `Ok` non spiega niente.
                match tributos_cent {
                    Some(tributos) if nfe.valor_tributos_inclusos.is_ok() => {
                        let identificados = somar_centesimos(
                            [frete_cent, seguro_cent, outras_cent, tributos],
                            "soma dos acessorios identificados",
                            &origem,
                        )?;
                        diferenca_cent == identificados
                    }
                    _ => {
                        esito_componenti = EsitoComponenti::Parcial;
                        let motivo = nfe
                            .valor_tributos_inclusos
                            .motivo()
                            .unwrap_or("componente nao verificada")
                            .to_string();
                        avisos_documento.push(aviso(
                            avisos::AGREGADO_PARCIAL,
                            format!(
                                "tributos inclusos com estado {}: a diferenca de total nao pode \
                                 ser declarada explicada ({motivo})",
                                nfe.valor_tributos_inclusos.status().rotulo()
                            ),
                        ));
                        false
                    }
                }
            } else {
                diferenca_cent.checked_abs().ok_or_else(|| {
                    capacidade(&origem, "diferenca de total", format!("|{diferenca_cent}|"))
                })? == desconto_cent
            };

            if !explicada {
                avisos_diferenca.push(avisos::VALOR_INCONSISTENTE.to_string());
            }

            lancamentos.push(montar_lancamento(
                nfe,
                competencia,
                conta.id.clone(),
                conta.nome.clone(),
                conta.grupo.clone(),
                conta.tipo,
                descricao.to_string(),
                de_centesimos(diferenca_cent, "diferenca de total", &origem)?,
                avisos_diferenca,
                explicada,
                &origem,
            )?);
            lancamentos_cent.push(diferenca_cent);
        }
    }

    // F3A-08/09/10 e D1: riparto delle rate.
    let resultado_rate = repartir_rate(
        nfe,
        &mut lancamentos,
        &lancamentos_cent,
        total_cent,
        &origem,
    )?;

    // D3/F3A-11: se il riparto non chiude, tutte le registrazioni vanno a
    // revisione, senza toccare conti ne' valori.
    if resultado_rate.esito == EsitoRate::Erro {
        for lancamento in &mut lancamentos {
            adicionar_aviso(lancamento, avisos::VALOR_INCONSISTENTE);
        }
    }
    for a in &resultado_rate.avisos {
        avisos_documento.push(a.clone());
    }

    // D5/F3A-23b: identita' incomplete non bloccano, ma vanno a revisione.
    for (codigo, vazio, detalhe) in [
        (
            avisos::DOCUMENTO_SEM_NUMERO,
            nfe.numero.trim().is_empty(),
            "NF-e sem numero (ide/nNF vazio)",
        ),
        (
            avisos::FORNECEDOR_SEM_IDENTIDADE,
            nfe.cnpj_emitente.trim().is_empty() || nfe.nome_emitente.trim().is_empty(),
            "emitente sem CNPJ ou sem nome",
        ),
        (
            avisos::EMPRESA_SEM_IDENTIDADE,
            nfe.cnpj_destinatario.trim().is_empty() || nfe.nome_destinatario.trim().is_empty(),
            "destinatario sem CNPJ/CPF ou sem nome",
        ),
    ] {
        if vazio {
            avisos_documento.push(aviso(codigo, detalhe));
            for lancamento in &mut lancamentos {
                adicionar_aviso(lancamento, codigo);
            }
        }
    }

    // F3A-15/27 e D4: hash, e segnalazione del payload ripetuto.
    let mut payloads: HashMap<String, usize> = HashMap::new();
    for lancamento in &lancamentos {
        *payloads.entry(payload_hash(lancamento)).or_insert(0) += 1;
    }
    for lancamento in &mut lancamentos {
        let payload = payload_hash(lancamento);
        lancamento.hash_registro = sha256_hex(&payload);
        if payloads.get(&payload).copied().unwrap_or(0) > 1 {
            adicionar_aviso(lancamento, avisos::HASH_DUPLICADO_NO_DOCUMENTO);
        }
    }
    if payloads.values().any(|n| *n > 1) {
        avisos_documento.push(aviso(
            avisos::HASH_DUPLICADO_NO_DOCUMENTO,
            "duas registrazioni do mesmo documento compartilham o payload do hash: \
             nenhuma e' descartada",
        ));
    }

    // F3A-12: qualunque avviso porta a revisione.
    for lancamento in &mut lancamentos {
        if !lancamento.alertas.is_empty() {
            lancamento.status_revisao = StatusRevisao::Revisar;
        }
    }

    // Copertura: l'atteso resta assente, lo scarto si legge dagli avvisi.
    let articoli_scartati: Vec<Aviso> = importada
        .avisos
        .iter()
        .filter(|a| AVISOS_DE_SCARTO.contains(&a.codigo.as_str()))
        .cloned()
        .collect();
    if lancamentos.is_empty() {
        avisos_documento.push(aviso(
            avisos::DOCUMENTO_SEM_ITENS,
            "nenhum item alocavel: nenhuma registrazione foi produzida",
        ));
    }
    let esito_copertura = if articoli_scartati.is_empty() && !lancamentos.is_empty() {
        EsitoCopertura::Ok
    } else {
        EsitoCopertura::Parcial
    };

    let soma_lancamentos_cent = somar_centesimos(
        lancamentos_cent.iter().copied(),
        "soma dos lancamentos",
        &origem,
    )?;
    let soma_lancamentos = de_centesimos(soma_lancamentos_cent, "soma dos lancamentos", &origem)?;
    let valor_total_nfe = de_centesimos(total_cent, "total da NF-e", &origem)?;
    let esito_total = if soma_lancamentos_cent == total_cent {
        EsitoTotal::Ok
    } else {
        avisos_documento.push(aviso(
            avisos::TOTAL_NAO_RECONCILIADO,
            format!(
                "soma dos lancamentos {soma_lancamentos} difere do total da NF-e {valor_total_nfe}"
            ),
        ));
        EsitoTotal::Erro
    };

    let reconciliacao = Reconciliacao {
        valor_total_nfe,
        soma_lancamentos,
        esito_total,
        soma_rate_documento: resultado_rate.soma_documento,
        soma_rate_allocate: resultado_rate.soma_allocate,
        esito_rate: resultado_rate.esito,
        articoli_allocati: nfe.produtos.len(),
        articoli_attesi: Dado::ausente(
            "nfe:det",
            StatusDado::NaoDisponivel,
            "a fase 2 nao expoe o numero de <det> lidos: a cobertura mede apenas a \
             ausencia de avisos de descarte",
        )
        .map_err(|erro| ErroAlocacao::ModeloIncoerente {
            origem: origem.clone(),
            erro,
        })?,
        articoli_scartati,
        esito_copertura,
        componenti: montar_componenti(nfe, tributos_usados),
        esito_componenti,
        campi_rinviati: campi_rinviati(),
        avisos_documento: deduplicar(avisos_documento),
        stato_documento: Reconciliacao::agregar(
            esito_total,
            resultado_rate.esito,
            esito_copertura,
            esito_componenti,
        ),
    };

    Ok(AlocacaoNfe {
        lancamentos,
        reconciliacao,
    })
}

fn adicionar_aviso(lancamento: &mut Lancamento, codigo: &str) {
    let existentes: Vec<&str> = lancamento
        .alertas
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if existentes.contains(&codigo) {
        return;
    }
    if lancamento.alertas.is_empty() {
        lancamento.alertas = codigo.to_string();
    } else {
        lancamento.alertas = format!("{}; {}", lancamento.alertas, codigo);
    }
    lancamento.status_revisao = StatusRevisao::Revisar;
}

#[allow(clippy::too_many_arguments)]
fn montar_lancamento(
    nfe: &Nfe,
    competencia: Competencia,
    conta_id: String,
    conta_nome: String,
    grupo_conta: String,
    tipo_lancamento: crate::modelos::TipoConta,
    descricao_itens: String,
    valor: Decimal,
    avisos_iniciais: Vec<String>,
    sem_revisao: bool,
    origem: &str,
) -> Result<Lancamento, ErroAlocacao> {
    let alertas = {
        let mut unicos: Vec<String> = Vec::new();
        for a in avisos_iniciais {
            if !unicos.contains(&a) {
                unicos.push(a);
            }
        }
        unicos.join("; ")
    };
    let status_revisao = if sem_revisao && alertas.is_empty() {
        StatusRevisao::Ok
    } else {
        StatusRevisao::Revisar
    };

    Ok(Lancamento {
        data_documento: nfe.data_emissao,
        competencia,
        cnpj_empresa: nfe.cnpj_destinatario.clone(),
        nome_empresa: nfe.nome_destinatario.clone(),
        fornecedor_ou_favorecido: nfe.nome_emitente.clone(),
        cnpj_cpf_favorecido: nfe.cnpj_emitente.clone(),
        documento: nfe.numero.clone(),
        tipo_documento: ORIGEM_XML_NFE.to_string(),
        conta_id,
        conta_nome,
        grupo_conta,
        descricao_itens,
        valor,
        duplicatas: Vec::new(),
        origem: OrigemLancamento::XmlNfe,
        arquivo_origem: nfe.arquivo_origem.clone(),
        chave_origem: nfe.chave.clone(),
        hash_registro: String::new(),
        status_revisao,
        alertas,
        observacoes: String::new(),
        centro_custo: String::new(),
        tipo_lancamento,
        dre_linha_id: String::new(),
        dre_linha_nome: String::new(),
        // F3A-17: una scadenza non e' un pagamento.
        data_pagamento: Dado::ausente(
            "nfe:cobr/dup/dVenc",
            StatusDado::NaoDisponivel,
            "a NF-e declara vencimentos, nao pagamentos",
        )
        .map_err(|erro| ErroAlocacao::ModeloIncoerente {
            origem: origem.to_string(),
            erro,
        })?,
    })
}

fn montar_componenti(nfe: &Nfe, tributos_usados: bool) -> Vec<Componente> {
    let presente =
        |nome: &str, valor: Decimal, fonte: &str, ruolo: RuoloComponente, usada: bool| {
            Componente {
                nome: nome.to_string(),
                // `Dado::presente` qui non puo' fallire: la fonte non e' vuota e il
                // valore c'e'. Se fallisse sarebbe un bug di questo modulo, e il
                // fallback mantiene lo stato visibile invece di inventare un valore.
                valor: Dado::presente(valor, fonte).unwrap_or_else(|_| {
                    Dado::ausente(fonte, StatusDado::Erro, "falha ao montar o componente")
                        .expect("fonte e motivo nao vazios")
                }),
                ruolo,
                usada_na_verificacao: usada,
            }
        };

    vec![
        presente(
            "vProd",
            centavos(nfe.valor_produtos),
            "nfe:total/ICMSTot/vProd",
            RuoloComponente::Base,
            true,
        ),
        presente(
            "vFrete",
            centavos(nfe.valor_frete),
            "nfe:total/ICMSTot/vFrete",
            RuoloComponente::Acessorio,
            tributos_usados,
        ),
        presente(
            "vSeg",
            centavos(nfe.valor_seguro),
            "nfe:total/ICMSTot/vSeg",
            RuoloComponente::Acessorio,
            tributos_usados,
        ),
        presente(
            "vOutro",
            centavos(nfe.valor_outras_despesas),
            "nfe:total/ICMSTot/vOutro",
            RuoloComponente::Acessorio,
            tributos_usados,
        ),
        presente(
            "vDesc",
            centavos(nfe.valor_desconto),
            "nfe:total/ICMSTot/vDesc",
            RuoloComponente::Deducao,
            true,
        ),
        Componente {
            nome: "tributos_inclusos".to_string(),
            valor: nfe.valor_tributos_inclusos.clone(),
            ruolo: RuoloComponente::Acessorio,
            usada_na_verificacao: tributos_usados,
        },
        Componente {
            nome: "valor_cobranca".to_string(),
            valor: nfe.valor_cobranca.clone(),
            ruolo: RuoloComponente::Informativa,
            usada_na_verificacao: false,
        },
    ]
}

/// F3A-18/19: i campi che la 3B non popola, con stato, fonte e motivo.
fn campi_rinviati() -> Vec<CampoRinviato> {
    let rinviato = |campo: &str, status: &str, fonte: &str, motivo: &str| CampoRinviato {
        campo: campo.to_string(),
        status: status.to_string(),
        fonte: fonte.to_string(),
        motivo: motivo.to_string(),
    };
    vec![
        rinviato(
            "dre_linha_id",
            "NAO_ASSEGNATO",
            "config/mapeamento_dre.yaml",
            "atribuicao da linha DRE adiada para a fase 3D: a 3B nao le o mapeamento",
        ),
        rinviato(
            "dre_linha_nome",
            "NAO_ASSEGNATO",
            "config/mapeamento_dre.yaml",
            "atribuicao da linha DRE adiada para a fase 3D: a 3B nao le o mapeamento",
        ),
        rinviato(
            "centro_custo",
            "NAO_DISPONIVEL",
            "-",
            "nenhuma fonte liga conta a centro de custo no plano atual",
        ),
        rinviato(
            "observacoes",
            "N.A.",
            "-",
            "campo de anotacao humana, preenchido na revisao (fase 3C)",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modelos::TipoConta;
    use crate::nfe::ler_nfe;
    use crate::regras::RegrasClassificacao;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn raiz() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn plano() -> PlanoContas {
        PlanoContas::carregar(raiz().join("config/plano_contas_gerencial.yaml")).unwrap()
    }

    fn classificador() -> Classificador {
        Classificador::compilar(
            &RegrasClassificacao::carregar(
                raiz().join("config/regras_classificacao_produtos.yaml"),
            )
            .unwrap(),
        )
    }

    fn campione() -> NfeImportada {
        ler_nfe(raiz().join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml")).unwrap()
    }

    fn alocar(importada: &NfeImportada) -> Result<AlocacaoNfe, ErroAlocacao> {
        alocar_nfe(importada, &classificador(), &plano())
    }

    fn dec(bruto: &str) -> Decimal {
        Decimal::from_str_exact(bruto).unwrap()
    }

    fn produto(n: u32, descricao: &str, valor: &str) -> ProdutoNfe {
        ProdutoNfe {
            n_item: n,
            codigo: format!("C{n}"),
            descricao: descricao.to_string(),
            ncm: "00000000".to_string(),
            cfop: "5102".to_string(),
            unidade: "UN".to_string(),
            quantidade: Decimal::ONE,
            valor_unitario: dec(valor),
            valor_total: dec(valor),
        }
    }

    fn duplicata(numero: &str, valor: &str) -> Duplicata {
        Duplicata {
            numero: numero.to_string(),
            vencimento: NaiveDate::from_ymd_opt(2026, 6, 5),
            valor: dec(valor),
        }
    }

    /// NF-e sintetica dichiarata: non sostituisce il campione canonico.
    struct Sintetica {
        nfe: Nfe,
        avisos: Vec<Aviso>,
    }

    impl Sintetica {
        fn nova(total: &str, produtos: Vec<ProdutoNfe>, duplicatas: Vec<Duplicata>) -> Self {
            let produtos_total = soma(produtos.iter().map(|p| p.valor_total));
            Self {
                nfe: Nfe {
                    chave: "29260512345678000195550010001256612345678901".to_string(),
                    numero: "12566".to_string(),
                    serie: "001".to_string(),
                    data_emissao: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
                    cnpj_emitente: "12345678000195".to_string(),
                    nome_emitente: "FORNECEDOR TESTE".to_string(),
                    cnpj_destinatario: "55746722000181".to_string(),
                    nome_destinatario: "TRATTORIA DO SOSSEGO".to_string(),
                    valor_total: dec(total),
                    natureza_operacao: "VENDA".to_string(),
                    valor_produtos: produtos_total,
                    valor_frete: Decimal::ZERO,
                    valor_seguro: Decimal::ZERO,
                    valor_desconto: Decimal::ZERO,
                    valor_outras_despesas: Decimal::ZERO,
                    valor_tributos_inclusos: Dado::presente(Decimal::ZERO, "nfe:total/ICMSTot")
                        .unwrap(),
                    produtos,
                    duplicatas: duplicatas.clone(),
                    tem_duplicatas: !duplicatas.is_empty(),
                    tem_cobranca: !duplicatas.is_empty(),
                    valor_cobranca: Dado::ausente(
                        "nfe:cobr",
                        StatusDado::NaoAplicavel,
                        "sintetica sem bloco cobr",
                    )
                    .unwrap(),
                    arquivo_origem: "teste://sintetica".to_string(),
                },
                avisos: Vec::new(),
            }
        }

        fn importada(self) -> NfeImportada {
            NfeImportada {
                nfe: self.nfe,
                avisos: self.avisos,
            }
        }
    }

    fn por_conta<'a>(alocacao: &'a AlocacaoNfe, conta_id: &str) -> &'a Lancamento {
        alocacao
            .lancamentos
            .iter()
            .find(|l| l.conta_id == conta_id)
            .unwrap_or_else(|| panic!("conta {conta_id} ausente"))
    }

    // -----------------------------------------------------------------
    // Allocazione e ordine
    // -----------------------------------------------------------------

    #[test]
    fn aloca_campione_canonico_em_tres_lancamentos() {
        let alocacao = alocar(&campione()).unwrap();
        assert_eq!(alocacao.lancamentos.len(), 3);
        let contas: Vec<&str> = alocacao
            .lancamentos
            .iter()
            .map(|l| l.conta_id.as_str())
            .collect();
        // Ordine di prima comparsa degli articoli nel documento.
        assert_eq!(contas, vec!["2.01.004", "2.01.002", "2.04.001"]);
        assert_eq!(por_conta(&alocacao, "2.01.004").valor, dec("10.00"));
        assert_eq!(por_conta(&alocacao, "2.01.002").valor, dec("20.00"));
        assert_eq!(por_conta(&alocacao, "2.04.001").valor, dec("5.00"));
        assert_eq!(por_conta(&alocacao, "2.01.004").conta_nome, "Refrigerantes");
        assert_eq!(alocacao.reconciliacao.soma_lancamentos, dec("35.00"));
        assert_eq!(alocacao.reconciliacao.esito_total, EsitoTotal::Ok);
        assert_eq!(
            alocacao.reconciliacao.stato_documento,
            StatoDocumento::Ok,
            "{:?}",
            alocacao.reconciliacao.avisos_documento
        );
        for lancamento in &alocacao.lancamentos {
            assert_eq!(lancamento.status_revisao, StatusRevisao::Ok);
            assert!(lancamento.alertas.is_empty());
            assert_eq!(lancamento.tipo_documento, "XML_NFE");
            assert_eq!(lancamento.origem, OrigemLancamento::XmlNfe);
            assert!(!lancamento.chave_origem.is_empty());
            assert!(!lancamento.arquivo_origem.is_empty());
            assert!(!lancamento.hash_registro.is_empty());
        }
    }

    #[test]
    fn ordem_de_lancamentos_rate_e_avisos_e_deterministica() {
        let primeiro = alocar(&campione()).unwrap();
        let segundo = alocar(&campione()).unwrap();
        assert_eq!(primeiro.lancamentos, segundo.lancamentos);
        assert_eq!(primeiro.reconciliacao, segundo.reconciliacao);
    }

    #[test]
    fn descricao_dos_itens_segue_o_formato_da_fonte() {
        let alocacao = alocar(&campione()).unwrap();
        assert_eq!(
            por_conta(&alocacao, "2.01.004").descricao_itens,
            "COCA-COLA KS 290ML (10.00)"
        );
    }

    #[test]
    fn competencia_vem_da_data_de_emissao() {
        let alocacao = alocar(&campione()).unwrap();
        let competencia = alocacao.lancamentos[0].competencia;
        assert_eq!(competencia.ano(), 2026);
        assert_eq!(competencia.mes(), 5);
        assert_eq!(
            alocacao.lancamentos[0].data_documento,
            NaiveDate::from_ymd_opt(2026, 5, 5).unwrap()
        );
    }

    // -----------------------------------------------------------------
    // Riparto e arrotondamento
    // -----------------------------------------------------------------

    #[test]
    fn riparto_das_rate_fecha_por_linha_e_por_coluna() {
        let alocacao = alocar(&campione()).unwrap();
        let valores = |conta: &str| -> Vec<Decimal> {
            por_conta(&alocacao, conta)
                .duplicatas
                .iter()
                .map(|d| d.valor)
                .collect()
        };
        assert_eq!(valores("2.01.004"), vec![dec("5.00"), dec("5.00")]);
        assert_eq!(valores("2.01.002"), vec![dec("10.00"), dec("10.00")]);
        assert_eq!(valores("2.04.001"), vec![dec("2.50"), dec("2.50")]);

        for lancamento in &alocacao.lancamentos {
            let allocato = soma(lancamento.duplicatas.iter().map(|d| d.valor));
            assert_eq!(allocato, lancamento.valor, "conta {}", lancamento.conta_id);
        }
        assert_eq!(alocacao.reconciliacao.esito_rate, EsitoRate::Ok);
        assert_eq!(
            alocacao.reconciliacao.soma_rate_allocate.valor(),
            Some(&dec("35.00"))
        );
    }

    #[test]
    fn residuo_de_arredondamento_fica_na_ultima_coluna() {
        let matriz = construir_matriz(
            &[dec("10.00"), dec("10.00")],
            &[dec("6.67"), dec("6.67"), dec("6.66")],
            dec("20.00"),
        )
        .unwrap();
        assert_eq!(matriz[0], vec![dec("3.34"), dec("3.34"), dec("3.32")]);
        assert_eq!(matriz[1], vec![dec("3.33"), dec("3.33"), dec("3.34")]);
        for (i, atteso) in [dec("10.00"), dec("10.00")].iter().enumerate() {
            assert_eq!(soma(matriz[i].iter().copied()), *atteso);
        }
        for (c, atteso) in [dec("6.67"), dec("6.67"), dec("6.66")].iter().enumerate() {
            assert_eq!(soma((0..2).map(|r| matriz[r][c])), *atteso);
        }
    }

    #[test]
    fn arredondamento_half_up_nao_bankers() {
        // 2.345 -> HALF_UP 2.35 ; Bankers darebbe 2.34.
        assert_eq!(centavos(dec("2.345")), dec("2.35"));
        assert_eq!(centavos(dec("2.355")), dec("2.36"));
        assert_eq!(centavos(dec("-2.345")), dec("-2.35"));
        // Prova che il default del crate divergerebbe.
        assert_eq!(dec("2.345").round_dp(2), dec("2.34"));
    }

    #[test]
    fn rata_unica_distribui_todo_o_total() {
        let matriz = construir_matriz(
            &[dec("35.00")],
            &[dec("10.00"), dec("20.00"), dec("5.00")],
            dec("35.00"),
        )
        .unwrap();
        assert_eq!(matriz[0], vec![dec("10.00"), dec("20.00"), dec("5.00")]);
    }

    // -----------------------------------------------------------------
    // Differenza di totale
    // -----------------------------------------------------------------

    #[test]
    fn diferenca_positiva_vai_para_conta_de_acessorios() {
        let mut s = Sintetica::nova(
            "110.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "110.00")],
        );
        s.nfe.valor_frete = dec("10.00");
        let alocacao = alocar(&s.importada()).unwrap();

        let acessorios = por_conta(&alocacao, CONTA_ACESSORIOS);
        assert_eq!(acessorios.valor, dec("10.00"));
        assert_eq!(
            acessorios.conta_nome,
            "Fretes, Seguros e Outras Despesas sobre Compras"
        );
        assert_eq!(acessorios.status_revisao, StatusRevisao::Ok);
        assert!(acessorios.alertas.is_empty());
        assert_eq!(alocacao.reconciliacao.soma_lancamentos, dec("110.00"));
        assert_eq!(alocacao.reconciliacao.esito_total, EsitoTotal::Ok);
    }

    #[test]
    fn diferenca_negativa_vai_para_descontos_obtidos() {
        let mut s = Sintetica::nova(
            "90.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "90.00")],
        );
        s.nfe.valor_desconto = dec("10.00");
        let alocacao = alocar(&s.importada()).unwrap();

        assert!(
            alocacao
                .lancamentos
                .iter()
                .all(|l| l.conta_id != CONTA_ACESSORIOS)
        );
        let descontos = por_conta(&alocacao, CONTA_DESCONTOS);
        assert_eq!(descontos.valor, dec("-10.00"));
        assert_eq!(descontos.conta_nome, "Descontos Obtidos sobre Compras");
        assert_eq!(descontos.status_revisao, StatusRevisao::Ok);
        assert_eq!(alocacao.reconciliacao.soma_lancamentos, dec("90.00"));
    }

    #[test]
    fn diferenca_nao_explicada_marca_revisar() {
        let mut s = Sintetica::nova(
            "115.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "115.00")],
        );
        s.nfe.valor_frete = dec("10.00");
        let alocacao = alocar(&s.importada()).unwrap();

        let acessorios = por_conta(&alocacao, CONTA_ACESSORIOS);
        assert_eq!(acessorios.valor, dec("15.00"));
        assert_eq!(acessorios.status_revisao, StatusRevisao::Revisar);
        assert!(
            acessorios.alertas.contains(avisos::VALOR_INCONSISTENTE),
            "{}",
            acessorios.alertas
        );
    }

    /// D2/F3A-07: una componente necessaria non `Ok` non spiega la differenza.
    #[test]
    fn componente_parcial_nao_explica_diferenca() {
        let mut s = Sintetica::nova(
            "112.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "112.00")],
        );
        s.nfe.valor_frete = dec("10.00");
        s.nfe.valor_tributos_inclusos = Dado::novo(
            Some(dec("2.00")),
            "nfe:total/ICMSTot",
            StatusDado::Parcial,
            Some("vFCPST e vIPIDevol ausentes".to_string()),
        )
        .unwrap();

        let alocacao = alocar(&s.importada()).unwrap();
        let acessorios = por_conta(&alocacao, CONTA_ACESSORIOS);
        assert_eq!(acessorios.valor, dec("12.00"));
        assert_eq!(acessorios.status_revisao, StatusRevisao::Revisar);
        assert!(acessorios.alertas.contains(avisos::VALOR_INCONSISTENTE));

        let r = &alocacao.reconciliacao;
        assert_eq!(r.esito_componenti, EsitoComponenti::Parcial);
        assert!(
            r.avisos_documento
                .iter()
                .any(|a| a.codigo == avisos::AGREGADO_PARCIAL)
        );
        assert!(
            r.avisos_documento
                .iter()
                .any(|a| a.detalhe.contains("vFCPST")),
            "{:?}",
            r.avisos_documento
        );
    }

    /// Prova che i quattro esiti restano distinti: il documento e' PARCIAL
    /// mentre totale, rate e copertura sono OK.
    #[test]
    fn documento_parcial_com_componente_necessaria_parcial() {
        let mut s = Sintetica::nova(
            "112.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "112.00")],
        );
        s.nfe.valor_frete = dec("10.00");
        s.nfe.valor_tributos_inclusos = Dado::novo(
            Some(dec("2.00")),
            "nfe:total/ICMSTot",
            StatusDado::Parcial,
            Some("vFCPST e vIPIDevol ausentes".to_string()),
        )
        .unwrap();

        let r = alocar(&s.importada()).unwrap().reconciliacao;
        assert_eq!(r.esito_total, EsitoTotal::Ok);
        assert_eq!(r.esito_rate, EsitoRate::Ok);
        assert_eq!(r.esito_copertura, EsitoCopertura::Ok);
        assert_eq!(r.esito_componenti, EsitoComponenti::Parcial);
        assert_eq!(r.stato_documento, StatoDocumento::Parcial);
    }

    #[test]
    fn sem_diferenca_nao_ha_lancamento_de_ajuste() {
        let alocacao = alocar(&campione()).unwrap();
        assert!(
            alocacao
                .lancamentos
                .iter()
                .all(|l| l.conta_id != CONTA_ACESSORIOS && l.conta_id != CONTA_DESCONTOS)
        );
    }

    // -----------------------------------------------------------------
    // Riconciliazione delle rate (D3)
    // -----------------------------------------------------------------

    #[test]
    fn rate_que_nao_fecham_geram_valor_inconsistente() {
        let s = Sintetica::nova(
            "35.00",
            vec![
                produto(1, "COCA-COLA KS 290ML", "10.00"),
                produto(2, "VINHO RIO SOL TINTO SECO 750ML", "20.00"),
                produto(3, "ESPONJA DUPLA FACE LIMPEZA 3UN", "5.00"),
            ],
            vec![duplicata("001", "17.50"), duplicata("002", "17.00")],
        );
        let alocacao = alocar(&s.importada()).unwrap();
        let r = &alocacao.reconciliacao;

        assert_eq!(r.esito_rate, EsitoRate::Erro);
        // Il totale resta la sua uguaglianza: le registrazioni sommano 35.00.
        assert_eq!(r.esito_total, EsitoTotal::Ok);
        assert_eq!(r.stato_documento, StatoDocumento::Erro);
        // D3: tutte le registrazioni vanno a revisione.
        for lancamento in &alocacao.lancamentos {
            assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
            assert!(lancamento.alertas.contains(avisos::VALOR_INCONSISTENTE));
        }
    }

    // -----------------------------------------------------------------
    // Denominatore zero (D1)
    // -----------------------------------------------------------------

    #[test]
    fn denominatore_zero_banale_e_accettato() {
        let s = Sintetica::nova(
            "0.00",
            vec![produto(1, "COCA-COLA KS 290ML", "0.00")],
            vec![duplicata("001", "0.00")],
        );
        let alocacao = alocar(&s.importada()).unwrap();
        let r = &alocacao.reconciliacao;
        assert_eq!(r.esito_total, EsitoTotal::Ok);
        assert_eq!(r.esito_rate, EsitoRate::Ok);
        assert_eq!(r.stato_documento, StatoDocumento::Ok);
        assert_eq!(alocacao.lancamentos[0].duplicatas[0].valor, Decimal::ZERO);
    }

    #[test]
    fn total_zero_com_rate_nao_vira_zero_tecnico() {
        let s = Sintetica::nova(
            "0.00",
            vec![produto(1, "COCA-COLA KS 290ML", "0.00")],
            vec![duplicata("001", "10.00")],
        );
        let erro = alocar(&s.importada()).unwrap_err();
        assert!(
            matches!(erro, ErroAlocacao::RipartoImpossivel { .. }),
            "{erro}"
        );
        assert!(erro.to_string().contains("0.00"), "{erro}");
    }

    /// D1 caso 3 / esempio M: totale netto zero con registrazioni non nulle.
    #[test]
    fn total_netto_zero_com_lancamentos_nao_nulos() {
        let mut s = Sintetica::nova(
            "0.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "0.00")],
        );
        s.nfe.valor_desconto = dec("100.00");
        let erro = alocar(&s.importada()).unwrap_err();
        assert!(
            matches!(erro, ErroAlocacao::RipartoImpossivel { .. }),
            "{erro}"
        );
        // Il Python usciva OK e silenzioso: qui il motivo e' nominato.
        assert!(erro.to_string().contains("lancamentos"), "{erro}");
    }

    #[test]
    fn sem_duplicatas_o_rateio_e_nao_aplicavel() {
        let s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        let r = alocar(&s.importada()).unwrap().reconciliacao;
        assert_eq!(r.esito_rate, EsitoRate::NaoAplicavel);
        assert_eq!(r.soma_rate_documento.status(), StatusDado::NaoAplicavel);
        assert_eq!(r.stato_documento, StatoDocumento::Ok);
    }

    // -----------------------------------------------------------------
    // Dominio monetario (D6)
    // -----------------------------------------------------------------

    #[test]
    fn importo_fora_do_dominio_monetario_e_recusado() {
        let s = Sintetica::nova(
            "0.01",
            vec![
                produto(1, "COCA-COLA KS 290ML", "0.005"),
                produto(2, "VINHO RIO SOL TINTO SECO 750ML", "0.005"),
            ],
            vec![],
        );
        let erro = alocar(&s.importada()).unwrap_err();
        match &erro {
            ErroAlocacao::EscalaMonetariaInvalida { campo, valor, .. } => {
                assert!(campo.contains("vProd"), "{campo}");
                assert_eq!(*valor, dec("0.005"));
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(erro.to_string().contains("0.005"), "{erro}");
    }

    #[test]
    fn zeros_finais_nao_squalificam_o_importo() {
        let s = Sintetica::nova(
            "10.0000",
            vec![produto(1, "COCA-COLA KS 290ML", "10.0000")],
            vec![duplicata("001", "10.0000")],
        );
        let alocacao = alocar(&s.importada()).unwrap();
        assert_eq!(alocacao.lancamentos[0].valor, dec("10.00"));
        assert_eq!(alocacao.reconciliacao.esito_total, EsitoTotal::Ok);
        assert_eq!(alocacao.reconciliacao.esito_rate, EsitoRate::Ok);
    }

    #[test]
    fn preco_unitario_com_mais_decimais_e_ammesso() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.produtos[0].valor_unitario = dec("0.3333");
        s.nfe.produtos[0].quantidade = dec("30.003");
        let alocacao = alocar(&s.importada()).unwrap();
        assert_eq!(alocacao.lancamentos[0].valor, dec("10.00"));
    }

    #[test]
    fn dado_assente_non_viene_azzerato_per_passare_il_dominio() {
        let mut s = Sintetica::nova(
            "110.00",
            vec![produto(1, "COCA-COLA KS 290ML", "100.00")],
            vec![duplicata("001", "110.00")],
        );
        s.nfe.valor_frete = dec("10.00");
        s.nfe.valor_tributos_inclusos = Dado::ausente(
            "nfe:total/ICMSTot",
            StatusDado::Mancante,
            "ICMSTot sem as tags de tributos",
        )
        .unwrap();

        let alocacao = alocar(&s.importada()).unwrap();
        let r = &alocacao.reconciliacao;
        // Nessun errore di dominio: il Dado assente e' saltato, non azzerato.
        let tributos = r
            .componenti
            .iter()
            .find(|c| c.nome == "tributos_inclusos")
            .unwrap();
        assert_eq!(tributos.valor.valor(), None);
        assert_eq!(tributos.valor.status(), StatusDado::Mancante);
        // E la differenza non viene dichiarata spiegata.
        assert_eq!(r.esito_componenti, EsitoComponenti::Parcial);
        assert_eq!(
            por_conta(&alocacao, CONTA_ACESSORIOS).status_revisao,
            StatusRevisao::Revisar
        );
    }

    // -----------------------------------------------------------------
    // Identificativi (D5)
    // -----------------------------------------------------------------

    #[test]
    fn chave_vazia_nao_produz_lancamento() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.chave = String::new();
        let erro = alocar(&s.importada()).unwrap_err();
        assert!(
            matches!(
                erro,
                ErroAlocacao::IdentidadeInsuficiente { campo: "chave", .. }
            ),
            "{erro}"
        );
    }

    #[test]
    fn numero_vazio_marca_revisar() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.numero = String::new();
        let alocacao = alocar(&s.importada()).unwrap();
        assert_eq!(alocacao.lancamentos.len(), 1);
        assert_eq!(
            alocacao.lancamentos[0].status_revisao,
            StatusRevisao::Revisar
        );
        assert!(
            alocacao.lancamentos[0]
                .alertas
                .contains(avisos::DOCUMENTO_SEM_NUMERO)
        );
    }

    #[test]
    fn fornecedor_sem_identidade_marca_revisar() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.cnpj_emitente = String::new();
        let alocacao = alocar(&s.importada()).unwrap();
        assert!(
            alocacao.lancamentos[0]
                .alertas
                .contains(avisos::FORNECEDOR_SEM_IDENTIDADE)
        );
        assert_eq!(
            alocacao.lancamentos[0].status_revisao,
            StatusRevisao::Revisar
        );
    }

    #[test]
    fn empresa_sem_identidade_marca_revisar() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.nome_destinatario = "   ".to_string();
        let alocacao = alocar(&s.importada()).unwrap();
        assert!(
            alocacao.lancamentos[0]
                .alertas
                .contains(avisos::EMPRESA_SEM_IDENTIDADE)
        );
    }

    // -----------------------------------------------------------------
    // Hash (D4)
    // -----------------------------------------------------------------

    #[test]
    fn hash_do_campione_canonico_corresponde_ao_payload() {
        let alocacao = alocar(&campione()).unwrap();
        assert_eq!(
            por_conta(&alocacao, "2.01.004").hash_registro,
            "56a719b4f1a1d43673094c8ec65c75a0757fe59d693f48168d229485fb535a53"
        );
        assert_eq!(
            por_conta(&alocacao, "2.01.002").hash_registro,
            "d0438efdb6cd9640c42ae97303654635ecf9df2393b2cd489f49caacc33b1edf"
        );
        assert_eq!(
            por_conta(&alocacao, "2.04.001").hash_registro,
            "8dcb5b1b8f413b766981dba322ee62c454c83e71cb18f810fc44e79e6fe111c8"
        );
    }

    #[test]
    fn sha256_corresponde_ao_vetor_fips() {
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_estavel_na_reimportacao() {
        let primeiro = alocar(&campione()).unwrap();
        let segundo = alocar(&campione()).unwrap();
        let hashes = |a: &AlocacaoNfe| -> Vec<String> {
            a.lancamentos
                .iter()
                .map(|l| l.hash_registro.clone())
                .collect()
        };
        assert_eq!(hashes(&primeiro), hashes(&segundo));
    }

    #[test]
    fn payload_duplicado_no_documento_gera_aviso() {
        // Un articolo classificato sul conto degli accessori, piu' una
        // differenza di totale sullo stesso conto e dello stesso valore.
        let mut s = Sintetica::nova(
            "20.00",
            vec![produto(1, "FRETE SOBRE COMPRAS", "10.00")],
            vec![],
        );
        s.nfe.valor_frete = dec("10.00");

        let classificador_acessorios = classificador_para_conta(
            CONTA_ACESSORIOS,
            "Fretes, Seguros e Outras Despesas sobre Compras",
        );
        let alocacao = alocar_nfe(&s.importada(), &classificador_acessorios, &plano()).unwrap();

        assert_eq!(alocacao.lancamentos.len(), 2);
        assert_eq!(
            alocacao.lancamentos[0].hash_registro,
            alocacao.lancamentos[1].hash_registro
        );
        for lancamento in &alocacao.lancamentos {
            assert!(
                lancamento
                    .alertas
                    .contains(avisos::HASH_DUPLICADO_NO_DOCUMENTO),
                "{}",
                lancamento.alertas
            );
            assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
        }
        assert!(
            alocacao
                .reconciliacao
                .avisos_documento
                .iter()
                .any(|a| a.codigo == avisos::HASH_DUPLICADO_NO_DOCUMENTO)
        );
    }

    /// Classificatore di prova che manda tutto su un conto scelto.
    ///
    /// Serve a costruire il caso del payload ripetuto, che la configurazione
    /// canonica non produce: non e' una nuova fonte di regole.
    fn classificador_para_conta(conta_id: &str, conta_nome: &str) -> Classificador {
        let yaml = format!(
            r#"versao: "1.0"
idioma: pt-BR
descricao: Regras de teste do alocador, nao sao fonte canonica
normalizacao:
  minusculas: true
  remover_acentos: true
  remover_pontuacao: true
  compactar_espacos: true
regras:
  - conta_id: "{conta_id}"
    conta_nome: "{conta_nome}"
    prioridade: 100
    aliases: ["frete"]
conta_fallback_id: "13.01.001"
conta_fallback_nome: "Revisar - Produto Nao Classificado"
"#
        );
        Classificador::compilar(
            &RegrasClassificacao::de_texto(&yaml, "teste://conta-unica").unwrap(),
        )
    }

    // -----------------------------------------------------------------
    // Classificazione
    // -----------------------------------------------------------------

    #[test]
    fn produto_nao_classificado_vai_para_conta_de_revisao() {
        let s = Sintetica::nova(
            "12.00",
            vec![produto(1, "PRODUTO XYZ DESCONHECIDO", "12.00")],
            vec![duplicata("001", "12.00")],
        );
        let alocacao = alocar(&s.importada()).unwrap();
        assert_eq!(alocacao.lancamentos.len(), 1);
        let lancamento = &alocacao.lancamentos[0];
        assert_eq!(lancamento.conta_id, "13.01.001");
        assert_eq!(lancamento.tipo_lancamento, TipoConta::Revisao);
        assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
        assert!(lancamento.alertas.contains(avisos::NAO_CLASSIFICADO));
    }

    #[test]
    fn classificacao_ambigua_marca_revisar_preservando_a_conta() {
        let s = Sintetica::nova("10.00", vec![produto(1, "PAPEL MANTEIGA", "10.00")], vec![]);
        let alocacao = alocar(&s.importada()).unwrap();
        let lancamento = &alocacao.lancamentos[0];
        // F3A-13b: la conta vincente resta, non diventa un conto di revisione.
        assert_eq!(lancamento.conta_id, "2.02.006");
        assert!(!lancamento.conta_id.starts_with("13."));
        assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
        assert!(
            lancamento.alertas.contains(avisos::CLASSIFICACAO_AMBIGUA),
            "{}",
            lancamento.alertas
        );
    }

    /// F3A-24: articolo senza descrizione conservato, con avviso.
    #[test]
    fn item_sem_descricao_e_conservado_com_aviso() {
        let s = Sintetica::nova("12.00", vec![produto(1, "", "12.00")], vec![]);
        let alocacao = alocar(&s.importada()).unwrap();
        let lancamento = &alocacao.lancamentos[0];
        assert_eq!(lancamento.valor, dec("12.00"), "o importo resta");
        assert_eq!(
            lancamento.descricao_itens, " (12.00)",
            "nessuna descrizione inventata"
        );
        assert!(lancamento.alertas.contains(avisos::ITEM_SEM_DESCRICAO));
        assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
    }

    // -----------------------------------------------------------------
    // Campi rinviati, copertura, C10
    // -----------------------------------------------------------------

    #[test]
    fn campi_rinviati_expone_centro_custo_e_observacoes() {
        let alocacao = alocar(&campione()).unwrap();
        let campi = &alocacao.reconciliacao.campi_rinviati;
        for campo in [
            "dre_linha_id",
            "dre_linha_nome",
            "centro_custo",
            "observacoes",
        ] {
            let voce = campi
                .iter()
                .find(|c| c.campo == campo)
                .unwrap_or_else(|| panic!("campo {campo} sem voce"));
            assert!(!voce.status.is_empty());
            assert!(!voce.motivo.is_empty());
        }
        // E i campi corrispondenti restano vuoti in ogni registrazione.
        for lancamento in &alocacao.lancamentos {
            assert!(lancamento.dre_linha_id.is_empty());
            assert!(lancamento.centro_custo.is_empty());
            assert!(lancamento.observacoes.is_empty());
        }
    }

    #[test]
    fn dre_fica_nao_assegnato_con_motivo() {
        let alocacao = alocar(&campione()).unwrap();
        let dre = alocacao
            .reconciliacao
            .campi_rinviati
            .iter()
            .find(|c| c.campo == "dre_linha_id")
            .unwrap();
        assert_eq!(dre.status, "NAO_ASSEGNATO");
        assert!(dre.motivo.contains("3D"), "{}", dre.motivo);
        assert!(
            !alocacao
                .reconciliacao
                .avisos_documento
                .iter()
                .any(|a| a.codigo.contains("DRE")),
            "la 3B non emette avvisi sul mapping DRE"
        );
    }

    #[test]
    fn data_pagamento_fica_ausente_com_motivo() {
        let alocacao = alocar(&campione()).unwrap();
        let data = &alocacao.lancamentos[0].data_pagamento;
        assert_eq!(data.valor(), None);
        assert_eq!(data.status(), StatusDado::NaoDisponivel);
        assert!(data.motivo().unwrap().contains("vencimentos"));
    }

    #[test]
    fn copertura_parcial_com_det_sem_prod() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.avisos.push(aviso("DET_SEM_PROD", "det 2 sem bloco prod"));
        let r = alocar(&s.importada()).unwrap().reconciliacao;

        assert_eq!(r.esito_copertura, EsitoCopertura::Parcial);
        assert_eq!(r.articoli_allocati, 1);
        assert_eq!(r.articoli_attesi.valor(), None);
        assert_eq!(r.articoli_attesi.status(), StatusDado::NaoDisponivel);
        assert_eq!(r.articoli_scartati.len(), 1);
        assert_eq!(r.stato_documento, StatoDocumento::Parcial);
    }

    #[test]
    fn documento_sem_itens_nao_produz_lancamentos() {
        let s = Sintetica::nova("10.00", vec![], vec![]);
        let alocacao = alocar(&s.importada()).unwrap();
        assert!(alocacao.lancamentos.is_empty());
        let r = &alocacao.reconciliacao;
        assert_eq!(r.esito_total, EsitoTotal::Erro);
        assert_eq!(r.esito_copertura, EsitoCopertura::Parcial);
        assert!(
            r.avisos_documento
                .iter()
                .any(|a| a.codigo == avisos::DOCUMENTO_SEM_ITENS)
        );
        assert_eq!(r.stato_documento, StatoDocumento::Erro);
    }

    /// C10 resta vigente: una NF-e senza `<fat>` e' allocabile.
    #[test]
    fn nfe_sem_fat_continua_alocavel() {
        let importada = ler_nfe(raiz().join("tests/fixtures/nfe_sem_fat.xml")).unwrap();
        let alocacao = alocar(&importada).unwrap();
        assert_eq!(alocacao.lancamentos.len(), 3);
        assert_eq!(alocacao.reconciliacao.esito_total, EsitoTotal::Ok);
        let cobranca = alocacao
            .reconciliacao
            .componenti
            .iter()
            .find(|c| c.nome == "valor_cobranca")
            .unwrap();
        assert_eq!(cobranca.valor.valor(), None);
        assert_eq!(cobranca.valor.status(), StatusDado::NaoDisponivel);
        // Le due rate restano, e il riparto usa vNF.
        assert_eq!(por_conta(&alocacao, "2.01.004").duplicatas.len(), 2);
    }

    /// F3A-20: i conti di aggiustamento vengono risolti dal piano, senza
    /// fallback silenzioso. Nel piano canonico esistono entrambi; se mancassero
    /// l'allocazione fallirebbe con [`ErroAlocacao::ContaDeAjusteAusente`].
    #[test]
    fn contas_de_ajuste_existem_no_plano_canonico() {
        let plano = plano();
        let acessorios = plano.por_id(CONTA_ACESSORIOS).expect("2.05.001 no plano");
        let descontos = plano.por_id(CONTA_DESCONTOS).expect("2.05.002 no plano");
        assert_eq!(
            acessorios.nome,
            "Fretes, Seguros e Outras Despesas sobre Compras"
        );
        assert_eq!(descontos.nome, "Descontos Obtidos sobre Compras");
    }

    // -----------------------------------------------------------------
    // R3B-01: capacita' numerica. Importi estremi ma legittimi secondo D6.
    // -----------------------------------------------------------------

    /// Caso A dell'audit: il prodotto del riparto eccede `Decimal` ma il
    /// risultato finale e' rappresentabile. Prima andava in **panico**; ora il
    /// calcolo in centesimi interi da' le quattro celle attese, come la
    /// funzione originale Python.
    #[test]
    fn riparto_com_importi_estremi_nao_entra_em_panico() {
        let s = Sintetica::nova(
            "1000000000000000.00",
            vec![
                produto(1, "COCA-COLA KS 290ML", "500000000000000.00"),
                produto(2, "VINHO RIO SOL TINTO SECO 750ML", "500000000000000.00"),
            ],
            vec![
                duplicata("001", "500000000000000.00"),
                duplicata("002", "500000000000000.00"),
            ],
        );
        let alocacao = alocar(&s.importada()).unwrap();

        assert_eq!(alocacao.lancamentos.len(), 2);
        for lancamento in &alocacao.lancamentos {
            assert_eq!(lancamento.valor, dec("500000000000000.00"));
            for duplicata in &lancamento.duplicatas {
                assert_eq!(duplicata.valor, dec("250000000000000.00"));
            }
        }
        let r = &alocacao.reconciliacao;
        assert_eq!(r.esito_total, EsitoTotal::Ok);
        assert_eq!(r.esito_rate, EsitoRate::Ok);
        assert_eq!(r.stato_documento, StatoDocumento::Ok);
    }

    /// Caso B del primo audit, rivisto dal secondo: gli importi **sono**
    /// rappresentabili, quindi non ci deve essere nessun errore. Il controllo
    /// vero e' la conservazione del centesimo e il trattamento della differenza.
    #[test]
    fn dois_itens_extremos_em_contas_distintas_preservam_o_centavo() {
        // Vecchio "caso B" dell'audit, ora **corretto**: gli importi sono tutti
        // rappresentabili, quindi l'allocazione riesce e il centesimo di
        // differenza viene registrato invece di sparire.
        let s = Sintetica::nova(
            "79228162514264337593543950335",
            vec![
                produto(1, "COCA-COLA KS 290ML", "79228162514264337593543950335"),
                produto(2, "VINHO RIO SOL TINTO SECO 750ML", "0.01"),
            ],
            vec![],
        );
        let alocacao = alocar(&s.importada()).unwrap();

        // Due gruppi piu' la differenza di totale di -0.01: niente e' perduto.
        assert_eq!(alocacao.lancamentos.len(), 3);
        assert_eq!(
            por_conta(&alocacao, "2.01.004").valor,
            dec("79228162514264337593543950335")
        );
        assert_eq!(por_conta(&alocacao, "2.01.002").valor, dec("0.01"));
        let ajuste = por_conta(&alocacao, CONTA_DESCONTOS);
        assert_eq!(ajuste.valor, dec("-0.01"));
        assert_eq!(ajuste.status_revisao, StatusRevisao::Revisar);
        assert!(ajuste.alertas.contains(avisos::VALOR_INCONSISTENTE));

        let r = &alocacao.reconciliacao;
        assert_eq!(
            r.soma_lancamentos,
            dec("79228162514264337593543950335"),
            "a soma fecha com o total declarado"
        );
        assert_eq!(r.esito_total, EsitoTotal::Ok);
    }

    /// Errore di rappresentazione **vero**: due articoli sullo stesso conto il
    /// cui valore aggregato (MAX + 0.01) non e' rappresentabile in `Decimal`
    /// nemmeno riducendo zeri finali. Nessun centesimo viene scartato.
    #[test]
    fn valor_agregado_nao_representavel_e_erro() {
        // Il totale e' rappresentabile; **il valore del gruppo** no, perche' i
        // due articoli finiscono sullo stesso conto e MAX + 0.01 non esiste in
        // `Decimal`. Nota: `MAX + 0.01` non e' nemmeno scrivibile come totale.
        let s = Sintetica::nova(
            "79228162514264337593543950335",
            vec![
                produto(1, "COCA-COLA KS 290ML", "79228162514264337593543950335"),
                produto(2, "COCA-COLA KS 290ML", "0.01"),
            ],
            vec![],
        );
        let erro = alocar(&s.importada()).unwrap_err();
        match &erro {
            ErroAlocacao::CapacidadeNumericaExcedida { operacao, .. } => {
                assert_eq!(*operacao, "valor do grupo");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(
            erro.to_string().contains("nem reduzindo zeros finais"),
            "{erro}"
        );
        assert!(
            erro.to_string().contains("Nenhum centavo foi descartado"),
            "{erro}"
        );
    }

    /// R3B-01 residuo A: `i128::MIN / -1` non e' rappresentabile. Prima questo
    /// documento faceva **panico** nella divisione del riparto; ora esce un
    /// errore tipizzato. Importi legittimi per D6: nessun filtro di segno e'
    /// stato introdotto per evitare il caso.
    #[test]
    fn divisao_do_rateio_nao_entra_em_panico_com_minimo() {
        let s = Sintetica::nova(
            "-0.01",
            vec![
                produto(1, "COCA-COLA KS 290ML", "-92233720368547758.08"),
                produto(2, "VINHO RIO SOL TINTO SECO 750ML", "92233720368547758.07"),
            ],
            vec![
                duplicata("001", "184467440737095516.16"),
                duplicata("002", "-184467440737095516.17"),
            ],
        );
        let erro = alocar(&s.importada()).unwrap_err();
        match &erro {
            ErroAlocacao::CapacidadeNumericaExcedida { operacao, .. } => {
                assert_eq!(*operacao, "divisao do rateio");
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(erro.to_string().contains("representavel em i128"), "{erro}");
    }

    /// Round-trip dei centesimi: il valore massimo e il suo simmetrico negativo
    /// sono esatti, e un centesimo non eliminabile resta un errore.
    #[test]
    fn round_trip_de_centesimos_respeita_o_dominio_de_decimal() {
        let origem = "teste://capacidade";
        for bruto in [
            "79228162514264337593543950335",
            "-79228162514264337593543950335",
            "792281625142643375935439503.35",
            "10.00",
            "-0.01",
            "0.00",
        ] {
            let valor = dec(bruto);
            let cent = em_centesimos(valor, "teste", origem).unwrap();
            let volta = de_centesimos(cent, "round-trip", origem).unwrap();
            assert_eq!(volta, valor, "round-trip de {bruto}");
        }

        // MAX + 0.01 in centesimi: la cifra finale non e' zero, non si puo'
        // ridurre la scala e non si arrotonda.
        let maximo_cent =
            em_centesimos(dec("79228162514264337593543950335"), "teste", origem).unwrap();
        assert!(de_centesimos(maximo_cent + 1, "teste", origem).is_err());
    }

    /// Perche' l'aritmetica non passa da `Decimal`: la somma perde un centesimo
    /// **anche con `checked_add`**, che restituisce `Some` col valore
    /// arrotondato. Questo test protegge la scelta dei centesimi interi.
    #[test]
    fn decimal_perde_centavo_na_soma_extrema() {
        let maximo = dec("79228162514264337593543950335");
        let centavo = dec("0.01");
        assert_eq!(maximo + centavo, maximo, "a soma Decimal perde o centavo");
        assert_eq!(
            maximo.checked_add(centavo),
            Some(maximo),
            "checked_add nao sinaliza a perda"
        );
        // In centesimi interi lo stesso calcolo e' esatto, e l'incapacita' di
        // rappresentare il risultato diventa errore, non perdita.
        let em_cent = em_centesimos(maximo, "teste", "teste://capacidade").unwrap();
        let soma = em_cent.checked_add(1).unwrap();
        assert!(de_centesimos(soma, "teste", "teste://capacidade").is_err());
    }

    /// E il prodotto `Decimal` del riparto va in overflow dove i centesimi
    /// interi non hanno problemi.
    #[test]
    fn decimal_estoura_no_produto_do_rateio() {
        let meio = dec("500000000000000.00");
        assert_eq!(meio.checked_mul(meio), None, "produto Decimal nao cabe");
        let cent = em_centesimos(meio, "teste", "teste://capacidade").unwrap();
        assert!(cent.checked_mul(cent).is_some(), "em centavos cabe");
    }

    // -----------------------------------------------------------------
    // R3B-02: la descrizione porta sempre due decimali.
    // -----------------------------------------------------------------

    #[test]
    fn descricao_usa_sempre_duas_casas_decimais() {
        for bruto in ["10", "10.0", "10.00", "10.0000"] {
            let s = Sintetica::nova(
                "10.00",
                vec![produto(1, "COCA-COLA KS 290ML", bruto)],
                vec![],
            );
            let alocacao = alocar(&s.importada()).unwrap();
            assert_eq!(
                alocacao.lancamentos[0].descricao_itens, "COCA-COLA KS 290ML (10.00)",
                "vProd de entrada {bruto}"
            );
            assert_eq!(alocacao.lancamentos[0].valor, dec("10.00"));
        }
    }

    #[test]
    fn descricao_vazia_mantem_duas_casas_e_o_aviso() {
        let s = Sintetica::nova("12.00", vec![produto(1, "", "12")], vec![]);
        let alocacao = alocar(&s.importada()).unwrap();
        let lancamento = &alocacao.lancamentos[0];
        assert_eq!(lancamento.descricao_itens, " (12.00)");
        assert!(lancamento.alertas.contains(avisos::ITEM_SEM_DESCRICAO));
        assert_eq!(lancamento.status_revisao, StatusRevisao::Revisar);
    }

    // -----------------------------------------------------------------
    // R3B-03: propagazione degli errori attraverso l'API pubblica.
    // -----------------------------------------------------------------

    #[test]
    fn classificacao_com_conta_inexistente_propaga_erro() {
        let s = Sintetica::nova(
            "10.00",
            vec![produto(1, "FRETE SOBRE COMPRAS", "10.00")],
            vec![],
        );
        // Regola di prova che punta a un conto che il piano canonico non ha.
        let classificador_fantasma = classificador_para_conta("9.99.999", "Conta inexistente");
        let erro = alocar_nfe(&s.importada(), &classificador_fantasma, &plano()).unwrap_err();
        match &erro {
            ErroAlocacao::ClassificacaoFalhou { erro: interno, .. } => {
                assert_eq!(interno.conta_id, "9.99.999");
                assert!(!interno.fonte.as_os_str().is_empty());
            }
            outro => panic!("erro inesperado: {outro}"),
        }
        assert!(erro.to_string().contains("9.99.999"), "{erro}");
    }

    #[test]
    fn competencia_fora_do_intervalo_propaga_erro() {
        let mut s = Sintetica::nova(
            "10.00",
            vec![produto(1, "COCA-COLA KS 290ML", "10.00")],
            vec![],
        );
        s.nfe.data_emissao = NaiveDate::from_ymd_opt(1800, 1, 1).unwrap();
        let erro = alocar(&s.importada()).unwrap_err();
        assert!(
            matches!(erro, ErroAlocacao::CompetenciaInvalida { .. }),
            "{erro}"
        );
        assert!(erro.to_string().contains("1800"), "{erro}");
    }

    /// Entrambi i segni della differenza, sul piano di prova senza i conti di
    /// aggiustamento: nessun fallback silenzioso, errore che nomina il conto.
    #[test]
    fn conta_de_ajuste_ausente_propaga_erro() {
        let plano_sem_ajuste =
            PlanoContas::carregar(raiz().join("tests/fixtures/plano_sem_contas_de_ajuste.yaml"))
                .unwrap();
        let classificador_teste = classificador_para_conta("2.01.001", "Cervejas");
        assert!(plano_sem_ajuste.por_id(CONTA_ACESSORIOS).is_none());
        assert!(plano_sem_ajuste.por_id(CONTA_DESCONTOS).is_none());

        for (total, frete, desconto, conta_esperada) in [
            ("110.00", "10.00", "0.00", CONTA_ACESSORIOS),
            ("90.00", "0.00", "10.00", CONTA_DESCONTOS),
        ] {
            let mut s = Sintetica::nova(
                total,
                vec![produto(1, "CERVEJA HEINEKEN LONG NECK 330ML", "100.00")],
                vec![],
            );
            s.nfe.valor_frete = dec(frete);
            s.nfe.valor_desconto = dec(desconto);

            let erro =
                alocar_nfe(&s.importada(), &classificador_teste, &plano_sem_ajuste).unwrap_err();
            match &erro {
                ErroAlocacao::ContaDeAjusteAusente { conta_id, .. } => {
                    assert_eq!(conta_id, conta_esperada, "total {total}");
                }
                outro => panic!("erro inesperado para total {total}: {outro}"),
            }
            assert!(erro.to_string().contains(conta_esperada), "{erro}");
        }
    }
}

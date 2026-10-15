//! Modello dati di dominio: NF-e, prodotti, lancamentos contabili.
//!
//! Porting di `src/fantozzi/models.py` (repo Python `Programma_registrazioni-contabili`).
//!
//! Contratti applicati qui:
//! - C1: ogni valore monetario e' `Decimal`, mai `f64`.
//! - C2: dato mancante != zero. L'assenza e' rappresentata da [`Dado`], che non
//!   puo' esistere in uno stato incoerente: la validazione e' nei costruttori e
//!   nella deserializzazione, non in `debug_assert`.
//! - C4: le stringhe-magiche di Python diventano enum chiusi: un valore
//!   sconosciuto nello YAML/XML e' un errore di deserializzazione, non un default.
//! - C8: i tipi validati ([`Competencia`], [`Dado`]) hanno campi privati e un
//!   solo percorso di validazione, condiviso da costruttori e serde.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Contratto C2 - stato di un dato importante.
///
/// Un dato senza stato non e' affidabile. Zero non e' uno stato: se il valore
/// manca si usa `Mancante`/`NaoDisponivel`, mai `Decimal::ZERO`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusDado {
    /// Valore presente e verificato alla fonte.
    Ok,
    /// La fonte doveva contenere il dato e non lo contiene.
    #[default]
    Mancante,
    /// Valore presente ma non ancora confrontato con la fonte canonica.
    NaoVerificado,
    /// La fonte non espone il dato (non e' un errore).
    NaoDisponivel,
    /// Valore presente ma con copertura incompleta del periodo o dell'insieme.
    Parcial,
    /// Valore derivato da stima dichiarata, non da misura.
    Estimado,
    /// Lettura fallita.
    Erro,
    /// Lettura bloccata da una regola di sicurezza.
    Bloqueado,
    /// Il dato non si applica a questo caso.
    #[serde(rename = "N.A.")]
    NaoAplicavel,
}

impl StatusDado {
    /// `true` se lo stato afferma che il valore **non** c'e'.
    ///
    /// E' l'unica regola che lega stato e valore: `Parcial` e `Estimado`
    /// descrivono un valore presente ma di qualita' dichiarata, quindi non
    /// implicano assenza.
    pub const fn implica_ausencia(self) -> bool {
        matches!(
            self,
            StatusDado::Mancante
                | StatusDado::NaoDisponivel
                | StatusDado::Erro
                | StatusDado::Bloqueado
                | StatusDado::NaoAplicavel
        )
    }
}

/// Violazione della carta d'identita' di un [`Dado`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroDado {
    /// Un dato senza fonte non e' verificabile.
    FonteVazia,
    /// Lo stato afferma presenza e il valore manca, o viceversa.
    EstadoIncoerente { status: StatusDado, tem_valor: bool },
    /// Un dato assente senza motivo non e' auditabile.
    MotivoAusente { status: StatusDado },
    /// Motivo presente ma vuoto: equivale a non averlo.
    MotivoVazio,
}

impl fmt::Display for ErroDado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroDado::FonteVazia => write!(f, "dado sem fonte declarada"),
            ErroDado::EstadoIncoerente { status, tem_valor } => write!(
                f,
                "status {status:?} e' incoerente com {}",
                if *tem_valor {
                    "um valor presente"
                } else {
                    "a ausencia de valor"
                }
            ),
            ErroDado::MotivoAusente { status } => {
                write!(f, "dado ausente com status {status:?} exige motivo")
            }
            ErroDado::MotivoVazio => write!(f, "motivo declarado mas vazio"),
        }
    }
}

impl std::error::Error for ErroDado {}

/// Dato con carta d'identita': valore, fonte, stato, motivo.
///
/// Contratto C2/C8: i campi sono privati e ogni percorso di costruzione
/// (costruttori e `Deserialize`) passa per [`Dado::novo`]. Non esiste un
/// `Dado` che dichiari `OK` senza valore, o un valore senza fonte.
///
/// Usato per i campi dove l'assenza va distinta dallo zero e deve restare
/// leggibile nell'output. Non va usato per i campi sempre presenti.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Dado<T> {
    valor: Option<T>,
    fonte: String,
    status: StatusDado,
    #[serde(skip_serializing_if = "Option::is_none")]
    motivo: Option<String>,
}

impl<T> Dado<T> {
    /// Unico percorso di validazione: costruttori e serde passano di qui.
    pub fn novo(
        valor: Option<T>,
        fonte: impl Into<String>,
        status: StatusDado,
        motivo: Option<String>,
    ) -> Result<Self, ErroDado> {
        let fonte: String = fonte.into();
        if fonte.trim().is_empty() {
            return Err(ErroDado::FonteVazia);
        }
        if status.implica_ausencia() != valor.is_none() {
            return Err(ErroDado::EstadoIncoerente {
                status,
                tem_valor: valor.is_some(),
            });
        }
        match &motivo {
            Some(m) if m.trim().is_empty() => return Err(ErroDado::MotivoVazio),
            None if valor.is_none() => return Err(ErroDado::MotivoAusente { status }),
            _ => {}
        }
        Ok(Self {
            valor,
            fonte,
            status,
            motivo,
        })
    }

    /// Dato letto con successo dalla fonte indicata.
    pub fn presente(valor: T, fonte: impl Into<String>) -> Result<Self, ErroDado> {
        Self::novo(Some(valor), fonte, StatusDado::Ok, None)
    }

    /// Dato assente: niente zero tecnico, lo stato e il motivo restano visibili.
    ///
    /// Fallisce se `status` non e' uno stato di assenza (vedi
    /// [`StatusDado::implica_ausencia`]).
    pub fn ausente(
        fonte: impl Into<String>,
        status: StatusDado,
        motivo: impl Into<String>,
    ) -> Result<Self, ErroDado> {
        Self::novo(None, fonte, status, Some(motivo.into()))
    }

    pub fn valor(&self) -> Option<&T> {
        self.valor.as_ref()
    }

    pub fn fonte(&self) -> &str {
        &self.fonte
    }

    pub fn status(&self) -> StatusDado {
        self.status
    }

    pub fn motivo(&self) -> Option<&str> {
        self.motivo.as_deref()
    }

    /// `true` solo per un dato presente e verificato alla fonte.
    pub fn is_ok(&self) -> bool {
        self.status == StatusDado::Ok
    }
}

/// Forma grezza usata solo per deserializzare: viene subito validata.
#[derive(Deserialize)]
// Senza questo bound esplicito, `#[serde(default)]` su `valor` farebbe
// richiedere `T: Default`, che qui non serve.
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
struct DadoBruto<T> {
    #[serde(default)]
    valor: Option<T>,
    fonte: String,
    status: StatusDado,
    #[serde(default)]
    motivo: Option<String>,
}

impl<'de, T> Deserialize<'de> for Dado<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bruto = DadoBruto::<T>::deserialize(deserializer)?;
        Dado::novo(bruto.valor, bruto.fonte, bruto.status, bruto.motivo)
            .map_err(serde::de::Error::custom)
    }
}

/// Tipo contabile di una conta del piano gerenziale.
///
/// Valori ammessi: quelli presenti in `config/plano_contas_gerencial.yaml`.
/// Un `tipo` non previsto fa fallire il caricamento (contratto C4).
///
/// E' anche il tipo di `Lancamento::tipo_lancamento`: vedi la nota la'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoConta {
    Receita,
    DeducaoReceita,
    Custo,
    Despesa,
    Imposto,
    Investimento,
    Passivo,
    Patrimonial,
    Provisao,
    Ajuste,
    /// Conte 13.01.* usate quando la classificazione non e' conclusa.
    Revisao,
}

impl TipoConta {
    /// Tutte le varianti, per i test di copertura e per l'audit.
    pub const TODOS: [TipoConta; 11] = [
        TipoConta::Receita,
        TipoConta::DeducaoReceita,
        TipoConta::Custo,
        TipoConta::Despesa,
        TipoConta::Imposto,
        TipoConta::Investimento,
        TipoConta::Passivo,
        TipoConta::Patrimonial,
        TipoConta::Provisao,
        TipoConta::Ajuste,
        TipoConta::Revisao,
    ];
}

/// Conta del piano dei conti gerenziale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conta {
    pub id: String,
    pub nome: String,
    pub grupo: String,
    pub tipo: TipoConta,
    /// Default dichiarato: una conta senza il campo non esige revisione manuale.
    #[serde(default)]
    pub exige_revisao: bool,
}

/// Centro di costo (`centros_custo` nello YAML del piano conti).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CentroCusto {
    pub id: String,
    pub nome: String,
}

/// Competenza invalida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroCompetencia {
    /// La stringa non ha la forma `YYYY-MM`.
    Formato(String),
    Mes(u32),
    /// Fuori dall'intervallo dichiarato in [`Competencia::ANO_MIN`]..=[`Competencia::ANO_MAX`].
    Ano(i32),
}

impl fmt::Display for ErroCompetencia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroCompetencia::Formato(s) => {
                write!(f, "competencia invalida (esperado YYYY-MM): {s:?}")
            }
            ErroCompetencia::Mes(m) => write!(f, "mes fora do intervalo 1..=12: {m}"),
            ErroCompetencia::Ano(a) => write!(
                f,
                "ano fora do intervalo {}..={}: {a}",
                Competencia::ANO_MIN,
                Competencia::ANO_MAX
            ),
        }
    }
}

impl std::error::Error for ErroCompetencia {}

/// Competenza contabile `YYYY-MM`.
///
/// Contratto C8: campi privati e un solo percorso di validazione
/// ([`Competencia::new`]). Non esiste una `Competencia` con mese 13, ne' una
/// che si serializzi in una stringa che il deserializzatore rifiuterebbe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Competencia {
    // L'ordine dei campi determina il derive di Ord: ano prima di mes.
    ano: i32,
    mes: u32,
}

impl Competencia {
    /// Limite inferiore dichiarato: prima non esistono documenti del ristorante.
    pub const ANO_MIN: i32 = 1900;
    /// Limite superiore dichiarato: oltre, e' quasi certamente un errore di data.
    pub const ANO_MAX: i32 = 2999;

    pub fn new(ano: i32, mes: u32) -> Result<Self, ErroCompetencia> {
        if !(1..=12).contains(&mes) {
            return Err(ErroCompetencia::Mes(mes));
        }
        if !(Self::ANO_MIN..=Self::ANO_MAX).contains(&ano) {
            return Err(ErroCompetencia::Ano(ano));
        }
        Ok(Self { ano, mes })
    }

    /// Competenza della data del documento.
    ///
    /// Passa per [`Competencia::new`]: una data fuori intervallo e' un errore,
    /// non una competenza silenziosamente fuori scala.
    pub fn from_data(data: NaiveDate) -> Result<Self, ErroCompetencia> {
        use chrono::Datelike;
        Self::new(data.year(), data.month())
    }

    pub fn ano(&self) -> i32 {
        self.ano
    }

    pub fn mes(&self) -> u32 {
        self.mes
    }
}

impl fmt::Display for Competencia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.ano, self.mes)
    }
}

impl FromStr for Competencia {
    type Err = ErroCompetencia;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalida = || ErroCompetencia::Formato(s.to_string());
        let (ano, mes) = s.split_once('-').ok_or_else(invalida)?;
        if ano.len() != 4 || mes.len() != 2 {
            return Err(invalida());
        }
        let ano: i32 = ano.parse().map_err(|_| invalida())?;
        let mes: u32 = mes.parse().map_err(|_| invalida())?;
        Competencia::new(ano, mes)
    }
}

impl TryFrom<String> for Competencia {
    type Error = ErroCompetencia;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<Competencia> for String {
    fn from(value: Competencia) -> Self {
        value.to_string()
    }
}

/// Duplicata (rata) di una NF-e o di un lancamento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duplicata {
    pub numero: String,
    /// `None` quando la NF-e non porta la data: non si inventa una scadenza.
    pub vencimento: Option<NaiveDate>,
    pub valor: Decimal,
}

/// Riga prodotto di una NF-e (`det/prod`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProdutoNfe {
    pub n_item: u32,
    pub codigo: String,
    pub descricao: String,
    pub ncm: String,
    pub cfop: String,
    pub unidade: String,
    pub quantidade: Decimal,
    pub valor_unitario: Decimal,
    pub valor_total: Decimal,
}

/// Nota Fiscal Eletronica importata da XML.
///
/// I totali valgono zero solo quando la NF-e li dichiara a zero; l'assenza del
/// blocco `ICMSTot` deve essere segnalata dal parser (fase 2) e non convertita
/// silenziosamente in zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nfe {
    pub chave: String,
    pub numero: String,
    pub serie: String,
    pub data_emissao: NaiveDate,
    pub cnpj_emitente: String,
    pub nome_emitente: String,
    pub cnpj_destinatario: String,
    pub nome_destinatario: String,
    pub valor_total: Decimal,
    pub natureza_operacao: String,
    pub valor_produtos: Decimal,
    pub valor_frete: Decimal,
    pub valor_seguro: Decimal,
    pub valor_desconto: Decimal,
    pub valor_outras_despesas: Decimal,
    pub valor_tributos_inclusos: Decimal,
    pub produtos: Vec<ProdutoNfe>,
    pub duplicatas: Vec<Duplicata>,
    pub tem_duplicatas: bool,
    pub tem_cobranca: bool,
    pub valor_cobranca: Decimal,
    pub arquivo_origem: String,
}

/// Esito della classificazione di una descrizione prodotto/despesa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confianca {
    /// Una sola conta candidata.
    Ok,
    /// Piu' conte candidate: si usa la priorita' maggiore ma si registra l'alerta.
    Ambiguo,
    /// Nessun alias corrisponde: va alla conta di fallback di revisione.
    NaoClassificado,
}

/// Risultato della classificazione, con tracciabilita' dei match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultadoClassificacao {
    pub conta: Conta,
    pub confianca: Confianca,
    pub alertas: Vec<String>,
    /// Alias che hanno prodotto il match: senza questo l'esito non e' verificabile.
    pub matches: Vec<String>,
}

/// Origine di un lancamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrigemLancamento {
    #[default]
    XmlNfe,
    Csv,
    Manual,
}

/// Stato di revisione di un lancamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusRevisao {
    #[default]
    Ok,
    /// Richiede intervento umano prima della contabilizzazione.
    Revisar,
    /// Revisionato e approvato.
    Aprovado,
}

/// Riga del pre-libro giornale: l'unita' di output del sistema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lancamento {
    pub data_documento: NaiveDate,
    pub competencia: Competencia,
    pub cnpj_empresa: String,
    pub nome_empresa: String,
    pub fornecedor_ou_favorecido: String,
    pub cnpj_cpf_favorecido: String,
    pub documento: String,
    pub tipo_documento: String,
    pub conta_id: String,
    pub conta_nome: String,
    pub grupo_conta: String,
    pub descricao_itens: String,
    pub valor: Decimal,
    pub duplicatas: Vec<Duplicata>,
    pub origem: OrigemLancamento,
    pub arquivo_origem: String,
    pub chave_origem: String,
    pub hash_registro: String,
    pub status_revisao: StatusRevisao,
    pub alertas: String,
    pub observacoes: String,
    pub centro_custo: String,
    /// Propagato da `Conta::tipo` (porting di `allocation.py`, dove
    /// `tipo_lancamento=conta.tipo` in tre punti).
    ///
    /// Per questo e' [`TipoConta`] e non un enum separato: un enum piu' stretto
    /// renderebbe non rappresentabili i lancamentos verso conte `revisao`,
    /// `passivo`, `provisao`, `patrimonial` e `deducao_receita`, che sono
    /// esattamente i casi che non devono sparire.
    pub tipo_lancamento: TipoConta,
    pub dre_linha_id: String,
    pub dre_linha_nome: String,
    /// Contratto C2: non pagato != pagato a zero.
    pub data_pagamento: Dado<NaiveDate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------- C1

    /// I valori monetari non devono soffrire dell'errore binario di f64.
    #[test]
    fn decimal_nao_introduz_erro_binario() {
        let a = Decimal::from_str_exact("0.1").unwrap();
        let b = Decimal::from_str_exact("0.2").unwrap();
        assert_eq!(a + b, Decimal::from_str_exact("0.3").unwrap());
        // controprova: con f64 la stessa somma non e' esatta
        assert_ne!(0.1_f64 + 0.2_f64, 0.3_f64);
    }

    /// Le scale decimali della NF-e (4 decimali) sopravvivono al round-trip.
    #[test]
    fn decimal_sobrevive_round_trip() {
        let v = Decimal::from_str_exact("1234.5678").unwrap();
        let yaml = serde_yaml::to_string(&v).unwrap();
        let back: Decimal = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(v, back);
    }

    // ---------------------------------------------------------- C2 / C8

    /// L'assenza di pagamento non e' un pagamento di zero.
    #[test]
    fn dado_ausente_nao_vira_zero() {
        let pago: Dado<NaiveDate> = Dado::ausente(
            "pre_livro_diario",
            StatusDado::NaoDisponivel,
            "titulo em aberto",
        )
        .unwrap();
        assert!(!pago.is_ok());
        assert_eq!(pago.valor(), None);
        assert_eq!(pago.status(), StatusDado::NaoDisponivel);
        assert_eq!(pago.motivo(), Some("titulo em aberto"));
    }

    /// Un dato senza fonte non e' verificabile: non deve essere costruibile.
    #[test]
    fn dado_exige_fonte_nao_vazia() {
        assert_eq!(Dado::presente(42, "").unwrap_err(), ErroDado::FonteVazia);
        assert_eq!(Dado::presente(42, "   ").unwrap_err(), ErroDado::FonteVazia);
        assert_eq!(
            Dado::<i32>::ausente("", StatusDado::Mancante, "motivo").unwrap_err(),
            ErroDado::FonteVazia
        );
    }

    /// `ausente` con status di presenza e' una contraddizione, non un avviso.
    #[test]
    fn dado_ausente_rejeita_status_de_presenca() {
        for status in [StatusDado::Ok, StatusDado::Parcial, StatusDado::Estimado] {
            assert!(
                Dado::<i32>::ausente("fonte", status, "motivo").is_err(),
                "status {status:?} nao pode descrever um dado ausente"
            );
        }
    }

    /// Un dato assente senza motivo non e' auditabile.
    #[test]
    fn dado_ausente_exige_motivo() {
        assert_eq!(
            Dado::<i32>::novo(None, "fonte", StatusDado::Mancante, None).unwrap_err(),
            ErroDado::MotivoAusente {
                status: StatusDado::Mancante
            }
        );
        assert_eq!(
            Dado::<i32>::ausente("fonte", StatusDado::Mancante, "  ").unwrap_err(),
            ErroDado::MotivoVazio
        );
    }

    /// Qualita' dichiarata != assenza: un valore stimato o parziale c'e'.
    #[test]
    fn status_de_qualidade_admite_valor_presente() {
        for status in [
            StatusDado::Parcial,
            StatusDado::Estimado,
            StatusDado::NaoVerificado,
        ] {
            let d = Dado::novo(Some(7), "fonte", status, Some("cobertura parcial".into()))
                .unwrap_or_else(|e| panic!("status {status:?} deveria aceitar valor: {e}"));
            assert_eq!(d.valor(), Some(&7));
            assert!(!d.is_ok(), "apenas OK significa verificado na fonte");
        }
    }

    /// Contratto C8: la deserializzazione usa la stessa validazione dei costruttori.
    #[test]
    fn dado_deserializado_incoerente_falha() {
        // OK sem valor
        let yaml = "valor: null\nfonte: x\nstatus: OK\n";
        assert!(serde_yaml::from_str::<Dado<i32>>(yaml).is_err());
        // valor com status de ausencia
        let yaml = "valor: 10\nfonte: x\nstatus: MANCANTE\nmotivo: m\n";
        assert!(serde_yaml::from_str::<Dado<i32>>(yaml).is_err());
        // fonte vazia
        let yaml = "valor: 10\nfonte: ''\nstatus: OK\n";
        assert!(serde_yaml::from_str::<Dado<i32>>(yaml).is_err());
        // ausente sem motivo
        let yaml = "valor: null\nfonte: x\nstatus: MANCANTE\n";
        assert!(serde_yaml::from_str::<Dado<i32>>(yaml).is_err());
        // campo desconhecido (C3)
        let yaml = "valor: 10\nfonte: x\nstatus: OK\nfont: y\n";
        assert!(serde_yaml::from_str::<Dado<i32>>(yaml).is_err());
    }

    #[test]
    fn dado_valido_sobrevive_round_trip() {
        for d in [
            Dado::presente(10, "nfe:vPag").unwrap(),
            Dado::ausente("nfe:vPag", StatusDado::NaoDisponivel, "NF-e sem cobranca").unwrap(),
        ] {
            let yaml = serde_yaml::to_string(&d).unwrap();
            let back: Dado<i32> = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(d, back);
        }
    }

    // ---------------------------------------------------------- C4 / C3

    /// Un `tipo` fuori dall'enum non deve passare come default.
    #[test]
    fn tipo_conta_desconhecido_falha() {
        let yaml = "id: \"9.99.999\"\nnome: X\ngrupo: Y\ntipo: inventado\n";
        assert!(serde_yaml::from_str::<Conta>(yaml).is_err());
    }

    /// Campo non previsto = errore, non silenzio.
    #[test]
    fn conta_com_campo_desconhecido_falha() {
        let yaml = "id: \"1.01.001\"\nnome: X\ngrupo: Y\ntipo: receita\ntpo: receita\n";
        assert!(serde_yaml::from_str::<Conta>(yaml).is_err());
    }

    #[test]
    fn exige_revisao_tem_default_declarado_false() {
        let yaml = "id: \"1.01.001\"\nnome: X\ngrupo: Receitas\ntipo: receita\n";
        let conta: Conta = serde_yaml::from_str(yaml).unwrap();
        assert!(!conta.exige_revisao);
    }

    /// Le 11 varianti coprono esattamente i `tipo` presenti nel piano conti reale
    /// e si serializzano con le stesse stringhe del Python.
    #[test]
    fn tipos_de_conta_correspondem_as_strings_do_python() {
        let esperados = [
            "receita",
            "deducao_receita",
            "custo",
            "despesa",
            "imposto",
            "investimento",
            "passivo",
            "patrimonial",
            "provisao",
            "ajuste",
            "revisao",
        ];
        assert_eq!(TipoConta::TODOS.len(), esperados.len());
        for (tipo, texto) in TipoConta::TODOS.iter().zip(esperados) {
            let yaml = serde_yaml::to_string(tipo).unwrap();
            assert_eq!(yaml.trim(), texto);
            assert_eq!(serde_yaml::from_str::<TipoConta>(texto).unwrap(), *tipo);
        }
    }

    // ------------------------------------------------------------- R2

    fn lancamento_de_exemplo(tipo: TipoConta) -> Lancamento {
        Lancamento {
            data_documento: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: "00000000000191".into(),
            nome_empresa: "Restaurante".into(),
            fornecedor_ou_favorecido: "Fornecedor".into(),
            cnpj_cpf_favorecido: "00000000000272".into(),
            documento: "12566".into(),
            tipo_documento: "NFE".into(),
            conta_id: "13.01.001".into(),
            conta_nome: "Revisar - Produto Nao Classificado".into(),
            grupo_conta: "Contas de Revisao e Classificacao Pendente".into(),
            descricao_itens: "KILOS".into(),
            valor: Decimal::from_str_exact("123.45").unwrap(),
            duplicatas: vec![],
            origem: OrigemLancamento::XmlNfe,
            arquivo_origem: "nfe.xml".into(),
            chave_origem: "35260500000000000191550010000125661000125667".into(),
            hash_registro: "abc".into(),
            status_revisao: StatusRevisao::Revisar,
            alertas: "NAO_CLASSIFICADO".into(),
            observacoes: String::new(),
            centro_custo: "CC-999".into(),
            tipo_lancamento: tipo,
            dre_linha_id: String::new(),
            dre_linha_nome: String::new(),
            data_pagamento: Dado::ausente(
                "pre_livro_diario",
                StatusDado::NaoDisponivel,
                "titulo em aberto",
            )
            .unwrap(),
        }
    }

    /// R2: ogni tipo di conta che il Python propaga deve essere rappresentabile
    /// in `tipo_lancamento`, inclusi revisao, passivo, provisao, patrimonial e
    /// deducao_receita.
    #[test]
    fn todo_tipo_de_conta_e_representavel_como_tipo_de_lancamento() {
        for tipo in TipoConta::TODOS {
            let l = lancamento_de_exemplo(tipo);
            let yaml = serde_yaml::to_string(&l).unwrap();
            let back: Lancamento = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(back.tipo_lancamento, tipo);
        }
    }

    /// Il percorso critico: prodotto non classificato -> conta di revisione ->
    /// lancamento di tipo revisao, che deve sopravvivere alla serializzazione.
    #[test]
    fn lancamento_de_conta_de_revisao_sobrevive_ao_round_trip() {
        let l = lancamento_de_exemplo(TipoConta::Revisao);
        let yaml = serde_yaml::to_string(&l).unwrap();
        assert!(yaml.contains("tipo_lancamento: revisao"), "{yaml}");
        let back: Lancamento = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back, l);
        assert_eq!(back.data_pagamento.status(), StatusDado::NaoDisponivel);
    }

    // ------------------------------------------------------------- C8

    #[test]
    fn competencia_valida_e_formatada_com_zero_a_esquerda() {
        let c: Competencia = "2026-05".parse().unwrap();
        assert_eq!(c, Competencia::new(2026, 5).unwrap());
        assert_eq!(c.to_string(), "2026-05");
        assert_eq!((c.ano(), c.mes()), (2026, 5));
    }

    #[test]
    fn competencia_rejeita_formatos_invalidos() {
        for entrada in [
            "2026-13", "2026-00", "2026/05", "26-05", "2026-5", "", "maio",
        ] {
            assert!(
                entrada.parse::<Competencia>().is_err(),
                "deveria rejeitar {entrada:?}"
            );
        }
    }

    #[test]
    fn competencia_derivada_da_data_do_documento() {
        let d = NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        assert_eq!(Competencia::from_data(d).unwrap().to_string(), "2026-05");
    }

    /// R4: `from_data` passa per la stessa validazione di `new`.
    #[test]
    fn from_data_rejeita_ano_fora_do_intervalo_declarado() {
        let antiga = NaiveDate::from_ymd_opt(1800, 5, 5).unwrap();
        assert_eq!(
            Competencia::from_data(antiga),
            Err(ErroCompetencia::Ano(1800))
        );

        let limite = NaiveDate::from_ymd_opt(Competencia::ANO_MIN, 1, 1).unwrap();
        assert!(Competencia::from_data(limite).is_ok());
    }

    /// R4: tutto cio' che si serializza deve poter essere riletto.
    /// Con campi privati non esiste un percorso per costruire 2026-13.
    #[test]
    fn competencia_round_trip_sempre_valido() {
        for ano in [Competencia::ANO_MIN, 2026, Competencia::ANO_MAX] {
            for mes in 1..=12 {
                let c = Competencia::new(ano, mes).unwrap();
                let texto = c.to_string();
                assert_eq!(texto.parse::<Competencia>().unwrap(), c);
                let yaml = serde_yaml::to_string(&c).unwrap();
                assert_eq!(serde_yaml::from_str::<Competencia>(&yaml).unwrap(), c);
            }
        }
    }

    #[test]
    fn competencia_ordena_por_ano_depois_mes() {
        let mut v = [
            Competencia::new(2026, 1).unwrap(),
            Competencia::new(2025, 12).unwrap(),
            Competencia::new(2026, 2).unwrap(),
        ];
        v.sort();
        assert_eq!(
            v.map(|c| c.to_string()),
            [
                "2025-12".to_string(),
                "2026-01".to_string(),
                "2026-02".to_string()
            ]
        );
    }
}

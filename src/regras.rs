//! Lettura e validazione delle regole versionate in `config/*.yaml`.
//!
//! Porting di `chart_of_accounts.py` e `product_classifier.py` (parte di caricamento).
//!
//! Contratti applicati qui:
//! - C3: `deny_unknown_fields` ovunque. Una chiave sbagliata nello YAML e' un
//!   errore di caricamento, non una regola ignorata in silenzio.
//! - C5: il piano conti viene validato (ID duplicati, campi vuoti, conte di
//!   revisione obbligatorie presenti).
//! - C6: **nuovo rispetto al Python** - ogni `conta_id` citato dalle regole di
//!   classificazione deve esistere nel piano conti, altrimenti la
//!   classificazione fallirebbe solo a runtime, sul dato reale.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::modelos::{CentroCusto, Conta, TipoConta};

/// Conte obbligatorie di destinazione quando la classificazione non conclude.
///
/// Porting di `REVIEW_ACCOUNT_MAP`. Sono un contratto: se una di queste manca
/// dal piano conti, un lancamento non classificato non avrebbe dove andare e
/// finirebbe scartato in silenzio.
pub const CONTAS_REVISAO_OBRIGATORIAS: &[(&str, &str)] = &[
    ("NAO_CLASSIFICADO", "13.01.001"),
    ("AMBIGUO", "13.01.002"),
    ("VALOR_INCONSISTENTE", "13.01.006"),
    ("POSSIVEL_INVESTIMENTO", "13.01.009"),
    ("PESSOAL_A_REGULARIZAR", "13.01.010"),
    ("OUTROS", "13.01.999"),
];

/// Motivo di fallback usato quando il motivo richiesto non e' mappato.
pub const MOTIVO_REVISAO_PADRAO: &str = "OUTROS";

// ---------------------------------------------------------------------------
// Errori
// ---------------------------------------------------------------------------

/// Errore di caricamento di una fonte di configurazione.
///
/// Porta sempre il percorso del file: un errore senza fonte non e' diagnosticabile.
#[derive(Debug)]
pub enum ErroConfig {
    Leitura {
        caminho: PathBuf,
        erro: std::io::Error,
    },
    Yaml {
        caminho: PathBuf,
        erro: serde_yaml::Error,
    },
}

impl fmt::Display for ErroConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroConfig::Leitura { caminho, erro } => {
                write!(f, "nao foi possivel ler {}: {erro}", caminho.display())
            }
            ErroConfig::Yaml { caminho, erro } => {
                write!(f, "YAML invalido em {}: {erro}", caminho.display())
            }
        }
    }
}

impl std::error::Error for ErroConfig {}

fn ler_yaml<T: serde::de::DeserializeOwned>(caminho: &Path) -> Result<T, ErroConfig> {
    let texto = std::fs::read_to_string(caminho).map_err(|erro| ErroConfig::Leitura {
        caminho: caminho.to_path_buf(),
        erro,
    })?;
    serde_yaml::from_str(&texto).map_err(|erro| ErroConfig::Yaml {
        caminho: caminho.to_path_buf(),
        erro,
    })
}

// ---------------------------------------------------------------------------
// Validazione
// ---------------------------------------------------------------------------

/// Gravita' di un problema di validazione.
///
/// `Erro` blocca l'uso della configurazione. `Alerta` e' da pianificare ma non
/// impedisce il caricamento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severidade {
    Erro,
    Alerta,
}

/// Un problema trovato nella configurazione, con codice stabile e posizione.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problema {
    pub severidade: Severidade,
    /// Codice stabile, citabile nei test e nell'audit (es. `C5_ID_DUPLICADO`).
    pub codigo: &'static str,
    /// File o elemento a cui il problema si riferisce.
    pub local: String,
    pub mensagem: String,
}

impl Problema {
    fn erro(codigo: &'static str, local: impl Into<String>, mensagem: impl Into<String>) -> Self {
        Self {
            severidade: Severidade::Erro,
            codigo,
            local: local.into(),
            mensagem: mensagem.into(),
        }
    }

    fn alerta(codigo: &'static str, local: impl Into<String>, mensagem: impl Into<String>) -> Self {
        Self {
            severidade: Severidade::Alerta,
            codigo,
            local: local.into(),
            mensagem: mensagem.into(),
        }
    }
}

impl fmt::Display for Problema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marca = match self.severidade {
            Severidade::Erro => "ERRO ",
            Severidade::Alerta => "ALERTA",
        };
        write!(
            f,
            "[{marca}] {:<24} {} :: {}",
            self.codigo, self.local, self.mensagem
        )
    }
}

// ---------------------------------------------------------------------------
// Piano dei conti
// ---------------------------------------------------------------------------

/// Struttura esatta di `config/plano_contas_gerencial.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanoContasYaml {
    pub versao: String,
    pub idioma: String,
    pub descricao: String,
    #[serde(default)]
    pub observacao: Option<String>,
    pub centros_custo: Vec<CentroCusto>,
    pub contas: Vec<Conta>,
}

/// Piano dei conti caricato e indicizzato per ID.
#[derive(Debug, Clone)]
pub struct PlanoContas {
    yaml: PlanoContasYaml,
    /// Indice ID -> posizione in `yaml.contas`. L'ordine di inserimento resta
    /// in `yaml.contas`, cosi' i duplicati restano visibili alla validazione.
    indice: HashMap<String, usize>,
    fonte: PathBuf,
}

impl PlanoContas {
    pub fn carregar(caminho: impl AsRef<Path>) -> Result<Self, ErroConfig> {
        let caminho = caminho.as_ref();
        let yaml: PlanoContasYaml = ler_yaml(caminho)?;
        Ok(Self::a_partir_de(yaml, caminho.to_path_buf()))
    }

    fn a_partir_de(yaml: PlanoContasYaml, fonte: PathBuf) -> Self {
        let mut indice = HashMap::with_capacity(yaml.contas.len());
        for (pos, conta) in yaml.contas.iter().enumerate() {
            // Il primo vince: i successivi restano duplicati segnalati da `validar`.
            indice.entry(conta.id.clone()).or_insert(pos);
        }
        Self {
            yaml,
            indice,
            fonte,
        }
    }

    pub fn fonte(&self) -> &Path {
        &self.fonte
    }

    pub fn versao(&self) -> &str {
        &self.yaml.versao
    }

    pub fn contas(&self) -> &[Conta] {
        &self.yaml.contas
    }

    pub fn centros_custo(&self) -> &[CentroCusto] {
        &self.yaml.centros_custo
    }

    /// Conta per ID. `None` invece di un fallback silenzioso: chi chiama decide
    /// se e' un errore o se va in conta di revisione.
    pub fn por_id(&self, id: &str) -> Option<&Conta> {
        self.indice.get(id).map(|&pos| &self.yaml.contas[pos])
    }

    /// Conta di revisione per un motivo (porting di `get_review_account`).
    ///
    /// Il fallback su `OUTROS` e' dichiarato, non silenzioso: il motivo
    /// effettivamente usato viene restituito insieme alla conta.
    pub fn conta_revisao(&self, motivo: &str) -> Option<(&'static str, &Conta)> {
        let (motivo_usado, id) = CONTAS_REVISAO_OBRIGATORIAS
            .iter()
            .find(|(m, _)| *m == motivo)
            .or_else(|| {
                CONTAS_REVISAO_OBRIGATORIAS
                    .iter()
                    .find(|(m, _)| *m == MOTIVO_REVISAO_PADRAO)
            })?;
        self.por_id(id).map(|conta| (*motivo_usado, conta))
    }

    /// Contratto C5: invarianti del piano conti.
    pub fn validar(&self) -> Vec<Problema> {
        let local = self.fonte.display().to_string();
        let mut problemas = Vec::new();
        let mut vistos: BTreeMap<&str, usize> = BTreeMap::new();

        for conta in &self.yaml.contas {
            *vistos.entry(conta.id.as_str()).or_insert(0) += 1;

            for (campo, valor) in [
                ("id", &conta.id),
                ("nome", &conta.nome),
                ("grupo", &conta.grupo),
            ] {
                if valor.trim().is_empty() {
                    problemas.push(Problema::erro(
                        "C5_CAMPO_VAZIO",
                        &local,
                        format!("campo '{campo}' vazio na conta '{}'", conta.id),
                    ));
                }
            }
        }

        for (id, quantas) in vistos {
            if quantas > 1 {
                problemas.push(Problema::erro(
                    "C5_ID_DUPLICADO",
                    &local,
                    format!("conta '{id}' aparece {quantas} vezes"),
                ));
            }
        }

        let mut ids_cc: BTreeMap<&str, usize> = BTreeMap::new();
        for cc in &self.yaml.centros_custo {
            *ids_cc.entry(cc.id.as_str()).or_insert(0) += 1;
        }
        for (id, quantas) in ids_cc {
            if quantas > 1 {
                problemas.push(Problema::erro(
                    "C5_CENTRO_CUSTO_DUPLICADO",
                    &local,
                    format!("centro de custo '{id}' aparece {quantas} vezes"),
                ));
            }
        }

        for (motivo, id) in CONTAS_REVISAO_OBRIGATORIAS {
            match self.por_id(id) {
                None => problemas.push(Problema::erro(
                    "C5_CONTA_REVISAO_AUSENTE",
                    &local,
                    format!("conta de revisao obrigatoria ausente: {id} ({motivo})"),
                )),
                // Bloccante: una conta di revisione di tipo ordinario farebbe
                // finire i casi da revisionare dentro i costi correnti.
                Some(conta) if conta.tipo != TipoConta::Revisao => problemas.push(Problema::erro(
                    "C5_CONTA_REVISAO_TIPO",
                    &local,
                    format!(
                        "conta de revisao {id} ({motivo}) tem tipo {:?}, esperado Revisao",
                        conta.tipo
                    ),
                )),
                Some(_) => {}
            }
        }

        problemas
    }
}

// ---------------------------------------------------------------------------
// Regole di classificazione
// ---------------------------------------------------------------------------

/// Politica di normalizzazione testuale dichiarata nello YAML.
///
/// Il codice di normalizzazione (fase 2) deve leggere questi flag, non
/// assumerli: altrimenti lo YAML descriverebbe una cosa e il codice ne farebbe
/// un'altra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Normalizacao {
    pub minusculas: bool,
    pub remover_acentos: bool,
    pub remover_pontuacao: bool,
    pub compactar_espacos: bool,
}

/// Politica di match dichiarata (solo nelle regole prodotti).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoliticaDeMatch {
    pub em_caso_de_multiplos_matches: String,
    pub em_caso_de_zero_match: String,
    #[serde(default)]
    pub observacao: Option<String>,
}

/// Regola generale: alias -> conta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Regra {
    pub conta_id: String,
    pub conta_nome: String,
    /// Default dichiarato: 0 (come `rule.get("prioridade", 0)` nel Python).
    /// Le regole despesas non portano priorita'.
    #[serde(default)]
    pub prioridade: i64,
    pub aliases: Vec<String>,
    /// Alias esplicitamente esclusi (es. `sch`, `sol` per evitare falsi match).
    #[serde(default)]
    pub evitar_aliases: Vec<String>,
    #[serde(default)]
    pub observacao: Option<String>,
}

/// Regola specifica per fornitore, valutata prima delle regole generali.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegraPorFornecedor {
    pub fornecedor_contem: String,
    pub produto_normalizado_contem: String,
    pub conta_id: String,
    pub conta_nome: String,
    #[serde(default)]
    pub observacao: Option<String>,
}

/// Struttura esatta di `regras_classificacao_produtos.yaml` e
/// `regras_classificacao_despesas.yaml` (stesso schema, sezioni opzionali diverse).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegrasClassificacaoYaml {
    pub versao: String,
    pub idioma: String,
    pub descricao: String,
    pub normalizacao: Normalizacao,
    #[serde(default)]
    pub politica_de_match: Option<PoliticaDeMatch>,
    #[serde(default)]
    pub regras_por_fornecedor: Vec<RegraPorFornecedor>,
    pub regras: Vec<Regra>,
    /// Conta di destinazione quando nessun alias corrisponde.
    /// Obbligatoria: senza, un prodotto non classificato sparirebbe.
    pub conta_fallback_id: String,
    pub conta_fallback_nome: String,
}

/// Regole caricate da un file, con la fonte in chiaro.
#[derive(Debug, Clone)]
pub struct RegrasClassificacao {
    yaml: RegrasClassificacaoYaml,
    fonte: PathBuf,
}

impl RegrasClassificacao {
    pub fn carregar(caminho: impl AsRef<Path>) -> Result<Self, ErroConfig> {
        let caminho = caminho.as_ref();
        let yaml: RegrasClassificacaoYaml = ler_yaml(caminho)?;
        Ok(Self {
            yaml,
            fonte: caminho.to_path_buf(),
        })
    }

    /// Carica da testo YAML. `origem` serve solo a dare una fonte ai messaggi.
    pub fn de_texto(texto: &str, origem: &str) -> Result<Self, ErroConfig> {
        let yaml: RegrasClassificacaoYaml =
            serde_yaml::from_str(texto).map_err(|erro| ErroConfig::Yaml {
                caminho: PathBuf::from(origem),
                erro,
            })?;
        Ok(Self {
            yaml,
            fonte: PathBuf::from(origem),
        })
    }

    pub fn fonte(&self) -> &Path {
        &self.fonte
    }

    pub fn versao(&self) -> &str {
        &self.yaml.versao
    }

    pub fn regras(&self) -> &[Regra] {
        &self.yaml.regras
    }

    pub fn regras_por_fornecedor(&self) -> &[RegraPorFornecedor] {
        &self.yaml.regras_por_fornecedor
    }

    pub fn normalizacao(&self) -> Normalizacao {
        self.yaml.normalizacao
    }

    pub fn conta_fallback_id(&self) -> &str {
        &self.yaml.conta_fallback_id
    }

    /// Validazione delle regole contro il piano conti.
    ///
    /// Contratto C6: nessuna regola puo' puntare a una conta inesistente.
    pub fn validar(&self, plano: &PlanoContas) -> Vec<Problema> {
        let local = self.fonte.display().to_string();
        let mut problemas = Vec::new();

        if self.yaml.regras.is_empty() {
            problemas.push(Problema::erro(
                "C6_SEM_REGRAS",
                &local,
                "arquivo de regras vazio: nenhuma classificacao seria aplicada",
            ));
        }

        // Contratto C13: sezioni dichiarate nello YAML che il classificatore
        // non applica. Non sono errori (il Python si comporta cosi'), ma non
        // devono restare invisibili: chi legge lo YAML crede che valgano.
        if !self.yaml.regras_por_fornecedor.is_empty() {
            problemas.push(Problema::alerta(
                "C13_SECAO_NAO_APLICADA",
                &local,
                format!(
                    "regras_por_fornecedor: {} regra(s) declarada(s) e nao aplicada(s) pelo classificador \
                     (no Python so classification_rule_proposal.py as le)",
                    self.yaml.regras_por_fornecedor.len()
                ),
            ));
        }
        if self.yaml.politica_de_match.is_some() {
            problemas.push(Problema::alerta(
                "C13_SECAO_NAO_APLICADA",
                &local,
                "politica_de_match: declarada e nao aplicada pelo classificador \
                 (p.ex. alias de ate 3 caracteres deveriam casar so por palavra inteira)",
            ));
        }

        // Fallback: deve esistere e deve essere una conta di revisione.
        match plano.por_id(&self.yaml.conta_fallback_id) {
            None => problemas.push(Problema::erro(
                "C6_FALLBACK_INEXISTENTE",
                &local,
                format!(
                    "conta_fallback_id '{}' nao existe no plano de contas",
                    self.yaml.conta_fallback_id
                ),
            )),
            // Bloccante: se il fallback non e' una conta di revisione, i prodotti
            // non classificati finiscono in una conta ordinaria senza che nessuno
            // se ne accorga. E' il caso che C6 esiste per impedire.
            Some(conta) if conta.tipo != TipoConta::Revisao => problemas.push(Problema::erro(
                "C6_FALLBACK_NAO_E_REVISAO",
                &local,
                format!(
                    "conta_fallback_id '{}' tem tipo {:?}, esperado Revisao",
                    conta.id, conta.tipo
                ),
            )),
            Some(_) => {}
        }

        // Alias -> conte, per individuare ambiguita' strutturali.
        let mut alias_para_contas: BTreeMap<String, BTreeMap<String, ()>> = BTreeMap::new();

        let referencias = self
            .yaml
            .regras
            .iter()
            .map(|r| (r.conta_id.as_str(), r.conta_nome.as_str(), "regras"))
            .chain(self.yaml.regras_por_fornecedor.iter().map(|r| {
                (
                    r.conta_id.as_str(),
                    r.conta_nome.as_str(),
                    "regras_por_fornecedor",
                )
            }));

        for (conta_id, conta_nome, secao) in referencias {
            match plano.por_id(conta_id) {
                None => problemas.push(Problema::erro(
                    "C6_CONTA_INEXISTENTE",
                    &local,
                    format!("{secao}: conta_id '{conta_id}' nao existe no plano de contas"),
                )),
                Some(conta) if conta.nome != conta_nome => problemas.push(Problema::alerta(
                    "C6_NOME_DIVERGENTE",
                    &local,
                    format!(
                        "{secao}: conta '{conta_id}' chamada '{conta_nome}' aqui e '{}' no plano de contas",
                        conta.nome
                    ),
                )),
                Some(_) => {}
            }
        }

        for regra in &self.yaml.regras {
            if regra.aliases.is_empty() {
                problemas.push(Problema::erro(
                    "C6_REGRA_SEM_ALIAS",
                    &local,
                    format!("regra da conta '{}' nao tem nenhum alias", regra.conta_id),
                ));
            }

            for alias in &regra.aliases {
                let alias_limpo = alias.trim();
                if alias_limpo.is_empty() {
                    problemas.push(Problema::erro(
                        "C6_ALIAS_VAZIO",
                        &local,
                        format!("regra da conta '{}' tem um alias vazio", regra.conta_id),
                    ));
                    continue;
                }
                alias_para_contas
                    .entry(alias_limpo.to_lowercase())
                    .or_default()
                    .insert(regra.conta_id.clone(), ());
            }

            for evitado in &regra.evitar_aliases {
                if regra.aliases.iter().any(|a| a.trim() == evitado.trim()) {
                    problemas.push(Problema::erro(
                        "C6_ALIAS_CONTRADITORIO",
                        &local,
                        format!(
                            "regra da conta '{}': '{evitado}' esta ao mesmo tempo em aliases e evitar_aliases",
                            regra.conta_id
                        ),
                    ));
                }
            }
        }

        for (alias, contas) in alias_para_contas {
            if contas.len() > 1 {
                let ids: Vec<&str> = contas.keys().map(|s| s.as_str()).collect();
                problemas.push(Problema::alerta(
                    "C6_ALIAS_AMBIGUO",
                    &local,
                    format!(
                        "alias '{alias}' aponta para varias contas: {}",
                        ids.join(", ")
                    ),
                ));
            }
        }

        problemas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLANO_MINIMO: &str = r#"
versao: "1.0"
idioma: pt-BR
descricao: plano de teste
contas:
  - id: "2.01.001"
    nome: "Cervejas"
    grupo: "Custos"
    tipo: custo
  - id: "13.01.001"
    nome: "Revisar - Produto Nao Classificado"
    grupo: "Revisao"
    tipo: revisao
    exige_revisao: true
  - id: "13.01.002"
    nome: "Revisar - Ambiguo"
    grupo: "Revisao"
    tipo: revisao
  - id: "13.01.006"
    nome: "Revisar - Valor Inconsistente"
    grupo: "Revisao"
    tipo: revisao
  - id: "13.01.009"
    nome: "Revisar - Possivel Investimento"
    grupo: "Revisao"
    tipo: revisao
  - id: "13.01.010"
    nome: "Revisar - Pessoal"
    grupo: "Revisao"
    tipo: revisao
  - id: "13.01.999"
    nome: "Revisar - Outros Casos"
    grupo: "Revisao"
    tipo: revisao
centros_custo:
  - id: CC-001
    nome: Cozinha
"#;

    fn plano(yaml: &str) -> PlanoContas {
        PlanoContas::a_partir_de(
            serde_yaml::from_str(yaml).expect("YAML de teste invalido"),
            PathBuf::from("teste://plano"),
        )
    }

    fn regras(yaml: &str) -> RegrasClassificacao {
        RegrasClassificacao {
            yaml: serde_yaml::from_str(yaml).expect("YAML de teste invalido"),
            fonte: PathBuf::from("teste://regras"),
        }
    }

    fn codigos(problemas: &[Problema]) -> Vec<&str> {
        problemas.iter().map(|p| p.codigo).collect()
    }

    #[test]
    fn plano_minimo_valido_nao_gera_problemas() {
        assert_eq!(plano(PLANO_MINIMO).validar(), vec![]);
    }

    #[test]
    fn c5_detecta_id_duplicado() {
        // la seconda conta ripete l'ID della prima
        let yaml = PLANO_MINIMO.replace(
            "  - id: \"13.01.001\"\n    nome: \"Revisar - Produto Nao Classificado\"",
            "  - id: \"2.01.001\"\n    nome: \"Cervejas (duplicada)\"\n    grupo: \"Custos\"\n    tipo: custo\n  - id: \"13.01.001\"\n    nome: \"Revisar - Produto Nao Classificado\"",
        );
        let problemas = plano(&yaml).validar();
        assert!(
            codigos(&problemas).contains(&"C5_ID_DUPLICADO"),
            "{problemas:?}"
        );
    }

    #[test]
    fn c5_detecta_conta_de_revisao_ausente() {
        let yaml = PLANO_MINIMO.replace("13.01.999", "13.01.998");
        let problemas = plano(&yaml).validar();
        assert!(
            codigos(&problemas).contains(&"C5_CONTA_REVISAO_AUSENTE"),
            "{problemas:?}"
        );
    }

    #[test]
    fn c5_detecta_campo_vazio() {
        let yaml = PLANO_MINIMO.replace("nome: \"Cervejas\"", "nome: \"\"");
        let problemas = plano(&yaml).validar();
        assert!(
            codigos(&problemas).contains(&"C5_CAMPO_VAZIO"),
            "{problemas:?}"
        );
    }

    /// Contratto C3: chiave sconosciuta nel file di regole = errore di caricamento.
    #[test]
    fn c3_chave_desconhecida_no_yaml_falha() {
        let yaml = PLANO_MINIMO.replace("idioma: pt-BR", "idiomas: pt-BR");
        assert!(serde_yaml::from_str::<PlanoContasYaml>(&yaml).is_err());
    }

    const REGRAS_MINIMAS: &str = r#"
versao: "1.0"
idioma: pt-BR
descricao: regras de teste
normalizacao:
  minusculas: true
  remover_acentos: true
  remover_pontuacao: true
  compactar_espacos: true
regras:
  - conta_id: "2.01.001"
    conta_nome: "Cervejas"
    prioridade: 100
    aliases:
      - cerveja
      - heineken
conta_fallback_id: "13.01.001"
conta_fallback_nome: "Revisar - Produto Nao Classificado"
"#;

    #[test]
    fn regras_minimas_validas_nao_geram_problemas() {
        let p = plano(PLANO_MINIMO);
        assert_eq!(regras(REGRAS_MINIMAS).validar(&p), vec![]);
    }

    /// Contratto C6: una regola che punta a una conta inesistente deve fallire
    /// al caricamento, non sul primo documento reale.
    #[test]
    fn c6_detecta_conta_inexistente() {
        let p = plano(PLANO_MINIMO);
        let yaml = REGRAS_MINIMAS.replace("conta_id: \"2.01.001\"", "conta_id: \"9.99.999\"");
        let problemas = regras(&yaml).validar(&p);
        assert!(
            codigos(&problemas).contains(&"C6_CONTA_INEXISTENTE"),
            "{problemas:?}"
        );
        assert_eq!(problemas[0].severidade, Severidade::Erro);
    }

    #[test]
    fn c6_detecta_nome_divergente_do_plano() {
        let p = plano(PLANO_MINIMO);
        let yaml = REGRAS_MINIMAS.replace("conta_nome: \"Cervejas\"", "conta_nome: \"Cerveja\"");
        let problemas = regras(&yaml).validar(&p);
        assert!(
            codigos(&problemas).contains(&"C6_NOME_DIVERGENTE"),
            "{problemas:?}"
        );
    }

    #[test]
    fn c6_detecta_fallback_inexistente() {
        let p = plano(PLANO_MINIMO);
        let yaml = REGRAS_MINIMAS.replace(
            "conta_fallback_id: \"13.01.001\"",
            "conta_fallback_id: \"0.00.000\"",
        );
        let problemas = regras(&yaml).validar(&p);
        assert!(
            codigos(&problemas).contains(&"C6_FALLBACK_INEXISTENTE"),
            "{problemas:?}"
        );
    }

    /// R1: un fallback che non e' conta di revisione deve bloccare la
    /// configurazione anche senza `--estrito`, perche' manderebbe i prodotti
    /// non classificati dentro una conta ordinaria.
    #[test]
    fn c6_fallback_de_tipo_errado_e_erro_bloqueante() {
        let p = plano(PLANO_MINIMO);
        // 2.01.001 esiste no plano, mas e' tipo custo
        let yaml = REGRAS_MINIMAS.replace(
            "conta_fallback_id: \"13.01.001\"",
            "conta_fallback_id: \"2.01.001\"",
        );
        let problemas = regras(&yaml).validar(&p);
        let encontrado = problemas
            .iter()
            .find(|x| x.codigo == "C6_FALLBACK_NAO_E_REVISAO")
            .unwrap_or_else(|| panic!("problema nao detectado: {problemas:?}"));
        assert_eq!(
            encontrado.severidade,
            Severidade::Erro,
            "um fallback fora da area de revisao nao pode ser apenas alerta"
        );
    }

    /// R1: stessa regola per le conte di revisione obbligatorie.
    #[test]
    fn c5_conta_de_revisao_de_tipo_errado_e_erro_bloqueante() {
        let yaml = PLANO_MINIMO.replace(
            "  - id: \"13.01.999\"\n    nome: \"Revisar - Outros Casos\"\n    grupo: \"Revisao\"\n    tipo: revisao\n",
            "  - id: \"13.01.999\"\n    nome: \"Revisar - Outros Casos\"\n    grupo: \"Revisao\"\n    tipo: despesa\n",
        );
        let problemas = plano(&yaml).validar();
        let encontrado = problemas
            .iter()
            .find(|x| x.codigo == "C5_CONTA_REVISAO_TIPO")
            .unwrap_or_else(|| panic!("problema nao detectado: {problemas:?}"));
        assert_eq!(encontrado.severidade, Severidade::Erro);
    }

    #[test]
    fn c6_detecta_regra_sem_alias() {
        let p = plano(PLANO_MINIMO);
        let yaml = REGRAS_MINIMAS.replace(
            "    aliases:\n      - cerveja\n      - heineken\n",
            "    aliases: []\n",
        );
        let problemas = regras(&yaml).validar(&p);
        assert!(
            codigos(&problemas).contains(&"C6_REGRA_SEM_ALIAS"),
            "{problemas:?}"
        );
    }

    #[test]
    fn c6_detecta_alias_contraditorio() {
        let p = plano(PLANO_MINIMO);
        let yaml = REGRAS_MINIMAS.replace(
            "      - heineken\n",
            "      - heineken\n    evitar_aliases:\n      - heineken\n",
        );
        let problemas = regras(&yaml).validar(&p);
        assert!(
            codigos(&problemas).contains(&"C6_ALIAS_CONTRADITORIO"),
            "{problemas:?}"
        );
    }

    /// Il fallback di motivo e' dichiarato: motivo sconosciuto -> OUTROS, e il
    /// motivo effettivamente usato viene restituito.
    #[test]
    fn conta_revisao_usa_fallback_declarado() {
        let p = plano(PLANO_MINIMO);
        let (motivo, conta) = p.conta_revisao("MOTIVO_QUE_NAO_EXISTE").unwrap();
        assert_eq!(motivo, MOTIVO_REVISAO_PADRAO);
        assert_eq!(conta.id, "13.01.999");

        let (motivo, conta) = p.conta_revisao("AMBIGUO").unwrap();
        assert_eq!(motivo, "AMBIGUO");
        assert_eq!(conta.id, "13.01.002");
    }

    /// Nessun fallback silenzioso su conta inesistente: `None`, non "a caso".
    #[test]
    fn por_id_inexistente_devolve_none() {
        assert!(plano(PLANO_MINIMO).por_id("0.00.000").is_none());
    }
}

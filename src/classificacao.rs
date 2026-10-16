//! Classificazione di descrizioni prodotto / despesa verso il piano conti.
//!
//! Porting di `product_classifier.py` (ed `expense_classifier.py`, che e' la
//! stessa logica con un fallback diverso, preso comunque dallo YAML).
//!
//! Logica del Python, riprodotta fedelmente:
//!
//! 1. normalizza la descrizione;
//! 2. raccoglie ogni `(regola, alias)` il cui alias e' sottostringa della
//!    descrizione normalizzata;
//! 3. nessun hit -> conta di fallback, `NAO_CLASSIFICADO`;
//! 4. una sola conta fra gli hit -> `OK`;
//! 5. piu' conte -> `AMBIGUO`, vince comunque la priorita' maggiore.
//!
//! Contratto C11: `max(hits, key=prioridade)` in Python restituisce il
//! **primo** massimo; `Iterator::max_by_key` in Rust restituisce l'**ultimo**.
//! Qui si sceglie esplicitamente il primo, ed esiste un test che lo blocca.
//!
//! Contratto C12: il Python costruisce `matches` e l'elenco delle conte
//! ambigue da `set`, il cui ordine non e' riproducibile. Qui sono ordinati.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::modelos::{Confianca, ResultadoClassificacao};
use crate::normalizacao::normalizar;
use crate::regras::{Normalizacao, PlanoContas, RegrasClassificacao};

/// Errore di classificazione: la regola punta a una conta che non esiste.
///
/// `validar-config` lo impedisce in anticipo (contratto C6); qui resta un
/// errore esplicito invece di un fallback silenzioso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErroClassificacao {
    pub conta_id: String,
    pub fonte: PathBuf,
}

impl std::fmt::Display for ErroClassificacao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "conta '{}' citada por {} nao existe no plano de contas",
            self.conta_id,
            self.fonte.display()
        )
    }
}

impl std::error::Error for ErroClassificacao {}

/// Alias come scritto nello YAML e come usato nel match.
#[derive(Debug, Clone)]
struct AliasCompilado {
    original: String,
    norm: String,
}

/// Regola con gli alias gia' normalizzati, come fa `load()` nel Python.
#[derive(Debug, Clone)]
struct RegraCompilada {
    conta_id: String,
    prioridade: i64,
    aliases: Vec<AliasCompilado>,
}

/// Esito dell'audit di un singolo alias: a quale conta porta davvero.
///
/// Un alias puo' essere dichiarato sotto una conta e, per via di un altro
/// alias piu' specifico o di una priorita' maggiore, finire su un'altra.
/// Il fatto va esposto, non corretto d'ufficio: le regole sono canoniche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditoriaAlias {
    /// Conta sotto cui l'alias e' scritto nello YAML.
    pub conta_declarada: String,
    pub alias: String,
    /// Conta che vince classificando il testo dell'alias da solo.
    pub conta_vencedora: String,
    pub confianca: Confianca,
    pub matches: Vec<String>,
}

impl AuditoriaAlias {
    /// L'alias porta a una conta diversa da quella sotto cui e' scritto.
    pub fn diverge(&self) -> bool {
        self.conta_declarada != self.conta_vencedora
    }

    pub fn ambiguo(&self) -> bool {
        self.confianca == Confianca::Ambiguo
    }

    pub fn nao_classificado(&self) -> bool {
        self.confianca == Confianca::NaoClassificado
    }
}

/// Classificatore compilato a partire da un file di regole.
#[derive(Debug, Clone)]
pub struct Classificador {
    regras: Vec<RegraCompilada>,
    fallback_id: String,
    normalizacao: Normalizacao,
    fonte: PathBuf,
}

impl Classificador {
    /// Compila le regole: normalizza gli alias una volta sola e scarta quelli
    /// che la normalizzazione riduce a stringa vuota (come il Python).
    pub fn compilar(regras: &RegrasClassificacao) -> Self {
        let normalizacao = regras.normalizacao();
        let compiladas = regras
            .regras()
            .iter()
            .map(|r| RegraCompilada {
                conta_id: r.conta_id.clone(),
                prioridade: r.prioridade,
                aliases: r
                    .aliases
                    .iter()
                    .map(|a| AliasCompilado {
                        original: a.clone(),
                        norm: normalizar(a, normalizacao),
                    })
                    .filter(|a| !a.norm.is_empty())
                    .collect(),
            })
            .collect();

        Self {
            regras: compiladas,
            fallback_id: regras.conta_fallback_id().to_string(),
            normalizacao,
            fonte: regras.fonte().to_path_buf(),
        }
    }

    pub fn fonte(&self) -> &Path {
        &self.fonte
    }

    pub fn normalizacao(&self) -> Normalizacao {
        self.normalizacao
    }

    /// Normalizza la descrizione con la politica di **questo** file di regole.
    pub fn normalizar(&self, descricao: &str) -> String {
        normalizar(descricao, self.normalizacao)
    }

    pub fn classificar(
        &self,
        descricao: &str,
        plano: &PlanoContas,
    ) -> Result<ResultadoClassificacao, ErroClassificacao> {
        let desc_norm = self.normalizar(descricao);

        // (indice regola, alias) di ogni corrispondenza, nell'ordine del file.
        let mut hits: Vec<(usize, &str)> = Vec::new();
        for (i, regra) in self.regras.iter().enumerate() {
            for alias in &regra.aliases {
                if desc_norm.contains(alias.norm.as_str()) {
                    hits.push((i, alias.norm.as_str()));
                }
            }
        }

        if hits.is_empty() {
            let conta = self.conta(&self.fallback_id, plano)?;
            return Ok(ResultadoClassificacao {
                conta,
                confianca: Confianca::NaoClassificado,
                alertas: vec![],
                matches: vec![],
            });
        }

        // C11: primo massimo di priorita', come `max()` del Python.
        let melhor = hits
            .iter()
            .map(|(i, _)| *i)
            .reduce(|a, b| {
                if self.regras[b].prioridade > self.regras[a].prioridade {
                    b
                } else {
                    a
                }
            })
            .expect("hits non vuoto");

        // C12: insiemi ordinati, non l'ordine arbitrario del set Python.
        let contas: BTreeSet<&str> = hits
            .iter()
            .map(|(i, _)| self.regras[*i].conta_id.as_str())
            .collect();
        let matches: Vec<String> = hits
            .iter()
            .map(|(_, a)| a.to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        let conta = self.conta(&self.regras[melhor].conta_id, plano)?;

        if contas.len() == 1 {
            return Ok(ResultadoClassificacao {
                conta,
                confianca: Confianca::Ok,
                alertas: vec![],
                matches,
            });
        }

        let ids: Vec<&str> = contas.into_iter().collect();
        Ok(ResultadoClassificacao {
            conta,
            confianca: Confianca::Ambiguo,
            alertas: vec![format!("Classificacao ambigua: contas {}", ids.join(", "))],
            matches,
        })
    }

    /// Classifica ogni alias del file e riporta dove finisce davvero.
    ///
    /// Serve a rispondere a una domanda che il solo "non e' NAO_CLASSIFICADO"
    /// non copre: l'alias porta alla conta sotto cui e' scritto? Restituisce
    /// tutte le righe, nell'ordine del file; filtrare e' di chi chiama.
    pub fn auditar_aliases(
        &self,
        plano: &PlanoContas,
    ) -> Result<Vec<AuditoriaAlias>, ErroClassificacao> {
        let mut linhas = Vec::new();
        for regra in &self.regras {
            for alias in &regra.aliases {
                let r = self.classificar(&alias.original, plano)?;
                linhas.push(AuditoriaAlias {
                    conta_declarada: regra.conta_id.clone(),
                    alias: alias.original.clone(),
                    conta_vencedora: r.conta.id,
                    confianca: r.confianca,
                    matches: r.matches,
                });
            }
        }
        Ok(linhas)
    }

    /// Numero di alias compilati (dopo lo scarto di quelli vuoti).
    pub fn total_aliases(&self) -> usize {
        self.regras.iter().map(|r| r.aliases.len()).sum()
    }

    fn conta(
        &self,
        id: &str,
        plano: &PlanoContas,
    ) -> Result<crate::modelos::Conta, ErroClassificacao> {
        plano.por_id(id).cloned().ok_or_else(|| ErroClassificacao {
            conta_id: id.to_string(),
            fonte: self.fonte.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Baseline misurata sulle regole canoniche al commit della fase 2.
    // Aggiornarla e' consentito solo insieme a una decisione dichiarata sulle
    // regole: non va "allineata" per far passare il test.
    // Totale: 571 alias in 55 regole, 7 divergenti, 11 ambigui.
    const BASELINE_PRODUTOS_ALIASES: usize = 454;
    const BASELINE_PRODUTOS_DIVERGENTES: usize = 6;
    const BASELINE_PRODUTOS_AMBIGUOS: usize = 9;
    const BASELINE_DESPESAS_ALIASES: usize = 117;
    const BASELINE_DESPESAS_DIVERGENTES: usize = 1;
    const BASELINE_DESPESAS_AMBIGUOS: usize = 2;

    fn raiz() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn plano() -> PlanoContas {
        PlanoContas::carregar(raiz().join("config/plano_contas_gerencial.yaml")).unwrap()
    }

    fn produtos() -> Classificador {
        Classificador::compilar(
            &RegrasClassificacao::carregar(
                raiz().join("config/regras_classificacao_produtos.yaml"),
            )
            .unwrap(),
        )
    }

    fn despesas() -> Classificador {
        Classificador::compilar(
            &RegrasClassificacao::carregar(
                raiz().join("config/regras_classificacao_despesas.yaml"),
            )
            .unwrap(),
        )
    }

    // --- Stessi casi attesi di tests/test_classifier.py ---------------------

    #[test]
    fn produto_coca_cola() {
        let r = produtos()
            .classificar("COCA-COLA KS 290ML", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "2.01.004");
        assert_eq!(r.confianca, Confianca::Ok);
    }

    #[test]
    fn produto_vinho_rio_sol() {
        let r = produtos()
            .classificar("VINHO RIO SOL TINTO SECO 750ML", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "2.01.002");
        assert_eq!(r.confianca, Confianca::Ok);
    }

    #[test]
    fn produto_esponja() {
        let r = produtos()
            .classificar("ESPONJA DUPLA FACE LIMPEZA", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "2.04.001");
        assert_eq!(r.confianca, Confianca::Ok);
    }

    #[test]
    fn produto_desconhecido_vai_para_revisao() {
        let r = produtos()
            .classificar("PRODUTO XYZ DESCONHECIDO", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "13.01.001");
        assert_eq!(r.confianca, Confianca::NaoClassificado);
        assert!(r.matches.is_empty());
    }

    #[test]
    fn despesa_energia_eletrica() {
        let r = despesas()
            .classificar("energia eletrica", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "4.01.011");
    }

    #[test]
    fn despesa_fgts() {
        let r = despesas().classificar("fgts guia", &plano()).unwrap();
        assert_eq!(r.conta.id, "3.03.001");
    }

    // --- Contratti ----------------------------------------------------------

    /// Il fallback delle despesas e' un'altra conta di revisione: viene dallo
    /// YAML, non dalla sottoclasse Python.
    #[test]
    fn fallback_das_despesas_vem_do_yaml() {
        let r = despesas()
            .classificar("DESPESA QUE NINGUEM PREVIU", &plano())
            .unwrap();
        assert_eq!(r.conta.id, "13.01.999");
        assert_eq!(r.confianca, Confianca::NaoClassificado);
    }

    /// Gli accenti non devono impedire il match: la descrizione e gli alias
    /// passano per la stessa normalizzazione.
    #[test]
    fn acentos_nao_impedem_o_match() {
        let p = plano();
        let c = produtos();
        let com = c.classificar("ÁGUA MINERAL SEM GÁS 500ML", &p).unwrap();
        let sem = c.classificar("AGUA MINERAL SEM GAS 500ML", &p).unwrap();
        assert_eq!(com.conta.id, sem.conta.id);
        assert_ne!(com.confianca, Confianca::NaoClassificado);
    }

    /// Contratto C11: a parita' di hit vince la **prima** regola di priorita'
    /// massima, come `max()` del Python (Rust `max_by_key` prenderebbe l'ultima).
    #[test]
    fn empate_de_prioridade_escolhe_a_primeira_regra() {
        let plano = plano();
        let yaml = r#"
versao: "1.0"
idioma: pt-BR
descricao: teste de empate
normalizacao:
  minusculas: true
  remover_acentos: true
  remover_pontuacao: true
  compactar_espacos: true
regras:
  - conta_id: "2.01.001"
    conta_nome: "Cervejas"
    prioridade: 100
    aliases: ["xyz"]
  - conta_id: "2.01.002"
    conta_nome: "Vinhos"
    prioridade: 100
    aliases: ["xyz"]
conta_fallback_id: "13.01.001"
conta_fallback_nome: "Revisar - Produto Nao Classificado"
"#;
        let regras = RegrasClassificacao::de_texto(yaml, "teste://empate").unwrap();
        let r = Classificador::compilar(&regras)
            .classificar("PRODUTO XYZ", &plano)
            .unwrap();
        assert_eq!(
            r.conta.id, "2.01.001",
            "deve vencer a primeira regra do arquivo"
        );
        assert_eq!(r.confianca, Confianca::Ambiguo);
    }

    /// Prioridade maior vence, independentemente da ordem no arquivo.
    #[test]
    fn prioridade_maior_vence() {
        let plano = plano();
        let yaml = r#"
versao: "1.0"
idioma: pt-BR
descricao: teste de prioridade
normalizacao:
  minusculas: true
  remover_acentos: true
  remover_pontuacao: true
  compactar_espacos: true
regras:
  - conta_id: "2.01.001"
    conta_nome: "Cervejas"
    prioridade: 10
    aliases: ["xyz"]
  - conta_id: "2.01.002"
    conta_nome: "Vinhos"
    prioridade: 90
    aliases: ["xyz"]
conta_fallback_id: "13.01.001"
conta_fallback_nome: "Revisar - Produto Nao Classificado"
"#;
        let regras = RegrasClassificacao::de_texto(yaml, "teste://prioridade").unwrap();
        let r = Classificador::compilar(&regras)
            .classificar("PRODUTO XYZ", &plano)
            .unwrap();
        assert_eq!(r.conta.id, "2.01.002");
        assert_eq!(r.confianca, Confianca::Ambiguo);
        assert_eq!(
            r.alertas,
            vec!["Classificacao ambigua: contas 2.01.001, 2.01.002"]
        );
    }

    /// Contratto C12: `matches` e l'elenco delle conte ambigue sono ordinati e
    /// senza ripetizioni, quindi l'output e' riproducibile.
    #[test]
    fn matches_sao_deterministicos_e_ordenados() {
        let plano = plano();
        let c = produtos();
        let primeiro = c.classificar("CERVEJA HEINEKEN LATA", &plano).unwrap();
        for _ in 0..5 {
            let outro = c.classificar("CERVEJA HEINEKEN LATA", &plano).unwrap();
            assert_eq!(outro, primeiro);
        }
        let mut ordenado = primeiro.matches.clone();
        ordenado.sort();
        assert_eq!(primeiro.matches, ordenado);
        assert!(primeiro.matches.len() >= 2, "{:?}", primeiro.matches);
    }

    /// Una regola che punta a una conta inesistente e' un errore, non un
    /// fallback silenzioso.
    #[test]
    fn conta_inexistente_e_erro() {
        let plano = plano();
        let yaml = r#"
versao: "1.0"
idioma: pt-BR
descricao: teste
normalizacao:
  minusculas: true
  remover_acentos: true
  remover_pontuacao: true
  compactar_espacos: true
regras:
  - conta_id: "0.00.000"
    conta_nome: "Inexistente"
    aliases: ["xyz"]
conta_fallback_id: "13.01.001"
conta_fallback_nome: "Revisar - Produto Nao Classificado"
"#;
        let regras = RegrasClassificacao::de_texto(yaml, "teste://fantasma").unwrap();
        let erro = Classificador::compilar(&regras)
            .classificar("PRODUTO XYZ", &plano)
            .unwrap_err();
        assert_eq!(erro.conta_id, "0.00.000");
    }

    /// Nessun alias reale deve restare senza classificazione: sarebbe una
    /// regola morta, che non casa nemmeno con se' stessa.
    #[test]
    fn nenhum_alias_real_fica_sem_classificacao() {
        let plano = plano();
        for c in [produtos(), despesas()] {
            let mortos: Vec<String> = c
                .auditar_aliases(&plano)
                .unwrap()
                .into_iter()
                .filter(|a| a.nao_classificado())
                .map(|a| format!("{} ({})", a.alias, a.conta_declarada))
                .collect();
            assert!(
                mortos.is_empty(),
                "alias que nao casam consigo mesmos: {mortos:#?}"
            );
        }
    }

    /// Caratterizzazione dell'audit alias sulle regole canoniche.
    ///
    /// Non giudica le regole: fissa lo stato di fatto. Un alias che porta a
    /// un'altra conta, o che resta ambiguo, non e' automaticamente un bug —
    /// ma deve essere una scelta vista, non una sorpresa. Se questi numeri
    /// cambiano, il test fallisce e mostra l'elenco.
    #[test]
    fn auditoria_de_aliases_corresponde_a_baseline_declarada() {
        let plano = plano();

        let linhas = produtos().auditar_aliases(&plano).unwrap();
        let divergentes: Vec<&AuditoriaAlias> = linhas.iter().filter(|a| a.diverge()).collect();
        let ambiguos: Vec<&AuditoriaAlias> = linhas.iter().filter(|a| a.ambiguo()).collect();
        assert_eq!(linhas.len(), BASELINE_PRODUTOS_ALIASES, "{linhas:#?}");
        assert_eq!(
            divergentes.len(),
            BASELINE_PRODUTOS_DIVERGENTES,
            "{divergentes:#?}"
        );
        assert_eq!(ambiguos.len(), BASELINE_PRODUTOS_AMBIGUOS, "{ambiguos:#?}");

        let linhas = despesas().auditar_aliases(&plano).unwrap();
        let divergentes: Vec<&AuditoriaAlias> = linhas.iter().filter(|a| a.diverge()).collect();
        let ambiguos: Vec<&AuditoriaAlias> = linhas.iter().filter(|a| a.ambiguo()).collect();
        assert_eq!(linhas.len(), BASELINE_DESPESAS_ALIASES, "{linhas:#?}");
        assert_eq!(
            divergentes.len(),
            BASELINE_DESPESAS_DIVERGENTES,
            "{divergentes:#?}"
        );
        assert_eq!(ambiguos.len(), BASELINE_DESPESAS_AMBIGUOS, "{ambiguos:#?}");
    }

    /// Ogni alias divergente deve almeno finire in una conta reale e in un
    /// esito dichiarato: divergente si', perso no.
    #[test]
    fn alias_divergente_continua_rastreavel() {
        let plano = plano();
        for c in [produtos(), despesas()] {
            for linha in c
                .auditar_aliases(&plano)
                .unwrap()
                .iter()
                .filter(|a| a.diverge())
            {
                assert!(
                    plano.por_id(&linha.conta_vencedora).is_some(),
                    "conta vencedora inexistente: {linha:?}"
                );
                assert!(
                    !linha.matches.is_empty(),
                    "divergencia sem match registrado: {linha:?}"
                );
            }
        }
    }
}

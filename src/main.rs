//! CLI do porte Rust do projeto `fantozzi` (registros contabeis de NF-e).
//!
//! Fase 1: modelo de dados + leitura contratualizada das regras YAML.
//! Fase 2: leitura de NF-e (quick-xml), normalizacao e classificacao.
//! Fase 3 (pendente): alocacao em lancamentos, pre-livro diario, DRE.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use rust_registrazioni_contabili::classificacao::Classificador;
use rust_registrazioni_contabili::modelos::Confianca;
use rust_registrazioni_contabili::nfe;
use rust_registrazioni_contabili::regras::{
    ErroConfig, PlanoContas, Problema, RegrasClassificacao, Severidade,
};

const PADRAO_PLANO: &str = "config/plano_contas_gerencial.yaml";
const PADRAO_REGRAS_PRODUTOS: &str = "config/regras_classificacao_produtos.yaml";
const PADRAO_REGRAS_DESPESAS: &str = "config/regras_classificacao_despesas.yaml";

#[derive(Debug, Parser)]
#[command(
    name = "fantozzi",
    version,
    about = "Registros contabeis de NF-e - porte Rust",
    long_about = "Porte Rust de Programma_registrazioni-contabili.\n\
                  Fases 1 e 2: modelo de dados, regras versionadas, leitura de\n\
                  NF-e e classificacao de produtos e despesas."
)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FonteRegras {
    /// Regras de produtos de NF-e.
    Produtos,
    /// Regras de despesas (boletos, extratos, lancamentos manuais).
    Despesas,
}

#[derive(Debug, Subcommand)]
enum Comando {
    /// Carrega e valida as regras versionadas (plano de contas + classificacao).
    ///
    /// Sai com codigo 1 se houver ao menos um ERRO. Alertas nao bloqueiam.
    ValidarConfig {
        #[arg(long, default_value = PADRAO_PLANO)]
        plano_contas: PathBuf,
        #[arg(long, default_value = PADRAO_REGRAS_PRODUTOS)]
        regras_produtos: PathBuf,
        #[arg(long, default_value = PADRAO_REGRAS_DESPESAS)]
        regras_despesas: PathBuf,
        /// Trata alertas como erros (uso em CI antes de promover a producao).
        #[arg(long)]
        estrito: bool,
    },

    /// Le uma NF-e em XML e classifica cada item contra o plano de contas.
    ///
    /// Nao grava nada: a alocacao em lancamentos e' da fase 3.
    ImportarNfe {
        /// Arquivo XML da NF-e.
        #[arg(long)]
        xml: PathBuf,
        #[arg(long, default_value = PADRAO_PLANO)]
        plano_contas: PathBuf,
        #[arg(long, default_value = PADRAO_REGRAS_PRODUTOS)]
        regras_produtos: PathBuf,
    },

    /// Audita os alias de um arquivo de regras: para onde cada um leva de fato.
    ///
    /// Nao altera nenhuma regra. Sai com 1 se houver alias sem classificacao,
    /// ou, com `--estrito`, tambem se houver divergencia ou ambiguidade.
    AuditarAliases {
        #[arg(long, value_enum, default_value_t = FonteRegras::Produtos)]
        tipo: FonteRegras,
        #[arg(long, default_value = PADRAO_PLANO)]
        plano_contas: PathBuf,
        /// Arquivo de regras; por omissao, o padrao do `--tipo` escolhido.
        #[arg(long)]
        regras: Option<PathBuf>,
        /// Divergencias e ambiguidades tambem reprovam.
        #[arg(long)]
        estrito: bool,
    },

    /// Classifica um texto avulso: util para auditar uma regra.
    Classificar {
        /// Descricao do produto ou da despesa.
        #[arg(long)]
        texto: String,
        #[arg(long, value_enum, default_value_t = FonteRegras::Produtos)]
        tipo: FonteRegras,
        #[arg(long, default_value = PADRAO_PLANO)]
        plano_contas: PathBuf,
        /// Arquivo de regras; por omissao, o padrao do `--tipo` escolhido.
        #[arg(long)]
        regras: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let resultado = match Cli::parse().comando {
        Comando::ValidarConfig {
            plano_contas,
            regras_produtos,
            regras_despesas,
            estrito,
        } => validar_config(&plano_contas, &regras_produtos, &regras_despesas, estrito)
            .map_err(|e| e.to_string()),
        Comando::ImportarNfe {
            xml,
            plano_contas,
            regras_produtos,
        } => importar_nfe(&xml, &plano_contas, &regras_produtos),
        Comando::Classificar {
            texto,
            tipo,
            plano_contas,
            regras,
        } => classificar(&texto, &plano_contas, &caminho_regras(tipo, regras)),
        Comando::AuditarAliases {
            tipo,
            plano_contas,
            regras,
            estrito,
        } => auditar_aliases(&plano_contas, &caminho_regras(tipo, regras), estrito),
    };

    match resultado {
        Ok(codigo) => codigo,
        Err(erro) => {
            eprintln!("ERRO: {erro}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// validar-config
// ---------------------------------------------------------------------------

fn validar_config(
    caminho_plano: &PathBuf,
    caminho_produtos: &PathBuf,
    caminho_despesas: &PathBuf,
    estrito: bool,
) -> Result<ExitCode, ErroConfig> {
    let plano = PlanoContas::carregar(caminho_plano)?;
    let produtos = RegrasClassificacao::carregar(caminho_produtos)?;
    let despesas = RegrasClassificacao::carregar(caminho_despesas)?;

    println!("AUDITORIA DE CONFIGURACAO");
    println!("=========================");
    println!(
        "Plano de contas    : {} (versao {}) -> {} contas, {} centros de custo",
        plano.fonte().display(),
        plano.versao(),
        plano.contas().len(),
        plano.centros_custo().len()
    );
    for (rotulo, r) in [
        ("Regras produtos    ", &produtos),
        ("Regras despesas    ", &despesas),
    ] {
        println!(
            "{rotulo}: {} (versao {}) -> {} regras, {} regras por fornecedor, fallback {}",
            r.fonte().display(),
            r.versao(),
            r.regras().len(),
            r.regras_por_fornecedor().len(),
            r.conta_fallback_id()
        );
    }
    println!();

    let mut problemas: Vec<Problema> = Vec::new();
    problemas.extend(plano.validar());
    problemas.extend(produtos.validar(&plano));
    problemas.extend(despesas.validar(&plano));

    let erros = problemas
        .iter()
        .filter(|p| p.severidade == Severidade::Erro)
        .count();
    let alertas = problemas.len() - erros;

    if problemas.is_empty() {
        println!("Nenhum problema encontrado.");
    } else {
        problemas.sort_by(|a, b| a.severidade.cmp(&b.severidade).then(a.codigo.cmp(b.codigo)));
        for p in &problemas {
            println!("{p}");
        }
    }

    println!();
    println!("RESUMO: {erros} erro(s), {alertas} alerta(s)");

    let reprovado = erros > 0 || (estrito && alertas > 0);
    println!(
        "STATUS: {}",
        if reprovado { "REPROVADO" } else { "APROVADO" }
    );

    Ok(if reprovado {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// Caminho do arquivo de regras: o explicito, ou o padrao do `--tipo`.
fn caminho_regras(tipo: FonteRegras, explicito: Option<PathBuf>) -> PathBuf {
    explicito.unwrap_or_else(|| match tipo {
        FonteRegras::Produtos => PathBuf::from(PADRAO_REGRAS_PRODUTOS),
        FonteRegras::Despesas => PathBuf::from(PADRAO_REGRAS_DESPESAS),
    })
}

// ---------------------------------------------------------------------------
// Carregamento comum: nenhum comando usa configuracao que validar-config reprova
// ---------------------------------------------------------------------------

/// Carica piano conti e regole **passando dalla stessa validazione** di
/// `validar-config`.
///
/// Senza questo, `importar-nfe` e `classificar` accetterebbero una
/// configurazione che `validar-config` boccia: per esempio un fallback che
/// manda i prodotti sconosciuti in una conta di costo ordinaria.
/// Gli errori bloccano; gli alerta vengono stampati su stderr e non spariscono.
fn carregar_validado(
    caminho_plano: &PathBuf,
    caminho_regras: &PathBuf,
) -> Result<(PlanoContas, RegrasClassificacao), String> {
    let plano = PlanoContas::carregar(caminho_plano).map_err(|e| e.to_string())?;
    let regras = RegrasClassificacao::carregar(caminho_regras).map_err(|e| e.to_string())?;

    let mut problemas = plano.validar();
    problemas.extend(regras.validar(&plano));
    problemas.sort_by(|a, b| a.severidade.cmp(&b.severidade).then(a.codigo.cmp(b.codigo)));

    let erros: Vec<&Problema> = problemas
        .iter()
        .filter(|p| p.severidade == Severidade::Erro)
        .collect();

    for problema in problemas
        .iter()
        .filter(|p| p.severidade == Severidade::Alerta)
    {
        eprintln!("{problema}");
    }

    if !erros.is_empty() {
        for erro in &erros {
            eprintln!("{erro}");
        }
        return Err(format!(
            "configuracao reprovada: {} erro(s). Rode `validar-config` para o relatorio completo.",
            erros.len()
        ));
    }

    Ok((plano, regras))
}

// ---------------------------------------------------------------------------
// importar-nfe
// ---------------------------------------------------------------------------

fn importar_nfe(
    xml: &PathBuf,
    caminho_plano: &PathBuf,
    caminho_regras: &PathBuf,
) -> Result<ExitCode, String> {
    let (plano, regras) = carregar_validado(caminho_plano, caminho_regras)?;
    let classificador = Classificador::compilar(&regras);

    let importada = nfe::ler_nfe(xml).map_err(|e| e.to_string())?;
    let n = &importada.nfe;

    println!("NF-e {} serie {} de {}", n.numero, n.serie, n.data_emissao);
    println!("Chave        : {}", n.chave);
    println!("Emitente     : {} ({})", n.nome_emitente, n.cnpj_emitente);
    println!(
        "Destinatario : {} ({})",
        n.nome_destinatario, n.cnpj_destinatario
    );
    println!("Natureza     : {}", n.natureza_operacao);
    println!(
        "Totais       : produtos {} | frete {} | desconto {} | NF {}",
        n.valor_produtos, n.valor_frete, n.valor_desconto, n.valor_total
    );
    println!("Tributos inc.: {}", n.valor_tributos_inclusos);
    // O proprio `Dado` sabe se dizer: valor, ou estado e motivo. Nunca zero
    // no lugar de uma ausencia.
    println!(
        "Cobranca     : {} | {} duplicata(s)",
        n.valor_cobranca,
        n.duplicatas.len()
    );
    println!();

    println!("ITENS E CLASSIFICACAO");
    println!("---------------------");
    let mut por_revisar = 0usize;
    for produto in &n.produtos {
        let r = classificador
            .classificar(&produto.descricao, &plano)
            .map_err(|e| e.to_string())?;
        let marca = match r.confianca {
            Confianca::Ok => "OK      ",
            Confianca::Ambiguo => "AMBIGUO ",
            Confianca::NaoClassificado => "REVISAR ",
        };
        if r.confianca != Confianca::Ok {
            por_revisar += 1;
        }
        println!(
            "{marca} item {:>3} | {:>10} | {} -> {} {}",
            produto.n_item, produto.valor_total, produto.descricao, r.conta.id, r.conta.nome
        );
        if !r.matches.is_empty() {
            println!("            match: {}", r.matches.join(", "));
        }
        for alerta in &r.alertas {
            println!("            alerta: {alerta}");
        }
    }

    println!();
    if importada.avisos.is_empty() {
        println!("Nenhum aviso de leitura.");
    } else {
        println!("AVISOS DE LEITURA");
        println!("-----------------");
        for aviso in &importada.avisos {
            println!("{aviso}");
        }
    }

    println!();
    println!(
        "RESUMO: {} item(ns), {} a revisar, {} aviso(s) de leitura",
        n.produtos.len(),
        por_revisar,
        importada.avisos.len()
    );

    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// classificar
// ---------------------------------------------------------------------------

fn classificar(
    texto: &str,
    caminho_plano: &PathBuf,
    caminho_regras: &PathBuf,
) -> Result<ExitCode, String> {
    let (plano, regras) = carregar_validado(caminho_plano, caminho_regras)?;
    let classificador = Classificador::compilar(&regras);

    let r = classificador
        .classificar(texto, &plano)
        .map_err(|e| e.to_string())?;

    println!("Texto       : {texto}");
    println!("Normalizado : {}", classificador.normalizar(texto));
    println!("Regras      : {}", classificador.fonte().display());
    println!("Conta       : {} {}", r.conta.id, r.conta.nome);
    println!("Grupo       : {}", r.conta.grupo);
    println!("Confianca   : {:?}", r.confianca);
    println!(
        "Match       : {}",
        if r.matches.is_empty() {
            "NENHUM".to_string()
        } else {
            r.matches.join(", ")
        }
    );
    for alerta in &r.alertas {
        println!("Alerta      : {alerta}");
    }

    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// auditar-aliases
// ---------------------------------------------------------------------------

fn auditar_aliases(
    caminho_plano: &PathBuf,
    caminho_regras: &PathBuf,
    estrito: bool,
) -> Result<ExitCode, String> {
    let (plano, regras) = carregar_validado(caminho_plano, caminho_regras)?;
    let classificador = Classificador::compilar(&regras);
    let linhas = classificador
        .auditar_aliases(&plano)
        .map_err(|e| e.to_string())?;

    println!("AUDITORIA DE ALIAS");
    println!("==================");
    println!("Regras   : {}", classificador.fonte().display());
    println!(
        "Cobertura: {} alias em {} regras",
        linhas.len(),
        regras.regras().len()
    );
    println!();

    let sem_classificacao: Vec<_> = linhas.iter().filter(|a| a.nao_classificado()).collect();
    let divergentes: Vec<_> = linhas.iter().filter(|a| a.diverge()).collect();
    let ambiguos: Vec<_> = linhas.iter().filter(|a| a.ambiguo()).collect();

    if !sem_classificacao.is_empty() {
        println!("ALIAS SEM CLASSIFICACAO (regra morta)");
        for a in &sem_classificacao {
            println!("  {:<30} declarado em {}", a.alias, a.conta_declarada);
        }
        println!();
    }

    if !divergentes.is_empty() {
        println!("ALIAS QUE LEVAM A OUTRA CONTA");
        println!("  (nao e' automaticamente um erro: outra regra e' mais especifica");
        println!("   ou tem prioridade maior. Mas tem de ser uma escolha vista.)");
        for a in &divergentes {
            println!(
                "  {:<30} declarado {} -> vence {} [{:?}] match: {}",
                a.alias,
                a.conta_declarada,
                a.conta_vencedora,
                a.confianca,
                a.matches.join(", ")
            );
        }
        println!();
    }

    if !ambiguos.is_empty() {
        println!("ALIAS AMBIGUOS");
        for a in &ambiguos {
            println!(
                "  {:<30} declarado {} -> vence {} match: {}",
                a.alias,
                a.conta_declarada,
                a.conta_vencedora,
                a.matches.join(", ")
            );
        }
        println!();
    }

    println!(
        "RESUMO: {} alias, {} sem classificacao, {} divergentes, {} ambiguos",
        linhas.len(),
        sem_classificacao.len(),
        divergentes.len(),
        ambiguos.len()
    );

    let reprovado = !sem_classificacao.is_empty()
        || (estrito && (!divergentes.is_empty() || !ambiguos.is_empty()));
    println!(
        "STATUS: {}",
        if reprovado { "REPROVADO" } else { "APROVADO" }
    );

    Ok(if reprovado {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_esta_bem_definida() {
        Cli::command().debug_assert();
    }

    /// Os caminhos padrao devem apontar para arquivos realmente versionados.
    #[test]
    fn caminhos_padrao_existem_no_repositorio() {
        let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for padrao in [PADRAO_PLANO, PADRAO_REGRAS_PRODUTOS, PADRAO_REGRAS_DESPESAS] {
            assert!(
                raiz.join(padrao).is_file(),
                "arquivo de configuracao ausente: {padrao}"
            );
        }
    }

    /// Auditoria automatizada: as regras reais do repositorio nao podem conter ERROS.
    /// Este teste e' o contrato que impede de comitar um YAML quebrado.
    #[test]
    fn configuracao_real_do_repositorio_nao_tem_erros() {
        let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let plano = PlanoContas::carregar(raiz.join(PADRAO_PLANO)).expect("plano de contas");
        let produtos =
            RegrasClassificacao::carregar(raiz.join(PADRAO_REGRAS_PRODUTOS)).expect("produtos");
        let despesas =
            RegrasClassificacao::carregar(raiz.join(PADRAO_REGRAS_DESPESAS)).expect("despesas");

        let mut problemas = plano.validar();
        problemas.extend(produtos.validar(&plano));
        problemas.extend(despesas.validar(&plano));

        let erros: Vec<&Problema> = problemas
            .iter()
            .filter(|p| p.severidade == Severidade::Erro)
            .collect();
        assert!(erros.is_empty(), "configuracao real com erros: {erros:#?}");
    }
}

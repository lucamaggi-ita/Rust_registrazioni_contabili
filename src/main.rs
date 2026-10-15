//! CLI do porte Rust do projeto `fantozzi` (registros contabeis de NF-e).
//!
//! Fase 1: modelo de dados + leitura contratualizada das regras YAML.
//! Fase 2 (ainda nao implementada): parser de NF-e com `quick-xml`,
//! normalizacao e classificacao.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use rust_schede_tecniche::regras::{
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
                  Fase 1: modelo de dados e validacao das regras versionadas em config/."
)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.comando {
        Comando::ValidarConfig {
            plano_contas,
            regras_produtos,
            regras_despesas,
            estrito,
        } => match validar_config(&plano_contas, &regras_produtos, &regras_despesas, estrito) {
            Ok(codigo) => codigo,
            Err(erro) => {
                eprintln!("ERRO DE CARREGAMENTO: {erro}");
                ExitCode::FAILURE
            }
        },
    }
}

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

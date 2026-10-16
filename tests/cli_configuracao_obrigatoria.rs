//! Nenhum comando pode usar uma configuracao que `validar-config` reprova.
//!
//! Regressao do riscontro P2: `classificar` e `importar-nfe` aceitavam o mesmo
//! arquivo de regras com fallback errado que `validar-config` rejeita, e
//! mandavam produtos desconhecidos para uma conta de custo com codigo 0.

use std::path::PathBuf;
use std::process::Command;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_defeituosa() -> String {
    raiz()
        .join("tests/fixtures/regras_produtos_fallback_custo.yaml")
        .to_str()
        .unwrap()
        .to_string()
}

/// Devolve (sucesso, stdout, stderr).
fn executar(args: &[&str]) -> (bool, String, String) {
    let saida = Command::new(env!("CARGO_BIN_EXE_rust_registrazioni_contabili"))
        .current_dir(raiz())
        .args(args)
        .output()
        .expect("falha ao executar a CLI");
    (
        saida.status.success(),
        String::from_utf8_lossy(&saida.stdout).into_owned(),
        String::from_utf8_lossy(&saida.stderr).into_owned(),
    )
}

#[test]
fn classificar_recusa_configuracao_reprovada() {
    let (ok, stdout, stderr) = executar(&[
        "classificar",
        "--regras",
        &fixture_defeituosa(),
        "--texto",
        "PRODUTO XYZ DESCONHECIDO",
    ]);
    assert!(!ok, "deveria sair com erro:\n{stdout}\n{stderr}");
    assert!(
        !stdout.contains("Cervejas"),
        "um produto desconhecido nao pode cair em Cervejas:\n{stdout}"
    );
    assert!(stderr.contains("C6_FALLBACK_NAO_E_REVISAO"), "{stderr}");
}

#[test]
fn importar_nfe_recusa_configuracao_reprovada() {
    let (ok, stdout, stderr) = executar(&[
        "importar-nfe",
        "--xml",
        "samples/nfe_exemplo/nfe_oesa_12566_20260505.xml",
        "--regras-produtos",
        &fixture_defeituosa(),
    ]);
    assert!(!ok, "deveria sair com erro:\n{stdout}\n{stderr}");
    assert!(stderr.contains("configuracao reprovada"), "{stderr}");
}

#[test]
fn auditar_aliases_recusa_configuracao_reprovada() {
    let (ok, _stdout, stderr) = executar(&["auditar-aliases", "--regras", &fixture_defeituosa()]);
    assert!(!ok, "{stderr}");
}

/// Os alertas C13 nao podem ficar invisiveis nos comandos operacionais.
#[test]
fn comandos_mostram_alertas_da_configuracao() {
    let (ok, _stdout, stderr) = executar(&["classificar", "--texto", "CERVEJA HEINEKEN"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("C13_SECAO_NAO_APLICADA"),
        "o alerta de regra declarada e nao aplicada deve aparecer:\n{stderr}"
    );
}

/// A configuracao versionada continua aceita pelos comandos operacionais.
#[test]
fn configuracao_real_e_aceita_pelos_comandos() {
    let (ok, stdout, stderr) = executar(&[
        "importar-nfe",
        "--xml",
        "samples/nfe_exemplo/nfe_oesa_12566_20260505.xml",
    ]);
    assert!(ok, "{stdout}\n{stderr}");
    assert!(stdout.contains("0 a revisar"), "{stdout}");
}

/// A auditoria de alias relata a baseline conhecida das regras canonicas.
#[test]
fn auditoria_de_alias_relata_baseline() {
    let (ok, stdout, stderr) = executar(&["auditar-aliases", "--tipo", "produtos"]);
    assert!(ok, "{stdout}\n{stderr}");
    assert!(
        stdout.contains("RESUMO: 454 alias, 0 sem classificacao, 6 divergentes, 9 ambiguos"),
        "{stdout}"
    );

    let (ok, stdout, stderr) = executar(&["auditar-aliases", "--tipo", "despesas"]);
    assert!(ok, "{stdout}\n{stderr}");
    assert!(
        stdout.contains("RESUMO: 117 alias, 0 sem classificacao, 1 divergentes, 2 ambiguos"),
        "{stdout}"
    );
}

/// Com `--estrito`, divergencias e ambiguidades reprovam.
#[test]
fn auditoria_estrita_reprova_divergencias() {
    let (ok, stdout, _stderr) = executar(&["auditar-aliases", "--tipo", "produtos", "--estrito"]);
    assert!(!ok, "{stdout}");
    assert!(stdout.contains("STATUS: REPROVADO"), "{stdout}");
}

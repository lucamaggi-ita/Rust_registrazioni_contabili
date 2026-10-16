//! `importar-nfe` le uma NF-e por arquivo, nunca a primeira de varias.
//!
//! Regressao do riscontro A1 (audit independente de 2026-10-16): um XML com
//! duas raizes (dois documentos concatenados) saia com codigo 0, mostrava so
//! a NF-e 12566 e descartava a 12567 em silencio. Contrato C17: erro
//! explicito, sem sucesso parcial.

use std::path::PathBuf;
use std::process::Command;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
fn duas_raizes_reprovam_sem_sucesso_parcial() {
    let (ok, stdout, stderr) = executar(&[
        "importar-nfe",
        "--xml",
        "tests/fixtures/nfe_duas_raizes.xml",
    ]);

    assert!(!ok, "deveria sair com erro:\n{stdout}\n{stderr}");
    assert!(
        stderr.contains("mais de um elemento raiz"),
        "o erro deve nomear a causa:\n{stderr}"
    );
    // Nenhuma das duas notas pode aparecer como lida.
    assert!(
        !stdout.contains("12566") && !stdout.contains("12567"),
        "nenhuma NF-e pode ser mostrada como importada:\n{stdout}"
    );
    assert!(
        !stdout.contains("ITENS E CLASSIFICACAO"),
        "nao pode haver classificacao parcial:\n{stdout}"
    );
}

/// Riscontro B1: `<infNFe>` aninhado dentro de outro. XML bem formado, duas
/// notas, uma raiz so: antes da correcao saia 0 mostrando so a 12566.
#[test]
fn infnfe_aninhado_reprova_sem_sucesso_parcial() {
    let (ok, stdout, stderr) = executar(&[
        "importar-nfe",
        "--xml",
        "tests/fixtures/nfe_infnfe_annidata.xml",
    ]);

    assert!(!ok, "deveria sair com erro:\n{stdout}\n{stderr}");
    assert!(
        stderr.contains("2 <infNFe> no mesmo documento"),
        "o erro deve dizer quantas notas encontrou:\n{stderr}"
    );
    assert!(
        !stdout.contains("12566") && !stdout.contains("12567"),
        "nenhuma NF-e pode ser mostrada como importada:\n{stdout}"
    );
    assert!(
        !stdout.contains("ITENS E CLASSIFICACAO"),
        "nao pode haver classificacao parcial:\n{stdout}"
    );
}

/// O caso bom continua bom: o campione canonico tem uma raiz e uma NF-e.
#[test]
fn documento_canonico_continua_aprovado() {
    let (ok, stdout, stderr) = executar(&[
        "importar-nfe",
        "--xml",
        "samples/nfe_exemplo/nfe_oesa_12566_20260505.xml",
    ]);

    assert!(ok, "o campione canonico deve ser lido:\n{stdout}\n{stderr}");
    assert!(stdout.contains("NF-e 12566"), "{stdout}");
    assert!(stdout.contains("Cobranca     : 35.00"), "{stdout}");
    assert!(stdout.contains("3 item(ns)"), "{stdout}");
}

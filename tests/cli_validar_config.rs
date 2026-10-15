//! Teste de regressao do codigo de saida da CLI (riscontro R1).
//!
//! Contrato: uma configuracao que viola C5/C6 deve REPROVAR na CLI padrao,
//! sem depender do flag opcional `--estrito`.

use std::path::PathBuf;
use std::process::Command;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn validar_config(args: &[&str]) -> (bool, String) {
    let saida = Command::new(env!("CARGO_BIN_EXE_rust_schede_tecniche"))
        .current_dir(raiz())
        .arg("validar-config")
        .args(args)
        .output()
        .expect("falha ao executar a CLI");
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    (saida.status.success(), texto)
}

#[test]
fn configuracao_real_do_repositorio_e_aprovada() {
    let (ok, texto) = validar_config(&[]);
    assert!(
        ok,
        "a configuracao versionada deveria ser aprovada:\n{texto}"
    );
    assert!(texto.contains("STATUS: APROVADO"), "{texto}");
}

/// R1: fallback fora da area de revisao reprova mesmo sem `--estrito`.
#[test]
fn fallback_fora_da_area_de_revisao_reprova_sem_estrito() {
    let fixture = raiz().join("tests/fixtures/regras_produtos_fallback_custo.yaml");
    let (ok, texto) = validar_config(&["--regras-produtos", fixture.to_str().unwrap()]);
    assert!(!ok, "deveria sair com codigo de erro:\n{texto}");
    assert!(texto.contains("C6_FALLBACK_NAO_E_REVISAO"), "{texto}");
    assert!(
        texto.contains("[ERRO"),
        "o problema deve ser classificado como erro:\n{texto}"
    );
    assert!(texto.contains("STATUS: REPROVADO"), "{texto}");
}

/// O flag `--estrito` continua existindo para os alertas, nao para os erros.
#[test]
fn modo_estrito_tambem_reprova_o_mesmo_caso() {
    let fixture = raiz().join("tests/fixtures/regras_produtos_fallback_custo.yaml");
    let (ok, texto) =
        validar_config(&["--estrito", "--regras-produtos", fixture.to_str().unwrap()]);
    assert!(!ok, "{texto}");
}

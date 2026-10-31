//! Il percorso della fase 3C.5.b dalla CLI: consultare, annotare, approvare,
//! revocare, e rigenerare l'export.
//!
//! Perché test di integrazione: questi contratti stanno negli esiti del
//! processo e nell'effetto sull'archivio. I contratti di `revisao.rs` — ordine
//! canonico, conflitti, i cinque motivi di non applicazione — sono già provati
//! nei suoi test di unità e qui non vengono ridimostrati: si prova che la CLI
//! li **rispetta** e che non archivia mai ciò che li violerebbe.

use std::path::{Path, PathBuf};
use std::process::Command;

use rust_registrazioni_contabili::arquivo_io::{self, Destino};
use rust_registrazioni_contabili::declaracao::{self, LinhaConsultada};
use rust_registrazioni_contabili::regras::PlanoContas;
use rust_registrazioni_contabili::revisao;

const CHAVE_A: &str = "29260512345678000195550010001256612345678901";
const CHAVE_B: &str = "29260512345678000195550010001256712345678902";
const PLANO: &str = "config/plano_contas_gerencial.yaml";
const INSTANTE: &str = "2026-10-31T09:00:00-03:00";

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn campione() -> String {
    std::fs::read_to_string(raiz().join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml"))
        .expect("o campione versionado")
}

/// Una NF-e derivata dal campione: solo chiave e numero cambiati. La chiave
/// contiene il numero, quindi si sostituisce prima la chiave intera.
fn derivar(chave: &str, numero: &str) -> String {
    campione()
        .replace(CHAVE_A, chave)
        .replace("<nNF>12566</nNF>", &format!("<nNF>{numero}</nNF>"))
}

fn arena(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("fantozzi_3c5b")
        .join(format!("{nome}_{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar a arena");
    }
    std::fs::create_dir_all(dir.join("entrada")).expect("criar a entrada");
    std::fs::create_dir_all(dir.join("dados")).expect("criar os dados");
    dir
}

fn executar(args: &[&str]) -> (i32, String, String) {
    let saida = Command::new(env!("CARGO_BIN_EXE_rust_registrazioni_contabili"))
        .current_dir(raiz())
        .args(args)
        .output()
        .expect("executar a CLI");
    (
        saida.status.code().expect("um codigo de saida"),
        String::from_utf8_lossy(&saida.stdout).into_owned(),
        String::from_utf8_lossy(&saida.stderr).into_owned(),
    )
}

fn sha256(caminho: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(std::fs::read(caminho).expect("ler"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

struct Caminhos {
    entrada: String,
    arquivo: String,
    saida: String,
}

fn caminhos(dir: &Path) -> Caminhos {
    Caminhos {
        entrada: dir.join("entrada").display().to_string(),
        arquivo: dir.join("dados/arquivo.json").display().to_string(),
        saida: dir.join("dados/diagnostico.xlsx").display().to_string(),
    }
}

/// Archivia una cartella con due NF-e e restituisce i percorsi.
fn com_duas_nfe(nome: &str, lote: &str) -> (PathBuf, Caminhos) {
    let dir = arena(nome);
    std::fs::write(dir.join("entrada/a.xml"), campione()).expect("escrever a");
    std::fs::write(dir.join("entrada/b.xml"), derivar(CHAVE_B, "12567")).expect("escrever b");
    let c = caminhos(&dir);
    let args = vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--inicializar",
        "--lote",
        lote,
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "importacao do teste",
        "--instante",
        INSTANTE,
    ];
    assert_eq!(executar(&args).0, 0, "a importacao prepara o percurso");
    (dir, c)
}

/// Le righe come la CLI le vede, lette con l'API pubblica: evita di
/// interpretare l'uscita stampata per ottenere le due impronte.
fn linhas_de(arquivo: &str, lote: &str) -> Vec<LinhaConsultada> {
    let plano = PlanoContas::carregar(raiz().join(PLANO)).expect("o plano");
    let caminho = Path::new(arquivo);
    let destino = Destino::novo(
        caminho.parent().expect("o diretorio").to_path_buf(),
        caminho
            .file_name()
            .expect("o nome")
            .to_string_lossy()
            .to_string(),
    );
    let leitura = arquivo_io::ler(&destino).expect("o arquivo publicado");
    let contextos = leitura.arquivo.contextos_do_lote(lote);
    let eventos = leitura.arquivo.eventos();
    let vista = revisao::aplicar_revisao(&contextos, &plano, &eventos).expect("a vista");
    declaracao::consultar(&vista)
}

/// La riga di **quel** documento e di quel conto.
///
/// Le due NF-e del percorso derivano dallo stesso campione, quindi entrambe
/// hanno una riga per ogni conto: cercare per il solo conto prenderebbe la
/// prima, e si dichiarerebbe la chiave di un documento con l'impronta
/// dell'altro.
fn linha_de(arquivo: &str, lote: &str, chave: &str, conta: &str) -> LinhaConsultada {
    linhas_de(arquivo, lote)
        .into_iter()
        .find(|l| l.chave_origem == chave && l.conta_id == conta)
        .unwrap_or_else(|| panic!("a linha {chave}/{conta} em {lote}"))
}

fn linha_da_conta(arquivo: &str, lote: &str, conta: &str) -> LinhaConsultada {
    linha_de(arquivo, lote, CHAVE_A, conta)
}

/// Gli argomenti di una dichiarazione, con l'impronta **dichiarata** a parte:
/// è il dato che la persona copia, e un test non deve farselo dedurre.
#[allow(clippy::too_many_arguments)]
fn args_declarar<'a>(
    c: &'a Caminhos,
    lote: &'a str,
    chave: &'a str,
    conta: &'a str,
    operacao: &'a str,
    autor: &'a str,
    motivo: &'a str,
    impressao: &'a str,
    extra: &[&'a str],
) -> Vec<&'a str> {
    let mut args = vec![
        "declarar-revisao",
        "--arquivo",
        &c.arquivo,
        "--lote",
        lote,
        "--chave",
        chave,
        "--conta",
        conta,
        "--origem",
        "xml-nfe",
        "--operacao",
        operacao,
        "--autor",
        autor,
        "--motivo",
        motivo,
        "--impressao-conteudo",
        impressao,
    ];
    // Clap **rifiuta** un'opzione ripetuta — misurato: «the argument
    // '--instante <INSTANTE>' cannot be used multiple times» — quindi i
    // predefiniti si aggiungono solo se chi chiama non li dichiara.
    if !extra.contains(&"--instante") {
        args.push("--instante");
        args.push(INSTANTE);
    }
    // L'ordinale e' quello della riga **dentro il documento**: `2.01.004` e'
    // 0, `2.01.002` e' 1. Chi guarda una riga diversa dalla prima lo dichiara.
    if !extra.contains(&"--ordinal") {
        args.push("--ordinal");
        args.push("0");
    }
    args.extend_from_slice(extra);
    args
}

fn args_exportar<'a>(c: &'a Caminhos, lote: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "exportar-diagnostico",
        "--arquivo",
        &c.arquivo,
        "--lote",
        lote,
        "--saida",
        &c.saida,
        "--operador",
        "Luca Maggi",
        "--motivo",
        "regeneracao do teste",
        "--instante",
        INSTANTE,
    ];
    args.extend_from_slice(extra);
    args
}

// ===========================================================================
// Consultazione
// ===========================================================================

/// `REV-03`: la consultazione non scrive, e mostra i cinque stati separati.
#[test]
fn tx70_consultar_nao_escreve() {
    let (_dir, c) = com_duas_nfe("tx70", "maio-r1");
    let antes = sha256(&c.arquivo);

    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--conta",
        "2.01.004",
    ]);
    assert_eq!(codigo, 0, "{stdout}");
    assert!(stdout.contains("status_tecnico"), "{stdout}");
    assert!(stdout.contains("estado_revisao   TECNICO_OK"), "{stdout}");
    assert!(stdout.contains("particao"), "{stdout}");
    assert!(stdout.contains("stato_documento"), "{stdout}");
    assert!(stdout.contains("no_extrato_revisado"), "{stdout}");
    // Le impronte intere, non troncate: sono quelle che un evento deve citare.
    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    let impressao = linha.impressao_conteudo.expect("a impressao da linha");
    assert_eq!(impressao.len(), 64);
    assert!(
        stdout.contains(&impressao),
        "a impressao inteira:\n{stdout}"
    );
    assert!(
        stdout.contains("--impressao-conteudo"),
        "o comando pronto a copiar:\n{stdout}"
    );
    assert_eq!(antes, sha256(&c.arquivo), "a consulta nao escreve");

    // Nessuna riga che corrisponda al filtro non e' un errore.
    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--conta",
        "9.99.999",
    ]);
    assert_eq!(codigo, 0, "um elenco vazio nao e' um erro:\n{stdout}");
    assert!(stdout.contains("0 mostrada(s)"), "{stdout}");
}

/// `REV-04`, `R3C5B-01`: il filtro guarda la **provenienza umana**, non la
/// partizione.
///
/// La riga `APROVADO_SEM_PROVENIENCIA` non si può ottenere dalla CLI:
/// `alocar_nfe` produce soltanto `OK` e `REVISAR`. Si costruisce quindi con
/// l'API pubblica, mutando lo stato tecnico del lancamento prima di
/// archiviare — ed è esattamente il caso che D9.4 modella: uno stato
/// `APROVADO` in ingresso che nessun evento giustifica.
#[test]
fn tx70b_sem_proveniencia_aparece() {
    use rust_registrazioni_contabili::arquivo::{Arquivo, IdentidadePlano};
    use rust_registrazioni_contabili::arquivo_io::MetadadosLucchetto;
    use rust_registrazioni_contabili::classificacao::Classificador;
    use rust_registrazioni_contabili::importacao;
    use rust_registrazioni_contabili::modelos::StatusRevisao;
    use rust_registrazioni_contabili::nfe;
    use rust_registrazioni_contabili::prelivro::IdentidadeLote;
    use rust_registrazioni_contabili::regras::RegrasClassificacao;

    let dir = arena("tx70b");
    std::fs::write(dir.join("entrada/a.xml"), campione()).expect("escrever");
    let c = caminhos(&dir);

    let plano = PlanoContas::carregar(raiz().join(PLANO)).expect("o plano");
    let regras =
        RegrasClassificacao::carregar(raiz().join("config/regras_classificacao_produtos.yaml"))
            .expect("as regras");
    let classificador = Classificador::compilar(&regras);
    let lote = IdentidadeLote {
        id: "maio-r1".to_string(),
        gerado_em: chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        descricao: "pasta de maio".to_string(),
    };
    let importada = nfe::ler_nfe(dir.join("entrada/a.xml")).expect("a NF-e");
    let mut contexto = importacao::contexto_de(&importada, "a.xml", &classificador, &plano, &lote)
        .expect("o contexto");
    // Uno stato `APROVADO` in ingresso, che nessun evento giustifica.
    contexto.alocacao.lancamentos[0].status_revisao = StatusRevisao::Aprovado;

    let mut arquivo = Arquivo::novo();
    arquivo
        .acrescentar_lote(
            std::slice::from_ref(&contexto),
            &plano,
            IdentidadePlano {
                versao: plano.versao().to_string(),
                sha256_arquivo: None,
                caminho_declarado: Some(PLANO.to_string()),
            },
        )
        .expect("o lote");
    let destino = Destino::novo(dir.join("dados"), "arquivo.json".to_string());
    arquivo_io::publicar(
        &destino,
        &arquivo,
        None,
        &MetadadosLucchetto {
            operador: "teste".to_string(),
            instante: chrono::DateTime::parse_from_rfc3339(INSTANTE).unwrap(),
            motivo: "preparar o caso D9.4".to_string(),
        },
    )
    .expect("publicar");

    // La riga ha stato ereditato APROVADO senza provenienza, e la 3C.2 le da'
    // particao APROVADO: filtrare per particao la nasconderebbe.
    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    assert_eq!(
        linha.estado,
        revisao::EstadoRevisao::AprovadoSemProveniencia
    );
    assert_eq!(
        linha.particao.rotulo(),
        "APROVADO",
        "e' il caso che renderebbe muto un filtro per partizione"
    );
    assert!(linha.sem_aprovacao_humana());

    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--so-sem-aprovacao-humana",
        "--conta",
        "2.01.004",
    ]);
    assert_eq!(codigo, 0, "{stdout}");
    assert!(
        stdout.contains("APROVADO_SEM_PROVENIENCIA"),
        "la riga da sanare deve comparire:\n{stdout}"
    );
    assert!(stdout.contains("1 mostrada(s)"), "{stdout}");
}

// ===========================================================================
// Dichiarare
// ===========================================================================

/// `REV-01`: approvazione applicata, e l'export rigenerato la mostra.
#[test]
fn tx71_aprovar_e_regenerar() {
    let (_dir, c) = com_duas_nfe("tx71", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");

    let (codigo, stdout, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "aprovar",
        "Luca Maggi",
        "conferido com a nota em papel",
        &impressao,
        &[],
    ));
    assert_eq!(codigo, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("Evento             : APLICADO"), "{stdout}");
    assert!(
        stdout.contains("TECNICO_OK -> APROVADO_POR_PESSOA"),
        "{stdout}"
    );
    assert!(stdout.contains("Revisao            : 2"), "{stdout}");

    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    assert_eq!(linha.estado, revisao::EstadoRevisao::AprovadoPorPessoa);
    assert!(linha.no_extrato_revisado, "entra no extrato revisado");
    assert_eq!(linha.decisoes_aplicadas, 1);

    let (codigo, stdout, _) = executar(&args_exportar(&c, "maio-r1", &[]));
    assert_eq!(codigo, 0, "{stdout}");
    assert!(stdout.contains("na revisao 2"), "{stdout}");
}

/// `REV-05`: la revoca, e la revoca senza niente da revocare.
#[test]
fn tx72_revogar() {
    let (_dir, c) = com_duas_nfe("tx72", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");

    // Revoca senza approvazione applicata: niente da revocare.
    let antes = sha256(&c.arquivo);
    let (codigo, _, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "revogar",
        "Luca Maggi",
        "tentativa sem aprovacao",
        &impressao,
        &[],
    ));
    assert_eq!(codigo, 3, "{stderr}");
    assert!(stderr.contains("NADA_A_REVOGAR"), "{stderr}");
    assert_eq!(antes, sha256(&c.arquivo), "nada publicado");

    // Approvazione, poi revoca a un istante successivo.
    assert_eq!(
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido",
            &impressao,
            &[]
        ))
        .0,
        0
    );
    let (codigo, stdout, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "revogar",
        "Luca Maggi",
        "a nota era de outro mes",
        &impressao,
        &["--instante", "2026-10-31T10:00:00-03:00"],
    ));
    assert_eq!(codigo, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("APROVACAO_REVOGADA"), "{stdout}");

    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    assert_eq!(linha.estado, revisao::EstadoRevisao::AprovacaoRevogada);
    assert!(!linha.no_extrato_revisado, "sai do extrato revisado");
}

/// `REV-05`: `ANOTAR` si applica **senza** decidere lo stato.
#[test]
fn tx73_anotar_nao_decide() {
    let (_dir, c) = com_duas_nfe("tx73", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");

    // Senza `--nota` la barriera rifiuta: e' la forma dell'evento.
    let antes = sha256(&c.arquivo);
    let (codigo, _, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "anotar",
        "Luca Maggi",
        "duvida",
        &impressao,
        &[],
    ));
    assert_eq!(codigo, 2, "a barreira recusa a forma:\n{stderr}");
    assert_eq!(antes, sha256(&c.arquivo), "nada publicado");

    let (codigo, stdout, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "anotar",
        "Luca Maggi",
        "duvida sobre o rateio",
        &impressao,
        &["--nota", "conferir com o fornecedor antes do fecho"],
    ));
    assert_eq!(codigo, 0, "{stdout}\n{stderr}");
    assert!(stdout.contains("Evento             : APLICADO"), "{stdout}");
    // Applicato **senza** cambiare lo stato: e' il punto.
    assert!(
        stdout.contains("TECNICO_OK -> TECNICO_OK"),
        "o estado nao muda:\n{stdout}"
    );
    assert!(stdout.contains("(identico"), "e lo si dice:\n{stdout}");

    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    assert_eq!(linha.anotacoes, 1);
    assert_eq!(linha.decisoes_aplicadas, 0, "uma anotacao nao e' decisao");
    assert_eq!(linha.estado, revisao::EstadoRevisao::TecnicoOk);
}

/// `REV-02`: l'impronta è dichiarata, e una stantia è un rifiuto.
#[test]
fn tx74_impressao_estantia_recusada() {
    let (_dir, c) = com_duas_nfe("tx74", "maio-r1");
    let antes = sha256(&c.arquivo);

    let (codigo, _, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "aprovar",
        "Luca Maggi",
        "impressao de outra versao",
        &"0".repeat(64),
        &[],
    ));
    assert_eq!(codigo, 3, "{stderr}");
    assert!(stderr.contains("VERSAO_SUPERADA"), "{stderr}");
    assert!(
        stderr.contains("publica apenas o evento aplicavel ao alvo selecionado"),
        "o motivo do recuso e' o do mandato:\n{stderr}"
    );
    assert_eq!(antes, sha256(&c.arquivo), "nada publicado");
}

/// `REV-06`, M3: la decisione sopravvive a un rilancio della cartella, e
/// **non** passa a un contenuto diverso.
#[test]
fn tx75_aprovacao_sobrevive_ao_relancamento() {
    let (dir, c) = com_duas_nfe("tx75", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");
    assert_eq!(
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido",
            &impressao,
            &[]
        ))
        .0,
        0
    );

    // Rilancio della cartella: esecuzione nuova, documenti identici.
    let args = vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r2",
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "relancamento",
        "--instante",
        INSTANTE,
    ];
    assert_eq!(executar(&args).0, 0);

    // Senza nessuna dichiarazione nuova, la decisione si ritrova.
    let linha = linha_da_conta(&c.arquivo, "maio-r2", "2.01.004");
    assert_eq!(
        linha.estado,
        revisao::EstadoRevisao::AprovadoPorPessoa,
        "mesma chave e mesma impressao: a decisao segue o documento"
    );
    assert_eq!(linha.decisoes_aplicadas, 1);

    // Un documento di contenuto **diverso** non eredita niente: la terza NF-e
    // ha un'altra chiave, e nessuno l'ha approvata.
    std::fs::write(
        dir.join("entrada/c.xml"),
        derivar(&format!("{}3", &CHAVE_A[..43]), "12568"),
    )
    .expect("escrever c");
    let args = vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r3",
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "terceira NF-e",
        "--instante",
        INSTANTE,
    ];
    assert_eq!(executar(&args).0, 0);
    let novas: Vec<LinhaConsultada> = linhas_de(&c.arquivo, "maio-r3")
        .into_iter()
        .filter(|l| l.chave_origem != CHAVE_A)
        .collect();
    assert!(
        novas
            .iter()
            .all(|l| l.estado != revisao::EstadoRevisao::AprovadoPorPessoa),
        "um conteudo diverso nao herda a aprovacao"
    );
}

/// La forma rifiutata non entra nell'archivio, e l'archivio **resta usabile**.
#[test]
fn tx76_forma_recusada_nao_entra_no_arquivo() {
    let (_dir, c) = com_duas_nfe("tx76", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");
    let antes = sha256(&c.arquivo);

    for (autor, motivo) in [("", "um motivo"), ("Luca Maggi", "")] {
        let (codigo, _, stderr) = executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            autor,
            motivo,
            &impressao,
            &[],
        ));
        assert_eq!(codigo, 2, "autor {autor:?} motivo {motivo:?}:\n{stderr}");
        assert_eq!(antes, sha256(&c.arquivo), "nada publicado");
    }

    // E, soprattutto: l'archivio e' ancora usabile.
    let (codigo, stdout, _) = executar(&args_exportar(&c, "maio-r1", &[]));
    assert_eq!(codigo, 0, "o arquivo continua utilizavel:\n{stdout}");
}

/// `REV-07`: ripetizione esatta rifiutata, dichiarazione distinta ammessa.
#[test]
fn tx77_repeticao_e_declaracao_distinta() {
    let (_dir, c) = com_duas_nfe("tx77", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");
    let aprovar = |extra: &[&str]| {
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido",
            &impressao,
            extra,
        ))
    };
    assert_eq!(aprovar(&[]).0, 0);
    let depois_da_primeira = sha256(&c.arquivo);

    // **Tutti** i dati dichiarati ripetuti, istante compreso.
    let (codigo, _, stderr) = aprovar(&[]);
    assert_eq!(codigo, 3, "{stderr}");
    assert!(stderr.contains("REPETICAO"), "{stderr}");
    assert!(
        stderr.contains("instante e offset incluidos"),
        "il motivo nomina l'identita' esatta:\n{stderr}"
    );
    assert_eq!(depois_da_primeira, sha256(&c.arquivo), "nada publicado");

    // Istante diverso: dichiarazione **distinta**, non ripetizione.
    let (codigo, stdout, stderr) = aprovar(&["--instante", "2026-10-31T11:00:00-03:00"]);
    assert_eq!(
        codigo, 0,
        "uma aprovacao a outro instante e' outra declaracao:\n{stdout}\n{stderr}"
    );
    let linha = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    assert_eq!(
        linha.decisoes_aplicadas, 2,
        "duas decisoes, nao uma repetida"
    );
}

/// `REV-08`: un evento che riguarda un altro lotto non blocca niente.
#[test]
fn tx78_evento_estraneo_non_blocca() {
    let (dir, c) = com_duas_nfe("tx78", "maio-r1");
    // La seconda NF-e esce dalla cartella: la nuova esecuzione non la contiene.
    let linha_b = linha_de(&c.arquivo, "maio-r1", CHAVE_B, "2.01.002");
    let impressao_b = linha_b
        .impressao_conteudo
        .clone()
        .expect("a impressao de b");
    let ordinal_b = linha_b.ordinal.to_string();
    assert_eq!(
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_B,
            "2.01.002",
            "aprovar",
            "Luca Maggi",
            "aprovada na primeira execucao",
            &impressao_b,
            &["--ordinal", &ordinal_b]
        ))
        .0,
        0
    );
    std::fs::remove_file(dir.join("entrada/b.xml")).expect("remover b");
    let args = vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r2",
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "relancamento sem b",
        "--instante",
        INSTANTE,
    ];
    assert_eq!(executar(&args).0, 0);

    // Nella vista di `maio-r2` l'evento di B e' ALVO_AUSENTE: normale.
    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r2",
        "--mostrar-nao-aplicados",
    ]);
    assert_eq!(codigo, 0, "{stdout}");
    assert!(stdout.contains("ALVO_AUSENTE"), "{stdout}");

    // E una dichiarazione valida su A si pubblica comunque.
    let impressao_a = linha_da_conta(&c.arquivo, "maio-r2", "2.01.004")
        .impressao_conteudo
        .expect("a impressao de a");
    let (codigo, stdout, stderr) = executar(&args_declarar(
        &c,
        "maio-r2",
        CHAVE_A,
        "2.01.004",
        "aprovar",
        "Luca Maggi",
        "aprovada na segunda execucao",
        &impressao_a,
        &["--instante", "2026-10-31T12:00:00-03:00"],
    ));
    assert_eq!(
        codigo, 0,
        "um evento estranho nao impede uma declaracao valida:\n{stdout}\n{stderr}"
    );

    // E l'evento storico resta nell'archivio.
    let plano = PlanoContas::carregar(raiz().join(PLANO)).expect("o plano");
    let caminho = Path::new(&c.arquivo);
    let destino = Destino::novo(
        caminho.parent().unwrap().to_path_buf(),
        "arquivo.json".to_string(),
    );
    let leitura = arquivo_io::ler(&destino).expect("o arquivo");
    assert_eq!(
        leitura.arquivo.eventos().len(),
        2,
        "a historia fica inteira"
    );
    let _ = plano;
}

/// `REV-09`: una coppia incompatibile non viene mai archiviata.
#[test]
fn tx79_decisoes_incompativeis() {
    let (_dir, c) = com_duas_nfe("tx79", "maio-r1");
    let impressao = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004")
        .impressao_conteudo
        .expect("a impressao");
    assert_eq!(
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido",
            &impressao,
            &[]
        ))
        .0,
        0
    );
    let antes = sha256(&c.arquivo);

    // Revoca allo **stesso** istante dell'approvazione, senza sequenza: due
    // decisioni incompatibili non ordinabili.
    let (codigo, _, stderr) = executar(&args_declarar(
        &c,
        "maio-r1",
        CHAVE_A,
        "2.01.004",
        "revogar",
        "Outra Pessoa",
        "revogo no mesmo instante",
        &impressao,
        &[],
    ));
    assert_eq!(codigo, 2, "a barreira para antes de escrever:\n{stderr}");
    assert_eq!(antes, sha256(&c.arquivo), "nada publicado");

    // L'archivio e l'export precedenti restano utilizzabili.
    let (codigo, stdout, _) = executar(&args_exportar(&c, "maio-r1", &[]));
    assert_eq!(codigo, 0, "{stdout}");
}

// ===========================================================================
// Deposito per la rilettura indipendente del workbook **revisionato**
// ===========================================================================

/// Deposita il workbook di un'esecuzione **con** un'approvazione applicata, e
/// il manifesto per `script/releitura_export.py --modo percurso-cli-revisado`.
///
/// Serve un modo nuovo del lettore: `percurso-cli` attende zero approvazioni —
/// fogli `decisoes`, `extrato_revisado` e `avisos` senza righe di dati — e non
/// può validare un workbook che ne ha. Le attese dei modi `export` e
/// `percurso-cli` restano intatte.
///
/// Il manifesto lo scrive questo test, nella propria arena, rileggendo con
/// l'API pubblica l'archivio che la CLI ha pubblicato: un secondo file scritto
/// dal comando resterebbe fuori dalla protezione di `FonteProtegida`.
#[test]
fn tx80_deposita_para_a_releitura_revisada() {
    use rust_registrazioni_contabili::arquivo;
    use rust_registrazioni_contabili::export::{self, Celula, Proveniencia};

    const LOTE: &str = "maio-r1";
    const PLANO_DECLARADO: &str = "config/plano_contas_gerencial.yaml";

    let (dir, c) = com_duas_nfe("deposito_revisado", LOTE);
    let impressao = linha_da_conta(&c.arquivo, LOTE, "2.01.004")
        .impressao_conteudo
        .expect("a impressao");
    assert_eq!(
        executar(&args_declarar(
            &c,
            LOTE,
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido com a nota em papel",
            &impressao,
            &[]
        ))
        .0,
        0
    );
    assert_eq!(executar(&args_exportar(&c, LOTE, &[])).0, 0);

    let plano = PlanoContas::carregar(raiz().join(PLANO_DECLARADO)).expect("o plano");
    let destino = Destino::novo(dir.join("dados"), "arquivo.json".to_string());
    let leitura = arquivo_io::ler(&destino).expect("o arquivo publicado pela CLI");
    let sub = export::sub_arquivo_do_lote(&leitura.arquivo, LOTE).expect("o sub-arquivo");
    let divergencias = arquivo::verificar_apresentacao(&sub, &plano).expect("a apresentacao");
    let contextos = leitura.arquivo.contextos_do_lote(LOTE);
    let eventos = leitura.arquivo.eventos();
    assert_eq!(eventos.len(), 1, "uma declaracao, a do teste");
    let vista = revisao::aplicar_revisao(&contextos, &plano, &eventos).expect("a vista");
    let execucao = sub.execucoes.first().expect("a execucao");
    let proveniencia = Proveniencia {
        arquivo_caminho: destino.corrente().display().to_string(),
        arquivo_revisao: leitura.base.revisao,
        arquivo_sha256: leitura.base.sha256_conteudo.clone(),
        plano_arquivado: execucao.plano.clone(),
        plano_usado_versao: plano.versao().to_string(),
        plano_usado_caminho: PLANO_DECLARADO.to_string(),
        gerado_em: chrono::DateTime::parse_from_rfc3339(INSTANTE).expect("o instante"),
        divergencias,
        eventos_no_arquivo: eventos.len(),
    };
    let modelo = export::modelo(&vista, &proveniencia).expect("o modelo");

    // La forma del manifesto e' quella degli altri depositi: se divergesse, la
    // rilettura lo direbbe subito, perche' confronta il manifesto col file.
    let folhas: Vec<serde_json::Value> = modelo
        .folhas
        .iter()
        .map(|f| {
            let linhas: Vec<serde_json::Value> = f
                .linhas
                .iter()
                .map(|linha| {
                    let celulas: Vec<serde_json::Value> = linha
                        .iter()
                        .map(|cel| match cel {
                            Celula::Texto(t) => serde_json::json!({"tipo": "texto", "valor": t}),
                            Celula::NumeroDeComodo(d) => {
                                serde_json::json!({"tipo": "numero", "valor": d.to_string()})
                            }
                            Celula::Vazia => serde_json::json!({"tipo": "vazia"}),
                        })
                        .collect();
                    serde_json::Value::Array(celulas)
                })
                .collect();
            serde_json::json!({"nome": f.nome, "cabecalho": f.cabecalho, "linhas": linhas})
        })
        .collect();
    let manifesto = dir.join("dados/diagnostico.manifesto.json");
    std::fs::write(
        &manifesto,
        serde_json::to_string_pretty(&serde_json::json!({
            "formato": export::FORMATO,
            "versao_schema": export::VERSAO_SCHEMA,
            "folhas": folhas,
        }))
        .expect("serializar"),
    )
    .expect("escrever o manifesto");

    println!("REVISADO_WORKBOOK {}", c.saida);
    println!("REVISADO_MANIFESTO {}", manifesto.display());
}

// ===========================================================================
// Le due regressioni dell'audit del 2026-10-31
// ===========================================================================

/// Un'arena il cui percorso ha **spazi e un apostrofo**: è il caso che
/// spezzava il comando generato (`R3C5B-I01`).
fn arena_ostile(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("fantozzi_3c5b").join(format!(
        "percorso con spazi e l'apostrofo {nome}_{}",
        std::process::id()
    ));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar a arena");
    }
    std::fs::create_dir_all(dir.join("entrada")).expect("criar a entrada");
    std::fs::create_dir_all(dir.join("dados")).expect("criar os dados");
    dir
}

/// Una copia del piano canonico, **fuori** dal repository, con il nome di un
/// conto cambiato. Le sei fonti canoniche non si toccano.
fn plano_com_conta_renomeada(dir: &Path, nome_novo: &str) -> PathBuf {
    let original = std::fs::read_to_string(raiz().join(PLANO)).expect("o plano canonico");
    let linha = "    nome: \"Refrigerantes\"";
    assert!(original.contains(linha), "a linha do conto 2.01.004");
    let copia = dir.join("plano com nome novo.yaml");
    std::fs::write(
        &copia,
        original.replacen(linha, &format!("    nome: \"{nome_novo}\""), 1),
    )
    .expect("escrever a copia do plano");
    copia
}

/// Gli argomenti di un'archiviazione con un piano dichiarato.
fn args_arquivar_com_plano<'a>(c: &'a Caminhos, lote: &'a str, plano: &'a str) -> Vec<&'a str> {
    vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--inicializar",
        "--lote",
        lote,
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "importacao do teste",
        "--instante",
        INSTANTE,
        "--plano-contas",
        plano,
    ]
}

/// `R3C5B-I01`: il comando generato dalla consultazione **si esegue**, anche
/// con un percorso che contiene spazi e un apostrofo, e conserva il piano
/// selezionato.
///
/// Non basta che l'uscita contenga degli apici: il comando va eseguito. Si
/// sostituisce il solo lanciatore `cargo run --` con l'eseguibile già
/// compilato — per non ricompilare dentro un test — e si completano i tre
/// segnaposto, lasciando **invariati** tutti gli argomenti generati.
#[cfg(windows)]
#[test]
fn tx81_comando_gerado_executa_com_caminho_ostil() {
    let dir = arena_ostile("i01");
    std::fs::write(dir.join("entrada/a.xml"), campione()).expect("escrever a");
    let c = caminhos(&dir);
    // Un piano **proprio**, con lo stesso contenuto del canonico: qui si prova
    // il percorso con spazi, non il nome del conto.
    let plano_proprio = plano_com_conta_renomeada(&dir, "Refrigerantes");
    let plano_arg = plano_proprio.display().to_string();
    assert_eq!(
        executar(&args_arquivar_com_plano(&c, "maio-r1", &plano_arg)).0,
        0,
        "a importacao prepara o percurso"
    );

    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--conta",
        "2.01.004",
        "--plano-contas",
        &plano_arg,
    ]);
    assert_eq!(codigo, 0, "{stdout}");

    let gerado = stdout
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("cargo run -- declarar-revisao"))
        .expect("o comando gerado")
        .to_string();
    // Nel comando il percorso compare **come letterale**, con l'apostrofo
    // raddoppiato: è la prova del quoting, non un dettaglio da aggirare.
    let plano_literal = plano_proprio.display().to_string().replace('\'', "''");
    assert!(
        gerado.contains(&plano_literal),
        "o comando leva o plano **selecionado**, com o apostrofo duplicado:\n{gerado}"
    );

    // Il motivo porta un apostrofo, per esercitare il raddoppio.
    let comando = gerado
        .replacen(
            "cargo run --",
            &format!("& '{}'", env!("CARGO_BIN_EXE_rust_registrazioni_contabili")),
            1,
        )
        .replacen("<OPERACAO>", "aprovar", 1)
        .replacen("'<AUTOR>'", "'Luca Maggi'", 1)
        .replacen("'<MOTIVO>'", "'conferido: l''original em papel'", 1);

    let saida = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &comando])
        .output()
        .expect("executar o comando gerado em PowerShell");
    assert_eq!(
        saida.status.code(),
        Some(0),
        "o comando gerado tem de correr:\n{comando}\n{}\n{}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );

    // E deve avere agito sul bersaglio giusto, con quel piano.
    let plano = PlanoContas::carregar(&plano_proprio).expect("o plano proprio");
    let destino = Destino::novo(dir.join("dados"), "arquivo.json".to_string());
    let leitura = arquivo_io::ler(&destino).expect("o arquivo");
    let vista = revisao::aplicar_revisao(
        &leitura.arquivo.contextos_do_lote("maio-r1"),
        &plano,
        &leitura.arquivo.eventos(),
    )
    .expect("a vista");
    let linha = declaracao::consultar(&vista)
        .into_iter()
        .find(|l| l.conta_id == "2.01.004")
        .expect("a linha");
    assert_eq!(linha.estado, revisao::EstadoRevisao::AprovadoPorPessoa);
    assert_eq!(
        leitura.arquivo.eventos()[0].motivo,
        "conferido: l'original em papel",
        "o apostrofo duplicado chega como um so'"
    );
}

/// `R3C5B-I02`: consultazione ed export concordano sul nome **corrente** del
/// conto, e l'avviso della vista è leggibile nella consultazione.
///
/// Non è una fattura cambiata: è la normale sostituzione del nome di un conto
/// nel piano. L'impronta di contenuto resta uguale, quella di presentazione
/// cambia, e l'approvazione precedente resta applicata (D8.3).
#[test]
fn tx82_nome_do_plano_atual_e_aviso_visivel() {
    const NOME_NOVO: &str = "NOME NOVO NO PLANO ATUAL - SONDA";

    let (dir, c) = com_duas_nfe("i02", "maio-r1");
    let antes = linha_da_conta(&c.arquivo, "maio-r1", "2.01.004");
    let impressao = antes.impressao_conteudo.clone().expect("a impressao");
    let apresentacao_antes = antes
        .impressao_apresentacao
        .clone()
        .expect("a apresentacao");
    assert_eq!(antes.conta_nome_plano, "Refrigerantes");
    assert_eq!(
        executar(&args_declarar(
            &c,
            "maio-r1",
            CHAVE_A,
            "2.01.004",
            "aprovar",
            "Luca Maggi",
            "conferido com a nota em papel",
            &impressao,
            &["--vista", &apresentacao_antes]
        ))
        .0,
        0
    );

    let plano_novo = plano_com_conta_renomeada(&dir, NOME_NOVO);
    let plano_arg = plano_novo.display().to_string();
    let (codigo, stdout, _) = executar(&[
        "consultar-linhas",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--conta",
        "2.01.004",
        "--plano-contas",
        &plano_arg,
    ]);
    assert_eq!(codigo, 0, "{stdout}");
    assert!(
        stdout.contains(NOME_NOVO),
        "o nome do plano atual tem de aparecer:\n{stdout}"
    );
    assert!(
        stdout.contains("nome arquivado"),
        "e o historico etiquetado como tal:\n{stdout}"
    );
    assert!(
        stdout.contains("Refrigerantes"),
        "com o nome do momento da alocacao:\n{stdout}"
    );
    assert!(
        stdout.contains("APRESENTACAO_MUDADA"),
        "o aviso da vista tem de ser legivel na consulta:\n{stdout}"
    );

    // Le due impronte: il contenuto non cambia, la presentazione sì.
    let plano = PlanoContas::carregar(&plano_novo).expect("o plano novo");
    let caminho = Path::new(&c.arquivo);
    let destino = Destino::novo(
        caminho.parent().expect("o diretorio").to_path_buf(),
        "arquivo.json".to_string(),
    );
    let leitura = arquivo_io::ler(&destino).expect("o arquivo");
    let vista = revisao::aplicar_revisao(
        &leitura.arquivo.contextos_do_lote("maio-r1"),
        &plano,
        &leitura.arquivo.eventos(),
    )
    .expect("a vista");
    let depois = declaracao::consultar(&vista)
        .into_iter()
        .find(|l| l.chave_origem == CHAVE_A && l.conta_id == "2.01.004")
        .expect("a linha");
    assert_eq!(depois.conta_nome_plano, NOME_NOVO, "o nome autoritativo");
    assert_eq!(depois.conta_nome_historico, "Refrigerantes", "o arquivado");
    assert_eq!(
        depois.impressao_conteudo.as_deref(),
        Some(impressao.as_str()),
        "a impressao de conteudo nao muda com o nome do conto"
    );
    assert_ne!(
        depois.impressao_apresentacao.as_deref(),
        Some(apresentacao_antes.as_str()),
        "a de apresentacao muda, porque o plano mudou"
    );
    // D8.3: le decisioni restano.
    assert_eq!(depois.estado, revisao::EstadoRevisao::AprovadoPorPessoa);

    // E l'export dello stesso lotto con lo stesso piano non fallisce.
    let (codigo, stdout, _) = executar(&[
        "exportar-diagnostico",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "maio-r1",
        "--saida",
        &c.saida,
        "--operador",
        "Luca Maggi",
        "--motivo",
        "export com o plano novo",
        "--instante",
        INSTANTE,
        "--plano-contas",
        &plano_arg,
    ]);
    assert_eq!(codigo, 0, "{stdout}");
}

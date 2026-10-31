//! Il percorso della fase 3C.5.a dalla CLI: da una **cartella** di NF-e
//! all'archivio autorevole, e da un'esecuzione all'Excel diagnostico.
//!
//! Perché test di integrazione: questi contratti stanno negli esiti del
//! processo — codici d'uscita, file pubblicati o non pubblicati — e nessun
//! test unitario li osserva. I contratti dei moduli composti sono già provati
//! altrove e qui non vengono ridimostrati.
//!
//! # Il campione, e le NF-e derivate
//!
//! Il repository versiona **una** NF-e reale,
//! `samples/nfe_exemplo/nfe_oesa_12566_20260505.xml`, che è l'oracolo del
//! parser e non va modificata. Le altre nascono qui, per sostituzione
//! testuale della chiave e del numero: la derivazione è dichiarata in
//! [`derivar`] e scritta nell'arena del test, mai nel repository.

use std::path::{Path, PathBuf};
use std::process::Command;

/// La NF-e reale: 3 articoli, totale 35.00, stato tecnico `OK` **misurato**.
const CHAVE_CAMPIONE: &str = "29260512345678000195550010001256612345678901";
const NUMERO_CAMPIONE: &str = "12566";

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn campione() -> String {
    std::fs::read_to_string(raiz().join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml"))
        .expect("o campione versionado")
}

/// Un'arena per test, ricreata a ogni esecuzione.
fn arena(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("fantozzi_3c5a")
        .join(format!("{nome}_{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar a arena");
    }
    std::fs::create_dir_all(dir.join("entrada")).expect("criar a entrada");
    std::fs::create_dir_all(dir.join("dados")).expect("criar os dados");
    dir
}

/// Una NF-e derivata dal campione: **solo** chiave e numero cambiati.
///
/// La chiave contiene il numero del documento, quindi si sostituisce prima
/// l'intera chiave di 44 caratteri e poi `<nNF>` con i suoi tag: invertire
/// l'ordine corromperebbe la chiave.
fn derivar(chave: &str, numero: &str) -> String {
    assert_eq!(chave.len(), CHAVE_CAMPIONE.len(), "a chave tem 44 digitos");
    campione().replace(CHAVE_CAMPIONE, chave).replace(
        &format!("<nNF>{NUMERO_CAMPIONE}</nNF>"),
        &format!("<nNF>{numero}</nNF>"),
    )
}

fn escrever(caminho: &Path, conteudo: &str) {
    std::fs::write(caminho, conteudo).expect("escrever o XML da arena");
}

/// Tre NF-e di chiavi diverse nella cartella d'ingresso.
fn tres_nfe(dir: &Path) {
    let entrada = dir.join("entrada");
    escrever(&entrada.join("a_12566.xml"), &campione());
    escrever(
        &entrada.join("b_12567.xml"),
        &derivar("29260512345678000195550010001256712345678902", "12567"),
    );
    escrever(
        &entrada.join("c_12568.xml"),
        &derivar("29260512345678000195550010001256812345678903", "12568"),
    );
}

/// Esegue la CLI e restituisce (codice, stdout, stderr).
///
/// Il codice è il contratto `CLI-01`: non si guarda soltanto se è fallita.
fn executar(args: &[&str]) -> (i32, String, String) {
    let saida = Command::new(env!("CARGO_BIN_EXE_rust_registrazioni_contabili"))
        .current_dir(raiz())
        .args(args)
        .output()
        .expect("executar a CLI");
    (
        saida.status.code().expect("a CLI terminou com um codigo"),
        String::from_utf8_lossy(&saida.stdout).into_owned(),
        String::from_utf8_lossy(&saida.stderr).into_owned(),
    )
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

/// Gli argomenti di un'archiviazione, con l'istante **dichiarato**: un test
/// non deve dipendere dall'orologio della macchina.
fn args_arquivar<'a>(c: &'a Caminhos, lote: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "arquivar-nfe",
        "--pasta",
        &c.entrada,
        "--arquivo",
        &c.arquivo,
        "--lote",
        lote,
        "--lote-data",
        "2026-05-31",
        "--lote-descricao",
        "pasta de maio",
        "--operador",
        "Luca Maggi",
        "--motivo",
        "teste do percurso",
        "--instante",
        "2026-10-31T09:00:00-03:00",
    ];
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
        "teste do percurso",
        "--instante",
        "2026-10-31T09:00:00-03:00",
    ];
    args.extend_from_slice(extra);
    args
}

fn sha256(caminho: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(caminho).expect("ler o arquivo");
    Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ===========================================================================
// Il percorso completo
// ===========================================================================

/// `CLI-01`, `CLI-02`, `CLI-09`: primo avvio su una cartella di tre NF-e.
#[test]
fn tx50_primeiro_arranque() {
    let dir = arena("tx50");
    tres_nfe(&dir);
    let c = caminhos(&dir);

    let (codigo, stdout, _) = executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"]));
    assert_eq!(codigo, 0, "primeiro arranque limpo:\n{stdout}");
    assert!(stdout.contains("Revisao            : 1"), "{stdout}");
    assert!(
        stdout.contains("3 novos, 0 ja' arquivados"),
        "os tres documentos sao novos:\n{stdout}"
    );
    assert!(stdout.contains("STATUS: ARQUIVADO"), "{stdout}");
    // `CLI-09`: l'istante dichiarato viene etichettato come tale.
    assert!(stdout.contains("Instante   : DECLARADO"), "{stdout}");

    // `CLI-02`: l'importazione non crea nessuna decisione umana. Lo si guarda
    // nell'archivio, non solo nel riassunto.
    let texto = std::fs::read_to_string(&c.arquivo).expect("o arquivo publicado");
    assert!(
        !texto.contains("APROVAR") && !texto.contains("APROVADO"),
        "nenhuma aprovacao pode nascer de uma importacao"
    );

    let (codigo, stdout, _) = executar(&args_exportar(&c, "fonte-maio-r1", &[]));
    assert_eq!(codigo, 0, "export da execucao:\n{stdout}");
    assert!(
        stdout.contains("Conteudo   : 3 documento(s)"),
        "o resultado cobre os tres documentos:\n{stdout}"
    );
    assert!(Path::new(&c.saida).exists(), "o XLSX foi publicado");
    assert!(
        std::fs::metadata(&c.saida).unwrap().len() > 5_000,
        "um XLSX de verdade, nao um residuo vazio"
    );
}

/// `CLI-04`, `CLI-08`: il rilancio sulla cartella non gonfia il risultato.
///
/// È la prova del flusso di Luca: rileggere tutto, e contare ogni fattura una
/// volta sola nel risultato corrente.
#[test]
fn tx51_relancamento_da_pasta() {
    let dir = arena("tx51");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );

    let (codigo, stdout, _) = executar(&args_arquivar(&c, "fonte-maio-r2", &[]));
    assert_eq!(codigo, 0, "o relancamento e' normal:\n{stdout}");
    assert!(stdout.contains("Revisao            : 2"), "{stdout}");
    assert!(
        stdout.contains("0 novos, 3 ja' arquivados"),
        "reuso entre execucoes, nao duplicacao:\n{stdout}"
    );
    assert_eq!(
        stdout.matches("JA_ARQUIVADO_IDENTICO").count(),
        3,
        "cada documento e' nomeado com o seu esito:\n{stdout}"
    );
    assert!(
        stdout.contains("arquivo.json.ger1 (conservada)"),
        "a geracao anterior resta:\n{stdout}"
    );

    // Il risultato della nuova esecuzione conta **tre** documenti, non sei.
    let (codigo, stdout, _) = executar(&args_exportar(&c, "fonte-maio-r2", &[]));
    assert_eq!(codigo, 0, "{stdout}");
    assert!(
        stdout.contains("Conteudo   : 3 documento(s)"),
        "o resultado nao soma as execucoes:\n{stdout}"
    );
}

/// `CLI-07`: un'esecuzione archiviata è immutabile.
#[test]
fn tx52_lote_ocupado() {
    let dir = arena("tx52");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    let antes = sha256(&c.arquivo);

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r1", &[]));
    assert_eq!(codigo, 6, "recusa de IO antes do commit:\n{stderr}");
    assert!(
        stderr.contains("imutavel"),
        "o motivo e' nomeado:\n{stderr}"
    );
    assert!(
        stderr.contains("NAO publicado"),
        "o fato atestavel e' dito:\n{stderr}"
    );
    assert_eq!(antes, sha256(&c.arquivo), "o arquivo nao mudou um byte");
}

/// `CLI-02` sul caso **non** pulito: senza approvazioni lo stato della vista
/// segue l'esito tecnico, e `TECNICO_OK` non è universale.
#[test]
fn tx53_documento_a_revisar() {
    let dir = arena("tx53");
    // Una descrizione che nessuna regola riconosce: la 3B la manda a REVISAR.
    let xml = derivar("29260512345678000195550010001256912345678904", "12569")
        .replace("COCA-COLA KS 290ML", "ARTIGO SEM REGRA NENHUMA XYZ");
    escrever(&dir.join("entrada/revisar.xml"), &xml);
    let c = caminhos(&dir);

    let (codigo, stdout, _) = executar(&args_arquivar(&c, "fonte-r", &["--inicializar"]));
    assert_eq!(
        codigo, 0,
        "um documento a revisar nao e' um erro de reconciliacao:\n{stdout}"
    );

    let texto = std::fs::read_to_string(&c.arquivo).expect("o arquivo");
    assert!(
        texto.contains("REVISAR"),
        "o esito tecnico da 3B fica conservado"
    );
    assert!(
        !texto.contains("APROVADO"),
        "e nenhuma aprovacao humana nasce da importacao"
    );
}

/// `CLI-06`, primo ramo: nessuna creazione implicita dell'archivio.
#[test]
fn tx54_sem_inicializar_em_arquivo_ausente() {
    let dir = arena("tx54");
    tres_nfe(&dir);
    let c = caminhos(&dir);

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r1", &[]));
    assert_eq!(codigo, 6, "{stderr}");
    assert!(stderr.contains("--inicializar"), "{stderr}");
    assert!(
        !Path::new(&c.arquivo).exists(),
        "nada foi criado sem o flag"
    );
}

/// `CLI-06`, secondo ramo: il flag non serve due volte.
#[test]
fn tx55_inicializar_em_arquivo_existente() {
    let dir = arena("tx55");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    let antes = sha256(&c.arquivo);

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r2", &["--inicializar"]));
    assert_eq!(codigo, 6, "{stderr}");
    assert!(stderr.contains("ja' existe"), "{stderr}");
    assert_eq!(antes, sha256(&c.arquivo), "o arquivo nao mudou");
}

/// `CLI-03`: due file della stessa esecuzione con la stessa chiave.
#[test]
fn tx56_duplicada_na_execucao() {
    let dir = arena("tx56");
    escrever(&dir.join("entrada/primeiro.xml"), &campione());
    escrever(&dir.join("entrada/segundo.xml"), &campione());
    let c = caminhos(&dir);

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"]));
    assert_eq!(codigo, 3, "{stderr}");
    assert!(
        stderr.contains("primeiro.xml") && stderr.contains("segundo.xml"),
        "os dois arquivos sao nomeados:\n{stderr}"
    );
    assert!(!Path::new(&c.arquivo).exists(), "nada foi publicado");
}

/// `CLI-05` e D33: stessa chiave, contenuto allocato diverso.
///
/// Il messaggio **non** attribuisce la causa: la differenza può venire dalla
/// fonte o dalla configurazione, perché le regole entrano nell'impronta.
#[test]
fn tx57_conteudo_alocado_diverso() {
    let dir = arena("tx57");
    escrever(&dir.join("entrada/nfe.xml"), &campione());
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    let antes = sha256(&c.arquivo);

    // Stessa chiave, un articolo descritto in modo diverso.
    escrever(
        &dir.join("entrada/nfe.xml"),
        &campione().replace("COCA-COLA KS 290ML", "COCA-COLA KS 290ML (CORRIGIDO)"),
    );

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r2", &[]));
    assert_eq!(codigo, 4, "{stderr}");
    assert!(
        stderr.contains("conteudo alocado diverso"),
        "o rotulo fala do que foi comparado:\n{stderr}"
    );
    assert!(
        stderr.contains("fonte OU da configuracao"),
        "nenhuma causa e' atribuida sem prova:\n{stderr}"
    );
    assert_eq!(antes, sha256(&c.arquivo), "o arquivo nao mudou um byte");
}

/// D30, modo conservativo: un documento non `OK` viene archiviato, con riserva.
#[test]
fn tx58_nao_ok_arquivado() {
    let dir = arena("tx58");
    // Totale della NF-e che non quadra con la somma degli articoli.
    escrever(
        &dir.join("entrada/desquadrada.xml"),
        &campione().replace("<vNF>35.00</vNF>", "<vNF>99.00</vNF>"),
    );
    let c = caminhos(&dir);

    let (codigo, stdout, stderr) =
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"]));
    assert_eq!(codigo, 5, "arquivado com reserva:\n{stdout}\n{stderr}");
    assert!(stdout.contains("STATUS: ARQUIVADO_COM_RESERVA"), "{stdout}");
    assert!(
        stdout.contains("Arquivo corrente PUBLICADO"),
        "o fato atestavel: foi publicado:\n{stdout}"
    );
    assert!(
        stderr.contains("estado tecnico Erro"),
        "o documento e' nomeado com o seu estado:\n{stderr}"
    );
    assert!(Path::new(&c.arquivo).exists());
}

/// D30, modo rifiuto: `--recusar-nao-ok` rifiuta **prima** di pubblicare.
#[test]
fn tx59_nao_ok_recusado() {
    let dir = arena("tx59");
    escrever(
        &dir.join("entrada/desquadrada.xml"),
        &campione().replace("<vNF>35.00</vNF>", "<vNF>99.00</vNF>"),
    );
    let c = caminhos(&dir);

    let (codigo, _, stderr) = executar(&args_arquivar(
        &c,
        "fonte-maio-r1",
        &["--inicializar", "--recusar-nao-ok"],
    ));
    assert_eq!(codigo, 7, "recusa distinta do codigo 5:\n{stderr}");
    assert!(stderr.contains("NAO publicado"), "{stderr}");
    assert!(
        !Path::new(&c.arquivo).exists(),
        "nada foi publicado com o flag"
    );
}

/// `CLI-08`: un risultato copre una esecuzione, e un lotto inesistente non
/// produce niente.
#[test]
fn tx60_lote_inexistente() {
    let dir = arena("tx60");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );

    let (codigo, _, stderr) = executar(&args_exportar(&c, "fonte-que-nao-existe", &[]));
    assert_eq!(codigo, 1, "{stderr}");
    assert!(
        stderr.contains("fonte-maio-r1"),
        "o erro diz quais execucoes existem:\n{stderr}"
    );
    assert!(!Path::new(&c.saida).exists(), "nenhum XLSX publicado");
}

/// `EXP-42` dalla CLI: la destinazione non può essere l'archivio.
#[test]
fn tx61_saida_sobre_o_arquivo() {
    let dir = arena("tx61");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    let antes = sha256(&c.arquivo);

    let args = vec![
        "exportar-diagnostico",
        "--arquivo",
        &c.arquivo,
        "--lote",
        "fonte-maio-r1",
        "--saida",
        &c.arquivo,
        "--operador",
        "Luca Maggi",
        "--motivo",
        "tentativa sobre a fonte",
    ];
    let (codigo, _, stderr) = executar(&args);
    assert_eq!(codigo, 4, "{stderr}");
    assert!(
        stderr.contains("intacto"),
        "o fato atestavel e' dito:\n{stderr}"
    );
    assert_eq!(antes, sha256(&c.arquivo), "o arquivo autoritativo intacto");
}

/// `EXP-23` dalla CLI: destinazione già esistente con `recusar`.
#[test]
fn tx62_destino_existente() {
    let dir = arena("tx62");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    assert_eq!(executar(&args_exportar(&c, "fonte-maio-r1", &[])).0, 0);
    let antes = sha256(&c.saida);

    let (codigo, _, stderr) = executar(&args_exportar(&c, "fonte-maio-r1", &[]));
    assert_eq!(codigo, 3, "recusa com codigo proprio:\n{stderr}");
    assert_eq!(antes, sha256(&c.saida), "o arquivo que existia resta igual");

    // Con `substituir` passa, e il precedente viene conservato.
    let (codigo, stdout, _) = executar(&args_exportar(
        &c,
        "fonte-maio-r1",
        &["--se-existir", "substituir"],
    ));
    assert_eq!(codigo, 0, "{stdout}");
    assert!(stdout.contains("(conservado)"), "{stdout}");
}

// ===========================================================================
// Deposito per la rilettura indipendente (R3C5A-02)
// ===========================================================================

/// Deposita il workbook della **CLI** e il manifesto per
/// `script/releitura_export.py --modo percurso-cli`.
///
/// Il manifesto non è un output del comando: sarebbe una seconda scrittura
/// fuori dalla protezione di `FonteProtegida`. Lo costruisce questo test,
/// nella propria arena, rileggendo con l'API pubblica l'archivio che la CLI
/// ha pubblicato e ricostruendo il modello con la **stessa** provenienza che
/// la CLI ha usato: se una delle due divergesse, la rilettura lo direbbe
/// subito, perché confronta il manifesto con il file.
#[test]
fn tx63_deposita_para_a_releitura_do_percurso_cli() {
    use rust_registrazioni_contabili::arquivo::{self, IdentidadePlano};
    use rust_registrazioni_contabili::arquivo_io::{self, Destino};
    use rust_registrazioni_contabili::export::{self, Celula, Proveniencia};
    use rust_registrazioni_contabili::regras::PlanoContas;
    use rust_registrazioni_contabili::revisao;

    const INSTANTE: &str = "2026-10-31T09:00:00-03:00";
    const LOTE: &str = "fonte-maio-r1";
    // Il percorso del piano **come la CLI lo ha dichiarato**: relativo, perché
    // il processo gira con la radice del repository come directory corrente.
    const PLANO_DECLARADO: &str = "config/plano_contas_gerencial.yaml";

    let dir = arena("deposito");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(executar(&args_arquivar(&c, LOTE, &["--inicializar"])).0, 0);
    assert_eq!(executar(&args_exportar(&c, LOTE, &[])).0, 0);

    // Il modello, ricostruito dall'archivio che la CLI ha scritto.
    let plano = PlanoContas::carregar(raiz().join(PLANO_DECLARADO)).expect("o plano");
    let destino = Destino::novo(dir.join("dados"), "arquivo.json".to_string());
    let leitura = arquivo_io::ler(&destino).expect("o arquivo publicado pela CLI");
    let sub = export::sub_arquivo_do_lote(&leitura.arquivo, LOTE).expect("o sub-arquivo");
    let divergencias = arquivo::verificar_apresentacao(&sub, &plano).expect("a apresentacao");
    let contextos = leitura.arquivo.contextos_do_lote(LOTE);
    let eventos = leitura.arquivo.eventos();
    assert!(
        eventos.is_empty(),
        "CLI-02: a importacao nao cria nenhuma decisao humana"
    );
    let vista = revisao::aplicar_revisao(&contextos, &plano, &eventos).expect("a vista");
    let execucao = sub.execucoes.first().expect("a execucao");
    let plano_arquivado: IdentidadePlano = execucao.plano.clone();
    assert!(
        plano_arquivado.sha256_arquivo.is_some(),
        "D28: a CLI grava a impressao real do plano"
    );
    let proveniencia = Proveniencia {
        arquivo_caminho: destino.corrente().display().to_string(),
        arquivo_revisao: leitura.base.revisao,
        arquivo_sha256: leitura.base.sha256_conteudo.clone(),
        plano_arquivado,
        plano_usado_versao: plano.versao().to_string(),
        plano_usado_caminho: PLANO_DECLARADO.to_string(),
        gerado_em: chrono::DateTime::parse_from_rfc3339(INSTANTE).expect("o instante"),
        divergencias,
        eventos_no_arquivo: eventos.len(),
    };
    let modelo = export::modelo(&vista, &proveniencia).expect("o modelo");

    // La forma del manifesto è quella di `tests/export_releitura.rs`. Una
    // deriva fra le due non resta nascosta: il lettore confronta il manifesto
    // con il workbook, e un manifesto di forma sbagliata fallisce subito.
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
        .expect("serializar o manifesto"),
    )
    .expect("escrever o manifesto");

    println!("PERCURSO_CLI_WORKBOOK {}", c.saida);
    println!("PERCURSO_CLI_MANIFESTO {}", manifesto.display());
    println!("PERCURSO_CLI_SHA256 {}", sha256(&c.saida));
}

/// `R3C5A-I01`: un errore **dopo** la creazione del temporaneo deve nominare
/// subito il temporaneo che resta.
///
/// Come si riproduce senza bloccare un file: il protocollo scrive il
/// temporaneo al passo 4 e copia il precedente al passo 5, e il passo 5
/// rifiuta se la copia del precedente esiste già (`AnteriorJaPresente`).
/// Precreando quel file si ottiene un errore dopo che il temporaneo è stato
/// scritto, in modo deterministico e su ogni sistema.
///
/// Non è la prova con un handle esclusivo di un altro processo: quella l'ha
/// misurata l'audit, e resta la sua. Questa fissa il **contratto** della
/// diagnostica, che è ciò che il riscontro chiedeva.
#[test]
fn tx64_residuo_nomeado_depois_de_erro_de_io() {
    let dir = arena("tx64");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    assert_eq!(executar(&args_exportar(&c, "fonte-maio-r1", &[])).0, 0);
    let antes = sha256(&c.saida);

    // Il passo 5 trovera' la copia del precedente gia' presente.
    let anterior = dir.join("dados/diagnostico.xlsx.anterior.ocupada");
    std::fs::write(&anterior, b"ocupado por outro tentativo").expect("preparar o anterior");
    let temporario = dir.join("dados/diagnostico.xlsx.novo.ocupada");

    let (codigo, _, stderr) = executar(&args_exportar(
        &c,
        "fonte-maio-r1",
        &["--se-existir", "substituir", "--etiqueta", "ocupada"],
    ));

    assert_eq!(codigo, 6, "recusa de IO antes do commit:\n{stderr}");
    assert!(
        stderr.contains("XLSX final NAO publicado"),
        "o fato atestavel e' dito:\n{stderr}"
    );
    // Il punto del riscontro: il temporaneo resta, e viene **nominato**.
    assert!(
        temporario.exists(),
        "o protocolo conserva o temporario: {}",
        temporario.display()
    );
    assert!(
        stderr.contains("diagnostico.xlsx.novo.ocupada"),
        "o temporario que resta tem de ser nomeado:\n{stderr}"
    );
    assert!(
        stderr.contains("TEMPORARIO") && stderr.contains("NAO apagados automaticamente"),
        "e tem de ser dito que nada e' apagado sozinho:\n{stderr}"
    );
    assert_eq!(antes, sha256(&c.saida), "o XLSX anterior resta igual");
}

/// `R3C5A-I01`, ramo dell'archivio: anche la pubblicazione fallita nomina il
/// temporaneo che resta.
///
/// Riproduzione deterministica, con la stessa forma di `tx64`: `publicar`
/// scrive il temporaneo al passo 5 e al passo 6 rifiuta se la generazione
/// esiste già, quindi precreare `arquivo.json.ger1` produce un errore dopo la
/// creazione del temporaneo. Copre `residuos_do_arquivo`, che passa da
/// `arquivo_io::candidatos`.
#[test]
fn tx65_residuo_do_arquivo_nomeado_depois_de_erro_de_io() {
    let dir = arena("tx65");
    tres_nfe(&dir);
    let c = caminhos(&dir);
    assert_eq!(
        executar(&args_arquivar(&c, "fonte-maio-r1", &["--inicializar"])).0,
        0
    );
    let antes = sha256(&c.arquivo);

    // Il passo 6 trovera' la generazione gia' presente.
    std::fs::write(
        dir.join("dados/arquivo.json.ger1"),
        b"geracao ocupada por outro tentativo",
    )
    .expect("preparar a geracao");

    let (codigo, _, stderr) = executar(&args_arquivar(&c, "fonte-maio-r2", &[]));

    assert_eq!(codigo, 6, "recusa de IO antes do commit:\n{stderr}");
    assert!(
        stderr.contains("Arquivo corrente NAO publicado"),
        "o fato atestavel e' dito:\n{stderr}"
    );
    assert!(
        stderr.contains("arquivo.json.novo.2"),
        "o temporario que resta tem de ser nomeado:\n{stderr}"
    );
    assert!(
        stderr.contains("TEMPORARIO") && stderr.contains("NAO apagados automaticamente"),
        "e tem de ser dito que nada e' apagado sozinho:\n{stderr}"
    );
    assert_eq!(
        antes,
        sha256(&c.arquivo),
        "o arquivo corrente nao mudou um byte"
    );
}

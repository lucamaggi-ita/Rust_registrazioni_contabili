//! Prova della persistenza **fra processi** (TP-19) e controllo fisico
//! indipendente del file pubblicato (TP-20), fase 3C.3.b.1.
//!
//! Perche' un test di integrazione e non uno unitario: l'affermazione di D7(a)
//! e' che l'archivio sia autorevole **fra esecuzioni**. Un test nello stesso
//! processo non la dimostra, perche' potrebbe passare per uno stato in memoria.
//! Qui il processo che scrive e quello che legge sono due.
//!
//! Il figlio e' questo stesso binario di test, rieseguito con
//! [`std::env::current_exe`] e `--exact`, con il ruolo e la directory passati
//! per variabile d'ambiente. Non si installa niente e non si usa nessun
//! archivio reale: i dati sono dichiarati qui sotto.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{FixedOffset, NaiveDate, TimeZone};
use rust_decimal::Decimal;

use rust_registrazioni_contabili::alocacao::{
    AlocacaoNfe, Componente, EsitoComponenti, EsitoCopertura, EsitoRate, EsitoTotal, Reconciliacao,
    RuoloComponente, StatoDocumento,
};
use rust_registrazioni_contabili::arquivo::{Arquivo, IdentidadePlano};
use rust_registrazioni_contabili::arquivo_io::{
    self, Destino, ErroIo, EstadoObservado, MetadadosLucchetto,
};
use rust_registrazioni_contabili::impressao;
use rust_registrazioni_contabili::modelos::{
    Competencia, Dado, Lancamento, OrigemLancamento, StatusDado, StatusRevisao, TipoConta,
};
use rust_registrazioni_contabili::prelivro::{ContextoDocumento, IdentidadeLote};
use rust_registrazioni_contabili::regras::PlanoContas;
use rust_registrazioni_contabili::revisao::{ChaveRevisao, EventoRevisao, OperacaoRevisao};

const VAR_DIRETORIO: &str = "FANTOZZI_3C3B1_DIRETORIO";
const VAR_PAPEL: &str = "FANTOZZI_3C3B1_PAPEL";
const NOME: &str = "arquivo.json";
const CHAVE: &str = "29260512345678000195550010001256612345678901";

// ---------------------------------------------------------------------------
// Dati dichiarati
// ---------------------------------------------------------------------------

fn plano() -> PlanoContas {
    PlanoContas::carregar(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("config/plano_contas_gerencial.yaml"),
    )
    .unwrap()
}

fn dec(bruto: &str) -> Decimal {
    Decimal::from_str_exact(bruto).unwrap()
}

fn data(dia: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, dia).unwrap()
}

fn documento() -> ContextoDocumento {
    let lancamento = Lancamento {
        data_documento: data(5),
        competencia: Competencia::new(2026, 5).unwrap(),
        cnpj_empresa: "55746722000181".to_string(),
        nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
        fornecedor_ou_favorecido: "DISTRIBUIDORA OESA LTDA".to_string(),
        cnpj_cpf_favorecido: "00012345678901".to_string(),
        documento: "12566".to_string(),
        tipo_documento: "NFE".to_string(),
        conta_id: "2.01.004".to_string(),
        conta_nome: "Refrigerantes".to_string(),
        grupo_conta: "Custos de Mercadorias e Insumos".to_string(),
        descricao_itens: "SUCO A\u{00E7}ucar \"MARCA\" (10.00)".to_string(),
        valor: dec("10.00"),
        duplicatas: Vec::new(),
        origem: OrigemLancamento::XmlNfe,
        arquivo_origem: "samples/nfe.xml".to_string(),
        chave_origem: CHAVE.to_string(),
        hash_registro: "payload-entre-processos".to_string(),
        status_revisao: StatusRevisao::Ok,
        alertas: String::new(),
        observacoes: String::new(),
        centro_custo: String::new(),
        tipo_lancamento: TipoConta::Custo,
        dre_linha_id: String::new(),
        dre_linha_nome: String::new(),
        data_pagamento: Dado::ausente(
            "nfe:cobr/dup/dVenc",
            StatusDado::NaoDisponivel,
            "a NF-e declara vencimentos, nao pagamentos",
        )
        .unwrap(),
    };
    ContextoDocumento {
        chave_origem: CHAVE.to_string(),
        numero: "12566".to_string(),
        serie: "1".to_string(),
        data_documento: data(5),
        competencia: Competencia::new(2026, 5).unwrap(),
        cnpj_empresa: "55746722000181".to_string(),
        nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
        arquivo_origem: "samples/nfe.xml".to_string(),
        alocacao: AlocacaoNfe {
            lancamentos: vec![lancamento],
            reconciliacao: Reconciliacao {
                valor_total_nfe: dec("10.00"),
                soma_lancamentos: dec("10.00"),
                esito_total: EsitoTotal::Ok,
                soma_rate_documento: Dado::ausente(
                    "nfe:cobr/dup/vDup",
                    StatusDado::NaoAplicavel,
                    "NF-e sem duplicatas",
                )
                .unwrap(),
                soma_rate_allocate: Dado::ausente(
                    "nfe:cobr/dup/vDup",
                    StatusDado::NaoAplicavel,
                    "NF-e sem duplicatas",
                )
                .unwrap(),
                esito_rate: EsitoRate::NaoAplicavel,
                articoli_allocati: 1,
                articoli_attesi: Dado::ausente(
                    "nfe:det",
                    StatusDado::NaoDisponivel,
                    "a fase 2 nao expoe o numero de <det>",
                )
                .unwrap(),
                articoli_scartati: Vec::new(),
                esito_copertura: EsitoCopertura::Ok,
                componenti: vec![Componente {
                    nome: "vProd".to_string(),
                    valor: Dado::presente(dec("10.00"), "nfe:total/ICMSTot/vProd").unwrap(),
                    ruolo: RuoloComponente::Base,
                    usada_na_verificacao: true,
                }],
                esito_componenti: EsitoComponenti::Ok,
                campi_rinviati: Vec::new(),
                avisos_documento: Vec::new(),
                stato_documento: StatoDocumento::Ok,
            },
        },
        avisos_leitura: Vec::new(),
        lote: IdentidadeLote {
            id: "lote-entre-processos".to_string(),
            gerado_em: data(31),
            descricao: "prova entre processos".to_string(),
        },
    }
}

fn evento(hora: u32, operacao: OperacaoRevisao) -> EventoRevisao {
    let contexto = documento();
    EventoRevisao {
        alvo: ChaveRevisao {
            chave_origem: CHAVE.to_string(),
            impressao_conteudo: Dado::presente(
                impressao::impressao_conteudo(&contexto),
                "revisao:conteudo",
            )
            .unwrap(),
            origem: OrigemLancamento::XmlNfe,
            conta_id: "2.01.004".to_string(),
            ordinal: 0,
        },
        operacao,
        autor: "Jos\u{00E9}".to_string(),
        instante: FixedOffset::east_opt(-3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 22, hora, 0, 0)
            .unwrap(),
        motivo: "prova entre processos".to_string(),
        nota: match operacao {
            OperacaoRevisao::Anotar => Some("nota".to_string()),
            _ => None,
        },
        sequencia: None,
        vista: Dado::presente("v".repeat(64), "revisao:vista").unwrap(),
    }
}

/// L'archivio che il **figlio** pubblica. Lo costruiscono entrambi i processi,
/// in modo indipendente: se coincide, non e' passato per la memoria condivisa.
fn arquivo_inicial() -> Arquivo {
    let mut arquivo = Arquivo::novo();
    arquivo
        .acrescentar_lote(
            &[documento()],
            &plano(),
            IdentidadePlano {
                versao: "1.0".to_string(),
                sha256_arquivo: None,
                caminho_declarado: None,
            },
        )
        .unwrap();
    arquivo.acrescentar_eventos(&[evento(12, OperacaoRevisao::Aprovar)]);
    arquivo
}

fn lucchetto(operador: &str, motivo: &str) -> MetadadosLucchetto {
    MetadadosLucchetto {
        operador: operador.to_string(),
        instante: FixedOffset::east_opt(-3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 22, 9, 30, 0)
            .unwrap(),
        motivo: motivo.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Il processo figlio
// ---------------------------------------------------------------------------

/// Esegue un ruolo quando il padre lo convoca per variabile d'ambiente.
///
/// Eseguito dalla suite normale **senza** quelle variabili non fa niente e non
/// afferma niente: e' un punto d'ingresso, non una verifica. Cio' che va
/// verificato lo verifica [`tp19_persistencia_entre_processos`].
#[test]
fn papel_de_processo_filho() {
    let (Ok(diretorio), Ok(papel)) = (env::var(VAR_DIRETORIO), env::var(VAR_PAPEL)) else {
        return;
    };
    let destino = Destino::novo(PathBuf::from(diretorio), NOME);
    match papel.as_str() {
        // 1) Pubblica l'archivio iniziale in un processo che non ha mai visto
        //    la memoria del padre.
        "publicar" => {
            let publicacao = arquivo_io::publicar(
                &destino,
                &arquivo_inicial(),
                None,
                &lucchetto("filho", "publicacao inicial"),
            )
            .expect("o filho deve conseguir publicar");
            assert_eq!(publicacao.revisao_publicada, 1);
            assert!(publicacao.geracao_anterior.is_none());
            assert!(!publicacao.lucchetto_residuo);
        }
        // 2) Rilegge quello che il **padre** ha pubblicato dopo di lui.
        "verificar" => {
            let leitura = arquivo_io::ler(&destino).expect("o filho deve conseguir ler");
            assert_eq!(leitura.base.revisao, 2, "o filho ve' a revisao do pai");
            assert_eq!(leitura.arquivo.eventos.len(), 2);
            assert_eq!(leitura.arquivo.snapshots.len(), 1);
            // E ricostruisce lo stesso contenuto che il padre ha scritto.
            let mut esperado = arquivo_inicial();
            esperado.acrescentar_eventos(&[evento(15, OperacaoRevisao::Anotar)]);
            assert_eq!(leitura.arquivo, esperado);
        }
        // 3) Trova il lucchetto di un altro processo e si ferma.
        "ocupado" => {
            match arquivo_io::publicar(
                &destino,
                &arquivo_inicial(),
                None,
                &lucchetto("filho", "tentativa durante manutencao"),
            )
            .expect_err("o lucchetto de outro processo deve bloquear")
            {
                ErroIo::ArquivoOcupado {
                    conteudo_declarado, ..
                } => assert!(
                    conteudo_declarado.contains("manutencao do pai"),
                    "o filho le' quem esta' dentro: {conteudo_declarado}"
                ),
                outro => panic!("erro inesperado: {outro}"),
            }
        }
        outro => panic!("papel desconhecido: {outro}"),
    }
}

/// Riesegue **questo** binario di test nel ruolo richiesto.
///
/// Un filtro che non trovasse nessun test uscirebbe con successo senza
/// eseguire niente, e il ruolo sembrerebbe superato: per questo si verifica
/// anche che il figlio dichiari di aver eseguito un test.
fn convocar_filho(diretorio: &Path, papel: &str) -> std::process::Output {
    let eu = env::current_exe().expect("o binario de teste tem um caminho");
    let saida = Command::new(eu)
        .args(["--exact", "papel_de_processo_filho", "--nocapture"])
        .env(VAR_DIRETORIO, diretorio)
        .env(VAR_PAPEL, papel)
        .output()
        .expect("o processo filho deve arrancar");
    let relato = String::from_utf8_lossy(&saida.stdout);
    assert!(
        relato.contains("1 passed") || !saida.status.success(),
        "o papel {papel} nao foi executado: {relato}"
    );
    saida
}

// ---------------------------------------------------------------------------
// TP-19
// ---------------------------------------------------------------------------

#[test]
fn tp19_persistencia_entre_processos() {
    let diretorio = env::temp_dir()
        .join("fantozzi_3c3b1")
        .join(format!("entre_processos_{}", std::process::id()));
    if diretorio.exists() {
        fs::remove_dir_all(&diretorio).unwrap();
    }
    fs::create_dir_all(&diretorio).unwrap();
    let destino = Destino::novo(diretorio.clone(), NOME);

    // Passo 1: il figlio pubblica. Il padre non ha scritto niente.
    assert_eq!(
        arquivo_io::observar(&destino).unwrap(),
        EstadoObservado::DestinoAusente
    );
    let saida = convocar_filho(&diretorio, "publicar");
    assert!(
        saida.status.success(),
        "o filho falhou: {}",
        String::from_utf8_lossy(&saida.stderr)
    );

    // Passo 2: il padre rilegge dal disco. Nessuno stato in memoria puo'
    // spiegare questo risultato: il processo che ha scritto e' finito.
    let leitura = arquivo_io::ler(&destino).expect("o pai le' o que o filho publicou");
    assert_eq!(leitura.base.revisao, 1);
    assert_eq!(leitura.arquivo, arquivo_inicial(), "nenhuma perda no disco");
    assert_eq!(leitura.arquivo.eventos.len(), 1);

    // Passo 3: il padre pubblica la revisione 2 sulla base che ha letto.
    let mut segunda = leitura.arquivo.clone();
    segunda.acrescentar_eventos(&[evento(15, OperacaoRevisao::Anotar)]);
    let publicacao = arquivo_io::publicar(
        &destino,
        &segunda,
        Some(&leitura.base),
        &lucchetto("pai", "segunda publicacao"),
    )
    .unwrap();
    assert_eq!(publicacao.revisao_publicada, 2);
    assert!(publicacao.geracao_anterior.unwrap().exists());

    // Passo 4: un **terzo** processo rilegge la revisione del padre.
    let saida = convocar_filho(&diretorio, "verificar");
    assert!(
        saida.status.success(),
        "o filho nao viu a revisao do pai: {}",
        String::from_utf8_lossy(&saida.stderr)
    );

    // Passo 5: il lucchetto e' cooperativo **fra processi**, non solo fra
    // thread. Il padre lo tiene scrivendo il file, come farebbe un operatore
    // in manutenzione, e il figlio si fermaa.
    let caminho_lucchetto = diretorio.join(format!("{NOME}.lock"));
    fs::write(&caminho_lucchetto, "manutencao do pai").unwrap();
    let saida = convocar_filho(&diretorio, "ocupado");
    assert!(
        saida.status.success(),
        "o filho devia parar com ArquivoOcupado: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(
        fs::read_to_string(&caminho_lucchetto).unwrap(),
        "manutencao do pai",
        "nenhum processo rouba o lucchetto de outro"
    );
    // Il corrente non e' cambiato durante il tentativo bloccato.
    assert_eq!(arquivo_io::ler(&destino).unwrap().base.revisao, 2);
    fs::remove_file(&caminho_lucchetto).unwrap();

    // TP-20: controllo fisico, senza passare per il codec.
    controle_fisico(&destino.corrente(), 1, 2);
}

// ---------------------------------------------------------------------------
// TP-20: controllo indipendente del file
// ---------------------------------------------------------------------------

/// Esamina i **byte** del file come farebbe uno strumento esterno: nessuna
/// deserializzazione, nessun tipo del progetto. Se il codec e il controllo
/// sbagliassero insieme, dovrebbero sbagliare per due vie diverse.
fn controle_fisico(caminho: &Path, snapshots: usize, eventos: usize) {
    let bytes = fs::read(caminho).expect("o arquivo publicado existe");
    assert!(!bytes.is_empty());
    assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]), "nenhum BOM");
    assert!(!bytes.contains(&b'\r'), "nenhum CRLF");
    assert_eq!(*bytes.last().unwrap(), b'\n', "uma LF final");
    assert_eq!(bytes[bytes.len() - 2], b'}', "o objeto fecha antes da LF");
    assert_eq!(bytes[0], b'{', "o arquivo e' um objeto JSON");

    let texto = String::from_utf8(bytes).expect("UTF-8 valido");
    assert!(texto.contains("\"formato\": \"FANTOZZI-ARQUIVO/v1\""));
    assert!(texto.contains("\"versao_schema\": 1"));
    assert!(texto.contains("\"revisao_arquivo\": 2"));
    assert!(texto.contains("\"IMPRESSAO-CONTEUDO/v1\""));
    assert!(texto.contains("\"IMPRESSAO-APRESENTACAO/v1\""));
    assert!(texto.contains("\"EVENTO-EXATO/v1\""));
    // Il valore monetario e' **testo**, con la sua scala.
    assert!(texto.contains("\"valor\": \"10.00\""), "monetario textual");
    assert!(!texto.contains("\"valor\": 10"), "nenhum numero monetario");
    // L'assenza e' scritta, non omessa.
    assert!(texto.contains("\"status\": \"NAO_DISPONIVEL\""));
    assert!(texto.contains("\"status\": \"N.A.\""));
    assert!(texto.contains("\"valor\": null"));
    // I testi non ASCII sopravvivono come caratteri, non come sequenze di
    // escape, e il CPF conserva gli zeri iniziali.
    assert!(texto.contains('\u{00E7}'), "NFC no arquivo");
    assert!(texto.contains("\"00012345678901\""), "zeros a' esquerda");
    // L'istante e' in due campi: la stringa RFC3339 perderebbe i secondi
    // dell'offset (§3.4 della especifica).
    assert!(texto.contains("\"offset_segundos\": -10800"));
    assert!(texto.contains("\"local\": \"2026-10-22T12:00:00.000000000\""));
    // E le quantita' contate si contano sul testo, non sul modello.
    assert_eq!(
        texto.matches("\"impressao_conteudo\": ").count(),
        // una nella grammatica, una per snapshot, una per evento (dentro alvo)
        1 + snapshots + eventos,
        "occorrenze di impressao_conteudo"
    );
    assert_eq!(texto.matches("\"operacao\": ").count(), eventos);
    // due per snapshot (record e contesto), una per riga, una per dichiarazione.
    assert_eq!(
        texto.matches("\"chave_origem\": ").count(),
        2 * snapshots + 1 + eventos
    );
}

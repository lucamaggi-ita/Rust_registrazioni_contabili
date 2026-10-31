//! Il percorso composto della fase 3C.4.1: dall'archivio autorevole al file
//! derivato, con la sola API pubblica.
//!
//! Perche' un test di integrazione: §1.1 della specifica dichiara una
//! composizione di funzioni che esistono in moduli diversi. Un test unitario
//! proverebbe un pezzo; qui si prova che i pezzi si compongono nell'ordine
//! dichiarato, barriera di `construir_pre_livro` compresa.
//!
//! La rilettura indipendente non vive qui: `cargo test` non deve dipendere da
//! Python. Questo test **deposita** il workbook, e
//! `script/releitura_export.py` lo confronta con un comando separato
//! (`EXP-29`).

use std::path::{Path, PathBuf};

use chrono::{FixedOffset, NaiveDate, TimeZone};
use rust_decimal::Decimal;

use rust_registrazioni_contabili::alocacao::{
    AlocacaoNfe, Componente, EsitoComponenti, EsitoCopertura, EsitoRate, EsitoTotal, Reconciliacao,
    RuoloComponente, StatoDocumento,
};
use rust_registrazioni_contabili::arquivo::{self, Arquivo, IdentidadePlano};
use rust_registrazioni_contabili::arquivo_io::{self, Destino, MetadadosLucchetto};
use rust_registrazioni_contabili::export::{self, Celula, Proveniencia};
use rust_registrazioni_contabili::export_io::{self, FonteProtegida, SeExistir};
use rust_registrazioni_contabili::impressao;
use rust_registrazioni_contabili::modelos::{
    Competencia, Dado, Duplicata, Lancamento, OrigemLancamento, StatusDado, StatusRevisao,
    TipoConta,
};
use rust_registrazioni_contabili::prelivro::{ContextoDocumento, IdentidadeLote};
use rust_registrazioni_contabili::regras::PlanoContas;
use rust_registrazioni_contabili::revisao::{self, ChaveRevisao, EventoRevisao, OperacaoRevisao};
use rust_registrazioni_contabili::xlsx;
use sha2::{Digest, Sha256};

const CHAVE: &str = "29260512345678000195550010001256612345678901";
const LOTE: &str = "lote-export-3c41";

fn dec(bruto: &str) -> Decimal {
    Decimal::from_str_exact(bruto).unwrap()
}

fn data(dia: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, dia).unwrap()
}

fn plano() -> PlanoContas {
    PlanoContas::carregar(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("config/plano_contas_gerencial.yaml"),
    )
    .unwrap()
}

fn documento() -> ContextoDocumento {
    let lancamento = Lancamento {
        data_documento: data(5),
        competencia: Competencia::new(2026, 5).unwrap(),
        cnpj_empresa: "55746722000181".to_string(),
        nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
        fornecedor_ou_favorecido: "DISTRIBUIDORA OESA LTDA".to_string(),
        // Zeri iniziali: un CPF non e' un numero.
        cnpj_cpf_favorecido: "00012345678901".to_string(),
        documento: "12566".to_string(),
        tipo_documento: "NFE".to_string(),
        conta_id: "2.01.004".to_string(),
        conta_nome: "Refrigerantes".to_string(),
        grupo_conta: "Custos de Mercadorias e Insumos".to_string(),
        // Testo ostile: formula di foglio, virgolette, NFC.
        descricao_itens: "=1+1 SUCO \"MARCA\" A\u{00E7}ucar".to_string(),
        valor: dec("10.00"),
        duplicatas: vec![Duplicata {
            numero: "001".to_string(),
            vencimento: Some(data(30)),
            valor: dec("10.00"),
        }],
        origem: OrigemLancamento::XmlNfe,
        arquivo_origem: "samples/nfe.xml".to_string(),
        chave_origem: CHAVE.to_string(),
        hash_registro: "payload-export".to_string(),
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
                soma_rate_documento: Dado::presente(dec("10.00"), "nfe:cobr/dup/vDup").unwrap(),
                soma_rate_allocate: Dado::presente(dec("10.00"), "nfe:cobr/dup/vDup").unwrap(),
                esito_rate: EsitoRate::Ok,
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
            id: LOTE.to_string(),
            gerado_em: data(31),
            descricao: "prova do percurso composto".to_string(),
        },
    }
}

fn evento(ctx: &ContextoDocumento, autor: &str, hora: u32) -> EventoRevisao {
    EventoRevisao {
        alvo: ChaveRevisao {
            chave_origem: CHAVE.to_string(),
            impressao_conteudo: Dado::presente(
                impressao::impressao_conteudo(ctx),
                "revisao:conteudo",
            )
            .unwrap(),
            origem: OrigemLancamento::XmlNfe,
            conta_id: "2.01.004".to_string(),
            ordinal: 0,
        },
        operacao: OperacaoRevisao::Aprovar,
        autor: autor.to_string(),
        instante: FixedOffset::east_opt(-3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 23, hora, 0, 0)
            .unwrap(),
        // `R3C41-04`: un ritorno a capo nudo. E' un carattere XML valido,
        // quindi il modello lo accetta e il writer lo codifica; la rilettura
        // con `openpyxl` e' un confronto a parte, registrato come divergenza.
        motivo: "visto e aceito a\rb".to_string(),
        nota: None,
        sequencia: None,
        vista: Dado::presente("v".repeat(64), "revisao:vista").unwrap(),
    }
}

fn arena(nome: &str) -> PathBuf {
    let diretorio = std::env::temp_dir()
        .join("fantozzi_3c41_percurso")
        .join(format!("{nome}_{}", std::process::id()));
    if diretorio.exists() {
        std::fs::remove_dir_all(&diretorio).unwrap();
    }
    std::fs::create_dir_all(&diretorio).unwrap();
    diretorio
}

fn lucchetto(motivo: &str) -> MetadadosLucchetto {
    MetadadosLucchetto {
        operador: "Jos\u{00E9}".to_string(),
        instante: FixedOffset::east_opt(-3 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 23, 9, 0, 0)
            .unwrap(),
        motivo: motivo.to_string(),
    }
}

/// Il percorso completo di §1.1, passo per passo.
struct Percurso {
    diretorio: PathBuf,
    destino_arquivo: Destino,
    modelo: export::ModeloExport,
    proveniencia: Proveniencia,
}

fn percorrer(nome: &str) -> Percurso {
    let diretorio = arena(nome);
    preparar(&diretorio);
    derivar(diretorio)
}

/// Pubblica l'archivio una volta nella directory data: passo 1.
fn preparar(diretorio: &Path) {
    let plano = plano();
    let ctx = documento();
    let mut arquivo = Arquivo::novo();
    arquivo
        .acrescentar_lote(
            std::slice::from_ref(&ctx),
            &plano,
            IdentidadePlano {
                versao: plano.versao().to_string(),
                sha256_arquivo: None,
                caminho_declarado: Some("config/plano_contas_gerencial.yaml".to_string()),
            },
        )
        .unwrap();
    // Due approvazioni di persone diverse: la seconda non sostituisce.
    arquivo.acrescentar_eventos(&[evento(&ctx, "Alice", 12), evento(&ctx, "Bob", 13)]);

    let destino = Destino::novo(diretorio.to_path_buf(), "arquivo.json");
    arquivo_io::publicar(&destino, &arquivo, None, &lucchetto("inicio")).unwrap();
}

/// I passi 2–5: rilettura, sotto-archivio, vista, modello.
///
/// Separato da [`preparar`] perche' il determinismo di `EXP-25` si misura
/// ripetendo **questo**, con gli stessi ingressi, non ripubblicando l'archivio
/// in un'altra directory: due directory sono due ingressi diversi.
fn derivar(diretorio: PathBuf) -> Percurso {
    let plano = plano();
    let destino_arquivo = Destino::novo(diretorio.clone(), "arquivo.json");

    // 2. rilettura e validazione dell'archivio.
    let leitura = arquivo_io::ler(&destino_arquivo).expect("l'archivio si rilegge");

    // 3. EXP-30: il controllo di presentazione sul **sotto-archivio** del lotto.
    let sub = export::sub_arquivo_do_lote(&leitura.arquivo, LOTE).unwrap();
    let divergencias = arquivo::verificar_apresentacao(&sub, &plano).unwrap();

    // 4. la barriera di `construir_pre_livro`, attraversata da aplicar_revisao.
    let contextos = leitura.arquivo.contextos_do_lote(LOTE);
    let eventos = leitura.arquivo.eventos();
    let vista = revisao::aplicar_revisao(&contextos, &plano, &eventos).expect("la vista");

    // 5. il modello dell'export.
    let proveniencia = Proveniencia {
        arquivo_caminho: destino_arquivo.corrente().display().to_string(),
        arquivo_revisao: leitura.base.revisao,
        arquivo_sha256: leitura.base.sha256_conteudo.clone(),
        plano_arquivado: leitura.arquivo.execucoes[0].plano.clone(),
        plano_usado_versao: plano.versao().to_string(),
        plano_usado_caminho: "config/plano_contas_gerencial.yaml".to_string(),
        gerado_em: lucchetto("x").instante,
        divergencias,
        eventos_no_arquivo: eventos.len(),
    };
    let modelo = export::modelo(&vista, &proveniencia).expect("il modello");

    Percurso {
        diretorio,
        destino_arquivo,
        modelo,
        proveniencia,
    }
}
/// SHA-256 esadecimale, calcolata qui: `impressao::sha256_hex` e' interna al
/// crate, e questo lotto non modifica quel modulo.
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Manifesto del modello, per la rilettura indipendente (`R3C41-04`).
///
/// La rilettura confronta **campo per campo**, non a campione: ha bisogno
/// delle attese di tutte le celle, e le attese sono il modello stesso, prima
/// che il writer lo trasformi in byte. Il manifesto quantifica quindi la
/// fedelta' della coppia writer + lettore.
///
/// Non sostituisce le attese esplicite: lo script tiene a parte un insieme di
/// valori scritti a mano (totali, chiavi, stati, autore, motivo), perche' un
/// manifesto generato dallo stesso modello non noterebbe una mutazione del
/// modello.
///
/// Il tipo della cella viaggia con il valore: `vazia` e' un'assenza, e non si
/// confonde con `texto` vuoto ne' con lo zero.
fn manifesto(modelo: &export::ModeloExport) -> serde_json::Value {
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
                        .map(|c| match c {
                            Celula::Texto(t) => {
                                serde_json::json!({"tipo": "texto", "valor": t})
                            }
                            // Il numero viaggia come **testo decimale esatto**:
                            // un JSON con un numero in virgola mobile
                            // introdurrebbe nel manifesto la stessa
                            // approssimazione che il confronto deve misurare.
                            Celula::NumeroDeComodo(d) => {
                                serde_json::json!({"tipo": "numero", "valor": d.to_string()})
                            }
                            Celula::Vazia => serde_json::json!({"tipo": "vazia"}),
                        })
                        .collect();
                    serde_json::Value::Array(celulas)
                })
                .collect();
            serde_json::json!({
                "nome": f.nome,
                "cabecalho": f.cabecalho,
                "linhas": linhas,
            })
        })
        .collect();
    serde_json::json!({
        "formato": export::FORMATO,
        "versao_schema": export::VERSAO_SCHEMA,
        "folhas": folhas,
    })
}

fn celula_texto(folha: &export::Folha, linha: usize, coluna: &str) -> String {
    let j = folha
        .cabecalho
        .iter()
        .position(|c| *c == coluna)
        .unwrap_or_else(|| panic!("colonna {coluna} in {}", folha.nome));
    match &folha.linhas[linha][j] {
        Celula::Texto(t) => t.clone(),
        Celula::NumeroDeComodo(d) => d.to_string(),
        Celula::Vazia => String::new(),
    }
}

#[test]
fn tx01_o_percurso_composto_produz_treze_folhas_coerentes() {
    let p = percorrer("composto");
    assert_eq!(p.modelo.folhas.len(), 13);

    let prov = p.modelo.folha("proveniencia").unwrap();
    assert_eq!(celula_texto(prov, 0, "formato"), export::FORMATO);
    assert_eq!(celula_texto(prov, 0, "lote_id"), LOTE);
    assert_eq!(celula_texto(prov, 0, "arquivo_revisao"), "1");
    assert_eq!(
        celula_texto(prov, 0, "arquivo_sha256"),
        p.proveniencia.arquivo_sha256
    );
    assert_eq!(celula_texto(prov, 0, "plano_coincide"), "SIM");
    assert_eq!(celula_texto(prov, 0, "apresentacao_divergente"), "NAO");
    assert_eq!(celula_texto(prov, 0, "eventos_no_arquivo"), "2");
    assert_eq!(celula_texto(prov, 0, "eventos_aplicados"), "2");
    assert_eq!(celula_texto(prov, 0, "eventos_nao_aplicados"), "0");

    // Il testo ostile e' passato dall'archivio al modello senza alterazioni.
    let diag = p.modelo.folha("diagnostico").unwrap();
    assert_eq!(
        celula_texto(diag, 0, "descricao_itens"),
        "=1+1 SUCO \"MARCA\" A\u{00E7}ucar"
    );
    assert_eq!(
        celula_texto(diag, 0, "cnpj_cpf_favorecido"),
        "00012345678901"
    );
    assert_eq!(celula_texto(diag, 0, "valor"), "10.00");

    // EXP-46: l'approvazione attiva e' la **prima**, non l'ultima ripetizione.
    let extrato = p.modelo.folha("extrato_revisado").unwrap();
    assert_eq!(extrato.linhas.len(), 1);
    assert_eq!(celula_texto(extrato, 0, "aprovado_por"), "Alice");
    assert_eq!(celula_texto(extrato, 0, "confirmacoes"), "1");
    assert_eq!(celula_texto(extrato, 0, "ultima_confirmacao_por"), "Bob");
}

#[test]
fn tx20_duas_derivacoes_dao_o_mesmo_conteudo() {
    // `EXP-25` quantifica su **tutti** gli ingressi: stesso archivio, stesso
    // lotto, stesso piano e stessa `Proveniencia`. Due directory diverse
    // sarebbero due ingressi diversi, non un difetto di determinismo, perche'
    // il percorso dell'archivio e' una cella del foglio `proveniencia`.
    let diretorio = arena("determinismo");
    preparar(&diretorio);
    let a = derivar(diretorio.clone());
    let b = derivar(diretorio);
    assert_eq!(a.proveniencia, b.proveniencia, "stessi ingressi");
    assert_eq!(a.modelo, b.modelo, "stesse celle, in ogni foglio");
}

#[test]
fn tx17_o_arquivo_nao_e_tocado_pelo_export() {
    let p = percorrer("arquivo_intacto");
    let caminho = p.destino_arquivo.corrente();
    let antes = std::fs::read(&caminho).unwrap();

    let pacote = xlsx::pacote(&p.modelo).expect("il pacchetto");
    let fonte = FonteProtegida {
        diretorio: p.diretorio.clone(),
        nome: "arquivo.json".to_string(),
    };
    let destino = p.diretorio.join("export.xlsx");
    let escrita = export_io::escrever(
        &destino,
        &pacote,
        SeExistir::Recusar,
        &fonte,
        "a",
        &lucchetto("export"),
    )
    .expect("la scrittura");

    assert_eq!(
        std::fs::read(&caminho).unwrap(),
        antes,
        "l'archivio e' identico prima e dopo, byte per byte"
    );
    assert_eq!(escrita.bytes, pacote.len());
    assert_eq!(
        escrita.sha256,
        sha256_hex(&pacote),
        "l'impronta e' quella del pacchetto"
    );
    assert!(!escrita.lucchetto_residuo);
    assert_eq!(&pacote[..2], b"PK");
}

/// Deposita il workbook perche' la prova indipendente lo rilegga.
///
/// Non afferma sul contenuto delle celle: da Rust non si vede il **tipo**
/// effettivo. Il confronto e' `script/releitura_export.py --modo export`, con
/// il comando documentato nel README.
#[test]
fn deposita_o_workbook_para_a_releitura_independente() {
    let p = percorrer("deposito");
    let pacote = xlsx::pacote(&p.modelo).expect("il pacchetto");
    let fonte = FonteProtegida {
        diretorio: p.diretorio.clone(),
        nome: "arquivo.json".to_string(),
    };
    let destino = std::env::temp_dir()
        .join("fantozzi_3c41")
        .join("export_percurso.xlsx");
    std::fs::create_dir_all(destino.parent().unwrap()).unwrap();
    if destino.exists() {
        std::fs::remove_file(&destino).unwrap();
    }
    let escrita = export_io::escrever(
        &destino,
        &pacote,
        SeExistir::Substituir,
        &fonte,
        "deposito",
        &lucchetto("deposito"),
    )
    .expect("la scrittura");
    assert!(escrita.caminho.exists());
    println!("EXPORT_WORKBOOK {}", escrita.caminho.display());
    println!("EXPORT_SHA256 {}", escrita.sha256);

    // Il manifesto accanto al workbook: la rilettura confronta campo per campo.
    let manifesto_caminho = destino.with_extension("manifesto.json");
    std::fs::write(
        &manifesto_caminho,
        serde_json::to_vec_pretty(&manifesto(&p.modelo)).expect("il manifesto"),
    )
    .unwrap();
    println!("EXPORT_MANIFESTO {}", manifesto_caminho.display());
}

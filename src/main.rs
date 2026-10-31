//! CLI do porte Rust do projeto `fantozzi` (registros contabeis de NF-e).
//!
//! Fase 1: modelo de dados + leitura contratualizada das regras YAML.
//! Fase 2: leitura de NF-e (quick-xml), normalizacao e classificacao.
//! Fase 3B/3C.2/3C.3: alocacao, pre-livro, revisao e arquivo autoritativo.
//!
//! Fase 3C.5.a: `arquivar-nfe` e `exportar-diagnostico`, a composicao do
//! percurso completo a partir de uma pasta de NF-e.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use clap::{Args, Parser, Subcommand, ValueEnum};

use rust_registrazioni_contabili::arquivo::{self, Arquivo, IdentidadePlano};
use rust_registrazioni_contabili::arquivo_io::{self, Destino, MetadadosLucchetto};
use rust_registrazioni_contabili::classificacao::Classificador;
use rust_registrazioni_contabili::export::{self, Proveniencia};
use rust_registrazioni_contabili::export_io::{self, ErroExportIo, FonteProtegida, SeExistir};
use rust_registrazioni_contabili::importacao::{self, DocumentoColhido};
use rust_registrazioni_contabili::modelos::Confianca;
use rust_registrazioni_contabili::nfe;
use rust_registrazioni_contabili::prelivro::{ContextoDocumento, IdentidadeLote};
use rust_registrazioni_contabili::regras::{
    ErroConfig, PlanoContas, Problema, RegrasClassificacao, Severidade,
};
use rust_registrazioni_contabili::revisao;
use rust_registrazioni_contabili::xlsx;

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

/// Che fare se la destinazione dell'export esiste già (`EXP-23`, D25(b)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SeExiste {
    /// Rifiuta: il file che c'è resta intatto.
    Recusar,
    /// Sostituisce, conservando il precedente.
    Substituir,
}

/// Argomenti di `arquivar-nfe` (fase 3C.5.a).
///
/// Tutto cio' che i moduli rifiutano di dedurre arriva da qui: identita' del
/// lotto, operatore, motivo. L'utente del sistema operativo e l'orologio non
/// sono fonti dichiarate.
#[derive(Debug, Args)]
struct ArgsArquivar {
    /// Pasta com os XML. Busca **nao** recursiva: os arquivos diretamente
    /// dentro dela, em ordem de nome.
    #[arg(long, conflicts_with = "xml")]
    pasta: Option<PathBuf>,
    /// Um unico XML, em vez de uma pasta.
    #[arg(long)]
    xml: Option<PathBuf>,
    /// Caminho completo do arquivo autoritativo corrente.
    #[arg(long)]
    arquivo: PathBuf,
    /// Primeira publicacao: o arquivo **nao** deve existir.
    #[arg(long)]
    inicializar: bool,
    /// Id da execucao. Uma execucao arquivada e' imutavel: cada relancamento
    /// declara um id novo.
    #[arg(long)]
    lote: String,
    #[arg(long, value_name = "AAAA-MM-DD")]
    lote_data: String,
    #[arg(long)]
    lote_descricao: String,
    /// Autor do lucchetto. Nunca o usuario do sistema operacional.
    #[arg(long)]
    operador: String,
    #[arg(long)]
    motivo: String,
    /// Instante RFC 3339 com offset. Omitido: relogio da maquina, e a fonte
    /// aparece na saida (D27).
    #[arg(long)]
    instante: Option<String>,
    #[arg(long, default_value = PADRAO_PLANO)]
    plano_contas: PathBuf,
    #[arg(long, default_value = PADRAO_REGRAS_PRODUTOS)]
    regras_produtos: PathBuf,
    /// Recusa **antes** de publicar se algum documento tiver estado tecnico
    /// diferente de OK (D30).
    #[arg(long)]
    recusar_nao_ok: bool,
}

/// Argomenti di `exportar-diagnostico` (fase 3C.5.a).
#[derive(Debug, Args)]
struct ArgsExportar {
    /// O arquivo autoritativo, em **somente leitura**.
    #[arg(long)]
    arquivo: PathBuf,
    /// A execucao a derivar. Um resultado cobre uma execucao, nao um mes.
    #[arg(long)]
    lote: String,
    #[arg(long)]
    saida: PathBuf,
    #[arg(long, value_enum, default_value_t = SeExiste::Recusar)]
    se_existir: SeExiste,
    /// Etiqueta do temporario; por omissao, o id do lote.
    #[arg(long)]
    etiqueta: Option<String>,
    #[arg(long)]
    operador: String,
    #[arg(long)]
    motivo: String,
    #[arg(long)]
    instante: Option<String>,
    #[arg(long, default_value = PADRAO_PLANO)]
    plano_contas: PathBuf,
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

    /// Arquiva as NF-e de uma pasta no arquivo autoritativo (fase 3C.5.a).
    ///
    /// Rele' a pasta inteira a cada relancamento e cria **uma** execucao nova:
    /// uma execucao arquivada e' imutavel. Nao aprova nada: a importacao nao
    /// cria nenhuma decisao humana.
    ArquivarNfe(ArgsArquivar),

    /// Escreve o Excel diagnostico de UMA execucao arquivada (fase 3C.5.a).
    ///
    /// Le o arquivo em somente leitura e escreve um unico arquivo derivado.
    ExportarDiagnostico(ArgsExportar),

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
        Comando::ArquivarNfe(args) => arquivar_nfe(&args),
        Comando::ExportarDiagnostico(args) => exportar_diagnostico(&args),
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

// ---------------------------------------------------------------------------
// Fase 3C.5.a: arquivar-nfe e exportar-diagnostico
// ---------------------------------------------------------------------------

/// Un rifiuto con il suo codice: il motivo va su stderr, il codice esce.
///
/// `CLI-01`: ogni messaggio dice **che cosa è stato pubblicato**, perché
/// «niente scritto» non è un fatto attestabile se non si nomina il file.
fn recusa(codigo: u8, mensagem: String) -> Result<ExitCode, String> {
    eprintln!("RECUSADO: {mensagem}");
    Ok(ExitCode::from(codigo))
}

/// L'istante e la sua **fonte** (D27).
///
/// L'etichetta vive soltanto qui, nell'uscita della CLI: `Proveniencia` non
/// ha un campo per la fonte dell'istante, e inventarne uno sarebbe una
/// modifica di schema, non composizione. Nessun testo libero in un campo data.
struct Instante {
    valor: DateTime<FixedOffset>,
    fonte: &'static str,
}

fn instante_de(declarado: &Option<String>) -> Result<Instante, String> {
    match declarado {
        None => Ok(Instante {
            valor: Local::now().fixed_offset(),
            fonte: "RELOGIO_DO_SISTEMA",
        }),
        Some(texto) => Ok(Instante {
            valor: DateTime::parse_from_rfc3339(texto)
                .map_err(|e| format!("--instante nao e' RFC 3339 com offset: {texto} ({e})"))?,
            fonte: "DECLARADO",
        }),
    }
}

/// `Destino` da un percorso completo: directory più nome del file.
fn destino_de(caminho: &Path) -> Result<Destino, String> {
    let nome = caminho
        .file_name()
        .ok_or_else(|| format!("--arquivo sem nome de arquivo: {}", caminho.display()))?;
    let diretorio = caminho
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(Destino::novo(
        diretorio.to_path_buf(),
        nome.to_string_lossy().to_string(),
    ))
}

/// Gli XML di una cartella: ricerca **non** ricorsiva, ordine per nome
/// minuscolo.
///
/// È la stessa regola del Python (`cli.py:_find_xml_files`): i file
/// direttamente dentro la cartella. Non si promette la scansione di
/// sottocartelle arbitrarie.
fn xmls_da_pasta(pasta: &Path) -> Result<Vec<PathBuf>, String> {
    let entradas = fs::read_dir(pasta)
        .map_err(|e| format!("nao foi possivel ler a pasta {}: {e}", pasta.display()))?;
    let mut achados: Vec<(String, PathBuf)> = Vec::new();
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("erro ao percorrer {}: {e}", pasta.display()))?;
        let caminho = entrada.path();
        if !caminho.is_file() {
            continue;
        }
        let e_xml = caminho
            .extension()
            .map(|e| e.eq_ignore_ascii_case("xml"))
            .unwrap_or(false);
        if !e_xml {
            continue;
        }
        let ordem = caminho
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        achados.push((ordem, caminho));
    }
    achados.sort();
    Ok(achados.into_iter().map(|(_, c)| c).collect())
}

/// Carica il piano **passando dalla stessa validazione** degli altri comandi.
///
/// `exportar-diagnostico` non classifica niente, quindi non carica le regole;
/// ma non deve nemmeno accettare un piano che `validar-config` boccia.
fn carregar_plano_validado(caminho: &PathBuf) -> Result<PlanoContas, String> {
    let plano = PlanoContas::carregar(caminho).map_err(|e| e.to_string())?;
    let mut problemas = plano.validar();
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
            "plano de contas reprovado: {} erro(s). Rode `validar-config`.",
            erros.len()
        ));
    }
    Ok(plano)
}

/// Un residuo di un tentativo: che cos'è, dove, e quanto misura.
#[derive(Debug)]
struct Residuo {
    papel: &'static str,
    caminho: PathBuf,
    bytes: u64,
}

/// I residui **realmente presenti** di un tentativo di export.
///
/// `R3C5A-I01`: dire «non pubblicato» non basta per riprendere il lavoro. Il
/// protocollo scrive il temporaneo al passo 4 e copia il precedente al passo
/// 5, e un errore nel secondo lascia il primo sul disco; i protocolli non
/// cancellano niente, e un temporaneo rimasto fa fallire il tentativo
/// successivo **con la stessa etichetta** (`ResiduoPresente`). Qui si guarda
/// se i file esistono: nessun file viene dichiarato presente se non c'è, e
/// nessuno viene cancellato.
fn residuos_do_export(saida: &Path, etiqueta: &str) -> Result<Vec<Residuo>, String> {
    let nome = saida
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("{} nao tem nome de arquivo", saida.display()))?;
    let derivado = |suffixo: String| saida.with_file_name(format!("{nome}.{suffixo}"));
    let candidatos = [
        ("TEMPORARIO", derivado(format!("novo.{etiqueta}"))),
        ("ANTERIOR", derivado(format!("anterior.{etiqueta}"))),
        ("LUCCHETTO", derivado("export.lock".to_string())),
    ];
    let mut encontrados = Vec::new();
    for (papel, caminho) in candidatos {
        if let Some(residuo) = olhar_residuo(papel, caminho)? {
            encontrados.push(residuo);
        }
    }
    Ok(encontrados)
}

/// Guarda **un** percorso: presente con la sua dimensione, assente, o la
/// verifica non è possibile.
///
/// I tre esiti sono distinti e nessuno dei due ultimi si traveste da primo.
/// `NotFound` è un'assenza, e un'assenza non è un residuo; qualunque altro
/// errore è una **misura non disponibile**, che risale al chiamante e diventa
/// `NON_DISPONIVEL`. Una dimensione sconosciuta non si scrive `0`: zero byte è
/// una misura reale e possibile, e confonderla con l'ignoto è lo zero tecnico
/// che questo progetto vieta.
fn olhar_residuo(papel: &'static str, caminho: PathBuf) -> Result<Option<Residuo>, String> {
    match std::fs::metadata(&caminho) {
        Ok(dados) => Ok(Some(Residuo {
            papel,
            caminho,
            bytes: dados.len(),
        })),
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(erro) => Err(format!("{}: {erro}", caminho.display())),
    }
}

/// I residui di un tentativo di pubblicazione dell'archivio.
///
/// Riusa [`arquivo_io::candidatos`], che è l'API già provata per elencarli, e
/// ne tiene i temporanei: le generazioni sono conservate per decisione
/// (`D16(a)`) e non sono residui di questo tentativo. Il lucchetto si guarda a
/// parte, perché non è un candidato al recupero.
fn residuos_do_arquivo(destino: &Destino) -> Result<Vec<Residuo>, String> {
    let lista = arquivo_io::candidatos(destino).map_err(|erro| erro.to_string())?;
    let mut encontrados: Vec<Residuo> = Vec::new();
    for candidato in lista
        .candidatos
        .iter()
        .filter(|c| c.papel == arquivo_io::PapelCandidato::TemporarioRestante)
    {
        // Il candidato esisteva quando `candidatos` l'ha elencato; se ora la
        // sua dimensione non si legge, la verifica non e' completa e lo si
        // dice, invece di scrivere una misura inventata.
        if let Some(residuo) = olhar_residuo("TEMPORARIO", candidato.caminho.clone())? {
            encontrados.push(residuo);
        }
    }
    let lucchetto = destino.diretorio.join(format!("{}.lock", destino.nome));
    if let Some(residuo) = olhar_residuo("LUCCHETTO", lucchetto)? {
        encontrados.push(residuo);
    }
    Ok(encontrados)
}

/// Dichiara i residui **dopo** l'errore, senza mai mascherarlo.
///
/// Se la verifica stessa non riesce, lo dice: `NON_DISPONIVEL` è un esito, e
/// non si trasforma in «nessun residuo».
fn declarar_residuos(residuos: Result<Vec<Residuo>, String>) {
    match residuos {
        Ok(lista) if lista.is_empty() => {
            eprintln!("  Residuos do tentativo: nenhum encontrado");
        }
        Ok(lista) => {
            eprintln!(
                "  Residuos do tentativo, NAO apagados automaticamente \
                 (a retomada e' uma escolha humana):"
            );
            for residuo in lista {
                eprintln!(
                    "    {:<11} {} ({} bytes)",
                    residuo.papel,
                    residuo.caminho.display(),
                    residuo.bytes
                );
            }
        }
        Err(motivo) => {
            eprintln!(
                "  Residuos do tentativo: NON_DISPONIVEL ({motivo}): a verificacao \
                 nao foi possivel, e nao se declara que nao existam"
            );
        }
    }
}

fn arquivar_nfe(a: &ArgsArquivar) -> Result<ExitCode, String> {
    use importacao::codigos_arquivar as cod;

    // Tutto ciò che è dichiarato viene convalidato prima di leggere un file.
    let instante = instante_de(&a.instante)?;
    let gerado_em = NaiveDate::parse_from_str(&a.lote_data, "%Y-%m-%d")
        .map_err(|e| format!("--lote-data nao e' AAAA-MM-DD: {} ({e})", a.lote_data))?;
    if a.lote.trim().is_empty() {
        return Err("--lote vazio: a identidade do lote e' declarada".to_string());
    }
    if a.operador.trim().is_empty() {
        return Err(
            "--operador vazio: o autor do lucchetto e' declarado, nunca deduzido".to_string(),
        );
    }
    if a.motivo.trim().is_empty() {
        return Err("--motivo vazio: o motivo do lucchetto fica nos metadados".to_string());
    }

    let caminhos = match (&a.pasta, &a.xml) {
        (Some(pasta), None) => xmls_da_pasta(pasta)?,
        (None, Some(xml)) => vec![xml.clone()],
        (None, None) => return Err("declare --pasta ou --xml".to_string()),
        (Some(_), Some(_)) => return Err("--pasta e --xml se excluem".to_string()),
    };
    if caminhos.is_empty() {
        return Err(format!(
            "nenhum XML em {}: nada a arquivar",
            a.pasta
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ));
    }

    let (plano, regras) = carregar_validado(&a.plano_contas, &a.regras_produtos)?;
    let classificador = Classificador::compilar(&regras);
    let lote = IdentidadeLote {
        id: a.lote.clone(),
        gerado_em,
        descricao: a.lote_descricao.clone(),
    };

    println!("ARQUIVAR NF-e");
    println!("=============");
    match (&a.pasta, &a.xml) {
        (Some(pasta), _) => println!(
            "Fonte      : pasta {} (busca nao recursiva)",
            pasta.display()
        ),
        (_, Some(xml)) => println!("Fonte      : arquivo {}", xml.display()),
        _ => unreachable!("a combinacao foi validada acima"),
    }
    println!("Arquivo    : {}", a.arquivo.display());
    println!(
        "Execucao   : {} de {} | {}",
        lote.id, lote.gerado_em, lote.descricao
    );
    println!(
        "Instante   : {} {}",
        instante.fonte,
        instante.valor.to_rfc3339()
    );
    println!("XML lidos  : {}", caminhos.len());
    println!();

    // 1. Lettura e allocazione: un XML non elaborabile ferma tutto, perché un
    //    lotto è un solo commit e un lotto parziale sarebbe un successo
    //    parziale silenzioso.
    let mut colhidos: Vec<DocumentoColhido> = Vec::with_capacity(caminhos.len());
    for caminho in &caminhos {
        let nome = caminho.display().to_string();
        let importada = match nfe::ler_nfe(caminho) {
            Ok(lida) => lida,
            Err(erro) => {
                return recusa(
                    cod::NAO_PROCESSAVEL,
                    format!("{nome}: {erro}. Arquivo corrente NAO publicado"),
                );
            }
        };
        let contexto =
            match importacao::contexto_de(&importada, &nome, &classificador, &plano, &lote) {
                Ok(contexto) => contexto,
                Err(erro) => {
                    return recusa(
                        cod::NAO_PROCESSAVEL,
                        format!("{erro}. Arquivo corrente NAO publicado"),
                    );
                }
            };
        colhidos.push(DocumentoColhido {
            arquivo: nome,
            contexto,
        });
    }

    // 2. Lo stato dell'archivio. Nessuna creazione implicita (CLI-06).
    let destino = destino_de(&a.arquivo)?;
    let corrente = destino.corrente();
    let leitura = if a.inicializar {
        if corrente.exists() {
            return recusa(
                cod::IO,
                format!(
                    "{} ja' existe: --inicializar e' so' para a primeira publicacao. \
                     Arquivo corrente NAO publicado, nada foi alterado",
                    corrente.display()
                ),
            );
        }
        None
    } else if !corrente.exists() {
        return recusa(
            cod::IO,
            format!(
                "{} nao existe: use --inicializar para a primeira publicacao. \
                 Nada foi criado",
                corrente.display()
            ),
        );
    } else {
        match arquivo_io::ler(&destino) {
            Ok(leitura) => Some(leitura),
            Err(erro) => {
                return recusa(cod::IO, format!("{erro}. Arquivo corrente NAO publicado"));
            }
        }
    };

    // 3. Che cosa l'esecuzione conterrebbe.
    let relatorio = match importacao::classificar(&colhidos, leitura.as_ref().map(|l| &l.arquivo)) {
        Ok(relatorio) => relatorio,
        Err(erro) => {
            return recusa(
                cod::DUPLICADA_NA_EXECUCAO,
                format!("{erro}. Arquivo corrente NAO publicado"),
            );
        }
    };
    for linha in &relatorio.linhas {
        match &linha.esito {
            importacao::EsitoDocumento::JaArquivadoIdentico { lote } => println!(
                "{:<26} {} | {} em {}",
                linha.esito.rotulo(),
                linha.chave_origem,
                linha.arquivo,
                lote
            ),
            _ => println!(
                "{:<26} {} | {}",
                linha.esito.rotulo(),
                linha.chave_origem,
                linha.arquivo
            ),
        }
    }
    println!();

    // 4. Contenuto allocato diverso: rifiuto (D33). Il messaggio non attribuisce
    //    la causa, perché le regole entrano nell'impronta e l'archivio non
    //    conserva la loro identita'.
    let diversos = relatorio.conteudos_diversos();
    if !diversos.is_empty() {
        for linha in &diversos {
            if let importacao::EsitoDocumento::ConteudoDiverso { lote, salva, atual } = &linha.esito
            {
                eprintln!(
                    "  {} ({}): mesma chave, conteudo alocado diverso do arquivado em {}\n    \
                     salva : {salva}\n    atual : {atual}",
                    linha.chave_origem, linha.arquivo, lote
                );
            }
        }
        return recusa(
            cod::CONTEUDO_DIVERSO,
            format!(
                "{} documento(s) com a mesma chave e conteudo alocado diverso. \
                 A diferenca pode vir da fonte OU da configuracao: as regras entram \
                 na impressao e o arquivo nao guarda a identidade delas. \
                 Arquivo corrente NAO publicado",
                diversos.len()
            ),
        );
    }

    // 5. Un'esecuzione archiviata è immutabile (CLI-07).
    if let Some(l) = &leitura
        && l.arquivo.execucoes.iter().any(|e| e.lote.id == lote.id)
    {
        return recusa(
            cod::IO,
            format!(
                "execucao {} ja' presente no arquivo: uma execucao arquivada e' imutavel. \
                 Declare um id de lote novo. Arquivo corrente NAO publicado",
                lote.id
            ),
        );
    }

    // 6. D30: rifiuto prima di pubblicare, oppure pubblicazione con riserva.
    let nao_ok = relatorio.nao_ok();
    if !nao_ok.is_empty() {
        for linha in &nao_ok {
            eprintln!(
                "  {} ({}): estado tecnico {:?}",
                linha.chave_origem, linha.arquivo, linha.stato_documento
            );
        }
        if a.recusar_nao_ok {
            return recusa(
                cod::RECUSADO_NAO_OK,
                format!(
                    "{} documento(s) com estado tecnico diferente de OK e --recusar-nao-ok \
                     declarado. Arquivo corrente NAO publicado",
                    nao_ok.len()
                ),
            );
        }
    }
    let com_reserva = !nao_ok.is_empty();

    // 7. Il candidato, e il solo punto di commit.
    let bytes_plano = fs::read(&a.plano_contas).map_err(|e| {
        format!(
            "nao foi possivel ler {} para a impressao do plano: {e}",
            a.plano_contas.display()
        )
    })?;
    let identidade_plano = IdentidadePlano {
        versao: plano.versao().to_string(),
        // D28: a impressao real do plano, nao `None`.
        sha256_arquivo: Some(arquivo::sha256_dos_bytes(&bytes_plano)),
        caminho_declarado: Some(a.plano_contas.display().to_string()),
    };
    let mut candidato = leitura
        .as_ref()
        .map(|l| l.arquivo.clone())
        .unwrap_or_else(Arquivo::novo);
    let contextos: Vec<ContextoDocumento> = colhidos.iter().map(|c| c.contexto.clone()).collect();
    if let Err(erro) = candidato.acrescentar_lote(&contextos, &plano, identidade_plano) {
        return recusa(cod::IO, format!("{erro}. Arquivo corrente NAO publicado"));
    }
    let lucchetto = MetadadosLucchetto {
        operador: a.operador.clone(),
        instante: instante.valor,
        motivo: a.motivo.clone(),
    };
    let publicacao = match arquivo_io::publicar(
        &destino,
        &candidato,
        leitura.as_ref().map(|l| &l.base),
        &lucchetto,
    ) {
        Ok(publicacao) => publicacao,
        Err(erro) => {
            eprintln!("RECUSADO: {erro}. Arquivo corrente NAO publicado");
            declarar_residuos(residuos_do_arquivo(&destino));
            return Ok(ExitCode::from(cod::IO));
        }
    };

    println!("Arquivo corrente PUBLICADO: {}", corrente.display());
    println!("Revisao            : {}", publicacao.revisao_publicada);
    if let Some(anterior) = &publicacao.geracao_anterior {
        println!("Geracao anterior   : {} (conservada)", anterior.display());
    }
    if publicacao.lucchetto_residuo {
        println!(
            "Lucchetto residuo  : SIM em {}.lock (a publicacao E' valida)",
            corrente.display()
        );
    }
    println!(
        "Documentos         : {} novos, {} ja' arquivados identicos",
        relatorio.novos(),
        relatorio.ja_arquivados()
    );
    println!("Eventos criados    : 0 (a importacao nao aprova nada)");
    println!(
        "Resultado          : cobre a pasta no momento deste relancamento \
         (execucao {})",
        lote.id
    );
    println!(
        "STATUS: {}",
        if com_reserva {
            "ARQUIVADO_COM_RESERVA"
        } else {
            "ARQUIVADO"
        }
    );

    Ok(ExitCode::from(if com_reserva {
        cod::ARQUIVADO_COM_RESERVA
    } else {
        cod::OK
    }))
}

fn exportar_diagnostico(a: &ArgsExportar) -> Result<ExitCode, String> {
    use importacao::codigos_exportar as cod;

    let instante = instante_de(&a.instante)?;
    if a.operador.trim().is_empty() || a.motivo.trim().is_empty() {
        return Err("--operador e --motivo sao declarados, nunca deduzidos".to_string());
    }
    let plano = carregar_plano_validado(&a.plano_contas)?;
    let destino_arquivo = destino_de(&a.arquivo)?;
    let etiqueta = a.etiqueta.clone().unwrap_or_else(|| a.lote.clone());

    println!("EXPORTAR DIAGNOSTICO");
    println!("====================");
    println!("Arquivo    : {} (somente leitura)", a.arquivo.display());
    println!("Execucao   : {}", a.lote);
    println!("Saida      : {}", a.saida.display());
    println!(
        "Instante   : {} {}",
        instante.fonte,
        instante.valor.to_rfc3339()
    );
    println!();

    // 1. Rilettura e validazione dell'archivio.
    let leitura = match arquivo_io::ler(&destino_arquivo) {
        Ok(leitura) => leitura,
        Err(erro) => {
            return recusa(cod::IO, format!("{erro}. XLSX final NAO publicado"));
        }
    };

    // 2. Il sotto-archivio della **sola** esecuzione chiesta: `--lote` accetta
    //    un valore, e non esiste modo di nominarne piu' d'uno (CLI-08).
    let sub = match export::sub_arquivo_do_lote(&leitura.arquivo, &a.lote) {
        Ok(sub) => sub,
        Err(erro) => return Err(format!("{erro}. XLSX final NAO publicado")),
    };
    let divergencias = arquivo::verificar_apresentacao(&sub, &plano)
        .map_err(|erro| format!("{erro}. XLSX final NAO publicado"))?;

    // 3. La barriera di `construir_pre_livro`, attraversata da aplicar_revisao.
    let contextos = leitura.arquivo.contextos_do_lote(&a.lote);
    let eventos = leitura.arquivo.eventos();
    let vista = revisao::aplicar_revisao(&contextos, &plano, &eventos)
        .map_err(|erro| format!("{erro}. XLSX final NAO publicado"))?;

    // 4. Il modello, con la provenienza dell'esecuzione chiesta: il piano
    //    archiviato e' quello di **questa** esecuzione, non del primo lotto.
    let execucao = sub
        .execucoes
        .first()
        .ok_or_else(|| format!("execucao {} sem metadados no arquivo", a.lote))?;
    let proveniencia = Proveniencia {
        arquivo_caminho: destino_arquivo.corrente().display().to_string(),
        arquivo_revisao: leitura.base.revisao,
        arquivo_sha256: leitura.base.sha256_conteudo.clone(),
        plano_arquivado: execucao.plano.clone(),
        plano_usado_versao: plano.versao().to_string(),
        plano_usado_caminho: a.plano_contas.display().to_string(),
        gerado_em: instante.valor,
        divergencias,
        eventos_no_arquivo: eventos.len(),
    };
    let modelo = export::modelo(&vista, &proveniencia)
        .map_err(|erro| format!("{erro}. XLSX final NAO publicado"))?;
    let pacote =
        xlsx::pacote(&modelo).map_err(|erro| format!("{erro}. XLSX final NAO publicado"))?;

    // 5. La scrittura, con l'archivio protetto (EXP-42).
    let fonte = FonteProtegida {
        diretorio: destino_arquivo.diretorio.clone(),
        nome: destino_arquivo.nome.clone(),
    };
    let se_existir = match a.se_existir {
        SeExiste::Recusar => SeExistir::Recusar,
        SeExiste::Substituir => SeExistir::Substituir,
    };
    let lucchetto = MetadadosLucchetto {
        operador: a.operador.clone(),
        instante: instante.valor,
        motivo: a.motivo.clone(),
    };
    let escrita = match export_io::escrever(
        &a.saida, &pacote, se_existir, &fonte, &etiqueta, &lucchetto,
    ) {
        Ok(escrita) => escrita,
        Err(erro) => {
            let codigo = match &erro {
                ErroExportIo::DestinoJaExistente { .. } => cod::DESTINO_EXISTENTE,
                ErroExportIo::DestinoCoincideComFonte { .. } => cod::DESTINO_E_FONTE,
                _ => cod::IO,
            };
            eprintln!(
                "RECUSADO: {erro}. XLSX final NAO publicado, o arquivo autoritativo esta' intacto"
            );
            declarar_residuos(residuos_do_export(&a.saida, &etiqueta));
            return Ok(ExitCode::from(codigo));
        }
    };

    println!("XLSX final PUBLICADO: {}", escrita.caminho.display());
    println!("Bytes      : {}", escrita.bytes);
    println!("SHA-256    : {}", escrita.sha256);
    if let Some(anterior) = &escrita.anterior {
        println!("Anterior   : {} (conservado)", anterior.display());
    }
    if escrita.lucchetto_residuo {
        println!("Lucchetto residuo: SIM (a escrita E' valida)");
    }
    println!(
        "Conteudo   : {} documento(s), {} linha(s), execucao {} na revisao {}",
        vista.pre_livro.documentos.len(),
        vista.pre_livro.linhas.len(),
        a.lote,
        leitura.base.revisao
    );
    println!(
        "Limites    : derivado sem retorno; compatibilidade com Microsoft Excel \
         NAO atestada (folha `limites`)"
    );
    println!("STATUS: EXPORTADO");

    Ok(ExitCode::from(cod::OK))
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

    // =================================================================
    // R3C5A-I01: i tre esiti di `olhar_residuo`, distinti
    // =================================================================

    fn arena_residuo(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("fantozzi_3c5a_residuo")
            .join(format!("{nome}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("criar a arena");
        dir
    }

    /// Presente: la dimensione dichiarata è quella **reale**.
    #[test]
    fn residuo_presente_declara_a_dimensao_real() {
        let dir = arena_residuo("presente");
        let caminho = dir.join("x.xlsx.novo.etiqueta");
        fs::write(&caminho, b"abc").expect("escrever");

        let visto = olhar_residuo("TEMPORARIO", caminho.clone()).expect("a verificacao");
        let residuo = visto.expect("o residuo existe");
        assert_eq!(residuo.papel, "TEMPORARIO");
        assert_eq!(residuo.caminho, caminho);
        assert_eq!(residuo.bytes, 3);
    }

    /// Un file di **zero byte** è un residuo con una misura reale, e non si
    /// confonde con un'assenza né con una misura ignota: è il punto per cui
    /// `.unwrap_or(0)` era sbagliato.
    #[test]
    fn zero_bytes_nao_e_ausencia() {
        let dir = arena_residuo("zero");
        let vazio = dir.join("x.xlsx.novo.vazio");
        fs::write(&vazio, b"").expect("escrever o vazio");

        let residuo = olhar_residuo("TEMPORARIO", vazio)
            .expect("a verificacao")
            .expect("um arquivo de zero bytes existe");
        assert_eq!(residuo.bytes, 0, "zero e' uma medida real");

        // Assente: nessun residuo, e **non** un errore.
        let ausente = dir.join("x.xlsx.novo.que_nao_existe");
        assert!(
            olhar_residuo("TEMPORARIO", ausente)
                .expect("ausencia nao e' erro")
                .is_none()
        );
    }

    /// Verifica impossibile: risale come errore, e diventa `NON_DISPONIVEL`
    /// nel riassunto. **Non** diventa zero e **non** diventa un'assenza.
    ///
    /// Quali caratteri rendono invalido un nome è una proprietà del sistema:
    /// misurato qui su Windows, `in*valido?` dà `InvalidFilename` (errore 123)
    /// e non `NotFound`. Su un altro sistema quegli stessi caratteri sono
    /// legittimi, quindi il test è dichiarato per questa piattaforma.
    #[cfg(windows)]
    #[test]
    fn verificacao_impossivel_nao_vira_zero_nem_ausencia() {
        let dir = arena_residuo("invalido");
        let invalido = dir.join("in*valido?");

        let erro = olhar_residuo("TEMPORARIO", invalido)
            .expect_err("um nome invalido nao permite declarar nem presenca nem ausencia");
        assert!(
            erro.contains("in*valido?"),
            "o erro nomeia o caminho: {erro}"
        );
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

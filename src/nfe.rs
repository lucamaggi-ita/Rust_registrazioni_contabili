//! Lettura di NF-e da XML.
//!
//! Porting di `src/fantozzi/nfe_importer.py`, che usa `ElementTree`. Qui si usa
//! `quick-xml` con risoluzione dei namespace: si costruisce un albero minimo
//! dei soli elementi nel namespace NF-e e lo si naviga come fa il Python.
//!
//! Contratto C10: il Python converte in zero ogni totale assente
//! (`Decimal(_text(elem, tag) or "0")`). Qui l'assenza **non diventa zero**:
//! un valore obbligatorio per il layout e' un errore con il percorso del
//! campo, un importo facoltativo assente resta assente in un [`Dado`] con
//! stato e motivo (`N.A.`, `NAO_DISPONIVEL`, `MANCANTE`), e un aggregato a
//! cui manca un componente esce `PARCIAL`. Nessun valore pubblico e' zero
//! solo perche' la tag non c'era.
//!
//! Contratto C17: il documento deve contenere **una sola radice XML e una
//! sola NF-e**. Piu' radici (due documenti concatenati) o piu' `<infNFe>`
//! nello stesso documento sono un errore con uscita 1, non un successo
//! parziale sulla prima fattura con le altre scartate in silenzio.

use std::path::Path;

use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::modelos::{Dado, Duplicata, ErroDado, Nfe, ProdutoNfe, StatusDado};

/// Namespace di tutti gli elementi NF-e.
pub const NS_NFE: &str = "http://www.portalfiscal.inf.br/nfe";

/// Profondita' massima accettata: un XML piu' annidato di cosi' non e' una NF-e.
const PROFUNDIDADE_MAXIMA: usize = 64;
/// Numero massimo di elementi: argine a file patologici.
const ELEMENTOS_MAXIMO: usize = 200_000;

// ---------------------------------------------------------------------------
// Errori e avvisi
// ---------------------------------------------------------------------------

/// Errore che impedisce di produrre una [`Nfe`].
#[derive(Debug)]
pub enum ErroNfe {
    Leitura {
        caminho: String,
        erro: std::io::Error,
    },
    Xml {
        origem: String,
        erro: quick_xml::Error,
    },
    /// Testo non decodificabile: entita' XML malformata.
    Texto { origem: String, detalhe: String },
    /// Nessun `<infNFe>`: evento, cancellazione o file non NF-e.
    NaoEhNfe { origem: String },
    /// Piu' di un elemento radice: non e' un documento XML ben formato, ed e'
    /// il caso dei due XML concatenati. C17: niente lettura parziale.
    DocumentoComMultiplasRaizes { origem: String },
    /// Documento ben formato ma con piu' di una NF-e (per esempio un lotto
    /// `enviNFe`). C17: leggere solo la prima perderebbe le altre in silenzio.
    MultiplasNfeNoDocumento { origem: String, encontradas: usize },
    /// Elemento obbligatorio assente (il Python fallirebbe allo stesso punto).
    CampoObrigatorio { origem: String, campo: &'static str },
    /// Valore obbligatorio per il layout assente: non diventa zero.
    ValorObrigatorioAusente { origem: String, caminho: String },
    /// Il lettore ha provato a costruire un dato incoerente: e' un bug qui,
    /// non un difetto della NF-e. Non resta silenzioso.
    ModeloIncoerente { origem: String, erro: ErroDado },
    ValorNumericoInvalido {
        origem: String,
        campo: String,
        valor: String,
    },
    DataInvalida {
        origem: String,
        campo: &'static str,
        valor: String,
    },
    LimiteExcedido {
        origem: String,
        limite: &'static str,
    },
}

impl std::fmt::Display for ErroNfe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroNfe::Leitura { caminho, erro } => {
                write!(f, "nao foi possivel ler {caminho}: {erro}")
            }
            ErroNfe::Xml { origem, erro } => write!(f, "XML invalido em {origem}: {erro}"),
            ErroNfe::Texto { origem, detalhe } => {
                write!(f, "texto XML invalido em {origem}: {detalhe}")
            }
            ErroNfe::NaoEhNfe { origem } => {
                write!(f, "nao e' uma NF-e valida (sem <infNFe>): {origem}")
            }
            ErroNfe::DocumentoComMultiplasRaizes { origem } => write!(
                f,
                "{origem}: XML com mais de um elemento raiz (documentos concatenados); \
                 nenhuma NF-e foi lida"
            ),
            ErroNfe::MultiplasNfeNoDocumento {
                origem,
                encontradas,
            } => write!(
                f,
                "{origem}: {encontradas} <infNFe> no mesmo documento; \
                 importar-nfe le uma NF-e por arquivo e nao descarta as outras em silencio"
            ),
            ErroNfe::CampoObrigatorio { origem, campo } => {
                write!(f, "{origem}: elemento obrigatorio ausente: {campo}")
            }
            ErroNfe::ValorObrigatorioAusente { origem, caminho } => write!(
                f,
                "{origem}: valor obrigatorio ausente em {caminho} (ausente nao e' zero)"
            ),
            ErroNfe::ModeloIncoerente { origem, erro } => {
                write!(f, "{origem}: dado incoerente montado pelo leitor: {erro}")
            }
            ErroNfe::ValorNumericoInvalido {
                origem,
                campo,
                valor,
            } => {
                write!(f, "{origem}: valor nao numerico em {campo}: {valor:?}")
            }
            ErroNfe::DataInvalida {
                origem,
                campo,
                valor,
            } => {
                write!(f, "{origem}: data invalida em {campo}: {valor:?}")
            }
            ErroNfe::LimiteExcedido { origem, limite } => {
                write!(f, "{origem}: limite excedido ({limite})")
            }
        }
    }
}

impl std::error::Error for ErroNfe {}

/// Fatto registrato durante la lettura che non impedisce il risultato ma che
/// nessuno deve scoprire a valle per caso.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aviso {
    /// Codice stabile, citabile nei test e nell'audit. I valori ammessi sono
    /// i letterali passati a [`Aviso::novo`].
    pub codigo: String,
    pub detalhe: String,
}

impl Aviso {
    fn novo(codigo: &'static str, detalhe: impl Into<String>) -> Self {
        Self {
            codigo: codigo.to_string(),
            detalhe: detalhe.into(),
        }
    }
}

impl std::fmt::Display for Aviso {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<26} {}", self.codigo, self.detalhe)
    }
}

/// NF-e letta, insieme a tutto cio' che la lettura ha dovuto assumere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NfeImportada {
    pub nfe: Nfe,
    pub avisos: Vec<Aviso>,
}

// ---------------------------------------------------------------------------
// Albero minimo
// ---------------------------------------------------------------------------

/// Elemento XML nel namespace NF-e. Gli elementi di altri namespace (per
/// esempio la firma `ds:Signature`) non entrano nell'albero, esattamente come
/// non entrano nelle `find()` namespacizzate del Python.
#[derive(Debug, Default)]
struct Elemento {
    nome: String,
    id: Option<String>,
    n_item: Option<String>,
    texto: String,
    filhos: Vec<Elemento>,
}

impl Elemento {
    fn filho(&self, nome: &str) -> Option<&Elemento> {
        self.filhos.iter().find(|e| e.nome == nome)
    }

    fn filhos_chamados<'a>(&'a self, nome: &'a str) -> impl Iterator<Item = &'a Elemento> {
        self.filhos.iter().filter(move |e| e.nome == nome)
    }

    /// Primo discendente con quel nome, in profondita' (come `.//tag`).
    fn descendente(&self, nome: &str) -> Option<&Elemento> {
        if self.nome == nome {
            return Some(self);
        }
        self.filhos.iter().find_map(|f| f.descendente(nome))
    }

    /// Quante occorrenze con quel nome esistono (come `len(.//tag)`).
    ///
    /// A differenza di [`Elemento::descendente`], la ricerca **non si ferma**
    /// sull'elemento trovato: scende anche sotto di lui. Riscontro B1
    /// dell'audit del 2026-10-16: fermandosi, un `<infNFe>` annidato dentro un
    /// altro contava come uno e C17 lasciava passare il documento.
    fn contar_descendentes(&self, nome: &str) -> usize {
        let proprio = usize::from(self.nome == nome);
        proprio
            + self
                .filhos
                .iter()
                .map(|f| f.contar_descendentes(nome))
                .sum::<usize>()
    }

    /// Testo di un figlio, stringa vuota se assente (come `_text` del Python).
    fn texto_de(&self, nome: &str) -> String {
        self.filho(nome)
            .map(|e| e.texto.trim().to_string())
            .unwrap_or_default()
    }
}

/// Costruisce l'albero dei soli elementi nel namespace NF-e.
fn construir_arvore(xml: &str, origem: &str) -> Result<Elemento, ErroNfe> {
    let mut leitor = quick_xml::NsReader::from_str(xml);

    let mut raiz = Elemento {
        nome: "#documento".to_string(),
        ..Default::default()
    };
    let mut pilha: Vec<Elemento> = Vec::new();
    // Profondita' degli elementi ignorati (fuori dal namespace NF-e).
    let mut ignorados = 0usize;
    let mut total = 0usize;
    // C17: elementi aperti al livello della radice. `quick-xml` non impone la
    // radice unica, quindi due documenti concatenati arriverebbero qui come
    // due figli di `#documento` e la seconda NF-e sparirebbe.
    let mut raizes = 0usize;

    loop {
        let (ns, evento) = leitor.read_resolved_event().map_err(|erro| ErroNfe::Xml {
            origem: origem.to_string(),
            erro,
        })?;

        match evento {
            Event::Eof => break,
            Event::Start(inicio) => {
                let no_namespace = matches!(ns, ResolveResult::Bound(Namespace(n)) if n == NS_NFE);
                // Il conteggio delle radici precede il filtro di namespace:
                // anche una seconda radice fuori dal namespace NF-e e' un
                // secondo documento, non un elemento da ignorare.
                if ignorados == 0 && pilha.is_empty() {
                    contar_raiz(&mut raizes, origem)?;
                }
                // Un elemento fuori namespace nasconde tutto il suo sottoalbero
                // (per esempio ds:Signature), come fanno le find() del Python.
                if ignorados > 0 || !no_namespace {
                    ignorados += 1;
                    continue;
                }
                total += 1;
                verificar_limites(total, pilha.len(), origem)?;
                pilha.push(montar_elemento(&inicio));
            }
            Event::Empty(inicio) => {
                let no_namespace = matches!(ns, ResolveResult::Bound(Namespace(n)) if n == NS_NFE);
                if ignorados == 0 && pilha.is_empty() {
                    contar_raiz(&mut raizes, origem)?;
                }
                // `<tag/>` non produce un End: va chiuso qui, altrimenti la
                // pila si disallinea.
                if ignorados > 0 || !no_namespace {
                    continue;
                }
                total += 1;
                verificar_limites(total, pilha.len(), origem)?;
                let elemento = montar_elemento(&inicio);
                match pilha.last_mut() {
                    Some(pai) => pai.filhos.push(elemento),
                    None => raiz.filhos.push(elemento),
                }
            }
            Event::End(_) => {
                if ignorados > 0 {
                    ignorados -= 1;
                    continue;
                }
                if let Some(terminado) = pilha.pop() {
                    match pilha.last_mut() {
                        Some(pai) => pai.filhos.push(terminado),
                        None => raiz.filhos.push(terminado),
                    }
                }
            }
            Event::Text(texto) => {
                if ignorados == 0
                    && let Some(atual) = pilha.last_mut()
                {
                    // Le entita' arrivano come evento separato (`GeneralRef`),
                    // quindi qui il testo e' gia' letterale e non va ri-decodificato.
                    atual.texto.push_str(&texto.xml10_content());
                }
            }
            // `&amp;`, `&#46;`, `&#x2E;`. Senza questo ramo il riferimento
            // sparirebbe in silenzio e "1&#46;25" diventerebbe "125".
            Event::GeneralRef(referencia) => {
                if ignorados == 0
                    && let Some(atual) = pilha.last_mut()
                {
                    let nome = referencia.xml10_content();
                    let completa = format!("&{nome};");
                    let resolvido =
                        quick_xml::escape::unescape(&completa).map_err(|erro| ErroNfe::Texto {
                            origem: origem.to_string(),
                            detalhe: format!("referencia {completa}: {erro}"),
                        })?;
                    atual.texto.push_str(&resolvido);
                }
            }
            // CDATA e' contenuto letterale: entra tale e quale.
            Event::CData(cdata) => {
                if ignorados == 0
                    && let Some(atual) = pilha.last_mut()
                {
                    atual.texto.push_str(&cdata.into_inner());
                }
            }
            _ => {}
        }
    }

    Ok(raiz)
}

fn montar_elemento(inicio: &quick_xml::events::BytesStart<'_>) -> Elemento {
    let nome = inicio.local_name().as_ref().to_string();
    let mut elemento = Elemento {
        nome,
        ..Default::default()
    };
    for atributo in inicio.attributes().flatten() {
        let valor = atributo.value.as_ref().to_string();
        match atributo.key.as_ref() {
            "Id" => elemento.id = Some(valor),
            "nItem" => elemento.n_item = Some(valor),
            _ => {}
        }
    }
    elemento
}

/// C17: conta un elemento aperto al livello della radice e rifiuta il secondo.
fn contar_raiz(raizes: &mut usize, origem: &str) -> Result<(), ErroNfe> {
    *raizes += 1;
    if *raizes > 1 {
        return Err(ErroNfe::DocumentoComMultiplasRaizes {
            origem: origem.to_string(),
        });
    }
    Ok(())
}

fn verificar_limites(total: usize, profundidade: usize, origem: &str) -> Result<(), ErroNfe> {
    if total > ELEMENTOS_MAXIMO {
        return Err(ErroNfe::LimiteExcedido {
            origem: origem.to_string(),
            limite: "numero de elementos",
        });
    }
    if profundidade >= PROFUNDIDADE_MAXIMA {
        return Err(ErroNfe::LimiteExcedido {
            origem: origem.to_string(),
            limite: "profundidade do XML",
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Lettura
// ---------------------------------------------------------------------------

/// Legge una NF-e da file.
pub fn ler_nfe(caminho: impl AsRef<Path>) -> Result<NfeImportada, ErroNfe> {
    let caminho = caminho.as_ref();
    let origem = caminho.display().to_string();
    let xml = std::fs::read_to_string(caminho).map_err(|erro| ErroNfe::Leitura {
        caminho: origem.clone(),
        erro,
    })?;
    ler_nfe_de_texto(&xml, &origem)
}

/// Legge una NF-e da testo XML. `origem` finisce in `Nfe::arquivo_origem`.
pub fn ler_nfe_de_texto(xml: &str, origem: &str) -> Result<NfeImportada, ErroNfe> {
    let documento = construir_arvore(xml, origem)?;
    // C17: un documento ben formato puo' comunque portare un lotto di NF-e
    // (`enviNFe`). Leggerne solo la prima perderebbe le altre in silenzio.
    let encontradas = documento.contar_descendentes("infNFe");
    if encontradas > 1 {
        return Err(ErroNfe::MultiplasNfeNoDocumento {
            origem: origem.to_string(),
            encontradas,
        });
    }
    let inf_nfe = documento
        .descendente("infNFe")
        .ok_or_else(|| ErroNfe::NaoEhNfe {
            origem: origem.to_string(),
        })?;

    let mut avisos = Vec::new();

    // chave: l'attributo Id porta il prefisso "NFe"
    let id_bruto = inf_nfe.id.clone().unwrap_or_default();
    let chave = id_bruto
        .strip_prefix("NFe")
        .unwrap_or(&id_bruto)
        .to_string();
    if chave.is_empty() {
        avisos.push(Aviso::novo("CHAVE_AUSENTE", "infNFe sem atributo Id"));
    }

    let ide = obrigatorio(inf_nfe, "ide", origem)?;
    let emit = obrigatorio(inf_nfe, "emit", origem)?;
    let dest = obrigatorio(inf_nfe, "dest", origem)?;

    let data_bruta = ide.texto_de("dhEmi");
    let data_emissao = analisar_data(&data_bruta, "dhEmi", origem)?;

    // dest usa CNPJ per le persone giuridiche o CPF per le fisiche
    let cnpj_destinatario = {
        let cnpj = dest.texto_de("CNPJ");
        if cnpj.is_empty() {
            dest.texto_de("CPF")
        } else {
            cnpj
        }
    };

    for (campo, valor) in [
        ("cnpj_emitente", emit.texto_de("CNPJ")),
        ("nome_emitente", emit.texto_de("xNome")),
        ("cnpj_destinatario", cnpj_destinatario.clone()),
        ("nome_destinatario", dest.texto_de("xNome")),
    ] {
        if valor.trim().is_empty() {
            avisos.push(Aviso::novo("CAMPO_IDENTIDADE_VAZIO", campo));
        }
    }

    // --- totali ---
    // `total/ICMSTot` e' obbligatorio nel layout 4.00. Tollerarlo assente
    // significherebbe produrre una NF-e da zero reais: zero tecnico vietato.
    let total = obrigatorio(inf_nfe, "total", origem)?;
    let icms_tot = obrigatorio(total, "ICMSTot", origem)?;

    let valor_total = decimal_obrigatorio(icms_tot, "vNF", "total/ICMSTot/vNF", origem)?;
    let valor_produtos = decimal_obrigatorio(icms_tot, "vProd", "total/ICMSTot/vProd", origem)?;
    let valor_frete = decimal_obrigatorio(icms_tot, "vFrete", "total/ICMSTot/vFrete", origem)?;
    let valor_seguro = decimal_obrigatorio(icms_tot, "vSeg", "total/ICMSTot/vSeg", origem)?;
    let valor_desconto = decimal_obrigatorio(icms_tot, "vDesc", "total/ICMSTot/vDesc", origem)?;
    let valor_outras_despesas =
        decimal_obrigatorio(icms_tot, "vOutro", "total/ICMSTot/vOutro", origem)?;

    // vFCPST e vIPIDevol sono condizionali nel layout. Se mancano, la somma
    // esiste ma copre meno componenti: l'aggregato esce `Parcial` e dice
    // quali mancano, invece di sembrare completo.
    let mut componentes_ausentes: Vec<&str> = Vec::new();
    let mut componente_condicional = |tag: &'static str| -> Result<Decimal, ErroNfe> {
        let bruto = icms_tot.texto_de(tag);
        if bruto.is_empty() {
            componentes_ausentes.push(tag);
            return Ok(Decimal::ZERO);
        }
        analisar_decimal(&bruto, tag, origem)
    };
    let soma_tributos = decimal_obrigatorio(icms_tot, "vST", "total/ICMSTot/vST", origem)?
        + componente_condicional("vFCPST")?
        + decimal_obrigatorio(icms_tot, "vII", "total/ICMSTot/vII", origem)?
        + decimal_obrigatorio(icms_tot, "vIPI", "total/ICMSTot/vIPI", origem)?
        + componente_condicional("vIPIDevol")?;

    let valor_tributos_inclusos = if componentes_ausentes.is_empty() {
        criar_dado(Dado::presente(soma_tributos, "nfe:total/ICMSTot"), origem)?
    } else {
        let motivo = format!(
            "componentes condicionais ausentes: {}",
            componentes_ausentes.join(", ")
        );
        avisos.push(Aviso::novo(
            "AGREGADO_PARCIAL",
            format!("total/ICMSTot: {motivo}"),
        ));
        criar_dado(
            Dado::novo(
                Some(soma_tributos),
                "nfe:total/ICMSTot",
                StatusDado::Parcial,
                Some(motivo),
            ),
            origem,
        )?
    };

    // --- prodotti ---
    let mut produtos = Vec::new();
    for det in inf_nfe.filhos_chamados("det") {
        let n_item_bruto = det.n_item.clone().unwrap_or_else(|| "0".to_string());
        let n_item: u32 = n_item_bruto
            .parse()
            .map_err(|_| ErroNfe::ValorNumericoInvalido {
                origem: origem.to_string(),
                campo: "det/@nItem".to_string(),
                valor: n_item_bruto.clone(),
            })?;

        let Some(prod) = det.filho("prod") else {
            avisos.push(Aviso::novo(
                "DET_SEM_PROD",
                format!("item {n_item} ignorado: <det> sem <prod>"),
            ));
            continue;
        };

        produtos.push(ProdutoNfe {
            n_item,
            codigo: prod.texto_de("cProd"),
            descricao: prod.texto_de("xProd"),
            ncm: prod.texto_de("NCM"),
            cfop: prod.texto_de("CFOP"),
            unidade: prod.texto_de("uCom"),
            // Obbligatori nel layout: un item senza quantita' o valore non
            // puo' entrare in contabilita' come zero.
            quantidade: decimal_obrigatorio(
                prod,
                "qCom",
                &format!("det[{n_item}]/prod/qCom"),
                origem,
            )?,
            valor_unitario: decimal_obrigatorio(
                prod,
                "vUnCom",
                &format!("det[{n_item}]/prod/vUnCom"),
                origem,
            )?,
            valor_total: decimal_obrigatorio(
                prod,
                "vProd",
                &format!("det[{n_item}]/prod/vProd"),
                origem,
            )?,
        });
    }

    if produtos.is_empty() {
        avisos.push(Aviso::novo(
            "SEM_PRODUTOS",
            "NF-e sem nenhum item <det>/<prod>",
        ));
    }

    // --- cobranca e duplicate ---
    let cobr = inf_nfe.filho("cobr");
    let mut duplicatas = Vec::new();

    // Contratto C10: quattro situazioni diverse, quattro stati diversi.
    // Nessuna di esse produce un importo zero inventato, e l'importo non
    // viene mai ricostruito sommando le rate.
    let valor_cobranca = match cobr {
        None => {
            // Senza <cobr> la fatturazione non si applica a questa NF-e.
            criar_dado(
                Dado::ausente(
                    "nfe:cobr",
                    StatusDado::NaoAplicavel,
                    "NF-e sem bloco <cobr>",
                ),
                origem,
            )?
        }
        Some(c) => match c.filho("fat") {
            None => {
                avisos.push(Aviso::novo(
                    "FATURA_AUSENTE",
                    "<cobr> sem <fat>: valor de faturamento nao disponivel",
                ));
                criar_dado(
                    Dado::ausente(
                        "nfe:cobr/fat",
                        StatusDado::NaoDisponivel,
                        "<cobr> presente mas sem bloco <fat>",
                    ),
                    origem,
                )?
            }
            Some(fat) => {
                let bruto = fat.texto_de("vLiq");
                if bruto.is_empty() {
                    avisos.push(Aviso::novo(
                        "VALOR_FATURA_AUSENTE",
                        "cobr/fat sem <vLiq>: valor de faturamento ausente",
                    ));
                    criar_dado(
                        Dado::ausente(
                            "nfe:cobr/fat/vLiq",
                            StatusDado::Mancante,
                            "<fat> presente mas sem <vLiq>",
                        ),
                        origem,
                    )?
                } else {
                    // Zero dichiarato dalla fonte resta zero, con stato OK.
                    let valor = analisar_decimal(&bruto, "cobr/fat/vLiq", origem)?;
                    criar_dado(Dado::presente(valor, "nfe:cobr/fat/vLiq"), origem)?
                }
            }
        },
    };

    if let Some(cobr) = cobr {
        for dup in cobr.filhos_chamados("dup") {
            let venc_bruto = dup.texto_de("dVenc");
            let vencimento = if venc_bruto.is_empty() {
                avisos.push(Aviso::novo(
                    "DUPLICATA_SEM_VENCIMENTO",
                    format!("duplicata {:?} sem dVenc", dup.texto_de("nDup")),
                ));
                None
            } else {
                Some(analisar_data(&venc_bruto, "dVenc", origem)?)
            };
            duplicatas.push(Duplicata {
                numero: dup.texto_de("nDup"),
                vencimento,
                // vDup e' obbligatorio quando esiste <dup>.
                valor: decimal_obrigatorio(dup, "vDup", "cobr/dup/vDup", origem)?,
            });
        }
    }

    let soma_itens: Decimal = produtos.iter().map(|p| p.valor_total).sum();
    if soma_itens != valor_produtos {
        avisos.push(Aviso::novo(
            "SOMA_ITENS_DIVERGE",
            format!("soma dos itens {soma_itens} != vProd {valor_produtos}"),
        ));
    }

    let nfe = Nfe {
        chave,
        numero: ide.texto_de("nNF"),
        serie: ide.texto_de("serie"),
        data_emissao,
        cnpj_emitente: emit.texto_de("CNPJ"),
        nome_emitente: emit.texto_de("xNome"),
        cnpj_destinatario,
        nome_destinatario: dest.texto_de("xNome"),
        valor_total,
        natureza_operacao: ide.texto_de("natOp"),
        valor_produtos,
        valor_frete,
        valor_seguro,
        valor_desconto,
        valor_outras_despesas,
        valor_tributos_inclusos,
        produtos,
        duplicatas: duplicatas.clone(),
        tem_duplicatas: !duplicatas.is_empty(),
        tem_cobranca: cobr.is_some(),
        valor_cobranca,
        arquivo_origem: origem.to_string(),
    };

    Ok(NfeImportada { nfe, avisos })
}

fn obrigatorio<'a>(
    pai: &'a Elemento,
    nome: &'static str,
    origem: &str,
) -> Result<&'a Elemento, ErroNfe> {
    pai.filho(nome).ok_or_else(|| ErroNfe::CampoObrigatorio {
        origem: origem.to_string(),
        campo: nome,
    })
}

fn analisar_decimal(bruto: &str, campo: &str, origem: &str) -> Result<Decimal, ErroNfe> {
    Decimal::from_str_exact(bruto).map_err(|_| ErroNfe::ValorNumericoInvalido {
        origem: origem.to_string(),
        campo: campo.to_string(),
        valor: bruto.to_string(),
    })
}

/// Valore obbligatorio per il layout NF-e 4.00.
///
/// Divergenza dichiarata dal Python: la' `_decimal` restituisce zero per un
/// tag assente, e lo zero entra in contabilita' come se fosse un valore
/// misurato. Qui l'assenza e' un errore che nomina il percorso esatto.
fn decimal_obrigatorio(
    pai: &Elemento,
    tag: &str,
    caminho: &str,
    origem: &str,
) -> Result<Decimal, ErroNfe> {
    let bruto = pai.texto_de(tag);
    if bruto.is_empty() {
        return Err(ErroNfe::ValorObrigatorioAusente {
            origem: origem.to_string(),
            caminho: caminho.to_string(),
        });
    }
    analisar_decimal(&bruto, caminho, origem)
}

/// Converte un errore di coerenza del modello in errore di lettura.
///
/// Non puo' scattare con gli ingressi costruiti qui: se scatta e' un bug del
/// lettore, e deve essere visibile invece di diventare un `unwrap`.
fn criar_dado<T>(resultado: Result<Dado<T>, ErroDado>, origem: &str) -> Result<Dado<T>, ErroNfe> {
    resultado.map_err(|erro| ErroNfe::ModeloIncoerente {
        origem: origem.to_string(),
        erro,
    })
}

/// Come `_parse_date` del Python: regge `2026-05-05T10:00:00-03:00` e `2026-05-20`.
fn analisar_data(
    bruto: &str,
    campo: &'static str,
    origem: &str,
) -> Result<chrono::NaiveDate, ErroNfe> {
    let so_data = bruto.split('T').next().unwrap_or("");
    chrono::NaiveDate::parse_from_str(so_data, "%Y-%m-%d").map_err(|_| ErroNfe::DataInvalida {
        origem: origem.to_string(),
        campo,
        valor: bruto.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn caminho_exemplo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("samples/nfe_exemplo/nfe_oesa_12566_20260505.xml")
    }

    fn exemplo() -> NfeImportada {
        ler_nfe(caminho_exemplo()).expect("a NF-e de exemplo deve ser legivel")
    }

    // --- Stessi valori attesi di tests/test_nfe_importer.py -----------------

    #[test]
    fn numero_da_nfe() {
        assert_eq!(exemplo().nfe.numero, "12566");
    }

    #[test]
    fn valor_total() {
        assert_eq!(
            exemplo().nfe.valor_total,
            Decimal::from_str_exact("35.00").unwrap()
        );
    }

    #[test]
    fn natureza_da_operacao() {
        assert_eq!(exemplo().nfe.natureza_operacao, "VENDA DE MERCADORIAS");
    }

    #[test]
    fn totais_detalhados() {
        let n = exemplo().nfe;
        let zero = Decimal::from_str_exact("0.00").unwrap();
        assert_eq!(n.valor_produtos, Decimal::from_str_exact("35.00").unwrap());
        assert_eq!(n.valor_frete, zero);
        assert_eq!(n.valor_seguro, zero);
        assert_eq!(n.valor_desconto, zero);
        assert_eq!(n.valor_outras_despesas, zero);
        // Aggregato completo: tutti i componenti presenti nel campione.
        assert_eq!(n.valor_tributos_inclusos.valor(), Some(&zero));
        assert_eq!(n.valor_tributos_inclusos.status(), StatusDado::Ok);
    }

    #[test]
    fn pelo_menos_tres_produtos() {
        assert!(exemplo().nfe.produtos.len() >= 3);
    }

    #[test]
    fn cfop_dos_produtos() {
        let cfops: std::collections::BTreeSet<String> = exemplo()
            .nfe
            .produtos
            .iter()
            .map(|p| p.cfop.clone())
            .collect();
        assert_eq!(cfops, ["5102".to_string()].into_iter().collect());
    }

    #[test]
    fn destinatario_preenchido() {
        let n = exemplo().nfe;
        assert!(!n.cnpj_destinatario.is_empty());
        assert!(!n.nome_destinatario.is_empty());
    }

    #[test]
    fn duas_duplicatas_de_17_50() {
        let n = exemplo().nfe;
        assert_eq!(n.duplicatas.len(), 2);
        for dup in &n.duplicatas {
            assert_eq!(dup.valor, Decimal::from_str_exact("17.50").unwrap());
        }
        assert!(n.tem_duplicatas);
    }

    #[test]
    fn cobranca() {
        let n = exemplo().nfe;
        assert!(n.tem_cobranca);
        assert_eq!(
            n.valor_cobranca.valor(),
            Some(&Decimal::from_str_exact("35.00").unwrap())
        );
        assert_eq!(n.valor_cobranca.status(), StatusDado::Ok);
        assert_eq!(n.valor_cobranca.fonte(), "nfe:cobr/fat/vLiq");
    }

    #[test]
    fn arquivo_origem_preenchido() {
        assert!(!exemplo().nfe.arquivo_origem.is_empty());
    }

    // --- Contratti nuovi ----------------------------------------------------

    #[test]
    fn chave_sem_prefixo_nfe() {
        let n = exemplo().nfe;
        assert_eq!(n.chave, "29260512345678000195550010001256612345678901");
        assert!(!n.chave.starts_with("NFe"));
        assert_eq!(n.chave.len(), 44, "a chave da NF-e tem 44 digitos");
    }

    #[test]
    fn data_de_emissao_ignora_hora_e_fuso() {
        assert_eq!(
            exemplo().nfe.data_emissao,
            chrono::NaiveDate::from_ymd_opt(2026, 5, 5).unwrap()
        );
    }

    /// A NF-e de exemplo e' integra: nenhuma suposicao foi necessaria.
    #[test]
    fn exemplo_nao_gera_avisos() {
        let avisos = exemplo().avisos;
        assert!(avisos.is_empty(), "avisos inesperados: {avisos:#?}");
    }

    /// Contratto C10: un totale assente resta zero, ma lo zero e' dichiarato.
    #[test]
    fn total_obrigatorio_ausente_e_erro() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05T10:00:00-03:00</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>10.00</vUnCom><vProd>10.00</vProd></prod></det>
<total><ICMSTot><vNF>10.00</vNF><vProd>10.00</vProd></ICMSTot></total>
</infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://parcial").unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::ValorObrigatorioAusente { caminho, .. } if caminho.contains("vFrete")),
            "{erro}"
        );
    }

    #[test]
    /// Il blocco dei totali e' obbligatorio: senza, la NF-e varrebbe zero reais.
    fn bloco_de_totais_ausente_e_erro() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>10.00</vUnCom><vProd>10.00</vProd></prod></det>
</infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://sem-totais").unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::CampoObrigatorio { campo, .. } if *campo == "total"),
            "{erro}"
        );
    }

    /// Regressione P1: le entita' XML arrivano come evento separato. Senza
    /// trattarle, "1&#46;25" diventava "125" e nessuno se ne accorgeva.
    #[test]
    fn referencias_de_caractere_sao_resolvidas() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>CASA &amp; CIA</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>CAF&#201;</xProd><qCom>1&#46;25</qCom><vUnCom>2&#x2E;00</vUnCom><vProd>2.50</vProd></prod></det>
<total><ICMSTot><vNF>2.50</vNF><vProd>2.50</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://entidades").unwrap();
        assert_eq!(lida.nfe.nome_emitente, "CASA & CIA");
        let item = &lida.nfe.produtos[0];
        assert_eq!(item.descricao, "CAFÉ");
        assert_eq!(item.quantidade, Decimal::from_str_exact("1.25").unwrap());
        assert_eq!(
            item.valor_unitario,
            Decimal::from_str_exact("2.00").unwrap()
        );
        assert!(lida.avisos.is_empty(), "{:?}", lida.avisos);
    }

    /// Regressione P1: il contenuto CDATA non puo' sparire.
    #[test]
    fn cdata_entra_no_texto() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome><![CDATA[BAR & RESTAURANTE]]></xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd><![CDATA[COCA-COLA]]></xProd><qCom>1</qCom><vUnCom>1.00</vUnCom><vProd>1.00</vProd></prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://cdata").unwrap();
        assert_eq!(lida.nfe.nome_emitente, "BAR & RESTAURANTE");
        assert_eq!(lida.nfe.produtos[0].descricao, "COCA-COLA");
    }

    /// Regressione P1: un valore obbligatorio assente non diventa zero.
    #[test]
    fn valor_obrigatorio_ausente_nao_vira_zero() {
        let modelo = |prod: &str| {
            format!(
                r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod>{prod}</prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#
            )
        };

        // qCom ausente
        let erro = ler_nfe_de_texto(
            &modelo("<xProd>P</xProd><vUnCom>1.00</vUnCom><vProd>1.00</vProd>"),
            "teste://sem-qcom",
        )
        .unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::ValorObrigatorioAusente { caminho, .. } if caminho.contains("qCom")),
            "{erro}"
        );

        // vUnCom ausente
        let erro = ler_nfe_de_texto(
            &modelo("<xProd>P</xProd><qCom>1</qCom><vProd>1.00</vProd>"),
            "teste://sem-vuncom",
        )
        .unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::ValorObrigatorioAusente { caminho, .. } if caminho.contains("vUnCom")),
            "{erro}"
        );

        // vProd ausente
        let erro = ler_nfe_de_texto(
            &modelo("<xProd>P</xProd><qCom>1</qCom><vUnCom>1.00</vUnCom>"),
            "teste://sem-vprod",
        )
        .unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::ValorObrigatorioAusente { caminho, .. } if caminho.contains("vProd")),
            "{erro}"
        );
    }

    /// Regressione P1: anche la rata senza valore e' un errore, non zero.
    #[test]
    fn duplicata_sem_valor_nao_vira_zero() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>1.00</vUnCom><vProd>1.00</vProd></prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
<cobr><dup><nDup>001</nDup><dVenc>2026-06-05</dVenc></dup></cobr>
</infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://dup-sem-valor").unwrap_err();
        assert!(
            matches!(&erro, ErroNfe::ValorObrigatorioAusente { caminho, .. } if caminho.contains("vDup")),
            "{erro}"
        );
    }

    /// Un valore facoltativo assente **non** e' zero: e' un'assenza dichiarata,
    /// e un aggregato a cui manca un componente esce `Parcial`.
    #[test]
    fn valor_facultativo_ausente_nao_vira_zero() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>1.00</vUnCom><vProd>1.00</vProd></prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vII>0</vII><vIPI>0</vIPI></ICMSTot></total>
<cobr><fat><nFat>1</nFat></fat></cobr>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://opcionais").unwrap();
        // <fat> senza <vLiq>: assenza, non zero.
        assert_eq!(lida.nfe.valor_cobranca.valor(), None);
        assert_eq!(lida.nfe.valor_cobranca.status(), StatusDado::Mancante);
        assert!(lida.nfe.valor_cobranca.motivo().is_some());

        // L'aggregato esiste ma non e' completo: deve dirlo.
        let tributos = &lida.nfe.valor_tributos_inclusos;
        assert_eq!(tributos.status(), StatusDado::Parcial);
        assert_eq!(tributos.valor(), Some(&Decimal::ZERO));
        let motivo = tributos.motivo().unwrap_or_default();
        assert!(
            motivo.contains("vFCPST") && motivo.contains("vIPIDevol"),
            "{motivo}"
        );

        let codigos: Vec<&str> = lida.avisos.iter().map(|a| a.codigo.as_str()).collect();
        assert!(codigos.contains(&"AGREGADO_PARCIAL"), "{:?}", lida.avisos);
        assert!(
            codigos.contains(&"VALOR_FATURA_AUSENTE"),
            "{:?}",
            lida.avisos
        );

        // Nell'output non puo' comparire un numero al posto dell'assenza.
        let saida = lida.nfe.valor_cobranca.to_string();
        assert!(saida.starts_with("MANCANTE"), "{saida}");
        assert!(!saida.contains('0'), "{saida}");
    }

    // -----------------------------------------------------------------
    // C10: le quattro situazioni di `cobr` sono quattro stati distinti.
    // Fixture dedicate, dichiarate: il campione canonico non viene toccato.
    // -----------------------------------------------------------------

    /// Costruisce una NF-e minima valida con il blocco `cobr` indicato.
    fn nfe_com_cobranca(cobr: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>35.00</vUnCom><vProd>35.00</vProd></prod></det>
<total><ICMSTot><vNF>35.00</vNF><vProd>35.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
{cobr}
</infNFe></NFe>"#
        )
    }

    const DUAS_RATAS: &str = "<dup><nDup>001</nDup><dVenc>2026-06-05</dVenc><vDup>17.50</vDup></dup>\
<dup><nDup>002</nDup><dVenc>2026-07-05</dVenc><vDup>17.50</vDup></dup>";

    /// `cobr` assente: non applicabile, distinto da un importo zero.
    #[test]
    fn cobranca_ausente_e_nao_aplicavel() {
        let lida = ler_nfe_de_texto(&nfe_com_cobranca(""), "teste://sem-cobr").unwrap();
        assert!(!lida.nfe.tem_cobranca);
        assert_eq!(lida.nfe.valor_cobranca.valor(), None);
        assert_eq!(lida.nfe.valor_cobranca.status(), StatusDado::NaoAplicavel);
        assert!(lida.nfe.valor_cobranca.to_string().starts_with("N.A."));
    }

    /// `cobr` presente senza `fat`, con rate: importo non disponibile e
    /// **non inventato** sommando le rate; le rate restano.
    #[test]
    fn cobranca_sem_fatura_nao_soma_as_parcelas() {
        let xml = nfe_com_cobranca(&format!("<cobr>{DUAS_RATAS}</cobr>"));
        let lida = ler_nfe_de_texto(&xml, "teste://sem-fat").unwrap();

        assert!(lida.nfe.tem_cobranca);
        assert_eq!(lida.nfe.duplicatas.len(), 2);
        assert_eq!(
            lida.nfe.valor_cobranca.valor(),
            None,
            "nao pode virar 35.00 nem 0"
        );
        assert_eq!(lida.nfe.valor_cobranca.status(), StatusDado::NaoDisponivel);
        assert!(
            lida.avisos.iter().any(|a| a.codigo == "FATURA_AUSENTE"),
            "{:?}",
            lida.avisos
        );
    }

    /// `vLiq` dichiarato a zero: zero reale, conservato con stato OK.
    #[test]
    fn cobranca_zero_declarado_continua_zero() {
        let xml = nfe_com_cobranca("<cobr><fat><nFat>1</nFat><vLiq>0.00</vLiq></fat></cobr>");
        let lida = ler_nfe_de_texto(&xml, "teste://vliq-zero").unwrap();
        assert_eq!(lida.nfe.valor_cobranca.valor(), Some(&Decimal::ZERO));
        assert_eq!(lida.nfe.valor_cobranca.status(), StatusDado::Ok);
        assert_eq!(lida.nfe.valor_cobranca.to_string(), "0.00");
        assert!(
            !lida
                .avisos
                .iter()
                .any(|a| a.codigo.starts_with("VALOR_FATURA")),
            "um zero real nao e' uma ausencia: {:?}",
            lida.avisos
        );
    }

    /// I quattro casi devono essere distinguibili fra loro: nessuno collassa
    /// sullo stesso stato o sullo stesso valore.
    #[test]
    fn os_quatro_casos_de_cobranca_sao_distinguiveis() {
        let casos = [
            ("", StatusDado::NaoAplicavel, None),
            (
                "<cobr><dup><nDup>001</nDup><dVenc>2026-06-05</dVenc><vDup>17.50</vDup></dup></cobr>",
                StatusDado::NaoDisponivel,
                None,
            ),
            (
                "<cobr><fat><nFat>1</nFat></fat></cobr>",
                StatusDado::Mancante,
                None,
            ),
            (
                "<cobr><fat><nFat>1</nFat><vLiq>35.00</vLiq></fat></cobr>",
                StatusDado::Ok,
                Some("35.00"),
            ),
        ];
        let mut estados = std::collections::BTreeSet::new();
        for (cobr, estado, valor) in casos {
            let lida = ler_nfe_de_texto(&nfe_com_cobranca(cobr), "teste://casos").unwrap();
            assert_eq!(lida.nfe.valor_cobranca.status(), estado, "caso {cobr:?}");
            assert_eq!(
                lida.nfe.valor_cobranca.valor(),
                valor.map(|v| Decimal::from_str_exact(v).unwrap()).as_ref(),
                "caso {cobr:?}"
            );
            estados.insert(estado.rotulo());
        }
        assert_eq!(
            estados.len(),
            4,
            "os quatro casos devem dar quatro estados: {estados:?}"
        );
    }

    /// Round-trip del modello con importo assente: l'assenza sopravvive e
    /// non diventa un default numerico.
    #[test]
    fn round_trip_preserva_a_ausencia_do_importe() {
        let xml = nfe_com_cobranca(&format!("<cobr>{DUAS_RATAS}</cobr>"));
        let lida = ler_nfe_de_texto(&xml, "teste://round-trip").unwrap();

        let serializado = serde_yaml::to_string(&lida.nfe).unwrap();
        assert!(serializado.contains("NAO_DISPONIVEL"), "{serializado}");

        let volta: Nfe = serde_yaml::from_str(&serializado).unwrap();
        assert_eq!(volta, lida.nfe);
        assert_eq!(volta.valor_cobranca.valor(), None);
        assert_eq!(volta.valor_cobranca.status(), StatusDado::NaoDisponivel);
    }

    #[test]
    fn xml_sem_inf_nfe_e_recusado() {
        let xml = r#"<?xml version="1.0"?>
<procEventoNFe xmlns="http://www.portalfiscal.inf.br/nfe"><evento/></procEventoNFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://evento").unwrap_err();
        assert!(matches!(erro, ErroNfe::NaoEhNfe { .. }), "{erro}");
    }

    // -----------------------------------------------------------------
    // C17: un documento, una NF-e.
    //
    // Riscontro A1 dell'audit indipendente del 2026-10-16: due XML
    // concatenati uscivano con codice 0, mostravano solo la NF-e 12566 e
    // scartavano la seconda in silenzio. Fixture derivate e dichiarate: il
    // campione canonico non viene toccato.
    // -----------------------------------------------------------------

    /// Due documenti concatenati non sono un documento XML ben formato.
    #[test]
    fn duas_raizes_sao_recusadas() {
        let primeiro = nfe_com_cobranca("<cobr><fat><nFat>1</nFat><vLiq>35.00</vLiq></fat></cobr>");
        let segundo = primeiro.replace(r#"<?xml version="1.0"?>"#, "");
        let erro =
            ler_nfe_de_texto(&format!("{primeiro}{segundo}"), "teste://duas-raizes").unwrap_err();
        assert!(
            matches!(erro, ErroNfe::DocumentoComMultiplasRaizes { .. }),
            "{erro}"
        );
    }

    /// La seconda radice conta anche se e' fuori dal namespace NF-e: e' un
    /// secondo documento, non un sottoalbero da ignorare.
    #[test]
    fn segunda_raiz_fora_do_namespace_tambem_e_recusada() {
        let xml = format!(
            r#"{}<outro xmlns="http://exemplo.invalido/outro"><a>1</a></outro>"#,
            nfe_com_cobranca("")
        );
        let erro = ler_nfe_de_texto(&xml, "teste://raiz-estranha").unwrap_err();
        assert!(
            matches!(erro, ErroNfe::DocumentoComMultiplasRaizes { .. }),
            "{erro}"
        );
    }

    /// Documento ben formato ma con due NF-e (lotto `enviNFe`): la radice
    /// unica non basta, serve anche una sola `<infNFe>`.
    #[test]
    fn lote_com_duas_nfe_e_recusado() {
        let interna = nfe_com_cobranca("").replace(r#"<?xml version="1.0"?>"#, "");
        let xml = format!(
            r#"<?xml version="1.0"?>
<enviNFe xmlns="http://www.portalfiscal.inf.br/nfe">{interna}{interna}</enviNFe>"#
        );
        let erro = ler_nfe_de_texto(&xml, "teste://lote").unwrap_err();
        assert!(
            matches!(
                erro,
                ErroNfe::MultiplasNfeNoDocumento { encontradas: 2, .. }
            ),
            "{erro}"
        );
    }

    /// `<infNFe>` annidato dentro un altro: documento ben formato, fuori
    /// layout NF-e, due occorrenze. Riscontro B1: il conteggio si fermava sul
    /// primo nodo trovato e il documento passava con la sola prima nota.
    #[test]
    fn infnfe_aninhado_e_recusado() {
        let xml = nfe_com_cobranca("").replace(
            "</infNFe>",
            r#"<infNFe Id="NFe456"><ide><nNF>2</nNF></ide></infNFe></infNFe>"#,
        );
        let erro = ler_nfe_de_texto(&xml, "teste://aninhado").unwrap_err();
        assert!(
            matches!(
                erro,
                ErroNfe::MultiplasNfeNoDocumento { encontradas: 2, .. }
            ),
            "{erro}"
        );
    }

    /// C17 non deve irrigidire il caso buono: il campione canonico ha una
    /// radice e una sola NF-e, e continua a essere letto.
    #[test]
    fn documento_canonico_tem_uma_so_nfe() {
        let xml = std::fs::read_to_string(caminho_exemplo()).expect("campione leggibile");
        let documento = construir_arvore(&xml, "teste://canonico").expect("uma raiz");
        assert_eq!(documento.contar_descendentes("infNFe"), 1);
        assert_eq!(exemplo().nfe.numero, "12566");
    }

    #[test]
    fn xml_malformado_e_recusado() {
        let erro = ler_nfe_de_texto("<NFe><infNFe>", "teste://quebrado").unwrap_err();
        assert!(
            matches!(erro, ErroNfe::Xml { .. } | ErroNfe::NaoEhNfe { .. }),
            "{erro}"
        );
    }

    /// Elementi fuori dal namespace NF-e non devono confondere la lettura:
    /// un `<infNFe>` di un altro namespace non e' una NF-e.
    #[test]
    fn namespace_errado_nao_e_nfe() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://exemplo.invalido/outro"><infNFe Id="NFe123"><ide><nNF>1</nNF></ide></infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://ns-errado").unwrap_err();
        assert!(matches!(erro, ErroNfe::NaoEhNfe { .. }), "{erro}");
    }

    /// La firma digitale e' in un altro namespace e non deve entrare nei dati.
    #[test]
    fn assinatura_em_outro_namespace_e_ignorada() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe">
<infNFe Id="NFe123">
<ide><nNF>99</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>10.00</vUnCom><vProd>10.00</vProd></prod></det>
<total><ICMSTot><vNF>10.00</vNF><vProd>10.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg><vDesc>0.00</vDesc><vOutro>0.00</vOutro><vST>0.00</vST><vFCPST>0.00</vFCPST><vII>0.00</vII><vIPI>0.00</vIPI><vIPIDevol>0.00</vIPIDevol></ICMSTot></total>
</infNFe>
<Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><nNF>FALSO</nNF></Signature>
</NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://assinada").unwrap();
        assert_eq!(lida.nfe.numero, "99");
        assert!(lida.avisos.is_empty(), "{:?}", lida.avisos);
    }

    #[test]
    fn det_sem_prod_e_declarado() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><imposto/></det>
<total><ICMSTot><vNF>0.00</vNF><vProd>0.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://det-vazio").unwrap();
        assert!(lida.nfe.produtos.is_empty());
        assert!(
            lida.avisos.iter().any(|a| a.codigo == "DET_SEM_PROD"),
            "{:?}",
            lida.avisos
        );
        assert!(
            lida.avisos.iter().any(|a| a.codigo == "SEM_PRODUTOS"),
            "{:?}",
            lida.avisos
        );
    }

    #[test]
    fn valor_nao_numerico_e_recusado() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>1.00</vUnCom><vProd>dez reais</vProd></prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://valor-ruim").unwrap_err();
        assert!(
            matches!(erro, ErroNfe::ValorNumericoInvalido { .. }),
            "{erro}"
        );
    }

    #[test]
    fn data_invalida_e_recusada() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>05/05/2026</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
</infNFe></NFe>"#;
        let erro = ler_nfe_de_texto(xml, "teste://data-ruim").unwrap_err();
        assert!(matches!(erro, ErroNfe::DataInvalida { .. }), "{erro}");
    }

    /// CPF al posto di CNPJ nel destinatario (persona fisica).
    #[test]
    fn destinatario_com_cpf() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CPF>12345678901</CPF><xNome>Pessoa</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>1.00</vUnCom><vProd>1.00</vProd></prod></det>
<total><ICMSTot><vNF>1.00</vNF><vProd>1.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://cpf").unwrap();
        assert_eq!(lida.nfe.cnpj_destinatario, "12345678901");
        assert!(lida.avisos.is_empty(), "{:?}", lida.avisos);
    }

    /// La somma degli item deve tornare con vProd: se non torna, va detto.
    #[test]
    fn soma_dos_itens_divergente_e_declarada() {
        let xml = r#"<?xml version="1.0"?>
<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe123">
<ide><nNF>1</nNF><serie>1</serie><dhEmi>2026-05-05</dhEmi><natOp>X</natOp></ide>
<emit><CNPJ>1</CNPJ><xNome>E</xNome></emit>
<dest><CNPJ>2</CNPJ><xNome>D</xNome></dest>
<det nItem="1"><prod><xProd>P</xProd><qCom>1</qCom><vUnCom>10.00</vUnCom><vProd>10.00</vProd></prod></det>
<total><ICMSTot><vNF>99.00</vNF><vProd>99.00</vProd><vFrete>0</vFrete><vSeg>0</vSeg><vDesc>0</vDesc><vOutro>0</vOutro><vST>0</vST><vFCPST>0</vFCPST><vII>0</vII><vIPI>0</vIPI><vIPIDevol>0</vIPIDevol></ICMSTot></total>
</infNFe></NFe>"#;
        let lida = ler_nfe_de_texto(xml, "teste://soma").unwrap();
        assert!(
            lida.avisos.iter().any(|a| a.codigo == "SOMA_ITENS_DIVERGE"),
            "{:?}",
            lida.avisos
        );
    }
}

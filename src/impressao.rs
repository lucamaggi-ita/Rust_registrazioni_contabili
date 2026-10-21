//! Impronte canoniche del contenuto allocato e della sua presentazione
//! (fase 3C.3.a).
//!
//! Attua i contratti F3C-34, F3C-37, F3C-38 e F3C-43 della specifica di fase
//! 3C.3, con le decisioni D8.2, D8.3 e D8.4 approvate il 2026-10-21.
//!
//! # A che serve
//!
//! `IdentidadeLinha` distingue le righe **dentro un lotto**, ma non dice
//! *quale contenuto* di un documento una persona ha esaminato. L'impronta di
//! contenuto e' quella identita': una stringa di 64 caratteri esadecimali,
//! calcolata da un testo canonico dichiarato, che cambia se e solo se cambia
//! il contenuto allocato.
//!
//! Due impronte separate, decisione D8.3:
//!
//! - [`impressao_conteudo`]: il documento e le sue registrazioni. **Non**
//!   include il piano dei conti, il percorso del file, l'identita' del lotto
//!   ne' i campi umani.
//! - [`impressao_apresentacao`]: l'impronta di contenuto **piu'** i nomi di
//!   conto che l'utente vede, presi dal piano. Un rinomino nel piano cambia
//!   solo questa, quindi segnala il cambiamento invece di invalidare le
//!   approvazioni.
//!
//! # Principi del testo canonico (F3C-34)
//!
//! 1. **Prefisso di lunghezza su ogni stringa**, in byte UTF-8 calcolati
//!    **dopo** la normalizzazione: `TAG=<byte>:<contenuto>`. Nessun escaping e
//!    nessun separatore vietato nei dati. Senza il prefisso, due insiemi di
//!    campi diversi uniti da un separatore possono dare lo stesso testo.
//! 2. **Token chiusi senza prefisso** per enum, date e interi.
//! 3. Una riga per campo, terminata da `LF`, **`LF` finale compreso**. Mai
//!    `CRLF`: il testo e' un flusso di byte, non un file di piattaforma.
//! 4. **Conteggio esplicito** prima di ogni collezione, `TAG#<n>`, cosi'
//!    "nessun elemento" non si confonde con "campo assente".
//! 5. SHA-256 del testo, esadecimale minuscolo.
//!
//! # Normalizzazione (D8.4, F3C-43)
//!
//! Le stringhe entrano in forma **NFC**, e le chiavi di ordinamento vengono
//! normalizzate **prima** del confronto, che e' ordinale sui byte UTF-8. Senza
//! quest'ordine di operazioni due documenti con contenuto canonicamente
//! identico otterrebbero impronte diverse solo per la forma Unicode degli
//! avvisi. Le stringhe memorizzate nel modello **non** vengono modificate.
//!
//! # Limiti dichiarati
//!
//! L'impronta **non** e' invariante alla modifica del **testo** di un avviso
//! nel codice, ne' a un cambio di regole che modifichi `conta_id`,
//! `conta_nome` o `grupo_conta` di una riga: sono contenuti diversi, ed e'
//! voluto. Un cambio dell'insieme dei campi o della codifica richiede
//! `IMPRESSAO-CONTEUDO/v2` e la conservazione delle impronte `v1`.

use std::fmt::Write as _;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::alocacao::{
    Componente, EsitoComponenti, EsitoCopertura, EsitoRate, EsitoTotal, Reconciliacao,
    RuoloComponente, StatoDocumento,
};
use crate::modelos::TipoConta;
use crate::modelos::{Dado, Duplicata, Lancamento, OrigemLancamento, StatusRevisao};
use crate::nfe::Aviso;
use crate::prelivro::ContextoDocumento;
use crate::regras::PlanoContas;

/// Versione dello schema dell'impronta di contenuto. Entra nei byte: un cambio
/// di grammatica cambia tutte le impronte, e per questo richiede un `v2`
/// esplicito invece di una migrazione silenziosa.
pub const VERSAO_CONTEUDO: &str = "IMPRESSAO-CONTEUDO/v1";

/// Versione dello schema dell'impronta di presentazione.
pub const VERSAO_APRESENTACAO: &str = "IMPRESSAO-APRESENTACAO/v1";

/// Un `conta_id` della riga che il piano ricevuto non contiene.
///
/// In pratica non si raggiunge passando per [`crate::prelivro::construir_pre_livro`],
/// che rifiuta lo stesso ingresso con `ContaNaoEncontradaNoPlano`: questo tipo
/// esiste per non avere un `unwrap` su un dato esterno.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContaAusenteNoPlano {
    pub chave_origem: String,
    pub conta_id: String,
}

/// Conto come l'utente lo vede: identificativo e nome dal **piano**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContaMostrada {
    pub conta_id: String,
    pub nome: String,
    pub grupo: String,
}

// ---------------------------------------------------------------------------
// Primitive della grammatica
// ---------------------------------------------------------------------------

/// Forma NFC di un testo. Non modifica il dato memorizzato: serve solo a
/// calcolare impronte e chiavi di ordinamento (D8.4).
pub(crate) fn nfc(texto: &str) -> String {
    texto.nfc().collect()
}

/// `str` della grammatica: lunghezza in byte **dopo** NFC, poi il testo.
fn str_com_prefixo(texto: &str) -> String {
    let normalizado = nfc(texto);
    format!("{}:{}", normalizado.len(), normalizado)
}

fn campo(saida: &mut String, tag: &str, texto: &str) {
    let _ = writeln!(saida, "{tag}={}", str_com_prefixo(texto));
}

fn token(saida: &mut String, tag: &str, valor: &str) {
    let _ = writeln!(saida, "{tag}={valor}");
}

fn contagem(saida: &mut String, tag: &str, n: usize) {
    let _ = writeln!(saida, "{tag}#{n}");
}

/// `dec` della grammatica: mantissa e scala della forma normalizzata.
///
/// `10`, `10.00` e `10.0000` danno `DEC:10:0`. Lo zero e' sempre `DEC:0:0`,
/// cosi' `-0.00` e `0.00` non producono due impronte per lo stesso denaro.
/// Nessuna conversione in centesimi: `0.005` e una quantita' a quattro
/// decimali non sono nel dominio dei centesimi, e convertirle sarebbe un
/// errore. Nessun passaggio in virgola mobile.
pub(crate) fn dec(valor: Decimal) -> String {
    let normalizado = valor.normalize();
    let mantissa = normalizado.mantissa();
    if mantissa == 0 {
        return "DEC:0:0".to_string();
    }
    format!("DEC:{}:{}", mantissa, normalizado.scale())
}

fn data(valor: NaiveDate) -> String {
    valor.format("%Y-%m-%d").to_string()
}

/// `dado` della grammatica **tecnica**: stato, fonte e valore.
///
/// Il `motivo` e' **escluso** per scelta: e' prosa del codice, e una sua
/// riscrittura non deve cambiare le impronte del contenuto contabile. Questa
/// produzione **non** e' un criterio di identita': per gli eventi di revisione
/// esiste una codifica esatta separata in [`crate::revisao`].
pub(crate) fn dado<T>(valor: &Dado<T>, formatar: impl Fn(&T) -> String) -> String {
    let conteudo = match valor.valor() {
        Some(v) => formatar(v),
        None => "-".to_string(),
    };
    format!(
        "DADO:{}:{}:{}",
        valor.status().rotulo(),
        str_com_prefixo(valor.fonte()),
        conteudo
    )
}

// ---------------------------------------------------------------------------
// Etichette degli enum
// ---------------------------------------------------------------------------
//
// Sono fissate qui perche' fanno parte della grammatica dell'impronta: devono
// restare stabili anche se un giorno cambiasse la rappresentazione serde dei
// tipi. I test `etiquetas_*` le fissano una per una.

fn origem(valor: OrigemLancamento) -> &'static str {
    match valor {
        OrigemLancamento::XmlNfe => "XML_NFE",
        OrigemLancamento::Csv => "CSV",
        OrigemLancamento::Manual => "MANUAL",
    }
}

fn tipo_conta(valor: TipoConta) -> &'static str {
    match valor {
        TipoConta::Receita => "RECEITA",
        TipoConta::DeducaoReceita => "DEDUCAO_RECEITA",
        TipoConta::Custo => "CUSTO",
        TipoConta::Despesa => "DESPESA",
        TipoConta::Imposto => "IMPOSTO",
        TipoConta::Investimento => "INVESTIMENTO",
        TipoConta::Passivo => "PASSIVO",
        TipoConta::Patrimonial => "PATRIMONIAL",
        TipoConta::Provisao => "PROVISAO",
        TipoConta::Ajuste => "AJUSTE",
        TipoConta::Revisao => "REVISAO",
    }
}

pub(crate) fn status_revisao(valor: StatusRevisao) -> &'static str {
    match valor {
        StatusRevisao::Ok => "OK",
        StatusRevisao::Revisar => "REVISAR",
        StatusRevisao::Aprovado => "APROVADO",
    }
}

fn esito_total(valor: EsitoTotal) -> &'static str {
    match valor {
        EsitoTotal::Ok => "OK",
        EsitoTotal::Erro => "ERRO",
    }
}

fn esito_rate(valor: EsitoRate) -> &'static str {
    match valor {
        EsitoRate::Ok => "OK",
        EsitoRate::Erro => "ERRO",
        EsitoRate::NaoAplicavel => "N.A.",
        EsitoRate::NaoVerificado => "NAO_VERIFICADO",
    }
}

fn esito_copertura(valor: EsitoCopertura) -> &'static str {
    match valor {
        EsitoCopertura::Ok => "OK",
        EsitoCopertura::Parcial => "PARCIAL",
    }
}

fn esito_componenti(valor: EsitoComponenti) -> &'static str {
    match valor {
        EsitoComponenti::Ok => "OK",
        EsitoComponenti::Parcial => "PARCIAL",
    }
}

fn stato_documento(valor: StatoDocumento) -> &'static str {
    match valor {
        StatoDocumento::Ok => "OK",
        StatoDocumento::Parcial => "PARCIAL",
        StatoDocumento::Erro => "ERRO",
    }
}

fn ruolo_componente(valor: RuoloComponente) -> &'static str {
    match valor {
        RuoloComponente::Base => "BASE",
        RuoloComponente::Acessorio => "ACESSORIO",
        RuoloComponente::Deducao => "DEDUCAO",
        RuoloComponente::Informativa => "INFORMATIVA",
    }
}

// ---------------------------------------------------------------------------
// Impronta di contenuto
// ---------------------------------------------------------------------------

/// Testo canonico del contenuto di un documento allocato.
///
/// Esposto, e non solo l'impronta, perche' un test deve poter confrontare i
/// **byte**: un hash che coincide per caso non dimostra niente, e un hash che
/// differisce non dice dove.
///
/// Non puo' fallire: ogni campo ha una codifica totale.
pub fn texto_canonico(contexto: &ContextoDocumento) -> String {
    let mut saida = String::with_capacity(1024);
    token(&mut saida, "V", VERSAO_CONTEUDO);
    campo(&mut saida, "DCH", &contexto.chave_origem);
    campo(&mut saida, "DNU", &contexto.numero);
    campo(&mut saida, "DSE", &contexto.serie);
    token(&mut saida, "DDT", &data(contexto.data_documento));
    token(&mut saida, "DCP", &contexto.competencia.to_string());
    campo(&mut saida, "DEM", &contexto.cnpj_empresa);
    campo(&mut saida, "DNE", &contexto.nome_empresa);

    // F3C-43: gli avvisi di lettura arrivano dal chiamante, che potrebbe
    // permutarli. Ordine canonico con chiavi normalizzate prima del confronto.
    escrever_avisos(&mut saida, "AVL", &ordenar_avisos(&contexto.avisos_leitura));

    contagem(&mut saida, "LAN", contexto.alocacao.lancamentos.len());
    for (ordinal, lancamento) in contexto.alocacao.lancamentos.iter().enumerate() {
        escrever_linha(&mut saida, ordinal as u32, lancamento);
    }

    escrever_reconciliacao(&mut saida, &contexto.alocacao.reconciliacao);
    saida
}

/// Impronta di contenuto: SHA-256 esadecimale minuscolo di [`texto_canonico`].
pub fn impressao_conteudo(contexto: &ContextoDocumento) -> String {
    sha256_hex(texto_canonico(contexto).as_bytes())
}

/// Le registrazioni compaiono in ordine di `ordinal`, che e' l'ordine
/// contrattuale C18 dell'allocazione: non vengono riordinate per comodita'.
///
/// I cinque campi che `verificar_coerencia` confronta fra contesto e riga
/// (`chave_origem`, `documento`, `data_documento`, `competencia`,
/// `cnpj_empresa`) sono scritti **una volta** a livello di documento: e' quel
/// controllo a rendere sicura l'omissione. Esclusi `arquivo_origem` (dato
/// d'ambiente), `hash_registro` (derivato) e `observacoes` (campo umano).
fn escrever_linha(saida: &mut String, ordinal: u32, lancamento: &Lancamento) {
    token(saida, "LOR", &ordinal.to_string());
    token(saida, "LOG", origem(lancamento.origem));
    campo(saida, "LCI", &lancamento.conta_id);
    campo(saida, "LCN", &lancamento.conta_nome);
    campo(saida, "LGC", &lancamento.grupo_conta);
    campo(saida, "LDS", &lancamento.descricao_itens);
    token(saida, "LVL", &dec(lancamento.valor));
    campo(saida, "LNE", &lancamento.nome_empresa);
    campo(saida, "LFV", &lancamento.fornecedor_ou_favorecido);
    campo(saida, "LCF", &lancamento.cnpj_cpf_favorecido);
    campo(saida, "LTD", &lancamento.tipo_documento);
    token(saida, "LTL", tipo_conta(lancamento.tipo_lancamento));
    campo(saida, "LDRI", &lancamento.dre_linha_id);
    campo(saida, "LDRN", &lancamento.dre_linha_nome);
    campo(saida, "LCC", &lancamento.centro_custo);
    token(saida, "LSR", status_revisao(lancamento.status_revisao));
    campo(saida, "LAL", &lancamento.alertas);
    token(
        saida,
        "LPG",
        &dado(&lancamento.data_pagamento, |d| data(*d)),
    );

    // Le rate restano nell'ordine memorizzato: il riparto mette i resti
    // nell'ultima (F3A-08/09), quindi l'ordine e' informazione.
    contagem(saida, "LDUP", lancamento.duplicatas.len());
    for duplicata in &lancamento.duplicatas {
        escrever_duplicata(saida, duplicata);
    }
}

fn escrever_duplicata(saida: &mut String, duplicata: &Duplicata) {
    campo(saida, "DN", &duplicata.numero);
    match duplicata.vencimento {
        Some(v) => token(saida, "DV", &data(v)),
        None => token(saida, "DV", "AUSENTE"),
    }
    token(saida, "DVL", &dec(duplicata.valor));
}

/// La riconciliazione entra **integra**: quattro esiti, componenti, campi
/// rinviati e avvisi. Una correzione che non toccasse i totali resterebbe
/// altrimenti invisibile all'impronta.
fn escrever_reconciliacao(saida: &mut String, reconciliacao: &Reconciliacao) {
    let _ = writeln!(saida, "REC");
    token(saida, "RVT", &dec(reconciliacao.valor_total_nfe));
    token(saida, "RSL", &dec(reconciliacao.soma_lancamentos));
    token(saida, "RET", esito_total(reconciliacao.esito_total));
    token(
        saida,
        "RRD",
        &dado(&reconciliacao.soma_rate_documento, |v| dec(*v)),
    );
    token(
        saida,
        "RRA",
        &dado(&reconciliacao.soma_rate_allocate, |v| dec(*v)),
    );
    token(saida, "RER", esito_rate(reconciliacao.esito_rate));
    token(saida, "RAA", &reconciliacao.articoli_allocati.to_string());
    token(
        saida,
        "RAT",
        &dado(&reconciliacao.articoli_attesi, |v| v.to_string()),
    );
    // Prodotti dalla fase 2 e dalla 3B in sequenza deterministica per un dato
    // documento: ordine memorizzato.
    escrever_avisos_diretos(saida, "RSC", &reconciliacao.articoli_scartati);
    token(saida, "RCO", esito_copertura(reconciliacao.esito_copertura));
    contagem(saida, "RCM", reconciliacao.componenti.len());
    for componente in &reconciliacao.componenti {
        escrever_componente(saida, componente);
    }
    token(
        saida,
        "RCE",
        esito_componenti(reconciliacao.esito_componenti),
    );
    // Dei campi rinviati entra **solo il nome**: la prosa di `status`, `fonte`
    // e `motivo` non e' contenuto del documento.
    contagem(saida, "RRN", reconciliacao.campi_rinviati.len());
    for rinviato in &reconciliacao.campi_rinviati {
        campo(saida, "FN", &rinviato.campo);
    }
    escrever_avisos_diretos(saida, "RAD", &reconciliacao.avisos_documento);
    token(saida, "RSD", stato_documento(reconciliacao.stato_documento));
}

fn escrever_componente(saida: &mut String, componente: &Componente) {
    campo(saida, "MN", &componente.nome);
    token(saida, "MV", &dado(&componente.valor, |v| dec(*v)));
    token(saida, "MR", ruolo_componente(componente.ruolo));
    token(
        saida,
        "MU",
        if componente.usada_na_verificacao {
            "1"
        } else {
            "0"
        },
    );
}

fn escrever_avisos(saida: &mut String, tag: &str, avisos: &[&Aviso]) {
    contagem(saida, tag, avisos.len());
    for aviso in avisos {
        campo(saida, "AC", &aviso.codigo);
        campo(saida, "AD", &aviso.detalhe);
    }
}

fn escrever_avisos_diretos(saida: &mut String, tag: &str, avisos: &[Aviso]) {
    contagem(saida, tag, avisos.len());
    for aviso in avisos {
        campo(saida, "AC", &aviso.codigo);
        campo(saida, "AD", &aviso.detalhe);
    }
}

/// Ordine canonico degli avvisi (F3C-43): **prima** si normalizzano le chiavi,
/// **poi** si confrontano i byte UTF-8.
///
/// Due avvisi con chiavi NFC uguali ma byte memorizzati diversi restano
/// **entrambi** presenti: nessuno viene scartato o fuso. Il pareggio e' innocuo
/// perche' dopo la normalizzazione i due emettono gli stessi byte, quindi
/// qualunque ordine relativo produce lo stesso testo canonico.
fn ordenar_avisos(avisos: &[Aviso]) -> Vec<&Aviso> {
    let mut com_chave: Vec<(String, String, &Aviso)> = avisos
        .iter()
        .map(|a| (nfc(&a.codigo), nfc(&a.detalhe), a))
        .collect();
    com_chave.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    com_chave.into_iter().map(|(_, _, a)| a).collect()
}

// ---------------------------------------------------------------------------
// Impronta di presentazione
// ---------------------------------------------------------------------------

/// Conti che le righe del documento mostrano, con i nomi del **piano**.
///
/// Ordinati per `conta_id` con la regola canonica di F3C-43. Gli
/// identificativi osservati sono ASCII, quindi oggi la normalizzazione non
/// cambia nulla: la regola e' dichiarata comunque, perche' un contratto che
/// vale "solo finche' i dati restano ASCII" non e' un contratto.
pub fn contas_mostradas(
    contexto: &ContextoDocumento,
    plano: &PlanoContas,
) -> Result<Vec<ContaMostrada>, ContaAusenteNoPlano> {
    let mut vistas: Vec<ContaMostrada> = Vec::new();
    for lancamento in &contexto.alocacao.lancamentos {
        if vistas.iter().any(|c| c.conta_id == lancamento.conta_id) {
            continue;
        }
        let conta = plano
            .por_id(&lancamento.conta_id)
            .ok_or_else(|| ContaAusenteNoPlano {
                chave_origem: contexto.chave_origem.clone(),
                conta_id: lancamento.conta_id.clone(),
            })?;
        vistas.push(ContaMostrada {
            conta_id: conta.id.clone(),
            nome: conta.nome.clone(),
            grupo: conta.grupo.clone(),
        });
    }
    vistas.sort_by_key(|a| nfc(&a.conta_id));
    Ok(vistas)
}

/// Testo canonico della presentazione: l'impronta di contenuto piu' i nomi
/// mostrati.
pub fn texto_apresentacao(impressao_conteudo: &str, contas: &[ContaMostrada]) -> String {
    let mut saida = String::with_capacity(256);
    token(&mut saida, "V", VERSAO_APRESENTACAO);
    token(&mut saida, "CT", impressao_conteudo);
    contagem(&mut saida, "PC", contas.len());
    for conta in contas {
        campo(&mut saida, "PI", &conta.conta_id);
        campo(&mut saida, "PN", &conta.nome);
        campo(&mut saida, "PG", &conta.grupo);
    }
    saida
}

/// Impronta di presentazione (D8.3).
///
/// Cambia quando cambia cio' che l'utente **vede**, non cio' che e' stato
/// allocato: un rinomino nel piano senza riallocazione produce un'impronta di
/// presentazione diversa e un'impronta di contenuto **identica**.
pub fn impressao_apresentacao(impressao_conteudo: &str, contas: &[ContaMostrada]) -> String {
    sha256_hex(texto_apresentacao(impressao_conteudo, contas).as_bytes())
}

/// SHA-256 in esadecimale minuscolo, come il contratto C23 della fase 3B.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alocacao::{AlocacaoNfe, CampoRinviato};
    use crate::modelos::{Competencia, StatusDado, TipoConta};
    use crate::prelivro::IdentidadeLote;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // -----------------------------------------------------------------
    // Vettori normativi di §4.8 della specifica.
    //
    // I valori attesi sono scritti a mano, copiati dalla specifica
    // pubblicata, e **non** rigenerati dal codice sotto test: un confronto
    // del codice con se stesso non dimostrerebbe nulla.
    // -----------------------------------------------------------------

    const VETOR_I_SHA: &str = "f8a6e8d0aade811e39e31f966c17e0c6623f21afdbde73a8c5866641031f614c";
    const VETOR_J_SHA: &str = "4ce5f31b07e4478dbb70e73cf03639a384be8e27aa802c7236de258c30f997a7";
    const VETOR_K_SHA: &str = "44195583d43090941ecc7f45c90636e859382a431928b8011941bc26c784b324";
    const VETOR_L_SHA: &str = "d1a8e9834a9a80efefb1430f28c49b53c5e1dac5a9f30c6a8996e4ca26566424";

    /// Il testo del vettore I, byte per byte come pubblicato nella specifica.
    ///
    /// Scritto come letterale con `\n` espliciti: un `CRLF` introdotto dal
    /// checkout non puo' alterarlo, e il blocco non dipende da Git a runtime.
    const VETOR_I_TEXTO: &str = concat!(
        "V=IMPRESSAO-CONTEUDO/v1\n",
        "DCH=44:29260512345678000195550010001256612345678901\n",
        "DNU=5:12566\n",
        "DSE=1:1\n",
        "DDT=2026-05-05\n",
        "DCP=2026-05\n",
        "DEM=14:55746722000181\n",
        "DNE=21:RESTAURANTE OESA LTDA\n",
        "AVL#0\n",
        "LAN#1\n",
        "LOR=0\n",
        "LOG=XML_NFE\n",
        "LCI=8:2.01.004\n",
        "LCN=13:Refrigerantes\n",
        "LGC=31:Custos de Mercadorias e Insumos\n",
        "LDS=26:COCA-COLA KS 290ML (10.00)\n",
        "LVL=DEC:10:0\n",
        "LNE=21:RESTAURANTE OESA LTDA\n",
        "LFV=23:DISTRIBUIDORA OESA LTDA\n",
        "LCF=14:12345678000195\n",
        "LTD=3:NFE\n",
        "LTL=CUSTO\n",
        "LDRI=0:\n",
        "LDRN=0:\n",
        "LCC=0:\n",
        "LSR=OK\n",
        "LAL=0:\n",
        "LPG=DADO:NAO_DISPONIVEL:18:nfe:cobr/dup/dVenc:-\n",
        "LDUP#0\n",
        "REC\n",
        "RVT=DEC:10:0\n",
        "RSL=DEC:10:0\n",
        "RET=OK\n",
        "RRD=DADO:N.A.:17:nfe:cobr/dup/vDup:-\n",
        "RRA=DADO:N.A.:17:nfe:cobr/dup/vDup:-\n",
        "RER=N.A.\n",
        "RAA=1\n",
        "RAT=DADO:NAO_DISPONIVEL:7:nfe:det:-\n",
        "RSC#0\n",
        "RCO=OK\n",
        "RCM#2\n",
        "MN=5:vProd\n",
        "MV=DADO:OK:23:nfe:total/ICMSTot/vProd:DEC:10:0\n",
        "MR=BASE\n",
        "MU=1\n",
        "MN=5:vDesc\n",
        "MV=DADO:OK:23:nfe:total/ICMSTot/vDesc:DEC:0:0\n",
        "MR=DEDUCAO\n",
        "MU=1\n",
        "RCE=OK\n",
        "RRN#4\n",
        "FN=12:centro_custo\n",
        "FN=12:dre_linha_id\n",
        "FN=14:dre_linha_nome\n",
        "FN=11:observacoes\n",
        "RAD#0\n",
        "RSD=OK\n",
    );

    pub(crate) fn dec_de(bruto: &str) -> Decimal {
        Decimal::from_str(bruto).unwrap()
    }

    fn rinviato(campo: &str) -> CampoRinviato {
        CampoRinviato {
            campo: campo.to_string(),
            status: "NAO_DISPONIVEL".to_string(),
            fonte: "fase 3B".to_string(),
            motivo: "campo non popolato in questo blocco".to_string(),
        }
    }

    /// Documento sintetico ridotto che riproduce il vettore I.
    ///
    /// Costruito a mano, come dichiara la specifica: **non** e' l'uscita di
    /// `alocar_nfe` sul campione canonico.
    pub(crate) fn documento_vetor_i(descricao: &str) -> ContextoDocumento {
        let lancamento = Lancamento {
            data_documento: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: "55746722000181".to_string(),
            nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
            fornecedor_ou_favorecido: "DISTRIBUIDORA OESA LTDA".to_string(),
            cnpj_cpf_favorecido: "12345678000195".to_string(),
            documento: "12566".to_string(),
            tipo_documento: "NFE".to_string(),
            conta_id: "2.01.004".to_string(),
            conta_nome: "Refrigerantes".to_string(),
            grupo_conta: "Custos de Mercadorias e Insumos".to_string(),
            descricao_itens: descricao.to_string(),
            valor: dec_de("10.00"),
            duplicatas: Vec::new(),
            origem: OrigemLancamento::XmlNfe,
            // Esclusi dall'impronta: qui valorizzati proprio per dimostrarlo.
            arquivo_origem: "C:/qualunque/percorso/nfe.xml".to_string(),
            chave_origem: "29260512345678000195550010001256612345678901".to_string(),
            hash_registro: "non entra nell'impronta".to_string(),
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
        let reconciliacao = Reconciliacao {
            valor_total_nfe: dec_de("10.00"),
            soma_lancamentos: dec_de("10.00"),
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
                "a fase 2 nao expoe o numero de <det> lidos",
            )
            .unwrap(),
            articoli_scartati: Vec::new(),
            esito_copertura: EsitoCopertura::Ok,
            componenti: vec![
                Componente {
                    nome: "vProd".to_string(),
                    valor: Dado::presente(dec_de("10.00"), "nfe:total/ICMSTot/vProd").unwrap(),
                    ruolo: RuoloComponente::Base,
                    usada_na_verificacao: true,
                },
                Componente {
                    nome: "vDesc".to_string(),
                    valor: Dado::presente(Decimal::ZERO, "nfe:total/ICMSTot/vDesc").unwrap(),
                    ruolo: RuoloComponente::Deducao,
                    usada_na_verificacao: true,
                },
            ],
            esito_componenti: EsitoComponenti::Ok,
            campi_rinviati: vec![
                rinviato("centro_custo"),
                rinviato("dre_linha_id"),
                rinviato("dre_linha_nome"),
                rinviato("observacoes"),
            ],
            avisos_documento: Vec::new(),
            stato_documento: StatoDocumento::Ok,
        };
        ContextoDocumento {
            chave_origem: "29260512345678000195550010001256612345678901".to_string(),
            numero: "12566".to_string(),
            serie: "1".to_string(),
            data_documento: NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
            competencia: Competencia::new(2026, 5).unwrap(),
            cnpj_empresa: "55746722000181".to_string(),
            nome_empresa: "RESTAURANTE OESA LTDA".to_string(),
            arquivo_origem: "C:/qualunque/percorso/nfe.xml".to_string(),
            alocacao: AlocacaoNfe {
                lancamentos: vec![lancamento],
                reconciliacao,
            },
            avisos_leitura: Vec::new(),
            lote: IdentidadeLote {
                id: "lote-2026-05-A".to_string(),
                gerado_em: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
                descricao: "qualunque descrizione".to_string(),
            },
        }
    }

    fn contas_do_vetor(nome: &str) -> Vec<ContaMostrada> {
        vec![ContaMostrada {
            conta_id: "2.01.004".to_string(),
            nome: nome.to_string(),
            grupo: "Custos de Mercadorias e Insumos".to_string(),
        }]
    }

    // -----------------------------------------------------------------
    // T3-12: forma canonica dei monetari (vettori A-F)
    // -----------------------------------------------------------------

    #[test]
    fn t3_12_forma_canonica_dos_monetarios() {
        // Vettori A-F di §4.8, con le forme attese scritte a mano.
        assert_eq!(dec(dec_de("10")), "DEC:10:0");
        assert_eq!(dec(dec_de("10.00")), "DEC:10:0");
        assert_eq!(dec(dec_de("10.0000")), "DEC:10:0");
        assert_eq!(dec(dec_de("0")), "DEC:0:0");
        assert_eq!(dec(dec_de("0.00")), "DEC:0:0");
        assert_eq!(dec(dec_de("-0.00")), "DEC:0:0");
        assert_eq!(dec(dec_de("0.005")), "DEC:5:3");
        assert_eq!(dec(dec_de("1.2345")), "DEC:12345:4");
        assert_eq!(dec(dec_de("-7.50")), "DEC:-75:1");
        assert_eq!(dec(Decimal::MAX), "DEC:79228162514264337593543950335:0");
        assert_eq!(dec(Decimal::MIN), "DEC:-79228162514264337593543950335:0");
    }

    #[test]
    fn t3_12_impressoes_dos_vetores_a_f() {
        // Le impronte pubblicate in §4.8: frammenti **senza** LF finale.
        for (texto, esperado) in [
            (
                "DEC:10:0",
                "aee69fbd7f62989bcc01ef8b12312a3ae2e29e46565803cb51d3c11a1ee0887e",
            ),
            (
                "DEC:0:0",
                "c8bcddd2d2f5eeaa5a4219ef9bca692407ec5f94bbd3463ffed24ffc0017ca09",
            ),
            (
                "DEC:5:3",
                "f4e645c10e090c5480ecf6ea999f3c80a694c51069cc8ed08c9c3f0f4ec886fb",
            ),
            (
                "DEC:12345:4",
                "1707861d7ba0a2019c7c5eed783104e488542f88bdd6e65c73b57627a0f3ed86",
            ),
            (
                "DEC:-75:1",
                "9894b03c850a9ea0adffa6ed4bf29e5f7d5ec19c66370b1bacd2c58df8bc11d4",
            ),
            (
                "DEC:79228162514264337593543950335:0",
                "aec9288e7db8a652090cf240be97465083d59a99a9caf778d09ff31db90d4655",
            ),
        ] {
            assert_eq!(sha256_hex(texto.as_bytes()), esperado, "vetor {texto}");
        }
        // Lunghezza del vettore F: 35 byte, non 33.
        assert_eq!("DEC:79228162514264337593543950335:0".len(), 35);
    }

    #[test]
    fn t3_12b_ausencia_difere_de_zero_real() {
        // Vettori G e H di §4.8.
        let ausente: Dado<Decimal> = Dado::ausente(
            "nfe:cobr/dup/dVenc",
            StatusDado::NaoDisponivel,
            "a NF-e declara vencimentos, nao pagamentos",
        )
        .unwrap();
        let zero = Dado::presente(Decimal::ZERO, "nfe:cobr/dup/dVenc").unwrap();
        let texto_ausente = dado(&ausente, |v| dec(*v));
        let texto_zero = dado(&zero, |v| dec(*v));
        assert_eq!(texto_ausente, "DADO:NAO_DISPONIVEL:18:nfe:cobr/dup/dVenc:-");
        assert_eq!(texto_zero, "DADO:OK:18:nfe:cobr/dup/dVenc:DEC:0:0");
        assert_eq!(
            sha256_hex(texto_ausente.as_bytes()),
            "705ef2219871afabb3c09d229bbfc02484e9a72f4aa7eb0d9ab9f643a9c834fc"
        );
        assert_eq!(
            sha256_hex(texto_zero.as_bytes()),
            "08046d522d2bc09e0253be9de2186d21390fb7c3eddf9790531935c69fea0292"
        );
        assert_ne!(texto_ausente, texto_zero);
    }

    #[test]
    fn t3_12b_prefixo_de_comprimento_separa_campos() {
        // Senza prefisso di lunghezza, due insiemi di campi diversi uniti da
        // un separatore danno lo stesso testo. Con il prefisso, no.
        let ingenuo_a = format!("{}|{}", "a|b", "c");
        let ingenuo_b = format!("{}|{}", "a", "b|c");
        assert_eq!(ingenuo_a, ingenuo_b, "il separatore ingenuo collide");

        let mut prefixado_a = String::new();
        campo(&mut prefixado_a, "F", "a|b");
        campo(&mut prefixado_a, "F", "c");
        let mut prefixado_b = String::new();
        campo(&mut prefixado_b, "F", "a");
        campo(&mut prefixado_b, "F", "b|c");
        assert_eq!(prefixado_a, "F=3:a|b\nF=1:c\n");
        assert_eq!(prefixado_b, "F=1:a\nF=3:b|c\n");
        assert_ne!(prefixado_a, prefixado_b);
    }

    #[test]
    fn t3_12b_separadores_e_lf_nos_textos_nao_quebram_a_codificacao() {
        // Testi con i caratteri della grammatica dentro i dati.
        let venenoso = "SUCO = 1:2 | #3 \"MARCA\"\nLINHA 2";
        let doc = documento_vetor_i(venenoso);
        let texto = texto_canonico(&doc);
        let esperado = format!("LDS={}:{}\n", venenoso.len(), venenoso);
        assert!(texto.contains(&esperado), "{texto}");
        // Il testo resta un flusso di byte leggibile: nessun escaping, e la
        // lunghezza dichiarata permette di ritrovare la fine del campo.
        assert!(!texto.contains("\r"));
    }

    // -----------------------------------------------------------------
    // T3-12/T3-13: il vettore I e la sua stabilita'
    // -----------------------------------------------------------------

    #[test]
    fn t3_12_vetor_i_byte_a_byte_e_impressao() {
        let doc = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        let texto = texto_canonico(&doc);
        assert_eq!(texto, VETOR_I_TEXTO, "il testo canonico non coincide");
        assert_eq!(texto.len(), 948, "il vettore I e' di 948 byte");
        assert!(texto.ends_with("RSD=OK\n"), "LF finale obbligatorio");
        assert_eq!(impressao_conteudo(&doc), VETOR_I_SHA);
    }

    #[test]
    fn t3_08_correcao_a_chave_e_total_invariados_muda_a_impressao() {
        // Vettore J: solo la descrizione cambia, chiave, conto e totale no.
        let doc = documento_vetor_i("COCA-COLA KS 290ML 2UN (10.00)");
        let texto = texto_canonico(&doc);
        assert_eq!(texto.len(), 952, "il vettore J e' di 952 byte");
        assert_eq!(impressao_conteudo(&doc), VETOR_J_SHA);
        assert_ne!(VETOR_J_SHA, VETOR_I_SHA, "la correzione e' visibile");
    }

    #[test]
    fn t3_13_impressao_invariante_a_ambiente_lote_e_campos_humanos() {
        let base = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        let esperado = impressao_conteudo(&base);

        let mut outro_arquivo = base.clone();
        outro_arquivo.arquivo_origem = "D:/outra/pasta/mesmo_conteudo.xml".to_string();
        outro_arquivo.alocacao.lancamentos[0].arquivo_origem = "D:/outra/pasta".to_string();

        let mut outro_lote = base.clone();
        outro_lote.lote = IdentidadeLote {
            id: "lote-2027-01-Z".to_string(),
            gerado_em: NaiveDate::from_ymd_opt(2027, 1, 2).unwrap(),
            descricao: "altra esecuzione".to_string(),
        };

        let mut campos_humanos = base.clone();
        campos_humanos.alocacao.lancamentos[0].observacoes =
            "annotazione scritta da una persona".to_string();
        campos_humanos.alocacao.lancamentos[0].hash_registro = "altro hash".to_string();

        for (caso, doc) in [
            ("percorso", outro_arquivo),
            ("lotto", outro_lote),
            ("campi umani", campos_humanos),
        ] {
            assert_eq!(impressao_conteudo(&doc), esperado, "caso {caso}");
        }
    }

    #[test]
    fn t3_13_impressao_invariante_a_forma_unicode() {
        // D8.4(a): la stessa descrizione in NFC e in NFD da' la stessa
        // impronta, e le stringhe memorizzate restano distinte.
        let nfc_texto = "A\u{00E7}ucar CRISTAL";
        let nfd_texto = "Ac\u{0327}ucar CRISTAL";
        assert_ne!(nfc_texto, nfd_texto, "i byte memorizzati sono diversi");
        assert_eq!(nfc_texto.len(), 15);
        assert_eq!(nfd_texto.len(), 16);

        let em_nfc = documento_vetor_i(nfc_texto);
        let em_nfd = documento_vetor_i(nfd_texto);
        assert_eq!(impressao_conteudo(&em_nfc), impressao_conteudo(&em_nfd));
        // Il dato memorizzato non viene riscritto.
        assert_eq!(em_nfd.alocacao.lancamentos[0].descricao_itens, nfd_texto);
    }

    // -----------------------------------------------------------------
    // T3-13/T3-20: N1, l'ordinamento canonico degli avvisi
    // -----------------------------------------------------------------

    fn aviso(codigo: &str, detalhe: &str) -> Aviso {
        Aviso {
            codigo: codigo.to_string(),
            detalhe: detalhe.to_string(),
        }
    }

    #[test]
    fn t3_13_n1_normalizar_antes_de_ordenar() {
        // Riscontro R3C3-02: due avvisi dello stesso codice, dettagli "f" e
        // "é". Nel secondo documento "é" e' in NFD. Se si ordinassero i byte
        // memorizzati, l'ordine cambierebbe con la forma Unicode, e due
        // contenuti equivalenti darebbero impronte diverse.
        let acento_nfc = "\u{00E9}";
        let acento_nfd = "e\u{0301}";

        let mut em_nfc = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        em_nfc.avisos_leitura = vec![aviso("AVISO", "f"), aviso("AVISO", acento_nfc)];
        let mut em_nfd = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        em_nfd.avisos_leitura = vec![aviso("AVISO", acento_nfd), aviso("AVISO", "f")];

        assert_eq!(
            impressao_conteudo(&em_nfc),
            impressao_conteudo(&em_nfd),
            "N1: l'ordine deve nascere dalle chiavi normalizzate"
        );
        // "f" (0x66) precede "é" in NFC (0xC3...): l'ordine canonico e' questo.
        let texto = texto_canonico(&em_nfd);
        let posicao_f = texto.find("AD=1:f").expect("avviso f presente");
        let posicao_acento = texto
            .find(&format!("AD=2:{acento_nfc}"))
            .expect("avviso con accento presente");
        assert!(posicao_f < posicao_acento, "{texto}");
    }

    #[test]
    fn t3_20_empate_canonico_conserva_os_dois_avisos() {
        // Chiavi NFC uguali, byte memorizzati diversi: entrambi restano, e
        // l'ordine relativo non puo' cambiare l'impronta perche' i byte
        // emessi coincidono.
        let acento_nfc = "\u{00E9}";
        let acento_nfd = "e\u{0301}";
        let mut direto = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        direto.avisos_leitura = vec![aviso("AVISO", acento_nfc), aviso("AVISO", acento_nfd)];
        let mut invertido = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        invertido.avisos_leitura = vec![aviso("AVISO", acento_nfd), aviso("AVISO", acento_nfc)];

        assert_eq!(impressao_conteudo(&direto), impressao_conteudo(&invertido));
        let texto = texto_canonico(&direto);
        assert!(texto.contains("AVL#2"), "nessun avviso scartato: {texto}");
        assert_eq!(
            texto.matches(&format!("AD=2:{acento_nfc}")).count(),
            2,
            "i due avvisi equivalenti emettono gli stessi byte: {texto}"
        );
        // Le stringhe originali restano distinte nel record.
        assert_ne!(
            direto.avisos_leitura[0].detalhe,
            direto.avisos_leitura[1].detalhe
        );
    }

    #[test]
    fn t3_25_permutacao_dos_avisos_de_leitura_e_indiferente() {
        let a = aviso("SOMA_ITENS_DIVERGE", "soma dos itens 9.00 != vProd 10.00");
        let b = aviso("DET_SEM_PROD", "item 2 ignorado: <det> sem <prod>");
        let mut direto = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        direto.avisos_leitura = vec![a.clone(), b.clone()];
        let mut invertido = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        invertido.avisos_leitura = vec![b, a];
        assert_eq!(texto_canonico(&direto), texto_canonico(&invertido));
    }

    // -----------------------------------------------------------------
    // T3-09: impronta di presentazione
    // -----------------------------------------------------------------

    #[test]
    fn t3_09_apresentacao_separa_nome_do_plano_do_conteudo() {
        // Vettori K e L: cambia solo il nome mostrato del conto.
        let com_nome = impressao_apresentacao(VETOR_I_SHA, &contas_do_vetor("Refrigerantes"));
        let com_nome_maiusculo =
            impressao_apresentacao(VETOR_I_SHA, &contas_do_vetor("REFRIGERANTES"));
        assert_eq!(com_nome, VETOR_K_SHA);
        assert_eq!(com_nome_maiusculo, VETOR_L_SHA);
        assert_ne!(com_nome, com_nome_maiusculo);
    }

    #[test]
    fn t3_09_texto_de_apresentacao_e_estavel() {
        let texto = texto_apresentacao(VETOR_I_SHA, &contas_do_vetor("Refrigerantes"));
        assert_eq!(
            texto,
            concat!(
                "V=IMPRESSAO-APRESENTACAO/v1\n",
                "CT=f8a6e8d0aade811e39e31f966c17e0c6623f21afdbde73a8c5866641031f614c\n",
                "PC#1\n",
                "PI=8:2.01.004\n",
                "PN=13:Refrigerantes\n",
                "PG=31:Custos de Mercadorias e Insumos\n",
            )
        );
    }

    // -----------------------------------------------------------------
    // Etichette: fissate una per una
    // -----------------------------------------------------------------

    #[test]
    fn etiquetas_dos_enums_sao_estaveis() {
        assert_eq!(origem(OrigemLancamento::XmlNfe), "XML_NFE");
        assert_eq!(origem(OrigemLancamento::Csv), "CSV");
        assert_eq!(origem(OrigemLancamento::Manual), "MANUAL");

        assert_eq!(status_revisao(StatusRevisao::Ok), "OK");
        assert_eq!(status_revisao(StatusRevisao::Revisar), "REVISAR");
        assert_eq!(status_revisao(StatusRevisao::Aprovado), "APROVADO");

        assert_eq!(esito_total(EsitoTotal::Ok), "OK");
        assert_eq!(esito_total(EsitoTotal::Erro), "ERRO");
        assert_eq!(esito_rate(EsitoRate::Ok), "OK");
        assert_eq!(esito_rate(EsitoRate::Erro), "ERRO");
        assert_eq!(esito_rate(EsitoRate::NaoAplicavel), "N.A.");
        assert_eq!(esito_rate(EsitoRate::NaoVerificado), "NAO_VERIFICADO");
        assert_eq!(esito_copertura(EsitoCopertura::Ok), "OK");
        assert_eq!(esito_copertura(EsitoCopertura::Parcial), "PARCIAL");
        assert_eq!(esito_componenti(EsitoComponenti::Ok), "OK");
        assert_eq!(esito_componenti(EsitoComponenti::Parcial), "PARCIAL");
        assert_eq!(stato_documento(StatoDocumento::Ok), "OK");
        assert_eq!(stato_documento(StatoDocumento::Parcial), "PARCIAL");
        assert_eq!(stato_documento(StatoDocumento::Erro), "ERRO");

        assert_eq!(ruolo_componente(RuoloComponente::Base), "BASE");
        assert_eq!(ruolo_componente(RuoloComponente::Acessorio), "ACESSORIO");
        assert_eq!(ruolo_componente(RuoloComponente::Deducao), "DEDUCAO");
        assert_eq!(
            ruolo_componente(RuoloComponente::Informativa),
            "INFORMATIVA"
        );

        // Tutte le varianti di TipoConta, per non lasciarne scoperta nessuna.
        let etiquetas: Vec<&str> = TipoConta::TODOS.iter().map(|t| tipo_conta(*t)).collect();
        assert_eq!(
            etiquetas,
            vec![
                "RECEITA",
                "DEDUCAO_RECEITA",
                "CUSTO",
                "DESPESA",
                "IMPOSTO",
                "INVESTIMENTO",
                "PASSIVO",
                "PATRIMONIAL",
                "PROVISAO",
                "AJUSTE",
                "REVISAO",
            ]
        );
        assert_eq!(StatusDado::NaoAplicavel.rotulo(), "N.A.");
    }

    // -----------------------------------------------------------------
    // Conti mostrati
    // -----------------------------------------------------------------

    #[test]
    fn contas_mostradas_vem_do_plano_e_sao_ordenadas() {
        let plano = crate::regras::PlanoContas::carregar(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("config/plano_contas_gerencial.yaml"),
        )
        .unwrap();
        let mut doc = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        let mut segunda = doc.alocacao.lancamentos[0].clone();
        segunda.conta_id = "2.01.002".to_string();
        segunda.conta_nome = "nome divergente, non usato qui".to_string();
        // Inserita prima, per verificare che l'ordine venga dal `conta_id`.
        doc.alocacao.lancamentos.insert(0, segunda);

        let contas = contas_mostradas(&doc, &plano).unwrap();
        assert_eq!(contas.len(), 2);
        assert_eq!(contas[0].conta_id, "2.01.002");
        assert_eq!(contas[1].conta_id, "2.01.004");
        // Il nome e' quello del piano, non quello della riga.
        assert_eq!(contas[1].nome, "Refrigerantes");
        assert_ne!(contas[0].nome, "nome divergente, non usato qui");
    }

    #[test]
    fn conta_fora_do_plano_e_erro_nomeado() {
        let plano = crate::regras::PlanoContas::carregar(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("config/plano_contas_gerencial.yaml"),
        )
        .unwrap();
        let mut doc = documento_vetor_i("COCA-COLA KS 290ML (10.00)");
        doc.alocacao.lancamentos[0].conta_id = "9.99.999".to_string();
        let erro = contas_mostradas(&doc, &plano).unwrap_err();
        assert_eq!(erro.conta_id, "9.99.999");
        assert_eq!(erro.chave_origem, doc.chave_origem);
    }
}

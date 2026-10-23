| il comportamento di Excel sul file prodotto, su# Specifica fase 3C.4 — export derivato e confine degli ingressi

| Voce | Valore |
|---|---|
| Data | 2026-10-22 |
| Mandato | `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` |
| Perimetro | **analisi e specifica**. Nessuna riga di Rust del progetto, nessuna modifica di `Cargo.*`, nessuna installazione, nessun archivio reale, nessun commit |
| Stato | **AUDIT DELLA SPECIFICA SUPERATO** il 2026-10-23 (§18.3); **D18–D26 ed `EXP-29` decise** (§3.3). Nessuna decisione resta aperta: la specifica è pronta per il mandato implementativo 3C.4.1. I contratti `EXP-nn` restano proposte e le prove `TX-nn` pianificate 
| Revisione 2 | correzione dei cinque riscontri dell'audit del 2026-10-23, con tre riproduzioni sul codice reale in §5.4. Contratti nuovi `EXP-30…EXP-45`, prove nuove `TX-29…TX-36`, proposta nuova D24 |
| Revisione 3 | correzione dei due residui dell'audit mirato del 2026-10-23, riprodotti in §5.5: autore dell'approvazione attiva e confronto dei nomi riservati. Contratti nuovi `EXP-46…EXP-50`, prove nuove `TX-34b`, `TX-21` e `TX-33` estesi. Quattro allineamenti accolti: §18.2 |
| Revisione 4 | precisazione recepita dall'esito finale: il file a zero byte viene dalla **rivendicazione** (`EXP-43`), non dal lucchetto (`EXP-44`). L'alternativa già descritta in §8.2 ha ora un nome proprio, **D25**, e le due scelte sono indipendenti: §18.3 |
| Revisione 5 | registrazione delle decisioni D18-D25 del 2026-10-23 e aggiornamento di tutto ciò che ne dipende: D18 porta a `rust_xlsxwriter` (§10.3, §10.5, §12), D25(b) elimina la rivendicazione e il residuo a zero byte (§8.2), D22 porta la condizione di Luca (§7.4.1, `EXP-51`). Contratti nuovi `EXP-43b`, `EXP-51`, `EXP-52`; `EXP-43` ed `EXP-49` decaduti; prove nuove `TX-37`, `TX-38` |
| Revisione 6 | recepito il controllo mirato delle decisioni del 2026-10-23: `R5-01`, l'incompatibilità fra D18 e il divieto assoluto di `f64`, verificata da me sulla firma dell'API e aperta come decisione **D26** (§7.2.3); lettura operativa di D22 precisata (§7.4.1); affermazione sull'offline circoscritta (§10.3); `EXP-29` registrato come raccomandazione (§10.2). `EXP-07`, `EXP-39` ed `EXP-40` sospesi: §18.5 |
| Revisione 7 | **D26(a) approvata** da Luca il 2026-10-23: `f64` ammesso soltanto nel confine di scrittura della cella numerica, con le cinque condizioni. Riallineati `EXP-07` ed `EXP-39`, `EXP-40` decaduto, `EXP-53` nuovo, `TX-04` sdoppiato fra confronto esatto del testo e confronto per valore della cella numerica: §7.2.1, §7.2.3 |
| Revisione 8 | **`EXP-29` approvata** da Luca il 2026-10-23: prova indipendente ripetibile con Python e `openpyxl` già disponibili, senza installazioni (§10.2). §11.1 allineata: tolto il rinvio operativo a `EXP-40` decaduto, e §11.2 nuova descrive il controllo dell'unica funzione autorizzata a usare `f64` secondo `EXP-53` |
| HEAD alla stesura | `af5683d4522166d5283fdd1f206c91cb3e841dac` |
| Baseline tecnica | **[RIUSATA]** dal lotto 3C.3.b.1: 337 test (319 lib, 3 binario, 15 integrazione), `fmt`/`clippy`/`diff --check` puliti. Non rieseguita: questo lotto produce solo Markdown |
| Decisioni approvate dal mandato | **D11(a)** import dello storico Excel rinviato; **D12(a)** importo non interpretabile → riga `INCOMPLETO` con testo originale e motivo |
| Decisioni già in vigore | D7(a), D8.1–D8.5, D9.1–D9.5, D10(a), D13(a), D14(a)–D17(a) |
| Identificativi | contratti **`EXP-nn`**, prove pianificate **`TX-nn`**. Nessuna collisione con `C-nn`, `F3C-nn`, `PER-nn`, `T3-*`, `TP-*` |
| Metodo | `programmatore-bravo`: prima contrattualizza, poi programma |

---

## 0. Come leggere questo documento

| Etichetta | Significato |
|---|---|
| **[ESEGUITO]** | prova eseguita per questo documento, con input, strumento ed esito in §5 |
| **[LETTO_NEL_CODICE]** | fatto letto nei sorgenti alla baseline, con file e riga |
| **[RIUSATO]** | evidenza di una fase o di un audit precedenti, non rieseguita, con la fonte nominata |
| **[PROPOSTA]** | scelta di questo documento, **non** approvata da nessuno |
| **[NON_VERIFICATO]** | affermazione plausibile che nessuno ha provato; dichiarata tale e non usata come fondamento |

Tre regole che valgono su tutto il testo:

1. Nessuna riga del writer esiste. Tutti i test `TX-nn` sono **pianificati**.
2. Le sonde di §5 misurano **il formato XLSX e i lettori disponibili**, non il
   futuro writer Rust. Una sonda che scrive una cella con PowerShell non
   dimostra che il writer Rust la scriverà così: lo dimostreranno i test
   pianificati.
3. L'etichetta di approvazione di D11(a)/D12(a) **non** si trasferisce ai
   contratti di formato `EXP-nn` né alle dipendenze proposte in §10.

---

## 1. Obiettivo operativo

Produrre, dall'archivio JSON già autorevole, **un file consultabile da una
persona** che risponda a tre domande senza aprire il JSON:

1. che cosa è stato registrato in questo lotto, con la riconciliazione e gli
   avvisi che ne dichiarano i limiti;
2. quali righe sono state **approvate da una persona**, distinguendole dalle
   righe che un esito tecnico ha marcato `OK`;
3. da dove viene ogni riga: lotto, documento, impronta del contenuto, identità
   del piano, stato della revisione.

Il file è un **derivato**. D7(a) resta: l'archivio è la fonte autorevole, e
nulla di ciò che accade al file torna indietro. Rigenerarlo dallo stesso
archivio e dallo stesso piano deve dare lo stesso contenuto.

### 1.1 Il percorso, con le funzioni reali

Tutte le funzioni citate esistono alla baseline. **[LETTO_NEL_CODICE]**

```text
arquivo_io::ler(&Destino)                                     src/arquivo_io.rs:525
  -> Leitura { arquivo, base: TokenBase }                     (valida la struttura)
  |
  +-- arquivo::verificar_apresentacao(&sotto_archivio, &plano) src/arquivo.rs:1465
  |     dove `sotto_archivio` contiene i soli snapshot e l'esecuzione del
  |     lotto scelto (§4.4): la funzione non si modifica, si restringe
  |     l'insieme su cui la si chiama
  |     -> Vec<Divergencia>                                    (D8.3: non e' un errore)
  |
Arquivo::contextos_do_lote(lote) -> Vec<ContextoDocumento>     src/arquivo.rs:221
Arquivo::eventos()               -> Vec<EventoRevisao>         src/arquivo.rs:230
  |
revisao::aplicar_revisao(&contextos, &plano, &eventos)         src/revisao.rs:656
  -> VistaRevisada {
       pre_livro: PreLivro,          <- costruito da prelivro::construir_pre_livro
       linhas: Vec<LinhaRevisada>,
       versoes, trilha, eventos_nao_aplicados,
       exclusoes_extrato_revisado, avisos
     }
  |
  +-- VistaRevisada::extrato_revisado() -> Vec<&LinhaRevisada> src/revisao.rs:511
  |
<<< NUOVO IN 3C.4 >>>
export::modelo(&VistaRevisada, &Proveniencia) -> ModeloExport  (puro, senza IO)
export_io::escrever(&Destino, &ModeloExport)                   (IO, un solo commit)
  -> file derivato
  |
rilettura con uno strumento indipendente -> confronto campo per campo
```

**La barriera di `construir_pre_livro` resta dove è.** `aplicar_revisao` la
attraversa due volte: una per validare i documenti originali, una per
proiettare la vista. **[LETTO_NEL_CODICE]** `src/revisao.rs:656` e seguenti:
la funzione valida, applica, **proietta una copia** dei contesti con il solo
`status_revisao` aggiornato e ricostruisce il pre-libro. Il writer non chiama
`construir_pre_livro` da sé e non duplica nessuna aritmetica: riceve la
`VistaRevisada` già composta.

| Chi controlla che cosa | Dove | Che cosa **non** attesta |
|---|---|---|
| i byte del file sono un archivio leggibile, le impronte tornano, i riferimenti chiudono | `arquivo::verificar_estrutura`, chiamata da `arquivo_io::ler` | nulla sui conti del piano corrente |
| i **nomi mostrati** dei conti sono ancora quelli dell'impronta di presentazione archiviata | `arquivo::verificar_apresentacao` → `Vec<Divergencia>` | **non** confronta identità, versione o hash del piano: due piani con gli stessi nomi di conto e versioni diverse danno zero divergenze (§4.4, misurato) |
| gli invarianti contabili del lotto: competenza, azienda, conti nel piano, documenti coerenti | `prelivro::construir_pre_livro`, **dentro** `aplicar_revisao` | nulla sugli altri lotti dell'archivio |
| le decisioni umane sono applicabili al contenuto che citano | `revisao::aplicar_revisao` | — |
| il file derivato dice la verità su tutto questo | **nuovo**, contratti `EXP-nn` di §7 | — |

**Attenzione allo scope**, riscontro `R3C4-05`: `verificar_apresentacao` visita
**tutti** gli snapshot dell'archivio, non il lotto scelto, e può restituire un
**errore** `ContaFora`, non solo una lista di divergenze. §4.4 misura la
conseguenza e dice dove va applicata.

Il validatore archivistico **non** sostituisce il secondo: è il riscontro
registrato dall'audit 3C.3.b.1 (§18.8 di
`SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`), e vale anche qui.

### 1.2 Che cosa questo lotto non fa

Fuori perimetro, per mandato e non per dimenticanza: import dello storico Excel
(D11(a)), correzioni che arrivino da un workbook, CLI (3C.5), DRE (3D),
autenticazione dell'autore, nuove regole contabili, riscrittura dell'archivio,
e la risoluzione dei limiti già dichiarati della persistenza. Nessuno di questi
è un prerequisito dell'export: il percorso di §1.1 funziona senza di loro.

---

## 2. Fonti e fotografie superate

| Fonte | Che cosa se ne prende |
|---|---|
| `SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md` | i contratti `F3C-06/10/14–21/24` e i requisiti generali dell'export |
| `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` | D8/D9 approvate, grammatiche delle impronte, selezione dell'estratto revisionato |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | D7 e D14–D17, codec e protocollo archivistico, §18 con le prove eseguite |
| il codice alla baseline | il comportamento **implementato**: `arquivo`, `arquivo_io`, `prelivro`, `revisao`, `impressao`, `dinheiro`, `modelos`, `regras` |

Alcune righe dei documenti storici fotografano stati superati. Le indico qui
una volta e non le riscrivo, perché non è il lavoro di questo lotto:

| Riga storica | Stato reale alla baseline |
|---|---|
| «D7/D8/D9 aperte» nella specifica generale 3C | approvate: D8/D9 il 2026-10-21, D7(a) col mandato 3C.3.b |
| `F3C-14`, `F3C-15`, `F3C-20`, `F3C-21` «NON_VERIFICATI: dipendono da IO che non esiste» | l'IO esiste: `F3C-14`/`F3C-15` sono coperti dallo storico archivistico (TP-09/TP-10), `F3C-20` dal protocollo di pubblicazione (TP-13), `F3C-21` resta **aperto per l'export** ed è oggetto di questo documento |
| «i crate XLSX non sono disponibili» | fotografia storica: lo stato attuale della cache è misurato in §10.1, e la conclusione **cambia** |
| README «aguarda apenas o registro em Git autorizado» per 3B/3C.3.b.1 | registrato: `9afd626` e `af5683d` |

`F3C-19` parlava di «26 vs 31 colonne» del Python. Questo documento **non**
adotta quel layout: §6.1 spiega perché e che cosa propone al suo posto.

---

## 3. Decisioni

### 3.1 Approvate dal mandato di questo lotto

| Decisione | Regola | Conseguenza verificabile in questo documento |
|---|---|---|
| **D11(a)** | import dello storico Excel **rinviato**; il flusso nuovo non dipende da workbook storici non forniti | §6.1 progetta lo schema dal modello reale, non dal layout 26/31; §10.2 non richiede un **lettore** XLSX in Rust |
| **D12(a)** | importo non interpretabile in un ingresso grezzo → riga `INCOMPLETO`, testo originale e motivo conservati, totali interessati dichiarati incompleti | §15 definisce il confine dell'adattatore futuro e la rappresentazione del valore; **nessun** test fittizio su un `Decimal`, che non può essere illeggibile |

D12(a) è una decisione funzionale presa, non un ingresso nuovo da costruire
qui. **[LETTO_NEL_CODICE]** `src/modelos.rs:553`: `Lancamento::valor` è un
`Decimal`, quindi una cella illeggibile **non è rappresentabile** in quel tipo,
e riempirlo di zero sarebbe esattamente lo zero tecnico che il metodo vieta.
`Particao::Incompleto` non esiste: **[LETTO_NEL_CODICE]** `src/prelivro.rs:303`
l'enum ha tre varianti, `Revisar`, `Aprovado`, `Ok`, e la specifica 3C.2
dichiara che `INCOMPLETO` e `NAO_RECONHECIDO` «richiedono un ingresso grezzo,
che questo blocco non accetta».

Quindi: D12 **non** autorizza a toccare gli enum né le regole di revisione
auditati. §15 dice dove quel confine passa e chi lo attraverserà.

### 3.2 Già in vigore, non riaperte

D7(a) l'archivio è autorevole e il workbook è un derivato; D8.1–D8.5 identità e
impronte; D9.1–D9.5 revisione, revoca e provenienza; D10(a) l'estratto esclude
i documenti `ERRO`; D13(a); D14(a)–D17(a) persistenza.

Una conseguenza che l'export deve rendere visibile e **non** può decidere da
sé: una modifica al workbook non è una decisione umana e non è una modifica
all'archivio. Il file derivato non ha un percorso di ritorno, e §8.4 lo
dichiara nel file stesso.

### 3.3 Decisioni D18–D25, approvate da Luca il 2026-10-23

Approvate **dopo** l'audit della specifica, e con una condizione esplicita su
D22. Non sono più proposte: ciò che segue nel documento è aggiornato di
conseguenza, e dove una sezione descriveva la strada non scelta lo dichiara.

| Decisione | Esito | Conseguenza in questo documento |
|---|---|---|
| **D18** | **rust_xlsxwriter**, non il writer scritto a mano. Il mandato implementativo deve indicare versione, funzionalità abilitate e dipendenze aggiunte. Restano i contratti di conservazione dei dati e la prova di rilettura indipendente | §10.3 e §10.5; `src/xlsx.rs` diventa un adattatore sottile, non un pacchettizzatore; la stima di 500–700 righe decade |
| **D19** | **approvata**: schema `FANTOZZI-EXPORT/v1` come in §6 | §6 è il contratto; cambiarlo richiede `v2` |
| **D20** | **approvata** con la regola di §7.2.1: colonna numerica presente, cella eleggibile a 15 cifre significative, assenza dichiarata con stato e motivo | §7.2.1 resta come scritta; `TX-04` la verifica |
| **D21** | **approvata**: tutti gli eventi dell'archivio, senza scarti silenziosi | §4.3; `ALVO_AUSENTE` è atteso e i conteggi di `proveniencia` lo rendono visibile |
| **D22** | **approvato l'obiettivo**: conservare esattamente il testo e segnalare i caratteri non rappresentabili. **Condizione**: la codifica va verificata **con rust_xlsxwriter**, senza nessuna trasformazione manuale aggiuntiva priva di prova; un testo che contenga letteralmente `_x0041_` deve tornare **identico** alla rilettura; la compatibilità Excel va verificata prima di dichiararla stabile | §7.4.1 e §7.4.3; `EXP-17` riformulato, `EXP-51` nuovo, prove `TX-19` e `TX-37` |
| **D23** | **approvata**: determinismo dei contenuti e dell'ordine, senza richiedere ZIP identici byte per byte | §9 resta come scritta |
| **D24** | **approvata**: lucchetto cooperativo sulla destinazione | §8.2, `EXP-44` |
| **D25** | **(b) approvata**: `Recusar` per `exists()` più `rename`, **senza** rivendicazione del nome finale | §8.2 e §8.4: nessun residuo a zero byte; `EXP-43` e `EXP-49` decadono, e il limite di non atomicità va dichiarato |
| **D26** | **(a) approvata**: `f64` consentito **soltanto** nel confine di scrittura della cella numerica di comodo, con le cinque condizioni. L'alternativa, rinunciare alla colonna, scartata | §7.2.1 e §7.2.3: `EXP-07` con una eccezione nominata, `EXP-39` per valore e non per caratteri, `EXP-40` decaduto, `EXP-53` nuovo, `TX-04` sdoppiato |

### 3.4 Che cosa resta da fissare

**D18–D26 e `EXP-29` sono tutte decise.** Nessuna decisione resta aperta.
Resta una cosa sola, che il mandato implementativo risolve quando parte:

| Id | Che cosa manca | Chi decide |
|---|---|---|
| versione e funzionalità di `rust_xlsxwriter` | il nome è deciso, la **versione esatta**, il set minimo di funzionalità e l'elenco transitivo no: §10.5 dice che cosa il mandato deve registrare e perché non si può fissare qui | il mandato implementativo, con la rete disponibile per l'acquisizione |

I contratti `EXP-01…EXP-53` restano **[PROPOSTA]**: le decisioni di Luca
riguardano le scelte funzionali D18–D26 ed `EXP-29`, non la formulazione di
ogni contratto,
che l'audit ha giudicato implementabile ma che nessuno ha approvato uno per uno.

## 4. Che cosa esiste già, e come si compone

### 4.1 I dati che l'export deve riversare, campo per campo

Non c'è nulla da calcolare: la `VistaRevisada` contiene già tutto. Elenco
completo, **[LETTO_NEL_CODICE]**, perché lo schema di §6 non lasci fuori niente
per distrazione.

| Struttura | Campi | File |
|---|---|---|
| `LinhaPreLivro` | `identidade`, `lancamento`, `particao`, `motivos`, `chave_resumo`, `conta_nome_plano`, `empresa_identificada`, `stato_documento`, `no_extrato_aprovado` | `prelivro.rs:336` |
| `IdentidadeLinha` | `lote`, `chave_origem`, `origem`, `conta_id`, `ordinal` | `prelivro.rs:326` |
| `Lancamento` | 26 campi, fra cui `valor: Decimal`, `duplicatas: Vec<Duplicata>`, `data_pagamento: Dado<NaiveDate>`, `competencia`, `tipo_lancamento` | `modelos.rs:540` |
| `LinhaRevisada` | `chave`, `identidade`, `status_tecnico`, `estado`, `decisoes`, `anotacoes`, `no_extrato_revisado` | `revisao.rs:471` |
| `DocumentoPreLivro` | `lote`, `chave_origem`, `numero`, `serie`, `data_documento`, `competencia`, `cnpj_empresa`, `nome_empresa`, `arquivo_origem`, `empresa_identificada`, `stato_documento`, `linhas`, `reconciliacao`, `avisos_leitura` | `prelivro.rs:358` |
| `Reconciliacao` | `valor_total_nfe`, `soma_lancamentos`, **quattro esiti** (`esito_total`, `esito_rate`, `esito_copertura`, `esito_componenti`), `soma_rate_documento`, `soma_rate_allocate`, `articoli_allocati`, `articoli_attesi`, `articoli_scartati`, `componenti`, `campi_rinviati`, `avisos_documento`, `stato_documento` | `alocacao.rs:270` |
| `Resumo` | `chave`, `conta_nome`, `grupo_conta`, `empresa_identificada`, `qtd_*` ×4, `valor_*` ×4, `documentos_*` ×3, `documentos_marcados`, `nomes_divergentes` | `prelivro.rs:391` |
| `Indicadores` | 15 contatori e somme del lotto | `prelivro.rs:427` |
| `Exclusao` | `identidade`, `valor`, `particao`, `stato_documento`, `motivo` | `prelivro.rs:418` |
| `VersaoDocumento` | `chave_origem`, `impressao_conteudo`, `impressao_apresentacao`, `sequencia: Dado<u32>`, `motivo: Dado<MotivoNovaVersao>` | `revisao.rs:459` |
| `DecisaoAplicada` | `evento: EventoRevisao`, `repeticoes` | `revisao.rs:400` |
| `EventoNaoAplicado` | `evento`, `motivo: MotivoNaoAplicacao`, `detalhe` | `revisao.rs:440` |
| `Aviso` | `codigo`, `detalhe` | `nfe.rs:147` |

Tre strutture annidate **ripetute** non entrano in una riga sola senza perdere
dati, e avranno un foglio proprio (§6.2): `Lancamento::duplicatas`,
`Reconciliacao::componenti`, `Reconciliacao::campi_rinviati`.

### 4.2 L'estratto revisionato è l'unico ammesso per «approvato»

**[LETTO_NEL_CODICE]** `revisao.rs:511`: `extrato_revisado()` filtra
`no_extrato_revisado`, ed è **più severo** di `PreLivro::extrato_aprovado()`
(`prelivro.rs:465`) perché esclude anche le righe `APROVADO` senza provenienza.
Il campo nasce in `aplicar_revisao` e richiede `EstadoRevisao::AprovadoPorPessoa`,
cioè un'approvazione umana **attiva**; non basta lo stato tecnico `APROVADO`.

| Caso da coprire nell'export | Stato risultante | Nell'estratto revisionato? |
|---|---|---|
| esito tecnico `OK`, nessun evento | `TECNICO_OK` | no |
| esito tecnico `REVISAR`, nessun evento | `TECNICO_REVISAR` | no |
| `APROVADO` in ingresso senza evento | `APROVADO_SEM_PROVENIENCIA` | no, e compare in `exclusoes_extrato_revisado` |
| `APROVADO` in ingresso **sanato** da un evento di approvazione | `APROVADO_POR_PESSOA` | sì |
| sanatoria **revocata** | `APROVACAO_REVOGADA` | no |
| approvazione, revoca, nuova approvazione | `APROVADO_POR_PESSOA` | sì, con tre decisioni in traccia |
| riga approvata di un documento `ERRO` | `APROVADO_POR_PESSOA` | no, per D10(a) |

**[RIUSATO]** dai test della fase 3C.3.a (`t3_16_*`, `r1_revogacao_da_sanatoria_*`,
`r1_revogacao_de_linha_ok_*`): questi sette casi sono già coperti nel dominio.
L'export deve soltanto **non perderli**, e il foglio `extrato_revisado` di §6.2
porta lo stato accanto a ogni riga perché la differenza sia leggibile.

### 4.3 Il lotto si sceglie, gli eventi no — D21, approvata

`Arquivo::contextos_do_lote(lote)` filtra per lotto; `Arquivo::eventos()`
restituisce **tutte** le dichiarazioni dell'archivio. **[LETTO_NEL_CODICE]**
`arquivo.rs:221` e `:230`.

Passare a `aplicar_revisao` soltanto gli eventi «di questo lotto» sarebbe un
filtro silenzioso, e sarebbe anche **sbagliato**: la chiave di un evento è
`ChaveRevisao { chave_origem, impressao_conteudo, origem, conta_id, ordinal }`
(`revisao.rs:90`) e **non contiene il lotto**. È la conseguenza voluta di D8:
se lo stesso documento compare identico in due lotti, un'approvazione
dichiarata sul suo contenuto vale per entrambi. L'archivio lo conserva così
(prova `TP-09`, §18.3 della specifica di persistenza).

**[PROPOSTA] D21**: passare l'elenco completo. Conseguenza dichiarata: gli
eventi che riguardano documenti assenti da questo lotto compaiono in
`eventos_nao_aplicados` con motivo `ALVO_AUSENTE`, e il foglio omonimo li
mostra. Non è rumore: è la registrazione che qualcuno ha deciso qualcosa che
non riguarda il lotto esportato. Il foglio `proveniencia` dichiara i numeri —
eventi nell'archivio, applicati, non applicati per motivo — così nessun
filtraggio può restare invisibile.

Alternativa rifiutata: filtrare per `chave_origem` presente nel lotto.
Nasconderebbe le decisioni su documenti che l'utente si aspetta di trovare, e
l'assenza non sarebbe spiegata da nessuna parte.

### 4.4 Lo scope del controllo di presentazione, e le due identità del piano

Riscontro `R3C4-05`. Due fatti misurati sul codice reale (§5.4).

**Primo: `verificar_apresentacao` non è ristretta al lotto.** Visita
`arquivo.snapshots` per intero, e per ogni snapshot chiama
`impressao::contas_mostradas`, che su un conto fuori dal piano restituisce
`ContaFora` — un **errore**, non una divergenza. Misurato: un archivio con
`lote-A` su `2.01.004` e `lote-B` su `2.05.001`, con un piano che contiene solo
il primo conto, fa **fallire** il controllo globale, mentre
`aplicar_revisao` del solo `lote-A` riesce. L'export del lotto A si fermerebbe
per un conto che appartiene soltanto al lotto B.

**EXP-30** **[PROPOSTA]**: il controllo di presentazione si applica a un
**sotto-archivio** costruito dal lotto scelto, con i soli snapshot di quel
lotto e la sua esecuzione. `verificar_apresentacao` **non** si modifica: si
restringe l'insieme. Un `ContaFora` che arrivi da quel sotto-archivio è un
errore legittimo dell'export, perché riguarda un conto che il lotto esportato
usa davvero, e va nominato con la chiave e il conto.

**EXP-31** **[PROPOSTA]**: la validazione strutturale dell'intero archivio
resta dov'è, in `arquivo_io::ler`. Le due cose non si confondono: l'archivio si
valida tutto, la presentazione si confronta sul lotto.

**Secondo: l'impronta di presentazione non è l'identità del piano.** Misurato:
una copia del piano canonico con la sola `versao` cambiata da `1.0` a `9.9`,
caricata e validata, dà **zero divergenze**, perché l'impronta copre i nomi dei
conti mostrati, non la versione del file. La riga della tabella di §1.1 che
diceva «il piano corrente è ancora quello con cui si è scritto» è stata
corretta: non è ciò che la funzione attesta.

**EXP-32** **[PROPOSTA]**: il foglio `proveniencia` porta **due** identità del
piano, non una:

| Gruppo di colonne | Significato |
|---|---|
| `plano_arquivado_versao`, `plano_arquivado_sha256`, `plano_arquivado_caminho` | `IdentidadePlano` registrata nell'esecuzione del lotto: il piano che ha prodotto le impronte archiviate |
| `plano_usado_versao`, `plano_usado_caminho` | il piano che il chiamante ha passato, e che ha prodotto i nomi e la vista **esportati** |
| `plano_coincide` | confronto **soltanto** di versione e percorso: `SIM` se coincidono entrambi, `PARCIALE` se coincide la versione e non il percorso, `NAO` altrimenti |

**`plano_coincide = SIM` non certifica che il piano sia lo stesso file.** Un
piano può essere modificato conservando versione e percorso, e il confronto non
se ne accorgerebbe. Il nome dice quello che la colonna fa — i due metadati
coincidono — e il foglio `limites` lo dichiara. Chi volesse la garanzia sul
contenuto deve passare un hash del piano usato, che `PlanoContas` non calcola:
sarebbe una colonna in più in `v2` e un dato in più nella `Proveniencia`.
Questo significato ristretto va conservato nel file e nella documentazione, e
non va esteso per comodità di lettura.

`PlanoContas` espone `versao()` e la propria fonte, **[LETTO_NEL_CODICE]**
`regras.rs:195`; non espone un hash del file, quindi
`plano_usado_sha256` non esiste e la colonna **non** viene inventata: la
coincidenza si dichiara su ciò che si può misurare. Se servisse l'hash del
piano usato, è il chiamante a doverlo calcolare e passare, e sarebbe una
colonna in più in `v2`.

### 4.5 Che cosa l'export **non** inventa

| Non inventa | Perché |
|---|---|
| una sequenza storica dei documenti | D17(a): `VersaoDocumento::sequencia` è `Dado<u32>` `NAO_DISPONIVEL`, e `revisao_arquivo` conta le pubblicazioni del file, non le versioni di un documento |
| un totale «completo» dove un dato manca | gli esiti della riconciliazione e `stato_documento` dicono già `PARCIAL`/`ERRO`; l'export li riporta, non li risolve |
| una percentuale o un indicatore derivato | `Indicadores` ha conteggi e somme, **[LETTO_NEL_CODICE]** `prelivro.rs:427`: nessuna percentuale esiste e nessuna viene aggiunta |
| un nome di conto diverso da quello del piano | `conta_nome_plano` viene dal piano validato; `Resumo::nomes_divergentes` registra le divergenze invece di appianarle |
| una somma nuova | i riepiloghi sono la fonte; se servisse una somma ulteriore, `dinheiro.rs` è l'unica aritmetica monetaria del crate |

---

## 5. Prove eseguite per questo documento

Tre sonde su **dati sintetici**, fuori dal repository, in
`…\scratchpad\sonda_3c4\`. Nessuna installazione, nessun download, nessun
archivio reale. Gli script sono `sonda_leitor.py`, `sonda_numeros.py`,
`sonda_escape.py` nella stessa cartella di scratchpad.

Strumenti usati, con provenienza misurata:

| Strumento | Versione | Provenienza |
|---|---|---|
| PowerShell + `System.IO.Compression` | .NET della macchina | scrive lo ZIP di prova |
| Python | 3.13.6 | `C:\Users\Recanto do Sossego\AppData\Local\Programs\Python\Python313\python.exe` — interprete dell'utente, **non** il working tree del repository Python |
| `openpyxl` | 3.1.5 | già presente in quell'interprete; eseguito con `py -I` per non caricare il percorso corrente |

### 5.1 Un XLSX non compresso, scritto a mano, è leggibile — [ESEGUITO]

Costruite cinque parti minime (`[Content_Types].xml`, `_rels/.rels`,
`xl/workbook.xml`, `xl/_rels/workbook.xml.rels`,
`xl/worksheets/sheet1.xml`), con celle `t="inlineStr"`, e impacchettate in uno
ZIP con **tutte le voci `stored`**, senza compressione.

| Misura | Esito |
|---|---|
| dimensione del file | 2882 byte |
| voci dello ZIP | 5, con `bruto == comprimido` per ognuna: nessun deflate |
| `openpyxl` apre il file | sì |
| `zipfile`: tutte le voci `ZIP_STORED` | sì |

Celle rilette, con il **tipo effettivo**:

| Cella | Scritta | Tipo riletto | Valore riletto |
|---|---|---|---|
| A2 | `00012345678901` | `s` | `00012345678901` — zeri iniziali conservati |
| B2 | `=1+1 SUCO "MARCA" Açucar` | `s` | identico; **non** una formula |
| C2 | `10.0000` | `s` | `10.0000` |
| D2 | `<v>10</v>` | `n` | `10` |
| A3 | `+55 11` | `s` | identico |
| B3 | `@linha` + LF + `com quebra` | `s` | il ritorno a capo sopravvive |
| C3 | `79228162514264337593543950335` | `s` | identico |
| celle di tipo formula in tutto il foglio | — | — | **nessuna** |

Conclusione **[ESEGUITO]**: un XLSX utile a questo export non richiede
compressione né una libreria. Il costo è scrivere lo ZIP a mano, §10.3.

### 5.2 Una cella numerica perde la scala — [ESEGUITO]

Le stesse cifre scritte due volte nello stesso foglio: come numero (`<v>`) e
come testo (`t="inlineStr"`).

| Scritto | Riletto dalla cella **numerica** | Riletto dalla cella **di testo** | Testo identico? |
|---|---|---|---|
| `10.0000` | `10.0` | `10.0000` | sì |
| `0.01` | `0.01` | `0.01` | sì |
| `-7.50` | `-7.5` | `-7.50` | sì |
| `12345678901234.56` | `12345678901234.56` | `12345678901234.56` | sì |
| `79228162514264337593543950335` | `79228162514264337593543950335` | `79228162514264337593543950335` | sì |
| `0.00` | `0.0` | `0.00` | sì |

Tre letture, da distinguere con cura:

1. **Misurato**: la cella numerica perde la **scala** in tre casi su sei
   (`10.0000`, `-7.50`, `0.00`). La scala è un dato che il codec dell'archivio
   conserva, ed è il riscontro `R3C3B1-01` dell'audit precedente: perderla
   nell'export sarebbe perdere lo stesso dato per un'altra via.
2. **Misurato**: la cella di testo riproduce i sei valori **identici**.
3. **[NON_VERIFICATO]**: che cosa fa **Excel** con `<v>79228162514264337593543950335</v>`.
   `openpyxl` restituisce l'intero intatto perché Python ha interi di
   precisione arbitraria; il modello numerico di Excel è un `double`
   IEEE-754 a 15 cifre significative, quindi la perdita è attesa ma **non
   provata qui**: non ho Excel e non lo deduco. La prova sta nel mandato
   implementativo, §11 `TX-18`.

Da qui la **[PROPOSTA] D20**: l'importo autorevole va in una cella di testo; la
colonna numerica di comodo esiste **accanto**, con un'intestazione che la
dichiara non autorevole. Non si sacrifica l'importo esatto per ottenere una
cella sommabile, e non si finge che una cella possa essere le due cose.

Anche l'Unicode è stato misurato per code point, non a video:

| Scritto | Code point riletti | Identico all'atteso? |
|---|---|---|
| `Açucar` in NFC | `41 e7 75 63 61 72` | sì |
| `Açucar` in NFD | `41 63 327 75 63 61 72` | sì |
| le due forme restano distinte | — | sì |

### 5.3 La convenzione `_xHHHH_` è una trappola, e dipende dal lettore — [ESEGUITO]

SpreadsheetML codifica i caratteri non rappresentabili come `_xHHHH_`. Un testo
che contenga **letteralmente** `_x0041_` può quindi essere decodificato come
`A`, e la difesa consueta è scrivere `_x005F_x0041_`.

| Scritto nel XML | Riletto da `openpyxl` 3.1.5 | Code point riletti |
|---|---|---|
| `_x0041_` | `_x0041_` | `5f 78 30 30 34 31 5f` |
| `_x005F_x0041_` | `_x005F_x0041_` | 13 code point, la «protezione» è rimasta nel dato |
| `_x0001_` | `_x0001_` | `5f 78 30 30 30 31 5f` |

Misurato: con questo lettore la decodifica **non avviene**, quindi scrivere il
testo così come è lo conserva, e «proteggerlo» lo **corrompe**.
**[NON_VERIFICATO]**: il comportamento di Excel, che la documentazione descrive
come decodificante.

**[PROPOSTA] D22**: non applicare la convenzione in scrittura, dichiarare il
limite nel foglio `limites` del file stesso, e inserire nel mandato
implementativo la prova `TX-19` con Excel. Rifiutata l'alternativa «proteggere
sempre», perché corromperebbe il dato con il solo lettore che possiamo
eseguire; rifiutata anche «modificare il testo originale per renderlo sicuro»,
che il mandato vieta esplicitamente.

Nota sul repertorio dei caratteri: i testi dell'archivio provengono da un
documento XML 1.0 letto con `quick-xml` (**[LETTO_NEL_CODICE]** `src/nfe.rs`) o
da costanti del progetto, quindi i caratteri di controllo che XML 1.0 vieta non
possono comparire. Non è una garanzia del writer: è una proprietà
dell'ingresso, e §7.4 la dichiara come tale.

### 5.4 Le sonde dell'audit, rieseguite sul codice reale — [ESEGUITO]

Dopo l'audit del 2026-10-23 ho riprodotto i cinque riscontri prima di
correggere. Crate **esterno** al repository, con dipendenza di percorso al
progetto, compilato `--offline`: `…\scratchpad\sonda_3c4_audit\`. Nessun file
del progetto modificato.

**`R3C4-01` — il `Dado` del bersaglio perde tre campi su quattro.** Due eventi
identici salvo `alvo.impressao_conteudo.motivo`, uno `None` e uno
`Some("motivo dichiarato")`.

| Misura | Esito |
|---|---|
| `Dado::novo` accetta entrambi | sì: **[LETTO_NEL_CODICE]** `modelos.rs:138` rifiuta un motivo vuoto e un motivo assente su un valore assente, **non** un motivo presente su un valore presente |
| `texto_exato_evento` li distingue | sì |
| le venti colonne proposte per `decisoes` | **righe identiche** |

Il disegno perdeva il dato. La correzione è in §6.3.

**`R3C4-02` — l'ingresso non è limitato al repertorio XML.** Un evento `Anotar`
con `autor` contenente U+0000 e U+0001 e una `nota` di 40000 caratteri:

| Misura | Esito |
|---|---|
| sopravvive a `serializar` + `desserializar` dell'archivio | sì, i due controlli sono nel testo riletto |
| `aplicar_revisao` lo accetta | sì: **[LETTO_NEL_CODICE]** `revisao.rs:1134` controlla solo che `autor` non sia vuoto dopo `trim`, e U+0000 non è spazio |
| la nota supera il limite di 32767 caratteri per cella | sì, 40000 |
| `quick_xml::Writer` emette i controlli grezzi | sì |
| un parser indipendente sul frammento (`xml.etree`, `minidom`) | **rifiutato**: «not well-formed (invalid token)» |
| lo stesso carattere scritto come riferimento numerico `&#0;` | **rifiutato**: «reference to invalid character number» |

Il secondo rifiuto è il fatto decisivo, e va oltre il rilievo: U+0000 **non è
rappresentabile in XML 1.0 in nessuna forma**, nemmeno escapato. L'unica strada
nel formato è la convenzione `_xHHHH_`, che §5.3 misura come dipendente dal
lettore. Da qui `EXP-18` riscritto e `EXP-33`/`EXP-34` in §7.4.

Il `Reader` di quick-xml **accetta** quel frammento: non è un validatore, e la
sua tolleranza non è una prova di validità. Per questo la misura usa due parser
esterni.

Controprova su CR, LF e tab: `BytesText::new` di quick-xml 0.42.0 emette
`a&#13;\nb\tc`, cioè protegge il CR e lascia LF e tab, e i due parser
indipendenti **accettano**. Su quel percorso non c'è difetto. Il CR letterale
sarebbe invece normalizzato dal parser, per XML 1.0 §2.11.

**`R3C4-05` — scope del piano.**

| Misura | Esito |
|---|---|
| archivio con `lote-A` su `2.01.004` e `lote-B` su `2.05.001`, piano ridotto con il solo `2.01.004`: `verificar_apresentacao` **globale** | `ERRORE`: «conta 2.05.001 do documento …6712345678901 ausente do plano recebido» |
| `aplicar_revisao` del **solo** `lote-A` con lo stesso piano ridotto | `Ok`, 1 riga |
| piano canonico con la sola `versao` cambiata in `9.9`: `verificar_apresentacao` | `Ok`, **0 divergenze**, con versione archiviata `1.0` e usata `9.9` |

Le due correzioni sono in §4.4 e §1.1.

**`R3C4-03` e `R3C4-04`** non hanno bisogno di una sonda: il primo è un buco
nella firma e nel protocollo scritti in §8 e §12.2, il secondo è una regola
assente in §7.2. Entrambi si leggono nel documento, e le correzioni sono lì.

### 5.5 I due residui dell'audit mirato, riprodotti — [ESEGUITO]

Stesso crate esterno, un secondo binario: `…\scratchpad\sonda_3c4_audit\src\bin\residui.rs`.

**Residuo `R3C4-01` — l'autore dell'approvazione attiva.** Tre casi con
`aplicar_revisao` reale, confrontando la regola «ultima approvazione» della
revisione 2 con quella corretta.

| Caso | Stato della riga | Decisioni in traccia | Avviso `APROVACAO_REPETIDA` | Regola «ultima» | Regola corretta |
|---|---|---|---|---|---|
| Alice approva 12, Bob ripete 13 | `APROVADO_POR_PESSOA` | 2 | sì | Bob, 13 | **Alice, 12** |
| le stesse, arrivo invertito | `APROVADO_POR_PESSOA` | 2 | sì | Bob, 13 | **Alice, 12** |
| Alice, Bob, Carla revoca, Dan, Eve | `APROVADO_POR_PESSOA` | 5 | sì | Eve, 16 | **Dan, 15** |

La riga resta nell'estratto in tutti e tre i casi, quindi l'attribuzione
sbagliata sarebbe finita nel file. Il secondo caso prova anche che la regola
corretta **non** dipende dall'ordine d'arrivo, perché `decisoes` è già in ordine
canonico. Correzione in §6.3.10.1.

**Residuo `R3C4-03` — il nome di un file che non esiste.** Su file sintetici,
con la sola libreria standard.

| Misura | Esito |
|---|---|
| `canonicalize` di `ARQUIVO.JSON`, **esistente** | riconduce a `arquivo.json`: la canonicalizzazione funziona sui file che ci sono |
| il confronto di famiglia **lessicale** della revisione 2, su `ARQUIVO.JSON.LOCK` **inesistente** | **accetta**: nessun rifiuto |
| scrittura su `ARQUIVO.JSON.LOCK` | `arquivo.json.lock` esiste e contiene quei byte |
| il JSON sorgente | invariato |
| lo stesso confronto, insensibile alle maiuscole | **rifiuta**, come deve |

Correzione in §8.1, `EXP-48`.

### 5.6 Che cosa queste sonde non dimostrano

Non dimostrano che il writer Rust scriverà celle corrette: nessuna riga di
writer esiste. Non dimostrano il comportamento di Excel, di LibreOffice o di
Google Sheets: nessuno dei tre è stato eseguito. Non dicono nulla su volumi
reali: il foglio di prova ha tre righe.

---

## 6. Schema — `FANTOZZI-EXPORT/v1` (D19, approvata)

### 6.1 Perché non il layout Python a 26 o 31 colonne

`F3C-19` registrava la differenza fra i due layout storici del Python. Questo
documento **non** la adotta, per tre motivi verificabili:

| Motivo | Evidenza |
|---|---|
| D11(a) rinvia l'import dello storico, quindi nessuna compatibilità di lettura è richiesta | mandato §3 |
| nessuno di quei due layout è stato provato su un file reale in questo porting | **[RIUSATO]** `SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md`: i workbook storici non sono stati forniti |
| il modello in memoria ha dati che quei layout non hanno: stato della revisione, provenienza delle decisioni, impronte, quattro esiti di riconciliazione, esclusioni motivate | §4.1 |

Un layout a colonna fissa costringerebbe a scegliere fra perdere quei dati e
affiancarli in colonne non dichiarate. Lo schema qui proposto parte dal
modello, e §6.4 mappa le differenze con il Python per chi dovesse confrontarli
a mano.

**Compatibilità realmente richiesta**: nessuna, in lettura. In scrittura, il
file deve aprirsi in un lettore di fogli di calcolo. È tutto.

### 6.2 Fogli

**Tredici fogli.** Ogni struttura annidata ripetuta ha un foglio proprio, con
la chiave che la lega alla riga padre: è l'alternativa a schiacciare un `Vec`
in una cella.

| # | Foglio | Una riga per | Lega a |
|---|---|---|---|
| 1 | `proveniencia` | il file: una sola riga di metadati, più una riga per divergenza di presentazione | — |
| 2 | `diagnostico` | riga del pre-libro, con lo stato della revisione | `lote`+`chave_origem`+`origem`+`conta_id`+`ordinal` |
| 3 | `rate` | `Duplicata` di un `Lancamento` | la chiave della riga |
| 4 | `documentos` | `DocumentoPreLivro`, con i quattro esiti | `lote`+`chave_origem` |
| 5 | `reconciliacao_componentes` | `Componente` di una riconciliazione | `lote`+`chave_origem` |
| 6 | `reconciliacao_campos_rinviati` | `CampoRinviato` | `lote`+`chave_origem` |
| 7 | `resumos` | `Resumo`, cioè `(empresa, competencia, conta)` | la propria chiave |
| 8 | `indicadores` | un indicatore: nome, valore, tipo | — |
| 9 | `extrato_revisado` | riga nell'estratto revisionato | la chiave della riga |
| 10 | `exclusoes` | `Exclusao`, dei due estratti, con la colonna `estrato` che dice quale | la chiave della riga |
| 11 | `decisoes` | `DecisaoAplicada` applicata, e `EventoNaoAplicado`, distinti dalla colonna `aplicado` | la chiave della riga, quando esiste |
| 12 | `avisos` | `Aviso`, con la colonna `fonte` che ne dichiara l'origine | `lote`+`chave_origem` quando c'è |
| 13 | `limites` | un limite dichiarato del file: una riga per limite | — |

### 6.3 Colonne

Riscontro `R3C4-01`: la versione precedente dichiarava tredici fogli e scriveva
le colonne di sei. Ordine e tipi delle colonne **sono** `FANTOZZI-EXPORT/v1`,
quindi lasciarli all'implementatore significava lasciargli il contratto. Qui
ci sono tutti e tredici.

#### 6.3.1 Convenzioni

| Convenzione | Regola |
|---|---|
| tipo di cella | `T` testo, `N` numero di comodo non autorevole, `D` data come testo ISO `AAAA-MM-GG`, `B` booleano come `SIM`/`NAO` |
| un `Dado<T>` | **quattro** colonne: `<campo>`, `<campo>_fonte`, `<campo>_status`, `<campo>_motivo`. Valore assente = cella vuota con `status` compilato. Vale per **ogni** `Dado`, anche annidato in un evento |
| un `Decimal` autorevole | colonna `T` con il testo esatto. Dove è prevista una cella numerica, tre colonne in più: `<campo>_numero` (`N`), `<campo>_numero_status`, `<campo>_numero_motivo`, secondo §7.2.1 |
| un identificativo numerico | `T`, per `EXP-41`: `arquivo_revisao`, `versao_schema` |
| un conteggio | `N`: dominio piccolo, somma sensata |
| un enum | il `rotulo()` del tipo: la stessa etichetta dell'archivio, una sola verità |
| una lista | **mai** una cella sola come unica rappresentazione: foglio figlio con la chiave del padre. Dove una cella concatenata è comoda per la lettura, è **in aggiunta**, con separatore `" | "` dichiarato |
| una lista vuota | nessuna riga nel foglio figlio, e la cella concatenata vuota. L'assenza di righe resta distinguibile da una riga con valori vuoti |
| ordine delle righe | quello deterministico di `prelivro` e `revisao`; mai l'ordine di arrivo |
| ordine delle colonne | fisso, quello scritto qui. Aggiungere, togliere o spostare una colonna richiede `FANTOZZI-EXPORT/v2` |

**Le due chiavi di collegamento**, nominate una volta perché i fogli figli le
usano senza riscriverle:

| Nome | Colonne | Identifica |
|---|---|---|
| **CHIAVE_LINHA** | `lote`, `chave_origem`, `origem`, `conta_id`, `ordinal` | una riga del pre-libro. `(lote, chave_origem)` **non** basta: un documento ha più righe |
| **CHIAVE_RESUMO** | `cnpj_empresa`, `competencia`, `conta_id` | un gruppo di riepilogo |

#### 6.3.2 Foglio `proveniencia`

Due blocchi, distinti da una prima colonna `tipo_linha`, perché un foglio con
due forme di riga va disambiguato invece di essere intuito:

| `tipo_linha` | Significato |
|---|---|
| `METADATO` | esattamente una riga, la prima |
| `DIVERGENCIA` | zero o più righe, una per `Divergencia` del lotto esportato |

Colonne, nell'ordine, valide per entrambi i blocchi; le celle non pertinenti al
tipo di riga restano **vuote**:

```text
tipo_linha,
formato, versao_schema,                                      (T, T)
gerado_em,                                                   (T, con offset)
arquivo_caminho, arquivo_revisao, arquivo_sha256,            (T, T, T)
lote_id, lote_gerado_em, lote_descricao,                     (T, D, T)
plano_arquivado_versao, plano_arquivado_sha256, plano_arquivado_caminho,
plano_usado_versao, plano_usado_caminho, plano_coincide,     (T, T, T)
gramatica_conteudo, gramatica_apresentacao, gramatica_evento,
eventos_no_arquivo, eventos_aplicados, eventos_nao_aplicados, (N, N, N)
documentos, linhas,                                           (N, N)
apresentacao_divergente,                                      (B)
div_chave_origem, div_impressao_salva, div_impressao_atual    (T, T, T)
```

Le tre colonne `div_*` sono vuote nella riga `METADATO`; tutte le altre sono
vuote nelle righe `DIVERGENCIA`, tranne `tipo_linha`.

`plano_arquivado_sha256` è un `Option<String>`: assente vuol dire cella vuota, e
la sua assenza è un dato di `IdentidadePlano`, non un errore.

#### 6.3.3 Foglio `diagnostico`

Una riga per riga del pre-libro, con lo stato della revisione accanto.

```text
# CHIAVE_LINHA
lote, chave_origem, origem, conta_id, ordinal,
# documento
numero, serie, data_documento, competencia,
cnpj_empresa, nome_empresa, empresa_identificada,
fornecedor_ou_favorecido, cnpj_cpf_favorecido,
documento, tipo_documento,
# conto
conta_nome, conta_nome_plano, grupo_conta, tipo_lancamento,
# contenuto
descricao_itens,
valor, valor_numero, valor_numero_status, valor_numero_motivo,
# stati, dalle tre fasi, senza confonderli
status_tecnico, estado_revisao, particao,
no_extrato_aprovado, no_extrato_revisado,
stato_documento,
motivos,                                     (cella concatenata, piu' foglio avisos)
# campi umani, conservati
alertas, observacoes, centro_custo,
dre_linha_id, dre_linha_nome,
# Dado: quattro colonne
data_pagamento, data_pagamento_fonte, data_pagamento_status, data_pagamento_motivo,
# provenienza
arquivo_origem, hash_registro,
impressao_conteudo, impressao_apresentacao,
# conteggi della revisione
decisoes_aplicadas, anotacoes_aplicadas,
# chiave del riepilogo che alimenta
resumo_cnpj_empresa, resumo_competencia, resumo_conta_id
```

Tipi: `ordinal`, `valor_numero`, `decisoes_aplicadas`, `anotacoes_aplicadas`
sono `N`; `data_documento` e `data_pagamento` sono `D`;
`empresa_identificada`, `no_extrato_aprovado`, `no_extrato_revisado` sono `B`;
tutto il resto è `T`.

#### 6.3.4 Foglio `rate`

Una riga per `Duplicata` di un `Lancamento`.

```text
# CHIAVE_LINHA
lote, chave_origem, origem, conta_id, ordinal,
# posizione nella lista, perche' due rate possono essere identiche
indice,                                                        (N)
numero,                                                        (T)
vencimento,                                                    (D, vuoto se None)
valor, valor_numero, valor_numero_status, valor_numero_motivo
```

`vencimento` è `Option<NaiveDate>`: cella vuota quando la NF-e non porta la
data. **[LETTO_NEL_CODICE]** `modelos.rs:435`, «`None` quando la NF-e non porta
la data: non si inventa una scadenza». Non esiste un `Dado` qui, quindi non
esistono le quattro colonne: l'assenza è nel tipo.

#### 6.3.5 Foglio `documentos`

```text
lote, chave_origem, numero, serie, data_documento, competencia,
cnpj_empresa, nome_empresa, empresa_identificada, arquivo_origem,
linhas, stato_documento,
valor_total_nfe, valor_total_nfe_numero, valor_total_nfe_numero_status,
    valor_total_nfe_numero_motivo,
soma_lancamentos, soma_lancamentos_numero, soma_lancamentos_numero_status,
    soma_lancamentos_numero_motivo,
# i quattro esiti, uno per colonna
esito_total, esito_rate, esito_copertura, esito_componenti,
# due Dado monetari: quattro colonne ciascuno, piu' la numerica del valore
soma_rate_documento, soma_rate_documento_fonte, soma_rate_documento_status,
    soma_rate_documento_motivo, soma_rate_documento_numero,
    soma_rate_documento_numero_status, soma_rate_documento_numero_motivo,
soma_rate_allocate, soma_rate_allocate_fonte, soma_rate_allocate_status,
    soma_rate_allocate_motivo, soma_rate_allocate_numero,
    soma_rate_allocate_numero_status, soma_rate_allocate_numero_motivo,
articoli_allocati,                                              (N)
articoli_attesi, articoli_attesi_fonte, articoli_attesi_status,
    articoli_attesi_motivo,
# conteggi delle liste, che hanno il loro foglio
articoli_scartati_qtd, avisos_documento_qtd, avisos_leitura_qtd,
componenti_qtd, campi_rinviati_qtd
```

Un documento **senza righe** ha la sua riga qui, con `linhas = 0`.

#### 6.3.6 Foglio `reconciliacao_componentes`

Una riga per `Componente`.

```text
lote, chave_origem, indice,                                     (T, T, N)
nome, ruolo, usada_na_verificacao,                              (T, T, B)
valor, valor_fonte, valor_status, valor_motivo,                 (Dado<Decimal>)
valor_numero, valor_numero_status, valor_numero_motivo
```

#### 6.3.7 Foglio `reconciliacao_campos_rinviati`

Una riga per `CampoRinviato`. I suoi quattro campi sono `String`, non un
`Dado`: **[LETTO_NEL_CODICE]** `alocacao.rs`, `CampoRinviato { campo, status,
fonte, motivo }`. Lo `status` è quindi un testo libero della fase 3B, non un
`StatusDado`, e si esporta come tale senza convertirlo.

```text
lote, chave_origem, indice, campo, status, fonte, motivo
```

#### 6.3.8 Foglio `resumos`

```text
# CHIAVE_RESUMO
cnpj_empresa, competencia, conta_id,
conta_nome, grupo_conta, empresa_identificada,
qtd_aprovado, qtd_ok, qtd_revisar, qtd_total,                   (N)
valor_aprovado, valor_ok, valor_revisar, valor_total,           (T, esatti)
valor_aprovado_numero, valor_ok_numero, valor_revisar_numero,
    valor_total_numero,                                         (N)
valor_numero_status, valor_numero_motivo,                       (uno per riga)
documentos_ok, documentos_parcial, documentos_erro,             (N)
documentos_marcados, nomes_divergentes                          (celle concatenate)
```

Le due colonne di status della numerica sono **una coppia per riga**, non una
per importo: l'eleggibilità di §7.2.1 si valuta sui quattro valori insieme, e
se uno solo non è eleggibile le quattro celle numeriche restano vuote con lo
stesso motivo. Mescolare quattro celle presenti e una assente in una riga di
totali sarebbe una riga che invita a sommare ciò che non è sommabile.

`documentos_marcados` e `nomes_divergentes` hanno anche le righe nel foglio
`avisos`, con la **CHIAVE_RESUMO** e le fonti `RESUMO_DOCUMENTO_MARCADO` e
`RESUMO_NOME_DIVERGENTE`.

#### 6.3.9 Foglio `indicadores`

Una riga per indicatore, invece di una riga larga sedici colonne: così
aggiungere un indicatore in `prelivro` non cambia lo schema.

```text
nome, valor, valor_numero, valor_numero_status, valor_numero_motivo, tipo
```

`nome` è il nome del campo di `Indicadores`; `valor` è sempre `T` ed esatto;
`tipo` è `CONTAGEM` o `MONETARIO`. Le sedici righe attese, nell'ordine dei
campi della struttura: `documentos`, `documentos_ok`, `documentos_parcial`,
`documentos_erro`, `documentos_sem_linhas`, `linhas`, `linhas_aprovado`,
`linhas_ok`, `linhas_revisar`, `valor_total`, `valor_aprovado`, `valor_ok`,
`valor_revisar`, `valor_extrato_aprovado`, `linhas_excluidas_do_extrato`.
Sono quindici: **[LETTO_NEL_CODICE]** `prelivro.rs:427` ne elenca quindici, e
il test `TX-01` le fissa una per una invece di contarle.

#### 6.3.10 Foglio `extrato_revisado`

Le righe che `VistaRevisada::extrato_revisado()` restituisce. Non duplica il
`diagnostico`: porta la chiave e le sole colonne che servono a leggere
l'estratto, perché chi lo consulta vuole un elenco e non un diagnostico.

```text
# CHIAVE_LINHA
lote, chave_origem, origem, conta_id, ordinal,
data_documento, competencia, cnpj_empresa, nome_empresa,
fornecedor_ou_favorecido, documento, conta_nome_plano, grupo_conta,
descricao_itens,
valor, valor_numero, valor_numero_status, valor_numero_motivo,
estado_revisao,
# chi ha APERTO l'approvazione attiva
aprovado_por, aprovado_em, aprovado_offset_segundos,
# le ripetizioni successive, con un nome proprio
confirmacoes,
ultima_confirmacao_por, ultima_confirmacao_em, ultima_confirmacao_offset_segundos
```

#### 6.3.10.1 Chi è l'autore dell'approvazione attiva — EXP-46

Residuo `R3C4-01` dell'audit mirato. La revisione 2 diceva «l'**ultima**
decisione di approvazione applicata». È sbagliato, e lo dice il codice:
**[LETTO_NEL_CODICE]** `revisao.rs:864-879`, una seconda `Aprovar` su una riga
già `AprovadoPorPessoa` finisce in traccia e produce l'avviso
`APROVACAO_REPETIDA` con il testo «a segunda aprovacao fica na trilha e nao
substitui a primeira». La regola «ultima» attribuirebbe l'approvazione a chi ha
soltanto ripetuto.

Misurato (§5.5):

| Scenario | Regola «ultima» | Chi ha davvero approvato |
|---|---|---|
| Alice approva alle 12, Bob ripete alle 13 | Bob, 13 | **Alice, 12** |
| le stesse due, in ordine d'arrivo invertito | Bob, 13 | **Alice, 12** |
| Alice approva, Bob ripete, Carla revoca, Dan approva, Eve ripete | Eve, 16 | **Dan, 15** |

**EXP-46** **[PROPOSTA]**: `aprovado_por`, `aprovado_em` e
`aprovado_offset_segundos` portano l'evento che **apre** l'approvazione
attualmente attiva, cioè la prima `Aprovar` dopo l'ultima `Revogar` efficace.
Si ricava scorrendo `LinhaRevisada::decisoes`, che è già in ordine canonico:

```text
aperta := nessuna
per ogni decisione in decisoes, in ordine canonico:
    Aprovar : se aperta e' nessuna, aperta := questa     (una ripetizione non sostituisce)
    Revogar : aperta := nessuna
    Anotar  : niente                                      (non decide lo stato)
```

Il risultato non dipende dall'ordine d'arrivo, perché `decisoes` è ordinata
canonicamente dalla 3C.3.a: misurato sulle due permutazioni.

**EXP-47** **[PROPOSTA]**: le ripetizioni hanno colonne proprie e un nome che
dice che cosa sono. `confirmacoes` conta le `Aprovar` applicate **dopo**
quella attiva; `ultima_confirmacao_*` porta la più recente, e le tre celle sono
vuote quando `confirmacoes = 0`. Così l'informazione non si perde e non si
traveste da approvazione.

Per una riga che non è nell'estratto revisionato il foglio non la contiene, e
il problema non si pone; per una riga `APROVACAO_REVOGADA` non c'è
approvazione attiva per definizione. Se un giorno servisse l'intera storia, è
il foglio `decisoes`: questo è una proiezione, e lo dichiara.

#### 6.3.11 Foglio `exclusoes`

Le `Exclusao` dei **due** estratti, distinte dalla prima colonna.

```text
estrato,                        (APROVADO | REVISADO)
# CHIAVE_LINHA
lote, chave_origem, origem, conta_id, ordinal,
particao, stato_documento,
valor, valor_numero, valor_numero_status, valor_numero_motivo,
motivo
```

`estrato = APROVADO` viene da `PreLivro::exclusoes_extrato`,
`estrato = REVISADO` da `VistaRevisada::exclusoes_extrato_revisado`. Le due
liste possono contenere la stessa riga per motivi diversi, e in quel caso
compaiono **due** volte: sono due affermazioni distinte, e fonderle ne
perderebbe una.

#### 6.3.12 Foglio `decisoes`

Decisioni applicate ed eventi non applicati, distinti dalla prima colonna. Il
`Dado` del bersaglio ha **quattro** colonne, correzione del riscontro
`R3C4-01`.

```text
aplicado,                                    (B)
motivo_nao_aplicacao, detalhe,               (T, vuoti quando aplicado = SIM)
# bersaglio dichiarato dall'evento: ChaveRevisao
alvo_chave_origem, alvo_origem, alvo_conta_id, alvo_ordinal,
alvo_impressao_conteudo, alvo_impressao_conteudo_fonte,
    alvo_impressao_conteudo_status, alvo_impressao_conteudo_motivo,
# la dichiarazione
operacao, autor,
instante_local, instante_offset_segundos,
motivo, nota, sequencia,
vista, vista_fonte, vista_status, vista_motivo,
repeticoes,                                  (N)
# collegamento alla riga, quando l'evento e' stato applicato
lote, chave_origem, origem, conta_id, ordinal
```

Le cinque colonne finali sono la **CHIAVE_LINHA** e sono vuote quando
`aplicado = NAO` con motivo `ALVO_AUSENTE`: l'evento non ha una riga a cui
legarsi, e inventarne una sarebbe falso. Le quattro colonne del bersaglio
restano sempre compilate, perché vengono dall'evento e non dalla riga.

`nota` e `sequencia` sono `Option`: cella vuota quando assenti. `nota` è
`Option<String>` e non un `Dado`, quindi non ha stato né motivo:
**[LETTO_NEL_CODICE]** `revisao.rs:145`.

#### 6.3.13 Foglio `avisos`

Tutti gli avvisi e i dettagli di lista, con la **fonte** che ne dichiara
l'origine e **la chiave del padre giusto** — correzione del riscontro
`R3C4-01`, che la chiave `(lote, chave_origem)` non era sufficiente.

```text
fonte,
# CHIAVE_LINHA, compilata solo per le fonti di riga
lote, chave_origem, origem, conta_id, ordinal,
# CHIAVE_RESUMO, compilata solo per le fonti di riepilogo
resumo_cnpj_empresa, resumo_competencia, resumo_conta_id,
indice,                                      (N, posizione nella lista d'origine)
codigo, detalhe
```

| `fonte` | Origine | Chiave compilata |
|---|---|---|
| `VISTA` | `VistaRevisada::avisos` | nessuna |
| `PRE_LIVRO` | `PreLivro::avisos` | nessuna |
| `LEITURA` | `DocumentoPreLivro::avisos_leitura` | `lote`, `chave_origem` |
| `DOCUMENTO` | `Reconciliacao::avisos_documento` | `lote`, `chave_origem` |
| `ARTIGO_DESCARTADO` | `Reconciliacao::articoli_scartati` | `lote`, `chave_origem` |
| `LINHA_MOTIVO` | `LinhaPreLivro::motivos` | **CHIAVE_LINHA** completa |
| `RESUMO_DOCUMENTO_MARCADO` | `Resumo::documentos_marcados` | **CHIAVE_RESUMO** |
| `RESUMO_NOME_DIVERGENTE` | `Resumo::nomes_divergentes` | **CHIAVE_RESUMO** |

Per le fonti che non sono `Aviso` ma stringhe — `LINHA_MOTIVO`,
`RESUMO_*` — `codigo` porta la `fonte` ripetuta e `detalhe` la stringa: non si
inventa un codice che il dato non ha.

#### 6.3.14 Foglio `limites`

```text
codigo, descricao, stato
```

`stato` è `DICHIARATO` o `NON_VERIFICATO`. Contenuto iniziale proposto, una
riga per voce: il file è un derivato e non rientra nell'archivio
(`DICHIARATO`); l'autore è dichiarato e non autenticato (`DICHIARATO`); le
colonne `_numero` non sono autorevoli e seguono §7.2.1 (`DICHIARATO`); un
testo che contenga letteralmente `_xHHHH_` può essere mostrato in modo diverso
da lettori diversi (`NON_VERIFICATO`); non esiste una sequenza storica dei
documenti, D17(a) (`DICHIARATO`); la durabilità dell'archivio dopo perdita di
alimentazione (`NON_VERIFICATO`); l'esclusione fra sorgente e destinazione non
copre hard link e volumi montati due volte (`NON_VERIFICATO`); con
`Substituir` concorrente la conservazione del precedente dipende dal lucchetto
cooperativo (`DICHIARATO`).

#### 6.3.15 Che cosa non viene esportato, e perché

Dichiarato per non lasciare dubbi su un'assenza:

| Struttura | Trattamento |
|---|---|
| `VistaRevisada::versoes` (`VersaoDocumento`) | le due impronte sono nel `diagnostico`; `sequencia` e `motivo` sono `Dado` `NAO_DISPONIVEL` per D17(a) e D8.5, quindi un foglio porterebbe solo celle vuote. **[PROPOSTA]**: non si esporta un foglio di assenze; il foglio `limites` dichiara che la sequenza non esiste |
| `Lancamento::duplicatas` dentro il `diagnostico` | non in cella: foglio `rate` |
| gli oggetti `PlanoContas` e `Arquivo` | non sono dati del lotto: dell'archivio restano revisione, impronta e percorso in `proveniencia` |

### 6.4 Mappa verso il Python, per chi confronta a mano

Non è una compatibilità: è una tabella di lettura. Le colonne del `Lancamento`
portano gli stessi nomi dei campi del porting, già allineati al Python in fase
3B (**[RIUSATO]**). Le differenze che un confronto incontrerebbe:

| Differenza | Dove |
|---|---|
| `valor` compare due volte, come testo e come numero | §6.3, D20 |
| `data_pagamento` occupa quattro colonne invece di una | convenzione `Dado<T>` |
| esistono `estado_revisao`, `no_extrato_revisado`, `impressao_*` | non hanno corrispondente nel Python |
| le rate non sono una cella JSON ma un foglio | foglio 3 |
| nessuna colonna di formula, nessun totale calcolato dal foglio | §7.5 |

---

## 7. Fedeltà dei dati: contratti `EXP-nn`

Tutti **[PROPOSTA]**. Riusano `F3C-16/17/18/20/21` e i contratti monetari
vigenti, senza toccare le grammatiche delle impronte: l'export è un formato di
presentazione e non entra in nessun digest.

### 7.1 Identità e struttura

| Id | Contratto | Caso che lo violerebbe | Prova |
|---|---|---|---|
| **EXP-01** | `FANTOZZI-EXPORT/v1` e `versao_schema` scritti nel foglio `proveniencia`; aggiungere, togliere o riordinare una colonna richiede `v2` | una colonna in più senza cambio di versione | `TX-01` |
| **EXP-02** | ogni riga del pre-libro compare una volta nel foglio `diagnostico`, con la chiave `(lote, chave_origem, origem, conta_id, ordinal)` | due righe con lo stesso `hash_registro` ridotte a una | `TX-03` |
| **EXP-03** | ogni documento compare nel foglio `documentos`, **anche senza righe** | documento con zero allocazioni assente dal file | `TX-02` |
| **EXP-04** | nessuna lista annidata viene schiacciata: rate, componenti e campi rinviati hanno un foglio con la chiave del padre | rate perse o concatenate in una cella senza foglio | `TX-07` |
| **EXP-05** | una lista vuota non produce righe, e l'assenza di righe resta distinguibile da una riga vuota | una riga di rate fittizia per un documento senza rate | `TX-07` |

### 7.2 Numeri

| Id | Contratto | Caso che lo violerebbe | Prova |
|---|---|---|---|
| **EXP-06** | ogni importo autorevole è una cella **di testo** con il testo esatto di `Decimal::to_string()`, scala compresa | `10.0000` scritto `10` | `TX-04` |
| **EXP-07** | nessun importo passa per `f64` nel writer, **con una sola eccezione nominata**: la scrittura della cella numerica di comodo (§7.2.3, D26(a)). Fuori da quella riga, nessuna conversione | una conversione a `f64` per formattare il testo autorevole, o in qualunque altro punto | ispezione con l'elenco di ogni occorrenza di `f64` e la sua giustificazione, più `TX-04` |
| **EXP-08** | la colonna `_numero` è di comodo e **dichiarata non autorevole**, ma non è arbitraria: §7.2.1 ne fissa la regola di presenza e di contenuto | una cella numerica presentata come l'importo; oppure una cella numerica con un valore qualunque | `TX-04` |
| **EXP-09** | nessun arrotondamento, nessuna correzione: quantità e prezzi non vengono portati a due decimali | `1.2345` scritto `1.23` | `TX-05` |
| **EXP-10** | i riepiloghi e gli indicatori sono copiati da `Resumo` e `Indicadores`; il writer non somma nulla, e un'eventuale somma ulteriore userebbe `dinheiro.rs` | un totale ricalcolato nel writer | `TX-06` |
| **EXP-11** | nessuna formula nel file; nessun totale delegato a Excel | una cella `=SUM(...)` | `TX-12` |

### 7.2.1 La regola della cella numerica di comodo

Riscontro `R3C4-04`. La versione precedente diceva che la colonna non esiste
«per gli importi la cui scala non sopravvive»: non è un algoritmo, non dice se
manca la cella o la colonna, e lasciava `TX-04` incapace di accorgersi di un
`valor_numero = 0` su ogni riga. «Non autorevole» non vuol dire «qualunque».

| Id | Regola **[PROPOSTA]** |
|---|---|
| **EXP-37** | la **colonna** c'è sempre: lo schema è fisso. Può mancare la **cella**, e l'assenza è dichiarata da due colonne affiancate, `<campo>_numero_status` e `<campo>_numero_motivo`, con la stessa convenzione dei `Dado` |
| **EXP-38** | criterio di eleggibilità, deterministico: la cella numerica si scrive **se e solo se** il decimale ha al massimo **15 cifre significative**, contate sulla mantissa dopo `normalize()`. Quindici è la precisione documentata di Excel. Altrimenti la cella è vuota, con `status = NAO_REPRESENTAVEL` e il motivo che riporta il numero di cifre |
| **EXP-39** | contenuto, dopo D26(a): la cella numerica è **numericamente uguale** all'importo autorevole, nel dominio eleggibile delle 15 cifre significative. **Nessuna promessa sulla scala**: `10.0000` può comparire come `10`, e la scala resta soltanto nella colonna testuale. Il writer non arrotonda e non riformatta di propria iniziativa: ciò che accade è la conversione a `f64` dell'eccezione nominata |
| **EXP-53** | l'eccezione di `f64` vive in **una sola funzione** di `xlsx.rs`, quella che scrive la cella numerica, ed è dichiarata nel codice con il motivo e il rinvio a D26(a). Nessun altro punto del writer converte |

Conseguenza verificabile, dopo D26(a): `TX-04` confronta in **due modi
distinti**, e la distinzione è il punto.

| Che cosa si confronta | Come |
|---|---|
| la colonna **testuale autorevole** | **esattamente**, carattere per carattere, con `Decimal::to_string()`, scala compresa |
| la **cella numerica** di comodo, dove è presente | per **valore**, nel dominio eleggibile: il numero riletto è uguale all'importo. Lo zero tecnico su una riga qualunque **fallisce**, e la scala **non** viene confrontata |
| la cella numerica dove è **assente** | `status = NAO_REPRESENTAVEL` e il motivo che riporta il numero di cifre |

Due esempi di cifre significative: `0.01` ne ha una ed è eleggibile;
`79228162514264337593543950335` ne ha ventinove e **non** lo è, quindi la sua
cella resta vuota con il motivo.

**Perché la soglia regge.** `f64` garantisce il round-trip di un decimale fino
a 15 cifre significative, e la documentazione della libreria dichiara la stessa
frontiera per `u64` (±999 999 999 999 999). Nel dominio eleggibile il valore
sopravvive; fuori, la cella non si scrive. La soglia era stata scelta prima di
sapere quale libreria si userebbe, e si è rivelata la stessa.

### 7.2.2 Identificativi numerici che non sono quantità

**EXP-41** **[PROPOSTA]**: `arquivo_revisao` è un `u64`, e il suo dominio
ammette valori oltre le 15 cifre significative di Excel. Non è una quantità da
sommare: è un identificativo, e per `EXP-12` gli identificativi sono **testo**.
La colonna `arquivo_revisao` di `proveniencia` diventa quindi `T` con il valore
esatto. Lo stesso vale per `versao_schema` e per i conteggi di `proveniencia`
che potrebbero crescere: dove un numero serve soltanto a essere letto, è testo.

I conteggi veri — `qtd_*`, `documentos_*`, `linhas` — restano `N`: sono
`usize` di dominio piccolo, e una somma su di essi ha senso. `TX-32` prova una
revisione oltre le quindici cifre.

### 7.2.3 R5-01 e la decisione D26(a): l'unica eccezione di `f64`

Riscontro `R5-01` del controllo mirato del 2026-10-23, verificato sulla
documentazione ufficiale — fonte ammessa dal mandato §6, e **letta, non
eseguita**.

| Fatto | Fonte |
|---|---|
| `Worksheet::write_number(&mut self, row, col, number: impl Into<f64>)` | `docs.rs/rust_xlsxwriter`, pagina di `Worksheet` |
| `write_number_with_format` ha la stessa firma | idem |
| `write(…, data: impl IntoExcelData)` accetta `u64` e `i64`, con perdita di precisione oltre ±999 999 999 999 999 | idem |
| con la funzionalità `rust_decimal`, `write` accetta un `Decimal` **solo se rappresentabile come `f64`** | idem |
| `write_string(…, string: impl Into<String>)` non passa per `f64` | idem |

Quindi: la colonna **testuale autorevole** è al sicuro, perché passa per
`write_string`. La **cella numerica di comodo** no: qualunque percorso numerico
della libreria passa per `f64`, e non esiste un'API che scriva le cifre del
`<v>` come testo.

Tre contratti attuali diventano perciò incompatibili con D18 + D20:

| Contratto | Che cosa dice oggi | Perché non regge |
|---|---|---|
| **EXP-07** | nessun importo attraverso `f64` **in nessun punto** del writer | il divieto è assoluto e la cella numerica lo violerebbe per costruzione |
| **EXP-39** | le cifre di `<v>` sono **identiche carattere per carattere** al testo autorevole | `f64` non conserva la scala: `10.0000` diventa `10` |
| **EXP-40** | il contenuto di `<v>` si ottiene dal testo, non da una conversione numerica | non è ottenibile con questa libreria |

Non è una proprietà «ancora da misurare»: è un conflitto che si legge nella
firma. La raccomandazione precedente su `rust_xlsxwriter` non ne aveva
dichiarato il costo.

**D26(a), approvata da Luca il 2026-10-23.** `f64` è consentito **soltanto**
nel confine di scrittura della cella numerica di comodo. L'alternativa —
rinunciare alla colonna, rivedendo D20 — è stata valutata e scartata.

| Contratto | Prima | Dopo D26(a) |
|---|---|---|
| `EXP-07` | divieto assoluto | divieto con **una** eccezione nominata e circoscritta, più l'obbligo di elencare ogni occorrenza di `f64` con la sua giustificazione |
| `EXP-39` | cifre di `<v>` identiche al testo | uguaglianza del **valore** nel dominio eleggibile; nessuna promessa sulla scala |
| `EXP-40` | il `<v>` si ottiene dal testo | **decaduto**: non è ottenibile con questa libreria |
| `EXP-53` | — | **nuovo**: l'eccezione vive in una sola funzione, dichiarata nel codice |
| `TX-04` | un solo confronto | **due** confronti distinti: esatto sul testo, per valore sulla cella numerica |

**Le cinque condizioni della decisione**, così come il controllo mirato le ha
formulate e come le riporto senza ammorbidirle:

1. `Decimal::to_string()` esatto, scala compresa, nella colonna **testuale
   autorevole**: invariato;
2. **nessun** `f64` nei calcoli, nelle riconciliazioni, nell'archivio, nelle
   impronte o nella produzione del testo autorevole: il nucleo monetario non si
   tocca;
3. eleggibilità di D20 invariata, massimo 15 cifre significative, assenza
   motivata per i valori non eleggibili, **nessuno zero sostitutivo**;
4. il test sul valore numerico riletto vive nel **dominio ammesso** ed è
   distinto dal confronto esatto del testo autorevole; **nessuna promessa** di
   conservare la scala nei byte della cella numerica;
5. **nessuna** modifica manuale di XML o ZIP per aggirare l'API della libreria.

Perché la soglia di 15 cifre regge anche qui: `f64` garantisce il
round-trip di un decimale fino a 15 cifre significative, e la documentazione
della libreria dichiara la stessa frontiera per `u64`
(±999 999 999 999 999). Il **valore** sopravvive nel dominio eleggibile; la
**scala** no, ed è ciò che la condizione 4 vieta di promettere.

Il riallineamento è **applicato**: §7.2.1 porta la nuova formulazione di
`EXP-39`, il nuovo `EXP-53` e la doppia forma di `TX-04`; §7.2 porta `EXP-07`
con l'eccezione; `EXP-40` è decaduto. Nessuno dei tre contratti resta sospeso.

**Che cosa l'eccezione non tocca**, e va riletto perché è la condizione 2:
calcoli, riconciliazioni, archivio, impronte e produzione del testo autorevole
restano senza `f64`. L'eccezione è una riga di scrittura di una cella
dichiarata non autorevole, e `EXP-53` la confina in una funzione sola.

### 7.3 Testi, identificativi, tempo

| Id | Contratto | Caso che lo violerebbe | Prova |
|---|---|---|---|
| **EXP-12** | CNPJ, CPF, chiavi, numero documento e `conta_id` sono testo; gli zeri iniziali restano | `00012345678901` scritto come numero | `TX-08` |
| **EXP-13** | un testo che inizia con `=`, `+`, `-`, `@`, o che contiene virgolette, ritorni a capo e Unicode, resta **il dato originale**: nessun apostrofo premesso, nessuna sostituzione | `=1+1 SUCO` trasformato in `'=1+1 SUCO` | `TX-09`, `TX-12` |
| **EXP-14** | la forma Unicode non viene normalizzata: NFC resta NFC, NFD resta NFD | NFD convertito in NFC «per pulizia» | `TX-10` |
| **EXP-15** | le date sono testo ISO `AAAA-MM-GG`; gli istanti della revisione vanno in **due** colonne, locale e offset in secondi | istante scritto come cella data di Excel | `TX-11` |
| **EXP-16** | un `Dado<T>` occupa quattro colonne; assenza, zero e stringa vuota restano distinti | assenza scritta come `0` o come cella vuota senza stato | `TX-13` |

### 7.4 Rappresentabilità: validazione al confine, non assunzioni sull'ingresso

Riscontro `R3C4-02`. La versione precedente di `EXP-18` affermava che il
repertorio dei caratteri dell'ingresso fosse già quello di XML 1.0, perché i
testi vengono da una NF-e. **È falso**, ed è misurato in §5.4: `autor`, `nota`
e `motivo` sono dichiarazioni **umane**, l'archivio le conserva come stringhe
JSON senza restringerle, e `aplicar_revisao` controlla soltanto che l'autore
non sia vuoto. Un `autor` con U+0000 attraversa l'intero flusso esistente.

L'export non può quindi assumere nulla sull'ingresso, e non può ripulirlo:
l'archivio resta come è, e le grammatiche delle impronte non si toccano. La
validazione sta **al confine dell'export**, ed è tecnica.

| Id | Contratto | Prova |
|---|---|---|
| **EXP-18** | il writer **valida la rappresentabilità** di ogni testo che riceve, prima di produrre byte. Nessuna assunzione sulla fonte: il controllo vale per i campi letti dalla NF-e, per quelli dichiarati da una persona e per quelli costruiti dal progetto | `TX-28`, `TX-29` |
| **EXP-33** | un testo che contiene un carattere **non rappresentabile in XML 1.0** — U+0000…U+0008, U+000B, U+000C, U+000E…U+001F — produce `ErroExport::TextoNaoRepresentavel { foglio, riga, colonna, codepoint, posizione }`, **prima** del punto di commit. Non viene troncato, sostituito, rimosso né convertito | `TX-29` |
| **EXP-34** | un testo più lungo del limite di cella del formato produce `ErroExport::TextoTroppoLungo { foglio, riga, colonna, caratteri, limite }`. **Nessun troncamento**: un dato tagliato in silenzio è peggio di un export che si rifiuta | `TX-30` |
| **EXP-35** | i limiti del formato dichiarati e controllati: 32767 caratteri per cella, 1 048 576 righe e 16 384 colonne per foglio, e la soglia oltre la quale servirebbe ZIP64 (4 GiB per voce o totale, 65 535 voci). Superarli è un errore nominato, non un comportamento indefinito | `TX-30`, `TX-31` |

Il limite di cella, di righe e di colonne è documentato da Microsoft nelle
specifiche e nei limiti di Excel; la soglia ZIP64 appartiene al formato ZIP. Le
due fonti sono citate come documentazione, e `TX-30`/`TX-31` le provano sul
nostro writer: **leggere una specifica non è eseguirla**.

**Perché un errore e non una codifica.** U+0000 non è rappresentabile in XML
1.0 **in nessuna forma**: §5.4 misura che anche `&#0;` è rifiutato come
«reference to invalid character number». L'unica strada nel formato sarebbe la
convenzione `_xHHHH_`, e §5.3 misura che il solo lettore eseguibile qui non la
decodifica: il dato apparirebbe come la stringa letterale `_x0000_`. Fra
produrre un file che mostra un dato sbagliato e rifiutare l'export con un
errore che nomina il campo e il carattere, il metodo impone il secondo. Non è
una regola nuova che vieta quel dato nel modello contabile: il dato resta
nell'archivio, ed è l'**export** a dichiararsi incapace di rappresentarlo.

### 7.4.1 La convenzione `_xHHHH_`, e la condizione posta su D22

**D22 è approvata nell'obiettivo**, con una condizione che cambia chi risponde
della codifica. Luca, 2026-10-23: conservare esattamente il testo e segnalare i
caratteri non rappresentabili; la codifica va verificata **con
`rust_xlsxwriter`**, senza nessuna trasformazione manuale aggiuntiva priva di
prova; un testo che contenga letteralmente `_x0041_` deve tornare **identico**
alla rilettura; la compatibilità Excel va verificata prima di dichiararla
stabile.

**EXP-17**, riformulato dopo la decisione: il writer **non applica nessuna
trasformazione propria** al testo. Lo passa alla libreria come è, e la codifica
sul file è quella che la libreria produce. Non aggiungiamo né togliamo
`_xHHHH_`, né l'underscore di protezione: §5.3 misura che su `openpyxl` la
protezione resterebbe nel valore, e `EXP-51` impone di misurare il percorso
reale invece di indovinarlo.

**EXP-51** **[PROPOSTA]**, la condizione di Luca resa verificabile:

| Requisito | Prova |
|---|---|
| un testo che contiene letteralmente `_x0041_` torna **identico** dalla rilettura indipendente del file prodotto da `rust_xlsxwriter` | `TX-37` |
| se il **testo riletto** differisse dall'originale, **non** si compensa con una trasformazione nostra: si riporta la misura a Luca e D22 si valuta prima di dichiarare la compatibilità | `TX-37`, con l'esito registrato come misura |
| la compatibilità Excel **non** si dichiara stabile prima di `TX-19` | `TX-19`, prova manuale |

**Che cosa conta come «trasformazione», con precisione.** Chiarimento del
controllo mirato del 2026-10-23, che non introduce una decisione nuova: la
tabella di §3.3 dice già l'obiettivo giusto, e qui la condizione operativa usa
lo stesso significato.

| Caso | Esito |
|---|---|
| la libreria usa una **codifica interna diversa** nel contenitore — per esempio protegge un carattere nel XML — e il **testo riletto è identico** all'originale | **conforme**. D22 non si riapre: la codifica dentro il pacchetto non è il dato |
| il **testo logico riletto** è diverso dall'originale | **prova fallita**: si registra la misura e si valuta la decisione prima di dichiarare la compatibilità. Nessuna compensazione silenziosa |

Conseguenza su `TX-37`: confronta il **testo letto** dallo strumento
indipendente con il testo originale. **Non** pretende che il XML dentro il
pacchetto sia identico al testo d'ingresso, perché quella non è la promessa.
`TX-19` resta necessaria per attestare il comportamento reale di Excel, che
nessuna lettura di documentazione può sostituire.

§7.4.3 conserva la tabella dei casi e dei lettori, perché descrive il **formato** e resta valida: cambia soltanto chi produce la codifica.

Che cosa **non** cambia con D18: `EXP-33` resta. Rifiutare un carattere non
rappresentabile non è una trasformazione, è un rifiuto, ed è esattamente ciò
che Luca ha approvato — «segnalare i caratteri non rappresentabili». Se la
libreria dovesse già rifiutarli per conto suo, la nostra validazione arriva
prima e produce un errore che nomina foglio, riga, colonna e codepoint: una
diagnostica migliore, non un doppione dannoso. `TX-29` misura quale dei due
interviene.

### 7.4.2 Il surrogato isolato non è raggiungibile da qui

La revisione 2 elencava i surrogati isolati fra i casi di `EXP-33` e ne
chiedeva una prova in `TX-29`. Non si può: una `String` di Rust è **UTF-8
valido per costruzione**, e un surrogato isolato non è UTF-8. Non esiste un
valore di `Lancamento::observacoes` o di `EventoRevisao::autor` che lo
contenga, e fabbricarne uno richiederebbe `unsafe` per costruire una `String`
invalida — cioè inventare il caso per poterlo provare.

**EXP-50** **[PROPOSTA]**: il surrogato isolato appartiene al **confine di
decodifica** che potesse riceverlo: un adattatore che legga byte non validati,
per esempio un ingresso grezzo di D12, o una lettura UTF-16 da un'altra fonte.
Là serve un `Dado` con stato e motivo, o un rifiuto, e là va provato. In
3C.4.1 è dichiarato **irraggiungibile attraverso l'API tipizzata**, e questa è
la ragione, non un'omissione.

### 7.4.3 Il dato logico, la codifica e i lettori

Tabella conservata dalla revisione 2, perché descrive il **formato** e resta
valida: cambia soltanto chi produce la codifica, che ora è la libreria.

| Caso | Dato logico | Che cosa arriva al file | Che cosa restituisce `openpyxl` 3.1.5 | Che cosa fa Excel |
|---|---|---|---|---|
| testo ordinario | `SUCO` | `SUCO` | `SUCO` | atteso identico |
| testo con un carattere non rappresentabile | `Jos<U+0000>e` | **niente**: `EXP-33`, errore prima del commit | — | — |
| testo che contiene letteralmente `_x0041_` | `_x0041_` | ciò che `rust_xlsxwriter` scrive: **[NON_VERIFICATO]**, `TX-37` | `_x0041_` quando il file lo contiene così, **misurato** su un pacchetto scritto a mano | **[NON_VERIFICATO]**: la documentazione di `ST_Xstring` descrive la decodifica dell'underscore iniziale, quindi Excel potrebbe mostrare `A` |

La frase «proteggere corrompe il dato», della revisione 1, era imprecisa:
**con `openpyxl` 3.1.5** la protezione resta nel valore riletto, e questa è una
misura su quel lettore, non un giudizio sulla codifica in sé. Con un lettore che
decodifica, la protezione sarebbe corretta. È questa dipendenza dal lettore che
rende necessaria la condizione di Luca: misurare il percorso reale, non
dedurlo.

**EXP-36**: finché `TX-19` non è eseguita su Excel, la compatibilità del
formato **non** si dichiara stabile, e il foglio `limites` del file lo scrive:
un testo che contenga letteralmente `_xHHHH_` può essere mostrato in modo
diverso da lettori diversi. Che cosa impedisce di dichiararla stabile prima di
quella prova: non possediamo la misura su Excel, e l'unica alternativa sarebbe
dedurla dalla documentazione, cioè presentare una lettura come un'esecuzione.

Se `TX-19` o `TX-37` mostrassero una trasformazione, D22 va **riaperta** con
tre strade: accettare la codifica della libreria e documentarla; trattare un
`_xHHHH_` letterale come non rappresentabile, con l'errore di `EXP-33`; oppure
cambiare libreria. La scelta sarebbe di Luca, e la condizione che ha posto su
D22 serve esattamente a non farla prendere all'implementatore.

### 7.5 Nessuna strada di ritorno

**EXP-19**: il writer non apre, non legge e non modifica l'archivio sorgente.
Riceve una `VistaRevisada` e una `Proveniencia` già costruite. Una modifica al
file derivato non è una decisione umana e non ha percorso di rientro: il foglio
`limites` lo dichiara, e il mandato 3C.5 o un blocco successivo dovrà decidere
se un rientro deve esistere.

---

## 8. Protocollo di scrittura

Riusa la forma già provata in `arquivo_io` (**[RIUSATO]** §9.1 della specifica
di persistenza, prove `TP-13`). La versione precedente di questa sezione
diceva che «non serve un lucchetto perché non c'è un contatore»: riscontro
`R3C4-03`, quell'argomento non dimostra né l'atomicità di `Recusar` né la
conservazione del precedente, e non protegge il sorgente.

Protocollo **dopo le decisioni D24 (lucchetto sì) e D25(b) (`Recusar` senza
rivendicazione)**:

```text
 0. ESCLUSIONE confronta destinazione e sorgente in forma canonica (EXP-42)
               coincidono, o la destinazione cade nella famiglia di nomi
               dell'archivio -> DestinoECoincideComFonte, nessun file toccato
 1. VALIDA     il modello e i testi; rappresentabilita' e limiti (EXP-18/33/35)
 2. LUCCHETTO  <nome>.export.lock con create_new                   (EXP-44)
               esiste -> ExportOcupado con il contenuto dichiarato
 3. ESISTENZA  se `Recusar` e la destinazione esiste -> DestinoJaExistente,
               senza toccare niente                                (EXP-43b)
 4. TEMPORANEO <nome>.novo.<etichetta> con create_new, write_all, sync_all
               esiste un residuo -> ResiduoPresente, esplicito
 5. PRECEDENTE se `Substituir` e la destinazione esiste:
               copia in <nome>.anterior.<etichetta>, mai sovrascritta
 6. PUBBLICA   fs::rename(temporaneo, destinazione)    <== PUNTO DI COMMIT
 7. RILASCIO   rimozione del lucchetto; un fallimento qui non annulla
               la pubblicazione, ed e' un campo dell'esito
 8. ESITO      percorso, byte, impronta SHA-256, precedente conservato,
               lucchetto residuo
```

Il passo 3 **non crea** il file: lo controlla. Nessuna destinazione di zero
byte può restare, e `EXP-49` decade con la rivendicazione che lo produceva.

| Id | Contratto | Prova |
|---|---|---|
| **EXP-20** | un solo punto di commit; qualunque errore prima del passo 6 lascia il file preesistente **invariato byte per byte** | `TX-14` |
| **EXP-21** | il temporaneo si crea con `create_new`: non tronca un residuo. Un residuo è un errore nominato, e la libreria **non** lo cancella | `TX-15` |
| **EXP-22** | la destinazione è un parametro esplicito del chiamante: il modulo non sceglie percorsi e non legge variabili d'ambiente | ispezione più `TX-16` |
| **EXP-23** | con una destinazione già esistente il comportamento è dichiarato dal chiamante: `Recusar` oppure `Substituir` conservando il precedente al passo 5. Nessun terzo comportamento implicito | `TX-16` |
| **EXP-24** | l'archivio sorgente non viene aperto in scrittura in nessun momento: l'impronta del file dell'archivio prima e dopo l'export è la stessa | `TX-17` |

### 8.1 L'esclusione del sorgente, con i dati per farla — EXP-42

Riscontro `R3C4-03`, parte P1. L'assenza di chiamate ad `arquivo_io::publicar`
**non** garantisce che l'archivio non cambi: `fs::rename` di un temporaneo
sulla destinazione sostituisce qualunque file, archivio compreso, se la
destinazione coincide col sorgente. Il protocollo precedente non aveva il dato
per accorgersene, perché `escrever` non riceveva l'identità del sorgente.

**EXP-42** **[PROPOSTA]**: `escrever` riceve la `FonteProtegida` e rifiuta
**prima** di creare qualunque file, al passo 0:

| Che cosa si rifiuta | Come si confronta |
|---|---|
| destinazione uguale al file dell'archivio | forma canonica di entrambi |
| destinazione nella **famiglia di nomi** dell'archivio: `<nome>`, `<nome>.lock`, `<nome>.novo.*`, `<nome>.ger*`, `<nome>.corrupto*`, `<nome>.recuperado.*` | directory canonica uguale **e** nome che cade nella famiglia |

Confronto in forma canonica: `fs::canonicalize` su entrambi i percorsi. Quando
la destinazione non esiste ancora, si canonicalizza la **directory padre** e si
confronta il **nome** del file. Questo copre separatori ripetuti, `.` e `..`,
nomi 8.3, prefisso `\\?\`, collegamenti e junction sui percorsi **intermedi**.

**EXP-48** **[PROPOSTA]**, residuo `R3C4-03` dell'audit mirato: il confronto
del nome **non** può essere lessicale. La revisione 2 attribuiva a
`fs::canonicalize(parent)` anche l'insensibilità alle maiuscole del nome del
file: è falso per un nome che **non esiste ancora**, perché quel nome arriva dal
chiamante e nessuna canonicalizzazione lo tocca. Misurato (§5.5):

| Misura | Esito |
|---|---|
| `canonicalize` di `ARQUIVO.JSON` **esistente** | riconduce a `arquivo.json` |
| il confronto lessicale della revisione 2 su `ARQUIVO.JSON.LOCK` **inesistente** | **accetta**: nessun rifiuto |
| scrivendo su `ARQUIVO.JSON.LOCK` | `arquivo.json.lock` esiste e contiene quei byte |
| il JSON sorgente | invariato |
| un confronto insensibile alle maiuscole | **rifiuterebbe**, come deve |

Il sorgente non viene sostituito, ma si occupa il nome del **lucchetto**
dell'archivio con byte XLSX, o si introduce un falso candidato o una falsa
generazione. `arquivo_io::candidatos` li elencherebbe e `recuperar` li
valuterebbe: non è cosmetico, ed è esattamente ciò che `EXP-42` prometteva di
impedire.

**La regola.** Si costruisce una **chiave di confronto** dal nome: la stessa
trasformazione applicata al nome della destinazione, al nome dell'archivio e ai
prefissi di famiglia, insensibile alle maiuscole secondo la semantica del
filesystem supportato. I nomi originali **non** vengono riscritti: la chiave
serve solo al confronto, e ciò che si memorizza e si scrive nei messaggi è il
nome come il chiamante l'ha dato.

Famiglia da rifiutare, con la chiave: `<nome>` esatto, e i prefissi
`<nome>.`, che coprono `<nome>.lock`, `<nome>.novo.*`, `<nome>.ger*`,
`<nome>.corrupto*`, `<nome>.recuperado.*` e qualunque nome futuro della stessa
famiglia. Un prefisso è più robusto di un elenco: un nome nuovo introdotto da
un blocco successivo resta protetto senza che nessuno si ricordi di
aggiungerlo qui.

**Direzione dell'errore, dichiarata.** Il confronto deve sbagliare per
**eccesso**: rifiutare una destinazione che non collide è un fastidio, lasciarne
passare una che collide è una perdita. Per questo la regola non tenta di
indovinare se la directory sia stata marcata come sensibile alle maiuscole —
NTFS lo permette per singola directory — e rifiuta comunque.

**Limiti dichiarati, [NON_VERIFICATO]**: due percorsi che la
canonicalizzazione **non** riconduce allo stesso nome e puntano comunque agli
stessi byte — un hard link al file dell'archivio, o lo stesso volume montato
due volte — sfuggono al controllo. E la piegatura delle maiuscole fuori
dall'ASCII dipende dalle tabelle del sistema: la chiave di confronto la applica
con gli strumenti della libreria standard, e un nome di archivio con caratteri
non ASCII non è stato provato. Non invento una difesa che non posso provare:
i limiti vanno nel foglio `limites`, e `TX-33` ne registra il comportamento
reale come misura.

### 8.2 `Recusar` e `Substituir` dopo D24 e D25(b)

Riscontro `R3C4-03`, parte P2: un `exists()` seguito da `rename` **non è
atomico**. Due esecuzioni che osservano l'assenza pubblicano entrambe, e la
seconda sostituisce la prima senza che nessuna se ne accorga.

Le due risposte possibili erano la rivendicazione del nome finale, atomica ma
con un residuo a zero byte, e il controllo di esistenza, senza residui ma non
atomico. **Luca ha deciso il 2026-10-23: D24 sì, D25(b).** Lucchetto
cooperativo, controllo di esistenza, nessuna rivendicazione.

**EXP-43b**, al posto di `EXP-43`: al passo 3, con `Recusar`, si **controlla**
l'esistenza della destinazione e si risponde `DestinoJaExistente` senza toccare
nulla. Il passo non crea file, quindi nessuna destinazione di zero byte può
restare: `EXP-49` e `ReivindicacaoAbandonada` **decadono** con la
rivendicazione che li rendeva necessari.

**EXP-44**, mutua esclusione fra scrittori cooperanti: un lucchetto
`<nome>.export.lock` preso al passo 2 con `create_new` e rilasciato al passo 7,
con i metadati dichiarati dal chiamante come in `arquivo_io`. Questo è ciò che
rende sensata la conservazione del precedente con `Substituir`: senza di esso,
due `Substituir` intercalati possono conservare due copie dello stesso vecchio
file e perdere un intermedio. **[RIUSATO]** §3.6 della specifica di persistenza
misura `create_new` come atomico su Windows: il secondo tentativo riceve
`AlreadyExists`, e qui diventa `ExportOcupado`.

**Il limite che la combinazione scelta porta, dichiarato.**

| Scenario | Comportamento |
|---|---|
| due esecuzioni **cooperanti**, cioè due usi di questa API | serializzate dal lucchetto: la seconda riceve `ExportOcupado` durante, e `DestinoJaExistente` se arriva dopo il rilascio e la destinazione esiste |
| un'esecuzione che **ignora** il lucchetto, o un altro programma | non è impedita: `Recusar` non è atomico, e due pubblicazioni possono sovrapporsi con la seconda che vince in silenzio |

**EXP-52** **[PROPOSTA]**: il limite di non atomicità di `Recusar` va scritto
nel foglio `limites` del file prodotto, non soltanto qui. È il prezzo di
D25(b), e chi legge il file deve poterlo sapere senza leggere questa
specifica.

Nessuna di queste scelte introduce account, ruoli o autenticazione: il
perimetro multiutente resta fuori progetto. Il lucchetto coordina **esecuzioni**
dello stesso operatore — un comando lanciato due volte, un export ripetuto
mentre il primo non è finito — che è lo scenario realistico di un primo export
monoutente.

### 8.3 Le etichette, che devono stare nella firma

**EXP-45** **[PROPOSTA]**: l'`etichetta` che compare nei nomi di temporaneo e
precedente è un **parametro del chiamante**, non un valore che il modulo
inventa da un orologio che non legge. Entra nella firma di `escrever`, insieme
alla destinazione, al pacchetto, alla politica e alla fonte protetta. Due
export nella stessa directory con la stessa etichetta si fermano al passo 4 con
`ResiduoPresente`, che è il comportamento voluto: due esecuzioni diverse
devono avere etichette diverse, e chi le sceglie è chi sa perché sta
esportando.

**File di destinazione aperto da un altro programma.** Su Windows il `rename`
sostituisce una destinazione esistente anche tenuta da un handle aperto da
Rust, ma un handle con `FileShare.None` la rifiuta: entrambi i fatti sono
**[RIUSATO]** dalla specifica di persistenza §3.6. Un file aperto in Excel
ricade nel secondo caso in modo **[NON_VERIFICATO]**: la prova `TX-16b` sta nel
mandato implementativo, e il contratto è che l'errore sia nominato e il
precedente resti intatto, non che la sostituzione riesca.

---

## 9. Determinismo — D23, approvata

Da distinguere in tre livelli, perché promettere il livello sbagliato sarebbe
una promessa non mantenibile.

| Livello | Promessa | Perché |
|---|---|---|
| **contenuto** | due esecuzioni dallo stesso archivio, stesso lotto e stesso piano producono gli **stessi valori nelle stesse celle** | **EXP-25**, prova `TX-20` |
| **ordine** | l'ordine delle righe di ogni foglio è quello deterministico già garantito da `prelivro` e `revisao`, non l'ordine di arrivo | **EXP-26**; **[RIUSATO]** `I7` della 3C.2 e l'ordine canonico della 3C.3.a |
| **byte** | **non promesso**. Un contenitore ZIP porta metadati per voce, fra cui la data di modifica, e due scritture possono differire nei byte pur avendo contenuto identico | `[NON_VERIFICATO]` per definizione, se non si fissa anche quella |

**EXP-27** **[PROPOSTA]**: se un giorno servisse l'identità dei byte, va
dichiarato un contratto sui metadati dello ZIP — data fissa per ogni voce,
ordine delle voci, nessun campo extra — e provato. Finché quel contratto non
esiste, il file non si confronta per hash fra due esecuzioni, e l'unico
confronto valido è quello del contenuto. L'impronta SHA-256 restituita al passo
5 di §8 identifica **quel** file, e non è un'attestazione di riproducibilità.

**La quantificazione di `EXP-25`, detta con precisione.** `gerado_em` è un
**parametro** del chiamante, non una lettura dell'orologio: due esecuzioni con
la **stessa** `Proveniencia` producono quindi celle identiche, `gerado_em`
compresa. Non cambia «per costruzione»: cambia se il chiamante gli passa un
valore diverso.

Quindi: **EXP-25** quantifica su *tutti* gli ingressi — archivio, lotto, piano
e `Proveniencia` — e promette l'identità di **tutte** le celle. **EXP-28**
riguarda il solo caso in cui il chiamante faccia variare `gerado_em` fra due
esecuzioni di confronto: allora quella cella si esclude, e l'esclusione è
dichiarata nel test invece di essere ottenuta congelando un orologio che il
modulo non legge.

---

## 10. Dipendenze: stato misurato e dipendenza scelta

### 10.1 Che cosa c'è davvero — [ESEGUITO]

Misurato alla baseline, non ereditato da una specifica precedente:

| Fatto | Esito |
|---|---|
| crate in `[dependencies]` | `chrono`, `clap`, `quick-xml`, `rust_decimal`, `serde`, `serde_json`, `serde_yaml`, `sha2`, `unicode-normalization` |
| crate XLSX nella cache locale (`rust_xlsxwriter`, `umya-spreadsheet`, `simple_excel_writer`, `xlsxwriter`, `calamine`) | **nessuno** |
| crate ZIP o di compressione nella cache (`zip`, `flate2`, `miniz_oxide`, `zstd`, `crc32fast`) | **nessuno** |
| voci totali nella cache del registro | 103 |
| `zip`/`flate2` in `Cargo.lock` | assenti |
| `cargo metadata --offline` | riesce: il lockfile attuale è risolvibile senza rete |
| `quick-xml` sa **scrivere** XML | sì: `Writer`, `create_element`, `write_event` in `…/quick-xml-0.42.0/src/writer.rs:109,213,262` |

Conclusione **[ESEGUITO]**: la vecchia frase «i crate XLSX non sono
disponibili» resta vera oggi, e per una ragione in più: non c'è nemmeno un
crate ZIP. Qualunque libreria XLSX richiederebbe un **download**, che questo
mandato non autorizza e che il mandato implementativo dovrebbe autorizzare
esplicitamente.

### 10.2 Il lettore indipendente non richiede nulla — [ESEGUITO]

Con D11(a) non serve un lettore XLSX in Rust. Per la prova di fedeltà
(`F3C-21`) è disponibile e misurato: Python 3.13.6 con `openpyxl` 3.1.5
nell'interprete dell'utente, eseguibile con `py -I`, §5. Provenienza
dichiarata, nessuna installazione.

**EXP-29**, **approvata da Luca il 2026-10-23**: la prova di rilettura
indipendente è uno **script esterno** al crate, non un test `cargo`, perché
dipende da un interprete che non è una dipendenza del progetto. Usa Python e
`openpyxl` **già disponibili**, **senza installazioni**.

| Requisito | Contenuto |
|---|---|
| ripetibile | percorso dichiarato, invocazione scritta nel README e nel rapporto del lotto, nessun passaggio a vista |
| esito controllato | codice di uscita diverso da zero in caso di discordanza, così che una differenza non possa passare inosservata |
| tracciabile | il rapporto nomina versione e provenienza dell'interprete e del lettore, come in §5 di questo documento |
| confinato | `openpyxl` **non** diventa una dipendenza del progetto, non entra in `Cargo.toml`, non viene installato e **non** introduce un runtime Python nel programma Rust |

È una decisione **organizzativa** su dove vive una prova, e non
un'approvazione di pacchetti. Il controllo mirato del 2026-10-23 aveva
consigliato questa forma; Luca l'ha approvata.

La strada alternativa — rinunciare alla rilettura indipendente finché un
lettore Rust non sia autorizzato, perdendo `F3C-21` nel primo lotto — è stata
quindi scartata.


### 10.3 Le tre strade pesate, e quella scelta

| Strada | Dipendenze nuove | Costo | Rischi |
|---|---|---|---|
| **(a) XLSX scritto a mano**: `quick-xml` per le parti XML, ZIP con voci `stored` e CRC-32 scritto nel progetto | **nessuna** | stimato 500–700 righe fra writer ZIP, parti del pacchetto e contratti; **[PROPOSTA]**, la stima non è misurata | il formato ZIP va implementato correttamente: intestazioni locali, directory centrale, offset, CRC-32. Il limite ZIP64 (4 GiB, 65535 voci) è lontano ma va dichiarato |
| **(b) `rust_xlsxwriter`** | una, da **scaricare** | basso in righe | dipendenza da autorizzare e scaricare; porta con sé un writer ZIP e la compressione; superficie molto più ampia di ciò che serve |
| **(c) solo CSV** | nessuna | il più basso | Excel reinterpreta all'apertura: un CSV non controlla il tipo di cella. Quoting e separatori non impediscono la reinterpretazione di identificativi e testi |

**D18, decisa da Luca il 2026-10-23: strada (b), `rust_xlsxwriter`.** Niente
writer ZIP o XLSX scritto a mano.

Che cosa resta valido delle misure di §5, e che cosa decade:

| Misura | Stato dopo la decisione |
|---|---|
| §5.2, il **controllo del tipo di cella** è indispensabile per `EXP-06` e `EXP-13`, e solo XLSX lo dà | **resta** il motivo che esclude la strada (c): vale qualunque writer si usi |
| §5.1, un XLSX con voci `stored` scritto a mano è leggibile | **resta come fatto sul formato**, e non è più la giustificazione di una strada: la (a) non è stata scelta |
| la stima di 500–700 righe | **decade**: non c'è nulla da scrivere a mano |
| l'argomento «il crate non è in cache» | **decade**, ed era giusto escluderlo: non era un requisito funzionale, ed è il costo d'ingresso che la decisione accetta |

Conseguenza operativa, circoscritta con precisione dopo il controllo mirato: la
rete serve **una volta**, per acquisire e risolvere la dipendenza mancante, e
il mandato implementativo deve autorizzare quell'acquisizione. Dopo di essa le
compilazioni successive usano la cache locale, come oggi accade per gli altri
nove crate.

Due cose che **non** diventano vere, e che la formulazione precedente
lasciava credere: il programma esportatore **non** ha un requisito di rete a
tempo di esecuzione, e il lotto non diventa permanentemente non eseguibile
offline. Cambia soltanto il primo passo.

### 10.4 Che cosa resta `NON_VERIFICATO` sul formato

| Affermazione | Prova rinviata al mandato implementativo |
|---|---|
| Microsoft Excel apre il file prodotto da `rust_xlsxwriter` con il nostro contenuto | `TX-18`: aprire il file in Excel e confrontare dieci celle note |
| Excel decodifica `_xHHHH_` | `TX-19` |
| Excel conserva un intero di 29 cifre in una cella numerica | `TX-18`; l'attesa è che **non** lo conservi, e per questo `EXP-06` esiste |
| LibreOffice e Google Sheets | nessuna prova pianificata: non sono un requisito dichiarato |

### 10.5 Che cosa il mandato implementativo deve registrare sulla dipendenza

Luca ha chiesto di indicare versione, funzionalità abilitate e dipendenze
aggiunte. **Non si possono fissare qui**: il crate non è in cache (§10.1) e
questo lotto non autorizza download, quindi qualunque numero scriverei sarebbe
una lettura di documentazione presentata come una verifica. Ecco invece che
cosa il mandato deve contenere, e come ottenerlo.

| Voce | Come si stabilisce |
|---|---|
| nome e **versione esatta** | risolta con la rete disponibile e scritta in `Cargo.toml` con la versione puntuale, come per `serde_json = "1.0.154"` |
| funzionalità | `default-features = false` più l'insieme **minimo** che serve, elencato una per una con il motivo. Nessuna funzionalità «perché c'era» |
| dipendenze transitive | l'elenco completo dal diff di `Cargo.lock`, **spiegato riga per riga**, come è stato fatto per `serde_json` |
| licenze | verificate e dichiarate, perché entrano nel progetto |

Quattro proprietà che vanno **misurate** prima di fidarsi, e che il mandato
deve trasformare in prove. Sono tutte `[NON_VERIFICATO]`: non ho eseguito la
libreria, e leggerne l'API non è eseguirla.

| Proprietà richiesta | Perché serve | Prova |
|---|---|---|
| scrive in un **buffer in memoria**, non solo su file | `xlsx.rs` deve restare puro e `export_io.rs` l'unico a toccare il disco (§12.2) | `TX-38` |
| una cella di testo resta **testo**: un valore che inizia con `=`, `+`, `-`, `@` non diventa una formula | `EXP-13`; nessuna delle tre strade è utile se la libreria interpreta | `TX-09`, `TX-12` |
| una stringa non viene convertita in numero: la colonna autorevole passa per `write_string` | `EXP-06` | `TX-04` |
| la cella numerica passa per `f64` **per costruzione** della libreria: si misura che il valore riletto sia uguale nel dominio eleggibile, non che la scala sopravviva | `EXP-39`, D26(a) | `TX-04` |
| che cosa fa la libreria con `_xHHHH_` e con i caratteri non rappresentabili | la condizione di Luca su D22: §7.4.3 | `TX-37`, `TX-19` |

Se una di queste quattro non reggesse, non si aggira con una trasformazione
nostra: si torna a Luca con la misura. È la condizione che ha posto su D22, e
vale per tutte e quattro.

**Perché non anche un CSV.** Il mandato chiede di non commissionare due writer
per inerzia. Un CSV aggiungerebbe un formato il cui contratto non può garantire
`EXP-12` e `EXP-13`: **[RIUSATO]** la contro-prova §1.4 della specifica 3C
mostra che un testo che inizia con `=` sopravvive nel file CSV ma non
all'apertura in Excel. Se un CSV servisse a un consumo automatico, è un lotto a
sé con un contratto proprio: non un secondo formato «gratis».


---

## 11. Matrice delle prove

Tutte **PIANIFICATE**: nessuna è stata eseguita, perché il writer non esiste.
Le sonde di §5 sono separate e non contano come test del prodotto.

| Prova | Contratto | Caso | Atteso | Blocco |
|---|---|---|---|---|
| **TX-01** | EXP-01 | file generato e riletto | `formato` e `versao_schema` presenti nel foglio `proveniencia`; l'ordine delle colonne di ogni foglio coincide con §6.3, verificato colonna per colonna contro una lista fissata nel test | 3C.4.1 |
| **TX-02** | EXP-03 | lotto con un documento **senza righe** | il documento compare in `documentos` con `linhas = 0`, la riconciliazione intera e i suoi avvisi; non compare in `diagnostico` | 3C.4.1 |
| **TX-03** | EXP-02 | due righe dello stesso documento con `hash_registro` identico | due righe distinte in `diagnostico`, con `ordinal` diverso; nessuno scarto | 3C.4.1 |
| **TX-04** | EXP-06, EXP-08, EXP-37, EXP-38, EXP-39, EXP-53 | importi `0`, `0.00`, `0.01`, `-7.50`, `10.0000`, `1.2345`, `12345678901234.56`, `Decimal::MAX`, `Decimal::MIN` | la cella di testo riproduce il testo esatto, scala compresa. Per la cella numerica, dopo D26(a): **eleggibilità** secondo §7.2.1 caso per caso (`0.01` eleggibile, `Decimal::MAX` no); dove è presente, il valore riletto è **uguale** all'importo e la **scala non si confronta**; dove è assente, `_numero_status = NAO_REPRESENTAVEL` con il numero di cifre nel motivo. Uno zero tecnico in una cella numerica **fa fallire** il test. Il test elenca anche ogni occorrenza di `f64` nei tre moduli, per `EXP-07` ed `EXP-53` | 3C.4.1 |
| **TX-05** | EXP-09 | quantità `1.2345` in un componente della riconciliazione | quattro decimali conservati; nessun arrotondamento ai centesimi | 3C.4.1 |
| **TX-06** | EXP-10 | lotto con tre aziende, due competenze e quattro conti | `resumos` e `indicadores` coincidono **cella per cella** con `Resumo` e `Indicadores` della vista; nessuna somma ricalcolata dal writer | 3C.4.1 |
| **TX-07** | EXP-04, EXP-05 | documento con tre rate, di cui una senza scadenza; documento senza rate; riconciliazione con due componenti e un campo rinviato | tre righe in `rate` con `vencimento` vuoto per quella senza data; **nessuna** riga per il documento senza rate; componenti e campi rinviati nei loro fogli | 3C.4.1 |
| **TX-08** | EXP-12 | `cnpj_cpf_favorecido = 00012345678901`, `conta_id = 2.01.004`, chiave di 44 cifre | celle di tipo testo, zeri iniziali presenti, nessuna notazione esponenziale | 3C.4.1 |
| **TX-09** | EXP-13 | `=1+1 SUCO "MARCA" Açucar`, `+55 11`, `-7`, `@utente`, testo con ritorno a capo e virgolette | il valore riletto è **identico** al dato; nessun apostrofo premesso, nessun carattere aggiunto o rimosso | 3C.4.1 |
| **TX-10** | EXP-14 | lo stesso testo in NFC e in NFD nello stesso foglio | le due forme restano distinte e identiche agli originali, confrontate per code point | 3C.4.1 |
| **TX-11** | EXP-15 | istante con offset di 1 e di 2 secondi e nanosecondi; `vencimento` assente; `data_pagamento` assente con motivo | due colonne per l'istante, con l'offset in secondi; la data assente ha cella vuota e `status` compilato | 3C.4.1 |
| **TX-12** | EXP-11, EXP-13 | il file generato, riletto dal lettore indipendente | **zero** celle di tipo formula in tutti i fogli; i testi ostili sono di tipo stringa | 3C.4.1 |
| **TX-13** | EXP-16 | `Dado` presente, `Dado` assente `NAO_DISPONIVEL` con motivo, `Dado` `N.A.`, valore `0.00` reale, stringa vuota reale | cinque esiti distinguibili nelle quattro colonne; nessuno confuso con un altro | 3C.4.1 |
| **TX-14** | EXP-20 | errore iniettato fra il passo 2 e il passo 4: residuo non scrivibile, destinazione non rinominabile | il file preesistente resta **identico byte per byte**; l'errore nomina l'operazione e il percorso | 3C.4.1 |
| **TX-15** | EXP-21 | temporaneo già presente | `ResiduoPresente` con il nome del residuo; il residuo **non** viene cancellato né troncato | 3C.4.1 |
| **TX-16** | EXP-22, EXP-23 | destinazione assente; destinazione esistente con `Recusar`; destinazione esistente con `Substituir` | creazione; rifiuto con il precedente intatto; sostituzione con il precedente conservato in `<nome>.anterior.<etichetta>` | 3C.4.1 |
| **TX-16b** | EXP-23 | destinazione tenuta da un handle `FileShare.None` | errore nominato, precedente intatto. **Non** si promette che la sostituzione riesca | 3C.4.1 |
| **TX-17** | EXP-24 | impronta del file dell'archivio prima e dopo l'export | identica; `arquivo_io` non è chiamato in scrittura | 3C.4.1 |
| **TX-18** | EXP-08, §10.4 | il file aperto in **Microsoft Excel** | dieci celle note confrontate a mano: importi di testo, identificativi con zeri, testo che inizia con `=`, intero di 29 cifre nella colonna numerica. Esito registrato come misura, non come attesa | 3C.4.1, prova manuale |
| **TX-19** | EXP-17 | testo contenente letteralmente `_x0041_`, aperto in Excel | misura del comportamento reale; se Excel decodifica, D22 va riaperta prima di dichiarare il formato stabile | 3C.4.1, prova manuale |
| **TX-20** | EXP-25, EXP-28 | due export dallo stesso archivio, lotto e piano | tutte le celle coincidono, esclusa `gerado_em`, dichiarata | 3C.4.1 |
| **TX-21** | §4.2, EXP-46, EXP-47 | i sette casi di stato della revisione, compresi sanatoria, revoca della sanatoria e ri-approvazione; **più** due approvazioni di persone diverse (Alice 12, Bob 13) e la catena approva/ripete/revoca/riapprova/ripete (Alice, Bob, Carla, Dan, Eve), ciascuna anche con l'ingresso permutato | `extrato_revisado` contiene **solo** `APROVADO_POR_PESSOA` di documenti non `ERRO`; ogni esclusione compare in `exclusoes` con il motivo. `aprovado_por/em` portano **Alice, 12** nel primo caso e **Dan, 15** nel secondo, invariati per permutazione; `confirmacoes` conta le ripetizioni e `ultima_confirmacao_*` porta Bob ed Eve | 3C.4.1 |
| **TX-22** | D21 | archivio con due lotti ed eventi per entrambi, export del primo | `decisoes` contiene gli eventi applicati e quelli `ALVO_AUSENTE`; i tre conteggi di `proveniencia` tornano con le righe | 3C.4.1 |
| **TX-23** | §1.1 | piano corrente con un conto rinominato rispetto a quello dell'esecuzione | `apresentacao_divergente = SIM` e una riga di divergenza; l'export **riesce**, perché D8.3 non la tratta come errore | 3C.4.1 |
| **TX-24** | EXP-03, D10(a) | documento `ERRO` con una riga approvata da una persona | la riga è in `diagnostico` con `estado = APROVADO_POR_PESSOA`, **non** è in `extrato_revisado`, e compare in `exclusoes` con il motivo di D10(a) | 3C.4.1 |
| **TX-25** | §6.3 | riepilogo alimentato da un documento `PARCIAL` e da uno `ERRO` | `documentos_parcial` e `documentos_erro` maggiori di zero, `documentos_marcados` elencati, e le stesse marcature in `avisos` | 3C.4.1 |
| **TX-26** | §4.3 | lotto inesistente richiesto | errore nominato che dice quali lotti l'archivio contiene; nessun file creato | 3C.4.1 |
| **TX-27** | EXP-29, F3C-21 | rilettura del file con lo strumento indipendente | confronto **campo per campo** fra il modello in memoria e ciò che il lettore restituisce, con i limiti e la provenienza del lettore dichiarati nel rapporto | 3C.4.1, script esterno |
| **TX-28** | EXP-18 | testo con i soli caratteri di controllo che XML 1.0 ammette: tabulazione, LF, CR | i tre sopravvivono e il valore riletto è identico all'originale; il CR protetto come `&#13;` è accettato dal lettore indipendente; nessuna sostituzione silenziosa | 3C.4.1 |
| **TX-29** | EXP-33 | `autor` con U+0000, `nota` con U+0001, `observacoes` con U+001F, `descricao_itens` con U+000B | `ErroExport::TextoNaoRepresentavel` che nomina foglio, riga, colonna, codepoint e posizione; **nessun file creato**, nessun temporaneo, nessun lucchetto residuo; l'archivio invariato. Il caso del **surrogato isolato** non è qui: §7.4.2 spiega perché non è raggiungibile | 3C.4.1 |
| **TX-30** | EXP-34, EXP-35 | `nota` di 32767 caratteri e di 32768 | la prima passa, la seconda dà `TextoTroppoLungo` con caratteri e limite; **nessun troncamento** in nessuno dei due casi | 3C.4.1 |
| **TX-31** | EXP-35 | modello con più righe del limite di foglio, e con un numero di colonne al limite | `LimiteDoFormato` con il valore trovato e il limite; nessun file parziale. Il caso ZIP64 è dichiarato e, se non praticabile su dati sintetici, resta `NON_ESEGUITA` con il motivo | 3C.4.1 |
| **TX-32** | EXP-41 | `arquivo_revisao = 9_007_199_254_740_993` (oltre le 15 cifre significative) | la cella è di tipo testo e riporta le cifre esatte; non esiste una cella numerica per quel campo | 3C.4.1 |
| **TX-33** | EXP-42, EXP-48 | destinazione uguale al file dell'archivio, esistente e con maiuscole diverse; separatori ripetuti, `..`, prefisso `\\?\`, junction sul padre; e soprattutto le destinazioni **inizialmente assenti** della famiglia con maiuscole diverse: `<nome>.LOCK`, `<nome>.GER1`, `<nome>.NOVO.2`, `<nome>.CORRUPTO7`, `<nome>.RECUPERADO.1`, più `<NOME>` intero | rifiuto con `DestinoECoincideComFonte` **prima** del lucchetto, della rivendicazione, del temporaneo e della copia del precedente; sorgente e intera famiglia invariati, verificati per impronta. I nomi nei messaggi sono quelli dati dal chiamante, non la chiave di confronto. Il caso hard link è eseguito e il suo esito **registrato come misura**, non come attesa | 3C.4.1 |
| **TX-34** | EXP-43b, EXP-44 | due esecuzioni cooperanti intercalate, con `Recusar` e poi con `Substituir`, negli intrecci che il protocollo ammette | il secondo riceve `ExportOcupado` se arriva mentre il primo tiene il lucchetto, e `DestinoJaExistente` se arriva **dopo** il rilascio con la destinazione esistente: il test copre i due intrecci e non pretende un solo esito. Con `Substituir` serializzati dal lucchetto, ogni precedente effettivamente sostituito è conservato. **Nessuna** destinazione di zero byte compare in nessuno degli intrecci | 3C.4.1 |
| **TX-34b** | EXP-52 | due pubblicazioni che **ignorano** il lucchetto, intercalate fra il controllo di esistenza e il `rename` | il comportamento è **registrato come misura**, non come garanzia: l'attesa è che la seconda vinca in silenzio, ed è il limite che D25(b) accetta e che `EXP-52` scrive nel foglio `limites`. Un test che pretendesse l'atomicità qui sarebbe un test di una promessa che non facciamo | 3C.4.1 |
| **TX-37** | EXP-17, EXP-51, D22 | un testo che contiene letteralmente `_x0041_`, più `_x005F_x0041_`, scritti da `rust_xlsxwriter` e riletti dallo strumento indipendente | i valori riletti sono **identici** agli originali. Se non lo fossero, l'esito si registra come misura e D22 si riapre: **non** si compensa con una trasformazione nostra | 3C.4.1 |
| **TX-38** | §10.5 | le quattro proprietà richieste alla libreria: scrittura in un buffer in memoria, cella di testo che resta testo, nessuna conversione implicita, comportamento su `_xHHHH_` | ognuna misurata e registrata. Una proprietà che non reggesse **non** si aggira: si riporta a Luca | 3C.4.1 |
| **TX-35** | EXP-16, §6.3.12 | due eventi identici salvo `alvo.impressao_conteudo.motivo`, uno `None` e uno `Some("motivo dichiarato")` | le due righe del foglio `decisoes` sono **diverse**, e la differenza è nella colonna `alvo_impressao_conteudo_motivo`. È la regressione del riscontro `R3C4-01` | 3C.4.1 |
| **TX-36** | EXP-30, EXP-31, EXP-32 | archivio con due lotti su conti diversi e piano corrente che copre solo il primo; piano con la sola `versao` diversa; conto rinominato nel piano | l'export del primo lotto **riesce** nonostante il secondo lotto usi un conto fuori dal piano; le due identità del piano sono entrambe nel foglio, con `plano_coincide`; la divergenza del conto rinominato è attribuita allo snapshot giusto | 3C.4.1 |

Non è una quota. Se un contratto non ha una prova che possa fallire, va
riscritto o cancellato: è la regola che ha già prodotto i riscontri dei lotti
precedenti.

### 11.1 I quattro contratti senza una prova propria, e perché

Dichiarati qui invece di lasciarli trovare all'audit. §13 criterio 1 rinvia a
questa lista: un'ispezione **non** si chiama test superato.

| Contratto | Come si verifica |
|---|---|
| **EXP-07** e **EXP-53**: `f64` solo nell'unica funzione autorizzata | si verifica l'**inventario**, non un divieto assoluto: §11.2 |
| **EXP-19** nessuna strada di ritorno | affermazione **negativa** sulla superficie: `export.rs` e `xlsx.rs` non importano `arquivo_io`, e `export_io.rs` non apre l'archivio. `TX-17` e `TX-33` coprono l'effetto osservabile |
| **EXP-26** ordine deterministico delle righe | **[RIUSATO]**: l'ordine viene da `prelivro` e `revisao`, già provato (`I7` della 3C.2, ordine canonico della 3C.3.a). `TX-20` lo verifica di riflesso; un test che riordinasse l'ingresso proverebbe quelle fasi, non l'export |
| **EXP-27** contratto sui byte dello ZIP | non esiste e non va provato: è la dichiarazione che l'identità binaria **non** è promessa. Diventerebbe un contratto solo se qualcuno la chiedesse |

`EXP-22` non è più in questa lista: con `TX-33` la destinazione e la fonte
hanno un test proprio, e l'ispezione resta soltanto per l'assenza di letture
d'ambiente.

Un'affermazione negativa provata per ispezione è più debole di un test, e la
dico debole: se l'audit preferisce un test, `EXP-19` si può provare con un
controllo sui sorgenti eseguito dal test stesso, al prezzo di un test che legge
il codice invece del comportamento.

### 11.2 Il controllo dell'unica funzione autorizzata a usare `f64`

Dopo D26(a) il divieto non è più assoluto, quindi non si verifica cercando
«zero occorrenze»: si verifica che le occorrenze siano **esattamente** quella
autorizzata. `EXP-40` è decaduto e non compare più fra le cose da controllare.

**Come si fa, e che cosa va nel rapporto del lotto.**

| Passo | Contenuto |
|---|---|
| 1. inventario | ricerca di `f64`, `as f64`, `to_f64`, `parse::<f64>`, `into()` verso `f64` e `write_number` nei tre moduli nuovi — `export.rs`, `xlsx.rs`, `export_io.rs` — con **file e riga** di ogni occorrenza |
| 2. confronto | l'elenco deve contenere **una sola** funzione, quella di `xlsx.rs` che scrive la cella numerica di comodo. Ogni altra occorrenza è una violazione di `EXP-53`, da correggere, non da giustificare nel rapporto |
| 3. dichiarazione nel codice | quella funzione porta un commento che dice perché l'eccezione esiste e rinvia a D26(a), così chi la legge dopo non la prende per una distrazione |
| 4. i due moduli puri sul testo | `export.rs` non deve contenere nessuna occorrenza: il testo autorevole si produce con `Decimal::to_string()`, e `TX-04` lo verifica carattere per carattere |

**Perché un'ispezione e non un test.** Un test che leggesse i sorgenti
proverebbe il testo del codice, non il suo comportamento, e passerebbe anche
con una conversione nascosta dietro una funzione chiamata diversamente. Il
comportamento è coperto da `TX-04`, che fallirebbe su `10.0000` o su
`Decimal::MAX` scritti attraverso `f64` nella colonna **autorevole**. L'ispezione
serve a ciò che `TX-04` non può vedere: una conversione innocua oggi, in un
punto che domani diventa autorevole.

L'inventario è quindi un **obbligo di rapporto**, non un test superato, e §13
criterio 1 lo tratta come tale.

---

## 12. Primo lotto implementativo — 3C.4.1

### 12.1 File

| File | Natura | Perché |
|---|---|---|
| `src/export.rs` | nuovo, **puro** | modello dell'export e sua costruzione da `VistaRevisada` + `Proveniencia`. Nessun IO, nessun orologio, nessun percorso |
| `src/xlsx.rs` | nuovo, **puro** | adattatore verso `rust_xlsxwriter`: traduce il `ModeloExport` in fogli e celle della libreria e restituisce i byte del pacchetto. Non apre file. Dopo D18 **non** contiene né XML né ZIP scritti a mano |
| `src/export_io.rs` | nuovo, IO | destinazione, temporaneo, conservazione del precedente, commit con `rename`, esito con impronta |
| `src/lib.rs` | +3 righe di `pub mod` | registrazione dei moduli |
| `tests/export_releitura.rs` | nuovo | il percorso completo da un archivio sintetico fino al file, più il confronto con la rilettura, se `EXP-29` è accettato |
| `script/releitura_export.py` | nuovo, **fuori dal crate** | lettore indipendente; nome e cartella da confermare nel mandato implementativo |
| `README.md` | aggiornato | stato di fase, contratti `EXP-nn`, API, conteggio test, limiti |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | appendice di esecuzione | come nei lotti precedenti |

**Nessuna modifica** a `arquivo.rs`, `arquivo_io.rs`, `revisao.rs`,
`impressao.rs`, `prelivro.rs`, `alocacao.rs`, `dinheiro.rs`, `modelos.rs`,
`nfe.rs`, `regras.rs`, `main.rs`, ai test esistenti, a `config/*`, a
`samples/*` o al repository Python. L'export **consuma** quelle API e non le
tocca.

### 12.2 Superficie proposta

```rust
// export.rs — puro
pub const FORMATO: &str = "FANTOZZI-EXPORT/v1";
pub const VERSAO_SCHEMA: u32 = 1;

pub struct Proveniencia {               // tutto dal chiamante, niente dall'ambiente
    pub arquivo_caminho: PathBuf,
    pub arquivo_revisao: u64,           // esportata come TESTO (EXP-41)
    pub arquivo_sha256: String,
    /// Identita' del piano registrata nell'esecuzione del lotto.
    pub plano_arquivado: IdentidadePlano,
    /// Versione e fonte del piano **effettivamente usato** (EXP-32).
    pub plano_usado_versao: String,
    pub plano_usado_caminho: PathBuf,
    pub gerado_em: DateTime<FixedOffset>,
    /// Divergenze del **solo lotto esportato** (EXP-30).
    pub divergencias: Vec<Divergencia>,
    pub eventos_no_arquivo: usize,
}

pub struct ModeloExport { /* un campo per foglio, righe gia' ordinate */ }
pub enum ErroExport {
    TextoNaoRepresentavel { folha: &'static str, linha: usize,
                            coluna: &'static str, codepoint: u32, posicao: usize },
    TextoTroppoLungo { folha: &'static str, linha: usize,
                       coluna: &'static str, caracteres: usize, limite: usize },
    LimiteDoFormato { folha: &'static str, o_que: &'static str,
                      encontrado: usize, limite: usize },
    ContaFora { chave_origem: String, conta_id: String },
    LoteAusente { pedido: String, disponiveis: Vec<String> },
}

pub fn modelo(vista: &VistaRevisada, proveniencia: &Proveniencia)
    -> Result<ModeloExport, ErroExport>;

/// Costruisce il sotto-archivio del lotto scelto, per restringere lo scope
/// del controllo di presentazione senza modificare `arquivo` (EXP-30).
pub fn sub_arquivo_do_lote(arquivo: &Arquivo, lote: &str)
    -> Result<Arquivo, ErroExport>;

// xlsx.rs — puro, adattatore verso rust_xlsxwriter (D18)
//
// Restituisce i byte del pacchetto: `export_io.rs` resta l'unico a toccare il
// disco. Che la libreria sappia scrivere in un buffer e' [NON_VERIFICATO],
// prova `TX-38`: se non sapesse, la firma cambia e la separazione fra puro e
// IO va ridiscussa invece di essere abbandonata in silenzio.
pub fn pacote(modelo: &ModeloExport) -> Result<Vec<u8>, ErroExport>;

// export_io.rs — IO
pub enum SeExistir { Recusar, Substituir }

/// Identita' da proteggere: la destinazione non puo' coincidere con il file
/// dell'archivio ne' cadere nella sua famiglia di nomi (EXP-42).
pub struct FonteProtegida {
    pub diretorio: PathBuf,
    pub nome: String,
}

pub struct Escrita {
    pub caminho: PathBuf,
    pub bytes: usize,
    pub sha256: String,
    pub anterior: Option<PathBuf>,
    pub lucchetto_residuo: bool,
}

pub fn escrever(
    destino: &Path,
    pacote: &[u8],
    se_existir: SeExistir,
    fonte: &FonteProtegida,          // EXP-42: senza questo il controllo e' impossibile
    etichetta: &str,                 // EXP-45: dal chiamante, non da un orologio
    lucchetto: &MetadadosLucchetto,  // EXP-44: riusa il tipo di arquivo_io
) -> Result<Escrita, ErroExportIo>;
```

La firma precedente non riceveva né la fonte né l'etichetta: erano due
controlli impossibili da realizzare, non due dimenticanze di scrittura.
`MetadadosLucchetto` è già pubblico in `arquivo_io` e viene **riusato**, non
ridefinito.

La separazione in tre moduli non è estetica: `export.rs` e `xlsx.rs` restano
provabili senza toccare il disco, e `export_io.rs` resta provabile con byte
qualunque. È la stessa divisione che ha permesso di provare il codec
dell'archivio senza file.

### 12.3 Ordine di lavoro proposto

1. **Prima di tutto, `TX-38`**: misurare le quattro proprietà di
   `rust_xlsxwriter` di §10.5 su un foglio minimo. È il passo che può far
   cadere D18, e scoprirlo dopo aver scritto l'adattatore costerebbe il
   doppio.
2. `xlsx.rs` con le prove del pacchetto, confrontate con il lettore
   indipendente, `TX-37` compresa.
3. `export.rs` e i test di riversamento campo per campo.
4. `export_io.rs` e i test del protocollo.
5. Il percorso completo in `tests/`, più le due prove manuali con Excel.

---

## 13. Criteri di accettazione del lotto 3C.4.1

Il lotto è consegnabile per audit soltanto se:

1. ogni contratto `EXP-01…EXP-53`, escluse le varianti decadute `EXP-40`, `EXP-43` ed `EXP-49`, ha almeno una prova che può fallire,
   **tranne** i quattro di §11.1 e l'inventario di §11.2, che si verificano per ispezione o per riuso e
   sono elencati là con il motivo. La mappa contratto → test, o contratto →
   ispezione, è nel rapporto, e un'ispezione **non** si chiama test superato;
2. tutte le prove `TX-01…TX-38`, più `TX-16b` e `TX-34b`, sono eseguite, oppure dichiarate
   `NON_ESEGUITA` con il motivo. `TX-18` e `TX-19` sono manuali e il loro esito
   va registrato come misura, non come attesa confermata;
3. i 337 test precedenti restano verdi **senza modifiche**;
4. `cargo test --locked`, `cargo fmt --check`,
   `cargo clippy --locked --all-targets -- -D warnings` e `git diff --check`
   sono puliti;
5. le sei fonti canoniche di `config/*` e `samples/*` sono invariate, con le
   impronte confrontate;
6. il perimetro escluso di §12.1 è intatto, dimostrato con
   `git diff --name-status`;
7. la dipendenza è **esattamente** quella autorizzata dal mandato, con la
   versione puntuale, le funzionalità minime elencate una per una e il diff del
   lockfile spiegato riga per riga, come è stato fatto per `serde_json`;
8. le prove `NON_VERIFICATE` sono elencate come tali, non presentate come
   coperte.

Il conteggio dei test **non** è un criterio: lo dice il metodo e lo hanno
confermato i quattro riscontri dell'audit precedente, trovati su una suite che
passava.

---

## 14. Limiti dichiarati

### 14.1 Di questo documento

| Limite | Stato |
|---|---|
| nessuna riga di writer esiste; le prove `TX-nn` sono pianificate | per costruzione |
| la versione, le funzionalità e le dipendenze transitive di `rust_xlsxwriter` | **[NON_VERIFICATO]**: il crate non è in cache e questo lotto non autorizza download. §10.5 dice che cosa il mandato deve registrare |
| le quattro proprietà richieste alla libreria (buffer, cella di testo, nessuna conversione, `_xHHHH_`) | **[NON_VERIFICATO]**, prova `TX-38`: leggere un'API non è eseguirla |
| il comportamento di Excel su XLSX non compresso, su `_xHHHH_` e sugli interi lunghi | **[NON_VERIFICATO]**, prove `TX-18`/`TX-19` |
| il comportamento con un file aperto in Excel durante la sostituzione | **[NON_VERIFICATO]**, prova `TX-16b` |
| volumi reali: la sonda ha tre righe | **[NON_VERIFICATO]** |
| LibreOffice e Google Sheets | fuori dai requisiti dichiarati |

### 14.2 Che l'export erediterà e non può risolvere

Vengono dall'archivio e dalla revisione, e il foglio `limites` del file li
dichiara invece di nasconderli: l'autore è **dichiarato** e non autenticato;
non esiste una sequenza storica dei documenti (D17(a)); la durabilità
dell'archivio dopo perdita di alimentazione è `NON_VERIFICATA`; il lucchetto
dell'archivio protegge gli scrittori cooperanti; l'archivio cresce senza
potatura (D16(a)).

### 14.3 Decisioni che restano fuori

D11(a) rinvia l'import dello storico e questo lotto non lo prepara. Non esiste
un percorso di ritorno dal file all'archivio, e crearne uno sarebbe una
decisione nuova, non un'estensione. La CLI (3C.5) e la DRE (3D) restano fuori.

---

## 15. Il confine di D12, senza inventare l'ingresso

D12(a) è approvata, e descrive il comportamento di un ingresso **grezzo** che
oggi non esiste. Questo è il confine, perché il blocco che lo riceverà non
debba inventarlo da zero.

### 15.1 Perché non si può attuare qui

| Fatto | Evidenza |
|---|---|
| `Lancamento::valor` è un `Decimal` | **[LETTO_NEL_CODICE]** `modelos.rs:553` |
| un `Decimal` non ha una forma «illeggibile»: riempirlo di zero è lo zero tecnico vietato | metodo |
| `Particao` ha tre varianti, e `INCOMPLETO` non è fra loro | **[LETTO_NEL_CODICE]** `prelivro.rs:303` |
| l'unico ingresso esistente è una NF-e XML, dove l'importo è già validato in fase 2 | **[RIUSATO]** fase 2 e 3B |

Un test su un `Decimal` che «non si legge» sarebbe finto, e il mandato lo
vieta. Quindi: nessun test di D12 in 3C.4.1.

### 15.2 La rappresentazione proposta, per il blocco che riceverà l'ingresso

**[PROPOSTA]**, da approvare in quel mandato, non in questo:

| Elemento | Proposta |
|---|---|
| tipo del valore | `Dado<Decimal>` al posto di `Decimal` **nella struttura dell'ingresso grezzo**, non in `Lancamento`: l'ingresso grezzo è un tipo nuovo, e `Lancamento` resta ciò che è |
| testo originale | campo obbligatorio `valor_bruto: String`, conservato così come è letto |
| motivo | `Dado::motivo` compilato, con la fonte che dice quale cella o colonna |
| partizione | una variante nuova `Particao::Incompleto`, che la specifica 3C.2 ha già dichiarato come precedenza più alta: `INCOMPLETO > NAO_RECONHECIDO > REVISAR > APROVADO > OK` |
| totali | i riepiloghi che includono una riga `INCOMPLETO` devono dichiararsi incompleti: serve un campo di stato nel `Resumo`, oppure la riga esclusa e contata a parte. **Due strade, da decidere allora** |
| export | il foglio `diagnostico` avrebbe `valor_texto` vuoto, `valor_bruto` compilato e `particao = INCOMPLETO`: lo schema di §6.3 va esteso, quindi `FANTOZZI-EXPORT/v2` |

Conseguenza da dire ora: introdurre `Particao::Incompleto` **cambia un enum
auditato** e tocca `prelivro.rs` e i suoi test. Non è un'estensione gratuita, e
il mandato che la chiederà deve metterla nel suo perimetro in modo esplicito.
D12 non autorizza a farlo di passaggio.

### 15.3 Stati grezzi e transizioni

`NAO_RECONHECIDO` ha lo stesso confine: dichiarato da `F3C-03`/`F3C-06`,
implementabile solo con un ingresso che lo produca. Nessuna transizione nuova
della revisione è necessaria per D12: una riga `INCOMPLETO` non è approvabile,
perché l'approvazione agisce su `status_revisao` e la riga non ha un importo da
approvare. **[PROPOSTA]**: la riga `INCOMPLETO` resta fuori da entrambi gli
estratti, e il motivo compare in `exclusoes`. Se invece dovesse essere
approvabile, è una decisione nuova su D9, da chiedere.

---

## 16. Resoconto del lavoro

### 16.1 Che cosa è stato letto, eseguito, riusato, proposto

| Etichetta | Contenuto |
|---|---|
| **[LETTO_NEL_CODICE]** | `prelivro.rs` (`Particao:303`, `IdentidadeLinha:326`, `LinhaPreLivro:336`, `DocumentoPreLivro:358`, `Resumo:391`, `Exclusao:418`, `Indicadores:427`, `extrato_aprovado:465`, `construir_pre_livro:506`); `revisao.rs` (`ChaveRevisao:90`, `DecisaoAplicada:400`, `EventoNaoAplicado:440`, `VersaoDocumento:459`, `LinhaRevisada:471`, `VistaRevisada:488`, `extrato_revisado:511`, `aplicar_revisao:656`); `arquivo.rs` (`contextos_do_lote:221`, `eventos:230`, `verificar_estrutura:1377`, `verificar_apresentacao:1465`); `arquivo_io.rs` (`observar:503`, `ler:525`, `publicar:557`); `modelos.rs` (`Lancamento:540`, `valor:553`); `alocacao.rs` (`Reconciliacao:270`); `nfe.rs` (`Aviso:147`); `config/csv_columns.yaml` |
| **[ESEGUITO]** | §10.1 stato di `Cargo.toml`, `Cargo.lock` e cache del registro (103 voci, nessun crate XLSX o ZIP), `cargo metadata --offline`, API di scrittura di `quick-xml`; §5.1 XLSX non compresso scritto a mano e riletto; §5.2 celle numeriche contro celle di testo, e Unicode per code point; §5.3 convenzione `_xHHHH_`; provenienza e versione di Python e `openpyxl` |
| **[RIUSATO]** | la baseline di 337 test e gli strumenti puliti del lotto 3C.3.b.1, non rieseguiti; le prove `TP-02`, `TP-09`, `TP-13` della persistenza; i test `t3_16_*`, `r1_revogacao_*` della revisione; §3.4 e §3.6 della specifica di persistenza; la contro-prova §1.4 della specifica 3C sul CSV in Excel |
| **[PROPOSTA]** | i contratti `EXP-01…EXP-53`, escluso `EXP-40` decaduto; le prove `TX-01…TX-38` con `TX-16b` e `TX-34b`; la rappresentazione di D12 in §15.2 |
| **[APPROVATO]** | D11(a) e D12(a) dal mandato; **D18–D26** ed **`EXP-29`** da Luca il 2026-10-23, con la condizione posta su D22: §3.3 |
| **[NON_VERIFICATO]** | il comportamento di Excel (apertura di uno XLSX non compresso, `_xHHHH_`, interi lunghi, file aperto durante la sostituzione); l'esclusione sorgente/destinazione su hard link e volumi montati due volte; i volumi reali; la stima di righe della strada (a); LibreOffice e Google Sheets |

Non ho rieseguito i 337 test: il mandato lo chiede esplicitamente e questo
lotto non tocca codice. Non ho installato nulla, non ho scaricato nulla, non ho
modificato `Cargo.toml` o `Cargo.lock`, e non ho aperto nessun archivio reale.

### 16.2 Le due cose che la misura ha cambiato rispetto alle attese

1. **La fotografia «nessun crate XLSX disponibile» è ancora vera, ed è peggio
   di così**: non c'è nemmeno un crate ZIP. Però §5.1 mostra che non serve: uno
   XLSX non compresso scritto a mano è leggibile. L'attesa iniziale era di
   dover chiedere una dipendenza; la misura ha prodotto una strada che non la
   richiede.
2. **La difesa consueta contro `_xHHHH_` corrompe il dato** con il solo lettore
   che si può eseguire. Era il contrario di ciò che mi aspettavo, ed è il
   motivo per cui D22 è una proposta con una prova rinviata, e non una regola
   applicata per prudenza.

### 16.3 Che cosa non ho fatto, pur potendo sembrare utile

Non ho riscritto le righe superate di `README.md` e della specifica generale
3C: sono elencate in §2 come fotografie, e il mandato vieta la pulizia generale
della documentazione in questo lotto. Non ho proposto un secondo writer CSV
«perché costa poco»: §10.3 spiega perché costerebbe un contratto intero. Non ho
scelto al posto di Luca fra lo script Python esterno e un lettore Rust per la
prova di fedeltà: era `EXP-29`, una domanda che Luca ha deciso il 2026-10-23 a favore dello script esterno.

### 16.4 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4 §1 | nuovo, completo | consegna a Codex per audit; commit documentale dopo audit e autorizzazione di Luca |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex | nuovo, con l'appendice di esito §17 | stesso commit documentale |

Nessun altro file è stato toccato. Sonde e script restano fuori dal repository,
in `…\scratchpad\sonda_3c4\` e nella cartella di scratchpad della sessione, e
non hanno un destino nel repository: sono prove di questo documento.

---

## 17. Che cosa serve per procedere

L'audit della specifica è **superato** (§18.3) e **D18–D26 ed `EXP-29` sono
decise** (§3.3). Il passo seguente è il **mandato implementativo 3C.4.1**, che
dovrà nominare:

| Che cosa | Dove sta scritto |
|---|---|
| i file del lotto e il perimetro escluso | §12.1 |
| `rust_xlsxwriter` con **versione puntuale**, funzionalità minime una per una, dipendenze transitive dal diff di `Cargo.lock` spiegato riga per riga, licenze | §10.5 |
| l'autorizzazione esplicita ad **acquisire** quella dipendenza: serve la rete una volta, non a ogni compilazione e mai a tempo di esecuzione | §10.3 |
| `TX-38` come **primo** passo, perché può far cadere D18 | §12.3 |
| l'inventario delle occorrenze di `f64` nei tre moduli, obbligo di rapporto dopo D26(a) | §11.2 |
| la forma della prova di rilettura indipendente, `EXP-29`, **approvata**: script esterno con Python e `openpyxl` già disponibili, senza installazioni | §10.2 |
| le due prove manuali con Excel, `TX-18` e `TX-19`, e che cosa fare se `TX-19` mostrasse una decodifica | §7.4.3 |
| che cosa fare se una delle quattro proprietà della libreria non reggesse: si riporta la misura a Luca, non si aggira | §10.5 |

Resta da fissare soltanto la versione con le funzionalità della dipendenza,
che il mandato stabilisce con l'acquisizione: §3.4.

---

## 18. Esito dell'audit del 2026-10-23 e correzioni

Codex ha prodotto cinque riscontri, tutti circoscritti all'export proposto.
Nessuno riapre D11, D12 o le decisioni precedenti. Tutti e cinque sono stati
**riprodotti prima** di correggere: le tre riproduzioni che richiedevano il
codice reale sono in §5.4, le altre due si leggono nel documento.

| Riscontro | Che cosa era sbagliato | Riproduzione | Correzione |
|---|---|---|---|
| **R3C4-01** | tredici fogli dichiarati, sei con le colonne scritte; `decisoes.impressao_conteudo_alvo` portava il solo valore di un `Dado<String>`, perdendo fonte, stato e motivo; le righe figlie di `avisos` avevano una chiave che non identificava il padre | due eventi identici salvo il `motivo` del `Dado` del bersaglio danno **righe identiche** sulle venti colonne proposte (§5.4) | §6.3 riscritta: tutti e tredici i fogli con ordine, tipo e chiave; `alvo_impressao_conteudo` in quattro colonne; **CHIAVE_LINHA** e **CHIAVE_RESUMO** nominate e usate nei fogli figli; `proveniencia` disambiguato da `tipo_linha`; §6.3.15 dichiara che cosa **non** si esporta. Regressione `TX-35` |
| **R3C4-02** | `EXP-18` affermava che l'ingresso fosse già nel repertorio XML 1.0 | un `autor` con U+0000 e una `nota` di 40000 caratteri attraversano archivio e `aplicar_revisao`; due parser indipendenti rifiutano il frammento, **e rifiutano anche `&#0;`** (§5.4) | `EXP-18` riscritto come **validazione al confine**; `EXP-33` errore di rappresentabilità nominato, `EXP-34` nessun troncamento, `EXP-35` limiti del formato. §7.4.1 separa dato logico e codifica e corregge la frase «proteggere corrompe il dato», che era una misura su un lettore presentata come un giudizio. `EXP-36`: la compatibilità non si dichiara stabile prima di `TX-19`. Prove `TX-29`…`TX-31` |
| **R3C4-03** | `escrever` non riceveva l'identità del sorgente, quindi `destino == arquivo_caminho` sostituiva l'archivio; `Recusar` era un `exists` più `rename`, non atomico; `etichetta` non era nella firma | il controesempio è nel protocollo e nella firma scritti in §8 e §12.2 | passo 0 di esclusione con `EXP-42` e confronto canonico, con il limite di hard link dichiarato; `EXP-43` `Recusar` atomico per rivendicazione con `create_new` sul nome finale; `EXP-44` lucchetto cooperativo (proposta D24); `EXP-45` etichetta nella firma. `FonteProtegida`, etichetta e lucchetto sono ora parametri. Prove `TX-33`, `TX-34` |
| **R3C4-04** | la colonna numerica non aveva una regola, e `TX-04` non confrontava il suo valore: uno zero su ogni riga sarebbe passato. `arquivo_revisao` era `N` | la regola assente si legge in §7.2 | §7.2.1: la colonna c'è sempre, la **cella** può mancare con stato e motivo; eleggibilità a **15 cifre significative**; le cifre di `<v>` identiche al testo autorevole. `EXP-41`: `arquivo_revisao` e `versao_schema` sono **testo**. `TX-04` esteso, `TX-32` aggiunta |
| **R3C4-05** | `verificar_apresentacao` visita tutto l'archivio e può restituire `ContaFora`; la tabella di §1.1 le attribuiva una garanzia sull'identità del piano; il foglio portava una sola identità | piano ridotto: controllo globale in **errore** su un conto di un altro lotto, mentre `aplicar_revisao` del lotto scelto riesce; piano con sola `versao` diversa: **zero divergenze** (§5.4) | §4.4 nuova: `EXP-30` sotto-archivio del lotto, `EXP-31` separazione dalla validazione strutturale, `EXP-32` **due** identità del piano con `plano_coincide`. Riga di §1.1 corretta, con una colonna su che cosa la funzione **non** attesta. Prova `TX-36` |

### 18.1 Le quattro osservazioni non bloccanti, accolte

| Osservazione | Che cosa ho cambiato |
|---|---|
| l'assenza del crate dalla cache non è un requisito funzionale | §10.3: i motivi di D18 sono riordinati per forza, e l'argomento d'ambiente è esplicitamente escluso dai motivi validi |
| `gerado_em` è un parametro, non cambia «per costruzione» | §9: `EXP-25` quantifica su tutti gli ingressi e promette tutte le celle; `EXP-28` copre il solo caso in cui il chiamante lo faccia variare |
| la prova indipendente può essere uno script, purché ripetibile; approvarla non approva pacchetti | `EXP-29`: aggiunti ripetibilità, codice di uscita e rapporto; detto che `openpyxl` non diventa una dipendenza |
| §13 ometteva `TX-28` e prometteva una prova per ogni `EXP` mentre §11.1 ne esentava alcuni | §13 criterio 1 e 2 riscritti, con il rinvio a §11.1; §11.1 aggiornata a quattro contratti, perché `EXP-22` ora ha `TX-33` |

### 18.2 Audit mirato del 2026-10-23: tre chiusi, due residui corretti

Esito dell'audit mirato: `R3C4-02`, `R3C4-04` e `R3C4-05` **chiusi come
specifica**; due residui circoscritti, entrambi riprodotti prima di correggere
(§5.5) e corretti qui.

| Residuo | Che cosa era sbagliato | Correzione |
|---|---|---|
| **R3C4-01 residuo** | §6.3.10 attribuiva `aprovado_por/em` all'**ultima** approvazione applicata. Ma una seconda `Aprovar` «fica na trilha e nao substitui a primeira» (`revisao.rs:864-879`): la regola avrebbe attribuito l'approvazione a chi ha solo ripetuto. Misurato: Bob invece di Alice, Eve invece di Dan | §6.3.10.1: `EXP-46` definisce l'approvazione **attiva** come la prima `Aprovar` dopo l'ultima `Revogar` efficace, con l'algoritmo scritto sulla traccia già ordinata; `EXP-47` dà alle ripetizioni colonne e nome propri (`confirmacoes`, `ultima_confirmacao_*`), così l'informazione non si perde e non si traveste. `TX-21` esteso ai due scenari con permutazione |
| **R3C4-03 residuo** | §8.1 attribuiva a `fs::canonicalize(parent)` l'insensibilità alle maiuscole del **nome**: falso per un nome che non esiste ancora. Misurato: il confronto lessicale accetta `ARQUIVO.JSON.LOCK`, la scrittura finisce in `arquivo.json.lock`, e si occupa il nome del lucchetto dell'archivio con byte XLSX | `EXP-48`: chiave di confronto insensibile alle maiuscole applicata a destinazione, nome dell'archivio e prefisso di famiglia; i nomi originali **non** si riscrivono; la famiglia si riconosce per **prefisso** `<nome>.`, così un nome futuro resta protetto; la direzione dell'errore è dichiarata per eccesso, perché NTFS può marcare una directory come sensibile. `TX-33` esteso alle varianti di maiuscole su destinazioni assenti |

Quattro allineamenti contestuali, accolti:

| Allineamento | Che cosa ho cambiato |
|---|---|
| la destinazione di zero byte resta anche dopo un **errore ordinario** dopo la rivendicazione, non solo dopo un arresto | `EXP-49`: `ReivindicacaoAbandonada` nomina il percorso e porta la causa; la libreria non cancella; un file di zero byte non è un export riuscito e nessun esito lo presenta come tale. Il costo entra nella decisione su D24, non nelle note. `TX-34b` aggiunta |
| con D24 il secondo writer può ricevere `ExportOcupado`, non necessariamente `DestinoJaExistente` | `TX-34` riscritto sui due intrecci: `ExportOcupado` durante il lucchetto, `DestinoJaExistente` dopo il rilascio. Detto che se D24 fosse respinta il test va riscritto e le garanzie diventano un limite |
| un surrogato isolato non sta in una `String` Rust valida | rimosso da `EXP-33` e da `TX-29`; §7.4.2 e `EXP-50` lo assegnano al confine di decodifica che potrebbe riceverlo, e lo dichiarano **irraggiungibile** dall'API tipizzata. Non fabbrico una `String` invalida per ottenere un errore |
| `plano_coincide = SIM` non certifica l'identità del contenuto del piano | §4.4: il significato ristretto è scritto accanto alla colonna, con la ragione — un piano può cambiare conservando versione e percorso — e va conservato nel file e nella documentazione |


### 18.3 Esito finale dell'audit della specifica: superato

Codex, 2026-10-23: **audit della specifica superato**, i due residui chiusi,
`R3C4-02`/`04`/`05` confermati chiusi, nessun nuovo rilievo tecnico bloccante.
Verificato con una sonda esterna sul modello reale che `EXP-46` restituisce
Alice e Dan nelle due permutazioni, con Bob ed Eve come conferme, e che la
regola di `EXP-48` rifiuta le sette varianti ASCII della famiglia.

Che cosa questo esito **non** è: non è l'approvazione di D18–D25, non è
l'autorizzazione di una dipendenza, e non è un collaudo del prodotto, che non
esiste. I limiti dichiarati restano: nomi non ASCII, alias non coperti dalla
canonicalizzazione, compatibilità Excel, volumi reali.

**La precisazione recepita.** Il rapporto segnala una mia imprecisione nella
presentazione dei costi: avevo scritto che il lucchetto «porta con sé la
destinazione di zero byte». È sbagliato, e sono due meccanismi distinti:

| Meccanismo | A che cosa serve | Costo |
|---|---|---|
| rivendicazione, `EXP-43` | rendere **atomico** `Recusar` | può lasciare una destinazione di zero byte |
| lucchetto, `EXP-44` | coordinare esecuzioni cooperanti e conservare coerentemente il precedente con `Substituir` | un file di lucchetto da rilasciare, e un `ExportOcupado` per chi arriva durante |

Rifiutare D24 **non** elimina il residuo a zero byte. La correzione è in §8.2,
dove l'alternativa già descritta ha ora un nome proprio — **D25** — così che le
due scelte si possano prendere separatamente, e in §3.3 e §18.3. Non è una
riprogettazione: la seconda strada era già scritta, le mancava l'etichetta.


### 18.4 Le decisioni di Luca del 2026-10-23, e che cosa hanno cambiato

Tutte e otto decise. §3.3 le registra; qui si legge l'effetto sul documento.

| Decisione | Esito | Che cosa è cambiato nel documento |
|---|---|---|
| **D18** | `rust_xlsxwriter`, **non** il writer a mano | §10.3 dichiara quale misura resta valida e quale decade; §10.5 nuova, con ciò che il mandato deve registrare sulla dipendenza e le quattro proprietà da misurare; `src/xlsx.rs` diventa un adattatore; §12.3 mette `TX-38` **prima** di tutto, perché è il passo che può far cadere D18; dichiarata l'acquisizione di rete necessaria **una volta**, non un requisito permanente né di esecuzione |
| **D19** | approvata | §6 invariata: è il contratto |
| **D20** | approvata con la regola di §7.2.1 | §7.2.1 invariata; l'alternativa «nessuna colonna numerica» non è più aperta |
| **D21** | approvata | §4.3 invariata |
| **D22** | obiettivo approvato, con **condizione** | `EXP-17` riformulato: nessuna trasformazione nostra, la codifica è della libreria; `EXP-51` nuovo rende verificabile la condizione; §7.4.3 conserva la tabella dei lettori e dichiara che la colonna «che cosa arriva al file» è ora `[NON_VERIFICATO]`; prove `TX-37` e `TX-19`. `EXP-33` resta, perché rifiutare non è trasformare |
| **D23** | approvata | §9 invariata |
| **D24** | approvata | `EXP-44` resta; §8.2 riscritta |
| **D25** | **(b)** approvata | `EXP-43b` sostituisce `EXP-43`: si **controlla** l'esistenza, non si rivendica. `EXP-49` e `ReivindicacaoAbandonada` **decadono**; nessuna destinazione di zero byte è possibile. `EXP-52` nuovo: il limite di non atomicità va nel foglio `limites`. `TX-34` riscritto, `TX-34b` riassegnato a misurare quel limite |
| **D26** | **(a)** approvata | riallineati `EXP-07` ed `EXP-39`, `EXP-40` decaduto, `EXP-53` nuovo, `TX-04` sdoppiato: §7.2.1 e §7.2.3. Il nucleo monetario non è toccato |

**Una conseguenza da dire chiaramente.** La precisazione che avevo recepito —
il residuo a zero byte viene dalla rivendicazione, non dal lucchetto — è
diventata operativa con D25(b): scegliendo il controllo di esistenza, il
residuo non esiste più, e il lucchetto di D24 resta per quello che fa davvero,
coordinare le esecuzioni e proteggere il precedente.

### 18.5 Controllo mirato delle decisioni, 2026-10-23: un allineamento

Il controllo mirato sulla revisione 5 ha trovato **un** punto da risolvere
prima di commissionare il codice, più tre precisazioni. Non richiede nuova
architettura e non riapre le decisioni.

| Punto | Esito |
|---|---|
| **R5-01** | **fondato, e verificato da me sulla fonte**: `Worksheet::write_number` prende `impl Into<f64>`, `write` accetta `u64`/`i64` con perdita oltre ±999 999 999 999 999, e con la funzionalità `rust_decimal` accetta un `Decimal` **solo se rappresentabile come `f64`**. Non esiste un percorso numerico senza `f64`. Quindi `EXP-07` (divieto assoluto), `EXP-39` (cifre identiche) ed `EXP-40` sono incompatibili con D18 + D20. §7.2.3 nuova: il conflitto, le due strade di **D26** e le cinque condizioni della (a). I tre contratti sono **sospesi**, non riallineati: l'eccezione non è implicita in D18 |
| lettura operativa di D22 | precisata in §7.4.1: una **codifica interna diversa** con **testo riletto identico** è conforme e non riapre D22; soltanto un **testo logico** diverso è una prova fallita. `TX-37` confronta il testo letto, non il XML dentro il pacchetto. `TX-19` resta necessaria |
| «il lotto non sarà più eseguibile offline» | circoscritta in §10.3: la rete serve **una volta**, per acquisire e risolvere la dipendenza; non a ogni compilazione e **mai** a tempo di esecuzione del programma |
| `EXP-29` | registrato in §10.2 che il controllo mirato **consiglia** la prova esterna ripetibile con interprete e `openpyxl` già presenti, e che è una raccomandazione, non un'approvazione; lo script non introduce un runtime Python nel programma Rust |

**Che cosa ho verificato da me.** Non ho preso la firma dell'API dal verbale:
l'ho letta sulla documentazione ufficiale, fonte ammessa dal mandato §6. È una
**lettura**, non un'esecuzione, e resta `[NON_VERIFICATO]` ciò che solo
l'esecuzione può dire: che cosa la libreria scrive davvero nel `<v>` e come
Excel lo mostra. `TX-38` e `TX-18` restano.

**Una conseguenza utile del conflitto.** La soglia di 15 cifre significative di
D20, scelta prima di sapere quale libreria si userebbe, coincide con la
frontiera che la documentazione dichiara per `u64`: nel dominio eleggibile il
**valore** sopravvive al passaggio per `f64`. Non è una giustificazione del
conflitto — la **scala** si perde comunque — ma restringe il danno a ciò che la
condizione 4 di D26(a) vieta di promettere.

### 18.6 Che cosa resta aperto

**Nessuna decisione.** D18–D26 ed `EXP-29` sono decise il 2026-10-23; D26(a)
ha chiuso l'ultima questione bloccante, e il riallineamento di `EXP-07`,
`EXP-39`, `EXP-40` e `TX-04` è applicato.

Resta da **fissare** una cosa sola, che non è una decisione: la versione con le
funzionalità di `rust_xlsxwriter`, che §10.5 non può stabilire senza
l'acquisizione, e che il mandato implementativo registrerà. Dettaglio in §3.4.

I contratti `EXP-01…EXP-53` restano **[PROPOSTA]**, `EXP-40` escluso perché
decaduto: l'audit li ha giudicati implementabili, nessuno li ha approvati uno
per uno. Le prove `TX-01…TX-38` restano **pianificate**: `TX-21` e `TX-33`
appartengono al futuro writer e non sono prove già superate.

**Questo documento non implementa nulla e non chiude la fase 3C.**


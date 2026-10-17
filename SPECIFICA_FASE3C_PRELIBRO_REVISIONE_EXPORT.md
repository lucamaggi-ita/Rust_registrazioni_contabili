# Specifica fase 3C — pre-libro, revisione ed esportazione

Analisi e specifica del blocco **3C.1**: rendere utilizzabili le registrazioni
della 3B in un pre-libro con revisione ed esportazione.

| Voce | Valore |
|---|---|
| Data | 2026-10-16 |
| Mandato | `MANDATO_CLAUDE_FASE3C_SPECIFICA_20261016.md` |
| Revisione | **2.1**: riscontri **R3C-01, R3C-02, R3C-03** e quattro rettifiche minori recepiti (audit 3C.1), piu l'allineamento della firma con `PlanoContas` chiesto dall'audit mirato del 2026-10-17 | |
| HEAD all'inizio | `12584e65d01141626bf461321f6d2c5bc3e3c032` (worktree pulito, solo il mandato untracked) |
| Baseline Rust | 166 test prima della 3C.2 (150 lib + 3 bin + 13 CLI); 199 alla prima consegna; **208 dopo le correzioni dell'audit** (192 lib + 3 bin + 13 CLI), con i 166 preesistenti verdi senza regressioni |
| Fonte Python canonica | `Programma_registrazioni-contabili` @ `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` |
| Stato | **3C.1 chiusa** e **3C.2 chiusa tecnicamente**: implementata il 2026-10-17 in `src/prelivro.rs` con il refactoring di `src/dinheiro.rs`, corretta dopo il primo audit (R3C2-01/02/03, §14) e **audit mirato superato** lo stesso giorno. Resta la **registrazione in Git, da autorizzare**. Nessun writer, archivio, comando o dipendenza nuova |

> **La suite canonica Python è stata ESEGUITA**, da me e indipendentemente da
> Codex: **47 test canonici passati** sul commit fissato (`test_excel_writer`,
> `test_pre_livro_diario`, `test_revision_report`, `test_excel_diagnostics`,
> `test_allocation`). Nelle fasi 3A/3B lo stesso perimetro era dichiarato
> `NON_ESEGUITO`: il limite è chiuso per questi cinque file. Restano fuori la
> suite Python completa e `test_fase3_end_to_end.py`.

Gli identificativi `F3C-nn` sono provvisori: **non** sono contratti `C25+`
implementati, e nessun test qui elencato esiste. I casi si citano come
**`CAS-01…CAS-15`**, i test pianificati come **`T-01…T-23`**: due insiemi distinti,
senza numeri condivisi (rettifica 3 dell'audit).

---

## 1. Fonti, prove eseguite e metodo

### 1.1 Rust, sola lettura

| Elemento | Fatto verificato |
|---|---|
`Lancamento` ([src/modelos.rs](src/modelos.rs)) | **26 campi**, ricontati |
`AlocacaoNfe` ([src/alocacao.rs](src/alocacao.rs)) | `lancamentos: Vec<Lancamento>` + `reconciliacao: Reconciliacao` |
`Reconciliacao` | **15 campi**; **non** contiene chiave NF-e, azienda né competenza — verificato: un documento senza righe non porta con sé la propria identità (§2.3) |
`Lancamento::valor` | `Decimal`, **sempre presente e interpretabile** per costruzione |
`Lancamento::status_revisao` | `StatusRevisao`, **enum chiuso**: `Ok`, `Revisar`, `Aprovado`. Non esiste "stato vuoto" o "sconosciuto" |
Avvisi / errori del modulo | 11 codici in `alocacao::avisos`; 8 varianti di `ErroAlocacao` |
Scritture esistenti | **nessuna**: `alocar_nfe` è pura, nessun comando la usa, non esiste archivio né writer |
Aritmetica | centesimi interi `i128`; `em_centesimos`, `de_centesimos`, `somar_centesimos`, `dividir_meio_para_cima` sono **privati** al modulo |

### 1.2 Python canonico: cosa è stato eseguito

Commit fissato estratto con `git archive` in area temporanea (nessun checkout,
nessuna scrittura nel repo Python), con `fantozzi.__file__` verificato
sull'estratto.

```text
pytest tests/test_excel_writer.py tests/test_pre_livro_diario.py
       tests/test_revision_report.py tests/test_excel_diagnostics.py
       tests/test_allocation.py
→ 47 passati, 0 falliti     (confermato indipendentemente dall'audit)
```

**Trappola metodologica registrata.** Il primo tentativo usava `py -I`, che
**ignora `PYTHONPATH`**: pytest importava il `fantozzi` installato in modo
editabile, cioè il **working tree** del repo Python, e due test fallivano. Non
era un difetto della fonte: era un'altra versione.

**Due schemi di export osservati nel codice** (rettifica 1 dell'audit):

| Schema | Colonne | Differenza |
|---|---|---|
| commit fissato `5b70f2b` | **26** | il layout di riferimento di questa analisi |
| working tree attuale del repo Python (letto in sola lettura) | **31** | aggiunge `cfop`, `natureza_operacao`, `tipo_operacao_especial`, `confianca_operacao_especial`, `tratamento_operacao_especial`; nessuna colonna rimossa |

La revisione 1 di questo documento nominava male le cinque colonne aggiunte:
elencava `tipo_lancamento`, `dre_linha_id`, `dre_linha_nome`, `data_pagamento`
e `duplicatas_json`, che sono **già presenti** nel layout canonico a 26.
Corretto qui.

**Limite da non confondere:** sono due schemi **di codice** osservati. Nessun
workbook storico è stato fornito o verificato, né per il layout a 26 né per
quello a 31. Che esistano due schemi nel codice non prova quanti e quali file
reali esistano: l'import dello storico (decisione **D11**) resta senza prove
sui dati.

### 1.3 Sonde eseguite sul commit fissato

Dati sintetici in cartella temporanea, nessun workbook reale.

| # | Caso | Esito osservato |
|---|---|---|
| S1 | due righe legittime con lo stesso payload nello **stesso lotto** | **la seconda è scartata in silenzio**: 2 passate, 1 scritta |
| S2 | `descricao_itens` = `=1+1 SUCO "MARCA"` | cella scritta come **formula** (`data_type='f'`); rilettura `data_only=True` → **`None`**: testo perduto |
| S3 | `valor` = `79228162514264337593543950335` in cella numerica | riletto **`7.922816251426434e+28` come `float`**: precisione perduta. `12345678901234.56` e `0.01` sopravvivono |
| S4 | `cnpj_cpf_favorecido` = `00012345678901`, chiave con zero iniziale | conservati come **stringa**, zeri iniziali intatti |
| S5 | stati `OK`, `REVISAR`, `APROVADO`, vuoto, `DESCONHECIDO`, `ok` | contenitore OK = **2** (`OK` e `ok`); `APROVADO`, vuoto e sconosciuto in revisione |
| S6 | importi non interpretabili | `None`, `''`, `'abc'`, `'10,50'` → **`0.00`**; `'1e3'` → `1000.00`; `Decimal('10.005')` → `10.00` |
| S7 | tre aziende diverse nello stesso mese | **una sola riga** nel `Resumo_Mensal`: `('2026-05', 3, 60, 0)` |
| S8 | stesso `conta_id` con nomi `Refrigerantes` e `REFRIGERANTES` | **due righe** nel `Resumo_por_Conta` |
| S9 | cella `valor` = formula `=5+5` mai aperta da Excel | `valor_total_lancamentos_ok` = **`0.00`** |
| S10 | rate con scadenza assente | `duplicatas_json` valido, `"vencimento": null` |

S6 merita una nota: `'10,50'` — la virgola decimale brasiliana — diventa
`0.00`, e `'1e3'` diventa mille.

### 1.4 Controprova dell'audit sul formato XLSX

L'audit ha eseguito una controprova decisiva per §2.2: un workbook sintetico
costruito con contratti espliciti **conserva** entrambe le righe con payload
uguale, `MAX` come **testo decimale esatto**, il testo `=1+1 SUCO "MARCA"` come
**stringa** anche alla rilettura `data_only`, e stati/fonti/motivi in colonne
con riconciliazione e `Dado` in un foglio `Documenti`, con confronto esatto
dopo rilettura.

**Conseguenza, recepita in questa revisione:** le sonde S1, S2, S3 e S9
dimostrano difetti del **writer Python attuale** e il limite della **cella
numerica**, non un'incapacità del **formato XLSX**. La revisione 1 confondeva
le due cose (R3C-03) e il confronto di §2.2 è stato riscritto.

### 1.5 Prove **non** eseguite

| Non eseguito | Motivo |
|---|---|
| suite Python completa, `test_fase3_end_to_end.py` | fuori perimetro di questo blocco |
| qualunque scrittura su workbook reali o cartelle di produzione | vietato dal mandato |
| scrittura atomica, errore di salvataggio, file bloccato (F3C-20) | dipendono da IO che non esiste ancora |
| suite Rust | baseline di 166 test riusata: nessun file di codice modificato |
| workbook storici reali | **nessuno fornito**: vedi §1.2 |

---

## 2. A — Flusso, ingressi e fonte autorevole

### 2.1 Il percorso

```text
[3B, esistente]                 [3C, da costruire]
AlocacaoNfe ─┐
             ├─> ContextoDocumento ─> PreLivro (in memoria) ─> Revisione ─┐
avisos 3B ───┘        (identità,          partizioni,         (umana,     │
                      azienda,            riepiloghi,          tracciata) │
                      competenza,         invarianti                      │
                      lotto)                                              ▼
                                            Archivio autorevole ──> Report derivati
                                            (append-only,           (workbook, testo)
                                             versionato)                  │
                                                      ▲                   │
                                                      └── rilettura e verifica
```

Quattro distinzioni da tenere separate: **autorevole / derivato**;
**esecuzione (lotto) / storico**; **flusso nuovo / import dello storico**;
**riga / documento / lotto**. La riconciliazione vive sul **documento**:
appiattire tutto in righe la perde.

### 2.2 Confronto onesto fra due soluzioni conformi ai vincoli

Riscritto dopo R3C-03. Entrambe le colonne descrivono soluzioni **conformi** a
C2/C10/C23 e alla conservazione degli esiti tecnici: i difetti del writer
Python non sono attribuiti al formato.

| Criterio | **(1) Workbook autorevole con contratti espliciti** | **(2) Archivio strutturato versionato + workbook derivato** |
|---|---|---|
| Importi esatti | **possibile**: colonna testuale decimale (controprova §1.4). Costo: le celle non sono più calcolabili in Excel senza conversione | nativo: testo decimale, nessun `float` |
| `Dado` (valore, fonte, stato, motivo) | **possibile**: quattro colonne per campo, o un foglio dedicato. Costo: lo schema cresce molto | nativo |
| `Reconciliacao` (15 campi, 4 esiti) | **possibile**: foglio `Documenti` separato con chiave di collegamento. Costo: integrità fra fogli da validare a ogni lettura | nativo |
| Conservazione delle righe | **possibile**: dipende dall'algoritmo, non dal formato; va sostituito il dedupe per hash (S1 è un difetto del writer) | dipende dallo stesso contratto di identità: **non viene gratis dal formato** |
| Testo tipo formula | **possibile**: scrittura come stringa inline con `quotePrefix` (controprova §1.4) | nessun rischio nel formato testuale; resta nell'export |
| Modifica umana | **diretta**, è il vero vantaggio: nessuno strumento nuovo | **indiretta**: serve un percorso di revisione e una validazione dell'ingresso |
| Calcoli in Excel per chi revisiona | perduti sulle colonne testuali: va deciso quali colonne restano numeriche e con quale limite | non applicabile: il workbook derivato può avere colonne numeriche *oltre* a quelle testuali |
| Compatibilità storica | nativa, ma con **due schemi di codice** osservati (26 e 31) e nessun file storico verificato | via export derivato, con `versao_schema` dichiarata |
| Ripetibilità | append su file esistente: ordine e stato dipendono dal contenuto precedente; la rigenerazione da zero va progettata | rigenerabile da zero e confrontabile |
| Scrittura atomica | realizzabile: scrittura su temporaneo e rinomina; `save()` in posto va evitato | realizzabile allo stesso modo |
| Lettura/scrittura | **due** dipendenze (scrittura e lettura XLSX), **nessuna in cache locale**: serve rete e autorizzazione | `serde_json` **già in cache locale**; XLSX serve solo per il derivato |
| Validazione delle modifiche umane | obbligatoria e non banale: ogni cella è modificabile, inclusi importi e stati | la revisione entra da un percorso dichiarato, più semplice da validare |

**Raccomandazione: (2)**, per tre motivi che restano veri anche dopo la
controprova: l'ingresso umano è **validabile** invece che illimitato; il
formato autorevole non richiede dipendenze di rete; la rigenerazione completa è
confrontabile. Il costo reale della (2) è che **nessuno può aprire la verità in
Excel**: serve il derivato, ed è un blocco in più. Non è una scelta approvata:
decisione **D7**.

### 2.3 Ingresso del pre-libro — delimitazione del contratto (R3C-01)

La 3B dà `AlocacaoNfe` per **un** documento, e **non** porta la sua identità:
verificato, `Reconciliacao` non contiene chiave, azienda né competenza, e un
documento senza righe non ha nemmeno una riga da cui dedurle. Dedurre l'identità
dalla prima registrazione è quindi sbagliato in generale.

**Scelta del contratto, opzione 1 dell'audit: 3C.2 accetta solo ingressi
tipizzati della 3B.** Gli ingressi grezzi (importi illeggibili, stati
sconosciuti, workbook storici) arrivano solo attraverso **adattatori** di
blocchi successivi, e non sono testabili nel modulo puro.

```text
ContextoDocumento {
  // identità del documento, obbligatoria e indipendente dalle righe
  chave_origem     : String            // chiave NF-e; vuota = rifiutata (C22)
  numero           : String
  serie            : String
  data_documento   : NaiveDate
  competencia      : Competencia        // validata (C8)
  cnpj_empresa     : String             // vuoto = partizione dichiarata, §6.1
  nome_empresa     : String
  arquivo_origem   : String

  // risultato della 3B e avvisi di lettura
  alocacao         : AlocacaoNfe        // può avere lancamentos vuoto
  avisos_leitura   : Vec<Aviso>         // dalla fase 2

  // appartenenza all'esecuzione
  lote             : IdentidadeLote
}

IdentidadeLote {
  id               : String   // dichiarato da chi invoca; non dedotto dall'ambiente
  gerado_em        : NaiveDate
  descricao        : String   // libera, non interpretata
}
```

Chi costruisce `ContextoDocumento` ha già in mano la `Nfe` letta dalla fase 2:
i campi di identità vengono **da lì**, non dalle registrazioni. Un documento
con `lancamentos` vuoto conserva così chiave, azienda e competenza, e compare
nel pre-libro come documento senza righe con il suo `stato_documento` (caso
**CAS-04**).

Conseguenze dirette sulla proposta 3C.2 (§8):

| Caso | Dove vive ora |
|---|---|
| importo mancante o non interpretabile | **non** in 3C.2: `Lancamento::valor` è `Decimal` sempre presente. Vive nell'adattatore di import (3C.4) |
| stato vuoto o sconosciuto | **non** in 3C.2: `StatusRevisao` è un enum chiuso. Vive nell'adattatore di import (3C.4) |
| documento senza righe | **in 3C.2**: rappresentabile, con identità dal contesto |
| partizione `INCOMPLETO`, stato `NAO_RECONHECIDO` | contratti **dichiarati** (F3C-03, F3C-06) ma **implementati e testati in 3C.4**, quando esisterà un ingresso che li produce |

Questo non chiede di cambiare il modello 3B, non converte assenze in zero e non
anticipa l'import.

---

## 3. B — Identità, reimportazione, conflitti

C23 resta intatto: `hash_registro` è sha256 di `origem|chave|conta_id|valor`,
confrontabile con lo storico Python, **non** chiave univoca e **mai** motivo
sufficiente per scartare una registrazione valida.

### 3.1 I sei casi, separati

| # | Caso | Fatto osservato | Comportamento richiesto |
|---|---|---|---|
| 1 | due righe legittime della stessa allocazione con payload uguale | la 3B le conserva e avvisa (`HASH_DUPLICADO_NO_DOCUMENTO`); il writer Python **scarta la seconda** (S1) | conservare entrambe, identità di riga distinta, avviso mantenuto |
| 2 | reimportazione identica | il writer non riscrive (dedupe per hash) | riconosciuta come **ripetizione**: nessun duplicato, nessuna riga persa, esito dichiarato |
| 3 | correzione a parità di chiave e totale di conto | hash **identico**: la correzione non si distingue | **nuova versione** del documento, storico conservato, differenza esplicita |
| 4 | cambio di regole o conto | la riga cambia `conta_id` → hash diverso → nel Python convive con la vecchia | nuova versione + versione delle regole registrata; la precedente non si cancella |
| 5 | riga manuale, annotazione, approvazione | il Python esclude `origem = MANUAL` dal dedupe (`excel_writer.py:119-121`) | righe umane e campi scritti dall'uomo **preservati** (F3C-22) |
| 6 | conflitto fra archivi o versioni | nessun meccanismo nel Python | conflitto **dichiarato**, stato `CONFLITO`, entrambe conservate |

### 3.2 Identità di riga, e il suo scope (rettifica 2)

```text
IdentidadeLinha {
  lote          : String   // a quale esecuzione appartiene
  chave_origem  : String   // quale documento
  versao        : u32      // quale versione di quel documento
  origem        : OrigemLancamento
  conta_id      : String
  ordinal       : u32      // posizione nella sequenza allocata del documento
}
```

**Scope dichiarato:** la tupla `(origem, chave_origem, conta_id, ordinal)` è
unica **dentro una singola versione di un singolo documento**. Non è unica nello
storico: la stessa tupla ricompare in ogni versione dello stesso documento, ed è
esattamente ciò che permette di confrontare le versioni fra loro. L'identità
completa, unica nello storico, include quindi `lote` e `versao`.

**Definizione di stabilità:** `ordinal` è stabile perché l'ordine delle
registrazioni della 3B è contrattualizzato (C18: ordine di prima comparsa del
conto fra gli articoli, differenza di totale in coda). Lo stesso XML riallocato
con le stesse regole dà gli stessi `ordinal`. **Non** è stabile rispetto a: un
diverso ordine degli articoli nell'XML, un cambio di regole che fonde o separa
gruppi, la comparsa di una differenza di totale prima assente. Per questo serve
la versione, e per questo non propongo né l'indice di riga del foglio (cambia a
ogni rigenerazione) né un UUID casuale (non riproducibile).

### 3.3 Versione del documento, e il testo canonico dell'impronta

```text
VersaoDocumento {
  chave_origem : String
  sequencia    : u32
  impressao    : String              // impronta del contenuto allocato
  motivo       : MotivoNovaVersao    // Reimportacao | Correcao | RegrasMudadas | Manual
}
```

**Il testo canonico dell'impronta non è ancora definito** (rettifica 2). Quello
che è già deciso: deve essere un testo **dichiarato e riproducibile**, calcolato
con `sha2` (già autorizzato), e **non** riusare il payload di C23.

Quello che resta da contrattualizzare prima di 3C.3/3C.4, e che qui è
dichiarato **rinviato**, non risolto:

- quali campi entrano (righe allocate, riconciliazione, avvisi) e quali no
  (percorso del file, data di esecuzione, identità del lotto: variano senza che
  il contenuto contabile cambi);
- come si normalizza un `Decimal` nel testo (proposta: centesimi interi, così
  `10`, `10.00` e `10.0000` danno la stessa impronta);
- l'ordine dei campi e delle righe (proposta: l'ordinamento già contrattuale di
  §6.1);
- la serializzazione delle assenze (`Dado` senza valore) e degli stati.

Finché questo contratto non esiste, `VersaoDocumento` è una **proposta di
struttura, non un algoritmo pronto**.

### 3.4 Se il primo blocco non crea storico

3C.2 **non** crea archivio. Restano separate:

- deduplicazione e import dello storico: **rinviate**, perimetro dichiarato;
- conservazione di **tutte** le righe del lotto corrente: **obbligatoria già in
  3C.2**, verificabile in memoria (F3C-02, T-15).

---

## 4. C — Stati, revisione umana, autorizzazioni

### 4.1 Macchina a stati

Stati rappresentabili oggi in Rust: `Ok`, `Revisar`, `Aprovado`.
`NAO_RECONHECIDO` serve solo a ciò che **arriva da un ingresso grezzo**, quindi
nasce con l'adattatore di 3C.4 (§2.3), non nel modulo puro.

```text
   [allocazione 3B]  ──> OK ───────────┐
                    └─> REVISAR ─┐     │
                                 ▼     ▼
   revisione umana tracciata ─> APROVADO (stato finale del flusso 3C)
                                 ▲
   ingresso grezzo (3C.4) ─> NAO_RECONHECIDO ──> solo revisione umana
```

| Transizione | Chi | Prerequisiti | Campi modificabili | Tracciamento minimo | Blocco |
|---|---|---|---|---|---|
| — → `OK` | allocatore 3B | classificazioni `Ok`, nessun avviso (C18) | nessuno | nessuno: è un esito calcolato | esistente |
| — → `REVISAR` | allocatore 3B | qualunque avviso | nessuno | nessuno | esistente |
| `REVISAR` → `APROVADO` | **persona** | riga presente nel lotto; motivo non vuoto | `observacoes` + campi dell'approvazione | `quem`, `quando`, `motivo`, `versao`, `valores_prima` | 3C.3 |
| `OK` → `APROVADO` | persona | idem | idem | idem; **mai automatica** | 3C.3 |
| `NAO_RECONHECIDO` → `REVISAR`/`APROVADO` | persona | valore originale conservato | idem | idem + `valor_original` | 3C.4 |
| `APROVADO` → `REVISAR` | persona | motivo | idem | idem: la revoca è tracciata come l'approvazione | 3C.3 |
| qualunque → `OK` | **vietato** | — | — | `OK` è un esito tecnico, non una decisione | — |

Regole non negoziabili:

1. **nessuna promozione automatica ad `APROVADO`**;
2. **nessun default `OK`**: nel Python un valore vuoto finisce in revisione
   (S5), prudente ma confonde "manca il dato" con "da revisionare";
3. **l'approvazione non cancella avvisi né cambia gli esiti tecnici**: dopo
   un'approvazione, `alertas`, i quattro esiti e `stato_documento` restano
   quelli calcolati;
4. alla rigenerazione, approvazioni e annotazioni sopravvivono se identità
   **e** versione coincidono; se la versione cambia, l'approvazione resta
   registrata sulla versione a cui si riferisce e la nuova nasce da
   revisionare.

### 4.2 Cinque operazioni distinte

| Operazione | Che cos'è | In 3C.2 | Dove |
|---|---|---|---|
| approvare una registrazione | decisione umana sullo stato | no, solo modellata | 3C.3 |
| correggere un conto | riclassificazione: cambia i riepiloghi | no | blocco separato |
| modificare un importo | tocca denaro, invarianti e riconciliazione | no | blocco separato; forse mai sulla riga, ma sulla fonte |
| riconoscere un pagamento | popola `data_pagamento`, oggi assente per contratto (F3A-17) | no | richiede una fonte di pagamenti che non esiste |
| cambiare una regola YAML | cambia la classificazione di molti documenti | no: le regole restano canoniche nel Python | mandato separato |

Nessuna è coperta dall'esportazione: l'`importer` della revisione
classificazioni (`classification_review_workbook_importer.py:48-85`) produce un
**CSV di regole riviste**, non un'approvazione di registrazioni.

### 4.3 Tracciamento minimo, senza inventare utenti e permessi

`quem` è una **stringa dichiarata** da chi opera, non l'identità del sistema
operativo: l'utente del processo non è una firma. Nessun ruolo, nessun
permesso, nessuna autenticazione. Una cella modificata a mano in un workbook,
da sola, **non** è un'autorizzazione: è un dato in ingresso da riconoscere,
attribuire e motivare.

---

## 5. D — Conservazione di riconciliazione, assenze, copertura

### 5.1 Che cosa non si può appiattire

| Elemento 3B | Dove va | Se si perde |
|---|---|---|
| quattro esiti + `stato_documento` (C24) | contenitore **documento** | un documento `ERRO` sembrerebbe sano perché le righe sono `OK` |
| `avisos_documento` | documento | la causa dell'`ERRO` diventa invisibile |
| `componenti` + `esito_componenti` | documento: nome, `Dado`, ruolo, se usata in verifica | un `PARCIAL` non spiegabile |
| `articoli_attesi` = `NAO_DISPONIVEL` | documento, come **assenza dichiarata** | una copertura che sembra completa |
| `articoli_scartati` | documento | righe sparite senza traccia |
| `campi_rinviati` | documento | `dre_linha_id` vuoto sembrerebbe un dato |
| `data_pagamento` assente | riga, come `Dado` | una scadenza confusa con un pagamento |
| identità del documento (§2.3) | contesto, **non** dedotta dalle righe | un documento senza righe diventa irrintracciabile |

### 5.2 Inclusione negli output

| Output | Documento `OK` | `PARCIAL` | `ERRO` | Riga `REVISAR` in documento `OK` |
|---|---|---|---|---|
| pre-libro **diagnostico** | incluso | incluso, marcato | incluso, marcato | nella partizione di revisione |
| riepiloghi per conto/competenza | incluso | incluso, con stato del documento | incluso, con stato | contata nella categoria di revisione |
| **estratto approvato** | solo righe `APROVADO` | solo righe `APROVADO`, documento marcato | **escluso** | esclusa finché non approvata |

Nessun filtro silenzioso: ogni riga esclusa compare in un elenco di esclusioni
con **identità, valore se noto, stato e motivo** (F3C-11). Lo stato del
documento viaggia con le sue righe.

### 5.3 Importi: dove vive il problema

Il Python rende `0.00` ciò che non sa leggere (S6). In 3C **mai**. Ma —
delimitazione di §2.3 — in **3C.2 il caso non può presentarsi**, perché
`Lancamento::valor` è un `Decimal` già valido. Il contratto F3C-06 vale quindi
per l'**adattatore** di 3C.4: un importo illeggibile produce una riga
`INCOMPLETO` con il testo originale nel motivo, e i riepiloghi che la
contengono si dichiarano **incompleti**. Decisione **D12**, necessaria per
3C.4, **non** per 3C.2.

---

## 6. E — Aggregazioni e invarianti

### 6.1 Granularità e isolamento

| Dimensione | Chiave proposta | Osservato nel Python | Stato della proposta |
|---|---|---|---|
| azienda | `cnpj_empresa` normalizzato | i riepiloghi **non hanno colonna azienda**: tre aziende in una riga (S7) | **PROPOSTO**: è un nuovo requisito di granularità, non una conseguenza di C2/C10/C23 |
| azienda non identificata | partizione dichiarata `EMPRESA_NAO_IDENTIFICADA` | confluisce con le altre | **PROPOSTO** |
| competenza | `Competencia` validata | `"SEM_COMPETENCIA"` come testo | in Rust è obbligatoria; la partizione residua serve solo agli ingressi importati |
| conto | **`conta_id`** | la chiave include `conta_nome`: il conto si sdoppia (S8) | **PROPOSTO** |
| nome del conto nei conflitti | il nome del **piano dei conti** per quel `conta_id`, non quello della riga. Il piano arriva come **parametro** di `construir_pre_livro` (§8.2): una funzione pura non può leggerlo altrove, e un `conta_id` assente dal piano è un errore esplicito | il Python usa il nome della riga | **PROPOSTO**: il piano è la fonte autorevole del nome; i nomi di riga divergenti generano `NOME_DE_CONTA_DIVERGENTE` con i valori visti |
| ordinamento | `(cnpj_empresa, competencia, conta_id, chave_origem, versao, ordinal)` | `sorted` per `conta_id, conta_nome` | **PROPOSTO**: deterministico e indipendente dall'ordine d'ingresso, compatibile con I7 |

La scelta del nome dal piano dei conti è ciò che rende I7 vero: se il nome
venisse dalla prima riga incontrata, permutare l'ordine d'ingresso cambierebbe
l'etichetta del gruppo.

### 6.2 Partizioni, precedenza e invarianti

Partizioni delle righe di un lotto, **mutuamente esclusive ed esaustive**:

```text
APROVADO  ∪  OK  ∪  REVISAR  ∪  NAO_RECONHECIDO  ∪  INCOMPLETO  =  righe ricevute
```

In 3C.2 solo le prime tre sono raggiungibili (§2.3); le altre due nascono con
l'adattatore.

**Regola di precedenza** (R3C-01): una riga con più problemi appartiene a **una
sola** partizione, scelta in quest'ordine, e **conserva tutte le cause** in un
elenco di motivi:

```text
INCOMPLETO  >  NAO_RECONHECIDO  >  REVISAR  >  APROVADO  >  OK
```

Un importo illeggibile **e** uno stato sconosciuto danno quindi una riga
`INCOMPLETO` con due motivi registrati: nessuna delle due cause si perde, e la
riga non compare in due partizioni. `APROVADO` precede `OK` perché
un'approvazione esplicita non va declassata a esito automatico.

| Invariante | Formulazione |
|---|---|
| I1 conservazione | ogni riga ricevuta appartiene a esattamente una partizione; nessuno scarto senza esito esplicito |
| I2 conteggi | somma dei conteggi di partizione = righe ricevute |
| I3 somme | per ogni chiave: `valor_total = Σ valori delle partizioni incluse`, in centesimi interi; una riga `INCOMPLETO` **non** entra nel valore e marca il totale come incompleto |
| I4 documento | per ogni documento: Σ valori delle sue righe = `reconciliacao.soma_lancamentos` |
| I5 nessun dedupe aritmetico | due righe con lo stesso `hash_registro` non vengono fuse né contate una volta |
| I6 esclusioni | ogni riga esclusa da un output è elencata con identità, valore se noto, stato, motivo |
| I7 ripetibilità | permutando l'ordine dei documenti in ingresso, pre-libro e riepiloghi sono identici |
| I8 identità del documento | ogni documento del lotto, **anche senza righe**, compare con chiave, azienda, competenza e stato |

### 6.3 Metriche: definizione, non percentuali inventate

| Metrica | Numeratore | Denominatore | Incluso/escluso |
|---|---|---|---|
| righe per partizione | righe della partizione | — | conteggio, nessun denominatore |
| valore per partizione e chiave | Σ `valor` in centesimi | — | `INCOMPLETO` escluso dal valore, **contato** nelle righe, totale marcato |
| documenti per `stato_documento` | documenti nello stato | documenti del lotto | ogni documento conta una volta |

Nessuna "percentuale di completezza": `articoli_attesi` è `NAO_DISPONIVEL`,
quindi non esiste un insieme atteso noto.

### 6.4 Aritmetica: prerequisito, non ripiego (R3C-02)

La revisione 1 ammetteva un ripiego: "in 3C.2 sommare solo valori già prodotti
dalla 3B e in dominio, dichiarando il limite". **È stato eliminato: non era
sicuro.** Prova: `MAX` e `0.01` sono entrambi importi legittimi e in dominio,
ma `MAX.checked_add(0.01)` restituisce `Some(MAX)` — il centesimo scompare
senza errore. Due documenti individualmente allocabili, sullo stesso conto,
azienda e competenza, ricreerebbero nel riepilogo il difetto chiuso in R3B-01.
Una dichiarazione generica del limite non è un errore controllato.

**Contratto:** tutte le somme di 3C avvengono in **centesimi interi**, con gli
stessi helper della 3B. Serve quindi un **refactoring minimo, prerequisito del
lotto 3C.2**, non un'opzione:

| Intervento | Dettaglio |
|---|---|
| file | `src/alocacao.rs`: `em_centesimos`, `de_centesimos`, `somar_centesimos`, `dividir_meio_para_cima` e l'errore di capacità passano da privati a `pub(crate)`, oppure si spostano in `src/dinheiro.rs` usato da entrambi i moduli |
| vincolo | **nessuna duplicazione**: due verità sul denaro sarebbero peggio del problema |
| superficie pubblica | invariata verso l'esterno del crate |
| test | i 166 esistenti restano verdi senza modifiche: il refactoring non cambia comportamento |
| autorizzazione | tecnica, può essere data insieme al lotto; **non** è una nuova decisione sui numeri |

**Esito per aggregati non rappresentabili:** se la somma in centesimi non è
rappresentabile come `Decimal` nemmeno riducendo gli zeri finali, il riepilogo
**non** produce un valore troncato: restituisce `CapacidadeNumericaExcedida`
con chiave, operazione e addendi, e il pre-libro **non** viene prodotto
parzialmente. Nessun tetto monetario, nessun arrotondamento correttivo, nessun
`f64`. Test: **T-21**.

---

## 7. F — Formato, scritture e fedeltà

### 7.1 Campi e tipi dell'archivio autorevole proposto

| Gruppo | Rappresentazione |
|---|---|
| importi | **testo decimale esatto** a due decimali, mai virgola mobile |
| identificativi (`chave_origem`, `cnpj_*`, `documento`, `conta_id`) | **testo**, zeri iniziali conservati |
| date | testo ISO `YYYY-MM-DD` / `YYYY-MM` |
| assenze | oggetto `Dado`: valore assente + `status` + `fonte` + `motivo` |
| stati | etichette stabili esistenti (`OK`, `REVISAR`, `APROVADO`, `PARCIAL`, `N.A.`, …) |
| revisione | `quem`, `quando`, `motivo`, `valores_prima`; mai dedotti dall'ambiente |
| schema | `versao_schema`, `versao_formato_origem`: senza versione non esiste migrazione |

### 7.2 Round-trip: che cosa va provato

Non basta che il file si apra: **rileggere e confrontare valore per valore**,
con un lettore indipendente dal writer, su importi estremi e centesimi,
identificativi con zeri iniziali, assenze con stato e motivo, rate con scadenza
nulla, testi che iniziano con `=`, `+`, `-`, `@`, stati e campi di revisione
(F3C-21).

### 7.3 Rischi: di chi sono (riscritto dopo R3C-03)

| Rischio | Natura | Prova | Trattamento |
|---|---|---|---|
| importo numerico impreciso | **limite della cella numerica** XLSX, non del formato | S3 | importi in celle **testuali**; se numeriche, solo quando il valore è esattamente rappresentabile e con il limite dichiarato |
| testo eseguito come formula | **scelta del writer** (openpyxl interpreta `=`) | S2 + controprova §1.4 | stringa inline con `quotePrefix`, testo originale conservato |
| formula senza valore in cache | **comportamento del lettore** con `data_only` | S9 | il lettore **rifiuta** la cella e dichiara `INCOMPLETO`; non la considera zero |
| riga scartata | **algoritmo di deduplicazione**, non formato | S1 | identità e conflitti dichiarati; nessuno scarto |
| header diversi | **schema storico**, non capacità del formato | 26 vs 31 nel codice | lettura per nome di colonna con schema dichiarato; incompatibilità = **errore** |
| file aperto, errore a metà salvataggio | ambiente | non provato | temporaneo + rinomina atomica; file preesistente invariato (F3C-20) |

### 7.4 Dipendenze

| Esigenza | Crate | Disponibilità verificata |
|---|---|---|
| archivio testuale strutturato | `serde_json` | **già in cache locale** |
| impronta del documento | `sha2` | **già dipendenza** autorizzata in 3B |
| scrittura XLSX | `rust_xlsxwriter` o simile | **assente dalla cache**: serve rete e autorizzazione |
| lettura XLSX | `calamine` o simile | **assente dalla cache**: serve rete e autorizzazione |

Questa analisi **non** autorizza installazioni.

---

## 8. G — Confini, e la proposta 3C.2

### 8.1 Funzione per funzione, dalla fonte Python

| Funzione Python (commit fissato) | Decisione |
|---|---|
`excel_writer.write_lancamentos_to_excel` | **rinviato** a 3C.4 come export derivato; non adottato come fonte |
`excel_writer._dedupe_key` / `_existing_dedupe_keys` | **non portato**: scarta righe valide (S1) |
`excel_writer._duplicatas_json` | **portato** come forma delle rate |
`pre_livro_diario.gerar_pre_livro_diario_teste` | **portato in due parti**: calcolo in memoria (3C.2), scrittura (3C.4) |
`pre_livro_diario._accumulate_resumo_*` | **portato con correzioni**: chiave con azienda, conto senza nome (S7, S8) |
`pre_livro_diario._decimal_value` | **non portato**: fallback a `0.00` (S6) |
`pre_livro_diario._build_auditoria`, `_derive_status_geral` | **portato** come indicatori del lotto in 3C.2 |
`revision_report.*` | **rinviato** a 3C.3 |
`excel_diagnostics.*` | **parzialmente portato**: le sue verifiche diventano **invarianti** (§6.2), non una diagnosi a valle |
`classification_review_*`, `operation_classifier` | **fuori perimetro** |
`run_manager`, `cli` | **rinviato** a 3C.5 |
`dre_gerencial_provisoria`, `mapeamento_dre` | **fuori perimetro**: 3D |

Le operazioni speciali **non esistono** nel Rust: nessun foglio vuoto che ne
attesti l'assenza. Se l'export derivato avrà quel foglio, dichiarerà
`NAO_ANALISADO`.

### 8.2 Blocco 3C.2 — pre-libro in memoria

| Elemento | Proposta |
|---|---|
| prerequisito | **refactoring aritmetico di §6.4**, autorizzabile insieme al lotto. Senza, il lotto **non è implementabile** in modo sicuro |
| file nuovi | `src/prelivro.rs`; `src/dinheiro.rs` **se** si scelgono gli helper condivisi in modulo proprio |
| file toccati | `src/lib.rs` (una riga); `src/alocacao.rs` (sola visibilità degli helper, nessun cambio di comportamento) |
| ingresso | `construir_pre_livro(documentos: &[ContextoDocumento], plano: &PlanoContas) -> Result<PreLivro, ErroPreLivro>` |
| tipi | `ContextoDocumento`, `IdentidadeLote`, `PreLivro { linhas, documentos, resumos, indicadores, exclusoes }`, `LinhaPreLivro { identidade, lancamento, particao, motivos }`, `ResumoChave { cnpj_empresa, competencia, conta_id }`, `Particao`, `IndicadorLote`, `ErroPreLivro` |
| errori | `CapacidadeNumericaExcedida` (propagato dagli helper condivisi), `ChaveDocumentoVazia`, `DocumentoDuplicadoNoLote`, `ContaNaoEncontradaNoPlano { chave_origem, conta_id }` |
| perché il piano è un parametro | D13(a) prende il nome del conto dal **piano**: una funzione pura non può leggerlo da nessun'altra parte. Il chiamante carica e valida il piano **fuori** dalla funzione (stessa validazione C15 degli altri comandi). Non è una decisione nuova: rende realizzabile D13(a). Un `conta_id` assente dal piano è un **errore esplicito** che nomina conto e documento: nessun nome inventato, nessun nome recuperato dall'ordine d'ingresso |
| dipendenze | **nessuna nuova** |
| fuori | nessuna scrittura, nessun workbook, nessun archivio, nessun comando, nessuna transizione di stato eseguita, nessun ingresso grezzo |

**Test pianificati: 23** (`T-01…T-23`, nessuno esistente).

| ID | Test | Caso |
|---|---|---|
| T-01 | campione canonico: tre righe, 35.00, rate conservate | CAS-01 |
| T-02 | riga `REVISAR` con conto e avvisi invariati | CAS-02 |
| T-03 | `APROVADO` distinto da `OK` nelle partizioni | CAS-03 |
| T-04 | documento `PARCIAL` con righe `OK`: marcatura presente | CAS-04 |
| T-05 | documento `ERRO`: nel diagnostico, fuori dall'estratto approvato | CAS-04 |
| T-06 | **documento senza righe**: presente con chiave, azienda, competenza, stato (I8) | CAS-04 |
| T-07 | `data_pagamento` assente e scadenza nota, distinte | CAS-06 |
| T-08 | payload ripetuto: due righe, due identità, nessuno scarto | CAS-07 |
| T-09 | riconciliazione conservata per documento: quattro esiti leggibili | CAS-04 |
| T-10 | due aziende nello stesso mese: due righe di riepilogo | CAS-10 |
| T-11 | azienda non identificata: partizione propria, mai unita | CAS-10 |
| T-12 | stesso conto con nomi divergenti: un gruppo, nome dal piano, avviso | CAS-11 |
| T-13 | riepiloghi con negativi e zeri: quadrature esatte | CAS-11 |
| T-14 | precedenza di partizione con due cause: una partizione, due motivi | CAS-03 |
| T-15 | I1/I2: conservazione e conteggi | CAS-15 |
| T-16 | I3: somme per chiave in centesimi | CAS-11 |
| T-17 | I5: nessuna fusione per hash | CAS-07 |
| T-18 | I6: esclusioni elencate con identità e motivo | CAS-04 |
| T-19 | I7: permutazione dell'ordine → risultato identico | CAS-15 |
| T-20 | lotto vuoto: `PreLivro` vuoto, indicatori dichiarati, nessun errore | CAS-15 |
| T-21 | **riepilogo su più documenti la cui somma eccede `Decimal`**: errore di capacità, nessun pre-libro parziale | CAS-12 |
| T-22 | **risoluzione del conto nel piano**: nome preso dal piano, non dalla riga | CAS-11 |
| T-23 | **`conta_id` assente dal piano**: `ContaNaoEncontradaNoPlano` con conto e documento; nessun nome inventato | CAS-11 |

Il numero finale dei test **deriva dai test effettivi** dell'implementazione:
23 è l'elenco pianificato, non una quota da raggiungere.

**Criteri di accettazione 3C.2**

1. i test pianificati verdi (23 nell'elenco, numero finale dal codice) **più i 166 esistenti** senza regressioni;
2. `cargo test --locked`, `fmt --check`, `clippy -D warnings`, `git diff --check` puliti;
3. sei fonti con SHA-256 invariato;
4. nessun `f64`; nessuna somma fuori dai centesimi interi; nessun aggregato troncato;
5. nessuna riga ricevuta senza partizione; ogni esclusione motivata; ogni documento presente anche senza righe;
6. funzione pura: nessun IO;
7. il refactoring aritmetico non cambia il comportamento della 3B: i test 3B restano identici e verdi.

### 8.3 Decisioni necessarie **a 3C.2** e decisioni dei blocchi successivi

| Decisione | Necessaria a 3C.2? |
|---|---|
| **D10** inclusione di documenti/righe non riconciliati negli output | **sì**: determina partizioni ed esclusioni del modulo puro |
| **D13** granularità delle chiavi di riepilogo (azienda, conto, nome dal piano) | **sì**: è il cuore dei riepiloghi |
| D7 fonte autorevole | no: 3C.2 non scrive |
| D8 identità persistente, versioni, impronta | no: serve a 3C.3/3C.4 |
| D9 significato dell'approvazione | no: 3C.2 non esegue transizioni |
| D11 import dello storico | no |
| D12 importi illeggibili | no: non rappresentabili in 3C.2 (§2.3); serve a 3C.4 |

### 8.4 I blocchi successivi

| Blocco | Contenuto | Dipende da |
|---|---|---|
| **3C.3** | stati e revisione: transizioni, tracciamento, preservazione alla rigenerazione | D8, D9 |
| **3C.4** | adattatori di ingresso grezzo, archivio autorevole, export derivato, round-trip | D7, D11, D12, dipendenze XLSX |
| **3C.5** | uso da CLI, validazione C15 condivisa | 3C.3, 3C.4 |

**La 3C non è completa col solo modulo puro.**

---

## 9. Matrice dei contratti F3C

| ID | Regola | Fonte e verifica | Stato | Input/assenza/conflitto | Esito | Codice futuro | Test |
|---|---|---|---|---|---|---|---|
| F3C-01 | Il pre-libro consuma `AlocacaoNfe` + contesto tipizzato, **non** un workbook | `pre_livro_diario.py:257` legge Excel | **PROPOSTO** | nessun workbook richiesto | pre-libro in memoria | `prelivro.rs` | T-01 |
| F3C-02 | Ogni riga ricevuta in esattamente una partizione; nessuno scarto implicito | S1 | **PROPOSTO** | riga duplicata | conservata e classificata | `prelivro.rs` | T-08, T-15 |
| F3C-03 | Partizioni con **precedenza dichiarata**; tutte le cause conservate | §6.2 | **PROPOSTO** | riga con due problemi | una partizione, più motivi | `prelivro.rs` | T-14 |
| F3C-04 | Nessuna promozione automatica ad `APROVADO` | `excel_writer.py:73` | **VINCOLO_GIA_APPROVATO** (metodo) | — | solo transizione umana | 3C.3 | T-03 |
| F3C-05 | L'approvazione non cancella avvisi né cambia gli esiti tecnici | C24 | **PROPOSTO** | approvazione su riga con avvisi | avvisi ed esiti invariati | 3C.3 | — (3C.3) |
| F3C-06 | Importo non interpretabile = `INCOMPLETO` con testo originale; **mai** `0.00` | S6 | **PROPOSTO** (D12) | solo da ingresso grezzo | riga incompleta, totale marcato | **3C.4** | — (3C.4) |
| F3C-07 | Partizioni mutuamente esclusive, conteggi riconciliati (I1, I2) | — | **PROPOSTO** | — | errore interno se violato | `prelivro.rs` | T-15 |
| F3C-08 | La chiave dei riepiloghi include l'azienda; azienda non identificata è partizione dichiarata | S7 | **PROPOSTO** (D13, nuovo requisito di granularità) | `cnpj_empresa` vuoto | mai unita ad altra azienda | `prelivro.rs` | T-10, T-11 |
| F3C-09 | Riepilogo per conto su `conta_id`; nome dal **piano ricevuto come parametro**; nomi di riga divergenti → avviso; `conta_id` assente dal piano = errore esplicito | S8 | **PROPOSTO** (D13) | due nomi per un `conta_id`; conto fuori piano | un gruppo + `NOME_DE_CONTA_DIVERGENTE`; oppure `ContaNaoEncontradaNoPlano` | `prelivro.rs` | T-12, T-22, T-23 |
| F3C-10 | Competenza validata in Rust; partizione residua solo per ingressi importati | C8 | **VINCOLO_GIA_APPROVATO** | competenza assente in import | partizione dichiarata | 3C.4 | — (3C.4) |
| F3C-11 | Inclusione per output dichiarata; esclusioni con identità, valore, stato, motivo | §5.2 | **PROPOSTO** (D10) | documento `ERRO`, riga `REVISAR` | nessun filtro silenzioso | `prelivro.rs` | T-05, T-18 |
| F3C-12 | Riconciliazione del documento conservata, non appiattita | `Reconciliacao`, 15 campi | **PROPOSTO** | — | quattro esiti per documento | `prelivro.rs` | T-09 |
| F3C-13 | Identità di riga con **scope** `(lote, chave, versao)` + `(origem, conta_id, ordinal)` e stabilità definita | C18 | **PROPOSTO** (D8) | riallocazione, cambio regole | nuova versione, non identità casuale | `prelivro.rs` / 3C.4 | T-08 |
| F3C-14 | Reimportazione identica: nessun duplicato, nessuna perdita | S1 | **PROPOSTO** (D8) | stessa esecuzione due volte | esito "ripetizione" | 3C.4 | — (3C.4) |
| F3C-15 | Correzione o cambio regole: nuova versione, storico conservato | §3.1 caso 3 | **PROPOSTO** (D8) | hash invariato, contenuto diverso | versioni affiancate | 3C.4 | — (3C.4) |
| F3C-16 | Importi nel formato autorevole come testo decimale esatto; mai `float` | S3 | **PROPOSTO** (D7) | valore oltre la capacità | errore dichiarato | 3C.4 | — (3C.4) |
| F3C-17 | Identificativi come testo, zeri iniziali conservati | S4 | **EREDITATO_VERIFICATO** | `00012345678901` | conservato | 3C.4 | — (3C.4) |
| F3C-18 | Testo che inizia con `=`, `+`, `-`, `@`: conservato, non eseguito | S2 + controprova §1.4 | **PROPOSTO** | `=1+1 SUCO` | testo intatto | 3C.4 | — (3C.4) |
| F3C-19 | Schema versionato; header incompatibili = errore, non adattamento | 26 vs 31 nel codice | **PROPOSTO** (D11) | layout storico diverso | errore con la differenza nominata | 3C.4 | — (3C.4) |
| F3C-20 | Scrittura atomica; in errore il file preesistente resta invariato | non provato | **PROPOSTO** | file aperto, percorso invalido | temporaneo + rinomina | 3C.4 | — (3C.4) |
| F3C-21 | Round-trip verificato valore per valore, lettore indipendente | controprova §1.4 | **PROPOSTO** | — | confronto esatto | 3C.4 | — (3C.4) |
| F3C-22 | Campi di revisione umana e righe manuali preservati alla rigenerazione | `excel_writer.py:119-121` | **EREDITATO_VERIFICATO** + PROPOSTO per l'estensione | rigenerazione | annotazioni conservate | 3C.3 | — (3C.3) |
| F3C-23 | Nessuna cancellazione o sostituzione dello storico senza decisione | metodo | **VINCOLO_GIA_APPROVATO** | conflitto di versioni | stato `CONFLITO` | 3C.4 | — (3C.4) |
| F3C-24 | Operazioni speciali, CSV, DRE fuori perimetro e dichiarati tali | `operation_classifier` non portato | **PROPOSTO** | — | etichetta `NAO_ANALISADO` | 3C.4 | — (3C.4) |
| F3C-25 | Ordinamento deterministico; permutazioni indifferenti (I7) | `sorted` Python | **PROPOSTO** (D13) | ordine d'ingresso diverso | risultato identico | `prelivro.rs` | T-19 |
| F3C-26 | **Aggregati in centesimi interi, helper condivisi con la 3B**; aggregato non rappresentabile = errore, nessun troncamento | R3B-01, R3C-02 | **PROPOSTO** (tecnico, prerequisito) | somma oltre `Decimal` | `CapacidadeNumericaExcedida` | `dinheiro.rs` / `alocacao.rs` | T-16, T-21 |
| F3C-27 | Ogni documento del lotto compare, **anche senza righe**, con la sua identità (I8) | verificato: `Reconciliacao` non porta chiave/azienda/competenza | **PROPOSTO** | documento senza prodotti | presente nel pre-libro | `prelivro.rs` | T-06 |

Nessuno di questi test esiste. `PROPOSTO` e `DA_DECIDERE` **non** sono
approvati da questa specifica.

### 9.1 I quindici casi richiesti dal mandato

| ID | Caso | Stato della prova | Blocco |
|---|---|---|---|
| CAS-01 | campione canonico: 3 registrazioni, 35.00, rate conservate | **eseguito** in 3B | 3C.2 (T-01) |
| CAS-02 | riga sconosciuta/ambigua `REVISAR`, conto e avvisi invariati | **eseguito** in 3B | 3C.2 (T-02) |
| CAS-03 | `APROVADO` distinto da `OK`; stato vuoto/sconosciuto | **eseguito** sul Python (S5) | 3C.2 (T-03, T-14) per `APROVADO`; **3C.4** per gli stati grezzi |
| CAS-04 | documento `PARCIAL` con righe `OK`; documento `ERRO`; documento senza righe | **eseguito** in 3B | 3C.2 (T-04, T-05, T-06, T-09, T-18) |
| CAS-05 | importo mancante/non valido e zero reale | **eseguito** sul Python (S6) | **3C.4**: non rappresentabile in 3C.2 |
| CAS-06 | `data_pagamento` assente, scadenza nota | **eseguito** in 3B | 3C.2 (T-07) |
| CAS-07 | payload uguali nello stesso documento | **eseguito** in 3B e sul Python (S1) | 3C.2 (T-08, T-17) |
| CAS-08 | reimportazione identica; documento corretto con hash invariato | da creare | 3C.4 |
| CAS-09 | correzione/approvazione manuale preservata o conflitto | da creare | 3C.3/3C.4 |
| CAS-10 | due aziende nello stesso mese; azienda non identificata | **eseguito** sul Python (S7: unite) | 3C.2 (T-10, T-11) |
| CAS-11 | riepiloghi con negativi, zeri, quadrature; nomi di conto divergenti | **eseguito** sul Python (S8) | 3C.2 (T-12, T-13, T-16) |
| CAS-12 | round-trip di importi grandi e identificativi con zeri iniziali | **eseguito** sul Python (S3, S4) e controprova §1.4 | 3C.4; la parte aritmetica in 3C.2 (T-21) |
| CAS-13 | header/schema errato, JSON rate invalido, formula senza valore | **eseguito** sul Python (S2, S9, S10; 26 vs 31) | 3C.4 |
| CAS-14 | errore di salvataggio/percorso, file preesistente invariato | da creare | 3C.4 |
| CAS-15 | input permutati: ordinamento e identità | da creare | 3C.2 (T-15, T-19, T-20) |

---

## 10. Divergenze Python / Rust proposto

Separate per natura, come chiesto dall'audit.

### 10.1 Difetti provati della fonte, rispetto a vincoli già vigenti

| Tema | Comportamento | Prova | Proposta | Vincolo violato |
|---|---|---|---|---|
| seconda riga con payload uguale scartata | silenziosamente | S1 | conservata, identità distinta | C23 / F3A-27 |
| importo illeggibile → `0.00` | silenzioso | S6 | `INCOMPLETO` con motivo | C2 / C10 |
| formula senza valore in cache → `0.00` | silenzioso | S9 | cella rifiutata, riga `INCOMPLETO` | C2 / C10 |
| testo che inizia con `=` eseguito come formula e perduto | alterazione del dato | S2 | testo conservato, nessuna esecuzione | C14 (il parser non altera il contenuto) |

### 10.2 Limiti del formato o del writer, non difetti logici

| Tema | Natura | Prova | Proposta |
|---|---|---|---|
| importo grande in cella numerica | limite della cella, non del formato | S3 + controprova §1.4 | testo decimale esatto |
| due schemi di export (26, 31) | evoluzione dello schema | lettura del codice | `versao_schema` dichiarata; nessun file storico verificato |

### 10.3 Nuovi requisiti di granularità: **PROPOSTI**, non derivati dai vincoli

| Tema | Comportamento osservato | Requisito proposto | Nota |
|---|---|---|---|
| riepilogo mensile senza azienda | aziende unite (S7) | chiave con azienda | C2/C10/C23 **non** definiscono le chiavi di riepilogo: è una scelta nuova (D13) |
| riepilogo per conto con `conta_nome` nella chiave | conto sdoppiato (S8) | chiave `conta_id`, nome dal piano | idem |
| `APROVADO` trattato come "da revisionare" | sì (S5) | partizione propria | scelta di porting (D9/D10) |
| stato vuoto → revisione | sì (S5) | `NAO_RECONHECIDO` distinto | scelta di porting (D12) |

### 10.4 Parità voluta

`MANUAL` escluso dal dedupe (le righe umane non si perdono); rate come JSON
ordinato; indicatori di audit del lotto.

### 10.5 Alternative NON AMMISSIBILI

Non sono opzioni approvabili: contraddicono vincoli vigenti. Restano qui come
controesempi.

| Alternativa | Perché non ammissibile |
|---|---|
| zero tecnico per un dato ignoto | viola C2/C10, già chiuso in 3A/3B |
| `hash_registro` come identità unica che scarta righe | viola C23 e F3A-27: scarta registrazioni valide (S1) |
| approvazione che riscrive gli esiti tecnici | cancella la memoria del problema; viola C24 |
| somma monetaria in `Decimal` fuori dai centesimi interi | perde un centesimo senza errore (§6.4) |
| `f64` per gli importi | viola C1 |

---

## 11. Decisioni funzionali da consegnare

Tutte **PROPOSTO**, nessuna approvata da questa specifica. Le prime due servono
a 3C.2; le altre ai blocchi successivi (§8.3).

### D10 — Inclusione di documenti e righe non riconciliati *(necessaria a 3C.2)*

- **(a)** diagnostico e riepiloghi includono tutto, marcato; l'estratto
  approvato esclude i documenti `ERRO` e le righe non approvate, elencando le
  esclusioni (proposta).
- (b) escludere i documenti `ERRO` da tutti gli output: il problema diventa
  invisibile.
- (c) includere tutto anche nell'estratto approvato: manda in contabilità
  documenti che non quadrano.

### D13 — Granularità delle chiavi di riepilogo *(necessaria a 3C.2)*

- **(a)** chiave `(cnpj_empresa, competencia, conta_id)`, nome del conto dal
  **piano dei conti ricevuto come parametro**, nomi di riga divergenti
  conservati e segnalati; azienda non identificata in partizione etichettata,
  mai assimilata a un'azienda identificata; ordinamento deterministico e nessuna
  deduzione di identità da nomi simili (proposta).
- (b) parità col Python: nessuna azienda nella chiave e `conta_nome` dentro la
  chiave → aziende unite (S7) e conti sdoppiati (S8).
- (c) chiave con azienda ma nome del conto dalla prima riga incontrata: rompe
  I7, perché permutare l'ordine cambierebbe l'etichetta.

### D7 — Fonte autorevole *(3C.4)*

- **(a) archivio strutturato versionato + workbook derivato** (proposta):
  ingresso umano validabile, nessuna dipendenza di rete per il formato
  autorevole, rigenerazione confrontabile. Costo: la verità non si apre in
  Excel.
- (b) workbook autorevole **con contratti espliciti**: la controprova §1.4
  mostra che è rappresentabile. Costo: schema molto più grande, due dipendenze
  XLSX, validazione di ogni cella modificabile, perdita dei calcoli sulle
  colonne testuali.
- (c) entrambi autorevoli: due verità sullo stesso dato; sconsigliata.

### D8 — Identità persistente, versioni, impronta *(3C.3/3C.4)*

- **(a)** identità con scope `(lote, chave, versao)` + `(origem, conta_id,
  ordinal)`, `VersaoDocumento` con impronta, conflitti dichiarati (proposta).
  **Il testo canonico dell'impronta resta da contrattualizzare** (§3.3).
- (b) identità = `hash_registro`: due righe legittime diventano una (S1).
- (c) UUID per riga: unico ma non riproducibile.

### D9 — Significato operativo dell'approvazione *(3C.3)*

- **(a)** "visto e accettato", con `quem`/`quando`/`motivo` dichiarati; non
  cancella avvisi; non si trasferisce a una nuova versione (proposta).
- (b) l'approvazione sostituisce gli esiti tecnici: **NON AMMISSIBILE** (§10.5).
- (c) nessuno stato di approvazione in 3C: lascia il pre-libro senza uscita
  contabile.

### D11 — Import dello storico Excel *(3C.4)*

- **(a)** rinviato, perimetro dichiarato: il flusso nuovo non dipende da uno
  storico non fornito (proposta).
- (b) import subito: richiede un lettore XLSX, la gestione di **almeno due
  schemi** (26 e 31, osservati nel codice, nessun file verificato) e i difetti
  provati del formato.

### D12 — Importi non interpretabili in ingresso *(3C.4)*

- **(a)** riga `INCOMPLETO` con testo originale e motivo; totali che la
  contengono dichiarati incompleti (proposta).
- (b) rifiuto dell'intero lotto: blocca per una cella.
- (c) parità col Python: `0.00`, **NON AMMISSIBILE** (§10.5).

Scelte tecniche ed editoriali già prese qui, senza effetto funzionale: nomi di
tipi, moduli e test, forma dei messaggi, ordine dei campi nei record.

---

## 12. Limiti di questa specifica

| Limite | Stato |
|---|---|
| Suite Python del perimetro | **ESEGUITA**: 47 test passati al commit fissato, confermati dall'audit. Il resto della suite Python e `test_fase3_end_to_end.py` non sono stati eseguiti |
| Prima esecuzione pytest fallita | **spiegata**: `py -I` ignora `PYTHONPATH` e importava il working tree (31 colonne) |
| Workbook storici reali | **nessuno fornito o verificato**: due schemi osservati nel **codice**, non nei dati |
| Testo canonico dell'impronta di versione (D8) | **rinviato e dichiarato**: struttura proposta, algoritmo non ancora contrattualizzato |
| `F3C-14`, `F3C-20`, `F3C-21` | **NON_VERIFICATI**: dipendono da IO che non esiste |
| Dipendenze XLSX | **assenti** dalla cache locale: valutazione su API dichiarate, non su prove |
| Refactoring aritmetico | **prerequisito dichiarato** di 3C.2, non eseguito |
| `articoli_attesi` | resta `NAO_DISPONIVEL`: la copertura misura l'assenza di avvisi di scarto |
| Operazioni speciali, CSV, DRE, regole YAML | fuori perimetro, dichiarati non analizzati |
| Suite Rust | baseline 166 riusata, non rieseguita: nessun file di codice toccato |

---

## 13. Esito dell'implementazione 3C.2 (2026-10-17)

Approvazione di Luca il 2026-10-17: **D10(a), D13(a), lotto 3C.2 e refactoring
tecnico minimo**. Implementato lo stesso giorno. Primo audit: tre riscontri,
corretti in §14; **audit mirato superato il 2026-10-17**. Nessun commit,
nessun push: la registrazione resta da autorizzare.

### Che cosa è stato scritto

| File | Natura | Contenuto |
|---|---|---|
| `src/dinheiro.rs` | nuovo, 240 righe | aritmetica monetaria **unica del crate**: `em_centesimos`, `de_centesimos`, `somar_centesimos`, `dividir_meio_para_cima`, `centavos`, `no_dominio`, `ErroDinheiro`; 6 test propri |
| `src/prelivro.rs` | nuovo, 2.032 righe con i test | `construir_pre_livro`, `ContextoDocumento`, `IdentidadeLote`, `PreLivro`, `LinhaPreLivro`, `DocumentoPreLivro`, `ResumoChave`, `Resumo`, `Exclusao`, `Indicadores`, `Particao`, `IdentidadeLinha`, `ErroPreLivro` (6 varianti), 3 avvisi; 27 test |
| `src/alocacao.rs` | **sola delega** | i quattro helper diventano wrapper di `dinheiro` con firme identiche; aggiunta la traduzione `ErroDinheiro → ErroAlocacao` che **preserva i messaggi**; rimossi due helper diventati morti. −121/+38 righe, nessun cambio di comportamento |
| `src/lib.rs` | 2 righe | espone `prelivro`, dichiara `dinheiro` come `pub(crate)` |
| `README.md` | documentazione | fasi 3C.1…3C.5, contratti `F3C-nn` del blocco con i test che li proteggono, conteggio |

**Numerazione:** i contratti restano `F3C-nn` e **non** passano a `C25+` prima
dell'audit, come chiesto. Il README lo dichiara.

### Scelte tecniche, già risolte senza nuove decisioni

| Scelta | Perché |
|---|---|
| modulo `dinheiro.rs` invece di `pub(crate)` sugli helper di `alocacao.rs` | il pre-libro non dipende dall'allocatore per sommare; il modulo rende la dipendenza esplicita e tiene **una sola** verità sul denaro |
| `ErroDinheiro` separato, tradotto dai due moduli | ogni modulo conserva i propri errori e i propri messaggi: i test 3B che asseriscono `operacao` e testo restano verdi senza modifiche |
| `PreLivro::lote: Option<IdentidadeLote>` | un lotto vuoto non ha identità: `None` invece di una stringa inventata |
| `Particao` con tre varianti | `INCOMPLETO` e `NAO_RECONHECIDO` non sono rappresentabili da ingressi tipizzati: nascono con gli adattatori 3C.4, e dichiararli ora sarebbe codice morto |
| `extrato_aprovado()` come vista | l'estratto è un dato in memoria, non una contabilizzazione |

### Verifiche

```text
cargo test --locked                                  199 passati, 0 falliti
                                                     (183 lib + 3 bin + 13 CLI)
cargo fmt --check                                    exit 0
cargo clippy --locked --all-targets -- -D warnings   exit 0
git diff --check                                     exit 0
sei fonti contro il manifesto della salvaguardia     6/6 invariate
```

Da 166 a 199: **33 test nuovi** (6 in `dinheiro`, 27 in `prelivro`), **nessuna
regressione** sui 166 preesistenti. I 53 test della 3B passano invariati dopo
il refactoring: è la prova che il comportamento non è cambiato.

| Criterio di accettazione §8.2 | Esito |
|---|---|
| 1. test pianificati verdi + 166 esistenti | **sì**: T-01…T-23 implementati, più 4 test di identità di lotto e documento e la propagazione degli avvisi |
| 2. `test`/`fmt`/`clippy`/`diff --check` | **sì**, tutti puliti |
| 3. sei fonti invariate | **sì**, 6/6 |
| 4. nessun `f64`, somme solo in centesimi, nessun aggregato troncato | **sì**: nessuna occorrenza di `f64` fuori da un commento, ogni somma passa per `dinheiro` |
| 5. nessuna riga senza partizione, esclusioni motivate, documenti senza righe presenti | **sì**: T-15, T-18, T-06 |
| 6. funzione pura | **sì**: zero occorrenze di `std::fs`/`File` in `prelivro.rs` |
| 7. il refactoring non cambia il comportamento della 3B | **sì**: i test 3B invariati e verdi |

### Un errore mio, corretto in corso d'opera

Il test `t22` asseriva `grupo_conta = "Custos de Mercadorias e Insumos"` per il
conto `2.04.001`; il piano canonico dice `"Materiais de Limpeza e Higiene"`.
Era sbagliata l'attesa del test, non il codice: corretta leggendo il piano.

### Limiti residui di questo blocco

| Limite | Stato |
|---|---|
| `INCOMPLETO` / `NAO_RECONHECIDO` | non rappresentabili: nascono con gli adattatori 3C.4 |
| versione del documento e impronta | non implementate: appartengono a 3C.3/3C.4, e il testo canonico dell'impronta resta da contrattualizzare |
| nessun IO, archivio, export, CLI | voluto: il blocco è una libreria pura |
| `articoli_attesi` | resta `NAO_DISPONIVEL`: la copertura misura l'assenza di avvisi di scarto |
| D7, D8, D9, D11, D12 | **aperte**, non necessarie a questo blocco |

---

## 14. Correzione dei riscontri dell'audit 3C.2 (2026-10-17)

Audit indipendente: **non approvato alla chiusura**, tre riscontri bloccanti.
Tutti e tre **riprodotti in esecuzione** da me prima di toccare il codice, con
un file di test temporaneo che chiamava la sola API pubblica e che e' stato
rimosso dopo la correzione.

### R3C2-01 — contesto e allocazione potevano riferirsi a documenti diversi

Riprodotto: abbinando il contesto di un documento all'allocazione di un altro,
`construir_pre_livro` restituiva `Ok` e il valore del campione finiva sotto
un'altra azienda e un altro mese.

```text
identita' riga   : "CHAVE-AUDIT-201"      chiave reale : 2926...901
azienda riepilogo: 00000000000001         azienda reale: 55746722000181
competenza riep. : 2026-06                reale        : 2026-05
```

C'era anche un'incoerenza interna mia: la chiave del riepilogo veniva dal
**contesto**, mentre la chiave di ordinamento veniva dalla **riga**.

Correzione: `verificar_coerencia` confronta i cinque campi che identificano il
documento prima di aggregare — `chave_origem`, `documento`/`numero`,
`data_documento`, `competencia`, `cnpj_empresa` — e un disaccordo e'
`ErroPreLivro::DocumentoIncoerente { chave_origem, campo, no_contexto,
no_lancamento }`. I campi **non** confrontati (`serie`, `nome_empresa`,
`arquivo_origem`) sono dichiarati nella nota di modulo. Le righe non vengono mai
riscritte, e un documento senza registrazioni continua a usare il contesto.
Inoltre `LinhaPreLivro` porta ora `chave_resumo`: **una sola** fonte per
azienda, competenza e conto, usata anche per l'ordinamento.

Regressioni: `r1_chave_discordante_*`, `r1_empresa_discordante_*`,
`r1_competencia_discordante_*` e il controllo positivo
`r1_contexto_coerente_e_aceito_sem_tocar_os_lancamentos`, che verifica anche che
la registrazione resti **identica**.

**Effetto collaterale utile:** la validazione ha smascherato che il mio helper
di test costruiva contesti incoerenti (`chave_origem` fissa nelle righe). Dodici
test passavano su dati che il contratto ora rifiuta. L'helper e' stato reso
coerente, come fa il chiamante reale.

### R3C2-02 — i riepiloghi non marcavano i documenti PARCIAL/ERRO

Riprodotto: lo stesso documento, allocato una volta sano e una volta con
copertura `PARCIAL`, produceva riepiloghi **byte-identici**.

Correzione: `Resumo` porta ora `documentos_ok`, `documentos_parcial`,
`documentos_erro` e `documentos_marcados` — le chiavi dei documenti non `OK`
che alimentano **quel** gruppo, in ordine canonico. Il conteggio e' per
**documento distinto**, non per riga. Le righe `OK` restano `OK`: la marcatura
e' del documento, non della riga. Un problema in un gruppo non marca un gruppo
estraneo: provato da `r2_problema_de_um_grupo_nao_marca_grupo_alheio`, che
verifica anche che l'esclusione dei documenti `ERRO` dall'estratto resti.

### R3C2-03 — risultato dipendente dall'ordine dei documenti

Riprodotto in due punti: `nomes_divergentes` e gli avvisi globali conservavano
l'ordine di arrivo, e `PreLivro.lote` **teneva in silenzio i metadati del primo
documento** mentre il controllo confrontava solo l'`id`:

```text
lotto diretto  : gerado_em 2026-05-30, descricao "primeira"
lotto invertito: gerado_em 2026-05-31, descricao "segunda"
```

Correzione: gli avvisi del lotto sono ordinati per `(codigo, detalhe)`, i nomi
divergenti per testo, e i documenti del riepilogo arrivano da una `BTreeMap`;
`verificar_lote` confronta **tutta** `IdentidadeLote` e un disaccordo e'
`ErroPreLivro::MetadadosDoLoteDivergentes { lote, campo, esperado, encontrado,
chave_origem }` — in entrambi gli ordini d'ingresso. Non vengono riordinati
articoli, ordinali o dati interni delle registrazioni 3B.

Regressioni: `r3_permutacao_com_avisos_e_nomes_divergentes_e_identica`, che
confronta l'**intero** `PreLivro`; `r3_metadados_do_lote_discordantes_sao_conflito_nos_dois_sentidos`;
il controllo positivo `r3_lote_inteiramente_identico_e_aceito`. Il vecchio T-19
resta, ma non era sufficiente: usava un caso senza avvisi.

### Verifiche dopo la correzione

```text
cargo test --locked                                  208 passati, 0 falliti
                                                     (192 lib + 3 bin + 13 CLI)
cargo test --locked --lib alocacao::                 53 passati (3B invariata)
cargo fmt --check                                    exit 0
cargo clippy --locked --all-targets -- -D warnings   exit 0
git diff --check                                     exit 0
sei fonti contro il manifesto della salvaguardia     6/6 invariate
```

Da 199 a 208: **9 regressioni nuove**, nessuna regressione sui test
preesistenti. `clippy` ha segnalato un `clone` evitabile in un test nuovo:
corretto, non silenziato.

### Stato dei riscontri

| Riscontro | Stato |
|---|---|
| R3C2-01 | **CHIUSO** dall'audit mirato del 2026-10-17, con controllo positivo |
| R3C2-02 | **CHIUSO** dall'audit mirato, incluso il caso del gruppo estraneo |
| R3C2-03 | **CHIUSO** dall'audit mirato in entrambe le cause |

Nessuna decisione di dominio e' stata cambiata: D10(a) e D13(a) restano quelle
approvate, D7/D8/D9/D11/D12 restano rinviate. Nessun commit, nessun push,
nessuna fase 3C.3/3C.4/3C.5 avviata.

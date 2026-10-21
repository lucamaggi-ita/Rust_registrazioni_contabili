# Specifica fase 3C.3 — revisione umana, versioni e impronta canonica

| Voce | Valore |
|---|---|
| Data | 2026-10-17, **revisione 4** del 2026-10-21 |
| Revisione 2 | correzione dei riscontri `R3C3-01` e `R3C3-02` e delle due rettifiche del primo audit del 2026-10-21: §11 |
| Revisione 3 | correzione del **residuo di `R3C3-01`** rilevato dall'audit mirato sulla revisione 2: identità esatta e ordine canonico sono ora due nozioni separate, §5.4.2 e §11 |
| Revisione 4 | correzione del secondo residuo di `R3C3-01` (`C4`, più `C5` e `C2` per la stessa causa): la codifica esatta dell'evento ha ora produzioni proprie che non perdono né il `motivo` dei `Dado` né la forma Unicode originale, §5.4.2 e §11 |
| Mandato | `MANDATO_CLAUDE_FASE3C3_REVISIONE_SPECIFICA_20261017.md` |
| Perimetro | **analisi e scrittura di specifica**. Nessuna riga di Rust, nessuna modifica di `Cargo.*`, nessun writer, archivio, CLI, export, import o produzione |
| Stato | Specifica: **AUDIT TECNICO SUPERATO** il 2026-10-21. Lotto `3C.3.a`: **implementato e in attesa di audit** (§13), con i riscontri `R3C3A-01/02/03` corretti (§13.6). La fase 3C.3 **non** è chiusa come fase software: manca 3C.3.b, e la 3C nel suo insieme richiede 3C.4 e 3C.5 |
| HEAD alla stesura | `37c0e039c3c9366d505dac07e0bf7a2d97661575` |
| Decisioni | **D8.1–D8.5 e D9.1–D9.5 approvate da Luca il 2026-10-21**, tutte nell'opzione (a): §12. D10(a) e D13(a) restano approvate e non vengono riaperte. D7, D11 e D12 restano rinviate |
| Metodo | `programmatore-bravo`: prima contrattualizza, poi programma |

---

## 0. Come leggere questo documento

Ogni affermazione porta un'etichetta. Senza etichetta, è struttura del documento.

| Etichetta | Significato |
|---|---|
| **[VERIFICATO]** | letto nel codice di questo repository durante la stesura, con file e riga citati |
| **[RIUSATO]** | evidenza di un audit o di una fase precedente, riusata senza rieseguirla, con la fonte nominata |
| **[SONDA]** | prova eseguita per questo documento: input, algoritmo, runtime ed esito sono in §1.3 |
| **[PROPOSTA]** | scelta di questo documento, non approvata da nessuno |
| **[APPROVATO]** | **decisione funzionale** approvata da Luca il 2026-10-21 (§12). Approvata la decisione, **non** il codice: il contratto che la applica resta da scrivere e i suoi test restano pianificati |
| **[NON_VERIFICATO]** | affermazione plausibile che nessuno ha provato; dichiarata tale e non usata come fondamento |

Regole che valgono su tutto il testo:

1. Nessun test Rust di questo blocco esiste. Dove scrivo "test" intendo **test pianificato** (§7), mai eseguito.
2. Le sonde di §1.3 sono prototipi in PowerShell su dati sintetici: fissano **byte e impronte**, non sono codice di produzione e non devono diventarlo.
3. I documenti precedenti contengono fotografie storiche (per esempio "registrazione da autorizzare", o helper ancora privati in `alocacao`). Lo stato corrente viene da Git e dai sorgenti, non da quelle frasi.

---

## 1. Baseline, prove eseguite, prove riusate

### 1.1 Stato del repository

**[VERIFICATO]** al momento della stesura:

```text
git status -sb                   ## main...origin/main
                                 ?? MANDATO_CLAUDE_FASE3C3_REVISIONE_SPECIFICA_20261017.md
git diff --name-status           (vuoto)
git diff --cached --name-status  (vuoto)
git rev-parse HEAD               37c0e039c3c9366d505dac07e0bf7a2d97661575
```

Impronte SHA-256 dei quattro file del lotto 3C.2, **[VERIFICATO]**:

| File | SHA-256 | Righe |
|---|---|---|
| `src/alocacao.rs` | `0527dcd7d77fdaeadfefed6783efb04b45ffde51cd203a712183e4e94fc009a2` | 2579 |
| `src/dinheiro.rs` | `143f696ca88da5afe57a4bd153be69e7c9b301d39c7f407f7e76fa4594ea001d` | 240 |
| `src/lib.rs` | `1ac3da1bbeff2e28806de56cdfff07efdd1575286b5a252ff086cb5d1b73b3b1` | 21 |
| `src/prelivro.rs` | `1b9d565d8f6235ecc77209edced557bc98dabac841a12ccc0f00be38feeaa1a0` | 2619 |

Coincidono con lo snapshot dell'audit finale 3C.2: il codice non è stato toccato.

### 1.2 Fatti riverificati nel codice

Questi fatti sono il fondamento della specifica. Ognuno è stato riletto durante la stesura.

| # | Fatto | Dove | Etichetta |
|---|---|---|---|
| V1 | la fase 3B produce **soltanto** `StatusRevisao::Ok` o `Revisar`: `Aprovado` non è mai assegnato fuori dai test | `alocacao.rs:1188-1192` (`montar_lancamento`), più `:1067` e `:1162` che forzano `Revisar` | **[VERIFICATO]** |
| V2 | `Lancamento::observacoes` nasce vuoto in 3B e nessuna funzione lo scrive; è anche dichiarato fra i `campi_rinviati` | `alocacao.rs:1215`, `:1329` | **[VERIFICATO]** |
| V3 | `hash_registro` (C23) è SHA-256 di `ORIGEM|chave_origem|conta_id|valor` con due decimali fissi | `alocacao.rs:810-815` (`payload_hash`), `:1051` | **[VERIFICATO]** |
| V4 | due righe dello stesso documento possono condividere il payload: si conservano entrambe con avviso `HASH_DUPLICADO_NO_DOCUMENTO`, nessuno scarto | `alocacao.rs:1044-1062`; test a `:2037` | **[VERIFICATO]** |
| V5 | `IdentidadeLinha` **contiene `lote`** e **non** contiene `versao`; `ordinal` è l'indice della registrazione nel documento | `prelivro.rs:320-331`, `:684-690` (`ordinal: indice as u32`) | **[VERIFICATO]** |
| V6 | `construir_pre_livro` confronta cinque campi fra contesto e registrazione (`chave_origem`, `documento`, `data_documento`, `competencia`, `cnpj_empresa`) e rifiuta il disaccordo | `prelivro.rs:808-870` (`verificar_coerencia`) | **[VERIFICATO]** |
| V7 | il nome autorevole del conto viene dal `PlanoContas` ricevuto come parametro; il nome di riga divergente resta conservato e segnalato | `prelivro.rs:588-608`, campo `conta_nome_plano` | **[VERIFICATO]** |
| V8 | `no_extrato_aprovado` = riga `APROVADO` **e** documento non `ERRO`; ogni esclusione è elencata con identità, valore, partizione, stato e motivo | `prelivro.rs:610-612`, `exclusoes_do_extrato` a `:933-960` | **[VERIFICATO]** |
| V9 | i `Dado` prodotti dalle fasi 2 e 3B hanno `fonte` di forma XPath (`nfe:det`, `nfe:cobr/dup/vDup`, `nfe:total/ICMSTot/vProd`): nessun percorso di file, nessun dato d'ambiente | `alocacao.rs:639`, `:1117-1119`, `:1221-1223`, `:1251-1290`; `nfe.rs:496`, `:586-626` | **[VERIFICATO]** |
| V10 | i `detalhe` degli avvisi nominano item, valori e campi XML; **nessuno** interpola il percorso del file | `nfe.rs:432`, `:459`, `:502-504`, `:530-532`, `:568`, `:595-597`, `:611-613`, `:636-638`, `:655-657`; `alocacao.rs:1020-1043` | **[VERIFICATO]** |
| V11 | `sha2 0.11.1` e `unicode-normalization 0.1.25` sono già dipendenze dirette; `chrono 0.4.45` è compilato con `std`, `clock`, `serde` | `Cargo.toml` | **[VERIFICATO]** |
| V12 | `dinheiro` è `pub(crate)` e contiene l'unica aritmetica monetaria del crate | `lib.rs:16`, `dinheiro.rs:1-14` | **[VERIFICATO]** |
| V13 | la chiave del campione canonico è `29260512345678000195550010001256612345678901`, con `vNF` 35.00 e tre articoli da 10.00, 20.00 e 5.00 | `samples/nfe_exemplo/nfe_oesa_12566_20260505.xml` | **[VERIFICATO]** |
| V14 | non esiste alcun modulo, tipo o funzione di revisione tracciata: `StatusRevisao` è l'unico residuo del concetto | `lib.rs` elenca `alocacao`, `classificacao`, `dinheiro`, `modelos`, `nfe`, `normalizacao`, `prelivro`, `regras` | **[VERIFICATO]** |

### 1.3 Sonde eseguite per questo documento

- **Strumento**: PowerShell 7.6.7 su .NET 10.0.13, `System.Security.Cryptography.SHA256`, byte UTF-8 **senza BOM**.
- **Script ed esito**: `sonda.ps1` con `esito.txt` (X1–X10), `sonda_r3c3.ps1` con `esito_r3c3.txt` (X12–X14) e `sonda_totalita.ps1` con `sonda_totalita.log` (X15), tutti in `C:\Users\RECANT~1\AppData\Local\Temp\claude\c--Users-Recanto-do-Sossego-Documents-Luca-GitHub-RUST-Rust-registrazioni-contabili\1898eaaf-4410-4449-8413-33f6928a9c29\scratchpad\sonda_impressao\`. **Fuori dal repository**, nessuna installazione, nessuna dipendenza aggiunta.
- **Confronto fra stringhe**: definito sui **byte UTF-8**, non sulle unità UTF-16. È la stessa relazione d'ordine di `Ord` per `&str` in Rust, ed è quella che conta perché l'impronta è calcolata su quei byte. Ogni sonda include una **controprova del comparatore** prima di usarlo: `f` = `0x66` precede `é` precomposto (`0xC3…`) e segue `é` combinante (`0x65…`), e le due forme diventano uguali dopo NFC. In `sonda_totalita.ps1` la controprova mostra anche che l'operatore `-eq` di PowerShell dichiara **uguali** `José` in NFC e in NFD: usarlo per verificare l'identità esatta avrebbe falsificato il caso C2.
- **Tre errori miei nelle sonde, dichiarati**: (1) la prima versione di `sonda_r3c3.ps1` usava un comparatore inerte e **non** riproduceva N1; (2) la prima esecuzione di `sonda_totalita.ps1` stampava un verdetto `False` per una precedenza sbagliata, mentre i due hash della traccia nella stessa riga erano identici; (3) la stessa sonda trattava `AIM` ed `EVI` come stringhe semplici e applicava NFC alla nota dentro il testo chiamato "esatto", quindi **non** poteva scoprire C4 e C5. Tutte corrette e rieseguite; `sonda_totalita.ps1` e il suo log sono alla versione 2 e sostituiscono la precedente. Nessun risultato delle versioni difettose è usato come prova: un'assenza di difetto misurata con uno strumento non validato non è una prova.
- **Dati**: sintetici, scritti a mano. Nessun workbook, nessun file di produzione, nessuna NF-e reale oltre al campione già versionato.
- **Limite dichiarato**: lo script **non** implementa l'impronta. Compone a mano i byte che questa specifica dichiara e li misura. Non dimostra che il futuro codice Rust produrrà quegli stessi byte: lo dovranno dimostrare i test pianificati T3-12 e T3-13.

| # | Sonda | Esito osservato |
|---|---|---|
| X1 | SHA-256 di `abc` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`, **uguale** al vettore FIPS fissato in `alocacao.rs:2016-2022` |
| X2 | i tre payload C23 del campione canonico | `56a719b4…a53`, `d0438efd…edf`, `8dcb5b1b…1c8`: **identici** ai tre valori attesi dal test `hash_do_campione_canonico_corresponde_ao_payload` (`alocacao.rs:1999-2013`) |
| X3 | separatore ingenuo contro prefisso di lunghezza | i campi `('a|b','c')` e `('a','b|c')` uniti con `|` danno **la stessa** impronta `a52dd81b…38e`; con prefisso di lunghezza danno `1a69a539…650` e `344fd61c…592`, **diverse** |
| X4 | forma canonica dei monetari | `10`, `10.00`, `10.0000` → `DEC:10:0`, una sola impronta `aee69fbd…87e`; `0`, `0.00`, `-0.00` → `DEC:0:0`; `0.005` → `DEC:5:3`; `1.2345` → `DEC:12345:4`; `-7.50` → `DEC:-75:1`; `79228162514264337593543950335` → `DEC:79228162514264337593543950335:0` |
| X5 | assenza contro zero reale | `DADO:NAO_DISPONIVEL:18:nfe:cobr/dup/dVenc:-` → `705ef221…4fc`; `DADO:OK:18:nfe:cobr/dup/dVenc:DEC:0:0` → `08046d52…292`: **diverse** |
| X6 | documento minimo completo, 948 byte | impronta `f8a6e8d0aade811e39e31f966c17e0c6623f21afdbde73a8c5866641031f614c`; testo integrale in §4.8 |
| X7 | correzione della descrizione a parità di chiave, conto e totale | impronta `4ce5f31b07e4478dbb70e73cf03639a384be8e27aa802c7236de258c30f997a7`, **diversa** da X6 |
| X8 | lo stesso documento con valore `10`, `10.00`, `10.0000` | impronta **identica** a X6: la forma canonica assorbe gli zeri finali |
| X9 | impronta di presentazione con nome di piano `Refrigerantes` e `REFRIGERANTES` | `44195583…324` e `d1a8e983…424`, **diverse**, mentre l'impronta di contenuto resta `f8a6e8d0…14c` |
| X10 | la stessa descrizione visibile in NFC e in NFD | 15 byte → `8f307c39…5b`; 16 byte → `93ec4905…ec`: **diverse**. La forma Unicode va decisa, non lasciata al caso |
| X11 | controprova con un **secondo** strumento: `sha256sum` di coreutils su sei vettori, fra cui il blocco del documento minimo **estratto da questo documento** | 948 byte e `f8a6e8d0…14c`, più J, K, A, F, G: **tutti identici** alla sonda .NET. Dettaglio in §4.8 |
| X12 | **N1**, riscontro R3C3-02: due avvisi canonicamente equivalenti, ordinati prima sui byte memorizzati e poi sulle chiavi normalizzate | regola ingenua: `6151e963…f3a` con `é` precomposto e `4bd43b99…dcf` con `é` combinante, **diverse**. Chiavi normalizzate prima del confronto: `6151e963…f3a` in **entrambi** i casi. Pareggio fra due avvisi equivalenti: `94e7ec13…032` in entrambi gli ordini. Dettaglio in §4.6.3 |
| X13 | **E1**, riscontro R3C3-01: due `Anotar` con note diverse, stesso istante, nessuna sequenza | regola della revisione 1 (spareggio per posizione): traccia `42742150…` contro `2d7404e8…`, **diverse**. Chiave canonica: `42742150…` in **entrambe** le permutazioni. Dettaglio in §5.4 |
| X14 | lunghezze in byte dei vettori A–H, ricontrollate una per una | `8, 7, 7, 11, 9, 35, 43, 37`: il vettore F è di **35** byte, non 33 come diceva la revisione 1 |
| X15 | **C1–C5**, residuo di R3C3-01: la chiave della revisione 2, la codifica esatta della revisione 3 e quella corretta, confrontate caso per caso | la chiave della revisione 2 è **uguale** in tutti e cinque i casi; la codifica esatta della revisione 3 collide in **C2, C4 e C5**, perché riusava le produzioni NFC e il `dado` senza `motivo`. Con `str_exato` e `dado_esatto_string` i cinque casi danno testi **diversi**, nessuno è una `Repeticao`, e la traccia è identica nelle due permutazioni: `54bc536b…130d` (C1), `6a0e4376…cd2c` (C2), `e4f70810…f837` (C3), `d29328eb…b69f` (C4), `4f619ebb…0195` (C5). E1 resta chiuso: `348ca271…6e2f` in entrambi gli ordini. Dettaglio in §5.4.2 |

X1 e X2 sono **controprove dello strumento**: la sonda riproduce byte per byte quattro impronte che il codice Rust già calcola e che i test già fissano. Tutto ciò che segue usa lo stesso strumento validato così.

### 1.4 Evidenze riusate, non rieseguite

| Evidenza | Fonte | Perché non viene rieseguita |
|---|---|---|
| 208 test verdi (192 lib + 3 bin + 13 CLI), `fmt`, `clippy -D warnings`, `diff --check` puliti, sei fonti canoniche invariate | verbale finale 3C.2, `AUDIT_3C2_R2_ESITO_20261017.md` | nessun file di codice è stato toccato: i quattro SHA di §1.1 lo dimostrano. Un documento Markdown non invalida una suite |
| difetti provati del writer Python S1, S2, S3, S5, S6, S7, S8, S9 | §1.3 della specifica 3C | sonde già eseguite e auditate; qui servono solo come confronto |
| 47 test canonici Python al commit `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` | §1.2 della specifica 3C | eseguiti e confermati. **Non** attestano un workflow di approvazione: il Python non ne implementa uno |
| C18 (ordine delle registrazioni), C23 (hash), C24 (quattro esiti), D10(a), D13(a), riscontri R3C2-01/02/03 chiusi | fasi 3B e 3C.2 | contratti già chiusi e protetti da test; non vengono riaperti |

### 1.5 Prove non eseguite

| Non eseguito | Motivo |
|---|---|
| suite Rust | nessun codice toccato; baseline riusata come sopra |
| implementazione dell'impronta in Rust | fuori perimetro: questo mandato autorizza la specifica |
| round-trip su archivio o workbook | l'archivio non esiste: è 3C.4 |
| confronto con un workflow di approvazione Python | **non esiste** nel repository canonico. L'importatore della revisione classificazioni produce un **CSV di regole riviste**, non un'approvazione di registrazioni (**[RIUSATO]**, §4.2 della specifica 3C) |
| autenticazione, ruoli, permessi | fuori perimetro e **non** proposti: §3.5 |
| persistenza degli eventi | è 3C.4 e dipende da D7: §5.6 |

---

## 2. Dove si inserisce la revisione

### 2.1 Il percorso

```text
XML  ──fase 2──>  NfeImportada  ──fase 3B──>  AlocacaoNfe (un documento)
                                                    |
                     ContextoDocumento (identità) ---+
                                                    v
                              3C.2:  construir_pre_livro(documentos, plano)
                                                    |
                                     PreLivro: righe, documenti, riepiloghi,
                                     indicatori, esclusioni, avvisi
                                                    |
        eventi di revisione dichiarati -------------+   <== 3C.3, questo blocco
        (bersaglio, autore, istante, motivo)        v
                              VistaRevisada: decisioni applicate + traccia
                                                    |
                                                    v
                      3C.4: archivio autorevole, esportazione, import storico
```

La 3C.3 **non** sostituisce la 3C.2 e **non** la modifica: la usa. **[PROPOSTA]**

### 2.2 Tre nozioni che non vanno confuse

È il cuore del blocco, e il rischio principale dichiarato dal mandato.

| Nozione | Chi la produce | Dove vive oggi | Chi può cambiarla |
|---|---|---|---|
| **esito tecnico** della riga | l'allocatore 3B, per calcolo | `Lancamento::status_revisao` ∈ {`OK`, `REVISAR`} (**[VERIFICATO]** V1) | solo una nuova allocazione |
| **decisione umana** | una persona, con un evento dichiarato | **non esiste**: niente nel repository la rappresenta (**[VERIFICATO]** V14) | la persona, con un nuovo evento |
| **stato della vista di revisione** | questo blocco, combinando le due | **non esiste** | nessuno direttamente: è derivato, e ricalcolabile dagli eventi |

Conseguenze immediate, tutte **[PROPOSTA]**:

1. La presenza di `StatusRevisao::Aprovado` **non** dimostra che un'approvazione sia stata eseguita, né da chi: oggi è un valore rappresentabile e mai prodotto (V1).
2. Una revoca **non** cancella l'approvazione precedente: produce un secondo evento, e la traccia conserva entrambi.
3. Se si perde la vista non si perde la decisione, purché gli eventi siano conservati. Dove vivono gli eventi finché non c'è archivio è detto in §5.6, e una dimostrazione in memoria **non** è uno storico salvato.

### 2.3 Inventario dei tipi riusabili

Niente di quanto segue va duplicato o riscritto. **[VERIFICATO]** per esistenza e forma.

| Tipo | Dove | Uso in 3C.3 |
|---|---|---|
| `ContextoDocumento` | `prelivro.rs:266-290` | ingresso: identità del documento, `AlocacaoNfe`, avvisi di lettura, lotto |
| `IdentidadeLote` | `prelivro.rs:255-262` | provenienza dell'esecuzione; **non** entra nell'impronta di contenuto (§4.5) |
| `IdentidadeLinha` | `prelivro.rs:320-331` | identità **nel lotto**; resta com'è, nessun cambio di semantica |
| `LinhaPreLivro`, `DocumentoPreLivro`, `Resumo`, `Exclusao`, `Indicadores`, `PreLivro` | `prelivro.rs:333-470` | la vista revisionata li riusa senza modificarli |
| `Particao` | `prelivro.rs:299-317` | partizione della riga; `Aprovado` esiste già |
| `Reconciliacao`, quattro esiti, `StatoDocumento`, `Componente`, `CampoRinviato` | `alocacao.rs:180-308` | entrano nell'impronta di contenuto; **mai** riscritti da un'approvazione |
| `Lancamento` | `modelos.rs:540-576` | la fonte; la proiezione ne scrive **un solo** campo (§5.2) |
| `Dado<T>`, `StatusDado` | `modelos.rs:26-205` | carta d'identità: usata per la sequenza di versione non disponibile e per il bersaglio senza provenienza |
| `Competencia` | `modelos.rs:356-400` | validata, `Display` `YYYY-MM`: forma già canonica |
| `dinheiro` | `dinheiro.rs` | **unica** aritmetica monetaria; 3C.3 non ne aggiunge nessuna |
| `Aviso` | `nfe.rs:148-160` | avvisi della vista, con codici testuali stabili |
| `PlanoContas` | `regras.rs` | fonte autorevole dei nomi; parametro, non lettura d'ambiente |
| `sha2::Sha256` | già usato in `alocacao.rs:55` | algoritmo dell'impronta: **nessuna** nuova dipendenza |
| `unicode_normalization` | già dipendenza (V11), usata in `normalizacao.rs` | forma Unicode del testo canonico (§4.6) |
| `chrono::DateTime<FixedOffset>` | dipendenza già presente (V11) | istante dichiarato dell'evento, con fuso esplicito (§3.3) |

---
## 3. D9 — significato e contratto della revisione

> **D9.1–D9.5 APPROVATE da Luca il 2026-10-21, tutte nell'opzione (a)** (§12).
> Approvata la decisione funzionale: il modulo che la applica non esiste e i
> test di questo capitolo restano pianificati. I vincoli **già vigenti** di
> §3.9 non erano in discussione e restano quello che erano.

### 3.1 Che cosa significa approvare

**D9.1(a), [APPROVATO]: "visto e accettato".** Approvare significa che una persona
dichiara di aver esaminato una **versione precisa** del contenuto allocato e di
accettarla come pronta per l'uscita contabile. Non significa:

| Approvare **non** significa | Perché |
|---|---|
| che i numeri siano corretti | la correttezza aritmetica è l'esito tecnico dei quattro esiti (C24), e resta quello che è |
| che gli avvisi siano risolti | gli avvisi non vengono cancellati: §3.9 punto 3 |
| che il documento sia contabilizzato | l'estratto approvato è una **vista in memoria** (`prelivro.rs:452-460`, **[VERIFICATO]**), non una contabilizzazione |
| che la persona sia chi dice di essere | §3.5 |
| che la stessa decisione valga per una versione successiva | §4.2: l'approvazione resta legata all'impronta esaminata |

### 3.2 Matrice delle operazioni

Stati della **vista** (nomi **[PROPOSTA]**): `TECNICO_OK`, `TECNICO_REVISAR`,
`APROVADO_POR_PESSOA`, `APROVACAO_REVOGADA`, `APROVADO_SEM_PROVENIENCIA`.
I primi due riflettono l'esito tecnico 3B senza decisioni umane (V1).

| # | Stato iniziale | Richiesta | Condizioni | Stato risultante | Evento prodotto | Errore / non applicazione | Campi invariati |
|---|---|---|---|---|---|---|---|
| O1 | `TECNICO_OK` | `Aprovar` | bersaglio presente nella versione corrente; autore, istante e motivo dichiarati | `APROVADO_POR_PESSOA` | approvazione in traccia | — | tutti tranne `status_revisao` della copia proiettata |
| O2 | `TECNICO_REVISAR` | `Aprovar` | idem | `APROVADO_POR_PESSOA` | approvazione in traccia | — | `alertas`, i quattro esiti, `stato_documento`, `motivos` restano identici |
| O3 | `APROVADO_POR_PESSOA` | `Aprovar` con evento **byte-identico** | idem | invariato | **nessun** nuovo evento: ripetizione dichiarata | — | tutti |
| O4 | `APROVADO_POR_PESSOA` | `Aprovar` con autore, istante o motivo diversi | idem | invariato | secondo evento conservato, avviso `APROVACAO_REPETIDA` | — | tutti |
| O5 | `APROVADO_POR_PESSOA` | `Revogar` | esiste un'approvazione applicata sullo stesso bersaglio e sulla stessa impronta | **D9.3**: `APROVACAO_REVOGADA`, con stato tecnico ripristinato | revoca in traccia; l'approvazione precedente **resta** | — | tutti tranne `status_revisao` |
| O6 | `TECNICO_OK` / `TECNICO_REVISAR` | `Revogar` | niente da revocare | invariato | nessuno | **non applicato**: `NadaARevogar`, elencato | tutti |
| O7 | qualunque | `Anotar` | nota non vuota, oltre ad autore, istante e motivo | **invariato** | annotazione in traccia | — | **tutti**, `observacoes` compreso (§5.2) |
| O8 | qualunque | qualunque | l'impronta del bersaglio **non** è quella della versione corrente | invariato | nessuno | **non applicato**: `VersaoSuperada`, elencato e consultabile | tutti |
| O9 | qualunque | qualunque | il bersaglio non esiste in questo lotto | invariato | nessuno | **non applicato**: `AlvoAusente`, elencato | tutti |
| O10 | qualunque | qualunque | bersaglio senza impronta dichiarata | invariato | nessuno | **non applicato**: `ProvenienciaAusente`, elencato (**D9.4**) | tutti |
| O11 | `APROVADO_SEM_PROVENIENCIA` | `Aprovar` | bersaglio e impronta corretti | `APROVADO_POR_PESSOA` | approvazione in traccia: è la **sanatoria esplicita** di un `APROVADO` ereditato | — | tutti tranne `status_revisao` |
| O12 | riga di documento `ERRO` | `Aprovar` | **D9.2** | `APROVADO_POR_PESSOA`, ma **fuori** dall'estratto, con esclusione motivata | approvazione in traccia + avviso `APROVACAO_EM_DOCUMENTO_ERRO` | — | tutti tranne `status_revisao`; D10(a) resta intatta |
| O13 | qualunque | portare a `OK` | — | — | — | **vietato**: `OK` è un esito tecnico, non una decisione (vincolo già vigente) | — |
| O14 | qualunque | cambiare conto, importo, pagamento o regola | — | — | — | **vietato in 3C.3**: §3.8 | — |

Precisazioni, tutte **[PROPOSTA]**:

- O3 contro O4: la differenza è l'uguaglianza **byte per byte** dell'evento. Due
  approvazioni diverse della stessa riga non sono un errore (una persona può
  rivedere e riconfermare), ma non si sovrascrivono: si accumulano.
- O5 ripristina lo stato tecnico, non `REVISAR` d'ufficio. La §4.1 della
  specifica 3C generale scriveva `APROVADO → REVISAR`: questa è una
  **raffinazione dichiarata**, non una correzione silenziosa, ed è la
  sottoscelta **D9.3**.
- O8 è il punto più delicato: un evento che cita una versione superata **non**
  viene applicato a nessun'altra riga. Non viene nemmeno cancellato: resta
  consultabile, con la sua impronta, perché documenta che qualcuno approvò
  *quel* contenuto.

### 3.3 Dati minimi dell'evento, e la loro provenienza

```text
EventoRevisao {
  alvo        : ChaveRevisao     // che cosa è stato esaminato: §4.1
  operacao    : OperacaoRevisao  // Aprovar | Revogar | Anotar
  autor        : String          // dichiarato da chi opera; non vuoto
  instante     : DateTime<FixedOffset>  // dichiarato, con offset esplicito
  motivo       : String          // non vuoto
  nota         : Option<String>  // obbligatoria per Anotar, ammessa altrove
  sequencia    : Option<u64>     // ordine dichiarato a parità di istante
  vista        : Dado<String>    // impronta di presentazione vista, §4.7
}
```

| Dato | Provenienza ammessa | Provenienza **vietata** |
|---|---|---|
| `alvo` | costruito da chi ha in mano la riga esaminata | dedotto dall'indice di una tabella o del foglio Excel |
| `autor` | stringa dichiarata da chi opera | utente del sistema operativo, nome macchina, variabile d'ambiente |
| `instante` | dichiarato dal chiamante, con fuso esplicito | `Utc::now()` **dentro** la funzione pura, o un orario senza fuso |
| `motivo` | dichiarato | testo di default, messaggio generato, stringa vuota |
| `nota` | dichiarata | derivata da `alertas` o da un avviso |
| `sequencia` | dichiarata | posizione nell'array di ingresso |
| `vista` | impronta di presentazione calcolata al momento dell'esame | ricalcolata a posteriori con il piano corrente |

**Convenzione temporale, [PROPOSTA]:** `instante` è un `DateTime<FixedOffset>`,
cioè porta **sempre** il proprio scostamento da UTC. Motivo: il ristorante opera
in fuso brasiliano, l'archivio futuro potrà vivere altrove, e un
`NaiveDateTime` renderebbe due eventi confrontabili solo per convenzione
implicita. Il confronto fra eventi avviene sull'istante **assoluto**; la
rappresentazione dichiarata resta quella scritta dall'operatore.

### 3.4 Obbligatorietà, errori, nessun default

Due famiglie separate, **[PROPOSTA]**:

| Famiglia | Casi | Esito |
|---|---|---|
| **evento malformato** | `autor` vuoto o solo spazi; `motivo` vuoto; `nota` assente su `Anotar`; `alvo` con `chave_origem` vuota; `conta_id` vuoto | **errore**, nessuna vista prodotta: `ErroRevisao` nomina campo, bersaglio e operazione |
| **evento non applicabile** | bersaglio assente (O9), versione superata (O8), provenienza assente (O10), nulla da revocare (O6) | la vista **viene prodotta**, l'evento è elencato in `eventos_nao_aplicados` con il motivo |

La ragione della separazione: un evento malformato è un difetto del chiamante e
va corretto; un evento non applicabile è un fatto normale della vita di un
archivio (si rigenera il pre-libro e le decisioni vecchie non combaciano più).
Fare fallire tutto il lotto per il secondo caso renderebbe lo strumento
inutilizzabile. È la sottoscelta **D9.5**.

Nessun valore di default somiglia a un'approvazione valida: non esiste un
`autor` predefinito, non esiste un `motivo` predefinito, e l'assenza di eventi
produce una vista in cui **nessuna** riga è approvata.

### 3.5 L'autore: una stringa attribuisce, non autentica

`autor` è una **dichiarazione**. **[PROPOSTA]** e non negoziabile nella
proposta: nessun ruolo, nessun permesso, nessuna autenticazione, nessuna firma.
La specifica dichiara esplicitamente il limite, perché il contrario sarebbe un
falso contratto:

- il sistema **non** sa chi ha scritto quella stringa;
- due eventi con lo stesso `autor` **non** provano che sia la stessa persona;
- una cella modificata a mano in un workbook, da sola, **non** è
  un'autorizzazione: è un dato in ingresso da riconoscere, attribuire e
  motivare (vincolo già vigente, §4.3 della specifica 3C).

Conseguenza pratica: l'etichetta corretta per il contenuto di `autor` è
`NON_VERIFICATO` come identità, `OK` come dichiarazione. L'autenticazione, se
mai servirà, è un blocco a sé e non è in questo perimetro.

### 3.6 Un `APROVADO` ricevuto senza evento di provenienza

Il caso esiste perché `StatusRevisao::Aprovado` è rappresentabile mentre la 3B
non lo produce mai (**[VERIFICATO]** V1): quindi una riga `APROVADO` in ingresso
può venire solo da un import (3C.4) o da un chiamante che l'ha costruita a mano.

**D9.4, raccomandazione:** la riga **non** viene equiparata a una revisione
verificata e **non** perde il dato originale.

| Aspetto | Comportamento proposto |
|---|---|
| dato originale | conservato: `status_tecnico = APROVADO` resta leggibile nella vista |
| stato della vista | `APROVADO_SEM_PROVENIENCIA` |
| avviso | `APROVACAO_SEM_PROVENIENCIA`, con l'identità della riga |
| estratto revisionato | **esclusa**, con motivo elencato (§5.2) |
| sanatoria | possibile e tracciata: un evento `Aprovar` esplicito la porta a `APROVADO_POR_PESSOA` (O11) |

Nota di coerenza, **[VERIFICATO]**: `PreLivro::extrato_aprovado()` della 3C.2
include qualunque riga `APROVADO` di documento non `ERRO` (`prelivro.rs:610-612`).
Il filtro più severo vive **nella vista di 3C.3**, non in 3C.2: il
comportamento della 3C.2 resta invariato, come chiede il mandato. La relazione
fra i due insiemi è un'invariante verificabile: `extrato_revisado` è sempre un
sottoinsieme di `extrato_aprovado` della proiezione, e ogni differenza è
elencata con il motivo (§5.2, test T3-16).

### 3.7 Approvazione su un documento `ERRO`

D10(a) è **già approvata** e dice che un documento `ERRO` non entra
nell'estratto approvato, anche se una sua riga porta `APROVADO`
(**[VERIFICATO]** V8). Il mandato chiede una cosa diversa: se sia **ammesso
registrare** una presa visione su quel documento.

**D9.2, raccomandazione (a): ammesso registrarla, mai promuoverla.**

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** l'`Aprovar` è valido, la riga risulta `APROVADO_POR_PESSOA`, resta **fuori** dall'estratto con esclusione motivata, più avviso `APROVACAO_EM_DOCUMENTO_ERRO` | la decisione è registrata nello stesso vocabolario di tutte le altre approvazioni, ha uno **stato** interrogabile, e la sua esclusione compare nell'elenco delle esclusioni con il motivo | **raccomandata** |
| (b) il comando rifiuta l'`Aprovar` con errore; la presa visione resta possibile come `Anotar`, che la matrice O7 ammette in qualunque stato | la traccia **non** si perde: resta un'annotazione con autore, istante e motivo. Si perde la **forma**: "visto e accettato, bloccato da D10(a)" diventa testo libero senza stato, non confrontabile né interrogabile, e la differenza fra "approvato ma escluso" e "solo annotato" scompare | ammissibile, meno informativa |
| (c) l'`Aprovar` è valido e la riga entra nell'estratto | **non ammissibile**: contraddice D10(a), già approvata | esclusa |

Rettifica dopo l'audit del 2026-10-21: la revisione 1 motivava (a) dicendo che
(b) impedirebbe all'operatore di annotare "visto, so che è rotto". Era
**falso**, e lo smentiva la stessa matrice O7, che ammette `Anotar` in qualunque
stato. La raccomandazione resta (a), ma per il motivo giusto: la differenza fra
le due opzioni è la **forma** della traccia — stato tipizzato contro testo
libero — non la sua esistenza.

Da non confondere: **nessuna** delle due opzioni cambia il filtro. D10(a) è
decisa e resta: un documento `ERRO` non entra nell'estratto in nessun caso.

### 3.8 Cinque operazioni che restano separate

Vincolo già vigente (§4.2 della specifica 3C), qui riconfermato: approvare non è
correggere. In 3C.3 esistono **solo** `Aprovar`, `Revogar`, `Anotar`.

| Operazione | In 3C.3 | Perché non qui |
|---|---|---|
| correggere un conto | **no** | è una riclassificazione: cambia i riepiloghi e l'impronta, quindi è una **nuova versione**, non una decisione |
| modificare un importo | **no** | tocca denaro, invarianti e riconciliazione; va sulla fonte, non sulla riga |
| riconoscere un pagamento | **no** | richiede una fonte di pagamenti che non esiste: `data_pagamento` è `NAO_DISPONIVEL` per contratto (F3A-17, **[VERIFICATO]** `alocacao.rs:1221-1223`) |
| cambiare una regola YAML | **no** | cambia la classificazione di molti documenti; mandato separato |
| creare o modificare una riga manuale | **no** | la 3C.2 non genera righe manuali, e conservare informazioni umane non autorizza a introdurne la creazione ora |

Nessuna di queste entra "di lato" attraverso `observacoes` o attraverso una
generica richiesta di revisione: `Anotar` scrive **solo** nella traccia, e
`observacoes` resta vuoto (§5.2).

### 3.9 Vincoli già vigenti che la proposta preserva

Non sono proposte: sono contratti chiusi che questo blocco non può violare.

| # | Vincolo | Fonte | Come la proposta lo rispetta |
|---|---|---|---|
| 1 | nessuna approvazione automatica; nessun autore, motivo o istante inventato | F3C-04, metodo | l'approvazione esiste solo come evento dichiarato; nessun orologio e nessun utente di sistema dentro la funzione pura |
| 2 | esito tecnico, decisione umana e stato della vista restano distinti; una revoca non cancella l'approvazione dalla traccia | §2.2 | tre campi separati per riga, più la traccia completa |
| 3 | approvare non modifica valori, conto, hash C23, rate, avvisi, copertura, componenti, campi rinviati, né i quattro esiti | C23, C24, F3C-05 | la proiezione scrive **un solo** campo e un test lo verifica campo per campo (T3-03) |
| 4 | D10(a): un documento `ERRO` resta fuori dall'estratto approvato | D10(a) approvata | invariata; §3.7 riguarda solo la registrazione dell'evento |
| 5 | correzione contabile, cambio conto, importo, pagamento e YAML sono operazioni separate | §4.2 della specifica 3C | §3.8 |
| 6 | D13(a): chiave di riepilogo e nome del conto dal piano | D13(a) approvata | la vista riusa i riepiloghi della 3C.2, non ne costruisce altri |
| 7 | C18: ordine delle registrazioni del documento, e quindi `ordinal` | C18 | `ordinal` resta quello dell'allocazione, mai l'indice del pre-libro ordinato né della futura riga Excel |

---
## 4. D8 — identità, versione, impronta

> **D8.1–D8.5 APPROVATE da Luca il 2026-10-21, tutte nell'opzione (a)** (§12).
> La D8(a) della specifica 3C generale non era approvata e il suo testo
> dichiarava l'impronta "rinviata, non risolta": qui è stata verificata,
> corretta dove non reggeva, resa implementabile e infine **decisa**. Approvata
> la decisione funzionale: `impressao.rs` non esiste ancora.

### 4.1 Cinque identità distinte

Confonderle è la causa del rischio "approvazione attribuita alla versione
sbagliata". **[PROPOSTA]** per i nomi, **[VERIFICATO]** per ciò che esiste già.

| Identità | Che cosa individua | Oggi | Proposta 3C.3 |
|---|---|---|---|
| **esecuzione (lotto)** | una generazione del pre-libro | `IdentidadeLote { id, gerado_em, descricao }` (**[VERIFICATO]** `prelivro.rs:255-262`) | invariata; **non** entra nell'impronta |
| **documento** | la NF-e | `chave_origem` (C22: vuota = rifiutata) | invariata |
| **versione del documento** | *quale contenuto* di quella NF-e | **non esiste** | `impressao_conteudo`: SHA-256 del testo canonico (§4.4) |
| **riga nella versione** | quale registrazione | `(origem, conta_id, ordinal)` dentro il documento (**[VERIFICATO]** V5) | invariata |
| **evento** | quale decisione umana | **non esiste** | `(alvo, operacao, autor, instante, motivo, nota, sequencia)` (§3.3) |

Chiave di **bersaglio** di una decisione, **[PROPOSTA]**:

```text
ChaveRevisao {
  chave_origem        : String        // quale documento
  impressao_conteudo  : Dado<String>  // quale versione del contenuto esaminato
  origem              : OrigemLancamento
  conta_id            : String
  ordinal             : u32
}
```

Notare che **non contiene `lote`**. È la differenza decisiva rispetto a
`IdentidadeLinha`, ed è il punto chiesto dal mandato.

### 4.2 La tensione fra `lote` e rigenerazione, risolta con esempi

Il fatto, **[VERIFICATO]** V5: `IdentidadeLinha` include `lote`. Il fatto
secondo: una rigenerazione può avere un lotto diverso e lo stesso contenuto
esaminato. Un algoritmo che cercasse l'identità **completa** (lotto compreso)
non ritroverebbe mai una decisione dopo una rigenerazione, e promettere il
contrario sarebbe falso.

Soluzione **[PROPOSTA]**: due chiavi con due scopi dichiarati.

| Chiave | Contiene `lote` | Serve a | Unicità |
|---|---|---|---|
| `IdentidadeLinha` | **sì** | dire **da quale esecuzione** arriva la riga che ho davanti; provenienza e diagnostica | unica nel lotto (**[VERIFICATO]**: `chave_ordenacao` la usa come ordine totale, `prelivro.rs:874-884`) |
| `ChaveRevisao` | **no** | legare una decisione al **contenuto esaminato** | unica dentro una versione di un documento |

Tre esempi di rigenerazione, **[PROPOSTA]** per l'esito, **[VERIFICATO]** per i
dati di partenza. Sono numerati `R1`–`R3` per non confonderli con i
controesempi `E1` e `N1` dell'audit, citati in §5.4 e §4.6.3:

| Esempio | `lote` | `impressao_conteudo` | `(origem, conta_id, ordinal)` | Esito |
|---|---|---|---|---|
| R1 — stessa NF-e rielaborata domani, nulla cambiato | `lote-A` → `lote-B` | **identica** | identici | la decisione **si ritrova**: la corrispondenza è esatta e unica |
| R2 — descrizione di un articolo corretta, totale identico | `lote-A` → `lote-B` | **diversa** (sonda X7) | identici | la decisione **non** si trasferisce: resta registrata su `impressao` vecchia, la riga nuova nasce da revisionare |
| R3 — due righe legittime con lo stesso payload C23 (V4) | qualunque | identica | `ordinal` **diversi** | due bersagli distinti, due decisioni indipendenti, nessuno scarto |

**Regola di corrispondenza, [PROPOSTA]:** un evento si applica se e solo se
esiste **esattamente una** riga del lotto corrente con la stessa
`(chave_origem, impressao_conteudo, origem, conta_id, ordinal)`.

- zero corrispondenze con impronta diversa → `VersaoSuperada` (O8);
- zero corrispondenze con impronta uguale → `AlvoAusente` (O9);
- più di una corrispondenza → **conflitto dichiarato** `AlvoAmbiguo`, errore,
  nessuna applicazione. **[NON_VERIFICATO]** che il caso sia raggiungibile:
  `ordinal` è l'indice nel vettore delle registrazioni e `chave_origem` è unica
  nel lotto (`DocumentoDuplicadoNoLote`, **[VERIFICATO]** `prelivro.rs:570-575`),
  quindi la duplicazione sembra impossibile. Il controllo resta perché una
  impossibilità non dimostrata non è una garanzia, e il test T3-14 ne verifica
  il messaggio costruendo il caso a mano.

La semantica dell'identità attuale **non** cambia per silenzio: `IdentidadeLinha`
resta esattamente com'è, con lo stesso significato documentato in
`prelivro.rs:313-319`.

### 4.3 Che cosa resta di C23

Invariato, **[VERIFICATO]** V3 e V4:

- `hash_registro` è SHA-256 di `ORIGEM|chave|conta_id|valor` a due decimali;
- **non** è una chiave univoca e non distingue tutte le correzioni (sonda X7
  mostra un contenuto diverso a parità dei quattro campi);
- due righe legittime con payload uguale restano **due** righe revisionabili
  separatamente (R3);
- serve ancora al confronto con lo storico Python, e non viene toccato.

L'impronta di contenuto **non riusa** il payload C23: sono due cose con due
scopi, e un'impronta che ereditasse quei quattro campi erediterebbe anche la
cecità alle correzioni.

### 4.4 Testo canonico dell'impronta — `IMPRESSAO-CONTEUDO/v1`

**[PROPOSTA]** completa e implementabile. Tutto è dichiarato: versione, campi,
ordine, codifica, separazione.

**Principi**

1. **Prefisso di lunghezza su ogni stringa.** Forma `TAG=<byte>:<contenuto>`,
   dove `<byte>` è la lunghezza in **byte UTF-8**. Nessun escaping, nessun
   separatore vietato nei dati. Motivo provato dalla sonda X3: due insiemi di
   campi diversi uniti con `|` producono la **stessa** impronta; col prefisso di
   lunghezza producono impronte diverse.
2. **Token chiusi senza prefisso** per enum, date e numeri interi: provengono da
   alfabeti finiti e non possono contenere il terminatore.
3. **Una riga per campo**, terminata da `LF` (`0x0A`), e `LF` finale. Il testo è
   un flusso di byte, non un file di testo di piattaforma: nessun `CRLF`.
4. **Conteggio esplicito prima di ogni collezione**: `TAG#<n>`. Una collezione
   vuota scrive `TAG#0` e nulla dopo, così "nessun avviso" non è
   indistinguibile da "campo assente".
5. **Algoritmo**: SHA-256 (`sha2`, già dipendenza, **[VERIFICATO]** V11) sui byte
   del testo, risultato in **esadecimale minuscolo** di 64 caratteri, come già fa
   `sha256_hex` (`alocacao.rs:817-824`).

**Grammatica**

```text
documento   := "V=IMPRESSAO-CONTEUDO/v1" LF
               "DCH=" str LF            // chave_origem
               "DNU=" str LF            // numero
               "DSE=" str LF            // serie
               "DDT=" data LF           // data_documento
               "DCP=" comp LF           // competencia  (YYYY-MM)
               "DEM=" str LF            // cnpj_empresa, come memorizzato
               "DNE=" str LF            // nome_empresa
               "AVL#" n LF  aviso*      // avisos_leitura, ordinati
               "LAN#" n LF  linha*      // registrazioni, in ordine di ordinal
               reconciliacao

linha       := "LOR=" u32 LF            // ordinal
               "LOG=" origem LF         // XML_NFE | CSV | MANUAL
               "LCI=" str LF  "LCN=" str LF  "LGC=" str LF
               "LDS=" str LF            // descricao_itens
               "LVL=" dec LF            // valor
               "LNE=" str LF  "LFV=" str LF  "LCF=" str LF  "LTD=" str LF
               "LTL=" tipo LF           // tipo_lancamento
               "LDRI=" str LF  "LDRN=" str LF  "LCC=" str LF
               "LSR=" status LF         // status_revisao TECNICO
               "LAL=" str LF            // alertas
               "LPG=" dado LF           // data_pagamento
               "LDUP#" n LF  duplicata*

duplicata   := "DN=" str LF  "DV=" (data | "AUSENTE") LF  "DVL=" dec LF

reconciliacao := "REC" LF
               "RVT=" dec LF  "RSL=" dec LF  "RET=" esito LF
               "RRD=" dado LF  "RRA=" dado LF  "RER=" esito LF
               "RAA=" usize LF  "RAT=" dado LF
               "RSC#" n LF  aviso*      // articoli_scartati
               "RCO=" esito LF
               "RCM#" n LF  componente*
               "RCE=" esito LF
               "RRN#" n LF  ("FN=" str LF)*   // campi_rinviati: solo i nomi
               "RAD#" n LF  aviso*      // avisos_documento
               "RSD=" stato LF          // stato_documento

componente  := "MN=" str LF  "MV=" dado LF  "MR=" ruolo LF  "MU=" ("0"|"1") LF
aviso       := "AC=" str LF  "AD=" str LF

str         := byte_len ":" bytes_utf8_nfc
dec         := "DEC:" mantissa ":" scala          // §4.6
dado        := "DADO:" status_dado ":" str_senza_tag ":" (valore | "-")
data        := YYYY "-" MM "-" DD
```

### 4.5 Campi inclusi ed esclusi, con il motivo

`Lancamento` ha 26 campi (**[VERIFICATO]** `modelos.rs:540-576`). La ripartizione
proposta li copre **tutti**, senza residui:

| Gruppo | Campi | Perché |
|---|---|---|
| **inclusi** (18) | `nome_empresa`, `fornecedor_ou_favorecido`, `cnpj_cpf_favorecido`, `tipo_documento`, `conta_id`, `conta_nome`, `grupo_conta`, `descricao_itens`, `valor`, `duplicatas`, `origem`, `status_revisao`, `alertas`, `tipo_lancamento`, `dre_linha_id`, `dre_linha_nome`, `centro_custo`, `data_pagamento` | sono il contenuto esaminato. Una loro variazione cambia ciò che la persona ha visto, quindi deve cambiare la versione |
| **omessi perché già confrontati** (5) | `chave_origem`, `documento`, `data_documento`, `competencia`, `cnpj_empresa` | scritti **una volta** a livello di documento. L'omissione è sicura **perché** `verificar_coerencia` rifiuta qualunque disaccordo fra contesto e riga (**[VERIFICATO]** V6). Senza quel controllo questa scelta sarebbe sbagliata |
| **esclusi** (3) | `arquivo_origem`, `hash_registro`, `observacoes` | §4.5.1 |

#### 4.5.1 I tre esclusi, uno per uno

| Campo | Motivo dell'esclusione | Che cosa si perde, dichiarato |
|---|---|---|
| `arquivo_origem` | è un **dato d'ambiente**: percorso o nome di file. Spostare o rinominare il file non cambia il contenuto contabile, e includerlo invaliderebbe tutte le approvazioni a ogni riorganizzazione delle cartelle | due file distinti con contenuto identico danno la stessa impronta. È corretto: è lo **stesso** contenuto. Il campo resta visibile nel record, fuori dall'impronta |
| `hash_registro` | derivato dai quattro campi C23, già inclusi singolarmente: ridondante, e riusarlo contraddirebbe §4.3 | nulla: è ricalcolabile |
| `observacoes` | è un **campo umano**. Includerlo significherebbe che annotare invalida l'oggetto appena annotato, cioè l'errore che il mandato chiede di evitare | nulla oggi: in 3B è sempre vuoto (**[VERIFICATO]** V2). Se un blocco futuro lo popolasse, l'impronta **non** lo coprirebbe: limite dichiarato, da rivedere in 3C.4 |

A livello di documento sono esclusi anche:

| Elemento | Motivo |
|---|---|
| `IdentidadeLote` (`id`, `gerado_em`, `descricao`) | varia a ogni esecuzione senza che il contenuto cambi: è esattamente la tensione di §4.2 |
| `arquivo_origem` del contesto | come sopra |
| il `PlanoContas` | è presentazione, non contenuto: §4.7 |

#### 4.5.2 Dati d'ambiente nei testi inclusi: verifica, non promessa

Il mandato chiede di non promettere invarianza al percorso se il percorso
rientra **indirettamente** nei byte. Verificato uno per uno:

| Possibile veicolo | Esito | Etichetta |
|---|---|---|
| `fonte` dei `Dado` | forma XPath (`nfe:det`, `nfe:cobr/dup/vDup`, `nfe:total/ICMSTot/*`): nessun percorso | **[VERIFICATO]** V9 |
| `detalhe` degli avvisi di lettura e di documento | nominano item, campi XML e valori; nessuna interpolazione di percorso | **[VERIFICATO]** V10 |
| `motivo` dei `Dado` | prose fisse del codice; **escluso** dall'impronta per scelta (§4.6) | **[VERIFICATO]** per il contenuto |
| `descricao_itens` | testo della NF-e più l'importo formattato (F3A-01): dato del documento | **[VERIFICATO]** |
| `alertas` | codici stabili separati da `"; "` | **[VERIFICATO]** |

Conclusione, **[PROPOSTA]**. L'impronta è **invariante** a:

| Invariante a | Perché |
|---|---|
| percorso e nome del file | `arquivo_origem` escluso (§4.5.1) |
| identità del lotto | `IdentidadeLote` esclusa (§4.5.1) |
| campi umani | `observacoes` escluso; annotazioni e approvazioni vivono negli eventi |
| **forma Unicode** del testo | D8.4(a) **[APPROVATO]**: normalizzazione NFC applicata sia alle chiavi di ordinamento sia all'emissione (§4.6.1 e §4.6.3) |
| ordine di arrivo degli `avisos_leitura` | ordinamento canonico (§4.6.3) |

e **non** è invariante a:

| Non invariante a | Perché |
|---|---|
| ~~forma Unicode~~ | non più pertinente: con D8.4(a) approvata l'impronta **è** invariante alla forma. Resta registrato che con l'opzione (b) non lo sarebbe stata, perché i byte memorizzati sarebbero entrati nell'impronta così come sono (**[SONDA]** X10) |
| cambio di regole che modifica `conta_id`, `conta_nome` o `grupo_conta` della riga | è contenuto diverso: nuova versione, voluta |
| correzione del testo di un articolo a parità di importo | voluta: **[SONDA]** X7 |
| modifica del **testo** di un avviso nel codice | limite reale e dichiarato: cambiare la frase di un `detalhe` cambia le impronte, e va trattato come cambio di schema (§4.9) |

La riga sulla forma Unicode dipendeva dalla sottoscelta D8.4, **approvata
nell'opzione (a)** il 2026-10-21: l'impronta è quindi invariante alla forma
Unicode, e le due tabelle qui sopra sono ora affermazioni, non alternative. La
revisione 1 affermava l'una e l'altra cosa in due punti diversi del documento;
rettificato dopo il riscontro R3C3-02.

### 4.6 Rappresentazione esatta dei valori

| Tipo | Forma canonica | Motivo e verifica |
|---|---|---|
| `Decimal` | `DEC:<mantissa>:<scala>` dopo `normalize()`, con la regola: mantissa `0` si scrive **sempre** `DEC:0:0` | `normalize()` elimina gli zeri finali, quindi `10`, `10.00` e `10.0000` danno `DEC:10:0` (**[SONDA]** X4 e X8). La regola sullo zero serve perché `-0.00` e `0.00` sono lo stesso denaro: senza di essa due impronte diverse per lo stesso valore. **Nessuna conversione in centesimi**: `0.005` e una quantità a quattro decimali non sono nel dominio dei centesimi, e convertirli sarebbe un errore (C21/D6 vale sugli importi, non su tutto). La mantissa di `Decimal::MAX` è rappresentabile: `DEC:79228162514264337593543950335:0` (**[SONDA]** X4) |
| `Dado<T>` | `DADO:` stato `:` fonte con prefisso di lunghezza `:` valore codificato, oppure `-` se assente | include **stato** e **fonte**, esclude il `motivo`: il motivo è prosa del codice e una sua riscrittura non deve cambiare le impronte. Lo stato distingue `MANCANTE` da `NAO_DISPONIVEL` da `N.A.`, quindi due assenze di natura diversa non collidono |
| assenza contro zero | `-` contro `DEC:0:0` | impronte **diverse** (**[SONDA]** X5): dato mancante non è zero tecnico, anche nell'impronta |
| enum | etichetta `SCREAMING_SNAKE_CASE` già usata in serde e nei `rotulo()` esistenti: `XML_NFE`, `OK`, `REVISAR`, `APROVADO`, `ERRO`, `PARCIAL`, `N.A.`, `NAO_VERIFICADO`, `BASE`, `ACESSORIO`, `DEDUCAO`, `INFORMATIVA`, `CUSTO`, … | **[VERIFICATO]**: le etichette esistono già (`modelos.rs:50-66`, `alocacao.rs:185-230`); non se ne inventano di nuove |
| `NaiveDate` | `YYYY-MM-DD` | forma di `Display` di chrono, già usata nei messaggi d'errore |
| `Competencia` | `YYYY-MM` | `Display` esistente (**[VERIFICATO]** `modelos.rs:396-400`) |
| `bool` | `0` / `1` | alfabeto chiuso |
| stringhe | byte UTF-8 in forma **NFC**, con prefisso di lunghezza | **D8.4**: §4.6.1 |
| collezioni | conteggio, poi gli elementi nell'ordine della tabella §4.6.2 | — |

#### 4.6.1 Forma Unicode — sottoscelta D8.4

Il problema è reale: la stessa descrizione visibile in NFC e in NFD occupa 15 e
16 byte e dà due impronte diverse (**[SONDA]** X10).

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** normalizzare in **NFC** solo per calcolare l'impronta, lasciando la stringa memorizzata intatta | due letture della stessa NF-e in forme Unicode diverse danno la **stessa** versione; nessun dato viene modificato; `unicode-normalization` è già dipendenza (**[VERIFICATO]** V11) | **raccomandata** |
| (b) usare i byte così come sono | una rigenerazione da un XML riscritto da un altro strumento perderebbe le approvazioni senza che nulla di contabile sia cambiato | sconsigliata |
| (c) normalizzare anche la stringa memorizzata | cambierebbe i dati del modello, cioè la 3B, fuori perimetro | esclusa |

Conseguenza dichiarata di (a): due stringhe **canonicamente equivalenti**
vengono unificate nell'impronta. È l'effetto desiderato, non un difetto, e
riguarda solo la versione: le stringhe originali restano distinte e intatte nel
record. Due stringhe soltanto *visivamente* simili ma non canonicamente
equivalenti restano distinte: NFC non è una normalizzazione di compatibilità.

**Attenzione:** la scelta (a) non si esaurisce nell'emissione. Se si normalizza
al momento di scrivere i campi ma si **ordina** sui byte memorizzati,
l'invarianza promessa non vale: è il riscontro R3C3-02, e la regola che lo
chiude è in §4.6.3. Senza quella regola, D8.4(a) è una promessa che la
grammatica non mantiene.

#### 4.6.2 Ordine delle collezioni

| Collezione | Ordine nel testo canonico | Perché |
|---|---|---|
| registrazioni (`LAN`) | per `ordinal` crescente | è l'ordine contrattuale C18 della 3B; `ordinal` è l'indice stesso (**[VERIFICATO]** V5) |
| `duplicatas` | ordine memorizzato | contrattuale: il riparto mette i resti nell'ultima rata (F3A-08/09), quindi l'ordine è informazione |
| `componenti` | ordine memorizzato | costruito da una lista letterale fissa (**[VERIFICATO]** `alocacao.rs:1246-1290`) |
| `articoli_scartati`, `avisos_documento` | ordine memorizzato | prodotti da 3B/fase 2 in sequenza deterministica per un dato documento |
| `campi_rinviati` | ordine memorizzato, e **solo** il nome del campo | la prosa di `status`, `fonte` e `motivo` non è contenuto del documento |
| `avisos_leitura` | **ordinati** con la regola canonica di §4.6.3 su `(codigo, detalhe)` | arrivano dal chiamante dentro `ContextoDocumento`: l'unico contenitore che un chiamante potrebbe permutare. L'ordinamento canonico segue il precedente già applicato in 3C.2 agli avvisi del lotto (**[VERIFICATO]** `prelivro.rs:739-741`, riscontro R3C2-03) |

#### 4.6.3 Ordinamento canonico: normalizzare **prima** di confrontare

Riscontro **R3C3-02** dell'audit del 2026-10-21. La revisione 1 diceva
"ordinati per `(codigo, detalhe)`" senza dire **su quale rappresentazione** si
confronta. La lettura naturale — ordinare i valori memorizzati e poi emetterli
in NFC — **rompe** l'invariante promessa da D8.4(a).

Controesempio N1, riprodotto (**[SONDA]** X12): due avvisi dello stesso codice,
dettagli `é` e `f`, in un documento dove `é` è precomposto e in un altro dove è
`e` + accento combinante. Testi canonicamente **equivalenti**:

```text
regola ingenua (ordina i byte memorizzati, poi emette NFC)
  precomposto -> ordine [f, é]  -> 6151e96332617e296c102f65e48fbfaa965dcc6ada53ef2be232b0fc1ef76f3a
  combinante  -> ordine [é, f]  -> 4bd43b9934781e9c7b35d9617c8f9ec46a4c05fa893e98fa84b3a6f63140bdcf
```

La causa è leggibile nel primo byte: `f` è `0x66`, `é` precomposto comincia con
`0xC3`, `é` combinante comincia con `0x65`. Ordinare i byte memorizzati mette
quindi `é` prima o dopo `f` **a seconda della forma Unicode**, e due documenti
con lo stesso contenuto ottengono due versioni diverse.

Nota sulle cifre: queste impronte sono quelle della **mia** costruzione
sintetica (documento ridotto, `LAN#0`, due avvisi con codice `AVISO`) e non
coincidono con quelle pubblicate dall'audit, che partiva da un documento base
diverso. Il **fenomeno** è lo stesso e il confronto fra le due coppie è
indipendente dalla costruzione: ciò che conta è che la coppia diverga con la
regola ingenua e coincida con la regola canonica.

**Regola canonica, [PROPOSTA], in quest'ordine esatto:**

1. **normalizza** in NFC ogni stringa che fa parte della chiave di ordinamento;
2. **ordina** confrontando le chiavi normalizzate, byte per byte, sui **byte
   UTF-8** (confronto ordinale: è la stessa relazione d'ordine di `Ord` per
   `&str` in Rust, e l'impronta è calcolata sugli stessi byte; **non** un
   confronto dipendente dalla lingua o dalle unità UTF-16);
3. **emetti** ogni campo in NFC, con il prefisso di lunghezza calcolato sui byte
   **dopo** la normalizzazione.

Con le chiavi normalizzate prima del confronto, i due documenti del
controesempio danno la **stessa** impronta
`6151e96332617e296c102f65e48fbfaa965dcc6ada53ef2be232b0fc1ef76f3a`
(**[SONDA]** X12, controllo positivo).

**Pareggi.** Due avvisi con chiavi NFC **uguali** ma byte memorizzati diversi
restano **entrambi** nell'impronta: nessuno viene scartato, fuso o dedotto
duplicato. Il pareggio è innocuo e non serve uno spareggio arbitrario, perché
dopo la normalizzazione i due avvisi emettono **gli stessi byte**: qualunque
ordine relativo produce lo stesso testo canonico. Verificato: permutando i due
avvisi equivalenti l'impronta resta
`94e7ec1303573254d1b4925fba9675edd9f8ecd088718b4a309966a20c534032`
(**[SONDA]** X12). Le stringhe originali restano distinte e intatte nel record.

La stessa disciplina vale per ogni contenitore il cui ordine è calcolato da un
testo: oggi `avisos_leitura` nell'impronta di contenuto, i conti mostrati
nell'impronta di presentazione (§4.7) e la traccia degli eventi (§5.4).

### 4.7 Il piano dei conti e l'impronta di presentazione — sottoscelta D8.3

Il mandato chiede: se cambia ciò che l'utente esamina, cambia la versione o
viene segnalato?

Distinzione **[PROPOSTA]**, fondata su un fatto **[VERIFICATO]**:
`Lancamento::conta_nome` è scritto dalla 3B al momento dell'allocazione, mentre
`LinhaPreLivro::conta_nome_plano` viene dal piano passato alla 3C.2 (V7). Sono
due cose diverse e vanno trattate diversamente.

| Che cosa cambia | Effetto sull'impronta di contenuto | Effetto sulla vista |
|---|---|---|
| il piano cambia **e** si rialloca: `conta_id`, `conta_nome` o `grupo_conta` della riga diventano altri | **cambia**: è contenuto diverso, nuova versione, approvazione non trasferita | la riga nuova nasce da revisionare |
| il piano cambia **senza** riallocare: cambia solo `conta_nome_plano` mostrato | **invariata** (**[SONDA]** X9) | `impressao_apresentacao` **diversa**, avviso `APRESENTACAO_MUDADA`, approvazioni **conservate** |

Testo canonico della presentazione, **[PROPOSTA]**:

```text
"V=IMPRESSAO-APRESENTACAO/v1" LF
"CT=" <64 hex dell'impronta di contenuto> LF
"PC#" n LF
  ( "PI=" str LF  "PN=" str LF  "PG=" str LF )*   // conti usati dalle righe,
                                                  // ordinati per conta_id con
                                                  // la regola di §4.6.3
```

L'ordinamento per `conta_id` segue la stessa disciplina di §4.6.3: chiavi
normalizzate prima del confronto, confronto ordinale sui byte UTF-8. Gli
identificativi del piano osservati sono ASCII (`2.01.004`, **[VERIFICATO]**
`config/plano_contas_gerencial.yaml`), quindi oggi la normalizzazione non
cambia nulla; la regola è dichiarata comunque, perché un contratto che vale
"solo finché i dati restano ASCII" non è un contratto.

Perché separarle: un'approvazione dice "ho esaminato questo contenuto". Se il
nome mostrato di un conto cambia, la persona non ha esaminato numeri diversi, ma
ha visto un'etichetta diversa: il fatto va **segnalato**, non trasformato in
perdita della decisione. L'avviso `NOME_DE_CONTA_DIVERGENTE` già esistente
(**[VERIFICATO]** `prelivro.rs:92-97`, `:588-608`) copre il caso gemello del
nome di riga divergente da quello del piano.

| Opzione D8.3 | Effetto | Giudizio |
|---|---|---|
| **(a)** due impronte separate, contenuto e presentazione | un rinomino nel piano segnala e non invalida | **raccomandata** |
| (b) una sola impronta che include i nomi del piano | rinominare un conto invalida **tutte** le approvazioni di tutti i documenti che lo usano | sconsigliata |
| (c) una sola impronta che ignora del tutto il piano | un cambio di etichetta diventa invisibile: la persona non sa che la vista non è più quella esaminata | sconsigliata |

### 4.8 Vettori calcolati

Tutti **[SONDA]**, con lo strumento validato in §1.3. Chi implementa deve
ottenere **esattamente** questi valori sugli stessi byte.

**Convenzione dei terminatori, da leggere prima di riprodurli:** i vettori A–H
sono impronte del **solo frammento**, **senza** `LF` finale; i vettori I e J
sono testi completi, con `LF` dopo **ogni** riga, ultima compresa. Sbagliare
questo dettaglio cambia l'impronta.

Le lunghezze in byte sono state ricontrollate una per una (**[SONDA]** X14): il
vettore F occupa **35** byte, non 33 come diceva la revisione 1 — `DEC:` più 29
cifre più `:0`. L'impronta pubblicata era e resta corretta; era sbagliato solo
il conteggio. Rettifica dopo l'audit del 2026-10-21.

**Avvertenza per chi riproduce il vettore I da questo repository**
(**[VERIFICATO]** al momento della registrazione): `core.autocrlf` è `true` e
non esiste `.gitattributes`. L'oggetto Git memorizzato contiene **LF** — il
blob registrato dà 948 byte e `f8a6e8d0…14c` — ma un checkout su Windows
converte le righe in **CRLF**, e con `CRLF` il blocco occupa 1.005 byte — 948
più un `CR` per ognuna delle 57 righe — e dà un'impronta diversa. Chi vuole riprodurre il vettore legge quindi il contenuto
registrato, per esempio con `git show HEAD:SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md`,
oppure converte i terminatori in LF prima di calcolare. Non è un difetto della
grammatica: la grammatica di §4.4 prescrive `LF` e vieta `CRLF`.

**Controprova con un secondo strumento** (**[SONDA]** X11): il blocco del
vettore I è stato estratto da **questo stesso documento** e misurato con
`sha256sum` di coreutils, cioè un'implementazione diversa da quella .NET usata
in §1.3. Risultato: 948 byte e `f8a6e8d0…14c`. Allo stesso modo sono stati
riprodotti J (952 byte, `4ce5f31b…7a7`), K (`44195583…324`), A
(`aee69fbd…87e`), F (`aec9288e…655`) e G (`705ef221…4fc`). I vettori di questa
tabella sono quindi verificabili **dal testo pubblicato**, non solo dalla sonda
originale.

| # | Oggetto | Byte | SHA-256 |
|---|---|---|---|
| A | `DEC:10:0` | 8 | `aee69fbd7f62989bcc01ef8b12312a3ae2e29e46565803cb51d3c11a1ee0887e` |
| B | `DEC:0:0` | 7 | `c8bcddd2d2f5eeaa5a4219ef9bca692407ec5f94bbd3463ffed24ffc0017ca09` |
| C | `DEC:5:3` | 7 | `f4e645c10e090c5480ecf6ea999f3c80a694c51069cc8ed08c9c3f0f4ec886fb` |
| D | `DEC:12345:4` | 11 | `1707861d7ba0a2019c7c5eed783104e488542f88bdd6e65c73b57627a0f3ed86` |
| E | `DEC:-75:1` | 9 | `9894b03c850a9ea0adffa6ed4bf29e5f7d5ec19c66370b1bacd2c58df8bc11d4` |
| F | `DEC:79228162514264337593543950335:0` | 35 | `aec9288e7db8a652090cf240be97465083d59a99a9caf778d09ff31db90d4655` |
| G | `DADO:NAO_DISPONIVEL:18:nfe:cobr/dup/dVenc:-` | 43 | `705ef2219871afabb3c09d229bbfc02484e9a72f4aa7eb0d9ab9f643a9c834fc` |
| H | `DADO:OK:18:nfe:cobr/dup/dVenc:DEC:0:0` | 37 | `08046d522d2bc09e0253be9de2186d21390fb7c3eddf9790531935c69fea0292` |
| I | documento minimo (testo sotto) | 948 | `f8a6e8d0aade811e39e31f966c17e0c6623f21afdbde73a8c5866641031f614c` |
| J | lo stesso documento con `descricao_itens` corretta in `COCA-COLA KS 290ML 2UN (10.00)` | 952 | `4ce5f31b07e4478dbb70e73cf03639a384be8e27aa802c7236de258c30f997a7` |
| K | presentazione di I con nome di piano `Refrigerantes` | — | `44195583d43090941ecc7f45c90636e859382a431928b8011941bc26c784b324` |
| L | presentazione di I con nome di piano `REFRIGERANTES` | — | `d1a8e9834a9a80efefb1430f28c49b53c5e1dac5a9f30c6a8996e4ca26566424` |

Il vettore I per intero. Ogni riga termina con un `LF`, **compresa l'ultima**;
il documento è **sintetico e ridotto**, costruito a mano per fissare i byte, e
**non** è l'uscita di `alocar_nfe` sul campione canonico:

```text
V=IMPRESSAO-CONTEUDO/v1
DCH=44:29260512345678000195550010001256612345678901
DNU=5:12566
DSE=1:1
DDT=2026-05-05
DCP=2026-05
DEM=14:55746722000181
DNE=21:RESTAURANTE OESA LTDA
AVL#0
LAN#1
LOR=0
LOG=XML_NFE
LCI=8:2.01.004
LCN=13:Refrigerantes
LGC=31:Custos de Mercadorias e Insumos
LDS=26:COCA-COLA KS 290ML (10.00)
LVL=DEC:10:0
LNE=21:RESTAURANTE OESA LTDA
LFV=23:DISTRIBUIDORA OESA LTDA
LCF=14:12345678000195
LTD=3:NFE
LTL=CUSTO
LDRI=0:
LDRN=0:
LCC=0:
LSR=OK
LAL=0:
LPG=DADO:NAO_DISPONIVEL:18:nfe:cobr/dup/dVenc:-
LDUP#0
REC
RVT=DEC:10:0
RSL=DEC:10:0
RET=OK
RRD=DADO:N.A.:17:nfe:cobr/dup/vDup:-
RRA=DADO:N.A.:17:nfe:cobr/dup/vDup:-
RER=N.A.
RAA=1
RAT=DADO:NAO_DISPONIVEL:7:nfe:det:-
RSC#0
RCO=OK
RCM#2
MN=5:vProd
MV=DADO:OK:23:nfe:total/ICMSTot/vProd:DEC:10:0
MR=BASE
MU=1
MN=5:vDesc
MV=DADO:OK:23:nfe:total/ICMSTot/vDesc:DEC:0:0
MR=DEDUCAO
MU=1
RCE=OK
RRN#4
FN=12:centro_custo
FN=12:dre_linha_id
FN=14:dre_linha_nome
FN=11:observacoes
RAD#0
RSD=OK
```

Osservazioni leggibili dal vettore: i campi vuoti si scrivono `0:` e non
spariscono; i quattro `campi_rinviati` della 3B compaiono per nome (F3A-17/18/19,
**[VERIFICATO]** `alocacao.rs:1320-1335`); `LSR=OK` è l'esito **tecnico**, e nessun
campo dell'evento di revisione compare in questo testo.

### 4.9 Impronta di contenuto e sequenza storica: due cose diverse — D8.5

| Domanda | Risposta |
|---|---|
| il modulo puro può calcolare l'impronta del contenuto? | **sì**: dipende solo dai dati ricevuti |
| il modulo puro può dire "questa è la versione 3 del documento"? | **no**: la numerazione richiede di sapere che cosa è già stato archiviato, e consultare l'ambiente per indovinarlo sarebbe un fallback silenzioso |
| che cosa restituisce allora? | `VersaoDocumento { chave_origem, impressao_conteudo: String, sequencia: Dado<u32>, motivo: Dado<MotivoNovaVersao> }` con `sequencia` e `motivo` **`NAO_DISPONIVEL`** e la fonte `"arquivo autorevole ausente (3C.4)"` |

| Opzione D8.5 | Effetto | Giudizio |
|---|---|---|
| **(a)** `sequencia` come `Dado` assente finché non esiste l'archivio | nessun numero inventato, carta d'identità coerente con C2 | **raccomandata** |
| (b) `sequencia: u32` con valore 1 di default | il primo lotto dichiarerebbe "versione 1" per ogni documento mai archiviato: un numero senza fonte | sconsigliata |
| (c) contare le versioni osservate in memoria | conterebbe solo le esecuzioni di questo processo: numero vero su un insieme falso | sconsigliata |

Il contratto di **confronto** del contenuto è quindi risolto ora; scritture,
conflitti fra archivi e gestione dello storico restano 3C.4 e **non** scelgono
D7 implicitamente: l'impronta non richiede un formato di archivio, richiede solo
che l'archivio, quando esisterà, memorizzi la stringa di 64 caratteri.

**Versionamento dello schema, [PROPOSTA]:** la prima riga `V=IMPRESSAO-CONTEUDO/v1`
entra nei byte, quindi un cambio di grammatica cambia tutte le impronte. Regola:
qualunque modifica dell'insieme dei campi, dell'ordine, della codifica **o del
testo di un avviso incluso** richiede `v2` e la conservazione dichiarata delle
impronte `v1` nell'archivio, perché le vecchie approvazioni restano legate a
`v1`. Una migrazione silenziosa sarebbe una perdita di decisioni.

### 4.10 Sottoscelte D8 — tutte approvate

| ID | Scelta | Opzione | Stato |
|---|---|---|---|
| D8.1 | chiave di bersaglio senza `lote`, corrispondenza unica, conflitto dichiarato (§4.1, §4.2) | **(a)** | **[APPROVATO]** 2026-10-21 |
| D8.2 | insieme dei campi dell'impronta: 18 inclusi, 5 omessi perché confrontati, 3 esclusi (§4.5) | **(a)** | **[APPROVATO]** 2026-10-21 |
| D8.3 | impronta di presentazione separata (§4.7) | **(a)** | **[APPROVATO]** 2026-10-21 |
| D8.4 | forma Unicode NFC solo per l'impronta, con normalizzazione **prima** dell'ordinamento (§4.6.1, §4.6.3) | **(a)** | **[APPROVATO]** 2026-10-21 |
| D8.5 | `sequencia` come `Dado` assente finché non esiste l'archivio (§4.9) | **(a)** | **[APPROVATO]** 2026-10-21 |

---
## 5. Proposta del primo lotto implementabile — 3C.3.a

### 5.1 Le due soluzioni, a confronto

| | **S1** — eventi accanto alle righe | **S2** — proiezione per copia validata |
|---|---|---|
| idea | la vista elenca le decisioni accanto a ogni riga; il `PreLivro` non viene ricostruito | gli eventi validati producono una **copia** dei `ContextoDocumento` con il solo `status_revisao` aggiornato, e si richiama `construir_pre_livro` |
| fonte 3B | intatta | intatta: la copia non tocca gli originali |
| riepiloghi e partizioni | **restano quelli pre-revisione**: una riga approvata continua a contarsi in `qtd_revisar` | coerenti: `qtd_aprovado`, `valor_aprovado`, `Particao`, `exclusoes_extrato` si aggiornano da sé |
| estratto approvato | non cambia: chi lo volesse aggiornato dovrebbe ricalcolarlo | si ottiene dal pre-libro proiettato, con il filtro più severo di §3.6 sopra |
| rischio principale | chiunque voglia i numeri rivisti deve **riscrivere l'aggregazione**: seconda verità sul denaro, cioè esattamente ciò che il contratto F3C-26 vieta | la copia proiettata non è più identica alla fonte: va reso impossibile confonderle |
| codice nuovo | poco, ma inutilizzabile a valle | poco, e riusa interamente la 3C.2 |

**Raccomandazione: S2**, con tre guardie dichiarate:

1. l'**unico** campo che la proiezione scrive è `status_revisao`; qualunque altra
   differenza fra fonte e copia è un bug, e il test T3-03 confronta i restanti
   25 campi di ogni `Lancamento`;
2. la vista conserva per ogni riga lo `status_tecnico` originale, quindi
   l'esito 3B non è recuperabile "per differenza" ma è scritto;
3. l'impronta di contenuto si calcola **sui documenti originali**, prima della
   proiezione: approvare non cambia l'impronta dell'oggetto approvato
   (requisito del mandato, test T3-15).

Nessun refactoring generale della 3C.2 è necessario, e nessuno è proposto.

### 5.2 API proposta

Due moduli nuovi. Firme **indicative**, non definitive.

```rust
// ----------------------------- src/impressao.rs -----------------------------
// Unica verità del testo canonico. Nessun errore possibile: la costruzione
// del testo e SHA-256 non hanno casi di fallimento.

pub const VERSAO_CONTEUDO: &str = "IMPRESSAO-CONTEUDO/v1";
pub const VERSAO_APRESENTACAO: &str = "IMPRESSAO-APRESENTACAO/v1";

/// Testo canonico di §4.4. Esposto perche' un test deve poter confrontare i
/// byte, non solo l'impronta.
pub fn texto_canonico(contexto: &ContextoDocumento) -> String;

/// SHA-256 esadecimale minuscolo del testo canonico.
pub fn impressao_conteudo(contexto: &ContextoDocumento) -> String;

/// Impronta di presentazione: §4.7. `conti` sono le coppie del piano
/// effettivamente usate dalle righe, ordinate per `conta_id`.
pub fn impressao_apresentacao(impressao_conteudo: &str, conti: &[ContaMostrada]) -> String;

// ------------------------------ src/revisao.rs ------------------------------

pub struct ChaveRevisao {
    pub chave_origem: String,
    pub impressao_conteudo: Dado<String>,   // assente = ProvenienciaAusente
    pub origem: OrigemLancamento,
    pub conta_id: String,
    pub ordinal: u32,
}

pub enum OperacaoRevisao { Aprovar, Revogar, Anotar }

pub struct EventoRevisao {
    pub alvo: ChaveRevisao,
    pub operacao: OperacaoRevisao,
    pub autor: String,                      // non vuoto
    pub instante: DateTime<FixedOffset>,    // dichiarato, fuso esplicito
    pub motivo: String,                     // non vuoto
    pub nota: Option<String>,               // obbligatoria per Anotar
    pub sequencia: Option<u64>,             // ordine dichiarato a pari istante
    pub vista: Dado<String>,                // impronta di presentazione vista
}

/// Come si nomina un evento in un messaggio, senza usare la sua posizione.
/// Porta le forme **dichiarate**: autore non normalizzato, istante con il suo
/// offset. Non e' richiesta unica: la determinatezza del messaggio viene dalla
/// scelta del primo evento in ordine canonico (§5.3), non dall'unicita' di
/// questa chiave.
pub struct ChaveEvento {
    pub chave_origem: String,
    pub conta_id: String,
    pub ordinal: u32,
    pub operacao: OperacaoRevisao,
    pub autor: String,
    pub instante: DateTime<FixedOffset>,
}

/// Codifica **iniettiva** dei dati dichiarati di un evento: produzioni
/// `str_exato` e `dado_esatto_string`, nessuna normalizzazione, istante con
/// l'offset dichiarato, tag di presenza e prefissi di lunghezza (§5.4.2). E'
/// l'unico criterio di identita' esatta, e il sesto livello dell'ordine
/// canonico. Esposto perche' un test deve poter confrontare i byte.
///
/// **Non** riusa le produzioni di `impressao.rs`: quelle normalizzano in NFC
/// ed escludono il `motivo` dei `Dado`, cioe' perdono informazione che qui
/// serve (riscontri C4 e C5).
pub fn texto_exato_evento(evento: &EventoRevisao) -> String;

/// Chiave di ordinamento canonico a sei livelli (§5.4.2). Non e' un criterio
/// di identita': due eventi distinti possono coincidere fino al livello 5.
pub fn chave_canonica(evento: &EventoRevisao) -> String;

pub enum EstadoRevisao {
    TecnicoOk, TecnicoRevisar,
    AprovadoPorPessoa, AprovacaoRevogada, AprovadoSemProveniencia,
}

pub struct LinhaRevisada {
    pub chave: ChaveRevisao,
    pub identidade: IdentidadeLinha,        // dalla 3C.2, con il lotto
    pub status_tecnico: StatusRevisao,      // l'esito 3B, mai riscritto
    pub estado: EstadoRevisao,
    pub decisoes: Vec<DecisaoAplicada>,     // ordine canonico di §5.4.2, tutte
    pub anotacoes: Vec<DecisaoAplicada>,    // idem
    pub no_extrato_revisado: bool,
}

/// Un evento applicato. Porta **l'evento**, non la sua posizione: nessun
/// indice d'ingresso compare nell'uscita (R3C3-01).
pub struct DecisaoAplicada {
    pub evento: EventoRevisao,
    /// Quante volte lo **stesso** evento e' arrivato, secondo l'identita'
    /// esatta di §5.4.2: 0 se unico. Due dichiarazioni diverse che si
    /// somigliano NON si contano qui, restano due decisioni.
    pub repeticoes: usize,
}

pub struct EventoNaoAplicado {
    pub evento: EventoRevisao,
    pub motivo: MotivoNaoAplicacao,
    pub detalhe: String,                    // che cosa non combaciava
}

pub enum MotivoNaoAplicacao {
    AlvoAusente, VersaoSuperada, ProvenienciaAusente, NadaARevogar, Repeticao,
}

pub struct VistaRevisada {
    pub pre_livro: PreLivro,                // proiettato: riepiloghi coerenti
    pub linhas: Vec<LinhaRevisada>,
    pub versoes: Vec<VersaoDocumento>,      // impronta per documento, §4.9
    pub trilha: Vec<DecisaoAplicada>,       // storia completa, nulla cancellato
    pub eventos_nao_aplicados: Vec<EventoNaoAplicado>,
    pub exclusoes_extrato_revisado: Vec<Exclusao>,
    pub avisos: Vec<Aviso>,                 // ordinati per (codigo, detalhe)
}

/// Gli errori nominano l'evento con la sua **chiave canonica** dichiarata
/// (bersaglio, operazione, autore, istante), non con la sua posizione
/// nell'array: permutare l'ingresso non cambia il messaggio (R3C3-01).
pub enum ErroRevisao {
    AutorNaoDeclarado { evento: ChaveEvento },
    MotivoNaoDeclarado { evento: ChaveEvento },
    NotaNaoDeclarada { evento: ChaveEvento },
    AlvoIncompleto { evento: ChaveEvento, campo: &'static str },
    AlvoAmbiguo { evento: ChaveEvento, chave_origem: String, encontrados: usize },
    DecisoesIncompativeis { primeiro: ChaveEvento, segundo: ChaveEvento, detalhe: String },
    PreLivro(ErroPreLivro),                 // propagato, non riscritto
}

/// Valida **tutti** gli eventi, poi proietta, poi costruisce la vista.
pub fn aplicar_revisao(
    documentos: &[ContextoDocumento],
    plano: &PlanoContas,
    eventos: &[EventoRevisao],
) -> Result<VistaRevisada, ErroRevisao>;
```

Chi fa che cosa, come chiede il mandato:

| Domanda | Risposta |
|---|---|
| chi **valida** una richiesta | la prima passata di `aplicar_revisao`: forma dell'evento (§3.4, famiglia "malformato") |
| chi verifica che riga e versione siano quelle esaminate | la seconda passata: corrispondenza su `ChaveRevisao` con `impressao_conteudo` calcolata dai documenti originali (§4.2) |
| chi **costruisce la vista** | la terza passata: copia proiettata, `construir_pre_livro`, poi assemblaggio di `VistaRevisada` |
| che cosa è letto | `documentos`, `plano`, `eventos`: tutti per riferimento, nessuna mutazione |
| che cosa è restituito | `VistaRevisada` per valore |
| che cosa è immutabile | tutto l'ingresso. La funzione è **pura**: nessun orologio, nessun filesystem, nessuna variabile d'ambiente |

### 5.3 Come si evita un risultato parzialmente applicato

Tre passate separate, **[PROPOSTA]**:

```text
0. ORDINA      mette gli eventi nell'ordine canonico COMPLETO di §5.4.2 (sei
               livelli, bersaglio e testo esatto compresi) e collassa soltanto
               le ripetizioni con dati dichiarati IDENTICI. Da qui in avanti la
               posizione nell'array d'ingresso non esiste piu'
1. VALIDA      tutti gli eventi, uno per uno, senza toccare niente
               -> errore di forma: Err(..), nessuna vista prodotta. Se piu'
                  eventi sono malformati si riporta il primo in ordine
                  CANONICO, non il primo arrivato
2. ABBINA      calcola le impronte dei documenti originali, risolve i bersagli,
               classifica gli eventi in applicabili / non applicabili,
               rileva conflitti e ambiguita'
               -> conflitto o ambiguita': Err(..), nessuna vista prodotta
3. PROIETTA    copia i contesti, scrive il solo status_revisao, richiama
               construir_pre_livro, assembla la vista
               -> errore del pre-libro: Err(ErroRevisao::PreLivro(..))
```

La passata 0 esiste per il riscontro R3C3-01: se l'ordinamento canonico
avvenisse **dopo** la validazione, l'errore riportato dipenderebbe ancora
dall'ordine d'ingresso, e due chiamate con gli stessi eventi permutati darebbero
due messaggi diversi. Non è un cambio di comportamento cosmetico: è ciò che
rende riproducibile anche il **fallimento**.

Due avvertenze sulla passata 0, dal residuo di R3C3-01:

1. deve usare la chiave **completa**: una chiave priva del bersaglio ordina gli
   eventi di righe diverse come se fossero intercambiabili;
2. il collasso usa l'**identità esatta**, non l'uguaglianza della chiave. Sono
   due relazioni diverse e confonderle significa scartare una decisione valida
   (C1) oppure lasciare la traccia in balìa dell'ingresso (C2, C3).

Poiché la funzione restituisce un valore e non muta nulla, un `Err` lascia il
chiamante **esattamente** come era: non esiste uno stato intermedio osservabile.
È la stessa disciplina già usata in 3C.2, dove un lotto è calcolabile
interamente o non viene prodotto (**[VERIFICATO]** `prelivro.rs:107-112`).

### 5.4 Duplicati, decisioni incompatibili, ordine

#### 5.4.1 L'errore della revisione 1

La revisione 1 ordinava per `(instante, sequencia, indice)` e ammetteva
l'**indice d'ingresso** come spareggio fra eventi compatibili, promettendo nella
riga successiva che permutare eventi compatibili non cambia la vista. Le due
affermazioni non stanno insieme: riscontro **R3C3-01** dell'audit del
2026-10-21, accolto.

Controesempio E1, riprodotto (**[SONDA]** X13): stesso bersaglio, due `Anotar`
con note diverse, **stesso istante**, **nessuna sequenza** in entrambi.

```text
regola della revisione 1 (spareggio per posizione nell'array)
  ingresso [nota A, nota B] -> traccia [nota A, nota B]
    -> 42742150e07f0eb273e34781ebb17dcb19eb9d289ec3ad7691d6273f2aa9daef
  ingresso [nota B, nota A] -> traccia [nota B, nota A]
    -> 2d7404e8732620374b23794f7ee549b07f1d80c2d7e899c78f23e625fc81b31a
```

Lo stato finale della riga coincide, ma la **traccia** no: due viste diverse per
gli stessi eventi. Dire che l'indice è "solo uno spareggio" non elimina la
dipendenza, perché proprio in quel caso è l'indice a decidere.

#### 5.4.2 Due nozioni distinte: identità esatta e ordine canonico

La revisione 2 aveva sostituito la regola difettosa con una chiave di
ordinamento, e aggiunto una dimostrazione di totalità **sbagliata**:

> «Due eventi che coincidono su tutti i livelli coincidono su tutti i campi
> dichiarati, quindi sono byte-identici.»

Falso, e per tre motivi diversi. Riscontro **R3C3-01 residuo** dell'audit
mirato del 2026-10-21, accolto; tre controesempi riprodotti (**[SONDA]** X15):

| Caso | Due eventi | Chiave della revisione 2 | Dati dichiarati |
|---|---|---|---|
| C1 | stesso documento, stesso autore, istante e motivo, **conti e ordinali diversi** | **uguale**: il bersaglio non era nei livelli | **diversi** |
| C2 | stesso bersaglio e dati, autore `José` precomposto contro `Jose` + accento combinante | **uguale**: il livello semantico normalizza in NFC | **diversi**: il record conserva i byte dichiarati |
| C3 | stesso bersaglio e dati, istante `12:00-03:00` contro `15:00+00:00` | **uguale**: il livello 1 confronta l'istante assoluto | **diversi**: l'evento conserva l'offset dichiarato |

A queste si sono aggiunti, nell'audit mirato successivo, `C4` e `C5` di §5.4.2:
non collisioni della chiave, ma della **codifica esatta** che la chiave usa al
livello 6.

Non sono collisioni di SHA-256: sono collisioni della **chiave di ordinamento**.
E hanno due conseguenze opposte, entrambe incompatibili col contratto: se il
collasso usasse l'uguaglianza della chiave, C1 chiamerebbe `Repeticao`
l'approvazione di **un'altra riga** e quella riga non verrebbe approvata; se il
collasso verificasse l'identità esatta, la chiave resterebbe in parità e
l'ordine della traccia tornerebbe a dipendere dall'ingresso. La revisione 2
lasciava aperte entrambe le letture.

**Correzione: due definizioni separate.** **[PROPOSTA]**

**(1) Identità esatta — l'unica che autorizza il collasso.** Due eventi sono
**lo stesso** evento se e solo se i loro **dati dichiarati** coincidono byte per
byte. Il criterio è il `texto_exato_evento`.

La revisione 3 lo descriveva a parole come "senza normalizzazione, testi
originali come sono", ma scriveva la grammatica **riusando le produzioni di
§4.4**, che normalizzano in NFC ed escludono il `motivo` dei `Dado`. Un
requisito astratto di iniettività non corregge una produzione che perde
informazione: riscontro `C4` dell'audit mirato del 2026-10-21, accolto, e lo
stesso difetto colpiva **ogni** testo del blocco, non solo i `Dado`.

Servono quindi **due produzioni nuove**, usate solo qui e distinte da quelle
dell'impronta tecnica, che restano invariate:

```text
str_exato          := byte_len ":" bytes_utf8_ORIGINALI      // nessun NFC
dado_esatto_string := "DADOX:" status_dado
                      ":" str_exato                          // fonte originale
                      ":" ("AUSENTE" | "PRESENTE:" str_exato) // valore
                      ":" ("AUSENTE" | "PRESENTE:" str_exato) // motivo
```

`dado_esatto_string` conserva **tutti** i dati tipizzati di `Dado<String>`:
stato, fonte, presenza e valore, presenza e motivo, ciascuno con tag di presenza
e lunghezza esplicita, così che `None` non possa mai coincidere con `Some`.
Entrambi i `Dado` del controesempio C4 sono validi secondo `Dado::novo`
(**[VERIFICATO]** `modelos.rs:146-157`: con valore presente il `motivo` può
essere `None` oppure `Some` non vuoto), quindi la differenza è un dato reale del
modello, non un caso impossibile.

```text
"EVENTO-EXATO/v1" LF
"ACH=" str_exato LF                 // chiave del documento
"AIM=" dado_esatto_string LF        // impronta di contenuto esaminata
"AOG=" origem LF
"ACI=" str_exato LF                 // conta_id
"AOR=" u32 LF                       // ordinal, come numero
"EOP=" operacao LF
"EAU=" str_exato LF                 // autore come dichiarato, NON normalizzato
"EIN=" str_exato LF                 // istante dichiarato, offset compreso
"ESQ=" ("SEM_SEQUENCIA" | u64) LF
"EMO=" str_exato LF                 // motivo
"ENO=" ("AUSENTE" | "PRESENTE:" str_exato) LF   // nota, NON normalizzata
"EVI=" dado_esatto_string LF        // impronta di presentazione vista
```

**Perché la correzione è sistematica e non puntuale.** Con la grammatica della
revisione 3, trascritta alla lettera, collidevano tre casi su cinque
(**[SONDA]** X15):

| Caso | Due eventi che differiscono per | Testo esatto della revisione 3 |
|---|---|---|
| C4 | `motivo` del `Dado` della vista: assente contro dichiarato | **uguale**: `dado` esclude il motivo |
| C5 | forma Unicode della **nota** | **uguale**: `str` normalizza in NFC |
| C2 | forma Unicode dell'**autore** | **uguale**: stessa causa di C5 |

Con `str_exato` e `dado_esatto_string` i cinque casi danno testi esatti
**diversi**, e nessuno di essi è una `Repeticao`.

**Requisito del contratto:** la codifica è **iniettiva** sui dati dichiarati —
non per asserzione, ma perché ogni campo di `EventoRevisao` compare, con tag di
presenza dove è opzionale e con lunghezza esplicita dove è testo. Conseguenza
operativa da rispettare: **ogni campo aggiunto in futuro a `EventoRevisao` va
aggiunto anche qui**, altrimenti il difetto C4 ritorna. Il test T3-22 verifica
l'iniettività sui casi noti, non su tutti i campi possibili: il limite è
dichiarato in §9.2.

**Che cosa resta invariato:** le produzioni `str` e `dado` di §4.4 e l'impronta
tecnica del contenuto. Qui **non** si sta chiedendo di includere la prosa dei
`motivo` nei byte del contenuto contabile: si sta completando soltanto
l'identità esatta dell'**evento**.

**(2) Ordine canonico — totale grazie al livello 6.** La posizione nell'array
non compare in nessun livello, né nell'uscita, né nella scelta dell'errore.

| Livello | Criterio | Regola |
|---|---|---|
| 1 | `instante` | confronto sull'**istante assoluto**: due dichiarazioni dello stesso momento in fusi diversi sono contemporanee, ed è giusto così |
| 2 | `sequencia` | le sequenze **dichiarate** si ordinano fra loro per valore crescente e precedono gli eventi **senza** sequenza: un'assenza non è uno zero e non scalza un dato dichiarato (C2 applicato all'ordinamento) |
| 3 | **bersaglio completo** | `(chave_origem, impressao_conteudo, origem, conta_id, ordinal)`, con `ordinal` confrontato come **numero**. Era il livello mancante, causa di C1 |
| 4 | `operacao` | etichetta chiusa |
| 5 | contenuto semantico | `(autor, motivo, nota, vista)` con la regola di §4.6.3: chiavi in NFC, confronto ordinale sui byte UTF-8; `nota` assente precede `nota` presente |
| 6 | **testo esatto** | il `texto_exato_evento` di (1), confrontato ordinalmente sui byte UTF-8 |

La **totalità** ora si dimostra, e in un passo solo: due eventi superstiti al
collasso hanno, per definizione di (1), testi esatti **diversi**, quindi il
livello 6 li separa sempre. Non serve nessuno spareggio residuo, e in
particolare nessun indice d'ingresso. La dimostrazione non passa più per
un'affermazione sui livelli semantici, che era il punto debole.

**Ordine globale e ordine per riga.** L'ordine canonico è **uno** e vale per la
traccia globale: si legge come una storia, cronologica, con i livelli 3–6 come
criteri di disambiguazione a parità di istante. `decisoes` e `anotacoes` di ogni
riga sono la **sottosequenza** della traccia globale relativa a quel bersaglio,
quindi ereditano lo stesso ordine senza una seconda regola da mantenere
allineata. Lo stesso vale per `eventos_nao_aplicados`.

Verifiche (**[SONDA]** X15, con le produzioni esatte di (1)): nei cinque
controesempi la chiave distingue i due eventi, nessuno è una `Repeticao`, e la
traccia è identica nelle due permutazioni — `54bc536b…130d` per C1,
`6a0e4376…cd2c` per C2, `e4f70810…f837` per C3, `d29328eb…b69f` per C4,
`4f619ebb…0195` per C5. Il controesempio E1 resta chiuso: `348ca271…6e2f` in
entrambi gli ordini.

**Nessuna deduplicazione semantica.** Due dichiarazioni **diverse** che
coincidono dopo la normalizzazione o dopo la perdita di un campo — C2, C3, C4,
C5 — restano **due** eventi: entrambi
applicati, entrambi consultabili, nessuno riscritto per farlo somigliare
all'altro. Trattarli come un unico evento sarebbe una **decisione nuova** da
sottoporre a Luca, non una conseguenza tecnica della NFC, e non serve a questo
mandato. C2 e C3 ricadono nel caso O4: due approvazioni della stessa riga con
dichiarazioni diverse, entrambe in traccia, avviso `APROVACAO_REPETIDA`, stato
invariato.

**Un solo ordine, per due usi.** L'ordine canonico è quello di **applicazione**
ed è anche quello della **traccia restituita**: una sola verità, nessuna
seconda regola da tenere allineata. Lo stesso vale per
`eventos_nao_aplicados`, per `decisoes` e `anotacoes` di ogni riga e per gli
`avisos` della vista, che restano ordinati per `(codigo, detalhe)` con la
regola di §4.6.3.

#### 5.4.3 Casi

| Caso | Comportamento proposto |
|---|---|
| due eventi con **dati dichiarati identici** (identità esatta di §5.4.2) | `Repeticao`: elencata, non riapplicata, nessun doppio conteggio (O3). Entrambe le occorrenze restano consultabili |
| **C1** — due `Aprovar` su bersagli diversi, tutto il resto uguale | **due** approvazioni, una per riga: bersagli diversi, quindi non è una ripetizione e nessuna delle due righe resta senza decisione |
| **C2** — stessa riga, autore in forme Unicode diverse | due eventi distinti, entrambi applicati e consultabili con i byte dichiarati; ordine deciso dal livello 6; avviso `APROVACAO_REPETIDA` (O4). Nessuna riscrittura dell'autore |
| **C3** — stessa riga, stesso istante assoluto, offset dichiarati diversi | idem: due eventi contemporanei ma distinti, entrambi conservati con il proprio offset |
| **C4** — stessa riga, `Dado` della vista con `motivo` assente contro dichiarato | due eventi distinti: `dado_esatto_string` conserva presenza e testo del motivo, quindi `None` non coincide con `Some` |
| **C5** — stessa riga, **nota** nelle due forme Unicode | due eventi distinti: la nota entra nel testo esatto **non normalizzata**; la forma NFC resta solo nel livello semantico 5 |
| due approvazioni della stessa riga, autore o motivo diversi | entrambe in traccia nell'ordine canonico, avviso `APROVACAO_REPETIDA`, stato invariato (O4) |
| due annotazioni, stesso istante, nessuna sequenza | **compatibili**: entrambe applicate, ordine canonico sul contenuto dichiarato, nessuna nota persa |
| approvazione e revoca con istanti **assoluti diversi** | si applicano in ordine canonico; l'ultima determina lo stato, **tutte** restano in traccia |
| approvazione e revoca con lo **stesso** istante e sequenze dichiarate **diverse** | la sequenza dichiarata decide: è un'informazione dell'operatore, non della struttura dati |
| approvazione e revoca con lo stesso istante e **nessuna** sequenza | `DecisoesIncompativeis`: **errore**. Nessun "vince l'ultimo" ricavato dall'array |
| approvazione e revoca con lo stesso istante e la **stessa** sequenza dichiarata | idem: errore. Due decisioni incompatibili alla stessa coordinata temporale non sono ordinabili, e inventare un ordine sarebbe peggio di fermarsi |
| una delle due ha la sequenza e l'altra no, stesso istante, operazioni incompatibili | **errore**: le due non sono confrontabili come intenzione umana. Il livello 2 serve a ordinare in modo riproducibile, non a dedurre che chi ha dichiarato una sequenza volesse precedere; per le operazioni incompatibili serve una coordinata esplicita su **entrambe** |

L'ultimo caso è la precisazione chiesta dall'audit: il livello 2 della chiave
**ordina** sempre, ma per le operazioni **incompatibili** l'ordinamento non
basta: serve che la precedenza sia dichiarata su entrambi gli eventi, altrimenti
si dichiara il conflitto. Gli eventi compatibili, invece, si ordinano e si
applicano tutti.

Il conflitto si valuta **per bersaglio**: due operazioni incompatibili sono tali
solo se riguardano la **stessa** riga nella **stessa** versione. Un'approvazione
e una revoca contemporanee su due righe diverse non sono in conflitto, e C1
mostra perché la distinzione non è teorica: senza il bersaglio nella chiave, due
decisioni rivolte a righe diverse sembravano la stessa decisione.

Questo è il gemello di I7/R3C2-03 per gli eventi: permutare gli eventi non
cambia la vista, traccia compresa (test T3-17).

### 5.5 Rigenerazione e consultabilità

| Situazione | Esito |
|---|---|
| rigenerazione **identica**, lotto diverso | approvazioni, revoche e annotazioni si riapplicano: la corrispondenza è su `ChaveRevisao`, che non contiene il lotto (R1, test T3-10) |
| contenuto cambiato | gli eventi restano **consultabili** in `eventos_nao_aplicados` con motivo `VersaoSuperada` e con l'impronta che citano; la riga nuova nasce con l'esito tecnico (R2, test T3-08) |
| annotazione su contenuto cambiato | stesso trattamento: nessuna annotazione viene spostata su un contenuto che nessuno ha annotato |

Nessuna promessa oltre questo: la vista non ricostruisce il documento vecchio.
Confrontare due versioni affiancate richiede l'archivio, cioè 3C.4.

### 5.6 Dove vivono le informazioni senza persistenza

**[PROPOSTA]** e dichiarato senza giri di parole:

- gli eventi sono un **parametro** di `aplicar_revisao`: il modulo non li legge
  e non li scrive da nessuna parte;
- non esiste archivio, non esiste file, non esiste database: chi invoca li tiene
  in memoria o li costruisce nel test;
- `VersaoDocumento::sequencia` è `NAO_DISPONIVEL` con fonte dichiarata (D8.5);
- una dimostrazione in memoria **non** è uno storico salvato, e questo blocco
  **non** va presentato come "revisione tracciata in modo persistente";
- la persistenza è 3C.4 e dipende da D7, che resta da decidere.

Conseguenza operativa onesta: finché non esiste l'archivio, chiudere il processo
perde gli eventi. Il valore del lotto è il **contratto**: quando l'archivio
arriverà, dovrà solo memorizzare gli eventi e le impronte, senza ridiscutere il
significato dell'approvazione.

### 5.7 File, dipendenze, superficie

| File | Intervento |
|---|---|
| `src/impressao.rs` | **nuovo**: testo canonico e impronte |
| `src/revisao.rs` | **nuovo**: eventi, validazione, proiezione, vista |
| `src/lib.rs` | due righe: `pub mod impressao;`, `pub mod revisao;` |
| `src/prelivro.rs` | **non toccato** |
| `src/alocacao.rs`, `src/dinheiro.rs`, `src/modelos.rs`, `src/nfe.rs`, `src/regras.rs` | **non toccati** |
| `Cargo.toml`, `Cargo.lock` | **non toccati**: `sha2`, `chrono`, `unicode-normalization` sono già dipendenze (**[VERIFICATO]** V11) |
| `src/main.rs` (CLI) | **non toccato**: l'uso da riga di comando è 3C.5 |
| `config/*`, `samples/*` | **non toccati** |
| `README.md` | aggiornato solo a lotto implementato, non adesso |

Comportamento della 3B e della 3C.2: **invariato**. I 208 test esistenti devono
restare verdi senza modifiche; se qualcuno dovesse cambiare, è il segnale che il
lotto ha superato il suo perimetro.

### 5.8 Che cosa questo lotto **non** fa

| Fuori perimetro | Dove vive |
|---|---|
| persistenza, archivio, conflitti fra archivi | 3C.4, D7 |
| import dello storico Excel, schemi 26/31 | 3C.4, D11 |
| adattatori di ingressi grezzi, `INCOMPLETO`, `NAO_RECONHECIDO` | 3C.4, D12 |
| esportazione, scrittura atomica, round-trip | 3C.4 |
| uso da CLI e validazione condivisa C15 | 3C.5 |
| creazione o modifica di righe manuali | nessun blocco pianificato: richiede una decisione a sé |
| correzione di conto, importo, pagamento, regole YAML | §3.8 |
| autenticazione, ruoli, permessi | fuori progetto per ora |
| DRE | 3D |

---

## 6. Matrice dei contratti

### 6.1 Contratti F3C esistenti

| ID | Regola | Stato oggi | Nel lotto 3C.3.a | Test pianificato |
|---|---|---|---|---|
| F3C-04 | nessuna promozione automatica ad `APROVADO` | **VINCOLO_GIA_APPROVATO** (metodo) | **coperto**: solo un evento dichiarato approva | T3-01, T3-05 |
| F3C-05 | l'approvazione non cancella avvisi né cambia gli esiti tecnici | PROPOSTO in 3C | **coperto** | T3-02, T3-03 |
| F3C-13 | identità di riga con scope e stabilità definite | PROPOSTO (D8) | **coperto e corretto**: lo scope per la revisione è `ChaveRevisao` senza lotto; `IdentidadeLinha` resta com'è | T3-07, T3-10 |
| F3C-22 | campi di revisione umana preservati alla rigenerazione | EREDITATO_VERIFICATO + PROPOSTO | **coperto in memoria**: le decisioni si riapplicano a contenuto identico. La persistenza resta 3C.4 | T3-10, T3-15 |
| F3C-23 | nessuna cancellazione dello storico senza decisione | VINCOLO_GIA_APPROVATO | **coperto nella traccia**: revoca e ripetizione non cancellano nulla | T3-04 |
| F3C-25 | ordinamento deterministico, permutazioni indifferenti | implementato in 3C.2 | **esteso agli eventi** | T3-11, T3-17 |
| F3C-26 | aggregati in centesimi interi con gli helper condivisi | implementato in 3C.2 | **rispettato per riuso**: nessuna nuova aritmetica | — (coperto dai test 3C.2) |
| F3C-14, F3C-15 | reimportazione identica; nuova versione con storico conservato | PROPOSTO (D8) | **parzialmente**: l'impronta che li rende decidibili nasce qui; il riconoscimento fra esecuzioni archiviate è 3C.4 | T3-08, T3-10 |
| F3C-06, F3C-10, F3C-16..21, F3C-24 | ingressi grezzi, formato, scritture, round-trip, schemi | PROPOSTO | **rinviati**: dipendono da IO o da adattatori che non esistono | — |

### 6.2 Contratti nuovi proposti

Numerazione nella stessa famiglia `F3C`, continuando da 27. **Nessuno** è
approvato da questo documento.

| ID | Regola | Input / assenza / conflitto | Esito | Codice futuro | Test |
|---|---|---|---|---|---|
| F3C-28 | un'approvazione esiste **solo** come evento dichiarato: autore, istante e motivo non sono dedotti da ambiente, utente di sistema o orologio | evento senza autore/motivo | errore nominato, nessun default | `revisao.rs` | T3-05 |
| F3C-29 | esito tecnico, decisione umana e stato della vista sono tre campi distinti, tutti conservati | riga approvata | `status_tecnico` resta leggibile | `revisao.rs` | T3-01, T3-02 |
| F3C-30 | la proiezione scrive **un solo** campo (`status_revisao`); gli altri 25 campi di `Lancamento` sono identici alla fonte | approvazione | confronto campo per campo | `revisao.rs` | T3-03 |
| F3C-31 | la revoca è un nuovo evento; l'approvazione precedente resta nella traccia | revoca dopo approvazione | due eventi, stato tecnico ripristinato | `revisao.rs` | T3-04 |
| F3C-32 | evento malformato = errore che nomina campo, evento e operazione; nessuna vista prodotta | autore vuoto, motivo vuoto, nota assente su `Anotar` | `Err` | `revisao.rs` | T3-05 |
| F3C-33 | un evento che cita una versione superata o un bersaglio assente **non** si applica ad altra riga o versione, e resta consultabile | impronta diversa | `eventos_nao_aplicados` con motivo | `revisao.rs` | T3-06, T3-08 |
| F3C-34 | impronta di contenuto `IMPRESSAO-CONTEUDO/v1`: testo canonico dichiarato, prefissi di lunghezza, campi inclusi/esclusi nominati, SHA-256 esadecimale minuscolo | qualunque documento | 64 caratteri riproducibili | `impressao.rs` | T3-12, T3-13 |
| F3C-35 | due righe con lo stesso `hash_registro` restano bersagli distinti e decidibili separatamente | payload C23 ripetuto | due decisioni indipendenti, nessuno scarto | `revisao.rs` | T3-07 |
| F3C-36 | una correzione a parità di chiave, conto e totale produce un'impronta **diversa** | descrizione corretta | nuova versione, nessun riuso automatico | `impressao.rs` | T3-08 |
| F3C-37 | l'impronta è invariante a percorso, lotto, campi umani, ordine di arrivo degli avvisi e **forma Unicode** (D8.4(a) approvata); **non** è invariante a cambio di `conta_id`/`conta_nome`/`grupo_conta` di riga, correzione del testo di un articolo, modifica del testo di un avviso nel codice. Le due liste complete sono in §4.5.2 | `arquivo_origem` diverso; stesso testo in NFC e NFD | impronta identica in entrambi i casi | `impressao.rs` | T3-13, T3-20 |
| F3C-38 | impronta di presentazione separata: un cambio di nome nel piano non invalida le approvazioni e produce `APRESENTACAO_MUDADA` | piano rinominato | approvazioni conservate, cambio segnalato | `impressao.rs`, `revisao.rs` | T3-09 |
| F3C-39 | evento con dati dichiarati **identici** (F3C-45) = ripetizione dichiarata; due decisioni incompatibili **sullo stesso bersaglio** alla stessa coordinata temporale = conflitto, senza "vince l'ultimo" implicito | due eventi opposti, stesso bersaglio, stesso istante | `Err(DecisoesIncompativeis)` | `revisao.rs` | T3-14 |
| F3C-40 | la sequenza storica non è calcolabile dal modulo puro: `Dado` `NAO_DISPONIVEL` con fonte dichiarata | nessun archivio | nessun numero inventato | `revisao.rs` | T3-16 |
| F3C-41 | tutto o niente: in errore non si produce una vista parzialmente applicata | evento malformato in mezzo a eventi validi | `Err`, nessuna vista | `revisao.rs` | T3-18 |
| F3C-42 | un `APROVADO` senza evento di provenienza non è una revisione verificata, non perde il dato originale, non entra nell'estratto revisionato | riga importata `APROVADO` | `APROVADO_SEM_PROVENIENCIA` + avviso + esclusione motivata | `revisao.rs` | T3-16 |
| F3C-43 | ogni ordinamento canonico **normalizza le chiavi prima** di confrontarle, confronta sui byte UTF-8 della forma NFC, ed emette i campi in NFC; i pareggi canonici non scartano né fondono nulla | due avvisi canonicamente equivalenti in forme Unicode diverse | impronta identica; entrambi gli avvisi presenti | `impressao.rs` | T3-13, T3-20 |
| F3C-44 | l'ordine canonico degli eventi ha **sei** livelli — istante assoluto, sequenza (dichiarata prima dell'assenza), **bersaglio completo**, operazione, contenuto semantico in NFC, **testo esatto** — ed è totale sugli eventi superstiti al collasso; la posizione nell'array non compare nell'ordine, nell'uscita, né nella scelta dell'errore riportato | eventi permutati, anche malformati; C1, C2, C3 | vista, traccia e messaggi d'errore identici | `revisao.rs` | T3-17, T3-21, T3-22 |
| F3C-45 | l'uguaglianza che autorizza il collasso è l'**identità esatta** dei dati dichiarati, **non** l'uguaglianza della chiave di ordinamento; nessuna deduplicazione semantica fra dichiarazioni diverse | C1 (bersagli), C2 (autore NFC/NFD), C3 (offset), C4 (`motivo` del `Dado`), C5 (nota NFC/NFD) | tutti gli eventi applicati e consultabili; `Repeticao` solo per dati identici | `revisao.rs` | T3-22 |
| F3C-46 | `texto_exato_evento` usa produzioni **proprie** (`str_exato`, `dado_esatto_string`) che conservano byte originali, presenza e `motivo` dei `Dado`; **non** riusa `str` e `dado` dell'impronta tecnica, che normalizzano ed escludono il motivo. Ogni campo aggiunto a `EventoRevisao` va aggiunto alla codifica | due `Dado` validi che differiscono solo nel `motivo`; nota in NFC e in NFD | testi esatti **diversi**; impronta tecnica del contenuto **invariata** | `revisao.rs` | T3-22 |

---

## 7. Casi minimi come test pianificati

**Non esistono e non sono stati eseguiti.** Per ognuno: ingresso, atteso,
errore o stato, e blocco che lo implementerà. La colonna "caso" riporta
l'identificativo del mandato.

| Test | Caso | Ingresso | Atteso | Errore / stato | Blocco |
|---|---|---|---|---|---|
| T3-01 | RC3-01 | documento `OK`, riga `OK`, un `Aprovar` completo | `estado = APROVADO_POR_PESSOA`, `status_tecnico = OK`, traccia con autore, istante, motivo, impronta | nessun errore | 3C.3.a |
| T3-02 | RC3-02 | riga `REVISAR` con due avvisi, un `Aprovar` | `alertas`, `motivos`, quattro esiti, `stato_documento` e `Reconciliacao` **identici** prima e dopo | nessun errore | 3C.3.a |
| T3-03 | RC3-02 | come T3-02 | i 25 campi di `Lancamento` diversi da `status_revisao` sono identici fra fonte e copia proiettata | nessun errore | 3C.3.a |
| T3-04 | RC3-04 | `Aprovar` poi `Revogar`, istanti diversi | `estado = APROVACAO_REVOGADA`, stato tecnico ripristinato, **due** eventi in traccia, avviso `APROVACAO_REVOGADA` | nessun errore | 3C.3.a |
| T3-05 | RC3-05 | quattro eventi difettosi: autore vuoto, motivo vuoto, `Anotar` senza nota, `alvo` con chiave vuota | quattro errori distinti che nominano il campo e l'evento con la sua `ChaveEvento`, **non** la sua posizione nell'array | `Err`, nessuna vista | 3C.3.a |
| T3-06 | RC3-06 | evento su `ordinal` inesistente e evento su `chave_origem` assente dal lotto | due `eventos_nao_aplicados` con `AlvoAusente`; **nessuna** riga toccata | vista prodotta | 3C.3.a |
| T3-07 | RC3-07 | documento con due righe di uguale `hash_registro` (caso già coperto in 3B, V4), `Aprovar` solo la seconda | la seconda approvata, la prima no; nessuno scarto; due bersagli distinti per `ordinal` | nessun errore | 3C.3.a |
| T3-08 | RC3-08 | approvazione su impronta `I`, poi stesso documento con `descricao_itens` corretta | impronta diversa (vettori I e J di §4.8), evento `VersaoSuperada`, riga nuova con esito tecnico | vista prodotta | 3C.3.a |
| T3-09 | RC3-09 | due costruzioni con piani che differiscono solo nel nome del conto | impronta di contenuto **identica**, impronta di presentazione **diversa**, avviso `APRESENTACAO_MUDADA`, approvazione conservata | nessun errore | 3C.3.a |
| T3-10 | RC3-10 | stesso contenuto con `IdentidadeLote` diversa | corrispondenza trovata, decisioni riapplicate, `IdentidadeLinha` con il lotto nuovo | nessun errore | 3C.3.a |
| T3-11 | RC3-11 | permutazione dei **documenti** in ingresso | `VistaRevisada` identica; i bersagli sono gli stessi | nessun errore | 3C.3.a |
| T3-11b | RC3-11 | permutazione degli **articoli dell'XML** | caso **distinto**: cambia `ordinal` e quindi l'impronta. Atteso: versione diversa, non "stessi bersagli" | vista prodotta | 3C.3.a |
| T3-12 | RC3-12 | i vettori A–F di §4.8 | ogni `Decimal` dà la forma canonica attesa; `10`, `10.00`, `10.0000` coincidono; `0.005` e `1.2345` non vengono arrotondati; `Decimal::MAX` è rappresentato | nessun errore | 3C.3.a |
| T3-12b | RC3-12 | vettori G e H, più testi con `|`, `:`, `=`, `#`, `LF` e virgolette dentro `descricao_itens` | assenza diversa da zero; nessuna collisione fra campi grazie al prefisso di lunghezza (sonda X3 come riferimento) | nessun errore | 3C.3.a |
| T3-13 | RC3-13 | stesso documento con `arquivo_origem`, `IdentidadeLote` e `observacoes` diversi; poi lo stesso testo in NFC e NFD; poi, **obbligatorio**, un documento con **due** `avisos_leitura` dello stesso codice le cui chiavi si ordinano in modo diverso secondo la forma Unicode (controesempio N1, §4.6.3) | impronta **identica** nei primi tre casi; identica fra NFC e NFD, essendo D8.4(a) approvata; identica fra i due documenti del caso a due avvisi, che è l'unico a scoprire l'ordinamento non canonico | nessun errore | 3C.3.a |
| T3-20 | RC3-13 | due `avisos_leitura` con chiavi NFC **uguali** ma byte memorizzati diversi, nei due ordini possibili | impronta identica nei due ordini; **entrambi** gli avvisi presenti nel testo canonico; le stringhe originali intatte nel record | nessun errore | 3C.3.a |
| T3-14 | RC3-14 | evento duplicato con dati dichiarati **identici**; poi `Aprovar` e `Revogar` sullo **stesso bersaglio** con lo stesso istante e nessuna sequenza; poi un bersaglio costruito a mano per forzare l'ambiguità | `Repeticao` elencata; `DecisoesIncompativeis` che nomina i due eventi con la loro `ChaveEvento`; `AlvoAmbiguo` | `Err` negli ultimi due | 3C.3.a |
| T3-15 | RC3-15 | `Anotar`, poi `Aprovar` | annotazione conservata in traccia, `observacoes` ancora vuoto, impronta di contenuto **identica** prima e dopo l'approvazione | nessun errore | 3C.3.a |
| T3-16 | RC3-16 | documento senza righe; riga in ingresso con `status_revisao = APROVADO` e nessun evento | documento presente con la sua identità (I8, già coperto in 3C.2); riga in `APROVADO_SEM_PROVENIENCIA`, avviso, esclusa dall'estratto revisionato con motivo; `sequencia` del documento `NAO_DISPONIVEL` | nessun errore | 3C.3.a |
| T3-17 | RC3-14 | due `Anotar` sullo stesso bersaglio con note diverse, **stesso istante** e **nessuna sequenza** in entrambi, nelle due permutazioni (controesempio E1, §5.4.1) | **l'intera** `VistaRevisada` identica, traccia e `decisoes`/`anotacoes` di riga comprese: non basta confrontare lo stato della riga. Un test con istanti diversi **non** chiuderebbe il controesempio | nessun errore | 3C.3.a |
| T3-21 | — | due eventi **malformati** (autore vuoto e motivo vuoto) nelle due permutazioni | lo **stesso** errore in entrambi i casi, scelto in ordine canonico e non per posizione; l'errore nomina l'evento con la sua `ChaveEvento` | `Err` identico | 3C.3.a |
| T3-22 | — | i **cinque** controesempi del residuo R3C3-01, ognuno nelle **due** permutazioni: **C1** due `Aprovar` su conti e ordinali diversi con autore, istante e motivo uguali; **C2** stessa riga con autore `José` precomposto e con accento combinante; **C3** stessa riga con `12:00-03:00` e `15:00+00:00`; **C4** stessa riga con il `Dado` della vista che differisce **solo** nel `motivo` (assente contro dichiarato), costruito con `Dado::novo` così da essere valido in entrambi i casi; **C5** stessa riga con la **nota** nelle due forme Unicode | in C1 **entrambe** le righe risultano approvate e nessun evento è `Repeticao`; in C2–C5 i due eventi restano **distinti** e consultabili con i byte, l'offset e il motivo dichiarati, con `APROVACAO_REPETIDA`; in tutti e cinque `VistaRevisada` e traccia sono identiche nelle due permutazioni. Più due controlli di codifica: due eventi distinti non producono lo stesso `texto_exato_evento`, due eventi con dati identici sì, e l'impronta **tecnica** del contenuto resta invariata fra C4 e C5 (i `motivo` e le forme Unicode non entrano in quei byte) | nessun errore | 3C.3.a |
| T3-18 | — | lotto di eventi validi con **uno** malformato in mezzo | `Err`; nessuna vista; nessun effetto osservabile | `Err` | 3C.3.a |
| T3-19 | RC3-03 | documento `ERRO` con un `Aprovar` | se D9.2(a): evento registrato, riga `APROVADO_POR_PESSOA`, **fuori** dall'estratto con motivo, avviso `APROVACAO_EM_DOCUMENTO_ERRO`. Se D9.2(b): errore dichiarato | dipende da D9.2 | 3C.3.a |

I casi del mandato sono coperti così: RC3-01→T3-01, RC3-02→T3-02/03,
RC3-03→T3-19, RC3-04→T3-04, RC3-05→T3-05, RC3-06→T3-06, RC3-07→T3-07,
RC3-08→T3-08, RC3-09→T3-09, RC3-10→T3-10, RC3-11→T3-11 e T3-11b (separati, come
chiesto), RC3-12→T3-12 e T3-12b, RC3-13→T3-13 e T3-20, RC3-14→T3-14 e T3-17,
RC3-15→T3-15, RC3-16→T3-16. T3-18 è aggiunto perché "tutto o niente" è una
proprietà che nessun altro caso verifica; T3-20 e T3-21 nascono dai riscontri
R3C3-02 e R3C3-01 dell'audit del 2026-10-21, T3-22 dal residuo di R3C3-01
rilevato dall'audit mirato della revisione 2.

**Distinzione dalle sonde:** le sonde X1–X10 di §1.3 sono **già eseguite** in
PowerShell e fissano i byte attesi. I test T3-* sono **futuri test Rust** e
dovranno riprodurre quegli stessi byte: è il collegamento che trasforma le
sonde in contratto, e senza di esso i vettori resterebbero un'affermazione.

---

## 8. Decisioni: approvate e rinviate

### 8.1 D9 — significato della revisione *(approvata)*

| ID | Opzione scelta | Opzioni scartate | Stato |
|---|---|---|---|
| **D9.1** | **(a)** "visto e accettato" con autore, istante e motivo dichiarati, senza cancellazione di avvisi e senza trasferimento a nuove versioni | (b) l'approvazione sostituisce gli esiti tecnici: **non ammissibile** (§10.5 della specifica 3C); (c) nessuno stato di approvazione in 3C: lascia il pre-libro senza uscita | **[APPROVATO]** 2026-10-21 |
| **D9.2** | **(a)** `Aprovar` valido su documento `ERRO`, registrato e **non** promosso, con esclusione motivata e avviso | (b) rifiutare l'`Aprovar` lasciando solo `Anotar`: la traccia esisterebbe ma senza stato interrogabile; (c) promuoverlo: contraddice D10(a) | **[APPROVATO]** 2026-10-21 |
| **D9.3** | **(a)** la revoca ripristina lo **stato tecnico** (`OK` o `REVISAR` come calcolato dalla 3B) | (b) la revoca porta sempre a `REVISAR`: inventerebbe un obbligo di revisione dove l'esito tecnico era `OK` | **[APPROVATO]** 2026-10-21. **Raffina** la §4.1 della specifica 3C generale, che scriveva `APROVADO → REVISAR`: la raffinazione è stata dichiarata e decisa, non ereditata per silenzio |
| **D9.4** | **(a)** `APROVADO` senza provenienza = stato dichiarato, escluso dall'estratto revisionato, sanabile con un evento esplicito | (b) accettarlo come revisione valida: l'approvazione diventerebbe non verificabile; (c) rifiutare il lotto: bloccherebbe l'import futuro | **[APPROVATO]** 2026-10-21 |
| **D9.5** | **(a)** evento malformato = errore; evento non applicabile = elencato e consultabile | (b) qualunque anomalia fa fallire il lotto: lo strumento diventerebbe inutilizzabile dopo la prima rigenerazione | **[APPROVATO]** 2026-10-21 |

### 8.2 D8 — identità, versione, impronta *(approvata)*

Le cinque sottoscelte, tutte **[APPROVATO]** nell'opzione (a) il 2026-10-21,
sono in §4.10 con il riferimento alla sezione che le definisce: **D8.1** chiave
senza lotto, **D8.2** insieme dei campi, **D8.3** impronta di presentazione
separata, **D8.4** NFC solo per l'impronta e prima dell'ordinamento, **D8.5**
sequenza come `Dado` assente.

### 8.3 Decisioni che restano rinviate, e verifica delle dipendenze

Il mandato chiede di dimostrare un'eventuale dipendenza reale prima di
allargare il perimetro. Verifica:

| Decisione | Serve a 3C.3.a? | Dimostrazione |
|---|---|---|
| **D7** fonte autorevole | **no** | l'impronta non richiede un formato: richiede che l'archivio, quando esisterà, conservi una stringa di 64 caratteri e gli eventi. Nessun campo della proposta dipende dalla scelta fra archivio strutturato e workbook |
| **D11** import dello storico | **no** | il caso "`APROVADO` senza provenienza" è trattato come stato dichiarato (D9.4) **senza** leggere alcuno storico. Serve a 3C.4 per decidere che farne |
| **D12** importi non interpretabili | **no** | non rappresentabili nell'ingresso tipizzato: `Lancamento::valor` è un `Decimal` sempre valido (**[VERIFICATO]**, già accertato in 3C.2) |
| **D10**, **D13** | già **approvate** | non riaperte; §3.7 riguarda la registrazione dell'evento, non il filtro |

Nessuna dipendenza reale trovata: il perimetro **non** viene allargato.

---

## 9. Rischi e limiti

### 9.1 I tre rischi nominati dal mandato

| Rischio | Dove nasce | Come la proposta lo chiude | Residuo |
|---|---|---|---|
| approvazione attribuita alla **versione sbagliata** | `IdentidadeLinha` non distingue le versioni (V5) | la decisione si lega a `impressao_conteudo`; corrispondenza esatta e unica, conflitto dichiarato (§4.2) | se D8.2 cambiasse l'insieme dei campi, le vecchie impronte non combacerebbero: per questo serve `v2` esplicito (§4.9) |
| cambiamenti **invisibili** all'impronta | un'impronta troppo stretta (per esempio il payload C23) | 18 campi di riga, riconciliazione completa, correzione a parità di totale rilevata (vettori I e J) | `observacoes` è escluso per scelta: se un blocco futuro lo popolasse, la modifica non cambierebbe la versione. **Limite dichiarato** |
| falsa equivalenza fra **stato tecnico e decisione umana** | `StatusRevisao` contiene `Aprovado` ma la 3B non lo produce (V1) | tre nozioni separate (§2.2), `APROVADO_SEM_PROVENIENCIA` per ciò che arriva senza evento (D9.4) | la vista di 3C.3 è più severa dell'`extrato_aprovado()` della 3C.2: due insiemi con due nomi, legati da un'invariante verificabile (T3-16) |
| **dipendenza dall'ordine d'ingresso**, cioè I7 violata sugli eventi e sulle collezioni testuali | la revisione 1: spareggio per posizione nell'array (R3C3-01) e ordinamento sui byte memorizzati con emissione in NFC (R3C3-02) | chiave canonica a sei livelli (§5.4.2) e normalizzazione prima del confronto (§4.6.3), entrambe con controesempio riprodotto | il rischio ricompare ovunque si aggiunga una collezione ordinata per testo: la regola è stata scritta una volta in §4.6.3 e richiamata, non ripetuta a mano in ogni punto |
| **scartare una decisione valida** credendola una ripetizione | la revisione 2 confondeva uguaglianza della chiave e identità dei dati (C1); la revisione 3 scriveva la codifica esatta riusando produzioni che perdono informazione (C4, C5, C2) | due definizioni separate, collasso solo sull'identità esatta, e produzioni **proprie** `str_exato` e `dado_esatto_string` (§5.4.2); cinque controesempi riprodotti (**[SONDA]** X15) | il difetto è di una classe che **ritorna** ogni volta che si aggiunge un campo dichiarato senza aggiungerlo alla codifica. F3C-46 lo scrive come obbligo, T3-22 lo verifica sui casi noti: resta un limite di copertura dichiarato in §9.2 |

### 9.2 Limiti di questa specifica

| Limite | Stato |
|---|---|
| nessun test Rust esiste | **dichiarato**: tutti i T3-* sono pianificati |
| le sonde non implementano l'impronta | **dichiarato** (§1.3): fissano i byte, non il codice |
| `AlvoAmbiguo` sembra irraggiungibile | **[NON_VERIFICATO]**: il controllo resta e viene testato con un bersaglio costruito a mano (T3-14) |
| nessuna persistenza | 3C.4; una dimostrazione in memoria non è uno storico (§5.6) |
| `autor` non è autenticato | **dichiarato** (§3.5): attribuisce, non prova |
| il testo degli avvisi entra nell'impronta | **limite dichiarato** (§4.5.2): cambiarlo richiede `v2` |
| `articoli_attesi` resta `NAO_DISPONIVEL` | invariato dalla 3B; la copertura misura l'assenza di avvisi di scarto |
| capacità numerica, `ModeloIncoerente` irraggiungibile, assenza di validazione XSD | limiti già dichiarati delle fasi precedenti, non toccati |
| nessuna produzione contabile | invariato: l'estratto è una vista in memoria |
| diagnostica senza indice d'ingresso | **costo dichiarato** della correzione R3C3-01: gli errori nominano l'evento con la sua `ChaveEvento` e non con la sua posizione. Chi ha in mano l'array deve ritrovare l'evento per contenuto. È il prezzo della riproducibilità del fallimento, e va messo in conto quando si scriveranno i messaggi |
| la regola di normalizzazione dipendeva da D8.4 | **risolto**: D8.4(a) approvata il 2026-10-21, §4.6.3 è il contratto da implementare. Resta registrato che con l'opzione (b) quella sezione sarebbe andata riscritta |
| le sonde X12, X13 e X15 sono trascrizioni delle regole | **dichiarato**: dimostrano che le regole vecchie erano difettose e che le nuove reggono sui controesempi, non che il futuro codice Rust le applicherà. Lo dovranno dimostrare T3-13, T3-17, T3-20, T3-21 e T3-22 |
| copertura dell'iniettività della codifica esatta | **parziale e dichiarata**: T3-22 verifica i campi dei cinque controesempi, non tutte le combinazioni di tutti i campi. L'obbligo di estendere la codifica a ogni campo nuovo è scritto in F3C-46, ma nessun test può dimostrare che un campo futuro non venga dimenticato: serve la disciplina, e per questo la regola è un contratto e non una nota |

---

## 10. Stato Git e destino dei file

Controllato prima della consegna:

```text
git status -sb                   ## main...origin/main
                                 ?? MANDATO_CLAUDE_FASE3C3_REVISIONE_SPECIFICA_20261017.md
                                 ?? SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md
git diff --name-status           (vuoto)
git diff --cached --name-status  (vuoto)
```

| File | Origine / mandato | Stato | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3C3_REVISIONE_SPECIFICA_20261017.md` | preparazione Codex su richiesta di Luca | mandato, continuità nominata | **registrato** con il commit documentale autorizzato il 2026-10-21 |
| `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` | esecuzione di questo mandato | audit tecnico superato; dieci sottoscelte approvate (§12) | **registrato** con lo stesso commit |

Nessun altro file del repository è stato modificato: nessun tracked risulta
modificato o staged, e i quattro SHA di §1.1 lo confermano. In particolare
**nessuna riga di codice**: il commit che registra questo documento non cambia
il comportamento del programma.

Script e output delle sonde vivono **fuori** dal repository, nella cartella
temporanea citata in §1.3: sono output di lavoro, non file di progetto, e non
vanno versionati.

**Audit della specifica superato e decisioni registrate. La fase 3C.3 non è
implementata e non è chiusa come fase software:** `impressao.rs` e `revisao.rs`
non esistono, e i ventidue test T3-* restano pianificati.

---

## 11. Correzione dei riscontri degli audit del 2026-10-21

Tre audit nello stesso giorno, ognuno sulla revisione precedente:

| Audit | Riscontri | Esito |
|---|---|---|
| sulla revisione 1 | `R3C3-01`, `R3C3-02`, rettifiche D9.2 e vettore F | corretti nella revisione 2 |
| mirato sulla revisione 2 | residuo di `R3C3-01`: la chiave dichiarata totale non lo era (`C1`, `C2`, `C3`) | corretto nella revisione 3 |
| mirato sulla revisione 3 | residuo di `R3C3-01`: la codifica esatta non era completa (`C4`, e per la stessa causa `C5` e `C2`) | corretto nella revisione 4 |

Un filo comune, che vale dichiarare: i tre riscontri appartengono alla **stessa
famiglia** — un'uscita che dipende dall'ingresso o da dati persi nella codifica
— e ogni correzione parziale ha lasciato emergere lo strato successivo. Per
questo la revisione 4 non si limita al caso segnalato: estende la correzione a
**tutte** le produzioni del testo esatto e trasforma l'iniettività in un obbligo
contrattuale verificabile (F3C-46) invece di un'asserzione.

Perimetro delle revisioni 2, 3 e 4: **solo questo documento**. Nessuna riga di
Rust, nessuna modifica alla specifica 3C generale, nessun cambio a D10/D13,
nessuna trasformazione di D8/D9 in approvazioni acquisite, nessuna riapertura di
R3C3-02, D9.2 o del vettore F.

### R3C3-01 — lo spareggio per indice contraddiceva la vista indipendente dall'ordine

**Accolto. Il riscontro era giusto e il difetto era mio.** La revisione 1
ordinava per `(instante, sequencia, indice)` e nella riga successiva prometteva
che permutare eventi compatibili non cambia la vista: con due `Anotar` allo
stesso istante e senza sequenza, la traccia seguiva l'array.

| Intervento | Dove |
|---|---|
| controesempio E1 riprodotto, con i due hash della traccia divergente e l'hash unico dopo la correzione | §5.4.1, sonda X13 in §1.3 |
| chiave canonica totale: istante assoluto, sequenze dichiarate prima dell'assenza, contenuto dichiarato in NFC. **Nessun indice** | §5.4.2 |
| dimostrazione che la chiave è **totale** dopo il collasso delle ripetizioni, quindi nessuno spareggio residuo serve | §5.4.2 |
| ordine di applicazione **e** ordine della traccia restituita sono lo stesso ordine canonico | §5.4.2 |
| caso con **una sola** sequenza dichiarata e operazioni incompatibili: conflitto, come chiesto dall'audit | §5.4.3, ultima riga |
| indici rimossi dall'uscita: `DecisaoAplicada` ed `EventoNaoAplicado` portano l'evento; gli errori nominano una `ChaveEvento` | §5.2 |
| passata 0 di ordinamento **prima** della validazione, così anche il fallimento è riproducibile | §5.3 |
| T3-17 riscritto: due annotazioni allo **stesso istante**, entrambe **senza sequenza**, confrontando **l'intera** vista; T3-21 nuovo per l'errore scelto in ordine canonico | §7 |
| contratto nuovo F3C-44 | §6.2 |

### R3C3-02 — normalizzazione e ordinamento non erano composti in modo canonico

**Accolto.** D8.4(a) prometteva invarianza alla forma Unicode, ma la tabella
delle collezioni diceva solo "ordinati per `(codigo, detalhe)`" senza dire su
quale rappresentazione: ordinare i byte memorizzati e poi emettere in NFC rompe
l'invariante.

| Intervento | Dove |
|---|---|
| controesempio N1 riprodotto: `6151e963…` contro `4bd43b99…` con la regola ingenua, `6151e963…` in entrambi i casi con le chiavi normalizzate | §4.6.3, sonda X12 in §1.3 |
| regola canonica in tre passi: normalizza le chiavi, confronta **ordinalmente sui byte UTF-8**, emetti in NFC con il prefisso calcolato dopo la normalizzazione | §4.6.3 |
| pareggi: entrambi gli avvisi conservati, nessuno scarto implicito, e dimostrazione che il pareggio non può cambiare l'impronta perché i byte emessi coincidono | §4.6.3 |
| disciplina estesa a tutti i contenitori ordinati per testo: `avisos_leitura`, conti della presentazione, traccia degli eventi | §4.6.3, §4.7, §5.4.2 |
| §4.5.2 e F3C-37 allineati: l'invarianza alla forma Unicode è **condizionata** a D8.4(a), e le due tabelle "invariante a" / "non invariante a" lo dicono esplicitamente | §4.5.2 |
| T3-13 esteso con il caso a **due** avvisi, l'unico che scopre il difetto; T3-20 nuovo per il pareggio canonico | §7 |
| contratto nuovo F3C-43 | §6.2 |

Nessuno dei due interventi sceglie D8.4 al posto di Luca: rendono coerente
l'opzione raccomandata con i suoi effetti dichiarati, e §9.2 dice cosa cambia se
la scelta fosse (b).

### Rettifica D9.2 — l'alternativa che rifiuta l'approvazione era descritta male

**Accolto.** La revisione 1 motivava la raccomandazione (a) dicendo che (b)
impedirebbe di annotare "visto, so che è rotto": falso, e smentito dalla stessa
matrice O7, che ammette `Anotar` in qualunque stato. Corretto in §3.7 e §8.1: la
differenza reale fra le due opzioni è la **forma** della traccia, stato
tipizzato contro testo libero, non la sua esistenza. La raccomandazione resta
(a), con la motivazione giusta. D10(a) resta intatta in entrambe.

### Rettifica del vettore F — lunghezza, non impronta

**Accolto.** `DEC:79228162514264337593543950335:0` occupa **35** byte, non 33:
`DEC:` più 29 cifre più `:0`. L'impronta pubblicata era corretta. Corretta la
tabella in §4.8 e ricontrollate tutte le lunghezze A–H una per una (sonda X14).

### Due errori miei, nelle sonde

| Errore | Effetto possibile | Correzione |
|---|---|---|
| la prima versione di `sonda_r3c3.ps1` usava un comparatore inerte e **non** riproduceva N1 | far concludere che il riscontro dell'audit fosse infondato | comparatore sui byte UTF-8 con **controprova** prima dell'uso |
| la prima esecuzione di `sonda_totalita.ps1` stampava un verdetto `False` per una precedenza sbagliata, mentre i due hash della traccia nella stessa riga erano identici | far credere a una dipendenza dall'ordine che non c'era | espressione corretta, sonda rieseguita, log sostituito |

In entrambi i casi l'errore era **nella sonda**, non nel riscontro né nella
regola. Nessun risultato delle versioni difettose è usato come prova. Un'assenza
di difetto misurata con uno strumento non validato non è una prova, e per questo
ogni sonda ora verifica il proprio comparatore prima di usarlo (§1.3).


### Residuo di R3C3-01, rilevato sulla revisione 2 — accolto

L'audit mirato ha mostrato che la **dimostrazione di totalità** aggiunta nella
revisione 2 era falsa: la chiave non comprendeva il bersaglio (C1) e
normalizzava o appiattiva dati che il record conserva (C2 autore in NFC/NFD, C3
offset dichiarati diversi). Tre collisioni riprodotte (**[SONDA]** X15).

| Intervento | Dove |
|---|---|
| le due nozioni separate: **identità esatta** per il collasso, **ordine canonico** per la traccia. Uguaglianza della chiave normalizzata ≠ identità dei dati dichiarati | §5.4.2 (1) e (2) |
| `texto_exato_evento`: codifica **iniettiva**, senza normalizzazione, con l'istante e il suo offset dichiarato e prefissi di lunghezza; grammatica scritta | §5.4.2 (1) |
| **bersaglio completo** nell'ordine, livello 3, con `ordinal` confrontato come numero: è il livello la cui assenza causava C1 | §5.4.2 (2) |
| **testo esatto** come livello 6: rende l'ordine totale sugli eventi superstiti, in un passo e senza indici | §5.4.2 (2) |
| ordine **globale** dichiarato cronologico; `decisoes`, `anotacoes` ed `eventos_nao_aplicados` sono sottosequenze della traccia globale, non una seconda regola | §5.4.2 |
| collasso **solo** per identità esatta; nessuna deduplicazione semantica, che sarebbe una decisione nuova da sottoporre a Luca | §5.4.2, §5.4.3, F3C-45 |
| C1, C2, C3 come casi espliciti; conflitto valutato **per bersaglio** | §5.4.3 |
| passata 0: chiave completa, collasso per identità esatta | §5.3 |
| `texto_exato_evento` e `chave_canonica` nell'API; `repeticoes` conta solo le ripetizioni esatte; `ChaveEvento` dichiarata **non** unica, con la determinatezza che viene dall'ordine canonico | §5.2 |
| T3-22 nuovo sui tre controesempi nelle due permutazioni, con il controllo d'iniettività; T3-05 e T3-14 allineati a `ChaveEvento` | §7 |
| contratti F3C-44 riscritto e F3C-45 nuovo | §6.2 |

Quello che la revisione 2 aveva fatto bene e resta: eliminazione degli indici
dall'uscita, scelta canonica dell'errore, passata 0 prima della validazione, e
la chiusura del controesempio E1.

### Residuo di R3C3-01, rilevato sulla revisione 3 — accolto

L'audit mirato ha mostrato che la grammatica del `texto_exato_evento`
**riusava** le produzioni dell'impronta tecnica: `dado`, che esclude il
`motivo`, e `str`, che normalizza in NFC. La prosa diceva "testi originali", la
grammatica diceva il contrario, e vince la grammatica. Riprodotto C4 e, con la
stessa causa, C5 (nota) e C2 (autore): tre collisioni su cinque casi
(**[SONDA]** X15).

| Intervento | Dove |
|---|---|
| due produzioni **nuove**, usate solo per l'identità dell'evento: `str_exato` (byte originali, nessun NFC) e `dado_esatto_string` (stato, fonte, presenza e valore, **presenza e motivo**, con tag di presenza e lunghezze) | §5.4.2 (1) |
| grammatica del testo esatto riscritta su quelle produzioni, rinominata `EVENTO-EXATO/v1` | §5.4.2 (1) |
| verificato nel codice che i due `Dado` di C4 sono **entrambi validi**: con valore presente il `motivo` può essere `None` o `Some` non vuoto (`modelos.rs:146-157`) | §5.4.2 (1) |
| iniettività motivata campo per campo invece che asserita, con l'obbligo di estendere la codifica a ogni campo futuro | §5.4.2 (1), F3C-46 |
| dichiarato esplicitamente che `str` e `dado` di §4.4 e l'impronta tecnica **non** cambiano: il `motivo` non entra nei byte del contenuto contabile | §5.4.2 (1) |
| C4 e C5 fra i casi di §5.4.3; T3-22 esteso ai cinque casi, con il controllo che l'impronta tecnica resti invariata fra C4 e C5 | §5.4.3, §7 |
| contratto nuovo F3C-46; F3C-45 esteso ai cinque casi | §6.2 |
| sonda riscritta: `AIM` ed `EVI` modellati con i quattro dati tipizzati dei `Dado`, nota **non** normalizzata nel testo esatto, e la grammatica difettosa conservata accanto a quella corretta per mostrare la differenza | §1.3, sonda X15 |

### Stato dei riscontri

| Riscontro | Stato |
|---|---|
| R3C3-01, controesempio E1 (eventi compatibili allo stesso istante) | **CHIUSO** nella specifica dalla revisione 2: regola sostituita, controesempio riprodotto |
| R3C3-01, residuo sulla totalità della chiave (C1, C2, C3) | **CHIUSO** nella specifica dalla revisione 3: identità esatta e ordine canonico separati, bersaglio e testo esatto nella chiave, collasso ristretto |
| R3C3-01, residuo sulla completezza della codifica esatta (C4, C5) | **CHIUSO** nella specifica dalla revisione 4: produzioni proprie `str_exato` e `dado_esatto_string`, iniettività motivata e resa un obbligo contrattuale |
| R3C3-02 normalizzazione prima dell'ordinamento | **CHIUSO** nella specifica: regola aggiunta, incoerenza fra §4.5.2 e §4.6.1 risolta |
| rettifica D9.2 | **CHIUSA**: confronto corretto |
| rettifica vettore F | **CHIUSA**: 35 byte |

"Chiuso" qui significa **chiuso nel documento**: nessuna di queste regole è
implementata, e tutti i test restano pianificati. Le baseline della 3C.2 non
sono state rieseguite perché questa revisione tocca solo Markdown e i quattro
SHA dei sorgenti sono invariati.

---

## 12. Decisioni approvate il 2026-10-21

Luca ha approvato **tutte e dieci** le sottoscelte nell'opzione **(a)**
raccomandata, dopo il superamento dell'audit tecnico della revisione 4.

| ID | Decisione approvata | Dove è definita | Contratto che la applicherà |
|---|---|---|---|
| D8.1 | la decisione umana si lega al **contenuto esaminato**: `ChaveRevisao` senza `lote`, corrispondenza unica, conflitto `AlvoAmbiguo` dichiarato | §4.1, §4.2 | F3C-13, F3C-44 |
| D8.2 | insieme dei campi dell'impronta di contenuto: 18 inclusi, 5 omessi perché già confrontati da `verificar_coerencia`, 3 esclusi (`arquivo_origem`, `hash_registro`, `observacoes`) | §4.5 | F3C-34, F3C-37 |
| D8.3 | **due** impronte distinte, contenuto e presentazione: un rinomino nel piano segnala `APRESENTACAO_MUDADA` e **non** invalida le approvazioni | §4.7 | F3C-38 |
| D8.4 | forma Unicode **NFC** per calcolare le impronte e **prima** di ogni ordinamento canonico; le stringhe memorizzate restano intatte | §4.6.1, §4.6.3 | F3C-43, F3C-37 |
| D8.5 | `VersaoDocumento::sequencia` è un `Dado` **`NAO_DISPONIVEL`** con fonte dichiarata finché non esiste l'archivio: nessun numero inventato | §4.9 | F3C-40 |
| D9.1 | approvare significa **"visto e accettato"** su una versione precisa: non cancella avvisi, non cambia gli esiti tecnici, non si trasferisce a una nuova versione | §3.1, §3.2 | F3C-04, F3C-05, F3C-28, F3C-29, F3C-30 |
| D9.2 | su un documento `ERRO` l'approvazione **si registra** e **non** entra mai nell'estratto, con esclusione motivata e avviso `APROVACAO_EM_DOCUMENTO_ERRO`. D10(a) resta intatta | §3.7 | F3C-11 (già), F3C-30 |
| D9.3 | la revoca riporta la riga allo **stato tecnico** calcolato dalla 3B, con avviso `APROVACAO_REVOGADA`; l'approvazione revocata **resta** in traccia | §3.2 O5 | F3C-31 |
| D9.4 | un `APROVADO` senza evento di provenienza è `APROVADO_SEM_PROVENIENCIA`: dato originale conservato, fuori dall'estratto revisionato, sanabile con un evento esplicito | §3.6 | F3C-42 |
| D9.5 | evento **malformato** = errore e nessuna vista; evento **non applicabile** = elencato e consultabile | §3.4 | F3C-32, F3C-33, F3C-41 |

### Che cosa questa approvazione è, e che cosa non è

| È | Non è |
|---|---|
| la chiusura delle **decisioni funzionali** aperte su D8 e D9 | l'approvazione di codice: `impressao.rs` e `revisao.rs` non esistono |
| il mandato di contenuto per il lotto 3C.3.a | un collaudo: i ventidue test T3-* sono **pianificati**, nessuno è stato eseguito |
| una base stabile su cui scrivere i contratti F3C-28…F3C-46 | la promozione di quei contratti da `PROPOSTO` a `IMPLEMENTATO` |
| una decisione che **raffina** dichiaratamente la §4.1 della specifica 3C generale (D9.3) | un'autorizzazione a riscrivere la specifica 3C generale, che resta come è |

### Decisioni che restano fuori

D7 (fonte autorevole), D11 (import dello storico) e D12 (importi non
interpretabili) **restano rinviate**, e §8.3 dimostra che nessuna di esse
blocca il lotto 3C.3.a. D10(a) e D13(a) restano approvate e non riaperte. Una
deduplicazione semantica degli eventi (§5.4.2) **non** è stata chiesta né
approvata: sarebbe una decisione nuova.

### Prossimo passo, non autorizzato da questo documento

Preparare il mandato implementativo del lotto **3C.3.a** nel perimetro di §5,
tenendo fuori persistenza, archivio, esportazione, import, CLI e fasi
successive. L'implementazione, i suoi commit e il suo push restano da
autorizzare separatamente.

---

## 13. Esito dell'implementazione del lotto 3C.3.a (2026-10-21)

Appendice di **esecuzione**: non cambia nessuna decisione, nessun vettore e
nessun contratto di questo documento. Le decisioni di §12 restano quelle, e i
contratti restano `PROPOSTO` fino all'audit di questo lotto.

### 13.1 File scritti

| File | Intervento | Righe |
|---|---|---|
| `src/impressao.rs` | **nuovo**: testi canonici e impronte di contenuto e presentazione | 1.111 |
| `src/revisao.rs` | **nuovo**: eventi, codifica esatta, ordine canonico, validazione, proiezione, vista | 2.859 |
| `src/lib.rs` | due righe: `pub mod impressao;`, `pub mod revisao;` | +2 |
| `README.md` | stato reale, contratti coperti, superficie pubblica, limiti, conteggio dei test | — |

**Non toccati**, come da mandato: `prelivro.rs`, `alocacao.rs`, `dinheiro.rs`,
`modelos.rs`, `nfe.rs`, `regras.rs`, `main.rs`, `classificacao.rs`,
`normalizacao.rs`, i test preesistenti, `Cargo.toml`, `Cargo.lock`, `config/*`,
`samples/*`. Verificato con `git status --porcelain` su ciascuno: nessuna
modifica. Nessun file di integrazione sotto `tests/` e' stato aggiunto: i test
interni ai due moduli coprono i contratti attraverso le funzioni pubbliche, e
un file in piu' non avrebbe verificato niente di nuovo.

### 13.2 Scelte tecniche, dentro i contratti approvati

Nessuna di queste cambia una regola: sono modi di realizzarla.

| Scelta | Perche' |
|---|---|
| `ChaveCanonica` e' una **struct** con `Ord` derivato, non una stringa unica | il confronto resta ordinale sui byte UTF-8, perche' le `String` di Rust si confrontano cosi', ma diventa verificabile campo per campo, e `Option<String>` da' gia' "assente prima di presente". La specifica dichiarava la firma come indicativa |
| `impressao::dado` serve il livello **semantico** della chiave; `dado_esatto_string` serve il livello 6 | una sola implementazione per ciascuna delle due produzioni, nessun codec duplicato |
| `aplicar_revisao` chiama `construir_pre_livro` **sugli originali** e poi, solo se una decisione cambia uno stato, **sulla copia proiettata** | la prima chiamata e' anche la validazione dell'ingresso: un abbinamento non puo' rendere valido cio' che la 3C.2 rifiuta. Senza decisioni applicate la vista riusa il pre-libro originale, quindi "nessuna decisione" non costa una seconda costruzione |
| gli errori portano `Box<ChaveEvento>` | tiene piccolo l'enum d'errore: senza il box `clippy::result_large_err` segnalava un `Result` sbilanciato. La diagnostica resta identica |
| `instante_exato` usa `to_rfc3339_opts(Nanos, false)` | larghezza fissa e offset dichiarato conservato: nessun ramo che dipenda dal valore |
| la traccia globale si ottiene ordinando le decisioni di riga con la stessa chiave | una sola regola d'ordine, come dichiarato in §5.4.2: le collezioni di riga sono sottosequenze, non una seconda regola |

### 13.3 Mappa dei test pianificati

59 test nuovi: **17** in `impressao.rs`, **42** in `revisao.rs`, dei quali sei
sono le regressioni dei riscontri dell'audit del lotto (§13.6). Dove un test
copre piu' casi della matrice, e' nominato una volta sola.

| Caso di §7 | Test |
|---|---|
| T3-01 | `t3_01_aprovacao_explicita_de_linha_ok`, `t3_01_sem_eventos_nenhuma_linha_e_aprovada` |
| T3-02 | `t3_02_aprovar_nao_apaga_avisos_nem_esitos` |
| T3-03 | `t3_03_projecao_muda_so_o_status_revisao` |
| T3-04 | `t3_04_revogacao_restaura_o_estado_tecnico_e_conserva_a_trilha`, `t3_04_revogacao_de_linha_revisar_volta_a_revisar` |
| T3-05 | `t3_05_eventos_malformados_sao_erros_nomeados` (cinque casi) |
| T3-06 | `t3_06_alvo_ausente_nao_toca_nenhuma_linha`, `t3_06_revogar_sem_aprovacao_nao_se_aplica`, `t3_06_proveniencia_ausente_nao_e_revisao_verificada` |
| T3-07 | `t3_07_duas_linhas_com_o_mesmo_hash_sao_alvos_distintos` |
| T3-08 | `t3_08_correcao_a_chave_e_total_invariados_muda_a_impressao` (vettore J), `t3_08_versao_superada_nao_herda_a_decisao` |
| T3-09 | `t3_09_apresentacao_separa_nome_do_plano_do_conteudo` (vettori K/L), `t3_09_texto_de_apresentacao_e_estavel`, `apresentacao_mudada_e_sinalizada_e_as_decisoes_permanecem` |
| T3-10 | `t3_10_mesmo_conteudo_em_outro_lote_reencontra_o_alvo` |
| T3-11 | `t3_11_permutacao_dos_documentos_e_indiferente`, `t3_25_permutacao_dos_avisos_de_leitura_e_indiferente` |
| T3-11b | `t3_11b_permutacao_dos_artigos_e_um_caso_distinto` |
| T3-12 | `t3_12_forma_canonica_dos_monetarios`, `t3_12_impressoes_dos_vetores_a_f`, `t3_12_vetor_i_byte_a_byte_e_impressao` |
| T3-12b | `t3_12b_ausencia_difere_de_zero_real`, `t3_12b_prefixo_de_comprimento_separa_campos`, `t3_12b_separadores_e_lf_nos_textos_nao_quebram_a_codificacao` |
| T3-13 | `t3_13_impressao_invariante_a_ambiente_lote_e_campos_humanos`, `t3_13_impressao_invariante_a_forma_unicode`, `t3_13_n1_normalizar_antes_de_ordenar` |
| T3-14 | `t3_14_repeticao_exata_nao_e_aplicada_duas_vezes`, `t3_14_decisoes_incompativeis_no_mesmo_instante_sao_conflito` (quattro combinazioni), `t3_14_eventos_contemporaneos_em_linhas_diferentes_nao_sao_conflito` |
| T3-15 | `t3_15_anotacao_conservada_e_aprovacao_nao_muda_a_impressao` |
| T3-16 | `t3_16_documento_sem_linhas_e_aprovado_sem_proveniencia`, `t3_16_sanatoria_explicita_de_aprovado_sem_proveniencia` |
| T3-17 | `t3_17_duas_anotacoes_no_mesmo_instante_sem_sequencia` |
| T3-18 | `t3_18_um_evento_malformado_impede_toda_a_vista` |
| T3-19 | `t3_19_aprovacao_em_documento_erro_e_registrada_mas_nao_entra` |
| T3-20 | `t3_20_empate_canonico_conserva_os_dois_avisos` |
| T3-21 | `t3_21_erro_escolhido_em_ordem_canonica_nao_por_posicao` |
| T3-22 | `t3_22_c1_alvos_distintos_nao_sao_repeticao`, `t3_22_c2_c3_c4_c5_sao_declaracoes_distintas`, `t3_22_codificacao_exata_e_injetiva_nos_campos_declarados` (dodici campi), `texto_exato_nao_normaliza_e_conserva_o_offset` |
| R3C3A-01 | `r1_revogacao_da_sanatoria_tira_a_linha_do_extrato_revisado`, `r1_revogacao_de_linha_ok_continua_fora_do_extrato` |
| R3C3A-02 | `r2_offset_declarado_com_segundos_nao_e_repeticao`, `r2_nanossegundos_e_repeticao_exata_continuam_corretos` |
| R3C3A-03 | `r3_escolha_do_conflito_nao_depende_da_ordem_dos_documentos`, `r3_conflito_estavel_permutando_eventos_e_documentos` |
| aggiunti | `etiquetas_dos_enums_sao_estaveis`, `contas_mostradas_vem_do_plano_e_sao_ordenadas`, `conta_fora_do_plano_e_erro_nomeado`, `avisos_da_vista_ficam_em_ordem_canonica`, `aprovacao_repetida_com_declaracoes_diferentes_avisa_e_conserva`, `erro_do_pre_livro_e_propagado_sem_reescrita`, `conta_fora_do_plano_e_rejeitada`, `lote_vazio_produz_vista_vazia_sem_inventar_identidade`, `alvo_ambiguo_e_interceptado_antes_pela_validacao_da_3c2` |

I vettori normativi vivono in attese **scritte a mano**, copiate da §4.8, non
rigenerate dal codice sotto test. Il testo del vettore I e' un letterale con
`\n` espliciti: un checkout con `CRLF` non lo altera, e i test non dipendono da
Git ne' dal sistema operativo a runtime.

### 13.4 Verifiche eseguite

| Comando | Esito |
|---|---|
| `cargo test --locked` | **267 test passati**: 251 di unita' nella lib, 3 nel binario, 13 di integrazione CLI. I 208 preesistenti restano verdi **senza modifiche** |
| `cargo fmt --check` | pulito |
| `cargo clippy --locked --all-targets -- -D warnings` | pulito, **nessuna soppressione**: `unnecessary_sort_by`, `collapsible_if`, `ptr_arg`, `cloned_ref_to_slice_refs` e `result_large_err` sono stati corretti nel codice |
| `git diff --check` | pulito |
| sei fonti canoniche | **6/6 invariate** (`git status --porcelain config samples` vuoto) |
| file esclusi dal mandato | nessuna modifica, verificato uno per uno |

I vettori sono ora riprodotti dal **codice Rust**, non solo dalla sonda: il
vettore I da' **948 byte** e `f8a6e8d0…14c`, J **952 byte** e `4ce5f31b…7a7`, K
e L le impronte di presentazione pubblicate, A–H i frammenti con le loro
lunghezze. La grammatica di §4.4 si e' rivelata implementabile **cosi' com'era
scritta**: nessuna sua regola e' stata cambiata per far tornare un hash.

### 13.5 Differenze rispetto al mandato, e limiti residui

| Punto | Stato |
|---|---|
| `AlvoAmbiguo` | **non raggiungibile** dall'API: due righe con la stessa `(chave, impressao, origem, conta, ordinal)` richiederebbero due documenti con la stessa chiave, che `construir_pre_livro` rifiuta prima con `DocumentoDuplicadoNoLote`. Il ramo resta nel tipo, e il test `alvo_ambiguo_e_interceptado_antes_pela_validacao_da_3c2` registra **quale** controllo intercetta il caso, invece di forzare un tipo incoerente. Limite dichiarato, come fu per `ModeloIncoerente` in 3B |
| copertura dell'iniettivita' | `t3_22_codificacao_exata_e_injetiva_nos_campos_declarados` varia **dodici** campi dichiarati uno per uno, non tutte le combinazioni: la garanzia per i campi **futuri** resta l'obbligo contrattuale F3C-46, non un test |
| persistenza | assente per contratto: e' 3C.3.b e dipende da D7 |
| `APRESENTACAO_MUDADA` | si attiva confrontando la `vista` **dichiarata nell'evento** con l'impronta di presentazione corrente. Un evento che non dichiara la vista (`Dado` assente) non produce nessuna affermazione: assenza dichiarata, non silenzio |
| nessun `f32`/`f64`, nessuna nuova aritmetica monetaria, nessun orologio o IO nei moduli puri | verificato: i due moduli non importano `std::fs`, `std::time` ne' `dinheiro` |

**La fase 3C.3 non e' chiusa**: questo e' il lotto `3C.3.a`, consegnato per
audit, e la 3C nel suo insieme richiede ancora 3C.4 e 3C.5. Nessuna promozione
in produzione contabile, nessun commit e nessun push.

### 13.6 Correzione dei riscontri dell'audit del lotto (2026-10-21)

L'audit indipendente del lotto ha trovato **tre** difetti, tutti riprodotti
attraverso l'API pubblica e tutti in combinazioni che i 53 test della prima
consegna non coprivano. Accolti e corretti. Le sei regressioni sono state
scritte **prima** della correzione, con le attese corrette: sono fallite, poi
sono passate.

#### R3C3A-01 — la revoca poteva lasciare la riga nell'estratto revisionato

Il filtro escludeva soltanto lo stato corrente `APROVADO_SEM_PROVENIENCIA`.
Partendo da una riga che arriva `APROVADO` senza evento — ingresso ammesso da
D9.4 — dopo la sanatoria esplicita e la sua **revoca** lo stato diventa
`APROVACAO_REVOGADA`, quindi non era piu' quello escluso; e la proiezione
ripristina lo stato tecnico, che in quel caso e' proprio `APROVADO`. Risultato:
una riga nell'estratto revisionato **senza nessuna approvazione attiva**.

| Intervento | Dove |
|---|---|
| l'inclusione richiede ora un'**approvazione umana attiva**: `estado == AprovadoPorPessoa`, oltre al filtro tecnico D10(a) gia' applicato dalla 3C.2 | `revisao.rs`, assemblaggio della vista |
| l'esclusione porta un motivo **distinto per stato**: provenienza assente contro approvazione revocata | idem |
| dato originale, stato tecnico, decisioni e riconciliazione restano invariati; nessun `OK` o `REVISAR` inventato come stato della fonte | invariato |
| la riga **rientra** se l'approvazione torna attiva: testato con il ri-`Aprovar` | `r1_revogacao_da_sanatoria_tira_a_linha_do_extrato_revisado` |

Regressioni: `r1_revogacao_da_sanatoria_tira_a_linha_do_extrato_revisado` (tre
fasi: senza eventi, sanatoria+revoca, ri-approvazione) e
`r1_revogacao_de_linha_ok_continua_fora_do_extrato` come controllo ordinario.

#### R3C3A-02 — il testo esatto perdeva i secondi dell'offset

`EventoRevisao::instante` e' un `DateTime<FixedOffset>`, e `FixedOffset`
ammette offset con i **secondi**. `to_rfc3339_opts(SecondsFormat::Nanos, false)`
rappresenta l'offset ai **minuti**: `Nanos` riguarda la frazione dell'orario,
non la precisione dell'offset. Due approvazioni con offset dichiarati di 1 e 2
secondi davano lo stesso testo esatto — entrambe `+00:00` — e la seconda
diventava una falsa `Repeticao`: una dichiarazione reale perduta. Non era una
collisione SHA-256, era una codifica **non iniettiva** di un valore che il tipo
pubblico rappresenta.

**Completamento tecnico della grammatica** (non una nuova decisione): la
produzione `EIN` di `EVENTO-EXATO/v1` scrive l'orario **locale** con i
nanosecondi, poi il separatore `OFF`, poi l'offset come **numero intero di
secondi** rispetto a UTC:

```text
EIN=38:2026-10-21T12:00:00.000000000OFF-10800
```

Orario locale piu' offset in secondi determinano univocamente il
`DateTime<FixedOffset>`, quindi la codifica resta iniettiva su tutto il dominio
del tipo. Nessun filtro di dominio e' stato introdotto: rifiutare gli offset
con secondi per evitare il controesempio sarebbe stato un vincolo non
approvato. L'impronta **tecnica del contenuto** e i dati degli eventi restano
invariati: `EIN` vive solo nella codifica esatta dell'evento.

Regressioni: `r2_offset_declarado_com_segundos_nao_e_repeticao` e
`r2_nanossegundos_e_repeticao_exata_continuam_corretos`. Il test
`texto_exato_nao_normaliza_e_conserva_o_offset` e' stato aggiornato alla forma
corretta: la sua vecchia attesa (`-03:00`) descriveva il difetto.

#### R3C3A-03 — la scelta del conflitto dipendeva dall'ordine dei documenti

Il controllo attraversava `estados.values()`, ordinato per `PosicaoLinha`, che
contiene l'**indice del documento ricevuto**, e restituiva il primo conflitto
trovato. Con due documenti, ciascuno con un proprio conflitto, permutare
l'array dei documenti cambiava quale conflitto veniva nominato — esattamente
dove F3C-44 esclude la posizione d'ingresso. Le singole coppie erano
riconosciute correttamente e nessuna vista parziale veniva prodotta: il difetto
era la **selezione** del fallimento.

| Intervento | Dove |
|---|---|
| `verificar_conflitos` diventa `conflitos_do_alvo`: restituisce **tutte** le coppie in conflitto di un bersaglio, invece di fermarsi alla prima | `revisao.rs` |
| il chiamante raccoglie i conflitti di tutti i bersagli e li ordina con la **chiave canonica degli eventi**, poi riporta il primo | `revisao.rs`, passata 2b |
| nessuna modifica a `prelivro.rs` ne' ai contratti della 3C.2 | invariato |

Regressioni: `r3_escolha_do_conflito_nao_depende_da_ordem_dos_documentos`, che
verifica anche **quale** conflitto viene scelto — il piu' antico in ordine
canonico — e `r3_conflito_estavel_permutando_eventos_e_documentos`, che permuta
eventi e documenti insieme.

#### Verifiche dopo le correzioni

| Comando | Esito |
|---|---|
| `cargo test --locked` | **267 test passati**: 251 lib, 3 binario, 13 CLI. 59 nuovi (17 `impressao`, 42 `revisao`); i 208 preesistenti verdi senza modifiche |
| `cargo fmt --check` | pulito |
| `cargo clippy --locked --all-targets -- -D warnings` | pulito, nessuna soppressione |
| `git diff --check` | pulito |
| sei fonti canoniche | 6/6 invariate |
| file esclusi dal mandato | nessuna modifica |
| vettori normativi | invariati: I 948 byte e `f8a6e8d0…14c`, J 952 byte, K/L, A–H. Le correzioni non toccano l'impronta del contenuto contabile |

Nota sugli avvisi d'ambiente: `rustc` puo' segnalare il fallimento degli hard
link della cache incrementale, e Git segnala la futura conversione LF/CRLF dei
tracked modificati. I comandi terminano con esito 0 e nessun avviso e' stato
nascosto o silenziato: "pulito" qui significa **nessun rilievo di `fmt`,
`clippy` o `diff --check`**, non assenza di messaggi d'ambiente.

#### Stato dei riscontri del lotto

| Riscontro | Stato |
|---|---|
| R3C3A-01 estratto revisionato senza approvazione attiva | **CORRETTO**, due regressioni permanenti |
| R3C3A-02 offset con secondi perduto nella codifica esatta | **CORRETTO**, due regressioni, grammatica `EIN` completata e dichiarata qui |
| R3C3A-03 scelta del conflitto dipendente dall'ordine dei documenti | **CORRETTO**, due regressioni permanenti |

Il lotto resta **consegnato per audit**, non chiuso: la conferma delle
correzioni spetta all'audit successivo, e nessun commit o push e' autorizzato.

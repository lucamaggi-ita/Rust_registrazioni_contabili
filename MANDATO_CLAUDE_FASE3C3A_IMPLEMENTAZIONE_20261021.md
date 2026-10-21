# Mandato operativo a Claude — implementazione del lotto 3C.3.a

Data: 2026-10-21. Repository: Rust_registrazioni_contabili.
Workspace: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Ruoli: Claude implementa e verifica; Codex svolge l'audit indipendente; Luca autorizza registrazione e pubblicazione.
Metodo: `programmatore-bravo`. Prima contrattualizza, poi programma.

## 1. Consegna e autorizzazioni

Luca ha richiesto questo **MD implementativo per Claude** dopo la chiusura dell'audit della specifica e l'approvazione/pubblicazione delle dieci scelte D8/D9. Codex, nella preparazione del mandato, scrive soltanto questo documento.

**Quando Luca ti consegna questo mandato, implementa il lotto puro 3C.3.a**: impronte, eventi di revisione, applicazione validata e vista revisionata in memoria. Non ripetere l'analisi generale né chiedere di approvare nuovamente D8/D9. Completa codice, test e documentazione entro il perimetro sotto, poi consegna per audit Codex.

Non sono autorizzati commit, push, riscrittura della storia, persistenza, export, import, CLI, fasi successive o produzione. Le autorizzazioni già consumate dai precedenti commit non si estendono a questa implementazione. Le normali scelte tecniche entro i contratti approvati non richiedono una nuova decisione di Luca; una variazione funzionale va invece dichiarata prima di scrivere la regola diversa.

## 2. Baseline e fonte normativa

- HEAD verificato durante la preparazione: **`10f5b2df048c2e56c33e6561baedd65cf987130b`**, commit documentale della specifica e delle decisioni D8/D9. Worktree inizialmente pulito, nessuno staged.
- Baseline software: 3C.2 pubblicata nei commit `03e16ec` e `37c0e03`; **208 test passati** nell'audit finale 3C.2, 192 lib + 3 bin + 13 CLI. Evidenza riusata, non rieseguita per preparare questo MD.
- Fonte normativa: **`SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` a `10f5b2d`**, revisione 4 con decisioni registrate in §12. D8.1–D8.5 e D9.1–D9.5 sono tutte approvate nell'opzione (a).
- La matrice F3C e i test T3-* descrivono ancora l'implementazione da fare: non dichiararli esistenti o superati prima delle prove. Gli identificativi restano F3C; nessuna rinumerazione generale C25+ in questo lotto.
- D10(a), D13(a), C18–C24 e i contratti 3C.2 restano invariati. D7/D11/D12 restano rinviate. Nessuna deduplicazione semantica degli eventi è approvata.

La specifica contiene fotografie storiche e passi inizialmente etichettati PROPOSTO: lo stato delle decisioni è quello di §12, mentre il comportamento dettagliato è definito nelle sezioni richiamate. Non confondere «decisione approvata» con «codice già approvato».

Lavoro MIRATO di implementazione su una specifica già auditata. Sono BLOCCANTE_ORA le violazioni di identità, conservazione, ordine e riconciliazione; i debiti estranei restano DA_PIANIFICARE. Se un requisito risultasse davvero non implementabile nel perimetro, dimostralo con un caso minimo e separalo dalle normali scelte di tipi/API.

## 3. File ammessi e confini

| File | Intervento ammesso |
|---|---|
| `src/impressao.rs` | nuovo modulo puro: testi canonici e impronte di contenuto/presentazione |
| `src/revisao.rs` | nuovo modulo puro: tipi di revisione, codifica esatta, ordine, validazione, proiezione e vista |
| `src/lib.rs` | esportazione dei due moduli; allineamento minimo della mappa delle fasi |
| `README.md` | stato reale 3C.3.a, API/limiti, contratti coperti e conteggio effettivo dei test |
| `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` | sola appendice di esecuzione: file, corrispondenza test/contratti, prove e limiti; preservare decisioni e vettori normativi |
| questo mandato | eventuale appendice breve di esito, senza riscrivere l'autorizzazione |

I test vanno preferibilmente nei due nuovi moduli. Un file di integrazione dedicato sotto `tests/` è ammesso soltanto se verifica un contratto attraverso l'API pubblica che i test interni non coprono adeguatamente; dichiararne nome e scopo. Fixture sintetiche piccole possono essere costruite nei test, con provenienza esplicita. Non modificare fixture esistenti o campioni canonici.

**Non toccare** `src/prelivro.rs`, `src/alocacao.rs`, `src/dinheiro.rs`, `src/modelos.rs`, `src/nfe.rs`, `src/regras.rs`, `src/main.rs`, gli altri sorgenti, i test preesistenti, `Cargo.toml`, `Cargo.lock`, `config/*`, `samples/*`, il repository Python o le specifiche/mandati storici degli altri lotti.

`sha2`, `chrono`, `unicode-normalization` e `serde` sono già disponibili. Nessuna dipendenza o installazione nuova. Non aggiungere `.gitattributes` per risolvere un problema dei vettori: il flusso canonico deve costruire LF indipendentemente dal checkout Windows.

## 4. Implementazione richiesta

### A. Impronta di contenuto e di presentazione

Attua §§4.4–4.9 della specifica: `IMPRESSAO-CONTEUDO/v1` e `IMPRESSAO-APRESENTACAO/v1`, campi inclusi/esclusi esatti, SHA-256 esadecimale minuscolo, LF finale. Esponi il testo canonico per i test dei byte oltre al solo hash.

- Stringhe dell'impronta in NFC; prefissi di lunghezza sui **byte UTF-8 dopo la normalizzazione**. Per gli ordinamenti testuali dell'impronta normalizza le chiavi **prima** del confronto ordinale UTF-8. Nessun confronto linguistico.
- Conserva molteplicità e ordine contrattuale delle collezioni; due avvisi equivalenti in NFC restano due elementi. Non riordinare registrazioni/duplicatas per comodità.
- `Decimal` normalizzato come `DEC:mantissa:scala`, zero canonico `DEC:0:0`, senza conversione universale in centesimi, perdita di precisione o passaggi float. Rispetta quantità/frazioni e massimi rappresentabili.
- La produzione tecnica `dado` include stato/fonte/valore ed esclude motivo secondo la specifica. Non confonderla con la codifica esatta degli eventi.
- Calcola le impronte sui **documenti originali**, prima della proiezione umana. Il piano corrente alimenta la presentazione; un suo rinomino senza riallocazione segnala `APRESENTACAO_MUDADA` e conserva le decisioni secondo D8.3.
- Proteggi la coerenza contesto/allocazione e gli errori del piano usando la 3C.2: non far diventare valido un ingresso che `construir_pre_livro` rifiuta.

### B. Identità esatta e ordine degli eventi

Attua §5.4.2 integralmente, compresa la correzione della revisione 4:

- `ChaveRevisao` senza lotto identifica contenuto e riga; `IdentidadeLinha` con lotto mantiene la provenienza dell'esecuzione. C23 non è identità univoca e non autorizza scarti.
- `EVENTO-EXATO/v1` usa esclusivamente `str_exato` e `dado_esatto_string`: byte originali, offset dichiarato, tag di presenza e tutti i dati dei Dado, incluso motivo. Nessuna NFC in questa codifica.
- Collasso soltanto su identità **esatta**. Non usare l'uguaglianza semantica/NFC né un'uguaglianza di `DateTime` che ignori la differenza dell'offset dichiarato come prova di evento identico. Ripetizioni esatte restano consultabili e contate.
- Ordine a sei livelli: istante assoluto, sequenza dichiarata prima dell'assenza, bersaglio completo, operazione, contenuto semantico NFC, testo esatto. Ordinali e sequenze si confrontano come numeri, non come stringhe non protette. Le firme della specifica sono indicative: una chiave strutturata/comparatore è una scelta tecnica ammessa se realizza esattamente l'ordine deciso.
- Un solo ordine globale; viste per riga e non applicazioni ne ereditano l'ordine. Nessun indice d'ingresso nello spareggio, nell'uscita o nella scelta dell'errore. La codifica deve coprire tutti i campi attuali di `EventoRevisao`; evitare due implementazioni divergenti del medesimo codec.

### C. Revisione pura e proiezione

Attua `aplicar_revisao(documentos, plano, eventos)` e i tipi/errori necessari, seguendo le passate di §5.3: ordine completo e ripetizioni, validazione, abbinamento/conflitti, proiezione e costruzione della vista. Input immutabili, nessuna vista parziale in errore.

- Autore, istante con offset e motivo vengono dal chiamante. Nessun default, utente di sistema, orologio o filesystem nella funzione. L'autore attribuisce una dichiarazione, non autentica una persona.
- `Aprovar`, `Revogar`, `Anotar` sono le sole operazioni. La proiezione per copia cambia **solo `status_revisao`** dei Lancamento; gli altri 25 campi e la riconciliazione restano intatti. `observacoes` originale non viene sovrascritto: le note vivono negli eventi.
- Conserva separati stato tecnico originale, decisione e stato della vista. Revoca = nuovo evento e ripristino dello stato tecnico originale, con la traccia precedente conservata (D9.3).
- Approvazione su documento ERRO registrata, avviso previsto e **nessun ingresso nell'estratto revisionato**. D10(a) resta intatta. Nessun avviso tecnico cancellato per approvare.
- APROVADO senza evento valido resta dato originale conservato ma non revisione verificata: `APROVADO_SEM_PROVENIENCIA`, esclusione motivata e sanatoria soltanto con evento esplicito.
- Evento malformato = errore nominato, nessuna vista; evento non applicabile = elencato e consultabile. Contenuto modificato non eredita decisioni; contenuto identico in un altro lotto ritrova il bersaglio univoco. Le sequenze storiche indisponibili restano Dado assenti, mai 0/1 inventati.
- Conflitti di approvazione/revoca sono per **stesso bersaglio e stessa versione**; applica le regole già definite su istanti e sequenze presenti/assenti. Eventi su righe diverse non sono un conflitto per il solo fatto di essere contemporanei.
- Ricostruisci riepiloghi/partizioni tramite `construir_pre_livro`, senza duplicare algoritmi monetari. Distingui l'estratto revisionato più severo dall'estratto della 3C.2; ogni differenza/esclusione deve essere interrogabile e motivata.

La vista e gli eventi sono in memoria. Nessuno storico persistente viene creato; la consultabilità della traccia non equivale alla conservazione su disco.

## 5. Test obbligatori e prove di accettazione

La matrice di riferimento è §7 della specifica: **T3-01…T3-22, incluse T3-11b e T3-12b**, con tutti i sottocasi. Non interpretare «22» come una quota di funzioni: possono essere più test e il conteggio finale deriva dall'esecuzione. Per ogni riga della matrice indica nella consegna il test che la verifica.

Oltre ai casi ordinari di approvazione/revoca, il minimo decisivo comprende:

| Gruppo | Prova richiesta |
|---|---|
| Byte e impronte | vettori A–L di §4.8, confronto del testo completo I e hash I/J; massimi Decimal, assenza/zero, separatori e LF nei testi |
| Ordinamento degli avvisi | N1 con due avvisi; pareggio NFC con entrambi gli avvisi conservati |
| Identità degli eventi | C1 bersagli distinti, C2 autore NFC/NFD, C3 offset diversi, C4 motivo Dado assente/presente, C5 nota NFC/NFD; nessuna falsa Repeticao |
| Permutazioni | intera vista/traccia uguale nei due ordini; E1 con due annotazioni allo stesso istante senza sequenza; errore canonico con due eventi malformati |
| Completezza del codec | copia identica = stesso testo; variazioni dei campi e dei metadati Dado = testi distinti; None diverso da Some; nessuna normalizzazione dei dati esatti |
| Conservazione | input immutabili, confronto dei 25 campi invariati, riconciliazione integra, eventi revocati/superati/ripetuti consultabili |
| Integrazione | documenti senza righe, payload C23 uguali su righe diverse, rigenerazione in altro lotto, correzione a totale invariato, piano rinominato, esclusioni ERRO/APROVADO senza provenienza |

I vettori devono vivere in attese di test stabili, non essere rigenerati dal codice sotto test per poi confrontarlo con sé stesso. I byte canonici I sono **948**, SHA-256 **`f8a6e8d0aade811e39e31f966c17e0c6623f21afdbde73a8c5866641031f614c`**; J è 952 byte; F è 35 byte. Il checkout può avere CRLF (`core.autocrlf=true`): normalizzare solo i delimitatori del blocco fixture, non alterare i dati per far tornare l'hash. Usare il blob Git come fonte di confronto; niente dipendenza dei test da Git o dal sistema operativo a runtime.

Se un ramo come `AlvoAmbiguo` non è raggiungibile attraverso l'API effettiva, verificare prima quale validazione lo intercetta. Non forzare tipi incoerenti o inventare un test che non raggiunge quel ramo: registrare la prova e l'eventuale limite, come già fatto nei lotti precedenti. La specifica propone quel test; una differenza di raggiungibilità va spiegata, non occultata.

Procedi prima con test mirati dei nuovi moduli, poi alla consegna esegui una volta:

```text
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
git diff --check
```

La suite finale deve includere i **208 preesistenti senza modifiche**. Controlla gli hash delle sei fonti e l'assenza di cambi ai file esclusi. Niente `f32`/`f64`, nuova aritmetica monetaria, clock/IO nei moduli puri o soppressioni generiche di warning. Se un criterio fallisce, correggi e riprova il flusso interessato; non dichiararlo soddisfatto con una riserva nascosta.

## 6. Prove precedenti riutilizzabili

Verbale di chiusura tecnica della specifica:

`C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\16\01a144b0-273a-7840-931d-94b0df795d2b\audit_specifica_3c3_r4_20261021\AUDIT_3C3_R4_ESITO_TECNICO_20261021.md`.

Nella stessa cartella: `probe_esatto.rs`, log, snapshot e manifesto delle sei fonti. Sono prove dei contratti, **non implementazione da copiare ciecamente**, né sostituti dei test Rust del nuovo codice. I vecchi script temporanei di Claude hanno avuto difetti dichiarati: preferire specifica pubblicata, vettori e tipi effettivi a un nome di file «sonda» considerato di per sé autorevole. Nessuna riesecuzione Python necessaria per questo lotto se non cambia la fonte Python.

## 7. Consegna, audit e stato Git

Alla consegna aggiorna il README allo stato **implementato, in attesa di audit**, con conteggio effettivo, superficie API e limiti. Aggiungi una breve appendice di esecuzione alla specifica senza cambiare retroattivamente le decisioni o promuovere i contratti prima delle prove.

Fornisci: file modificati, contratti implementati, mappa T3→test, comandi e risultati reali, differenze rispetto al mandato, limiti residui ed esiti separati per implementazione/verifica/audit/Git. Non dichiarare chiusa l'intera fase 3C né attestare produzione.

Esegui `git status -sb`, `git diff --name-status`, `git diff --cached --name-status`. Nessun `git add .`, reset, restore, checkout, clean o stash; niente processi chiusi o installazioni per aggirare un blocco. Lascia il lavoro nominato e classificato per l'audit, con una tabella per **ogni** tracked modificato e untracked:

| File/tema | Origine | Stato/destino previsto |
|---|---|---|
| nuovi moduli, `src/lib.rs`, README, eventuale test pubblico nominato | questo mandato implementativo | pronti per audit; futuro commit mirato 3C.3.a solo dopo autorizzazione |
| appendice della specifica e questo mandato | registrazione del lotto | continuità nominata; futuro commit documentale separato solo dopo autorizzazione |

Il presente MD è già la continuità nominata del lotto; non creare altri mandati autoreferenziali. Se compare una modifica estranea, classificarla e conservarla senza incorporarla o scartarla. La chiusura Git resta successiva all'audit e all'autorizzazione di Luca. **Consegna attesa: implementazione pronta per audit, nessun commit o push.**

---

## Appendice di esito (Claude, 2026-10-21)

Aggiunta in coda senza toccare il mandato e le sue autorizzazioni, che restano
quelle sopra.

**Consegna: implementazione pronta per audit. Nessun commit, nessun push.**

| Voce | Esito |
|---|---|
| file nuovi | `src/impressao.rs` (1.111 righe), `src/revisao.rs` (2.859 righe) |
| file modificati | `src/lib.rs` (due righe), `README.md`, appendice §13 della specifica |
| file esclusi dal mandato | **nessuno toccato**, verificato uno per uno con `git status --porcelain` |
| dipendenze | **nessuna nuova**: `sha2`, `chrono`, `unicode-normalization` erano gia' dichiarate |
| test | **267 passati** (251 lib, 3 binario, 13 CLI); 59 nuovi; i 208 preesistenti verdi **senza modifiche** |
| audit del lotto | tre riscontri `R3C3A-01/02/03` **accolti e corretti**, con sei regressioni permanenti scritte prima della correzione: §13.6 della specifica |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti, nessuna soppressione di warning |
| sei fonti canoniche | 6/6 invariate |
| vettori normativi | riprodotti dal codice Rust: I 948 byte e `f8a6e8d0…14c`, J 952 byte, K/L, A–H |
| contratti implementati | F3C-04, F3C-05, F3C-13, F3C-22, F3C-23 e F3C-28…F3C-46 |
| limite dichiarato | `AlvoAmbiguo` non e' raggiungibile dall'API: la validazione della 3C.2 intercetta il caso prima, e un test registra **quale** controllo lo fa invece di forzare un tipo incoerente |

Dettaglio completo, mappa test/contratti e limiti residui: §13 di
`SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md`.

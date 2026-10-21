# Mandato a Claude — specifica 3C.3: revisione umana e versioni

Data: 2026-10-17. Destinatari: Claude (analisi), Luca (decisioni), Codex (audit).
Repository: Rust_registrazioni_contabili.
Workspace: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Metodo: `programmatore-bravo`. Prima contrattualizza, poi programma.

## 1. Obiettivo e autorizzazione

Luca ha risposto «procedi» alla proposta di preparare un mandato circoscritto per **specificare** la revisione umana e proporre il lotto 3C.3 prima del codice.

Consegna nella radice del repository un solo nuovo documento:

**`SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md`**

Deve permettere di scegliere D9 e la parte di D8 necessaria alla revisione, collegare ogni decisione alla versione effettivamente esaminata e proporre un primo lotto implementabile con API, errori, file e test precisi.

**Questo mandato autorizza analisi e scrittura della specifica, non implementazione.** D8 e D9 restano proposte da presentare a Luca; «procedi» non ne approva automaticamente le opzioni. Completa il lavoro indipendente dalle scelte aperte senza interromperti per normali scelte editoriali.

Puoi fare letture e piccole sonde sintetiche fuori dal repository, senza installazioni o nuove dipendenze. Nessun codice Rust, modifica di Cargo, writer, archivio, CLI, export, import dello storico o produzione. Nessun commit, push, cancellazione o riscrittura della storia Git. Le precedenti autorizzazioni di pubblicazione sono state consumate dai lotti già chiusi.

## 2. Baseline verificata e problema da risolvere

- HEAD locale: **`37c0e039c3c9366d505dac07e0bf7a2d97661575`**.
- Commit 3C.2: `03e16ec`; documentazione: `37c0e03`.
- Prima di creare questo mandato: worktree pulito, nessuno staged, `main...origin/main` senza divergenza. Il ref locale `origin/main` coincide con HEAD; questa preparazione non ha ripetuto una verifica di rete del remoto.
- Baseline indipendentemente verificata nell'audit precedente: **208 test passati** (192 lib + 3 bin + 13 CLI), inclusi 53 test 3B, fmt/clippy/diff puliti, sei fonti canoniche invariate. Non rieseguita per creare Markdown.
- I quattro file `src/alocacao.rs`, `src/dinheiro.rs`, `src/lib.rs`, `src/prelivro.rs` sono stati confrontati via SHA-256 con lo snapshot dell'audit finale 3C.2: identici.
- D10(a) e D13(a) sono approvate; R3C2-01/02/03 chiusi. Non riaprirli.
- Il pre-libro è puro, conserva riconciliazioni e documenti senza righe, marca i riepiloghi per stato del documento, ordina deterministicamente e rifiuta contesti incoerenti.
- Oggi `StatusRevisao` rappresenta `Ok`, `Revisar`, `Aprovado`, ma non esiste un servizio di revisione tracciata. La presenza dell'enum non dimostra che un'approvazione sia stata eseguita o attribuita a qualcuno.

Il lavoro esistente non basta perché `IdentidadeLinha` distingue righe **nel lotto**, ma non definisce ancora la versione storica del documento, e mancano transizioni umane e provenienza della decisione. Questi sono **BLOCCANTE_ORA prima di implementare la revisione**. Persistenza, autenticazione e import dello storico sono **DA_PIANIFICARE nei loro blocchi**, non prerequisiti impliciti di una funzione pura.

I documenti precedenti contengono fotografie storiche: per esempio «registrazione da autorizzare», helper ancora privati in `alocacao` o test ancora pianificati. Per lo stato corrente usa Git e i sorgenti. Non riscrivere i vecchi mandati né fare pulizia documentale generale in questo incarico.

## 3. Fonti e letture mirate

Usa in sola lettura:

- `SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md`: §§3, 4, 5.2, 8.4, D8/D9, F3C-04/05/13/22/23 e §§13/14.
- `src/prelivro.rs`: identità, ingresso e uscita pubblici, verifiche di coerenza/lotto, estratto approvato, marcatura dei riepiloghi e test pertinenti.
- `src/modelos.rs`: `Lancamento`, `StatusRevisao`, `Dado`, `Competencia`.
- `src/alocacao.rs`: `AlocacaoNfe`, `Reconciliacao`, stato tecnico originale, hash C23 e ordinali C18; niente rilettura integrale dei test monetari.
- `src/dinheiro.rs` solo per riusare il contratto monetario se necessario; nessun algoritmo nuovo o duplicato.

Riusa gli audit precedenti come evidenza, distinguendoli dalle prove che esegui ora. Verbale finale 3C.2:

`C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\16\01a144b0-273a-7840-931d-94b0df795d2b\audit_implementazione_3c2_r2_20261017\AUDIT_3C2_R2_ESITO_20261017.md`.

Se serve un confronto Python, usa solo il commit canonico **`5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901`** del repository `lucamaggi-ita/Programma_registrazioni-contabili`, senza checkout o scritture nel progetto Python. I 47 test canonici già eseguiti non attestano un workflow di approvazione che non implementano. Un report di revisione o un importatore di regole non equivale a una transizione umana delle registrazioni.

## 4. D9 — significato e contratto della revisione

Raccomandazione da valutare: **D9(a), «visto e accettato»**, con autore/data/motivo dichiarati, nessuna cancellazione di avvisi e nessun trasferimento automatico a una nuova versione. Etichettala **PROPOSTA_NON_APPROVATA**, distinta dai vincoli già vigenti.

La specifica deve contenere una matrice delle operazioni: stato iniziale, richiesta, condizioni, stato risultante, evento prodotto, errore e campi invariati. Copri approvazione da OK o REVISAR, revoca, annotazione, richieste ripetute e richieste riferite a una versione superata.

Vincoli da preservare:

1. Nessuna approvazione automatica e nessun autore, motivo o istante inventato dal sistema operativo o dall'orologio dentro una funzione pura.
2. Distinguere **esito tecnico originale**, **decisione umana** e **stato della vista di revisione**. Una revoca non cancella l'approvazione precedente dalla traccia.
3. Approvare non modifica valori, conto, hash C23, rate, avvisi, copertura, componenti, campi rinviati o i quattro esiti di riconciliazione.
4. Conservare D10(a): un documento ERRO resta escluso dall'estratto approvato, anche se una sua riga porta APROVADO. Proporre esplicitamente se sia ammesso registrare comunque una presa visione/accettazione su quel documento o se il comando debba rifiutarla; non confondere questa policy con il filtro già deciso.
5. Correzione contabile, cambio conto, modifica importo, pagamento e modifica YAML sono operazioni separate: non introdurle attraverso `observacoes` o una generica richiesta di revisione.

Dichiara i dati minimi dell'evento e la loro provenienza: identità del bersaglio, versione, operazione, autore dichiarato, istante con convenzione temporale esplicita, motivo, stato precedente e risultante, dati necessari per capire cosa è stato esaminato. Specifica cosa è obbligatorio e l'errore per assenza; nessun default che sembri un'approvazione valida.

Non inventare ruoli o autenticazione: una stringa attribuisce una dichiarazione, non dimostra chi fosse la persona. Definisci anche l'esito di un `Aprovado` ricevuto senza evento di provenienza: non equipararlo silenziosamente a una revisione verificata e non perdere il dato originale. Presenta una soluzione circoscritta compatibile con la 3C.2 già esistente.

## 5. D8 — identità, versione e impronta necessarie alla revisione

La proposta D8(a) della specifica generale non è ancora approvata. Verificane la coerenza prima di ripeterla.

### Identità e rigenerazione

Distingui lotto/esecuzione, identità del documento, identità di riga nella versione, identità della versione e identità dell'evento. Risolvi con esempi la tensione fra questi due fatti:

- `IdentidadeLinha` attuale include `lote`;
- una rigenerazione può avere un lotto diverso ma lo stesso contenuto esaminato.

Non promettere conservazione delle approvazioni fra lotti se l'algoritmo cerca soltanto l'identità completa che contiene il vecchio lotto. Proponi un legame esplicito e verificabile, con conflitto se la corrispondenza non è univoca. Non cambiare la semantica dell'identità attuale per silenzio.

C23 resta invariato: `hash_registro` a quattro campi **non è una chiave univoca** e non distingue tutte le correzioni. Due righe legittime con payload uguale devono poter essere revisionate separatamente. L'ordinale resta quello dell'allocazione 3B, non l'indice del pre-libro ordinato o della futura riga Excel.

### Testo canonico dell'impronta

Consegna una proposta **implementabile**, non «faremo un hash del documento». Deve definire:

- versione dello schema di impronta e campi esatti inclusi/esclusi, con motivazione;
- contenuto di documento, righe e riconciliazione rilevante per conservare un'approvazione; nessuna correzione a parità di totale deve essere invisibile;
- ruolo del piano corrente e dei nomi di conto nella vista revisionata: se cambia ciò che l'utente esamina, dire se cambia la versione o come viene segnalato il cambiamento;
- rappresentazione esatta dei monetari, compresi estremi Decimal, zeri finali e quantità/prezzi unitari che non appartengono necessariamente al dominio dei centesimi;
- assenza rispetto a zero, varianti degli enum, campi `Dado` inclusi, stringhe, UTF-8, separazione/escaping e ordine dei campi/collezioni;
- trattamento di fonti, percorsi e dettagli degli avvisi che possono contenere dati ambientali: non promettere invarianza al percorso se esso rientra indirettamente nei byte;
- esclusione o separazione degli eventi di revisione dall'impronta tecnica: approvare non deve invalidare da solo l'oggetto appena approvato;
- algoritmo SHA-256 già disponibile via `sha2`, con alcuni vettori di byte/hash realmente calcolati e provenienza dichiarata. Nessuna nuova dipendenza necessaria alla sola specifica.

Separa impronta di contenuto e sequenza storica: senza archivio non inventare una numerazione persistente consultando l'ambiente. Definisci cosa può calcolare il modulo puro e cosa dovrà ricevere dal futuro archivio. Il contratto di confronto del contenuto va risolto ora; scritture e gestione dell'archivio restano 3C.4 e D7 non viene scelta implicitamente.

## 6. Proposta del primo lotto implementabile

Confronta brevemente eventi/decisioni separati dai dati allocati con una proiezione revisionata ottenuta per copia validata. Raccomanda la soluzione minima che mantenga intatta la fonte 3B e funzioni con il pre-libro attuale. Non imporre un refactoring generale per introdurre la revisione.

Indica tipi, firme indicative, errori e file da toccare. Deve risultare chiaro:

- quale funzione valida/applica una richiesta e quale costruisce la vista con le decisioni;
- come si verifica che riga e versione siano quelle esaminate;
- quali dati sono letti, restituiti o immutabili e come si evita un risultato parzialmente applicato in errore;
- come annotazioni, approvazioni e revoche vengono riprodotte su rigenerazione identica e come restano consultabili se la versione cambia;
- come vengono trattati eventi duplicati, due decisioni incompatibili e ordine delle operazioni: nessun «vince l'ultimo» basato sull'ordine d'ingresso non dichiarato;
- dove vivono le informazioni necessarie se non esiste ancora persistenza. Una dimostrazione in memoria non equivale a uno storico salvato.

La 3C.2 non genera righe manuali: preservare informazioni umane non autorizza a introdurre ora creazione/modifica di movimenti manuali. Identifica i contratti F3C realmente coperti dal lotto e quelli rinviati.

## 7. Casi minimi da trasformare in test pianificati

| Caso | Proprietà da specificare |
|---|---|
| RC3-01 | OK → approvazione esplicita, con attribuzione completa |
| RC3-02 | REVISAR → approvazione: avvisi e riconciliazione restano identici |
| RC3-03 | Documento ERRO: esito della richiesta dichiarato; nessun ingresso nell'estratto |
| RC3-04 | Revoca: nuovo evento, approvazione precedente conservata |
| RC3-05 | Autore/motivo/istante o bersaglio mancanti: errore esplicito, nessun default |
| RC3-06 | Versione superata o riga assente: nessuna applicazione ad altra riga/versione |
| RC3-07 | Due righe con uguale hash C23: decisioni distinte, nessuno scarto |
| RC3-08 | Correzione a parità di chiave e totale: versione diversa, nessun riuso automatico |
| RC3-09 | Cambio conto/regole o contenuto esaminato: effetto sulla versione dichiarato |
| RC3-10 | Contenuto identico in altro lotto: corrispondenza esplicita secondo la policy proposta |
| RC3-11 | Riordino input del pre-libro: stessi bersagli; riordino articoli XML distinto da questo caso |
| RC3-12 | `10`, `10.00`, `10.0000`; assenza/zero; Decimal estremi; separatori nei testi: vettori canonici |
| RC3-13 | Percorso o metadati ambientali cambiati: impronta secondo inclusioni/esclusioni effettive |
| RC3-14 | Richiesta/evento duplicato e decisioni incompatibili: ripetizione o conflitto dichiarati |
| RC3-15 | Annotazione conservata; approvazione non altera da sola l'impronta tecnica |
| RC3-16 | Documento senza righe e APROVADO senza provenienza: esiti espliciti, nessuna identità inventata |

Sono **test pianificati**, non esistenti né superati. Puoi raggrupparli o aggiungere casi soltanto quando migliorano una verifica concreta; il numero non è una quota. Per ogni caso nomina ingresso, atteso, errore/stato e blocco che lo implementerà. Distingui le eventuali sonde eseguite dai futuri test Rust.

## 8. Consegna e criteri di accettazione

La specifica deve includere: flusso breve, inventario dei tipi riusabili, matrice transizioni, tabella identità/versioni, algoritmo canonico proposto con vettori, API/errori, matrice di contratti con riferimenti ai F3C esistenti, casi e piano del primo lotto.

Per ogni vera scelta residua indica opzioni ammissibili, raccomandazione, effetto pratico e che cosa blocca. D8/D9 possono avere sottoscelte motivate: non creare decisioni editoriali né ereditare approvazioni per silenzio. D7/D11/D12 restano rinviate; se individui una dipendenza reale da una di esse, dimostrala e restringi la proposta prima di allargare il mandato.

Etichetta ogni affermazione come verificata nel codice, evidenza precedente riusata, proposta o non verificata. Cita file/funzioni/righe effettive; non attribuire al Python semantiche lette soltanto nella proposta Rust. Nessuna dichiarazione di test eseguito senza log o risultato.

In questa attività di solo Markdown non rieseguire le 208 prove per cambiare una frase. Se fai una sonda sull'impronta, registra input, algoritmo, runtime e atteso osservato; non trasformare il prototipo in codice di produzione.

## 9. Disciplina Git, rischi e chiusura dell'incarico

Prima della consegna esegui `git status -sb`, `git diff --name-status`, `git diff --cached --name-status`. Non usare reset, restore, checkout, clean o stash. Se trovi modifiche inattese, classificale senza cancellarle o incorporarle.

Destini previsti dopo l'analisi:

| File | Origine/mandato | Stato previsto | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3C3_REVISIONE_SPECIFICA_20261017.md` | preparazione Codex su richiesta di Luca | mandato, continuità nominata | conservato; eventuale commit documentale dopo audit/autorizzazione |
| `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` | esecuzione di questo mandato | proposta pronta per audit | audit Codex, decisioni di Luca, poi eventuale registrazione autorizzata |

Non modificare altri file del repository. I rischi principali da risolvere nella specifica sono approvazioni attribuite alla versione sbagliata, cambiamenti invisibili all'impronta e falsa equivalenza fra stato tecnico e decisione umana. I limiti contabili e di capacità già dichiarati restano tali; niente produzione.

Chiudi con: file scritto, contratti proposti, prove svolte/riusate/non eseguite, decisioni ancora aperte, rischi residui e stato Git con destinazione di ogni file. Puoi dichiarare consegnata la specifica per audit; **non dichiarare implementata o chiusa la 3C.3**. Nessun tracked modificato senza classificazione; i due untracked nominati sopra costituiscono continuità esplicita.

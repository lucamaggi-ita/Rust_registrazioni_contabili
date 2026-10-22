# Mandato a Claude — specifica della persistenza, lotto 3C.3.b

Data: 2026-10-21. Destinatari: Claude (analisi e specifica), Codex (audit), Luca (decisioni funzionali).
Repository: `Rust_registrazioni_contabili`.
Workspace: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Metodo obbligatorio: `programmatore-bravo`. Prima contrattualizza, poi programma.

## 1. Autorizzazione, decisione D7 e consegna richiesta

Dopo la pubblicazione di 3C.3.a, Codex ha raccomandato un **archivio strutturato e versionato come fonte autorevole**, con Excel generato come derivato. Luca ha risposto **«procedi»**. Questo mandato registra l'adozione di **D7(a)**, già descritta in §11 della specifica generale 3C, e autorizza la preparazione della specifica del prossimo blocco.

**D7(a) ADOTTATA:** documenti e decisioni conservati nell'archivio strutturato sono la fonte autorevole; il workbook è un prodotto derivato. La modifica di un workbook non modifica automaticamente l'archivio. Il percorso umano di revisione resta quello di D8/D9. Nessuna seconda fonte autorevole parallela.

Non riaprire questa scelta né le dieci sottoscelte D8/D9. Il formato fisico, le API e il protocollo di salvataggio devono invece essere dimostrati e specificati: l'adozione di D7 non approva in anticipo uno schema tecnico ancora inesistente.

**Consegna:** scrivi soltanto `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`, per audit prima dell'implementazione. Non implementare il writer o il lettore in questo giro. Il file deve consentire a un programmatore di realizzare il primo lotto senza inventare regole su perdita dei dati, identità, conflitti o recupero da errore.

Lavoro **STRUTTURALE di specifica**, perché introduce la fonte persistente. Il codice attuale non basta: chiudere il processo perde gli eventi in memoria. È BLOCCANTE_ORA per la persistenza la mancata definizione di cosa viene salvato, come si valida e cosa accade se la scrittura fallisce; export, autenticazione e miglioramenti estranei restano rinviati.

## 2. Baseline verificata da Codex

- HEAD locale: **`e6dfca2669c56d9070fd5141908da44fe9ac8cc3`**.
- Codice 3C.3.a registrato in `fe5cafa`; specifica e mandato registrati in `e6dfca2`.
- Prima della creazione di questo mandato: `git status -sb` mostra soltanto `main...origin/main`, diff unstaged e staged vuoti, nessuno stash. Nessuna rilettura di rete del remoto effettuata da Codex per preparare il mandato.
- Baseline software: **267 test passati**, 251 lib + 3 bin + 13 CLI, dall'audit mirato finale 3C.3.a; fmt, Clippy e diff check con exit 0. Evidenza riusata: nessuna suite rieseguita per scrivere Markdown.
- I tre riscontri R3C3A-01/02/03 sono chiusi: approvazione attiva richiesta nell'estratto, offset conservato in secondi, scelta canonica del conflitto indipendente dall'ordine dei documenti.
- `Cargo.toml` contiene `serde`, `chrono`, `rust_decimal`, `sha2`, `unicode-normalization`; **non contiene `serde_json` come dipendenza diretta**. La vecchia nota «in cache» non equivale a una dipendenza già autorizzata o utilizzabile: verificarla solo se serve alla proposta.

Fonti: specifica generale `SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md`, specifica D8/D9 `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md` compresa §13.6, mandato implementativo 3C.3.a, codice attuale dei tipi e delle API coinvolte.

Le fotografie storiche possono dire ancora «D7 aperta» o «in attesa di audit»: per la nuova scelta D7 vale questo mandato; per la registrazione già effettuata vale Git. Non aggiornare tutti i documenti storici per questo motivo.

## 3. Confini: 3C.3.b e rapporto con 3C.4

README e appendici nominano 3C.3.b come persistenza degli eventi; altre sezioni storiche assegnano tutta la persistenza a 3C.4. Dichiarare questa sovrapposizione e produrre una mappa unica dei confini, senza cambiare il comportamento approvato.

Il prossimo blocco deve rendere possibile conservare gli eventi e rigiocare la revisione dopo una nuova esecuzione. Se questo richiede una porzione minima dell'archivio documentale, descriverla apertamente come prerequisito condiviso, anziché creare un secondo archivio incompatibile per i soli eventi.

**Base tecnica raccomandata da verificare:** archivio locale strutturato, inizialmente JSON UTF-8 versionato, con snapshot dei contesti originali non proiettati, metadati delle esecuzioni, impronte e dichiarazioni degli eventi. La vista, i riepiloghi e gli estratti sono ricostruibili e non sostituiscono gli originali. Confrontare questa base con un registro dei soli eventi che richieda al chiamante i documenti originali: precisare cosa renderebbe riproducibile ciascuna soluzione e raccomandare il minimo coerente con D7. Non dare per risolto il recupero se mancano i documenti necessari.

Fuori da questo giro: codice Rust del progetto, Cargo, installazioni, archivi reali, migrazioni di dati, export/import Excel, adattatori di importi grezzi, CLI, DRE, produzione e autenticazione. D11 e D12 restano aperte; nessuno di questi blocchi va avviato per dimostrare la persistenza degli ingressi già tipizzati.

## 4. Contratti da definire nella specifica

### 4.1 Fonte, schema e oggetti conservati

1. Definire radice dello schema, identificatore del formato, versione, campi obbligatori, tipi e collegamenti fra documento, versione di contenuto, esecuzione e evento. Fornire un esempio minimo completo e leggibile, distinguendo valori sintetici da quelli del campione canonico.
2. Separare `versao_schema`, versione della grammatica delle impronte, versione del contenuto e identità del lotto. Un numero di schema non è una sequenza storica del documento.
3. Elencare tutti i campi salvati con fonte, rappresentazione, assenza ammessa, validatore e motivo dell'eventuale esclusione. Conservare `Reconciliacao` integra, avvisi, quattro esiti, `Dado` con fonte/stato/valore/motivo, rate, documenti senza righe e campi legacy rinviati.
4. Conservare contesti **prima** della proiezione umana e dichiarazioni degli eventi. Non salvare soltanto lo stato `APROVADO` ricostruito, né la sola vista finale come se fosse la fonte tecnica.
5. L'impronta di contenuto esclude intenzionalmente lotto, file e alcuni dati umani. Perciò due snapshot con la stessa impronta non sono automaticamente identici per l'archivio: definire come conservare esecuzioni, originali e metadati diversi senza perdita o deduplicazione nascosta.
6. Se la riproduzione richiede il piano dei conti, nominarlo fra gli ingressi. Per il round-trip confrontare a parità di piano; un cambio dei nomi deve seguire D8.3 e segnalare la presentazione cambiata. Non ricavare nomi dal primo documento o da configurazioni implicite.

### 4.2 Rappresentazione esatta e validazione in lettura

7. Nessun `f32`/`f64` nel percorso autorevole. Denaro come testo decimale esatto; quantità e altri decimali non monetari mantengono la precisione ammessa dal modello. Non applicare la scala 2 a ogni `Decimal` indistintamente. Provare zero, valori negativi, centesimi e `Decimal::MAX` senza restringerne accidentalmente il dominio.
8. Identificativi testuali, zeri iniziali, Unicode originale, note e testi che iniziano con `=`, `+`, `-`, `@` devono sopravvivere alla rilettura. L'archivio conserva gli originali: la NFC serve alle impronte/chiavi canoniche dove già prevista, non a riscrivere i record salvati.
9. Conservare **offset in secondi e nanosecondi** degli eventi. Il normale `Serialize` di un tipo non dimostra che la sua codifica sia iniettiva. Ripetere la controprova con offset di 1 e 2 secondi; confrontare anche `texto_exato_evento` prima e dopo la lettura. Non reintrodurre RFC3339 ai soli minuti per comodità del formato.
10. Distinguere assenza, zero, stringa vuota e valore dichiarato; decidere esplicitamente il significato di campo omesso contro `null`, senza nuovi default silenziosi. `None` e `Some` del motivo dei `Dado` restano distinti dove il modello li ammette.
11. Documentare lettore grezzo, validazione e costruzione del modello. `Dado<T>` ha già una deserializzazione validata che chiama `Dado::novo`: riusarla o garantire gli stessi vincoli, non aggirarla con un DTO che costruisce stati incoerenti. Gli eventi hanno attualmente `Serialize`, non `Deserialize`: non inventare l'esistenza di un loader completo.
12. Schema sconosciuto, enum non ammesso, campo necessario assente, chiavi JSON duplicate, dato incoerente e riferimenti rotti devono avere esiti nominati. Nessun importo illeggibile convertito in zero e nessuna riparazione automatica dell'archivio. La policy per import grezzi D12 è un'altra cosa.
13. Definire controllo delle impronte salvate contro quelle ricalcolate e controllo della coerenza contesto/righe. Nessun aggiornamento automatico dell'hash per far accettare un contenuto alterato. Un digest rileva incoerenza, non autentica l'autore o la provenienza.

### 4.3 Identità, versioni e storico

14. Riutilizzare `ChaveRevisao`, `IdentidadeLinha`, `IMPRESSAO-CONTEUDO/v1`, impronta di presentazione ed `EVENTO-EXATO/v1` nel loro scopo distinto. Non usare l'hash C23 come chiave univoca o criterio di scarto.
15. Definire reingresso identico, nuovo contenuto con stessa chiave, stesso contenuto in altro lotto, note diverse e medesima dichiarazione ripresentata. Il replay può riconoscere ripetizioni esatte come fa 3C.3.a; la conservazione su disco non può cancellare dichiarazioni o conteggi senza una regola esplicita.
16. In caso di contenuto cambiato, preservare versioni ed eventi precedenti; non trasferire un'approvazione alla versione corretta solo perché totale o hash C23 coincidono. Lo storico è logicamente append-only anche se il file fisico viene riscritto in modo sicuro.
17. Definire i conflitti, il loro ordine canonico e l'esito consultabile. Nessuna politica implicita «vince l'ultimo file». Se la selezione di una versione corrente richiede una nuova decisione funzionale, isolarla e proporre opzioni; non bloccare per questo il salvataggio/rilettura di una storia non conflittuale.
18. La sequenza storica assente della vista pura non diventa automaticamente `1`: eventuali numeri devono avere fonte nell'archivio e una regola esplicita. Preferire rinviare la numerazione se non serve al primo lotto. Non cambiare D8.5 in `revisao.rs` per consentire il writer.

### 4.4 Salvataggio, errore e recupero

19. Definire il protocollo prima di promettere «scrittura atomica»: file temporaneo nella stessa destinazione/filesystem, validazione, eventuale flush/sincronizzazione, pubblicazione e gestione del precedente. Specificare il **punto di commit** e gli esiti prima/dopo quel punto.
20. Non usare «cancella l'originale, poi rinomina» come strategia di sostituzione sicura. Dimostrare cosa consente realmente Windows quando il file è già presente, aperto o bloccato. Nessuna garanzia dedotta dal solo nome `rename` o da una prova su un sistema diverso.
21. Distinguere errore prima della pubblicazione, esito incerto dopo un commit possibile, interruzione del processo e perdita di alimentazione. Non promettere che ogni errore lascia sempre il precedente invariato se il protocollo non può dimostrarlo. Spiegare cosa si può recuperare e cosa resta NON_VERIFICATO.
22. Definire la copia precedente o generazione recuperabile, la sua validazione e quando viene creata; nessuna cancellazione dello storico. Un file corrotto non deve far caricare in silenzio una copia vecchia come se fosse quella corrente. Recupero esplicito, con fonte ed esito dichiarati.
23. Separare contenuto da pubblicare e revisione dell'archivio letto per prepararlo: niente perdita silenziosa di eventi se due chiamanti scrivono da basi diverse. Proporre un protocollo realizzabile di esclusione o rilevamento dei conflitti; un semplice controllo seguito da scrittura senza protezione non elimina la corsa. La libreria non può assumere un solo writer senza un vincolo dichiarato e verificabile.
24. Percorso e metadati operativi provengono dal chiamante; autore, istante e sequenza degli eventi restano dichiarati. Gli eventuali metadati tecnici di una scrittura non sostituiscono quelli umani.

### 4.5 Riproduzione dopo riavvio e API

25. Definire API proposte del blocco puro di codifica/validazione e dello strato IO. Scrivere firme realistiche, tipi di errore, dati richiesti al chiamante e file da toccare. Privilegiare nuovi DTO/adattatori dedicati, mantenendo puri i moduli già auditati.
26. Specificare la prova «nuova esecuzione»: salvare, terminare il processo che possiede gli oggetti, rileggere in un processo distinto e ricostruire con `aplicar_revisao`. Confrontare contesti originali, testo esatto degli eventi, impronte, riconciliazioni, traccia, esclusioni, riepiloghi ed estratto a parità degli ingressi esterni dichiarati. Una variabile clonata nello stesso processo non prova la persistenza.
27. Prevedere un controllo indipendente del file fisico, non solo writer e reader che condividono lo stesso errore. Precisare cosa verifica il secondo strumento e quali garanzie non dà.

## 5. Matrice minima di prove

Nella nuova specifica ogni caso deve avere identificatore, input minimo, esito, contratto, blocco che lo realizza e stato della prova. I test futuri sono **PIANIFICATI**, non esistenti.

| Caso | Prova richiesta |
|---|---|
| P-01 | campione canonico: snapshot + eventi, rilettura e stessa vista in nuova esecuzione |
| P-02 | documento senza righe: identità, riconciliazione e avvisi conservati |
| P-03 | mancante contro zero reale; `Dado` con motivo presente/assente valido |
| P-04 | `Decimal::MAX`, valore negativo, `0.01`, quantità a più decimali, nessun float o perdita |
| P-05 | chiave/CNPJ con zeri iniziali, Unicode NFC/NFD originali, nota tipo formula |
| P-06 | offset di 1 e 2 secondi, nanosecondi, stesso istante con offset diversi: eventi esatti conservati |
| P-07 | payload C23 ripetuto su due righe: nessuno scarto, bersagli distinti |
| P-08 | `APROVADO` senza provenienza, sanatoria, revoca, ri-approvazione: stessi esiti dopo rilettura |
| P-09 | stesso contenuto in altro lotto; nuovo contenuto a parità di totale: storico distinto, nessun trasferimento improprio |
| P-10 | due eventi identici ripresentati: ripetizione dichiarata, non perdita silenziosa dello storico |
| P-11 | schema/enum non supportato, campo assente, JSON troncato e chiave duplicata: errore nominato |
| P-12 | impronta salvata alterata, contenuto alterato, contesto incoerente: rifiuto senza riparazione |
| P-13 | errore durante la preparazione del nuovo file: precedente intatto, errore consultabile |
| P-14 | destinazione bloccata o sostituzione fallita su Windows: esito provato, precedente recuperabile |
| P-15 | interruzione nelle fasi del protocollo: riavvio e scelta esplicita fra generazioni validate |
| P-16 | due scrittori da stessa base: nessun evento sovrascritto in silenzio |
| P-17 | permutazioni documenti/eventi: vista ed errore canonici; ordine fisico del file definito separatamente |
| P-18 | cambio del solo nome del conto: contenuto stabile, presentazione cambiata secondo D8.3 |

Non trasformare questa lista in una quota numerica: copertura delle condizioni e significato delle prove contano più del numero dei test. Riutilizzare i 267 esistenti come baseline; non riscriverli per accomodare il nuovo formato.

## 6. Come svolgere il lavoro e quali prove eseguire ora

- Prima rileggi soltanto i tipi e i passaggi necessari; cita file/riga della baseline quando affermi fatti sul codice. Non ripetere l'analisi Python o gli audit già chiusi.
- Puoi eseguire sonde piccole fuori dal repository, con file sintetici temporanei, per verificare precisione della serializzazione, codec temporale e comportamento del filesystem Windows. Nessuna installazione, nessuna scrittura su archivi o workbook reali, nessun processo utente chiuso.
- Non creare sorgenti del progetto per simulare un'implementazione già approvata. Se una dipendenza manca, dichiarare il limite o usare una sonda disponibile, senza installarla.
- Distinguere prove ESEGUITE, LETTE_DAL_CODICE, RIUSATE e NON_ESEGUITE. Non attribuire a una sonda JSON generica la validazione del futuro lettore Rust.
- Proporre `serde_json` con versione/features e impatto Cargo solo dopo verifica locale. La cache non è autorizzazione a modificarlo; l'aggiunta futura va inclusa nel mandato implementativo, insieme al perimetro che la usa.
- Non rieseguire i 267 test, fmt e Clippy soltanto per il Markdown. Rieseguirli avrebbe senso se emergesse un cambiamento di codice, che questo mandato non richiede.

## 7. Struttura e accettazione della consegna

Il documento deve contenere: fonte autorevole e flusso; schema/esempio; carta dei campi; identità e storico; codec e validatori; protocollo di salvataggio/recupero e concorrenza; API proposte; matrice contratti/prove; confini con 3C.4; eventuali decisioni residue; proposta del **primo lotto implementativo minimo**.

Usare nuovi identificatori senza collisioni con F3C-01…46 e con i vecchi test T3-*. Nominare esplicitamente eventuali contratti 3C generali ora ripresi. I contratti del nuovo writer restano PROPOSTI fino all'audit e all'autorizzazione implementativa; D7(a) è invece adottata in questo mandato.

La specifica è accettabile per l'audit se:

1. ogni dato necessario alla rilettura ha posizione, tipo, fonte e validazione; nessuna informazione necessaria resta solo in memoria;
2. byte degli eventi, valori decimali e identità non dipendono da conversioni implicite o da default;
3. il protocollo distingue atomicità, recuperabilità e durabilità, con fatti provati su Windows e limiti dichiarati;
4. non perde versioni, eventi o metadati diversi solo perché condividono un hash;
5. propone una prova riproducibile fra processi distinti e un primo lotto implementabile senza export/import Excel, CLI o DRE;
6. le nuove scelte funzionali indispensabili sono isolate con opzioni ed effetto concreto, senza chiedere di riapprovare quelle già decise;
7. non cambia alcun file preesistente e consegna uno stato Git completamente classificato.

Preferire una specifica compatta con tabelle ed esempi verificabili, evitando di ricopiare i contratti di 3C.3.a. Nessuna lunghezza o conta di test è un obiettivo in sé.

## 8. Git, file consentiti e resoconto

File di progetto consentito in questo giro: **solo il nuovo `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`**. Questo mandato è continuità nominata da conservare. Non modificare README, specifiche storiche, sorgenti, test, Cargo, YAML, campione canonico o repository Python. Non aggiungere file di dati reali o output di sonde al progetto.

Nessun commit, push, amend, rebase, reset, restore, checkout, clean, stash o cancellazione autorizzato. Nessuna autorizzazione Git delle fasi precedenti si estende automaticamente a questa consegna.

Prima della consegna eseguire:

```text
git status -sb
git diff --name-status
git diff --cached --name-status
```

Classificare ogni tracked modificato se ne comparisse uno, senza scartarlo. Dichiarare tutti gli untracked con origine, stato e destino. Il risultato atteso è nessun tracked modificato e due untracked nominati: questo mandato e la nuova specifica.

Resoconto finale: file scritto; D7 adottata e scelte tecniche ancora proposte; contratti specificati; prove eseguite/riusate/non eseguite; limiti e decisioni residue; piano implementativo; stato Git e destinazione dei due documenti. Consegna per audit Codex: **la specifica non implementa né chiude la persistenza**.

## 9. Continuità al momento della preparazione

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3C3B_PERSISTENZA_SPECIFICA_20261021.md` | richiesta «procedi» di Luca | nuovo mandato documentale | conservato per Claude; eventuale registrazione documentale dopo audit e autorizzazione |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | da eseguire da Claude | **NON_CREATA** da Codex | specifica per audit, non implementazione |

Codex prepara soltanto questo mandato. Nessuna dipendenza, codice, archivio, export o comando nuovo; nessuna produzione contabile.

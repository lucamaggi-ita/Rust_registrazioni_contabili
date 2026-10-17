# Mandato a Claude — 3C.1: specifica di pre-libro, revisione ed esportazione

Data: 2026-10-16.
Repository: Rust_registrazioni_contabili.
Workspace: C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili.
Ruoli: Claude analizza e propone l'implementazione; Codex audita fonti, contratti e prove.
Metodo: programmatore-bravo. Prima contrattualizza, poi programma.

## 1. Obiettivo e autorizzazione di questo blocco

Luca ha detto «procediamo con 3c» dopo la proposta di definire il mandato della fase. Avvia il blocco **3C.1 di analisi e specifica** per rendere utilizzabili le registrazioni della 3B in un pre-libro con revisione ed esportazione.

Consegna un unico file nella radice del repository:

**SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md**

Deve contenere il contratto del flusso completo, le vere decisioni ancora aperte e una proposta concreta del primo blocco implementabile, con API indicative, file, dipendenze e test. Il risultato deve permettere una successiva implementazione da parte tua e un audit mirato di Codex.

Questo passaggio autorizza letture, analisi, sonde su dati sintetici in cartelle temporanee e scrittura della specifica. Non è un'autorizzazione a scegliere per silenzio nuove regole di revisione, cancellazione o sostituzione dello storico.

**In questo blocco non implementare codice Rust, writer, archivio, nuovi comandi o dipendenze.** Non modificare workbook reali, YAML canonici, campione, repository Python o file di produzione. Non installare runtime/pacchetti. Commit e push della nuova fase non sono compresi nelle autorizzazioni consumate per la 3B.

Completa tutta l'analisi indipendente dalle decisioni aperte prima di consegnare. Non interromperti per normali scelte editoriali o tecniche di esposizione. Le decisioni funzionali non deducibili dai contratti vanno proposte con opzioni ed effetto concreto.

## 2. Baseline e perimetro

- HEAD iniziale verificato: **12584e65d01141626bf461321f6d2c5bc3e3c032**, main pubblicato e allineato.
- Commit 3B: e28b7412417d9ac3c2d553fa31b243d473855467.
- Commit documentale: 12584e65d01141626bf461321f6d2c5bc3e3c032.
- Fasi 1, 2, 3A e 3B chiuse. C18–C24, D1–D6 e F3A-24 restano vigenti.
- Baseline di test indipendentemente verificata: **166 passati** (150 lib + 3 bin + 13 CLI); non rieseguita nella sola preparazione di questo mandato.
- La 3B restituisce AlocacaoNfe: registrazioni **e** riconciliazione a quattro esiti, avvisi e campi rinviati. È una funzione pura.
- Nessuna CLI attuale utilizza alocar_nfe; non esistono ancora archivio, writer o flusso di revisione Rust.
- Fase 3D/DRE fuori da questo blocco; mapeamento_dre.yaml e csv_columns.yaml restano NON_LETTO dal Rust.
- Le regole YAML restano canoniche nel Python fino alla migrazione dichiarata.

Lavoro: MIRATO di specifica, preparatorio a una futura modifica STRUTTURALE con persistenza/writer. **BLOCCANTE_ORA** per introdurre scritture è l'assenza dei contratti di identità, conservazione e revisione. I debiti già dichiarati non diventano automaticamente prerequisiti.

README e vecchi mandati possono contenere istantanee precedenti alla pubblicazione («attesa registrazione», «fase 3C non avviata»). Per HEAD e stato corrente usa Git e questo mandato. Non creare un commit o una revisione generale dei vecchi documenti per correggere tali istantanee.

## 3. Fonti a commit fissato e prove disponibili

### Rust, sola lettura

Leggi in modo mirato:
- src/alocacao.rs: AlocacaoNfe, Reconciliacao, esiti, avvisi, campi_rinviati, errori e test sui payload ripetuti;
- src/modelos.rs: Lancamento, Dado, StatusRevisao, Competencia, Duplicata e tipi contabili;
- src/main.rs: CLI e validazione comune C15;
- src/regras.rs e config/FONTE.md;
- specifica 3A, mandato 3B, README e test direttamente pertinenti;
- configurazioni e campione soltanto per verificare dati/fonte, mai per modificarli.

Non assumere il numero dei campi dal vecchio resoconto: inventaria campi e tipi reali e confrontali con le colonne Python.

### Python canonico

Repository: https://github.com/lucamaggi-ita/Programma_registrazioni-contabili
Commit: **5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901**

Usa quel commit per comportamento, righe e test. Nessun checkout/reset nel progetto Python. Eventuali confronti con versioni nuove sono distinti dalla baseline.

Codex ha acquisito e verificato contro lo SHA del blob Git otto file a questo commit:
- src/fantozzi/excel_writer.py;
- src/fantozzi/excel_diagnostics.py;
- src/fantozzi/pre_livro_diario.py;
- src/fantozzi/revision_report.py;
- src/fantozzi/classification_review_workbook_importer.py;
- tests/test_excel_writer.py;
- tests/test_pre_livro_diario.py;
- tests/test_revision_report.py.

Copie esterne riutilizzabili in sola lettura:
C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\16\01a144b0-273a-7840-931d-94b0df795d2b\mandato_fase3c\python_snapshot

Manifesto: fonti_python_3c.json, nella cartella superiore. **Hash verificato non significa file già auditato integralmente:** la preparazione del mandato ha svolto una lettura mirata. Completa le letture necessarie alla specifica.

Individua inoltre, al medesimo commit, modelli, test diagnostici/end-to-end, CLI e run_manager. Leggi operation_classifier o gli altri moduli classification_review_* soltanto per il confine realmente coinvolto. Non espandere il porting a tutti questi sottosistemi perché esistono.

Se il runtime e le dipendenze consentono prove senza installazioni, eseguile fuori dai dati reali; altrimenti distingui sorgenti/test LETTI da test ESEGUITI. Un runtime Python 3.12.14 era disponibile nel bundle Codex durante l'audit 3A; pytest/PyYAML risultavano assenti allora: verifica lo stato attuale se serve, non dichiarare che il runtime non esiste o che pytest sia stato eseguito.

## 4. Punti già osservati da verificare e trattare esplicitamente

Questi riscontri orientano l'analisi; non sono una specifica completa né decisioni funzionali nuove.

| Fonte al commit fissato | Comportamento letto | Conseguenza per la specifica |
|---|---|---|
| pre_livro_diario.py:108–213, 257–284 | il pre-libro legge Excel già esportati; solo status OK entra nel contenitore OK, ogni altro valore nel contenitore di revisione | definire l'ingresso Rust e il trattamento di APROVADO, vuoto e sconosciuto; non confondere OK automatico con approvazione umana |
| pre_livro_diario.py:374–412 | riassunti per conto e mese; valore totale = valore OK + valore da revisione | dichiarare popolazione, azienda, competenza, categorie e somme esatte |
| pre_livro_diario.py:738–744 | importo vuoto o non interpretabile diventa 0.00 | conflitto con C2/C10: non ereditare il fallback silenzioso |
| excel_writer.py, write_lancamentos_to_excel / _existing_dedupe_keys / _dedupe_key | append su workbook, deduplicazione per hash o fallback; chiave aggiunta al set durante il ciclo | l'hash C23 non è un'identità univoca; il writer può scartare la seconda riga valida con payload ripetuto, anche nello stesso lotto |
| excel_writer.py, _row_values / _duplicatas_json | export delle righe e delle rate, non della nuova Reconciliacao Rust | definire come conservare quattro esiti, Dado e motivi, oltre alle colonne storiche |
| excel_diagnostics.py:106–150 | diagnostica hash ripetuti e righe di revisione | distinguere conflitto, stessa impronta valida e reimportazione; una diagnosi a valle non restituisce una riga già scartata |
| classification_review_workbook_importer.py:48–85 | importa una planilha di revisione classificazioni e produce un CSV rivisto | non dimostra da solo un flusso di approvazione dei Lancamento: revisione di regole e revisione contabile sono oggetti distinti |

I riferimenti sono letture del codice, non prove eseguite del flusso Python. Per ogni contratto della nuova specifica cita funzione/righe, test pertinente e livello di verifica.

## 5. Contenuto richiesto della specifica

### A. Flusso, ingressi e fonte autorevole

Disegna il percorso:
AlocacaoNfe con contesto documento → pre-libro → revisione → eventuale persistenza/esportazione → rilettura e verifica.

Distingui:
- rappresentazione autorevole e report derivato;
- archivio di singola esecuzione e storico cumulativo;
- nuovo flusso Rust e import eventuale degli Excel Python preesistenti;
- righe, documenti, lotti e loro provenienza.

Confronta almeno due soluzioni praticabili per il formato autorevole: workbook e archivio strutturato/versionato con workbook derivato. Valuta fedeltà di Decimal/Dado, riconciliazione, modifica umana, compatibilità storica, ripetibilità e dipendenze. Non scegliere un database o un formato per inerzia.

Non far dipendere il nuovo flusso dall'esistenza di uno storico Excel non fornito: se l'import dello storico è rinviato, indicare perimetro e limite in modo esplicito.

### B. Identità, reimportazione e conflitti

C23 rimane intatto: hash di quattro campi confrontabile con il Python, **non chiave univoca e mai motivo sufficiente per scartare una registrazione valida**.

Definisci con esempi separati:
1. due righe legittime della stessa allocazione con payload uguale;
2. reimportazione identica del medesimo documento;
3. correzione del documento a parità di chiave/totale;
4. cambio di regole o conto;
5. riga manuale e annotazione/approvazione da preservare;
6. conflitto fra archivi o versioni.

Proponi l'identità persistente e la politica di riconciliazione senza alterare retroattivamente l'hash C23. Nessun indice di riga o UUID dichiarato stabile senza la definizione di stabilità. Nessuna sostituzione/cancellazione automatica dello storico proposta come già approvata.

Se il primo blocco non crea uno storico, deduplicazione e import dello storico vanno separati dalla conservazione di tutte le righe del lotto corrente: la seconda è comunque obbligatoria.

### C. Stati, revisione umana e autorizzazioni

Definisci la macchina a stati per OK, REVISAR e APROVADO e gli eventuali stati di input non riconosciuti. Per ogni transizione: chi la richiede, prerequisiti, campi modificabili, fonte, motivo, valori prima/dopo, versione e comportamento alla rigenerazione.

Nessuna promozione automatica a APROVADO. Nessun default OK per stato vuoto/sconosciuto. L'approvazione umana non cancella avvisi o cambia retroattivamente gli esiti tecnici della riconciliazione.

Distingui approvare una registrazione, correggere un conto, modificare un importo, riconoscere un pagamento e cambiare una regola YAML. Se una di queste operazioni è fuori dal primo blocco, deve restare dichiarata come non implementata.

Non assumere che l'identità OS sia automaticamente una firma dell'approvatore o che una cella modificata costituisca un'autorizzazione completa. Proponi il minimo tracciamento necessario, senza trasformare la fase in un sistema di utenti/permessi non richiesto.

### D. Conservazione di riconciliazione, assenze e copertura

Non appiattire AlocacaoNfe in un elenco di righe perdendo il documento:
- quattro esiti e regola di aggregazione C24;
- avvisi del documento, componente parziale e relativa motivazione;
- articoli attesi non disponibili e limite di copertura;
- campi_rinviati e dati non popolati dalla 3B;
- data_pagamento assente, scadenze distinte dai pagamenti.

Decidi esplicitamente cosa entra nel pre-libro diagnostico, nei riepiloghi e nell'eventuale estratto approvato quando il documento è ERRO/PARCIAL o una riga è REVISAR. Nessun filtro silenzioso né documento “ripulito” perché le sue righe hanno status OK.

Per dati monetari di input mancanti/non validi, descrivere rifiuto o stato incompleto e motivo; non sommarli come zeri. I riepiloghi non possono dichiararsi completi quando non conoscono gli importi inclusi.

### E. Aggregazioni e invarianti

Dichiara granularità, ordinamento e isolamento per azienda, competenza e conto. Un CNPJ mancante non deve riunire inconsapevolmente aziende diverse. Lo stesso conto con nomi divergenti richiede un esito dichiarato, non una correzione manuale implicita.

Definisci le partizioni e gli invarianti:
- tutte le righe ricevute sono conservate oppure hanno un esito esplicito;
- categorie di revisione mutuamente esclusive, conteggi riconciliati;
- somme di categoria e totale, per documento/azienda/periodo secondo il perimetro;
- nessuna eliminazione di duplicati da una semplice somma per hash;
- eventuali righe escluse riportano identità, valore se noto, stato e motivo.

Per formule o metriche indica numeratore, denominatore e insieme incluso/escluso. Non inventare una percentuale di completezza. Le somme monetarie devono riusare il criterio esatto/controllato della 3B; segnala se gli helper oggi privati richiedono un refactoring minimo futuro, senza duplicarli o implementarli ora.

### F. Formato di esportazione e scritture

Specifica campi e tipi: importi, date, identificativi testuali, rate, stati, fonti e motivi. Studia la fedeltà del round-trip: non basta che il file “si apra”.

Punti obbligatori:
- cifre monetarie esatte e gestione della capacità numerica;
- CNPJ/CPF, chiavi NF-e e zeri iniziali conservati come identificativi;
- stringhe, celle vuote, date/scadenze/pagamenti, formule e valori memorizzati;
- celle testuali che iniziano con caratteri interpretabili come formule: conservare il testo, senza esecuzione involontaria;
- header mancanti, duplicati o incompatibili; schema/versione e migrazione;
- campi di revisione umana preservati alla rigenerazione;
- conflitti fra percorso di input/output, file esistente/aperto/bloccato, errore durante il salvataggio.

Proponi scritture verificabili: output di prova, rilettura indipendente, destinazioni dichiarate e nessuna modifica parziale dell'archivio in caso di errore. Non convertire denaro a f64 per comodità del writer né forzare importi rappresentabili in celle numeriche che ne perdano la precisione.

La dipendenza Rust per XLSX/lettura o per un nuovo archivio è una scelta da motivare con API, capacità e modifica Cargo mirata; questa analisi non ne autorizza installazione o integrazione.

### G. Confini e passaggio operativo

Distingui un primo percorso NF-e utilizzabile da una dichiarazione di parità completa col Python:
- classificazione di operazioni speciali e suoi fogli non esiste ancora nel Rust;
- import CSV, modifiche delle regole e migrazione storica non sono già coperti dal solo export;
- aggregazione DRE, mapping e promozione contabile rimangono 3D o mandati separati.

Per ogni funzione Python pertinente: incluso nel primo blocco, rinviato con motivo o fuori perimetro. Non creare un foglio vuoto che sembri attestare l'assenza di operazioni mai analizzate.

## 6. Matrice dei contratti e prove

Usa identificativi provvisori F3C-01, F3C-02, ecc.; non numerare C25+ come contratti già implementati.

| ID | Regola | Fonte e verifica | Stato decisione | Input/assenza/conflitto | Esito | Futuro codice | Test e risultato atteso |
|---|---|---|---|---|---|---|---|

Stati della decisione: EREDITATO_VERIFICATO, VINCOLO_GIA_APPROVATO, PROPOSTO, DA_DECIDERE. Distingui un vincolo C2/C10/C23 già vigente da una nuova proposta di persistenza o revisione.

Prevedi almeno questi casi concreti, indicando se letti, eseguiti o da creare:
1. campione canonico: tre registrazioni per 35.00 e rate conservate;
2. riga sconosciuta/ambigua REVISAR con conto e avvisi invariati;
3. APROVADO distinto da OK e REVISAR; stato vuoto/sconosciuto;
4. documento PARCIAL con righe singolarmente OK; documento ERRO;
5. importo mancante/non valido e zero reale: nessuna confusione;
6. data_pagamento assente e scadenza nota;
7. payload uguali nello stesso documento: nessuno scarto, richiamare il test 3B esistente;
8. reimportazione identica e documento corretto con hash invariato;
9. correzione/approvazione manuale preservata oppure conflitto esplicito;
10. due aziende nello stesso mese, azienda non identificata;
11. riepiloghi per conto/competenza con negativi, zeri e quadrature;
12. round-trip esatto di importi grandi/centesimi e identificativi con zeri iniziali;
13. header/schema errato, JSON rate invalido, formula/cella senza valore disponibile;
14. errore di salvataggio/percorso e file preesistente invariato;
15. input permutati/rigenerazione: ordinamento e identità secondo il contratto dichiarato.

Non tutti devono diventare test del primo blocco: collega ogni caso al blocco che lo implementa e impedisci che un test pianificato venga presentato come già verde.

## 7. Decisioni da consegnare e primo blocco proposto

Raggruppa in una lista breve soltanto le scelte funzionali che cambiano dati, esclusioni, revisioni o storico. Per ciascuna: fatto osservato, opzioni, raccomandazione, esempio prima/dopo, rischio, blocco interessato e stato PROPOSTO.

Temi attesi, da ridurre/precisare con l'analisi:
- fonte autorevole, compatibilità/migrazione degli Excel e percorso degli output;
- identità/versioni e trattamento della reimportazione/conflitto;
- transizioni di revisione e significato operativo dell'approvazione;
- inclusione di documenti/righe parziali o non riconciliati nei diversi output;
- fedeltà del formato quando celle numeriche e assenze non rappresentano i dati esatti.

Non chiedere a Luca di scegliere nomi di funzioni, file o organizzazione dei test. Non inferire approvazione dal silenzio.

Nel medesimo documento proponi **3C.2**, il lotto minimo coerente: preferibilmente costruzione e verifica del pre-libro in memoria prima del writer, se l'analisi conferma che fornisce una base utile. Non dare tale preferenza per una scelta di archivio già approvata.

Includi API indicative con tipi realizzabili, file da toccare, dipendenze necessarie o assenti, numero dei test derivato dalla lista, invarianti, criteri di accettazione, limiti residui e ciò che rimarrà fuori. Mostra come i blocchi successivi completeranno revisione, IO e uso tramite CLI: non dichiarare l'intera 3C completata con il solo modulo puro.

## 8. Verifica e consegna

Per ogni affermazione importante: fonte, stato e prova. Nessuna suite va rieseguita per una sola modifica Markdown se i contenuti coperti sono invariati; riusa la baseline dei 166 test dichiarandolo.

Consegna:
1. specifica unica completa;
2. tabella divergenze Python/Rust, separando bug della fonte e scelte di porting;
3. decisioni funzionali ancora aperte con raccomandazioni;
4. proposta verificabile 3C.2;
5. prove eseguite/non eseguite, limiti, stato Git e destinazione dei file.

Non modificare altri file del repository per comodità, non correggere debiti fuori mandato. Se emerge un vero blocco nuovo, spiega la prova e il suo effetto prima di allargare il perimetro.

La specifica passa poi all'audit di Codex. Questo mandato non attesta la correttezza di una futura implementazione e non autorizza produzione, scritture reali, commit o push.

## 9. Continuità Git del nuovo lavoro

Prima dell'handoff: git status -sb, git diff --name-status, git diff --cached --name-status. Nessun git add ., reset, restore, checkout, clean o stash.

Stato alla creazione: worktree pulito su 12584e6; il solo nuovo file del repository è questo mandato. È un output documentale dichiarato, non un file da ignorare.

| File | Origine/mandato | Stato iniziale | Destino |
|---|---|---|---|
| MANDATO_CLAUDE_FASE3C_SPECIFICA_20261016.md | richiesta Luca «procediamo con 3c», preparazione Codex | untracked, mandato pronto | conservare per analisi/audit 3C.1; futuro commit documentale dopo autorizzazione |
| SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md | esecuzione di questo mandato | DA_CREARE | audit Codex e futura registrazione documentale dopo autorizzazione |

La copia nominata del mandato e le fonti della preparazione sono conservate fuori dal repository in:
C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\16\01a144b0-273a-7840-931d-94b0df795d2b\mandato_fase3c

**FASE 3C.1 AVVIATA; specifica ancora da redigere e auditare.** Nessuna implementazione 3C/3D, commit o push avviati con questo documento.


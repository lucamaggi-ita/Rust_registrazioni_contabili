# Mandato a Claude — fase 3A: confronto Python e contratti delle registrazioni

Data: 2026-10-16.
Progetto: `Rust_registrazioni_contabili`.
Directory: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Ruoli: Claude analizza e redige la specifica; Codex verifica fonti, contratti ed esempi.
Metodo obbligatorio: skill `programmatore-bravo`, prima contrattualizza, poi programma.

## 1. Obiettivo e autorizzazione

Luca ha chiesto di preparare questo mandato dopo aver individuato nella fase 3A il prossimo passo.
È autorizzato il lavoro di **analisi e specifica** per trasformare, nella futura fase 3B, le NF-e già lette e classificate in registrazioni `Lancamento`.

Consegna un unico documento: **`SPECIFICA_FASE3A_REGISTRAZIONI.md`**, nella radice di questo repository.
Il documento deve consentire di implementare e verificare la fase 3B senza dover ricostruire decisioni da una chat.
Non limitarti a una roadmap: servono regole, fonti, formule, esempi con risultati attesi e una lista breve delle vere decisioni aperte.

**Questo mandato non autorizza l'implementazione della fase 3**, nuovi comandi CLI, dipendenze, modifiche ai modelli o scritture contabili.
Non estende alla nuova fase le autorizzazioni di commit/push del precedente mandato C17.
Non chiedere conferme durante le normali letture o scelte editoriali: completa prima tutta l'analisi possibile e consegna insieme le decisioni funzionali ancora da risolvere.

## 2. Stato iniziale e problema da risolvere

Baseline documentale: `3a24ba0f1abb3b71e4bed650bf246c72edb1a485`, README riallineato al codice pubblicato.
Fasi 1 e 2 chiuse; C17 verificato; fase 3 non implementata.
Suite Rust già verificata: **113 test** (97 lib + 3 bin + 13 CLI).
La lettura/classificazione esiste; il modello `Lancamento` esiste; manca il flusso che produce registrazioni e ne riconcilia gli importi.

Problema: un porting letterale del Python potrebbe importare assunzioni non dichiarate su riparti, assenze, arrotondamenti e revisione.
Decisione funzionale già vigente: dato mancante diverso da zero, nessun fallback silenzioso, valori monetari Decimal, provenienza e stato espliciti.
Contratto da preparare: ogni futura registrazione deve essere riconducibile alla fonte e a una trasformazione definita e testabile.

Classificazione del lavoro: MIRATO di analisi, preliminare a un'implementazione strutturale.
Rischi principali: leggere la versione Python sbagliata, assumere che Python sia un oracolo corretto su ogni comportamento, confondere ipotesi e decisioni approvate, presentare casi derivati come dati reali.
La mancanza di questa specifica impedisce l'avvio ordinato della fase 3B; gli altri debiti non diventano automaticamente prerequisiti.

## 3. Fonti da leggere e loro stato

### Rust, in sola lettura

Leggi in modo mirato:
- `src/modelos.rs`: `Dado<T>`, `Nfe`, `ProdutoNfe`, `Duplicata`, `Lancamento`, stati e tipi;
- `src/nfe.rs`, `src/classificacao.rs`, `src/regras.rs`: dati realmente disponibili e contratti che li proteggono;
- `README.md` e `config/FONTE.md`;
- `config/plano_contas_gerencial.yaml` e `config/mapeamento_dre.yaml`;
- campione canonico e test esistenti direttamente pertinenti.

`csv_columns.yaml` serve solo a delimitare il futuro ingresso CSV: l'import CSV non è parte della prima implementazione NF-e.
`mapeamento_dre.yaml` è ancora NON_LETTO dal Rust: non scrivere che la sua validazione sia già implementata.

### Python canonico, a versione fissata

Fonte dichiarata in `config/FONTE.md`:
- repository: `https://github.com/lucamaggi-ita/Programma_registrazioni-contabili`;
- commit: **`5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901`**.

Usa i contenuti di quel commit, non la sola branch corrente né un eventuale working tree locale modificato.
Se utilizzi un clone esistente, leggi con `git show <commit>:<path>`; non fare checkout/reset o modifiche nel progetto Python.
Un confronto con una versione più recente è facoltativo: va etichettato separatamente e non sostituisce la baseline.

Individua e verifica i percorsi effettivi, partendo da:
- `src/fantozzi/allocation.py`, già citato dal modello Rust;
- modello Python, punto d'ingresso dell'allocazione, gestione di revisione e riconciliazione;
- funzioni di hash/identità/deduplicazione se realmente presenti;
- test di allocazione, pre-libro e mapping DRE pertinenti.

Non assumere che esista una funzione solo perché il nome è plausibile.
Per ogni affermazione sul Python cita **commit, file e funzione/righe**; indica anche quale test o esempio la conferma.
Se la fonte non è accessibile, segna NON_DISPONIBILE, completa l'analisi Rust e dichiara quali conclusioni restano NON_VERIFICATE. Non ricostruire il comportamento Python per intuizione.

Il repository Python resta la fonte canonica delle regole YAML. Un comportamento storico del suo codice non diventa automaticamente una regola di dominio approvata.

## 4. Contenuto obbligatorio della specifica

### A. Mappa del flusso e carta d'identità degli output

Descrivi il flusso previsto:
`NfeImportada + classificazioni + regole validate → registrazioni + riconciliazione + elementi da revisione`.

Definisci:
- granularità della registrazione: per documento, articolo, conto o raggruppamento;
- identità dell'azienda, del fornitore, del documento e della riga;
- contratto di ciascun campo di `Lancamento`: fonte, trasformazione, stato, copertura, motivo, periodo e comportamento in assenza;
- dati che l'attuale modello non può rappresentare correttamente, con proposta minima di evoluzione **senza implementarla**;
- distinzione fra fonte letta, valore calcolato, valore parziale e dato non applicabile.

Non usare stringhe vuote o zero tecnico per completare campi ancora senza fonte. Per le lacune della specifica usa MANCANTE, NON_VERIFICATO, NON_DISPONIBILE o N.A.
Distingui un campo obbligatorio per produrre una registrazione da un campo facoltativo mancante: l'assenza di fat non deve diventare automaticamente un blocco dell'intera NF-e senza una motivazione funzionale verificata.

### B. Trasformazioni e importi

Confronta Python e proposta Rust su:
- raggruppamento degli articoli e propagazione di conto, tipo e stato di classificazione;
- trasporto, assicurazione, sconti, altre spese e imposte incluse;
- denominatore e base di ogni riparto, articoli esclusi, segni e valori nulli;
- precisione interna, momento dell'arrotondamento, scala dell'output e assegnazione del residuo;
- invarianti fra somma degli articoli, componenti della fattura e totale finale.

Per ogni formula scrivi esplicitamente numeratore, denominatore, componenti incluse/escluse e comportamento quando il denominatore è zero o manca una componente.
Non introdurre un riparto proporzionale o una regola di arrotondamento soltanto perché sembra naturale: dimostra la regola esistente oppure etichetta la proposta come tale.
Indica come evitare che trasporto o imposte già inclusi siano sommati due volte.
Non ricostruire l'importo di fatturazione sommando le rate: C10 resta vigente.

### C. Copertura, dati parziali e revisione

Proponi il contratto minimo per la copertura di `Dado<T>` e per l'output del documento:
- quale insieme è atteso e quale è effettivamente coperto;
- componenti note, assenti ed escluse con motivazione;
- stato e motivo dei valori parziali e dei totali;
- quando si può produrre una registrazione da revisione e quando l'operazione deve fallire;
- propagazione di AMBIGUO, NAO_CLASSIFICADO, conti di revisione e avvisi del parser.

La copertura non deve essere una percentuale senza definizione o un nuovo numero hardcoded.
Non dichiarare un risultato completo se una componente necessaria è assente.
Se un importo obbligatorio per una specifica trasformazione manca, descrivi quale parte viene bloccata e come viene segnalata.
Le rate non attestano da sole un pagamento: definisci la provenienza della data di pagamento e il suo stato di assenza.

### D. Competenza, identità e ripetibilità

Verifica e descrivi:
- data documento, competenza e rapporto con scadenze/pagamento;
- chiave d'origine e hash: campi inclusi, normalizzazione, ordine e stabilità;
- duplicato identico, documento corretto e conflitto fra versioni;
- comportamento atteso alla reimportazione;
- ordine deterministico delle righe e degli avvisi.

Il primo blocco NF-e non deve scrivere su database. Se deduplicazione persistente o aggiornamenti richiedono uno storage futuro, separa il requisito dalla funzionalità realizzabile nella fase 3B.
Non inventare oggi un archivio, un algoritmo di hash o una regola di sostituzione dello storico.

### E. Confine fra registrazioni, pre-libro e DRE

Il modello `Lancamento` espone già campi DRE: chiarisci quando e da quale mapping saranno popolati, senza riempirli silenziosamente.
Distingui:
1. allocazione NF-e e riconciliazione (3B);
2. pre-libro, revisione ed esportazione (3C);
3. aggregazione DRE per competenza (3D).

Se la 3B ha una dipendenza minima dal mapping DRE, descrivila e proponi la validazione necessaria; non anticipare il calcolo completo della DRE.
Segnala lacune o conti non mappati come dati da revisione/errore secondo il contratto proposto, non assegnarli a una linea arbitraria.
CSV, KPI e report completi restano fuori da questo mandato.

## 5. Matrice dei contratti e casi di prova

Usa identificativi provvisori `F3A-01`, `F3A-02`, ecc.; non assegnare nuovi numeri C18+ come se fossero già implementati.

Per ogni contratto includi almeno:

| ID | Regola | Fonte esatta | Stato decisione | Assenza/errore | Codice futuro coinvolto | Caso e atteso | Prova disponibile / da creare |
|---|---|---|---|---|---|---|---|

Stati distinti:
- **EREDITATO_VERIFICATO**: comportamento osservato nella fonte fissata, con prova;
- **VINCOLO_GIÀ_VIGENTE**: contratto Rust già approvato, per esempio C10/C17;
- **PROPOSTO**: scelta consigliata ma non approvata;
- **DA_DECIDERE**: ambiguità funzionale con alternative;
- **NON_VERIFICATO**: fonte o prova insufficiente.

EREDITATO_VERIFICATO non significa automaticamente approvato per il porting, specialmente se contrasta con C10 o con altre regole già vigenti.
In questa fase i test nuovi sono **specificati, non implementati**: evita qualunque etichetta che li faccia apparire già superati.

Prepara esempi numerici completi almeno per:
- campione canonico: tre articoli, vProd/vNF 35.00, fat 35.00 e due rate da 17.50; verifica i valori dalla fonte prima di usarli;
- due articoli assegnati allo stesso conto e uno a conto diverso;
- riparto con almeno una componente accessoria non nulla;
- residuo di arrotondamento di un centesimo;
- denominatore zero e denominatore incompleto;
- assenza fat con due rate; zero reale dichiarato;
- aggregato di imposte parziale;
- classificazione ambigua e prodotto non classificato;
- totale non riconciliato;
- reimportazione identica e documento con dati cambiati.

Un esempio può coprire più casi. Ogni esempio deve mostrare input, passaggi di calcolo, output atteso, stato e motivo.
Se una regola è ancora DA_DECIDERE, calcola e confronta le alternative, senza presentare una sola risposta come definitiva.
Etichetta gli esempi sintetici come SINTETICO: non sono nuove fonti canoniche.
Usa Decimal o calcolo esatto per le eventuali prove; nessun f64 per gli importi.

## 6. Divergenze e decisioni da consegnare

Consegna una tabella Python/Rust con:
- comportamento alla fonte e prova;
- proposta per Rust;
- ragione della parità o della divergenza;
- classificazione: bug, differenza intenzionale, dato mancante, lacuna di prova o scelta da approvare;
- effetto su importi, copertura, revisione e test.

Non correggere il Python, gli YAML, gli alias divergenti/ambigui o C13.
Per la prima 3B proponi di preservare la classificazione attuale e i suoi avvisi, salvo una vera incompatibilità dimostrata.
La scelta di applicare regole per fornitore o politica degli alias brevi resta un mandato separato.

Concludi con una lista breve delle sole decisioni che cambiano il risultato funzionale:
per ciascuna, opzioni concrete, raccomandazione motivata, esempi dell'effetto e cosa impedisce di implementarla senza decisione.
Le scelte tecniche o editoriali prive di effetto funzionale possono essere risolte autonomamente e documentate.

## 7. Confini, verifiche ed efficienza

Consentiti:
- letture mirate, confronto del codice e test esistenti;
- calcoli/probe in area temporanea o di audit esterna al repository;
- creazione del solo documento di specifica.

Esclusi:
- modifiche a `src/`, `tests/`, `config/`, `Cargo.toml`, `Cargo.lock`, campioni canonici, README e mandati precedenti;
- nuove fixture canoniche, nuovi moduli o comandi;
- modifica dei progetti Python, installazione di dipendenze, database, staging/produzione;
- sostituzione di serde_yaml, validazione XSD, pulizia della vecchia directory;
- commit, push, reset, restore, checkout, clean, stash o cancellazioni.

Non ripetere la suite Cargo per scrivere Markdown: riusa la baseline di 113 test se il codice resta invariato.
Se esegui prove Python o calcoli, riporta comando, versione/commit, input, esito e limiti.
Un test della fonte non eseguito va indicato NON_ESEGUITO; non sostituire la prova con “dovrebbe passare”.

Per il documento controlla:
- ogni regola ha fonte o stato esplicito;
- gli esempi numerici tornano e gli arrotondamenti sono dichiarati;
- nessun caso di assenza diventa zero tecnico;
- nessun test pianificato è dichiarato superato;
- gli ID e i riferimenti sono coerenti;
- il perimetro della 3B è implementabile e distinto da 3C/3D.

## 8. Git, output e criterio di consegna

Prima e dopo il lavoro:
```powershell
git status -sb
git diff --name-status
git diff --cached --name-status
git ls-files --others --exclude-standard
```

Il repository era pulito prima della creazione di questo mandato. Il nuovo mandato e la futura specifica devono restare classificati:
| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| MANDATO_CLAUDE_FASE3A_CONTRATTI_20261016.md | richiesta di Luca, preparazione Codex | mandato | conservare per audit della 3A e successiva registrazione documentale |
| SPECIFICA_FASE3A_REGISTRAZIONI.md | esecuzione fase 3A | da produrre, poi pronto/parziale secondo prove | audit Codex; eventuale commit documentale dopo audit e autorizzazione |

Se emergono altre modifiche, non includerle o scartarle automaticamente: classificane origine, stato e destino.
Non dichiarare il worktree pulito se contiene questi due untracked.
Se servono script temporanei per l'analisi, tienili fuori dal repository e dichiara il percorso.
Se non usi una copia esterna, il presente mandato costituisce il mandato di continuità nominato per questi documenti fuori commit.

Consegna:
1. link alla specifica completa;
2. riepilogo di ciò che è verificato, proposto e ancora da decidere;
3. fonti effettivamente lette e prove eseguite/non eseguite;
4. proposta del primo blocco 3B con file/logiche, test e criterio di accettazione;
5. rischi residui e stato Git classificato.

**FASE 3A PRONTA PER AUDIT** solo se esistono la matrice dei contratti, il confronto verificabile con la fonte, i casi numerici e la lista delle vere decisioni residue.
Se mancano fonti essenziali, consegna PARZIALE specificando cosa manca: non inventare le risposte e non nascondere il limite.
La chiusura della 3A non autorizza automaticamente l'implementazione 3B, commit o push.

Formula di chiusura del lavoro documentale:
1. nessun commit/push richiesto da questo mandato; documenti conservati con questo mandato di continuità nominato e destino esplicito;
2. `git status -sb` controllato;
3. nessun tracked modificato senza classificazione;
4. ogni untracked dichiarato come mandato, specifica o output di audit.

Nessuna fase 3B avviata e nessuna produzione contabile toccata.


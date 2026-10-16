# Mandato a Claude — chiusura C17, registrazione e pubblicazione

Data: 2026-10-16.
Repository: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Ruoli: Claude esegue; Codex audita. Applica `programmatore-bravo`.

## 1. Delega e decisioni operative

Luca ha chiesto esplicitamente a Codex: «ci sono delle decisioni da prendere. prendile tu e prepara un md per claude».
Le decisioni seguenti sono assunte da Codex su quella delega. Costituiscono il mandato esecutivo per questo perimetro e sostituiscono le precedenti attese D/E/F del documento di continuità.
Non richiedere nuovamente a Luca un ok sulle stesse azioni già definite qui. Se emergono un impedimento reale o uno scostamento sostanziale, conserva il lavoro e riferisci l'evidenza.

| Decisione | Esito | Limite e motivo |
|---|---|---|
| D — Registrare il lavoro post-audit | SÌ, due commit tematici descritti sotto | C17 supera ora l'audit indipendente; evitare ulteriore accumulo fuori commit |
| E — Pubblicare su GitHub | SÌ, push ordinario di main su origin dopo i controlli | Comprende i tre commit locali esistenti e i due nuovi; nessun force-push |
| F — Rimuovere il guscio vuoto | SÌ, un tentativo ordinario se il percorso è ancora una directory vuota | Se bloccata, registrare RINVIATO_PROCESSO_IN_USO; non blocca commit e push |
| Pulizia fixture C10 | SÌ, ora, intervento MICRO | Dichiarazione XML prima del commento; nessun altro cambiamento semantico |
| Fase 3 e altri debiti | RINVIATI | Nessuno sviluppo contabile aggiuntivo in questa chiusura |

Il mandato resta valido nelle eventuali riprese fino al completamento di queste azioni; non autorizza interventi su altri progetti, cancellazioni ulteriori o nuove regole di dominio.

## 2. Audit indipendente concluso da Codex

Baseline: HEAD `df2db06886c1abcfd9eabc47fa64f0e72f4c57dc`, main ahead 3 sul riferimento locale origin/main `ad53556`.
Il riferimento remoto effettivo va riletto da Claude prima del push.

Verifiche ripetute da Codex sul working tree consegnato da Claude:
- test mirato `cargo test --locked infnfe_aninhado`: superato;
- `cargo test --locked`: **113 passati, 0 falliti** (97 lib + 3 bin + 13 CLI);
- `cargo fmt --check`: exit 0;
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0;
- `git diff --check`: exit 0;
- stessa fixture indipendente che aveva riprodotto B1: ora exit **1**, messaggio con **2 <infNFe>**, stdout di **0 byte**;
- cinque YAML e XML canonico: **6/6 SHA-256 invariati** rispetto al manifesto della salvaguardia;
- un solo worktree, nessuno stash, niente staged.

**Esito: A1 e B1 chiusi rispetto al contratto C17 dichiarato.** Il conteggio ricorsivo include ora i nodi annidati.
Non è un'attestazione di validazione XSD, di XML generico completo o di idoneità alla produzione contabile.
I debiti di dominio già dichiarati restano aperti.

Evidenze esterne, da preservare:
`C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\16\01a144b0-273a-7840-931d-94b0df795d2b\audit_c17_finale`.
I precedenti audit in `audit_stato` e `audit_c17` sono fotografie storiche: non sovrascriverli.

## 3. Perimetro e contratti

Lavoro MIRATO di chiusura; la sola fixture C10 è una correzione MICRO.
Il lavoro corrente basta per C17: nessun'altra modifica al parser è richiesta da questo mandato.

| Contratto | Regola verificabile | Prova di accettazione |
|---|---|---|
| C17 | Più radici o più infNFe nel namespace NF-e, anche annidati, producono errore senza successo parziale | Regressioni unitarie e CLI verdi; stdout senza dati di fattura |
| C10 | Senza fat l'importo resta assente, due rate conservate; non diventa zero o somma delle rate | NAO_DISPONIVEL, FATURA_AUSENTE, due rate, exit 0 |
| Fixture C10 | XML ben formato con dichiarazione prima del commento; difetto intenzionale limitato alla rimozione di fat | Parsing XML indipendente riuscito e non regressione C10 |
| Provenienza | Cinque YAML e campione canonico intatti | Hash SHA-256 invariati |
| Git | Due commit tematici, nessun file perso o senza destino; pubblicazione ordinaria | Worktree pulito e SHA main remoto uguale a HEAD dopo push |
| Directory residua | Eliminabile soltanto la directory esatta, vuota e non un collegamento | Assenza del percorso oppure residuo esplicito non bloccante |

File/logiche da toccare: sola intestazione di `tests/fixtures/nfe_sem_fat.xml` e aggiornamento del mandato di continuità; conservare le modifiche C17 già verificate.
Rischi: staging incompleto dei test untracked, documento di stato nuovamente stale, remoto avanzato, directory bloccata. I controlli sotto li rendono espliciti.
Esclusi: regole YAML, campione canonico, dipendenze, Python, database, alias, C13, modello di copertura Dado, XSD, fase 3, chiusura forzata di applicazioni.

## 4. Esecuzione minima

### A. Inventario iniziale

Prima di operare e prima di ogni commit/push:
```powershell
git status -sb
git diff --name-status
git diff --cached --name-status
git ls-files --others --exclude-standard
```

Confronta con la tabella del punto 5. Se trovi differenze estranee, classificale e preservale senza includerle nei commit di questo mandato.
Non usare `git add .`, reset, restore, checkout, clean, stash, amend, rebase o riscritture della storia.
Rileggi il remote: fetch/push devono puntare a `https://github.com/lucamaggi-ita/Rust_registrazioni_contabili.git`.

### B. Correzione MICRO della fixture C10

In `tests/fixtures/nfe_sem_fat.xml` sposta la dichiarazione `<?xml version="1.0" encoding="UTF-8"?>` in testa, prima del commento.
Mantieni il commento di derivazione e tutto il contenuto della fattura invariato. Nessuna modifica al parser per accomodare la fixture.

Verifica con un parser XML indipendente, per esempio XmlReader .NET in modalità Document:
- il documento è ben formato;
- fat resta assente, le due dup e gli importi da 17.50 restano presenti;
- la CLI restituisce exit 0, NAO_DISPONIVEL, due rate e FATURA_AUSENTE.
Confronta il diff: solo riposizionamento dell'intestazione.
Non servono nuovi test permanenti che replichino questo semplice spostamento; riusa le regressioni C10 esistenti e registra la prova XML indipendente.

Esegui dopo questa ultima modifica funzionale ai test:
```powershell
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
git diff --check
```
Il numero atteso resta 113; se differisce, spiega il motivo e riporta il numero reale. Non aggiustare conteggi per far sembrare verde un controllo.

### C. Directory obsoleta: tentativo limitato e indipendente

Unico bersaglio autorizzato:
`C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_schede_tecniche`.

1. Verifica il percorso assoluto esatto e che non sia un reparse point/collegamento.
2. Se assente, registra GIÀ_ASSENTE.
3. Se contiene anche un solo elemento, non cancellare nulla: registra BLOCCATO_CONTENUTO_INATTESO.
4. Se è vuota, tenta la sola rimozione non ricorsiva, per esempio `[IO.Directory]::Delete($percorsoVerificato, $false)`.
5. Se il sistema segnala un processo in uso, registra RINVIATO_PROCESSO_IN_USO e prosegui la chiusura Git.

Nessun `-Recurse`, nessuna cancellazione forzata, nessuna chiusura/killing di processi, nessuna installazione di strumenti per gli handle.
Non attribuire l'handle a VS Code senza una prova. Non è necessario identificarlo prima di un normale tentativo su directory vuota.
La salvaguardia `_SALVAGUARDIA_contabile_20261015` e il suo manifesto restano intatti.

### D. Due commit tematici

**Commit 1:** `fix(nfe): applica C17 e corregge la fixture C10`

Staging nominativo, soltanto:
- `src/nfe.rs`
- `README.md`
- `tests/cli_importar_nfe.rs`
- `tests/fixtures/nfe_duas_raizes.xml`
- `tests/fixtures/nfe_infnfe_annidata.xml`
- `tests/fixtures/nfe_sem_fat.xml`

Il commento A3 in nfe.rs appartiene allo stesso contratto di lettura NF-e.
Tutti i test e le fixture da cui dipende il codice devono essere presenti in questo commit.
Prima del commit controlla il diff staged e classifica i documenti che restano fuori per il commit 2.

**Commit 2:** `docs: chiude audit C17 e aggiorna il mandato di continuita`

Staging nominativo, soltanto:
- `config/FONTE.md`
- `MANDATO_CONTINUITA_20261016.md`
- `MANDATO_CLAUDE_CHIUSURA_C17_20261016.md` (questo documento)

Prima di questo commit aggiorna il mandato di continuità:
- audit indipendente B1 superato, con evidenze e perimetro C17;
- fixture C10 corretta e verificata;
- D/E autorizzati dalla delega di Luca registrata qui, non più MAI DATA;
- F nello stato effettivamente osservato;
- distinzione fra fotografia storica, situazione corrente e azioni ancora da eseguire;
- nessun residuo chiuso senza prova, nessuna fase 3 avviata.

Evita il ciclo di nuovi commit per aggiornare l'hash del documento che sta per essere committato:
usa una baseline datata e rimanda a Git per HEAD/stato live. Non scrivere in anticipo che il push è già riuscito.
Il consuntivo finale con SHA e stato remoto va nel resoconto di consegna; non creare un terzo commit soltanto per rendere autoreferenziale il mandato.

### E. Push autorizzato

Dopo i due commit, verifica che tutti i file pertinenti siano registrati e che il worktree sia pulito.
Rileggi il remoto con `git fetch origin` e controlla che origin/main sia antenato di main.
Spiega tutti i commit locali: i tre esistenti (`7f87a63`, `b9a4995`, `df2db06`) più i due di questo mandato, salvo che qualcuno sia già arrivato al remoto.
Se il remoto è divergente, conserva i commit, non fare force-push e non risolvere automaticamente con merge/rebase: riporta il blocco concreto.

Se i controlli passano, esegui `git push origin main`.
Poi rileggi `refs/heads/main` da origin e verifica che il suo SHA sia uguale a `git rev-parse HEAD`.
Non dichiarare pubblicato sulla sola base del riferimento locale origin/main.

## 5. Classificazione del lavoro alla consegna a Claude

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| src/nfe.rs | A1+B1, C17 e commento A3 | audit Codex superato | commit 1 |
| README.md | C17 e conteggio 113 | verificato | commit 1 |
| tests/cli_importar_nfe.rs (untracked) | regressioni C17 | 3 test verdi | commit 1 |
| tests/fixtures/nfe_duas_raizes.xml (untracked) | fixture A1 | verificata | commit 1 |
| tests/fixtures/nfe_infnfe_annidata.xml (untracked) | fixture B1 | verificata | commit 1 |
| tests/fixtures/nfe_sem_fat.xml (ora invariato rispetto a HEAD) | pulizia MICRO autorizzata qui | da correggere e verificare | commit 1 dopo correzione |
| config/FONTE.md | A3 documentale | verificato | commit 2 |
| MANDATO_CONTINUITA_20261016.md | A2+B2 | da riallineare a questo audit/mandato | commit 2 |
| MANDATO_CLAUDE_CHIUSURA_C17_20261016.md (nuovo) | decisioni delegate da Luca | mandato esecutivo | commit 2 |

Le patch nominate e le copie di audit sono esterne al repository; non aggiungerle ai commit.
Nessun file può essere scartato o lasciato senza destino. Un impedimento su un tema non autorizza ad abbandonare gli altri lavori eseguibili.

## 6. Consegna e chiusura

Consegna un solo resoconto con:
1. modifiche e contratti verificati, compresa la prova XML della fixture C10;
2. esito dei controlli finali, hash invariati delle sei fonti;
3. SHA, oggetto e file dei due commit nuovi;
4. esito del push e confronto SHA locale/remoto;
5. esito distinto della directory, senza attribuzioni non verificate;
6. stato Git finale e classificazione di eventuali residui;
7. rischi di dominio ancora aperti, senza aprire nuove lavorazioni.

FASE CHIUSA solo se:
1. commit/push necessari completati oppure, in caso di impedimento reale, patch nominata creata e pubblicazione dichiarata NON_COMPLETATA;
2. `git status -sb` controllato;
3. nessun file tracked modificato resta senza classificazione;
4. eventuali untracked sono dichiarati come output temporanei, audit, patch o file da ignorare.

Distingui l'esito: C17 verificato; registrazione Git; pubblicazione GitHub; residuo cartella.
La cartella bloccata non impedisce la chiusura di C17/Git/pubblicazione, se è rimasta vuota e il suo stato è dichiarato.
La chiusura di questo mandato non avvia la fase 3 e non è una promozione in produzione contabile.


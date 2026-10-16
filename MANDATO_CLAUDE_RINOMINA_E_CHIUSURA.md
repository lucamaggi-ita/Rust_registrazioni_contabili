# Mandato a Claude: rinomina completa del progetto contabile e chiusura del residuo dati mancanti

## Ruoli e priorita' dell'utente

Claude esegue il lavoro. Codex ne audita il risultato e non modifica il progetto.

Il lavoro avviato e' il porting Rust di **Fantozzi / registrazioni contabili**. E' stato inserito per errore nel repository e nella cartella destinati alle schede tecniche.

**Prima priorita': rinominare completamente il progetto contabile, su GitHub e in locale, senza pregiudizio per codice, cronologia e modifiche non committate.** La correzione dei bug non sostituisce questa richiesta.

Se la rinomina senza perdita non puo' essere garantita, l'utente preferisce eliminare il progetto avviato per errore. Questa preferenza non autorizza una cancellazione automatica: in caso di impedimento concreto, conserva lo stato e sottoponi a Luca gli esatti bersagli locali/remoti prima di qualsiasi eliminazione.

Non limitarti a proporre nuovamente la rinomina: procedi alla sistemazione reversibile richiesta, se gli strumenti e i permessi disponibili lo consentono. Segnala eventuali impedimenti reali con evidenze. Non sostituire questo mandato con ulteriore sviluppo contabile.

## 1. Identita' da ottenere

| Elemento | Stato attuale verificato | Destinazione |
|---|---|---|
| Repository GitHub | `lucamaggi-ita/Rust_schede_tecniche` | `lucamaggi-ita/Rust_registrazioni_contabili` |
| Cartella locale | `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_schede_tecniche` | `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili` |
| Package Cargo / crate | `rust_schede_tecniche` | `rust_registrazioni_contabili` |
| Origin fetch/push | `https://github.com/lucamaggi-ita/Rust_schede_tecniche.git` | `https://github.com/lucamaggi-ita/Rust_registrazioni_contabili.git` |
| Titolo e descrizione GitHub/README | Schede tecniche, con contenuto contabile | Fantozzi / registrazioni contabili in Rust |

Aggiorna anche riferimenti alla crate, percorsi e nome dell'eseguibile nei test, inclusi gli usi di `CARGO_BIN_EXE_rust_schede_tecniche`. Cerca gli altri riferimenti prima di modificarli. Le vecchie denominazioni possono restare nei documenti storici solo se chiaramente etichettate come precedenti.

Non modificare gli URL del repository Python canonico nei documenti di provenienza. Non fare sostituzioni globali indiscriminate su `schede_tecniche`: il vero progetto Python delle schede tecniche e' un prodotto distinto.

Verifica i collegamenti al progetto nell'app e negli strumenti usati. Se un collegamento non e' aggiornabile con gli strumenti disponibili, dichiaralo come residuo, senza affermare che tutto e' stato riallineato.

Non creare ora il nuovo porting delle schede tecniche. La sua fonte futura sara' `programma-schede-tecniche`; questo mandato riguarda la sistemazione del contabile esistente.

## 2. Stato di partenza: non e' un worktree pulito

Ultimo controllo di Codex:

```text
HEAD: ad53556 — fase 1 contabile, gia' pubblicata
Branch: main, allineato al riferimento locale origin/main
Staged: nessuno
Tracked modificati: 6
Voci untracked nel riepilogo Git: 5
```

Ripeti il controllo prima di operare: non assumere che questa fotografia sia ancora attuale.

```powershell
git status -sb
git diff --name-status
git diff --cached --name-status
git remote -v
git log -5 --oneline --decorate
git ls-files --others --exclude-standard
```

| File | Origine/mandato | Stato all'ultimo audit | Destino |
|---|---|---|---|
| `Cargo.toml` | Fase 2 Claude, dipendenza e identita' package | Modificato | Salvaguardia completa; rinomina package preservando le dipendenze |
| `Cargo.lock` | Fase 2 Claude, dipendenze | Modificato | Salvaguardia; aggiornamento coerente del package senza aggiornamenti generalizzati delle dipendenze |
| `README.md` | Contratti e resoconto fase 2 Claude | Modificato, residuo C10 | Salvaguardia; riallineamento identita' e contratto dati mancanti |
| `src/lib.rs` | Esposizione dei moduli fase 2 Claude | Modificato | Salvaguardia; conservare nella rinomina |
| `src/main.rs` | Comandi e validazione fase 2 Claude | Modificato | Salvaguardia; aggiornare crate e output dei dati assenti |
| `src/regras.rs` | Caricamento e avvisi C13 Claude | Modificato | Salvaguardia; conservare nella rinomina |
| `src/classificacao.rs` | Classificatore e audit alias Claude | Untracked | Includere nel salvataggio completo; conservare |
| `src/nfe.rs` | Parser e correzioni Claude | Untracked, residuo aperto | Includere nel salvataggio; correggere dopo la sistemazione dell'identita' |
| `src/normalizacao.rs` | Normalizzazione Claude | Untracked | Includere nel salvataggio completo; conservare |
| `tests/cli_configuracao_obrigatoria.rs` | Sette regressioni CLI Claude | Untracked | Includere nel salvataggio completo; conservare |
| `samples/FONTE.md` e XML di esempio | Campione copiato dalla fonte Python | Untracked | Includere nel salvataggio; conservare campione e provenienza |
| Questo file Markdown | Mandato di consegna chiesto dall'utente | Nuovo documento | Conservare come mandato/audit, classificato separatamente dal codice |

Nessuno di questi file puo' essere scartato implicitamente.

## 3. Esecuzione della rinomina senza pregiudizio

1. **Inventario e salvaguardia completa, prima della rinomina.** Salva cronologia Git, diff tracked e tutti gli untracked pertinenti in una destinazione esterna alla cartella da rinominare. Registra percorso, elenco file e hash, e verifica che la copia sia leggibile. Una patch `git diff` non include gli untracked; un `git bundle` non include il working tree: nessuno dei due, da solo, e' un backup completo dello stato corrente. `target/` puo' essere escluso se confermato come solo output rigenerabile.
2. **Verifica le destinazioni.** Controlla che il nome GitHub e il percorso locale di destinazione siano disponibili. Non sovrascrivere, unire o eliminare un progetto omonimo gia' esistente. In caso di collisione, conserva tutto e riferisci l'impedimento.
3. **Rinomina il repository GitHub esistente.** Mantieni la stessa storia e gli altri oggetti del repository. Non sostituirlo creando un nuovo repository vuoto e cancellando il vecchio. Controlla il nome effettivo restituito da GitHub: il fatto che il vecchio URL continui a rispondere non prova che `origin` sia corretto.
4. **Rinomina la cartella locale preservando `.git` e l'intero working tree.** Gestisci i processi e le sessioni che possono bloccare il percorso su Windows; esegui la rinomina dal percorso padre e riprendi il lavoro dal nuovo percorso. Verifica il percorso assoluto di partenza e di arrivo prima dell'operazione.
5. **Riallinea origin, package, import, test e documentazione.** Mantieni le versioni delle dipendenze; aggiorna il lockfile solo quanto necessario al cambio del nome del package. Non usare un aggiornamento generalizzato delle dipendenze per risolvere la rinomina.
6. **Verifica la preservazione.** Confronta HEAD/storia e inventario con la salvaguardia. Ogni differenza nei contenuti deve appartenere alle modifiche dichiarate di rinomina; nessun file della fase 2 puo' sparire. Poi esegui build, test, formattazione e Clippy dal nuovo percorso e verifica che gli errori dei vecchi percorsi non siano mascherati.

Non usare `git add .`, `reset`, `restore`, `checkout`, `clean` o `stash` come scorciatoia. Non cancellare la cartella di salvaguardia. Questo mandato non richiede nuovi commit o push di codice: prepara un risultato verificabile e una classificazione Git completa. Un'eventuale proposta di commit/push va distinta dalla rinomina del repository GitHub.

Prima di modificare, applica `programmatore-bravo` e dichiara problema, decisione, contratto, file, test, audit, rischi e perimetro escluso. Se una modifica non e' pronta per commit, preservala con patch nominata e mandato di continuita', includendo separatamente gli untracked nel salvataggio.

## 4. Solo dopo la sistemazione dell'identita': chiudere il residuo sui dati mancanti

L'ultimo audit ha confermato **100 test superati**, formattazione e Clippy puliti. Tre riscontri della fase 2 sono chiusi:

- entita' XML e CDATA preservati;
- validazione condivisa dei comandi e avvisi C13 visibili;
- audit veritiero dei 571 alias: 55 regole, 7 destinazioni divergenti e 11 risultati ambigui.

Non riaprire o riscrivere queste parti senza una regressione concreta.

**Il riscontro "mancante diverso da zero" resta parzialmente aperto.** L'opzionalita' di una tag non autorizza un valore numerico zero per ogni assenza.

Prova riprodotta da Codex sul campione canonico, tramite una copia esterna:

```text
Rimuovere <fat>...</fat>, lasciando <cobr> e le due rate da 17.50.

Risultato attuale:
tem_cobranca = true
valor_cobranca = 0
duplicatas = 2
avisos = []

CLI:
Cobranca : 0 | 2 duplicata(s)
Nenhum aviso de leitura.
EXIT=0
```

Rimuovendo soltanto `vLiq`, compare l'avviso `VALOR_OPCIONAL_AUSENTE_ZERADO`, ma il dato pubblico resta comunque `0`. Il README ha indebolito il contratto in "mai zero in silenzio": non e' equivalente alla regola dell'utente.

### Contratto richiesto

- Valore obbligatorio mancante: mantenere l'errore esplicito, con fonte e percorso del campo. Non trasformarlo in un semplice avviso.
- Importo facoltativo assente: rappresentare l'assenza nel modello, per esempio con `Option` o `Dado` e stato/motivo; nell'output usare `MANCANTE`, `NON_DISPONIBILE` o equivalente esplicito.
- Trattare distintamente `cobr` assente, `cobr` senza `fat`, `fat` senza `vLiq` e `vLiq` esplicitamente a zero.
- `valor_cobranca = 0` deve restare possibile quando la fonte dichiara davvero zero.
- Non inventare l'importo sommando le rate o ricostruendolo da altri campi come fallback implicito.
- Applicare la stessa distinzione agli altri valori facoltativi e agli aggregati che ne dipendono. Un aggregato incompleto non deve sembrare completo.
- Se per uno specifico campo fiscale l'assenza significa davvero non applicabilita' e consente uno zero derivato, serve una regola di dominio con fonte, stato e test. Non introdurre una regola generale "tag opzionale assente = zero".

Parti coinvolte: modello `Nfe`, lettura in `nfe.rs`, output CLI, serializzazione, test e README. Mantieni il diff mirato. Non cambiare le regole canoniche YAML per chiudere questo difetto.

### Prove minime di regressione

| Caso | Esito da verificare |
|---|---|
| `cobr` assente | Assenza distinta da un importo zero |
| `cobr` presente, `fat` assente, rate presenti | Importo di fatturazione assente, non inventato; rate conservate |
| `fat` presente, `vLiq` assente | Assenza rappresentata nel dato e nell'output |
| `vLiq` esplicitamente `0.00` | Zero reale conservato |
| Campione canonico con `vLiq=35.00` | 35.00, due rate da 17.50, tre prodotti; nessuna regressione |
| Valore obbligatorio mancante | Errore con percorso, senza zero tecnico |
| Round-trip del modello con importo assente | Assenza conservata, nessun default numerico |

Non modificare il campione canonico per far passare i test: genera fixture separate e dichiarate.

## 5. Perimetro escluso

- Nessuna modifica ai progetti Python originali o ai database operativi.
- Nessuna variazione delle classificazioni canoniche, delle priorita', delle regole per fornitore o della politica sugli alias brevi. Le due decisioni restano aperte e separate.
- Nessuna fase 3, DRE, scrittura contabile o nuovo porting delle schede tecniche.
- Nessuna cancellazione, riscrittura della storia o force-push.
- Nessuna dichiarazione "fase chiusa" basata soltanto sul numero di test superati.

## 6. Consegna da far auditare a Codex

Fornisci un resoconto concreto con:

1. URL GitHub effettivo, nuovo percorso locale e origin fetch/push.
2. Nome del package e ricerca degli eventuali riferimenti residui al vecchio nome, distinguendo storia e riferimenti attivi.
3. Percorso della salvaguardia completa e prova che commit e tutti i file non committati sono stati preservati.
4. Modifiche effettuate, contratti implementati e eventuali scostamenti dal mandato.
5. Esiti di `cargo test --locked`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, eseguiti dal nuovo percorso dopo il necessario riallineamento del lockfile.
6. Evidenze dei casi di dato mancante della tabella, incluso l'output CLI senza `fat`.
7. Conferma degli hash dei cinque YAML e del campione canonico invariati.
8. `git status -sb`, `git diff --name-status`, `git diff --cached --name-status` e tabella **File / Origine-mandato / Stato / Destino** per tutti i tracked modificati e gli untracked.
9. Indicazione esplicita di eventuali commit/push realmente effettuati; non presentarli come avvenuti se soltanto proposti. Non eseguire push di codice sulla base di questo documento.

Se la rinomina non riesce senza pregiudizio, non continuare lo sviluppo sotto l'identita' errata: riferisci il blocco concreto e conserva tutto per la decisione di Luca sull'eliminazione.

## Evidenze dell'audit disponibili

Cartella degli audit di Codex:

```text
C:\Users\Recanto do Sossego\.codex\visualizations\2026\10\15\01a140dd-66a7-71e0-829a-8342fff22bab\review_claude
```

Riferimenti principali: `AUDIT_DISALLINEAMENTO_PROGETTO.md`, `VERIFICA_CORREZIONI_FASE2.md`, `probe_fase2_correzioni.rs`, `probe_fase2_correzioni.log`, `fase2_sem_fat.xml`, `fase2_sem_fat_cli.log`. Le vecchie patch e probe sono fotografie storiche: non applicarle automaticamente e non considerarle backup completi degli untracked.

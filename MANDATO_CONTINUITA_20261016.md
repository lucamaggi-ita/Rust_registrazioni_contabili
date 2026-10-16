# Mandato di continuita' — 2026-10-16

Documento di passaggio per una sessione nuova aperta in
`C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.

Non e' un riassunto di cortesia: contiene lo stato verificato, i divieti attivi
e le decisioni ancora in mano a Luca. Leggilo prima di toccare qualunque file.

**Seconda revisione, baseline documentale 2026-10-16, dopo la chiusura
dell'audit C17.** Questo documento non insegue i propri hash: per HEAD,
divergenza e stato del worktree **leggi Git** (`git status -sb`,
`git rev-parse HEAD`, `git log --oneline`), non questo testo. Qui resta cio'
che Git non dice: decisioni, autorizzazioni, perimetri e residui.

Tre strati, da non confondere:

| Strato | Che cos'e' |
|---|---|
| Fotografia storica | il mandato originale e le revisioni precedenti di questo file, conservate nella storia Git |
| Situazione corrente | i punti 1-4 e 6-7, aggiornati a questa revisione |
| Azioni ancora da eseguire | solo quelle elencate al punto 5 come aperte |

Documenti correlati nello stesso percorso:

- `MANDATO_CLAUDE_RINOMINA_E_CHIUSURA.md` — mandato originale (rinomina + residuo C10).
- `MANDATO_CLAUDE_CHIUSURA_C17_20261016.md` — mandato esecutivo di chiusura C17,
  registrazione e pubblicazione, con le decisioni D/E/F delegate da Luca.
- `README.md` — contratti implementati, verifica, rischi noti.

---

## 1. Identita' del progetto: chiusa e verificata

Questo e' il porting Rust di **Fantozzi / registrazioni contabili** (NF-e di un
ristorante in Brasile). Era stato avviato per errore nel repository e nella
cartella delle schede tecniche. Il 2026-10-15 identita' GitHub, cartella locale
e crate sono state rinominate; il 2026-10-16 un audit indipendente di Codex ha
confermato la rinomina senza perdita.

| Elemento | Valore attuale |
|---|---|
| Repository GitHub | `lucamaggi-ita/Rust_registrazioni_contabili` (stesso repository ID, vecchio endpoint in Moved Permanently) |
| Cartella locale | `...\GitHub\RUST\Rust_registrazioni_contabili` |
| Package / crate | `rust_registrazioni_contabili` |
| Origin fetch/push | `https://github.com/lucamaggi-ita/Rust_registrazioni_contabili.git` |
| HEAD e divergenza | **da leggere da Git**, non da qui. Baseline documentale: `df2db06` + i due commit di chiusura C17, pubblicazione su `origin/main` autorizzata dal mandato di chiusura |

**Il nome `Rust_schede_tecniche` sopravvive solo in due posti legittimi:** il
riquadro storico del README e i documenti di mandato/audit. Non ci sono
riferimenti attivi in codice, test o Cargo. Non fare sostituzioni globali su
`schede_tecniche`: il vero progetto Python delle schede tecniche e' un prodotto
distinto, e il suo porting futuro avra' come fonte `programma-schede-tecniche`.

---

## 2. Stato tecnico: fase 1 e fase 2 pronte, fase 3 non iniziata

Verifiche rieseguite il 2026-10-16 da questo percorso, sia da Claude sia da
Codex in modo indipendente:

```text
cargo test --locked                                      113 passati, 0 falliti
                                                         (97 lib + 3 bin + 13 CLI)
cargo fmt --check                                        pulito
cargo clippy --locked --all-targets -- -D warnings       pulito
```

I quattro riscontri della fase 2 sono **chiusi**: entita' XML e CDATA
preservati, validazione condivisa dei comandi con avvisi C13 su stderr, audit
veritiero dei 571 alias, e il residuo C10 "mancante diverso da zero".

### C17 — un documento, una NF-e: implementato, verificato, con un residuo

Riscontro **A1** dell'audit indipendente del 2026-10-16: un XML con due radici
(due documenti concatenati) usciva con codice 0, leggeva la sola NF-e 12566 e
scartava la seconda in silenzio. Contratto dichiarato: piu' radici XML o piu'
`<infNFe>` sono errore con uscita 1, mai successo parziale.

Lo stato va letto separando implementazione, verifica e residui:

| Caso | Implementato | Verificato da Codex | Esito |
|---|---|---|---|
| Due radici (A1) | si | si | exit 1, stdout senza dati |
| Lotto `enviNFe` con due NF-e sorelle | si | si | exit 1, stdout senza dati |
| `<infNFe>` annidato in un altro (**B1**) | si | **si**, audit del 2026-10-16 | exit 1, messaggio con `2 <infNFe>`, stdout di 0 byte |
| Campione canonico e stati C10 | invariati | si | nessuna regressione, 6/6 hash invariati |

**A1 e B1 sono chiusi rispetto al contratto C17 dichiarato**, con audit
indipendente concluso. Evidenze esterne conservate in
`...\.codex\visualizations\2026\10\16\01a144b0-...\audit_c17_finale`; gli audit
precedenti (`audit_stato`, `audit_c17`) sono fotografie storiche da non
sovrascrivere.

B1 era un difetto della mia prima patch: `contar_descendentes` si fermava sul
primo nodo trovato e non scendeva sotto di lui, quindi un `infNFe` annidato
contava come uno. Corretto contando ogni occorrenza.

**Validazione XSD esplicitamente fuori contratto.** C17 impone un documento e
una NF-e, non la conformita' al layout: un XML ben formato ma fuori layout
viene rifiutato dal conteggio, non da uno schema.

Il lavoro C17 e' **registrato** nel commit `7f0b824`, insieme alla pulizia
della fixture C10 (vedi 4).

**Fixture C10 corretta.** In `tests/fixtures/nfe_sem_fat.xml` la dichiarazione
XML stava dopo il commento di derivazione, quindi `XmlReader` in modalita'
Document la rifiutava: la fixture portava un difetto XML in piu' rispetto al
difetto voluto. Dichiarazione spostata in testa, diff di sola intestazione.
Verificato con `XmlReader` e `XmlDocument` .NET: documento ben formato, `<fat>`
assente, due `<dup>` da 17.50, `nNF` 12566, `vNF` 35.00, primo nodo
`XmlDeclaration`. La CLI resta `NAO_DISPONIVEL` + `FATURA_AUSENTE`, exit 0.
Nessuna modifica al parser per accomodare la fixture.

### C10: contratto in vigore, non riaprirlo senza una regressione concreta

`valor_cobranca` e `valor_tributos_inclusos` usano `Dado<T>` (valore, fonte,
stato, motivo). Gli obbligatori mancanti restano **errori** con il percorso del
campo, non avvisi. Comportamento verificato:

| Input | Valore | Stato | Output CLI |
|---|---|---|---|
| `cobr` assente | `None` | `NaoAplicavel` | `N.A. ...` |
| `cobr` presente, `fat` assente | `None` | `NaoDisponivel` | `NAO_DISPONIVEL (...)` + aviso `FATURA_AUSENTE` |
| `fat` presente, `vLiq` assente | `None` | `Mancante` | `MANCANTE (...)` + aviso `VALOR_FATURA_AUSENTE` |
| `vLiq` dichiarato `0.00` | `Some(0.00)` | `Ok` | `0.00` |
| Campione canonico | `Some(35.00)` | `Ok` | `35.00`, due rate da 17.50 |

Regole da non violare: l'importo **non** si ricostruisce sommando le rate; un
aggregato con componenti assenti e' `Parcial` ed emette `AGREGADO_PARCIAL`;
nessuna regola generale "tag opzionale assente = zero".

### Campione canonico: intoccabile

`samples/nfe_exemplo/nfe_oesa_12566_20260505.xml` e i cinque YAML di `config/`
hanno hash SHA-256 identici alla salvaguardia e alla fonte Python. Non
modificarli per far passare un test: le fixture volutamente difettose stanno in
`tests/fixtures/` e vanno dichiarate.

---

## 3. Divieti attivi

Ogni autorizzazione vale **solo per il suo perimetro** e si esaurisce con
l'azione che copre. Un audit non crea autorizzazioni: le due autorizzazioni
operative (D, E) nascono dalla delega di Luca a Codex del 2026-10-16,
registrata in `MANDATO_CLAUDE_CHIUSURA_C17_20261016.md`.

| Autorizzazione data da Luca il 2026-10-16 | Perimetro | Stato |
|---|---|---|
| Registrare fase 2 + rinomina + C10 in commit atomici | i tre commit `7f87a63`, `b9a4995`, `df2db06` | CONSUMATA |
| Versionare i due mandati | commit `df2db06` | CONSUMATA |
| Eliminare la cartella `Rust_schede_tecniche` | solo quella cartella | CONSUMATA (parziale, vedi 6) |
| Registrare in Git il lavoro post-audit (D) | i due commit di chiusura C17: `fix(nfe)` e `docs` | DATA il 2026-10-16 da Luca, delegata a Codex e scritta in `MANDATO_CLAUDE_CHIUSURA_C17_20261016.md` |
| Push ordinario di `main` su `origin` (E) | push ordinario, **nessun force-push**, nessuna riscrittura | DATA con la stessa delega |
| Rimuovere il guscio vuoto della vecchia cartella (F) | un tentativo ordinario, solo se directory vuota e non reparse point | DATA con la stessa delega |
| Pulizia della fixture C10 | solo riposizionamento della dichiarazione XML | DATA con la stessa delega |
| Fase 3, nuove regole di dominio, altre cancellazioni | — | **MAI DATA** |

Divieti che restano in vigore:

- **Nessun commit e nessun push** senza autorizzazione esplicita di Luca per
  ogni lavoro **successivo alla chiusura C17**: le autorizzazioni D ed E
  coprono quei due commit e quel push, non il lavoro che verra' dopo.
- **Nessuna cancellazione** della salvaguardia
  `...\GitHub\RUST\_SALVAGUARDIA_contabile_20261015` ne' del suo manifesto.
- Nessun `git add .`, `reset`, `restore`, `checkout`, `clean`, `stash`,
  riscrittura della storia o force-push.
- Nessuna modifica ai progetti Python originali o ai database operativi.
- Nessuna variazione delle regole canoniche YAML, delle priorita', delle regole
  per fornitore o della politica sugli alias brevi.
- Nessuna fase 3 (lanciamenti, pre-libro diario, DRE, report) e nessun porting
  delle schede tecniche.
- Non dichiarare "fase chiusa" sulla base del solo numero di test superati.

---

## 4. Stato Git e classificazione dei file

Fase 2 + rinomina + C10 sono **registrati**, con autorizzazione di Luca e
senza push:

| Commit | Tema | Contenuto |
|---|---|---|
| `7f87a63` | dati | `samples/FONTE.md` + campione canonico XML (2 file) |
| `b9a4995` | codice | commit atomico, 13 file, +2913/-79: rinomina crate, fase 2, C10, README, test, fixture |
| `df2db06` | documenti | i due mandati (420 righe) |

Il commit intermedio `7f87a63` e' stato verificato in un worktree temporaneo
(42 test verdi) prima di procedere: nessun commit intermedio rotto.

La chiusura C17 aggiunge due commit tematici:

| Commit | Tema | Contenuto |
|---|---|---|
| `7f0b824` | codice | C17 (A1, lotto, B1), commento A3, 3 regressioni CLI, 2 fixture nuove, pulizia della fixture C10, conteggio test nel README |
| (commit `docs` di questa revisione) | documenti | `config/FONTE.md`, questo file, il mandato di chiusura C17. L'hash e' in Git: `git log --oneline -3` |

La pubblicazione su `origin/main` e' autorizzata dalla decisione E. **Lo stato
effettivo del push va letto da Git e dal remoto**, non da questo documento: il
resoconto di consegna riporta SHA locale e remoto confrontati.

Ripeti sempre il controllo prima di operare, non fidarti di questa fotografia:

```bash
git status -sb
git diff --name-status
git diff --cached --name-status
git ls-files --others --exclude-standard
```

Classificazione alla chiusura C17 (destino di ogni file toccato):

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/nfe.rs` | A1 + B1: contratto C17; commento A3 | audit indipendente superato | REGISTRATO in `7f0b824` |
| `README.md` | riga C17 + conteggio test 113 (B2) | coerente col codice | REGISTRATO in `7f0b824` |
| `tests/cli_importar_nfe.rs` | regressioni CLI C17 (A1, B1, caso buono) | 3 test verdi | REGISTRATO in `7f0b824` |
| `tests/fixtures/nfe_duas_raizes.xml` | fixture dichiarata di A1 | riproduceva il difetto pre-fix | REGISTRATO in `7f0b824` |
| `tests/fixtures/nfe_infnfe_annidata.xml` | fixture dichiarata di B1 | riproduceva il difetto pre-fix | REGISTRATO in `7f0b824` |
| `tests/fixtures/nfe_sem_fat.xml` | pulizia MICRO autorizzata | ben formato, difetto limitato alla rimozione di `<fat>` | REGISTRATO in `7f0b824` |
| `config/FONTE.md` | A3 documentale | verificato contro il README | commit `docs` di questa revisione |
| `MANDATO_CONTINUITA_20261016.md` (questo file) | A2 + B2 + chiusura C17 | riallineato | commit `docs` di questa revisione |
| `MANDATO_CLAUDE_CHIUSURA_C17_20261016.md` | decisioni D/E/F delegate da Luca | mandato esecutivo | commit `docs` di questa revisione |

Patch nominate e copie di audit restano **fuori dal repository**: non vanno
aggiunte ai commit.

Nessun file puo' restare senza destino esplicito. Nessun commit misto con file
non appartenenti allo stesso tema. I 5 YAML di `config/` e il campione
canonico restano invariati: la modifica a `config/FONTE.md` tocca la
documentazione di provenienza, non le regole.

### Salvaguardia dello stato pre-rinomina

`C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\_SALVAGUARDIA_contabile_20261015`
contiene `repo_completo.bundle` (verificato con `git bundle verify`, ref `main`
al vecchio HEAD `ad53556`), `tracked_modificati_20261015.patch`,
`working_tree_completo` e `manifesto_sha256.txt`. Codex ha confrontato 23 file
del manifesto e 21 file del proprio snapshot precedente: nessun mancante,
nessun divergente. Non eliminarla.

---

## 5. Decisioni: eseguite e ancora aperte

### Eseguite il 2026-10-16 (storico, non ripetere)

**(A) Registrazione in Git: ESEGUITA.** Tre commit invece dei due previsti,
perche' la decisione (B) ha aggiunto i documenti e un commit misto
codice+documenti avrebbe violato la regola sui commit tematici. Ordine
rispettato: prima `samples/`, poi il commit atomico codice+rinomina. I vincoli
che lo imponevano (vecchio `use rust_schede_tecniche::regras`,
`CARGO_BIN_EXE_rust_schede_tecniche`, campione letto da tre punti) erano stati
verificati su HEAD prima di operare. Nessun push.

**(B) I due file di mandato: ESEGUITA.** Versionati nel repository, commit
`df2db06`, separato dal codice.

**(C) Rimozione della cartella residua: ESEGUITA IN PARTE.** Il contenuto di
`...\RUST\Rust_schede_tecniche` e' stato eliminato; la directory vuota resta
perche' un processo ne detiene l'handle (vedi 6). Prima di cancellare era stato
verificato che conteneva un solo file fuori da `.git` e che i due ref extra
(`refs/agents/...`, `refs/codex/turn-diffs/...`) contenevano quello stesso file:
nessun sorgente, configurazione o campione unico. Il marcatore e' stato copiato
in `_SALVAGUARDIA_contabile_20261015\NOTA_CARTELLA_OBSOLETA_20261016.md` con
hash SHA-256 identico verificato; `manifesto_sha256.txt` non e' stato toccato.

**(D) Registrazione del lavoro post-audit: ESEGUITA.** Due commit tematici,
codice e documentazione separati, staging nominativo file per file, come
deciso nel mandato di chiusura C17.

**(E) Pubblicazione su `origin/main`: ESEGUITA secondo la decisione E**, push
ordinario senza force-push. **Lo stato va verificato in Git**, confrontando
`git rev-parse HEAD` con `refs/heads/main` su origin; il resoconto di consegna
riporta quel confronto.

**(F) Guscio vuoto della vecchia cartella: tentativo eseguito, RINVIATO.**
Vedi il punto 6 per lo stato osservato.

### Ancora aperte

Nessuna azione di questo perimetro resta aperta, tranne il guscio vuoto (F),
non bloccante. Tutto il resto e' debito di dominio: punto 7, da non chiudere
di iniziativa propria.

---

## 6. Residuo operativo P2: collegamento dell'app

Il percorso del progetto non era in un `.code-workspace` (non ne esiste
nessuno), ma nello stato globale di VS Code:
`%APPDATA%\Code\User\globalStorage\storage.json`, in
`profileAssociations.workspaces`, `backupWorkspaces.folders` e
`windowsState.lastActiveWindow.folder`.

Il 2026-10-16 e' stata aperta una finestra nuova su questo percorso
(`code --new-window`), e le prime due voci ora contengono
`Rust_registrazioni_contabili`. `storage.json` non e' stato editato a mano:
VS Code lo riscrive alla chiusura e sovrascriverebbe la modifica.

**Residuo:** `windowsState.lastActiveWindow.folder` punta ancora alla vecchia
cartella e si aggiorna soltanto quando VS Code viene chiuso. Finche' la vecchia
finestra resta aperta, il ripristino di sessione riporta al percorso obsoleto.
Azione in mano a Luca: lavorare in questa finestra, chiudere quella vecchia,
riavviare VS Code e verificare il percorso effettivo.

**Fotografia da riconfermare**, come osservato dall'audit: le voci di
`storage.json` citate sopra sono state lette il 2026-10-16 prima dei commit e
non sono state rilette dopo.

**Collegato — stato della directory residua (F), osservato il 2026-10-16:**

| Controllo | Esito |
|---|---|
| Percorso assoluto | `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_schede_tecniche` |
| Attributi | `Directory`, **non** reparse point ne' collegamento |
| Elementi contenuti | **0** |
| `[IO.Directory]::Delete(percorso, $false)` | **RINVIATO_PROCESSO_IN_USO** |

Nessuna cancellazione ricorsiva, nessun processo chiuso, nessuno strumento per
gli handle installato. **L'identita' del processo resta NON_VERIFICATA**: non
attribuirla a VS Code senza prova. Il guscio e' vuoto, dichiarato e non
bloccante: non impedisce la chiusura di C17, della registrazione Git o della
pubblicazione. Si ritenta quando il percorso non e' piu' in uso.

Lato Claude Code non c'e' nulla da correggere: la chiave di progetto deriva
dalla cwd, quindi una sessione avviata qui usa gia' il percorso giusto.

---

## 7. Residui di dominio ancora aperti (non bug)

1. **Doppia fonte di regole**: `config/` e' copia del repo Python, vedi
   `config/FONTE.md`.
2. `serde_yaml` deprecato.
3. **C13**: `regras_por_fornecedor` e `politica_de_match` dichiarate e non
   applicate, oggi visibili come avviso. Decisione pendente.
4. `mapeamento_dre.yaml` e `csv_columns.yaml` versionati ma non letti.
5. **Copertura non modellata in `Dado<T>`**: oggi solo valore, fonte, stato e
   motivo. Debito da chiudere prima di output operativi che dipendano da essa.
6. Il parser legge `dhEmi` (NF-e 4.00); il layout vecchio con `dEmi` sarebbe
   rifiutato, come nel Python.
7. Il parser e' **piu' stretto del Python** sui valori obbligatori assenti:
   scelta intenzionale, ma un fornitore che omette una tag blocca l'import.
8. I 7 alias divergenti e gli 11 ambigui della tabella C16 restano come sono:
   sono dati per una decisione, non bug da correggere d'ufficio.
9. **Copertura di `Dado<T>` e trattamento dei parziali**: da definire prima
   della fase 3, insieme a input/output, riconciliazione e test contro la
   fonte Python. Decisione separata da C13 e da `serde_yaml`.
10. **Validazione XML/XSD**: C17 conta radici e NF-e, non valida il layout.
    Un XML ben formato ma fuori layout (per esempio `infNFe` annidato) viene
    rifiutato dal conteggio, non da uno schema. Se servira' una validazione
    vera, e' una decisione separata con la sua fonte e i suoi test.
    (La fixture `nfe_sem_fat.xml` e' stata ripulita: non e' piu' un residuo.)

Nessuno di questi punti e' stato modificato dall'audit. Non chiuderli di
iniziativa propria.

### Nota d'ambiente (indicazione storica, non causa universale verificata)

Il 2026-10-16 un `cargo test` lanciato da un worktree temporaneo con percorso
molto lungo e' fallito con `linking with link.exe failed: exit code 1104` sui
build script (`proc-macro2`, `quote`, `serde_core`). Rieseguito con un
`CARGO_TARGET_DIR` corto, e' passato (42 test verdi). La spiegazione piu'
probabile e' il limite di lunghezza dei percorsi, ma **non e' stata provata
isolando la variabile**: trattala come indizio, non come diagnosi.

---

## 8. Metodo obbligatorio in questo progetto

Prima di modifiche non banali applica la skill `programmatore-bravo` e dichiara
problema, decisione funzionale, contratto, file e logiche da toccare, test,
audit, rischi e perimetro escluso. Dopo il lavoro dichiara modifiche, contratti
implementati, test eseguiti, audit, rischi residui e stato del repository.

Dato mancante diverso da zero tecnico: usa `MANCANTE`, `NON_VERIFICATO`,
`NON_DISPONIBILE` o `N.A.`, con valore, fonte, stato, copertura e motivo.
Vietati fallback silenziosi e hardcode nascosti.

Nessuna fase e' chiusa se `git status -sb` mostra file tracked modificati non
classificati: in quel caso produci la tabella File / Origine-mandato / Stato /
Destino prima di qualunque commit, push, cambio fase o handoff.

Il ruolo dell'audit resta separato: Codex audita, non modifica il progetto.

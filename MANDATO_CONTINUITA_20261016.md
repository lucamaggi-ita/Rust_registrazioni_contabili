# Mandato di continuita' — 2026-10-16

Documento di passaggio per una sessione nuova aperta in
`C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.

Non e' un riassunto di cortesia: contiene lo stato verificato, i divieti attivi
e le decisioni ancora in mano a Luca. Leggilo prima di toccare qualunque file.

Documenti correlati nello stesso percorso:

- `MANDATO_CLAUDE_RINOMINA_E_CHIUSURA.md` — mandato originale (rinomina + residuo C10).
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
| HEAD | `ad53556` (fase 1), 4 commit, `main` 0/0 con il riferimento locale `origin/main`, nessuno stash |

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
cargo test --locked                                      105 passati, 0 falliti
                                                         (92 lib + 3 bin + 10 CLI)
cargo fmt --check                                        pulito
cargo clippy --locked --all-targets -- -D warnings       pulito
```

I quattro riscontri della fase 2 sono **chiusi**: entita' XML e CDATA
preservati, validazione condivisa dei comandi con avvisi C13 su stderr, audit
veritiero dei 571 alias, e il residuo C10 "mancante diverso da zero".

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

- **Nessun commit e nessun push** senza autorizzazione esplicita di Luca.
- **Nessuna cancellazione** della vecchia cartella `Rust_schede_tecniche`, ne'
  della salvaguardia `...\GitHub\RUST\_SALVAGUARDIA_contabile_20261015`.
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

Tutto il lavoro di fase 2 + rinomina + C10 e' **fuori commit**: HEAD e' ancora
la fase 1. Circa 642 righe di diff su 8 tracked e circa 2.400 righe untracked.
Ripeti sempre il controllo prima di operare, non fidarti di questa fotografia:

```bash
git status -sb
git diff --name-status
git diff --cached --name-status
git ls-files --others --exclude-standard
```

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `Cargo.toml` | rinomina package + dipendenze fase 2 | verificato | commit atomico (vedi 5) |
| `Cargo.lock` | nome package, dipendenze invariate | verificato con `--locked` | commit atomico |
| `README.md` | identita', contratti fase 2, C10 | verificato | commit atomico |
| `src/lib.rs` | esposizione moduli fase 2 | verificato dalla build | commit atomico |
| `src/main.rs` | comandi, crate, output C10 | verificato | commit atomico |
| `src/modelos.rs` | `Dado<T>` e importi facoltativi | C10 verificato | commit atomico |
| `src/regras.rs` | caricamento regole e avvisi C13 | verificato | commit atomico |
| `tests/cli_validar_config.rs` | nome eseguibile | test superati | commit atomico |
| `src/classificacao.rs` (untracked) | classificatore e audit alias | verificato | commit atomico |
| `src/nfe.rs` (untracked) | parser NF-e e C10 | verificato | commit atomico |
| `src/normalizacao.rs` (untracked) | normalizzazione testo | test superati | commit atomico |
| `tests/cli_configuracao_obrigatoria.rs` (untracked) | 7 regressioni CLI | superati | commit atomico |
| `tests/fixtures/nfe_sem_fat.xml` (untracked) | regressione C10 | fixture dichiarata | commit atomico |
| `samples/FONTE.md` + XML (untracked) | campione dalla fonte Python | hash preservati | commit 1 (vedi 5) |
| `MANDATO_CLAUDE_RINOMINA_E_CHIUSURA.md` (untracked) | mandato utente/Codex | documento | decisione di Luca: versionare o archivio audit |
| `MANDATO_CONTINUITA_20261016.md` (questo file, untracked) | passaggio di sessione | documento | decisione di Luca: versionare o archivio audit |

Nessun file puo' restare senza destino esplicito. Nessun commit misto con file
non appartenenti allo stesso tema.

### Salvaguardia dello stato pre-rinomina

`C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\_SALVAGUARDIA_contabile_20261015`
contiene `repo_completo.bundle` (verificato con `git bundle verify`, ref `main`
al vecchio HEAD `ad53556`), `tracked_modificati_20261015.patch`,
`working_tree_completo` e `manifesto_sha256.txt`. Codex ha confrontato 23 file
del manifesto e 21 file del proprio snapshot precedente: nessun mancante,
nessun divergente. Non eliminarla.

---

## 5. Decisioni aperte in mano a Luca

**(A) Registrazione in Git.** Piano corretto, due commit entrambi compilabili e
con test verdi, `git add` nominativo file per file:

1. `samples/FONTE.md` + `samples/nfe_exemplo/*.xml` — solo dati, non rompe HEAD.
2. **Commit atomico**: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/main.rs`,
   `src/modelos.rs`, `src/regras.rs`, `src/classificacao.rs`, `src/nfe.rs`,
   `src/normalizacao.rs`, `tests/cli_validar_config.rs`,
   `tests/cli_configuracao_obrigatoria.rs`, `tests/fixtures/nfe_sem_fat.xml`,
   `README.md`.

Vincoli che rendono obbligatorio questo ordine, verificati:

- `HEAD:src/main.rs:12` contiene ancora `use rust_schede_tecniche::regras::{`,
  e `HEAD:tests/cli_validar_config.rs` usa `CARGO_BIN_EXE_rust_schede_tecniche`:
  un commit che rinomini solo `Cargo.toml`/`Cargo.lock`/il test **non
  compilerebbe**.
- Il `main.rs` attuale importa `classificacao` e `nfe`, che sono untracked:
  non si possono separare dal commit del codice.
- I test leggono `samples/nfe_exemplo/nfe_oesa_12566_20260505.xml`
  (`src/nfe.rs:685`, `tests/cli_configuracao_obrigatoria.rs:58` e `:89`):
  `samples/` non puo' arrivare dopo il codice, o quel commit avrebbe test rossi.

Non spezzare modifiche dipendenti in commit intermedi rotti. Nessun push e'
autorizzato da questo documento.

**(B) I due file di mandato.** Versionarli nel repository o spostarli
nell'archivio audit esterno.

**(C) Rimozione della cartella residua.** `...\RUST\Rust_schede_tecniche`
contiene soltanto una copia di `.git` e `_CARTELLA_OBSOLETA_DA_ELIMINARE.md`.
Le 16 cancellazioni tracked visibili nel suo `git status` derivano dallo
spostamento dei file: **non vanno committate come rimozione del progetto**. La
cancellazione richiede autorizzazione esplicita e va fatta solo dopo che
nessuna finestra la sta usando come workspace.

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

Nessuno di questi punti e' stato modificato dall'audit. Non chiuderli di
iniziativa propria.

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

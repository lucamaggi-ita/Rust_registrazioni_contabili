# Proposta di mandato Claude — 3C.4.1, export XLSX derivato

Data: 2026-10-23. Preparazione: Codex. Destinatari: Luca e Claude.

**Stato attuale: MANDATO APPROVATO ED ESEGUITO, AUDIT SUPERATO.** Le sezioni iniziali conservano la proposta originaria; l'approvazione implementativa e' registrata in §11. Il controllo finale e la successiva autorizzazione ai due commit locali del 2026-10-30 sono in §17. Nessun push autorizzato in questo passaggio.

## 1. Obiettivo e fonte normativa

Produrre un file XLSX derivato dall'archivio autorevole, con diagnostico completo, estratto revisionato, riepiloghi, decisioni e limiti secondo lo schema approvato. L'archivio resta autorevole; il workbook non diventa un ingresso modificabile del sistema.

Fonte normativa: `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md`, revisione 8, registrata in `9d99832`. Applicare D18-D26 ed EXP-29 come registrate, i contratti EXP-01..EXP-53 con le varianti decadute escluse, e la matrice TX della specifica. Non riaprire decisioni per preferenza implementativa.

Baseline osservata: HEAD `9d99832`, `main...origin/main`, worktree pulito. I 337 test sono la baseline precedente registrata, non una suite rieseguita durante la preparazione di questo mandato. Toolchain osservata: rustc e cargo 1.99.1.

Classificazione: lavoro STRUTTURALE limitato all'export. Il lavoro esistente conserva e revisiona dati ma non produce ancora questo XLSX: questo e' il perimetro necessario. Nessun ulteriore studio generale richiesto prima della prima prova della libreria.

## 2. Acquisizione proposta: esplicita e circoscritta

All'approvazione di questo mandato, acquisire tramite Cargo la sola nuova dipendenza diretta seguente e le sue dipendenze transitive necessarie:

```toml
rust_xlsxwriter = { version = "=0.101.0", default-features = false, features = [] }
```

La versione e' fissata anche nel requisito Cargo, non solo nel lockfile. La documentazione consultata espone 0.101.0; il manifest upstream osservato dichiara licenza `MIT OR Apache-2.0`, Rust minimo 1.88.0, nessuna feature predefinita e dipendenza ZIP con compressione. Questi sono fatti documentali, **non** una prova di acquisizione o compilazione del pacchetto pubblicato.

Prima dell'uso confermare nel pacchetto effettivamente risolto versione, checksum, licenza, MSRV e feature. Fonti: [manifest upstream consultato](https://raw.githubusercontent.com/jmcnamara/rust_xlsxwriter/main/Cargo.toml), [API Workbook](https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/workbook/struct.Workbook.html). Il riferimento `main` e le pagine `latest` sono mobili: il rapporto finale nomina il pacchetto realmente usato e la sua versione.

Non abilitare feature opzionali: niente `constant_memory`, zlib C, polars, serde, chrono, rust_decimal, autofit avanzato o acceleratori numerici. L'API di base e il buffer bastano all'obiettivo proposto; verificarlo con TX-38. Non aggiungere un secondo writer, un lettore XLSX Rust o una dipendenza ZIP diretta per i test.

L'acquisizione autorizzata dopo approvazione riguarda registro e cache Cargo; nessuna installazione di Python, pacchetti pip, programmi globali o toolchain. Nessun `cargo update` generale. Le dipendenze preesistenti restano alle versioni del lockfile se compatibili. Mostrare nel rapporto ogni pacchetto nuovo/cambiato, versione, motivo, feature e licenza; rendere spiegabile tutto il diff del lockfile. Le transitive sono NON_VERIFICATE finché non vengono risolte, non inventarne l'elenco.

Se la versione non e' disponibile o non e' compatibile, riportare l'errore e una proposta concreta; non sostituire versione/libreria o aggiungere feature in silenzio. Dopo l'acquisizione, le verifiche possono usare la cache; il programma esportatore non richiede rete per generare il workbook.

## 3. File ammessi

| File | Modifica ammessa | Destino della lavorazione |
|---|---|---|
| `src/export.rs` | nuovo: modello dell'export e costruzione pura, secondo §12 della specifica | gruppo codice, dopo audit e autorizzazione Git |
| `src/xlsx.rs` | nuovo: adattatore puro verso la libreria, restituisce byte in memoria | gruppo codice |
| `src/export_io.rs` | nuovo: protocollo di scrittura approvato | gruppo codice |
| `src/lib.rs` | sole esportazioni dei tre moduli e mappa delle fasi se necessario | gruppo codice |
| `Cargo.toml`, `Cargo.lock` | sola dipendenza proposta e conseguenze motivate | gruppo codice |
| `tests/export_releitura.rs` | nuovo: percorso composto e regressioni pertinenti | gruppo codice |
| `script/releitura_export.py` | nuovo: lettura indipendente ripetibile secondo EXP-29 | gruppo codice |
| `README.md` | stato reale, API, verifiche, conteggio effettivo e limiti | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | sola appendice di esecuzione e stato; nessuna riscrittura silenziosa della storia | gruppo documentale |
| questo mandato | registrazione dell'approvazione e appendice di esito | gruppo documentale |

Per sonde iniziali e dati sintetici usare un'area temporanea fuori dal repository. Non lasciare workbook, rapporti o script provvisori non classificati nel progetto. Una fixture versionata aggiuntiva va prima motivata nel rapporto di perimetro; preferire dati sintetici costruiti dai nuovi test.

## 4. Confini da mantenere

Non modificare `arquivo.rs`, `arquivo_io.rs`, `revisao.rs`, `impressao.rs`, `prelivro.rs`, `alocacao.rs`, `dinheiro.rs`, `modelos.rs`, `nfe.rs`, `regras.rs`, `main.rs`, i test esistenti, `config/`, `samples/` o il repository Python.

Fuori: CLI 3C.5, DRE 3D, import dello storico Excel, adattatori grezzi D12, nuovi stati di `Particao`, CSV aggiuntivo, autenticazione, utenti/permessi, altri protocolli di concorrenza, nuove grammatiche di impronta, produzione contabile. Non riprogettare lo ZIP/XML e non correggere a mano i byte prodotti dalla libreria.

Se un difetto reale impedisce l'obiettivo senza uscire da questo elenco, fornire la riproduzione minima e la proposta circoscritta prima di cambiare il componente escluso. Una pulizia non bloccante si pianifica a parte.

## 5. Primo passo: prova minima della libreria

Prima dei tredici fogli, eseguire TX-38 e TX-37 su un workbook minimo, con la versione fissata:

1. `save_to_buffer` o API equivalente produce i byte senza scritture del writer su disco; niente modalita' che usino file temporanei.
2. Scrittura esplicita come stringa di `=`, `+`, `-`, `@`, zeri iniziali, virgolette, LF e Unicode: rilettura identica e nessuna formula indesiderata.
3. Testo monetario identico a `Decimal::to_string()`, scala inclusa; colonna numerica distinta secondo D20 e D26(a).
4. `_x0041_` e `_x005F_x0041_` restituiti identici dal lettore indipendente.

Una codifica interna diversa con testo riletto identico e' conforme a D22. Non richiedere XML identico al testo. Un testo logico cambiato o un'API non adatta deve essere misurato e riportato: nessuna compensazione manuale o nuova decisione implicita. Leggere l'API non vale come prova eseguita.

Superata questa prova, continuare direttamente con implementazione e verifiche previste. Non chiedere un'altra conferma per operazioni gia' comprese nel mandato approvato.

## 6. Implementazione vincolata alle decisioni

- Seguire le firme e la composizione delle funzioni reali di §4 e §12 della specifica. Mantenere la barriera di `construir_pre_livro`: il validatore archivistico non ne attesta gli invarianti.
- Tredici fogli e colonne secondo lo schema approvato, senza aggiunte funzionali; chiavi complete per righe e figli, documenti senza registrazioni conservati, eventi non scartati in silenzio.
- Piano archiviato e piano effettivamente usato distinti. Controllo di presentazione limitato al lotto esportato. `plano_coincide` significa soltanto il confronto di metadati gia' approvato.
- Importi autorevoli sempre testuali ed esatti, identificativi testuali con zeri. Nessun totale ricostruito o delegato a formule Excel.
- Una sola funzione di `xlsx.rs` contiene l'eccezione numerica di D26(a), documentata con motivo e rinvio a EXP-53. Gli altri due moduli non convertono importi in `f64`. D20 decide presenza/assenza della cella; l'assenza ha stato e motivo, mai zero sostitutivo.
- Testare separatamente esattezza testuale e valore numerico riletto. La colonna numerica non conserva la scala per contratto. Nessuna conversione binaria nei calcoli, archivio o impronte.
- Testi non rappresentabili e limiti di cella/formato: errore nominato prima del commit, senza troncamento o sostituzione.
- Approvazione attiva e successive conferme mantengono le attribuzioni distinte approvate, senza scegliere l'ultimo nome per comodita'.
- Ordine deterministico dei contenuti, non promessa di byte ZIP identici.
- IO secondo D24 e D25(b): lucchetto cooperativo, `Recusar` con controllo di esistenza, nessuna rivendicazione del nome finale; temporaneo e conservazione del precedente secondo §8. Proteggere archivio e famiglia dei suoi nomi, comprese le varianti di maiuscole.
- Nessuna garanzia inventata contro scrittori che ignorano il lucchetto. Il limite e' dichiarato nel workbook. Errori e lucchetti residui non si trasformano in successi.

## 7. Prova indipendente ed Excel

`script/releitura_export.py` deve essere versionato e ripetibile, con invocazione documentata e codice di uscita non zero su discordanza. Usa soltanto Python e openpyxl gia' presenti; nessun pip. Rapporto con versione/provenienza di interprete e lettore, input, confronti e limiti.

La verifica confronta campo per campo contro attese dichiarate nel test o nel modello, non contro una seconda copia della logica del writer. Includere le regressioni di TX-04, TX-09/12, TX-35 e TX-37. L'esecuzione standard di `cargo test` non deve dipendere silenziosamente da Python: dichiarare chiaramente il comando separato della prova indipendente. Un confronto non eseguito non risulta superato.

Predisporre workbook e checklist di TX-18/TX-19 per Microsoft Excel. Se quelle prove non sono eseguibili nell'ambiente disponibile, consegnare file e istruzioni concrete e marcarle NON_ESEGUITE, con la conseguenza sulla compatibilita' stabile. La rilettura openpyxl non sostituisce una misura su Excel.

## 8. Verifiche e accettazione

Seguire la matrice TX e i criteri di §13, comprese TX-16b e TX-34b. Nessuna quota di test da raggiungere; il conteggio finale deriva dalle prove effettive.

Eseguire prima le prove mirate, poi a implementazione coerente:

```text
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
git diff --check
```

I 337 precedenti restano verdi senza modificarli. Eseguire separatamente il comando Python della rilettura e registrarne esito/codice di uscita. Non installare pacchetti per farlo funzionare.

Confrontare prima/dopo e contro gli attesi canonici le cinque fonti YAML e `samples/nfe_exemplo/nfe_oesa_12566_20260505.xml`; non sostituire una verifica col solo stato Git. Verificare che i componenti esclusi siano invariati, inclusi i nuovi file inattesi che non compaiono in `git diff`.

Rapporto obbligatorio: mappa EXP -> prova/ispezione, TX eseguite e non eseguite, inventario `f64`/conversioni/`write_number` nei tre nuovi moduli con file e riga secondo §11.2, grafo delle dipendenze effettive e diff Cargo spiegato, limiti osservati, perimetro ed eventuali scostamenti. Nessun "criterio soddisfatto" per una misura mancante.

Non espandere i limiti gia' dichiarati in nuovi progetti. Riaprire una decisione soltanto con una prova pertinente che mostri l'impossibilita' o una perdita reale rispetto al contratto approvato.

## 9. Git e consegna

Questo mandato, anche se approvato per l'implementazione, **non autorizza commit, push o riscritture di storia**. Alla consegna il codice resta disponibile per l'audit indipendente.

Prima della consegna eseguire `git status -sb`, `git diff --name-status` e `git diff --cached --name-status`. Classificare tutti i tracked modificati e gli untracked nella tabella File / Origine-mandato / Stato / Destino. Destini: gruppo codice e gruppo documentale della tabella §3, entrambi dopo audit e autorizzazione Git. Il presente mandato e' continuita' nominata del lavoro non ancora registrato.

Niente `git add .`, reset, restore, checkout, clean, stash, amend, rebase o force-push. Non dichiarare chiusa la fase con file senza classificazione. Distinguere implementazione pronta per audit, audit superato, registrazione Git e pubblicazione.

## 10. Approvazione ed esito

**Approvazione dell'implementazione/acquisizione: NON ANCORA REGISTRATA.** Luca puo' approvare questo mandato esplicitamente nella chat di esecuzione; Claude registra qui l'atto prima di iniziare, senza attribuirlo alla sola presenza del file.

Appendice di esito da compilare dopo il lavoro: modifiche, contratti implementati, test e audit, scostamenti, dipendenze acquisite, prove manuali mancanti, rischi residui e stato Git. Nessuna promozione in produzione o avvio di 3C.5/3D impliciti.

---

## 11. Registrazione dell'approvazione — 2026-10-23

**Approvazione registrata.** Luca ha approvato questo mandato in modo esplicito
nella chat di esecuzione del 2026-10-23, scegliendo «implementazione e
acquisizione». L'atto è registrato qui **prima** di iniziare, e non è
attribuito alla sola presenza del file, come §10 richiede.

| Voce | Valore |
|---|---|
| Che cosa è autorizzato | implementazione del lotto 3C.4.1 nel perimetro di §3, e acquisizione Cargo della sola dipendenza di §2 |
| Che cosa resta non autorizzato | commit, push, riscritture di storia; qualunque file fuori da §3; le esclusioni di §4 |
| Baseline verificata prima di iniziare | HEAD `9d99832`, `main...origin/main` allineato (0 avanti, 0 dietro), nessun tracked modificato |
| Toolchain misurata | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`: soddisfa l'MSRV 1.88.0 dichiarato dalla libreria |
| Stato della cache | `rust_xlsxwriter` **assente**, come nessun crate ZIP o di compressione: l'acquisizione richiede la rete una volta |

### 11.1 Fatti documentali confermati prima dell'acquisizione

Letti sul manifest upstream, **lettura e non prova di acquisizione**; il
pacchetto realmente risolto va riconfermato dopo l'acquisizione, come §2
richiede.

| Voce | Valore letto |
|---|---|
| versione | `0.101.0`, la stessa fissata dal mandato |
| MSRV | `1.88.0` |
| licenza | `MIT OR Apache-2.0` |
| feature predefinite | `default = []`: `default-features = false, features = []` coincide col predefinito |
| dipendenze non opzionali | **una**: `zip = { version = "9.0", default-features = false, features = ["deflate"] }` |

**Conseguenza da dichiarare subito**: con `zip/deflate` il workbook prodotto
sarà **compresso**, non `stored` come la sonda §5.1 della specifica. Quella
sonda resta valida come fatto sul formato; il file prodotto dal lotto è un
altro contenitore, e D23 non promette i byte. La differenza va nel rapporto
finale invece di essere scoperta dopo.

Le dipendenze transitive di `zip 9.0` restano **NON_VERIFICATE** finché non
sono risolte: non ne invento l'elenco.

---

## 12. Primo passo eseguito: acquisizione, TX-38 e TX-37 — 2026-10-23

### 12.1 Acquisizione, confermata sul pacchetto risolto

`cargo fetch` ha risolto e scaricato **10 pacchetti nuovi**. Versioni,
checksum e licenze letti dal lockfile e dai manifest in cache, non dedotti.

| Pacchetto | Versione | Checksum (SHA-256) | Licenza | MSRV | Compilato su Windows |
|---|---|---|---|---|---|
| `rust_xlsxwriter` | 0.101.0 | `49aa1d15…f8908` | MIT OR Apache-2.0 | 1.88.0 | sì, diretta |
| `zip` | 9.0.5 | `6e04976d…a11e` | MIT | 1.88 | sì |
| `flate2` | 1.1.10 | `6e634e2e…dcb` | MIT OR Apache-2.0 | 1.67.0 | sì |
| `zlib-rs` | 0.6.8 | `b268e58e…4112` | Zlib | 1.75 | sì |
| `zopfli` | 0.8.4 | `aaf7fc5d…fb11` | Apache-2.0 | 1.88.0 | sì |
| `crc32fast` | 1.5.3 | `0ba106e8…2055` | MIT OR Apache-2.0 | 1.63 | sì |
| `simd-adler32` | 0.3.10 | `3a219298…deea` | MIT | non dichiarato | sì |
| `typed-path` | 0.12.4 | `54ec2755…7336c` | MIT OR Apache-2.0 | 1.65.0 | sì |
| `nix` | 0.31.3 | `cf20d2fd…7c3d` | MIT | 1.69 | **no**: dipendenza di `zip` condizionata a macOS/BSD |
| `bitflags` | 2.13.2 | `3ded4057…0b06` | MIT OR Apache-2.0 | 1.56.0 | **no**: transitiva di `nix` |

Tutte le licenze sono permissive. Tutti gli MSRV sono soddisfatti dalla
toolchain 1.99.1. Due pacchetti sono **bloccati nel lockfile ma non
compilati** su questo target, e `cargo tree -i` lo conferma: «nothing to
print» per entrambi.

Albero effettivo su questo target, misurato con `cargo tree`:
`rust_xlsxwriter` → `zip` → { `crc32fast` → `cfg-if`; `flate2` → `zlib-rs`;
`indexmap` → `equivalent`, `hashbrown`; `memchr`; `typed-path`; `zopfli` →
`bumpalo`, `crc32fast`, `log`, `simd-adler32` }. `indexmap`, `memchr`,
`cfg-if`, `bumpalo`, `log` erano **già** nel lockfile: non compaiono fra i
dieci aggiunti.

Diff del lockfile: **+91 righe**, tutte le dieci voci nuove con checksum, più i
riferimenti nelle dipendenze dei pacchetti coinvolti. `Cargo.toml`: **+1 riga**,
`rust_xlsxwriter = { version = "=0.101.0", default-features = false, features = [] }`.
Nessun `cargo update` generale: le dipendenze preesistenti restano alle versioni
del lockfile.

Nessuna installazione: niente Python, pip, programmi globali o toolchain.

### 12.2 TX-38: le quattro proprietà, misurate

| Proprietà | Esito |
|---|---|
| 1. buffer in memoria | **confermata**: `Workbook::save_to_buffer() -> Result<Vec<u8>, XlsxError>` scrive su un `Cursor<&mut Vec<u8>>` (`workbook.rs:1402`, letto in cache). Il test verifica anche che la directory di lavoro non cambi |
| 2. una cella di testo resta testo | **confermata** dal lettore indipendente: `=1+1 SUCO "MARCA" Açucar`, `+55 11`, `-7`, `@utente` tornano di tipo `s` e **zero** celle di tipo formula in tutti i fogli |
| 3. nessuna conversione implicita | **confermata**: il testo autorevole torna identico per tutti i casi, `10.0000`, `0.00`, `-7.50`, `79228162514264337593543950335` compresi |
| 4. comportamento su `_xHHHH_` | **misurato, con una divergenza**: §12.4 |

Risultati della cella numerica di comodo, come D26(a) condizione 4 prevedeva:
testo `10.0000` → numero `10`; `0.00` → `0`; `-7.50` → `-7.5`; `0.01` → `0.01`;
`12345678901234.56` → esatto. Il **valore** sopravvive, la **scala** no, e il
testo autorevole la conserva.

Altri fatti misurati: zeri iniziali conservati (`00012345678901`); ritorno a
capo e tabulazione conservati; NFC e NFD restano **distinti** e identici agli
originali; il pacchetto ha 11 voci, **tutte compresse** con `deflate`, e il CRC
di ognuna è integro.

### 12.3 Prova indipendente: lo script esiste e ha un esito

`script/releitura_export.py`, versionato, eseguito con
`py -I script/releitura_export.py <workbook> --modo tx38`. Si riconfigura lo
stdout in UTF-8 da sé, perché `py -I` ignora `PYTHONIOENCODING` e una console
cp1252 non sa stampare i code point della prova: «ripetibile» significa che non
dipende dalla console di chi lo lancia.

Esito dell'esecuzione: **68 confronti superati, 1 discordanza**, codice di
uscita **1**. Provenienza registrata dallo script stesso: Python 3.13.6 e
`openpyxl` 3.1.5 dell'interprete dell'utente. Nessun pip.

`cargo test` **non** invoca lo script: un test deposita il workbook e finisce,
il confronto è un comando separato.

### 12.4 La divergenza, e di chi è

Caso che fallisce: un testo logico che contiene letteralmente
`_x005F_x0041_`, cioè una sequenza di escape già protetta.

| Testo logico | XML scritto da `rust_xlsxwriter` | `openpyxl` rilegge | Esito |
|---|---|---|---|
| `_x0041_` | `_x005F_x0041_` | `_x0041_` | **identico**: la condizione posta da Luca su D22 è soddisfatta |
| `_x005F_x0041_` | `_x005F_x005F_x005F_x0041_` | `_x0041_` | **perde un livello** |

**Il writer è corretto.** Misurato con un decodificatore a **passata singola**
della convenzione `ST_Xstring`, la forma corretta: applicato all'XML del
writer restituisce `_x0041_` per il primo caso e `_x005F_x0041_` per il
secondo, cioè **esattamente** i testi logici. L'XML prodotto è quindi
l'escaping giusto, e la perdita viene dal decodificatore del lettore, che su
quell'ingresso decodifica più di una volta.

Che cosa **non** ho fatto, perché il mandato §5 lo vieta: nessuna
compensazione manuale, nessuna trasformazione nostra aggiunta, nessuna nuova
decisione implicita. Il caso resta nello script e lo fa uscire con 1, perché
un confronto che non torna non va dichiarato superato.

La decisione è di Luca, e le strade sono tre:

| Strada | Che cosa comporta |
|---|---|
| (a) limite dichiarato | il workbook dichiara nel foglio `limites` che un testo contenente una sequenza simile a `_xHHHH_` può non tornare identico **con questo lettore**; il file resta corretto e `TX-37` accetta il caso come divergenza nota del lettore |
| (b) non rappresentabile | un testo che corrisponde a `_x[0-9A-Fa-f]{4}_` diventa `TextoNaoRepresentavel` di `EXP-33`: si rifiuta il dato invece di scriverlo in un modo che un lettore mostrerà male |
| (c) altro lettore | cercare un lettore indipendente che decodifichi a passata singola, cambiando la forma di `EXP-29` |

La strada (a) è coerente con la lettura operativa di D22 già approvata — la
codifica interna è della libreria e il file è corretto — ma lascia un dato che
**quel** lettore mostra diverso. La (b) rifiuta dati contabili validi per un
difetto altrui. La (c) costa una ricerca e non garantisce un esito.

`TX-19` su Excel resta necessaria e **non eseguita**: senza di essa la
compatibilità non si dichiara stabile, come `EXP-36` impone.

---

## 13. Decisione sulla divergenza nota del lettore — 2026-10-23

**Strada (a) approvata da Luca**: limite dichiarato. Registrata in §7.4.1.1
della specifica, con i contratti `EXP-54…EXP-58`.

| Condizione posta da Luca | Come è attuata |
|---|---|
| mantenere il caso come **discordanza nota del lettore**, identificata con input, versione e risultato osservato | `script/releitura_export.py` ha un registro `DIVERGENZE_NOTE` con confronto, testo in ingresso, lettore, versione misurata, risultato osservato, XML del writer, ragione e decisione |
| non eliminarlo | il caso resta fra quelli confrontati; viene classificato, non rimosso |
| non dichiarare `TX-37` integralmente superata | lo script lo **scrive** nel proprio esito: «TX-37 NON e' integralmente superata» |
| ogni discordanza **diversa** resta un errore da esaminare | un risultato diverso da quello registrato esce con codice 1 e segnala che era attesa un'altra divergenza. Anche la **scomparsa** della divergenza esce con 1: una registrazione stantia è una falsa dichiarazione |
| nessuna compensazione nel writer | `xlsx.rs` non trasforma il testo: lo passa alla libreria come è |
| nessun rifiuto aggiuntivo dei dati | nessun nuovo caso di `TextoNaoRepresentavel`: un dato contabile valido non viene rifiutato per un difetto di un lettore |
| conservare la controprova sulla codifica | la decodifica a passata singola e il suo esito sono in §7.4.1.1 della specifica e nel registro dello script |
| `TX-19` NON_ESEGUITA, compatibilità non stabile | dichiarato in §7.4.1.1, nel foglio `limites` (`EXCEL_NAO_ATESTADO`) e nell'esito dello script |

Esito dello script dopo la decisione: **68 confronti superati, 1 divergenza
nota, 0 discordanze**, codice di uscita **0**, con la divergenza e il suo
perché stampati e `TX-37` dichiarata non integralmente superata.

Nel foglio `limites` del workbook la voce è `ESCAPE_LEITOR` con stato
`DIVERGENZA_NOTA`, che è uno stato nuovo accanto a `DICHIARATO` e
`NON_VERIFICATO`.

---

## 14. Appendice di esito — lotto 3C.4.1 consegnato per audit (2026-10-23)

| Voce | Valore |
|---|---|
| Stato | **IMPLEMENTATO**; audit indipendente del 2026-10-30 **NON SUPERATO**, quattro rilievi corretti in §15. Nessun commit, nessun push |
| Baseline | `9d99832`, worktree pulito prima di iniziare |
| Approvazione | registrata in §11 **prima** di iniziare, come §10 richiede |
| Test | alla consegna **382** (360 lib, 3 binario, 19 integrazione); dopo le correzioni di §15 sono **386**, con i **337** precedenti verdi senza modifiche |
| Verifiche | `cargo test --locked`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check`: pulite |
| Fonti canoniche | 6/6 invariate, confrontate per impronta |
| Perimetro escluso | intatto: nessuna modifica ai moduli, ai test preesistenti, a `config/` o `samples/` |
| Dipendenza | `rust_xlsxwriter =0.101.0`, `default-features = false`, più dieci pacchetti nuovi nel lockfile: otto compilati su questo target, dipendenza diretta compresa, e due solo bloccati. Dettaglio in §19.2 della specifica |
| Prova indipendente | `script/releitura_export.py`, due modi, esito con codice di uscita controllato |
| Mappa contratti e prove | §19.3 della specifica |
| Inventario `f64` | §19.5 della specifica |

### 14.1 Che cosa è stato scritto

| File | Natura |
|---|---|
| `src/export.rs` | nuovo, **puro**: modello dell'export, validazione al confine, i tredici fogli. **Zero** occorrenze di `f64` |
| `src/xlsx.rs` | nuovo, **puro**: adattatore verso `rust_xlsxwriter`, restituisce i byte. Contiene l'unica eccezione di `f64`, in una sola funzione |
| `src/export_io.rs` | nuovo, IO: protocollo con D24 e D25(b), esclusione del sorgente, lucchetto cooperativo |
| `tests/export_releitura.rs` | nuovo: il percorso composto dall'archivio al file, determinismo, archivio intatto, deposito per la rilettura |
| `script/releitura_export.py` | nuovo, fuori dal crate: rilettura indipendente con Python e `openpyxl` già presenti |
| `src/lib.rs` | registrazione dei tre moduli e mappa delle fasi aggiornata |
| `Cargo.toml`, `Cargo.lock` | la sola dipendenza autorizzata e le sue conseguenze |
| `README.md` | stato di fase, contratti `EXP-nn`, API, conteggio reale, inventario `f64`, dipendenza e limiti |

### 14.2 Scostamenti dal piano del mandato

Tre, tutti dichiarati e nessuno in riduzione del perimetro.

1. **La conversione a `f64` era nel posto sbagliato.** L'inventario obbligato
   da §11.2 ha trovato che `pacote` convertiva e passava un `f64` alla funzione
   autorizzata: formalmente `EXP-53` era violato dalla riga che converte.
   Corretto: la funzione riceve un `Decimal` e converte dentro di sé.
   `TX-04` non l'avrebbe visto, perché il comportamento era già corretto.
2. **La prova di determinismo era mal posta.** Confrontava due esecuzioni in
   **directory diverse**, e il percorso dell'archivio è una cella del foglio
   `proveniencia`: erano due ingressi diversi, non un difetto. Riscritta come
   `EXP-25` la quantifica, ripetendo la derivazione sugli **stessi** ingressi.
3. **Lo script di rilettura si configura da sé.** `py -I` ignora
   `PYTHONIOENCODING`, e una console cp1252 non stampa i code point della
   prova. «Ripetibile» significa che non dipende dalla console di chi lo
   lancia, quindi lo script imposta UTF-8 sui propri flussi.

Quattro attese mie sbagliate, corrette nei test e non nel codice: il pre-libro
della 3C.2 rifiuta una scala monetaria oltre i due decimali, quindi la
quantità a quattro decimali vive in un componente e non in `Lancamento::valor`;
`12345678901234.56` ha **sedici** cifre significative e non è eleggibile per la
cella numerica; `-7.50` ha due cifre significative e non tre; e un carattere
non rappresentabile nell'autore viene segnalato dal **primo** foglio che lo
contiene nell'ordine dello schema.

### 14.3 Audit interno, prima della consegna

| Controllo | Esito |
|---|---|
| i test sono scritti contro il contratto, non contro il codice | quattro hanno fallito al primo giro, e in tre casi l'errore era nella mia attesa, in uno nel codice (`EXP-53`) |
| nessun fallback silenzioso | ogni percorso d'errore ha una variante nominata; nessuna cella numerica con valore arbitrario; nessun troncamento |
| nessuna compensazione della divergenza del lettore | il writer non trasforma nulla, e la divergenza è registrata con input, versione e risultato |
| `TX-37` non dichiarata superata | lo script lo scrive nel proprio esito |
| nessuna cancellazione fuori dalle aree temporanee | le arene stanno sotto `%TEMP%\fantozzi_3c41*\`, una per test |
| nessun calcolo contabile duplicato | i riepiloghi e gli indicatori sono copiati dalla vista, cella per cella, e `tx06_*` lo verifica |

### 14.4 Rischi residui

`NON_VERIFICATO`: la compatibilità con Microsoft Excel in due punti (`TX-18`,
`TX-19`), il limite ZIP64 (`TX-31`), un file di destinazione tenuto da un altro
programma (`TX-16b`), i volumi reali, l'esclusione sorgente/destinazione su
hard link e volumi montati due volte, la divergenza del lettore su lettori
diversi da `openpyxl` 3.1.5.

`DICHIARATO` nel file prodotto, foglio `limites`: il derivato non rientra
nell'archivio; l'autore non è autenticato; le colonne `_numero` non sono
autorevoli; `Recusar` non è atomico contro chi ignora il lucchetto.

Fuori perimetro e non avviati: CLI 3C.5, DRE 3D, import dello storico (D11),
adattatori grezzi (D12), nuovi stati di `Particao`, CSV, autenticazione.

### 14.5 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/export.rs` | mandato 3C.4.1 §3 | nuovo, pronto | **gruppo codice**: commit mirato dopo audit e autorizzazione di Luca |
| `src/xlsx.rs` | mandato §3 | nuovo, pronto | gruppo codice |
| `src/export_io.rs` | mandato §3 | nuovo, pronto | gruppo codice |
| `tests/export_releitura.rs` | mandato §3 | nuovo, pronto | gruppo codice |
| `script/releitura_export.py` | mandato §3, `EXP-29` | nuovo, pronto | gruppo codice |
| `src/lib.rs` | mandato §3 | modificato: tre `pub mod` e la mappa delle fasi | gruppo codice |
| `Cargo.toml` | mandato §2 | modificato, +1 riga | gruppo codice |
| `Cargo.lock` | conseguenza di `Cargo.toml` | modificato, dieci voci nuove spiegate in §19.2 | gruppo codice |
| `README.md` | mandato §3 | modificato: stato, contratti, API, conteggio, inventario `f64`, dipendenza, limiti | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato §3 | modificato: §7.4.1.1 con la decisione sulla divergenza nota, §19 appendice di esecuzione | **gruppo documentale**: commit documentale dopo audit e autorizzazione |
| `PROPOSTA_MANDATO_CLAUDE_FASE3C41_IMPLEMENTAZIONE_20261023.md` | Codex, con §11 approvazione, §12 primo passo, §13 decisione, §14 esito | continuità nominata | gruppo documentale |

Nessun file resta senza destino. Nessun commit, push, amend, rebase, reset,
restore, checkout, clean o stash eseguito. Nessun `git add`.

**LOTTO 3C.4.1 NON CHIUSO**: attende l'audit indipendente, e poi la
registrazione autorizzata nei due gruppi.

---

## 15. Appendice di esito — audit NON SUPERATO del 2026-10-30 e correzioni

| Voce | Valore |
|---|---|
| Rapporto | audit indipendente Codex del lotto 3C.4.1: **NON SUPERATO**, quattro rilievi `R3C41-01`…`R3C41-04` più due correzioni documentali non bloccanti |
| Mandato | invariato. Nessuna decisione approvata cambiata, nessun contratto indebolito, 3C.5 e 3D non avviate |
| Git | **nessun commit e nessun push** durante la correzione, come il rapporto prescrive. Nessun `add`, `reset`, `restore`, `checkout`, `clean`, `stash`, `amend` o `rebase` |
| Baseline | `9d99832`, `main...origin/main` allineati |
| Stato | controllo mirato del 2026-10-30: `R3C41-01`, `R3C41-02` e `R3C41-03` **CHIUSI**; un residuo su `R3C41-04`, chiuso in §16 |
| Dettaglio | §19.7 della specifica: un rilievo per riga, con il difetto reale, la correzione e la riproduzione |

### 15.1 I quattro rilievi

Ognuno riprodotto **prima** della correzione.

| Rilievo | Esito |
|---|---|
| **R3C41-01** (P1) | **confermato e corretto.** L'esclusione era solo lessicale, e l'alias 8.3 della fonte passava: con `Substituir` l'archivio veniva sostituito dall'export con esito riuscito. Ora due confronti: per **identità** canonica sui percorsi esistenti, e per chiave lessicale sui nomi che non esistono ancora. Regressione con l'alias 8.3 **reale** della fonte e di un membro della famiglia, chiesto a `cmd`; su un volume che non genera alias il test stampa `NON_ESEGUITO` invece di simulare. Controprova: disattivando il confronto per identità il test osserva una `Escrita` riuscita sull'archivio |
| **R3C41-02** (P1) | **confermato e corretto.** `U+FFFE` e `U+FFFF` non sono nella produzione `Char` di XML 1.0 e il modello li accettava. Due regressioni: una li rifiuta, l'altra fissa che tab, LF, CR, `U+D7FF`, `U+E000`, `U+FFFD` e `U+1FFFE` restano ammessi |
| **R3C41-03** (P2) | **confermato e corretto.** Il parent vuoto di un nome semplice non era la directory corrente, e un percorso legittimo veniva rifiutato per un errore di IO inventato. Nessun vincolo nuovo di soli percorsi assoluti, e il problema non è stato rinviato al chiamante CLI |
| **R3C41-04** (P2) | **confermato e corretto.** Il modo `export` confrontava 32 valori a campione; ora confronta **campo per campo** contro un manifesto del modello, più attese esplicite scritte a mano. 13 fogli, 411 celle, **992** confronti. Sette mutazioni di prova — totale testuale, totale numerico, chiave, stato, autore, motivo, indicatore — danno sette uscite con codice 1, ognuna nominando la cella |

Il confronto nuovo ha trovato **due** divergenze del lettore che il
campionamento non guardava, di cause distinte, registrate in §7.4.1.1.1 della
specifica: la frase del limite `ESCAPE_LEITOR`, che cita la sequenza protetta,
e il ritorno a capo nudo. Per il ritorno a capo: il modello lo accetta perché
`U+000D` è un carattere XML valido, il writer lo codifica come `_x000D_`
secondo `ST_Xstring` — verificato nel `xl/sharedStrings.xml` con `zipfile` — e
`openpyxl` 3.1.5 non decodifica quella sequenza. La fedeltà del **file** è
dimostrata, quella di **questo lettore** su un ritorno a capo no. Nessuna
compensazione nel writer, nessun dato rifiutato in più, e la frase del limite
**non** è stata riscritta per aggirare il lettore.

### 15.2 Le due correzioni documentali

| Dove | Correzione |
|---|---|
| `README.md`, questo mandato §14 | «nove pacchetti transitivi compilati e due bloccati» → dieci pacchetti nuovi nel lockfile, **otto** compilati su questo target, dipendenza diretta compresa, e due solo bloccati. `log` e `bumpalo` erano già nel lockfile: compilarli di nuovo non è aggiungere un pacchetto. §19.2 della specifica era già corretta |
| Specifica §6.3.9 | «sedici» → «quindici» nelle due frasi; l'elenco di quindici righe e la frase «Sono quindici» erano giusti |

### 15.3 Un difetto trovato da me, fuori dai rilievi

Nell'intestazione della specifica, **già committata** in `9d99832`: un
frammento di riga di tabella incollato davanti al titolo, e la riga `Stato`
senza barra di chiusura. Vengono da una mia giunzione mal fatta di una
revisione precedente. Nessun'altra riga del documento risulta troncata, e il
contenuto del frammento è già detto per intero in §17 e §19.6. Corretti
entrambi, dichiarati in §19.7 e nella revisione 10; nessun altro testo
rimosso.

### 15.4 Verifiche rieseguite

| Controllo | Esito |
|---|---|
| `cargo test --locked` | **386** test verdi: 364 nella lib, 3 nel binario, 19 di integrazione. I 337 precedenti restano verdi senza modifiche |
| `cargo fmt --check` | pulito |
| `cargo clippy --locked --all-targets -- -D warnings` | pulito. Due `collapsible_if` nel codice nuovo corretti con le catene `let` |
| `git diff --check` | pulito |
| Sei fonti canoniche | invariate, 6/6 |
| Perimetro escluso | intatto: `git status` non riporta nessun file fuori da quelli del mandato |
| Rilettura, modo `tx38` | 68 confronti, una divergenza nota, uscita **0** |
| Rilettura, modo `export` | 992 confronti, cinque divergenze note, zero discordanze, uscita **0** |
| Rilettura, modo `export` senza manifesto | uscita **2**, con «la verifica NON è stata eseguita» |

### 15.5 Stato Git alla consegna per il controllo mirato

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/export.rs` | mandato §3; correzione `R3C41-02` | nuovo, pronto | **gruppo codice**: commit mirato dopo il controllo mirato e l'autorizzazione di Luca |
| `src/export_io.rs` | mandato §3; correzioni `R3C41-01` e `R3C41-03` | nuovo, pronto | gruppo codice |
| `src/xlsx.rs` | mandato §3 | nuovo, pronto, invariato in questo giro | gruppo codice |
| `tests/export_releitura.rs` | mandato §3; `R3C41-04`: manifesto e caso del ritorno a capo | nuovo, pronto | gruppo codice |
| `script/releitura_export.py` | mandato §3, `EXP-29`; `R3C41-04`: confronto campo per campo | nuovo, pronto | gruppo codice |
| `src/lib.rs` | mandato §3 | modificato: tre `pub mod` e la mappa delle fasi | gruppo codice |
| `Cargo.toml` | mandato §2 | modificato, +1 riga | gruppo codice |
| `Cargo.lock` | conseguenza di `Cargo.toml` | modificato, dieci voci nuove | gruppo codice |
| `README.md` | mandato §3; correzioni documentali | modificato: conteggi, comando della rilettura, tabella delle divergenze note, conteggio dei pacchetti | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato §3; §19.7, §7.4.1.1.1, §6.3.9, intestazione | modificato | **gruppo documentale**: commit documentale dopo il controllo mirato e l'autorizzazione |
| `PROPOSTA_MANDATO_CLAUDE_FASE3C41_IMPLEMENTAZIONE_20261023.md` | Codex; §11 approvazione, §12 primo passo, §13 decisione, §14 esito, §15 questa appendice | continuità nominata | gruppo documentale |

File temporanei fuori dal repository, non versionati e non da versionare:
`%TEMP%\fantozzi_3c41\` (workbook depositati, manifesto, esiti della
rilettura), `%TEMP%\fantozzi_3c41_io\`, `%TEMP%\fantozzi_3c41_percurso\`,
`%TEMP%\fantozzi_3c41_mutazioni\` (le copie mutate della controprova) e gli
script di giunzione nella directory di lavoro della sessione.

Nessun file resta senza destino. Nessun file tracciato modificato resta non
classificato.

**LOTTO 3C.4.1 NON CHIUSO**: attende il controllo mirato dei quattro rilievi,
e poi la registrazione autorizzata nei due gruppi.

---

## 16. Appendice di esito — residuo del controllo mirato, chiuso (2026-10-30)

| Voce | Valore |
|---|---|
| Rapporto | controllo mirato Codex delle correzioni `R3C41-01`…`R3C41-04`: tre **chiusi**, un **residuo** su `R3C41-04` |
| Residuo | `Esito.confronta` concedeva l'eccezione di una divergenza nota sul **solo** risultato di `openpyxl`. Un file alterato con `_x005F_` passava come limite dichiarato |
| Perimetro | solo il confine di verifica. **Codice Rust invariato**, nessuna decisione nuova, D18/D20/D26 intatte, 3C.5 e 3D non avviate |
| Git | nessun commit, nessun push, nessun `add`, nessun reset/restore/checkout/clean/stash/amend/rebase |
| Stato | **RESIDUO CHIUSO, IN ATTESA DEL CONTROLLO DEL CONFINE DI VERIFICA** |
| Dettaglio | §7.4.1.1.2 della specifica per il criterio, §19.8 per le misure |

### 16.1 Riproduzione, prima della correzione

Copia del workbook depositato dai test, manifesto **invariato**, una sola
cella riscritta: voce nuova nelle stringhe condivise con il testo codificato
`visto e aceito a_x005F_x000D_b`, e `decisoes!2:motivo` indirizzata a quella
voce; ZIP riscritto con i CRC ricalcolati. Lo script di allora dava **411
celle, 992 confronti, 5 divergenze note, 0 discordanze, uscita 0**, concedendo
l'eccezione due volte alla cella alterata.

Il riscontro di Codex è confermato nei fatti, non solo accettato: la verifica
non distingueva un difetto del file dal limite del lettore che dichiara.

### 16.2 Correzione

| Elemento | Contenuto |
|---|---|
| `Pacote` | legge le celle delle **sole** eccezioni registrate, con `zipfile` e `xml.etree`; i fogli si risolvono dalle relazioni del pacchetto, non dall'ordine dei file |
| `decodifica_passata_singola` | `ST_Xstring` a passata singola: `a_x000D_b` è un ritorno a capo, `a_x005F_x000D_b` sono quindici caratteri letterali |
| `Esito._valuta_eccezione` | le cinque condizioni: modo, ingresso, lettore **e versione** della misura, risultato registrato, valore rappresentato dal pacchetto. Se una manca, discordanza con il motivo nominato |
| registrazioni | ognuna dichiara la cella da verificare; la riga del caso di `tx38` si calcola dall'elenco delle prove invece di essere scritta a mano |
| rapporto | stampa l'XML **misurato** in questo giro accanto a quello dichiarato alla registrazione |
| `script/controprova_rilettura.py` | nuovo: la prova permanente di ciò che la verifica **deve** respingere |

Nessuna deroga aggiunta, e il criterio vale per tutte le eccezioni registrate,
compresa quella della frase `ESCAPE_LEITOR`.

### 16.3 Accettazione, punto per punto

| Richiesta del controllo | Osservato |
|---|---|
| la baseline corretta mantiene il suo esito con i limiti dichiarati | uscita **0**, 992 confronti, 5 divergenze note, 0 discordanze |
| `masked_corruption` con manifesto originale esce con 1 e nomina `decisoes!2:motivo` | uscita **1**, e la discordanza dice «il **file** rappresenta `visto e aceito a_x000D_b` — nel pacchetto `visto e aceito a_x005F_x000D_b` — e non il valore atteso» |
| le sette mutazioni ordinarie restano respinte | uscita **1** in tutti e sette, con la cella nominata |
| l'assenza del manifesto resta uscita 2 | uscita **2** |
| stesso criterio alle altre eccezioni | `mascherata_escape` su `limites!5:descricao`, con un livello di protezione in meno: uscita **1** |
| nessuna trasformazione nel writer, nessun rifiuto nuovo di CR valido, D18/D20/D26 invariate | `src/xlsx.rs` e `src/export.rs` non toccati in questo giro |

In più, condizione 3 esercitata: una sonda carica lo script come modulo con la
versione di `openpyxl` sostituita da `9.9.9-sonda`, e tutte e cinque le
eccezioni diventano discordanze con «va rimisurata prima di essere concessa»,
uscita 1.

Modo `tx38` rieseguito: 68 confronti, una divergenza nota, uscita **0**, e
l'XML misurato coincide con quello dichiarato.

### 16.4 Verifiche del codice Rust, invariato

`cargo test --locked` **386** verdi · `cargo fmt --check` pulito · `cargo
clippy --locked --all-targets -- -D warnings` pulito · `git diff --check`
pulito · sei fonti canoniche 6/6 invariate.

### 16.5 Stato Git

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `script/releitura_export.py` | `R3C41-04`, residuo del controllo mirato | modificato: `Pacote`, decodifica a passata singola, cinque condizioni, celle nelle registrazioni | **gruppo codice**, dopo il controllo e l'autorizzazione |
| `script/controprova_rilettura.py` | prova del residuo | **nuovo** | gruppo codice |
| `src/export.rs`, `src/export_io.rs`, `src/xlsx.rs`, `tests/export_releitura.rs` | giri precedenti del lotto | nuovi, invariati in questo giro | gruppo codice |
| `src/lib.rs`, `Cargo.toml`, `Cargo.lock` | mandato §2/§3 | modificati nei giri precedenti | gruppo codice |
| `README.md` | esito del lotto | modificato: criterio delle eccezioni, comando della controprova, riga di contratto | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | §7.4.1.1.2, §19.8, revisione 11, stato | modificato | **gruppo documentale** |
| `PROPOSTA_MANDATO_CLAUDE_FASE3C41_IMPLEMENTAZIONE_20261023.md` | continuità nominata, §16 | modificato | gruppo documentale |

Un `script/__pycache__/releitura_export.cpython-313.pyc` è comparso quando la
sonda della versione ha caricato lo script come modulo. L'ho rimosso: è un
byproduct del mio stesso comando di oggi, rigenerabile, non una modifica di
nessuno. `.gitignore` copre solo `/target`, quindi se in futuro si esegue
quella sonda conviene aggiungere `__pycache__/`: lo propongo, non lo faccio in
questo giro perché è fuori perimetro.

Temporanei fuori dal repository: `%TEMP%\fantozzi_3c41\`,
`%TEMP%\fantozzi_3c41_io\`, `%TEMP%\fantozzi_3c41_percurso\`,
`%TEMP%\fantozzi_3c41_controprova\` (baseline, sette mutazioni e due copie
mascherate).

Nessun file tracciato modificato resta non classificato.

**LOTTO 3C.4.1 NON CHIUSO**: attende il controllo del confine di verifica, e
poi la registrazione autorizzata nei due gruppi.

---

## 17. Audit finale, pulizia e registrazione locale autorizzata (2026-10-30)

Questa sezione aggiorna lo stato corrente; gli esiti di attesa delle appendici
precedenti descrivono il momento in cui furono scritte.

**Audit indipendente superato.** Il controllo finale di Codex accetta la
baseline con i limiti noti, respinge tutte le alterazioni e il precedente
controesempio indipendente, e non concede eccezioni a una versione del lettore
non misurata. R3C41-01..04 sono chiusi. Codice Rust e test di integrazione
identici tramite SHA-256 alla baseline dei 386 test gia' verificati; nessuna
nuova build di Codex durante la registrazione. La successiva riesecuzione
completa riferita da Claude ha confermato gli stessi esiti.

**Autorizzazione:** Luca, «PRECODIAMO», dopo la proposta esplicita di eliminare
la sola cache `C:\t3c41`, conservare `target` e registrare i due gruppi. Il
push non e' compreso. Nessun'altra cancellazione e nessuna nuova fase sono
autorizzate da questo atto.

**Pulizia eseguita:** `C:\t3c41` risolto e verificato come directory ordinaria,
senza reparse point al suo interno, contenente soltanto `debug`, `tmp`,
`.rustc_info.json` e `CACHEDIR.TAG` con firma Cargo valida. Rimossi 3.224 file
di cache, circa 1,32 GiB logici; directory eliminata, `target` del progetto
conservato. Spazio libero osservato dopo la pulizia: **9,34 GiB**. Nessun
sorgente, campione o archivio reale cancellato.

### 17.1 Classificazione prima dei commit

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `Cargo.toml` | §2, dipendenza autorizzata | pronto, versione puntuale | commit codice |
| `Cargo.lock` | risoluzione della dipendenza | pronto, dieci pacchetti nuovi | commit codice |
| `src/lib.rs` | §3, esportazioni e mappa fasi | pronto | commit codice |
| `src/export.rs` | §3 e R3C41-02 | audit superato | commit codice |
| `src/export_io.rs` | §3 e R3C41-01/03 | audit superato | commit codice |
| `src/xlsx.rs` | §3, adattatore approvato | audit superato | commit codice |
| `tests/export_releitura.rs` | §3, percorso composto e manifesto | verificato | commit codice |
| `script/releitura_export.py` | EXP-29 e residuo R3C41-04 | controllo finale superato | commit codice |
| `script/controprova_rilettura.py` | regressione permanente del residuo | eseguita, casi conformi | commit codice |
| `README.md` | stato reale e contratti del lotto | pronto, audit finale registrato | commit codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | appendici di esecuzione e audit | pronta, revisione 12 | commit documentale |
| questo mandato | continuita', approvazioni ed esiti | pronto, §17 aggiunta | commit documentale |

Non si modificano ulteriormente codice, dipendenze o contratti. Le sei fonti
canoniche restano quelle verificate dall'audit. Prima e dopo la registrazione
si controllano status, diff e indice; staging nominativo, nessun `git add .`.

### 17.2 Limiti e passo successivo

La prova manuale in Microsoft Excel resta NON_ESEGUITA; la pubblicazione dei
commit resta da autorizzare. Nessun avvio di CLI o DRE, nessuna promozione
contabile. L'eventuale regola `.gitignore` per `__pycache__` resta una pulizia
separata: non e' stata aggiunta durante questa registrazione.

Gli SHA e lo stato corrente dei commit si leggono da Git, senza creare un
ciclo di commit per riscrivere nel documento il suo stesso identificativo.

---

## 18. Appendice di esito — `R3C41-05`, fonte di `status_tecnico` (2026-10-30)

| Voce | Valore |
|---|---|
| Origine del riscontro | la **prova visiva** di `TX-18`: il print del foglio `diagnostico` mostrato a Luca, `Z2 = APROVADO` dove l'esito tecnico del campione è `OK` |
| Classificazione | MICRO, P2: correzione di un contratto già deciso (`F3C-29`). Nessuna decisione funzionale nuova, nessuna nuova fase |
| Perimetro toccato | `src/export.rs` (una fonte), le sue regressioni, le attese della rilettura, la controprova, e le appendici. **Nessuna** modifica al nucleo contabile, allo schema, alle dipendenze o ai contratti |
| Stato | **CHIUSO** dal controllo mirato del 2026-10-30: §19. Registrazione e pubblicazione non autorizzate |
| Dettaglio | §19.10 della specifica per le misure, §6.3.3 per il contratto ora esplicito |
| Git | nessun commit, nessun push. I due commit locali restano `f6aeb3d` e `f89e702` |

### 18.1 Il difetto, riprodotto prima della correzione

`diagnostico.status_tecnico` era riempita da `lancamento.status_revisao`, cioè
dalla copia che la 3C.3 **proietta** per costruire il pre-libro, dove
un'approvazione diventa `APROVADO`. La fonte corretta,
`LinhaRevisada::status_tecnico`, era già nella vista ricevuta.

Tre regressioni scritte contro il contratto hanno fallito tutte e tre:
`OK` compariva come `APROVADO`, e — il caso che conta — un `REVISAR` tecnico
veniva **mascherato** da un'approvazione umana, nascondendo nell'export
l'esigenza di revisione.

Una delle tre attese era **mia** e sbagliata: lo stato della vista senza eventi
si chiama `TECNICO_OK`, non `OK`, e il codice lo scriveva già bene. Corretta
l'attesa, non il codice.

### 18.2 Che cosa è stato aggiunto

| Prova | Contenuto |
|---|---|
| `r5_status_tecnico_ok_non_diventa_aprovado` | tecnico `OK` + approvazione → `OK`, `APROVADO_POR_PESSOA`, `APROVADO`, importo e inclusione invariati |
| `r5_status_tecnico_revisar_resta_revisar_dopo_approvazione` | tecnico `REVISAR` + approvazione → il tecnico resta `REVISAR` |
| `r5_senza_eventi_il_tecnico_e_lo_stato_concordano` | senza eventi: `OK` e `TECNICO_OK`, perché la fonte non sia costante per caso |
| attese esplicite della rilettura | `status_tecnico = OK`, `estado_revisao = APROVADO_POR_PESSOA`, `particao = APROVADO`: `Z2`, `AA2`, `AB2` del print |
| controprova del confine | mutazione `stato_tecnico`: `APROVADO` nel workbook → uscita 1 con la cella nominata |

### 18.3 Esito nel campione

`Z2 = OK`, `AA2 = APROVADO_POR_PESSOA`, `AB2 = APROVADO`, con `V2 = 10.00`,
`W2 = 10` e `no_extrato_revisado = SIM`: importo, quadrature e inclusione
nell'estratto non cambiano. Campione e manifesto rigenerati dai test.

### 18.4 Verifiche

| Controllo | Esito |
|---|---|
| `cargo test --locked` | **389** verdi: 367 lib, 3 binario, 19 integrazione. Nessun test preesistente ha dovuto cambiare |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti |
| Rilettura `tx38` | uscita 0 |
| Rilettura `export` | 411 celle, **998** confronti, 5 divergenze note, 0 discordanze, uscita 0 |
| Rilettura `export` senza manifesto | uscita 2 |
| Controprova del confine | dodici casi, tutti conformi |

### 18.5 Stato Git

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/export.rs` | `R3C41-05` | modificato: una fonte, tre regressioni, un helper di test | **gruppo codice**, dopo il controllo e l'autorizzazione |
| `script/releitura_export.py` | `R3C41-05` punto 5 | modificato: tre attese esplicite | gruppo codice |
| `script/controprova_rilettura.py` | `R3C41-05` | modificato: mutazione `stato_tecnico` | gruppo codice |
| `README.md` | conteggi reali | modificato: 389 test, 998 confronti | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | §6.3.3, §19.10, revisione 13, stato | modificato | **gruppo documentale** |
| questo mandato | continuità, §18 | modificato | gruppo documentale |

Gli altri file del lotto sono già registrati in `f6aeb3d` e `f89e702` e non
sono stati toccati. Nessun file tracciato modificato resta non classificato.
Nessun `add`, commit, push, reset, restore, checkout, clean, stash, amend o
rebase.

### 18.6 Limiti

`TX-18` resta **in corso**: la prova visiva ha confermato `V2`, `W2`, lo stato
della colonna numerica e le due stringhe ostili del campione piccolo, e ha
fatto emergere questo riscontro; le altre celle non sono state guardate.
`TX-19` resta **NON_ESEGUITA**. Gli altri limiti dichiarati — ZIP64, volumi
reali, hard link, altri lettori, durabilità — restano tali.

---

## 19. Esito del controllo mirato su `R3C41-05` (2026-10-30)

**Chiuso nel perimetro mirato.** Codex ha verificato la fonte, le tre
regressioni, il campione rigenerato, i 998 confronti della rilettura e i dodici
casi della controprova, con `git diff --check` pulito. Nessuna modifica
richiesta al codice.

Le provenienze restano distinte: la suite completa di **389** test, `fmt`,
`clippy` e la ricompilazione da cache fredda sono attestazioni della mia
consegna, non rieseguite da Codex in questo controllo MICRO.

**Il SHA del campione.** `a5645236…` pubblicato per
`EXPORT_PROVA_3C41_CORRETTO_R5.xlsx` coincide con il workbook depositato in
`%TEMP%\fantozzi_3c41\`: la copia per i print viene dal codice corretto.
Quel SHA identifica però **quella copia** e non il contenuto: il foglio
`proveniencia` porta `arquivo_caminho`, che contiene l'arena del test con
l'identificativo del processo, quindi due esecuzioni danno due SHA — la mia
dava `f1b077a2…`. È `D23`: determinismo dei contenuti e dell'ordine, non dei
byte dello ZIP. Dettaglio in §19.10.1 della specifica.

### 19.1 Che cosa resta da autorizzare

| Voce | Stato |
|---|---|
| registrazione della correzione `R3C41-05` nei due gruppi | **autorizzata da Luca il 2026-10-30** ed eseguita: §20 |
| pubblicazione dei commit locali | **autorizzata da Luca il 2026-10-30** ed eseguita: §20 |
| `TX-18`, prova visiva in Microsoft Excel | **parziale**: confermate `V2`, `W2`, lo stato della colonna numerica, le due stringhe ostili e, dopo la correzione, `Z2`/`AA2`/`AB2`. Le altre celle non sono state guardate. Per i prossimi print va usata la copia nuova: un workbook già aperto in Excel non si aggiorna da sé |
| `TX-19`, decodifica di `_xHHHH_` in Excel | **NON_ESEGUITA** |
| fase 3C | **non chiusa**. CLI 3C.5, DRE 3D e la promozione contabile non sono avviati |
| regola `.gitignore` per `__pycache__` | proposta, non applicata: pulizia separata |

Gli altri limiti dichiarati — ZIP64, volumi reali, hard link, durabilità, altri
lettori — restano tali e nessuna nuova deroga è stata aggiunta.

---

## 20. Registrazione e pubblicazione autorizzate (2026-10-30)

**Autorizzazione:** Luca, alla domanda sui sei file modificati di `R3C41-05`,
ha scelto «due commit e push dei tre». Autorizza quindi la registrazione nei
due gruppi tematici **e** la pubblicazione dei commit locali, che l'audit di
Codex lasciava esplicitamente fuori dal proprio perimetro.

**Procedura seguita**, come le regole di lavoro richiedono:

| Passo | Contenuto |
|---|---|
| prima | `git status -sb`, `git diff --name-status`, `git diff --cached --name-status` |
| staging | **nominativo**, file per file. Nessun `git add .` |
| gruppo codice | `src/export.rs`, `script/releitura_export.py`, `script/controprova_rilettura.py`, `README.md` |
| gruppo documentale | `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md`, questo mandato |
| dopo ogni commit | `git status -sb` e indice controllati |
| pubblicazione | `git push` del ramo `main`, senza forzature |

Nessun commit misto, nessun reset, restore, checkout, clean, stash, amend,
rebase o riscrittura di storia. Gli SHA si leggono da Git: non vengono
ricopiati qui, per non aprire un ciclo di commit che riscriva il proprio
identificativo.

**Che cosa questa pubblicazione non dichiara.** `TX-18` resta parziale e
`TX-19` **NON_ESEGUITA**: la compatibilità con Microsoft Excel non si dichiara
stabile. La fase 3C **non** è chiusa; CLI 3C.5, DRE 3D e la promozione
contabile non sono avviati. Restano in piedi i limiti dichiarati nel foglio
`limites` e in §19.6 della specifica, comprese le due divergenze note del
lettore.

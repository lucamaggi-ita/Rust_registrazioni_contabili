# Mandato operativo a Claude — implementazione 3C.3.b.1

Data: 2026-10-22. Repository: `Rust_registrazioni_contabili`.
Workspace: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Ruoli: Claude implementa e verifica; Codex esegue l'audit indipendente; Luca autorizza registrazione e pubblicazione.
Metodo: `programmatore-bravo`. Prima contrattualizza, poi programma.

## 1. Autorizzazione e obiettivo

Luca ha risposto **«PROCEDI»** alla preparazione del mandato implementativo 3C.3.b.1, comprendente nuovi moduli di persistenza, test, aggiornamenti mirati e aggiunta esplicita di `serde_json = "1.0.154"`. Codex prepara soltanto questo Markdown.

**Alla consegna di questo mandato, implementa il lotto 3C.3.b.1 secondo la specifica già auditata e le decisioni approvate.** Non ripetere l'analisi generale né chiedere di riapprovare D7, D8/D9 o D14–D17.

Obiettivo: conservare su archivio locale versionato i contesti originali e le dichiarazioni degli eventi, rileggerli e ricostruire la stessa revisione in una nuova esecuzione, con gestione verificabile di errori, recupero e scrittori cooperanti.

Lavoro STRUTTURALE sul confine di persistenza, applicato a contratti già definiti. Sono BLOCCANTE_ORA perdita o arrotondamento dei valori, perdita della storia, approvazioni trasferite impropriamente, scritture che aggirano i controlli della base e risultati non riproducibili. I debiti estranei restano DA_PIANIFICARE.

Non sono autorizzati commit, push, riscrittura della storia, uso di archivi contabili reali, installazioni, export/import Excel, CLI, DRE, autenticazione o produzione. Completa codice, test e documentazione ammessi, poi consegna per audit Codex. La disponibilità della libreria su disco non significa ancora che la contabilità sia pronta all'uso operativo.

## 2. Baseline e fonte normativa

- HEAD verificato: **`27eb6083b75643b02adca84cfa7373911e847950`**. Prima di questo mandato: `main...origin/main`, worktree pulito, nessuno staged. All'inizio del lavoro rileggi lo stato effettivo; non sovrascrivere eventuali modifiche intervenute.
- Fonte normativa: **`SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` a `27eb608`**, revisione 3 con decisioni registrate in §17, audit tecnico superato e riscontri R3C3B-01…04 chiusi nella specifica.
- Contratti da implementare: **PER-01…PER-24**. Prove richieste: **TP-01…TP-22 più TP-15b**. Sono condizioni da coprire, non una quota di ventitré funzioni di test. Nessuno di questi test è già implementato per il writer.
- Baseline software riusata: **267 test passati**, 251 lib + 3 bin + 13 CLI, dall'audit finale 3C.3.a. Nessun sorgente è cambiato nei successivi commit documentali. Non dichiarare una nuova esecuzione finché non la fai.
- C18–C24, contratti della 3C.2 e F3C-28…46 della 3C.3.a restano invariati. I moduli puri esistenti sono la fonte delle logiche di allocazione, impronta e revisione, non da riscrivere nel loader.

La specifica conserva sezioni storiche e contratti etichettati PROPOSTI perché il codice non esisteva. Lo stato delle decisioni è quello di §17; il comportamento richiesto è quello della revisione 3, comprese §7.3, §8.4, §9.4.2 e le API allineate. Non interpretare una fotografia storica come una decisione riaperta.

## 3. Decisioni già approvate e dipendenza autorizzata

| Decisione | Regola da applicare |
|---|---|
| D7(a) | archivio strutturato autorevole, workbook derivato; nessuna seconda fonte autorevole |
| D14(a) | un file JSON per archivio; temporanei, generazioni e prove del recupero hanno il ruolo definito dalla specifica |
| D15(a) | lock cooperativo e base completa verificata; confronto dell'accrescimento contro il corrente riletto sotto lock |
| D16(a) | nessuna potatura automatica delle generazioni o delle prove conservate |
| D17(a) | nessuna numerazione storica automatica dei documenti; D8.5 invariata, contatore dell'archivio distinto |

**È autorizzata l'aggiunta diretta di `serde_json = "1.0.154"` a `[dependencies]`**, con le feature predefinite. La versione da risolvere nel lockfile per questo lotto è **1.0.154**, già disponibile offline. Non aggiungere feature o altre dipendenze per comodità; in particolare nessun percorso monetario in virgola mobile.

Aggiorna Cargo soltanto quanto serve a questa aggiunta. Nessun `cargo update` generale, upgrade o downgrade delle dipendenze esistenti. Spiega ogni differenza di `Cargo.lock`; non rigenerarlo da zero. Dopo l'aggiunta usa la modalità offline per risolvere/compilare; i controlli finali devono anche rispettare `--locked`. Se emerge una dipendenza ulteriore indispensabile, dimostra il problema e proponi il minimo, senza installarla o introdurla di nascosto.

## 4. File ammessi

| File | Intervento ammesso |
|---|---|
| `src/arquivo.rs` | nuovo: schema/DTO restrittivi, codec, conversioni validate, verifiche di struttura e accrescimento, ordinamento fisico |
| `src/arquivo_io.rs` | nuovo: lettura, pubblicazione, lock, osservazione, candidati, recupero e sblocco esplicito |
| `src/lib.rs` | esportazioni dei due moduli e allineamento minimo della mappa delle fasi |
| `tests/persistencia_entre_processos.rs` | nuovo test di integrazione: prova fra processi distinti e controlli attraverso l'API pubblica |
| `Cargo.toml`, `Cargo.lock` | sola aggiunta autorizzata di `serde_json`, con relativa risoluzione mirata |
| `README.md` | stato reale, contratti coperti, API, limiti e numero effettivo dei test; nessuna rinumerazione generale dei contratti |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | appendice di esecuzione e aggiornamento puntuale dello stato/autorizzazione della dipendenza; preservare schema, decisioni, alternative e prove storiche |
| questo mandato | eventuale appendice breve degli esiti, conservando l'autorizzazione originale |

I test unitari dei nuovi moduli possono usare moduli privati interni allo stesso file. Piccoli dati sintetici vanno costruiti nei test, con provenienza dichiarata; non è necessario aggiungere fixture estese o dipendenze per i test.

**Non modificare** `src/revisao.rs`, `src/impressao.rs`, `src/prelivro.rs`, `src/alocacao.rs`, `src/dinheiro.rs`, `src/modelos.rs`, `src/nfe.rs`, `src/regras.rs`, `src/main.rs`, gli altri sorgenti, i test e le fixture preesistenti, `config/*`, `samples/*`, il repository Python o i mandati storici. Nessun nuovo comando CLI o binario applicativo.

Le normali scelte tecniche di tipi, helper privati e dettagli degli errori entro questi confini non richiedono un nuovo sì. Se un requisito risulta non implementabile mantenendo i confini, dimostralo con un caso minimo: non cambiare silenziosamente un contratto o un modulo già auditato.

## 5. Implementazione richiesta

### A. Codec puro e conservazione

- Implementare il nuovo lettore con DTO restrittivi su tutto l'albero. Non passare prima per un `serde_json::Value` che collassi le chiavi duplicate. Rifiutare campi sconosciuti e duplicati anche annidati, con diagnostica che identifichi il punto dell'errore.
- Tutti i `Decimal` sono testi: conversione senza perdita con `from_str_exact` e confronto del testo riprodotto, poi costruzione del modello. Nessuna scala 2 applicata a quantità o prezzi unitari; nessuna perdita di un valore piccolo in zero. Conservare i valori validi e le loro scale.
- Passare per `Dado::novo`, validazione della competenza ed enum chiusi. Non sostituire mancanze con zero, stringa vuota o stato inventato. Applicare la policy per campo su omesso/`null` di §7.4.
- Conservare originali, riconciliazione completa, esecuzioni con il loro piano, snapshot anche senza righe, avvisi, rate e molteplicità delle dichiarazioni. Gli snapshot precedono la proiezione umana.
- Codificare gli istanti con orario locale, nanosecondi e offset in secondi. Non usare il `Serialize` ordinario di `DateTime<FixedOffset>` per persisterli. Il testo esatto dell'evento deve restare identico prima e dopo la lettura.
- Ricalcolare e confrontare le impronte richieste, senza correggerle automaticamente per far accettare un contenuto alterato. Separare struttura e confronto della presentazione come nella specifica; un nome di conto cambiato segue D8.3.
- Ordinare il file secondo §8.4. Il livello finale degli eventi è il **record archivistico interamente serializzato**, includendo i metadati che non appartengono alla dichiarazione umana. Conservare entrambi i record con digest assente/presente corretto; nessun riempimento silenzioso o scarto. Gli array a ordine memorizzato restano tali.
- Mantenere `arquivo.rs` puro: nessun caricamento del piano, percorso implicito, accesso al filesystem o orologio nel codice di produzione di questo modulo.

### B. IO, pubblicazione e recupero

- Implementare le API della specifica con errori nominati, validazione del candidato e del corrente e gestione esplicita degli stati. Le firme possono ricevere adattamenti tecnici minimi motivati, preservando tutti i dati necessari e le garanzie.
- Per la pubblicazione ordinaria, prendere il lock cooperativo, rileggere e validare il corrente, verificare il `TokenBase` completo e confrontare l'accrescimento contro quel contenuto. Un candidato da un altro archivio con lo stesso contatore non deve passare.
- L'append-only confronta anche la molteplicità degli eventi e i metadati storici protetti; l'identità del piano appartiene all'esecuzione. Non usare l'hash C23 come criterio di scarto.
- Il pubblicatore assegna la revisione con incremento controllato sotto lock e serializza dopo l'assegnazione. La prima pubblicazione usa il ramo `base = None`, revisione 1 e nessuna generazione precedente inventata.
- Temporanei e generazioni non possono troncare residui esistenti: `create_new`, nomi e risultati espliciti come nel protocollo auditato. Scrittura e sincronizzazione del candidato precedono il punto di commit; nessuna cancellazione del corrente prima della sostituzione.
- Distinguere errore prima del commit, pubblicazione incerta e pubblicazione riuscita con lock residuo. Un errore di rilascio non deve indurre il chiamante a credere che il commit non sia avvenuto.
- Il recupero riceve `EstadoObservado`, prende il lock e confronta lo stato effettivo **prima** di conservare il corrente o promuovere il candidato. Coprire assente/presente e byte di un corrente corrotto. `EstadoMudou` impedisce interventi sul corrente e sul candidato; una nuova osservazione rende possibile il rollback intenzionale.
- `candidatos` elenca e valida senza ripieghi automatici; `liberar` richiede conferma esplicita del lock. D16 vieta la potatura automatica delle generazioni: lo sblocco previsto dal protocollo non è una cancellazione dello storico.
- Nei test e nelle normali scelte tecniche conservare le prove di recupero: un nome già usato non deve portare a sovrascriverle. Errori di percorso e lock devono nominare l'operazione e la fonte, senza interpretare un errore IO come archivio vuoto.

## 6. Prove richieste sul codice reale

Implementare la matrice TP-01…TP-22 e TP-15b con una mappa esplicita identificatore → test/casi effettivi. I casi possono essere parametrizzati o condividere helper; il numero finale deriva dai test esistenti, non è una quota da raggiungere.

Le sonde della specifica provano fattibilità e controesempi: **non sostituiscono** i test delle nuove API. In particolare:

1. **Codec:** rifiuto di numeri JSON al posto dei decimali, stringhe oltre precisione, notazione non canonica, campi sconosciuti, chiavi duplicate annidate e stati incoerenti. Conservare `MAX`/`MIN`, centesimi, segni e scale valide, quantità, `Dado`, Unicode originale e identificativi con zeri iniziali.
2. **Tempo ed eventi:** offset di 1 e 2 secondi, nanosecondi, stesso istante con offset dichiarati diversi, note differenti, digest assente/presente corretto e ripetizioni reali. Confrontare `texto_exato_evento`, offset e precisione, non soltanto l'uguaglianza dell'istante assoluto.
3. **Revisione:** persistere e rileggere approvazione, revoca e ri-approvazione, compresa la sanatoria di un `APROVADO` senza provenienza. Stessi originali, riconciliazione, traccia ed estratto; nessuna approvazione attiva dopo la revoca.
4. **Storico:** stesso contenuto in altro lotto, contenuto corretto a parità di totale/hash C23, righe con payload ripetuto, metadati storici e piano diversi per esecuzioni diverse, due dichiarazioni identiche che non possono ridursi a una sul disco.
5. **Ciclo reale:** destinazione nuova, seconda pubblicazione, fallimenti prima del commit, temporanei/generazioni già presenti, lock occupato o residuo e corrente corrotto. Usare il vero codice IO, verificando i byte del precedente e l'assenza di troncamenti. Il solo allestimento manuale di file non prova tutti i passaggi del writer.
6. **Base e concorrenza:** due chiamanti cooperanti dalla stessa base, base di un altro archivio con identico contatore e candidato che omette storia del corrente. Per l'esclusione tra processi, esercitare un lock realmente tenuto, non solo un confronto di numeri in memoria.
7. **Recupero:** TP-15b nelle tre transizioni corrotto→valido, assente→presente e valido→valido con revisione diversa. Verificare `EstadoMudou` prima di copia/promozione, poi nuova osservazione e recupero intenzionale riuscito con il precedente conservato.
8. **Determinismo:** due serializzazioni della stessa revisione e permutazioni degli array ordinabili producono gli stessi byte; conservare gli array a ordine memorizzato e tutte le occorrenze. Nessuna conversione di `null` a digest calcolato per eliminare un pareggio.

### Prova fra processi e controllo indipendente

È autorizzato il metodo TP-19: test di integrazione che riesegue il proprio eseguibile di test con `std::process::Command`, filtro del test figlio e percorso di prova esplicito. Il processo scrittore termina, il lettore parte come processo distinto; nessun nuovo binario applicativo o comando CLI.

Le eventuali variabili d'ambiente servono soltanto al protocollo del test figlio e vanno passate al `Command`, senza mutare globalmente l'ambiente dei test concorrenti. Directory e metadati di prova sono espliciti; non diventano default di produzione.

Il confronto deve coprire valori originali, scale, byte testuali, offset, testo esatto degli eventi e vista ricostruita a parità di piano. Non usare come unica prova un JSON della vista che potrebbe a sua volta perdere precisione temporale. Il test deve fallire se scrittura o lettura nel processo figlio falliscono.

TP-20 usa un lettore diverso per controllare il file fisico. Sono autorizzati gli strumenti locali già disponibili, senza installazioni. Dove il parser indipendente converte date o numeri, controllare anche il testo grezzo: l'indipendenza dello strumento non ne garantisce la precisione.

Test di errori e interruzioni possono usare file sintetici e processi di test creati appositamente. Non chiudere processi utente né operare su archivi reali. Per il fallimento della sostituzione su Windows usare un handle senza condivisione della cancellazione o tecnica equivalente, dichiarando ciò che è stato realmente provato.

Ogni variante di errore raggiungibile deve avere una prova pertinente che controlli variante, operazione e dati di contesto. Per un ramo dimostrabilmente irraggiungibile, documentare il percorso e la protezione che lo precede: non introdurre dati impossibili o dichiarare una copertura totale non dimostrata.

## 7. Sequenza e controlli finali

Prima di modificare, dichiarare piano minimo, gruppi di file, contratto applicato, test e rischi. Rilevare lo stato Git e gli hash delle **sei fonti canoniche**: cinque YAML in `config/` e XML canonico in `samples/nfe_exemplo/`; riutilizzare gli hash attesi dei precedenti audit se disponibili e confrontarli anche a fine lavoro.

Procedere in modo verificabile: codec e test mirati, quindi IO e test del protocollo, quindi prova fra processi e controllo indipendente. Non chiedere una nuova approvazione per passare tra questi passaggi già inclusi nel lotto. Se emerge una violazione, riprodurla, correggerla e conservarne una regressione.

Alla consegna eseguire:

```text
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
git diff --check
git status -sb
git diff --name-status
git diff --cached --name-status
```

I 267 test preesistenti devono restare verdi senza modifiche ai loro file. Verificare hash delle sei fonti invariati, perimetro escluso intatto, nessun percorso monetario in float, codec di produzione privo di IO e nessuna fonte implicita per autore/istante.

Non nascondere avvisi con soppressioni per dichiarare i controlli puliti. Distinguere i rilievi sul codice dagli eventuali messaggi ambientali della cache incrementale o della conversione LF/CRLF. Non ripetere suite ampie dopo il passaggio senza nuove modifiche o dubbi concreti.

## 8. Documentazione e limiti da conservare

README e appendice di esecuzione devono descrivere il comportamento realmente implementato, API disponibili, corrispondenza PER/TP, test effettivi, verifiche IO, differenze tecniche motivate e stato **consegnato per audit**. Non dichiarare superato l'audit indipendente prima di Codex.

Aggiornare puntualmente le diciture storiche «dipendenza da autorizzare» e «writer inesistente» dove indicano lo stato corrente, citando questo mandato e la nuova appendice; non riscrivere le prove storiche o le decisioni. Nessuna rinumerazione generale in C25+ né nuovo documento di analisi sostitutivo.

Conservare i limiti: durabilità dopo perdita di alimentazione NON_VERIFICATA; ambito dei filesystem effettivamente provati; protezione degli scrittori cooperanti; autore dichiarato e non autenticato; spazio crescente senza potatura; numerazione storica rinviata. La riuscita di `rename`/`sync_all` o di una prova dopo arresto del processo non dimostra la resistenza a ogni perdita di alimentazione.

3C.4, 3C.5 e 3D restano fuori; D11 e D12 restano aperte. Nessuna promozione in produzione contabile e nessuna promessa operativa sull'archivio reale.

## 9. Git e resoconto di consegna

Nessun `git add .`, nessun commit misto, nessun commit o push in questo giro. Nessun reset, restore, checkout, clean, stash, amend o rebase autorizzato. Non scartare modifiche incontrate o prove del lavoro senza una destinazione esplicita.

Consegnare: modifiche; contratti applicati; test realmente eseguiti e mappa PER/TP; hash e perimetro verificati; audit interno e limiti; differenze dal piano; stato Git con tabella **File / Origine-mandato / Stato / Destino** per ogni tracked modificato e untracked. Tutto deve avere un destino nominato.

Destini proposti dopo l'audit e l'autorizzazione futura: gruppo codice (`arquivo`, IO, esportazioni, dipendenza, test, README) in un commit mirato; appendice di esecuzione della specifica e mandato in un commit documentale. Non eseguirli ora.

**Il lotto non è chiuso alla sola consegna dei test verdi:** richiede audit indipendente e successiva registrazione/pubblicazione autorizzata, oppure una continuità nominata che classifichi il lavoro ancora aperto. Non avviare fasi successive per occupare l'attesa dell'audit.

## 10. Continuità della preparazione Codex

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3C3B1_IMPLEMENTAZIONE_20261022.md` | «PROCEDI» di Luca | nuovo mandato implementativo, unico file creato da Codex | consegna a Claude; conservato per audit e futura registrazione documentale autorizzata |

Nessun codice o dipendenza modificato da Codex per preparare questo documento. La baseline dei test è riusata, non rieseguita. Nessun commit, push o archivio reale toccato.

---

## 11. Appendice di esito — eseguito da Claude il 2026-10-22

Aggiunta dopo l'esecuzione del mandato. Il testo sopra non è stato modificato.

| Voce | Valore |
|---|---|
| Baseline | `27eb6083b75643b02adca84cfa7373911e847950`, verificata prima di toccare qualunque file |
| Stato | **CORRETTO DOPO L'AUDIT DEL 2026-10-22, IN ATTESA DI AUDIT MIRATO**. I quattro riscontri R3C3B1-01…04 sono chiusi, con regressioni permanenti: §12. Nessun commit, nessun push |
| File nuovi | `src/arquivo.rs`, `src/arquivo_io.rs`, `tests/persistencia_entre_processos.rs` |
| File modificati | `Cargo.toml` (+1 riga), `Cargo.lock` (+1 riga), `src/lib.rs` (+2 righe) |
| Dipendenza | `serde_json = "1.0.154"`, quella autorizzata da §1 del mandato. Nessun'altra |
| Test | 337 totali (319 lib, 3 binario, 15 integrazione). 70 nuovi, di cui 13 regressioni dell'audit; i 267 precedenti verdi senza modifiche |
| Verifiche | `cargo test --locked`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check`: tutte pulite |
| Perimetro escluso | intatto: `git diff --name-status` elenca soltanto i tre file sopra. Le sei impronte canoniche di `config/*` e `samples/*` sono invariate |
| Mappa PER/TP | §18.3 di `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` |

### Differenze dal piano del mandato

Tre, tutte in aggiunta e nessuna in riduzione:

1. **`OperacaoDto` locale.** `OperacaoRevisao` deriva solo `Serialize` e
   `revisao.rs` è nel perimetro escluso, quindi il codec ha un enum locale con
   le stesse etichette. Il test `etiquetas_dos_enums_do_arquivo` le confronta
   con `OperacaoRevisao::rotulo()`, così una deriva futura fallisce invece di
   cambiare il formato in silenzio.
2. **Helper `sincronizar` in `arquivo_io.rs`.** Non era previsto: serve perché
   su Windows `sync_all` su un handle di sola lettura è negato (§18.4 della
   specifica). Senza di esso **ogni** pubblicazione con una generazione da
   conservare falliva.
3. **Un test in più sul rollback.** `tp15b_rollback_intencional_e_declarado_nao_renumerado`
   registra che riprendere dopo un rollback richiede un gesto umano sul
   residuo e sulla generazione conservata. È una conseguenza di §9.3 e di D16,
   non un difetto: la libreria non cancella candidati.

### Audit interno, prima della consegna

| Controllo | Esito |
|---|---|
| i test sono stati scritti contro il contratto, non contro il codice | due test hanno fallito al primo giro e hanno trovato un difetto reale (`sync_all`) e un'aspettativa sbagliata mia (il residuo che resta per contratto) |
| nessun motivo di errore supposto | gli undici motivi di rifiuto dei decimali sono misurati uno per uno, tabella in §18.4 |
| nessun fallback silenzioso | ogni percorso d'errore ha una variante nominata; nessuna correzione automatica di un archivio alterato |
| nessuna impronta aggiornata per accettare un contenuto cambiato | `tp12_impressao_ou_conteudo_alterados_sao_rejeitados` |
| il controllo fisico non si confronta con sé stesso | i byte sono esaminati senza codec, e la controprova è di uno strumento esterno (PowerShell), §18.5 |
| nessuna cancellazione fuori dalle cartelle temporanee dei test | le arene stanno sotto `%TEMP%\fantozzi_3c3b1\`, una per test |

### Limiti dichiarati, non risolti

`[NON_VERIFICATO]`: durabilità dopo perdita di alimentazione (TP-08);
`lucchetto_residuo = true`, che esigerebbe l'iniezione di un guasto fra i passi
7 e 8; comportamento su volumi reali; mutua esclusione contro scrittori non
cooperanti, esclusa da D15(a). L'autore resta **dichiarato**, non autenticato.
L'archivio cresce senza limite per D16(a).

### Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/arquivo.rs` | mandato 3C.3.b.1 §2 | nuovo, pronto | commit mirato del gruppo codice, dopo audit e autorizzazione |
| `src/arquivo_io.rs` | mandato 3C.3.b.1 §2 | nuovo, pronto | idem |
| `tests/persistencia_entre_processos.rs` | mandato 3C.3.b.1 §2 | nuovo, pronto | idem |
| `src/lib.rs` | mandato 3C.3.b.1 §2 | modificato, +2 righe di `pub mod` | idem |
| `Cargo.toml` | mandato 3C.3.b.1 §1 | modificato, +1 riga `serde_json` | idem |
| `Cargo.lock` | conseguenza di `Cargo.toml` | modificato, +1 riga | idem |
| `README.md` | mandato 3C.3.b.1 §2 | modificato: stato di fase, contratti PER, API, conteggio test, limiti 12–17 | idem |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | mandato 3C.3.b.1 §2 | modificato: appendice §18 più tre formulazioni storiche datate | commit documentale, dopo audit e autorizzazione |
| `MANDATO_CLAUDE_FASE3C3B1_IMPLEMENTAZIONE_20261022.md` | Codex | modificato: questa appendice | commit documentale, dopo audit e autorizzazione |

Nessun file resta senza destino. Nessun commit, push, amend, rebase, reset,
restore, checkout, clean o stash è stato eseguito.

---

## 12. Esito dell'audit indipendente e correzioni — 2026-10-22

Audit di Codex: **lotto non chiuso, quattro correzioni richieste**
(R3C3B1-01…04). Tutte eseguite nel perimetro di questo mandato. Nessuna
decisione riaperta, nessuna dipendenza nuova, nessuna riscrittura
dell'architettura, nessuna CLI, export o fase successiva.

### Metodo seguito per ognuno

Riprodotto prima, corretto poi, con regressione permanente scritta **prima**
della correzione e con l'aspettativa giusta. Le tre regressioni di
R3C3B1-01/03 sono fallite su asserzione; quelle di R3C3B1-04 sono fallite con
lo stesso panico descritto dall'audit; per R3C3B1-02 ho registrato anche il
comportamento a runtime del codice prima della correzione, perché l'API da
aggiungere non esisteva e il solo errore di compilazione non è una
riproduzione.

| Riscontro | Prima | Dopo |
|---|---|---|
| **R3C3B1-01** | `verificar_acrescimo` → `Ok(())` con la sola scala cambiata (`10.00` → `10.0000`) in riga, `Dado`, rata e totale; `publicar` accettava | confronto sulla **codifica esatta** di snapshot ed esecuzioni; `HistoriaTruncada` nei quattro punti; candidato davvero identico, anche dopo round-trip, ancora pubblicabile |
| **R3C3B1-02** | `observar`, `candidatos`, `ler` e `recuperar` → `falha de io em ler ...: stream did not contain valid UTF-8` | byte e testo separati; `ErroArquivo::NaoUtf8 { posicao }` come errore di contenuto; `observar` dà `token: None` con l'impronta dei byte; `candidatos` continua a elencare gli altri; la prova conserva i byte corrotti; `EstadoMudou` se i byte corrotti cambiano |
| **R3C3B1-03** | `verificar_estrutura` → `Ok(())` con due esecuzioni dello stesso lotto, anche con piani diversi, nelle due permutazioni e anche rilette da JSON | `ExecucaoConflitante`, lo stesso errore del costruttore; nessuna deduplicazione inventata; due lotti distinti restano ammessi |
| **R3C3B1-04** | panico `end byte index 120 is not a char boundary` nel percorso che doveva **riportare** l'errore | abbreviazione su confine di carattere; 18 casi provati (`é`, `€`, `𝄞` × sei prefissi) più il test diretto della funzione |

### Verifiche dopo le correzioni

| Controllo | Esito |
|---|---|
| `cargo test --locked` | 337 passati: 319 lib, 3 binario, 15 integrazione |
| `cargo fmt --check` | pulito |
| `cargo clippy --locked --all-targets -- -D warnings` | pulito |
| `git diff --check` | pulito |
| 267 prove precedenti al lotto | verdi, senza alcuna modifica |
| sei fonti canoniche | invariate |
| formato dei byte | **invariato**: stesso archivio di prova, 10557 byte e `fb871504…a05`, riconfermato con `Get-FileHash` dopo le correzioni |
| perimetro escluso | intatto: nessuna modifica tracked fuori dai file di questo lotto |

### Differenze dal piano, aggiornate

Alle tre già dichiarate in §11 si aggiungono le quattro correzioni di cui
sopra, più due elementi di superficie pubblica che l'audit ha reso necessari:
`arquivo::desserializar_bytes` e la variante `ErroArquivo::NaoUtf8`. La
funzione interna `sha256_do_texto` è stata sostituita da `sha256_dos_bytes`:
l'impronta deve esistere anche dove un testo non esiste.

### Limiti, invariati

Restano `[NON_VERIFICATO]` la durabilità dopo perdita di alimentazione,
`lucchetto_residuo = true`, il comportamento su volumi reali e la mutua
esclusione contro scrittori non cooperanti. L'autore resta dichiarato, non
autenticato. Registrata anche la premessa dell'audit: il validatore
archivistico non attesta tutti gli invarianti della 3C.2, e la barriera di
`construir_pre_livro` va mantenuta nella composizione futura.

### Stato Git alla riconsegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/arquivo.rs` | mandato §2 + riscontri 01/02/03/04 | nuovo, corretto, pronto | commit mirato gruppo codice, dopo audit mirato e autorizzazione |
| `src/arquivo_io.rs` | mandato §2 + riscontro 02 | nuovo, corretto, pronto | idem |
| `tests/persistencia_entre_processos.rs` | mandato §2 | nuovo, invariato in questo giro | idem |
| `src/lib.rs` | mandato §2 | modificato, +2 righe | idem |
| `Cargo.toml` | mandato §1 | modificato, +1 riga | idem |
| `Cargo.lock` | conseguenza di `Cargo.toml` | modificato, +1 riga | idem |
| `README.md` | mandato §2 | modificato: attestazioni riallineate, contratti `PER-04`/`PER-09`, i quattro riscontri | idem |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | mandato §2 | modificato: §18 più §18.7 con l'esito dei riscontri | commit documentale, dopo audit e autorizzazione |
| `MANDATO_CLAUDE_FASE3C3B1_IMPLEMENTAZIONE_20261022.md` | Codex | modificato: §11 e questa §12 | idem |

Nessun file resta senza destino. Nessun commit, push, amend, rebase, reset,
restore, checkout, clean o stash eseguito.

---

## 13. Audit mirato superato — 2026-10-22

Codex ha chiuso tutti e quattro i riscontri `R3C3B1-01…04`. Dettaglio e
controprove sui byte in §18.8 di `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`.

| Voce | Valore |
|---|---|
| Stato del lotto | **implementato, corretto e con audit mirato superato**. Non chiuso in Git |
| Test | 337 passati, verificati anche dall'esecuzione indipendente dell'audit |
| Formato dei byte | invariato dalle correzioni, provato dall'audit contro una generazione anteriore |
| Decisioni | nessuna riaperta: D7, D8, D9, D14–D17 restano approvate |
| Limiti | invariati: §18.6 della specifica |
| Git | nessun commit, push o staging. I due commit tematici previsti restano **da autorizzare** |

Prima di registrare: ricontrollare stato, perimetro e impronte; prima del
push, verificare la relazione col remoto.

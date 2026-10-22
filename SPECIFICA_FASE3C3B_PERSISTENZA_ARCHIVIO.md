# Specifica fase 3C.3.b — persistenza e archivio autorevole

| Voce | Valore |
|---|---|
| Data | 2026-10-21 |
| Mandato | `MANDATO_CLAUDE_FASE3C3B_PERSISTENZA_SPECIFICA_20261021.md` |
| Perimetro | **analisi e specifica**. Nessuna riga di Rust del progetto, nessuna modifica di `Cargo.*`, nessun archivio reale, nessun export, CLI o produzione |
| Stato | **AUDIT TECNICO DELLA SPECIFICA SUPERATO** il 2026-10-22, con R3C3B-01…04 chiusi. La persistenza **non** è implementata: nessuna riga di writer o reader esiste |
| HEAD alla stesura | `e6dfca2669c56d9070fd5141908da44fe9ac8cc3` |
| Decisione adottata | **D7(a)**: l'archivio strutturato è la fonte autorevole, il workbook è un derivato. Adottata dal mandato, non riaperta qui |
| Decisioni già approvate | D8.1–D8.5, D9.1–D9.5, D10(a), D13(a): invariate |
| Revisione 2 | correzione dei quattro riscontri `R3C3B-01..04` dell'audit del 2026-10-21: §16 |
| Revisione 3 | correzione dei **due residui** dell'audit mirato del 2026-10-22: recupero vincolato allo stato osservato e ordine fisico totale sul **record** archivistico. `R3C3B-01` e `R3C3B-02` sono **chiusi** da quell'audit e non riaperti: §16 |
| Decisioni nuove | **D14, D15, D16 e D17 approvate da Luca il 2026-10-22**, tutte nell'opzione (a): §11 e §17 |
| Identificativi | contratti **`PER-nn`**, prove pianificate **`TP-nn`**: nessuna collisione con F3C-01…46 né con i test T3-* |
| Metodo | `programmatore-bravo`: prima contrattualizza, poi programma |

---

## 0. Come leggere questo documento

| Etichetta | Significato |
|---|---|
| **[ESEGUITO]** | prova eseguita per questo documento, con input, strumento ed esito in §3 |
| **[LETTO_NEL_CODICE]** | fatto letto nei sorgenti alla baseline, con file e riga |
| **[RIUSATO]** | evidenza di una fase o di un audit precedenti, non rieseguita, con la fonte nominata |
| **[PROPOSTA]** | scelta di questo documento, non approvata da nessuno |
| **[NON_VERIFICATO]** | affermazione plausibile che nessuno ha provato; dichiarata tale e non usata come fondamento |

Due regole che valgono su tutto il testo:

1. Nessuna riga del writer o del reader esiste. Tutti i test `TP-nn` sono
   **pianificati**: nessuno è stato eseguito.
2. Le sonde di §3 misurano **il formato e il filesystem**, non il futuro
   lettore Rust. Una sonda che serializza un `Decimal` non dimostra che il
   lettore validerà l'archivio: lo dimostreranno i test pianificati.

---

## 1. Fonte autorevole e flusso

**D7(a), adottata.** L'archivio strutturato è la **fonte autorevole** di
documenti e decisioni. Il workbook è un **derivato**: modificarlo non modifica
l'archivio. Il percorso umano di revisione resta quello di D8/D9. Non esiste
una seconda fonte autorevole parallela.

```text
XML --fase 2--> NfeImportada --3B--> AlocacaoNfe + ContextoDocumento
                                             |
                                             v
                        3C.3.b:  ARQUIVO  (fonte autorevole)
                        snapshot dei contesti ORIGINALI
                        + dichiarazioni degli eventi
                        + metadati delle esecuzioni
                                             |
                    rilettura in un processo nuovo
                                             v
            3C.2 construir_pre_livro  +  3C.3.a aplicar_revisao
                                             |
                                             v
                  vista, riepiloghi, estratto: RICOSTRUITI,
                  mai salvati come se fossero la fonte
                                             |
                                             v
                        3C.4: workbook e export DERIVATI
```

**Che cosa l'archivio conserva e che cosa no**, **[PROPOSTA]**:

| Conservato | Ricostruito, **non** conservato come fonte |
|---|---|
| `ContextoDocumento` **prima** della proiezione umana, integro | la vista revisionata |
| dichiarazioni degli eventi, con autore, istante e motivo | lo `status_revisao` proiettato |
| metadati dell'esecuzione (`IdentidadeLote`) | riepiloghi, indicatori, partizioni |
| impronte di contenuto e di presentazione, come valori **riverificabili** | l'estratto revisionato e le sue esclusioni |
| identità del piano dei conti usato | i nomi di conto mostrati |

Il motivo è D7(a) letta alla lettera: se si salvasse la vista, la vista
diventerebbe una seconda verità, e un bug di aggregazione diventerebbe un dato
storico. Si salva ciò che **non è derivabile**: gli ingressi tipizzati e le
dichiarazioni umane.

---

## 2. Confini: 3C.3.b e 3C.4

C'è una sovrapposizione reale nei documenti, e va dichiarata invece di
risolverla di nascosto.

| Documento | Che cosa dice |
|---|---|
| `README.md`, voce `3C.3.b` | "Persistencia dos eventos e do historico de revisao — depende de D7" |
| §8.4 della specifica 3C generale | assegna a **3C.4** "archivio autorevole, export derivato, round-trip" |
| §5.6 e §5.8 della specifica 3C.3 | dichiarano la persistenza fuori dal lotto puro, "3C.4, dipende da D7" |

**Mappa unica proposta**, **[PROPOSTA]**, senza cambiare nessun comportamento
approvato:

| Blocco | Contenuto | Dipende da |
|---|---|---|
| **3C.3.b** | archivio **minimo** che rende persistente ciò che la 3C.3.a produce e consuma: snapshot dei contesti originali, eventi, metadati delle esecuzioni, identità del piano. Codec, validazione, protocollo di salvataggio e recupero, rilettura in un processo nuovo | D7(a) adottata |
| **3C.4** | adattatori di ingressi grezzi (`INCOMPLETO`, `NAO_RECONHECIDO`), import dello storico Excel, **export** derivato e round-trip del workbook | D11, D12, dipendenze XLSX |
| **3C.5** | uso da CLI, validazione C15 condivisa | 3C.3.b, 3C.4 |

La porzione di archivio **documentale** che serve agli eventi non è un secondo
archivio: è **la stessa** radice, con gli snapshot dei contesti. Salvare solo
gli eventi e pretendere che il chiamante riporti i documenti originali sarebbe
una falsa persistenza, e §4.2 lo dimostra con un confronto.

---

## 3. Prove eseguite, riusate, non eseguite

### 3.1 Strumenti

- **Sonda `sonda_persistencia`**: crate temporaneo **fuori dal repository**, in
  `C:\Users\RECANT~1\AppData\Local\Temp\claude\c--Users-Recanto-do-Sossego-Documents-Luca-GitHub-RUST-Rust-registrazioni-contabili\1898eaaf-4410-4449-8413-33f6928a9c29\scratchpad\sonda_persistencia`.
  Compilata con `cargo build --offline`, con le **stesse versioni** fissate in
  `Cargo.lock` del progetto: `chrono 0.4.45` (`std`, `clock`, `serde`),
  `rust_decimal 1.43.0` (`serde`), `serde 1.0.229`, `serde_json 1.0.154`.
  Nessuna installazione: le crate erano già nella cache locale.
- **Nessuna scrittura** nel repository, in archivi reali o in workbook. Le
  prove di filesystem usano `%TEMP%\sonda_persistencia_3c3b`.

### 3.2 `serde_json` è disponibile ma **non** autorizzato

**[ESEGUITO]**. Tre fatti distinti, da non confondere:

| Fatto | Esito |
|---|---|
| `serde_json` **non** è in `[dependencies]` del progetto | **[LETTO_NEL_CODICE]** `Cargo.toml` |
| `serde_json 1.0.154` **è** in `Cargo.lock`, come dipendenza **opzionale** di `rust_decimal` | `Cargo.lock`, pacchetto `rust_decimal` → `serde_json`; voce a sé con checksum |
| la crate e le sue dipendenze (`itoa`, `memchr`, `serde`, `serde_core`, `zmij`) sono nella **cache locale**, e il sorgente è estratto | `~/.cargo/registry/cache/.../serde_json-1.0.154.crate` e `registry/src/.../serde_json-1.0.154` |
| la rlib **non** è costruita nel target del progetto | nessun `libserde_json-*.rlib` fra le dipendenze compilate |

Conclusione, **[PROPOSTA]**: l'aggiunta di `serde_json = "1.0.154"` a
`Cargo.toml` è **realizzabile offline** e alla versione già risolta, ma **non è
autorizzata** da questo documento. Va chiesta nel mandato implementativo,
insieme al perimetro che la usa. Essere in cache non è un'autorizzazione.

### 3.3 S1 — i `Decimal` si serializzano come **testo esatto**

**[ESEGUITO]**. Con la feature `serde` del progetto, `rust_decimal` serializza
in JSON come **stringa**, e la rilettura conserva valore **e scala**:

| Valore | JSON prodotto | Riletto | Scala |
|---|---|---|---|
| `0` | `{"valor":"0"}` | `0` | 0 → 0 |
| `0.00` | `{"valor":"0.00"}` | `0.00` | 2 → 2 |
| `0.01` | `{"valor":"0.01"}` | `0.01` | 2 → 2 |
| `-7.50` | `{"valor":"-7.50"}` | `-7.50` | 2 → 2 |
| `10.0000` | `{"valor":"10.0000"}` | `10.0000` | 4 → 4 |
| `1.2345` | `{"valor":"1.2345"}` | `1.2345` | 4 → 4 |
| `12345678901234.56` | `{"valor":"12345678901234.56"}` | identico | 2 → 2 |
| `Decimal::MAX` | `{"valor":"79228162514264337593543950335"}` | identico | 0 → 0 |
| `Decimal::MIN` | `{"valor":"-79228162514264337593543950335"}` | identico | 0 → 0 |
| `0.0000000000000000000000000001` | stringa con 28 decimali | identico | 28 → 28 |

Conseguenze dirette:

1. **nessun `f32`/`f64`** nel percorso autorevole: non serve un codec
   monetario fatto a mano, serve **non** attivare la feature `serde-float` e
   **non** scrivere i valori come numeri JSON;
2. la **scala memorizzata sopravvive**, quindi `10.00` e `10.0000` restano
   distinti nel record anche se l'impronta li unifica — ed è corretto: il
   record conserva, l'impronta normalizza;
3. **`-0.00` viene scritto `"0.00"`**: il segno dello zero negativo si perde.
   Coerente con la regola `DEC:0:0` dell'impronta, e dichiarato;
4. **rischio provato**: un **numero** JSON come `{"valor":12345678901234.56}`
   viene **accettato** in lettura e produce il valore giusto. Un archivio
   modificato a mano, o prodotto da uno strumento che passa per `double`,
   verrebbe letto senza proteste pur avendo potenzialmente già perso cifre.
   Da qui il contratto **PER-07**: il lettore **rifiuta** un monetario che non
   sia una stringa.

### 3.4 S2 — il `Serialize` normale di `chrono` **perde** i secondi dell'offset

**[ESEGUITO]**, ed è il fatto più importante di questa specifica.

```text
a = 2026-10-21T12:00:00+00:00:01   offset 1s
b = 2026-10-21T12:00:00+00:00:02   offset 2s
a != b come valore?                true

serde di a: {"instante":"2026-10-21T12:00:00+00:00"}
serde di b: {"instante":"2026-10-21T12:00:00+00:00"}
i due JSON sono uguali?            true
round-trip conserva a? false       conserva b? false
```

Il `Serialize` derivato di `chrono` per `DateTime<FixedOffset>` scrive RFC3339
con l'offset **arrotondato ai minuti**. Due dichiarazioni distinte diventano lo
stesso JSON, e la rilettura **non** restituisce il valore originale. È
esattamente il difetto chiuso in R3C3A-02: usare `#[derive(Serialize)]`
sull'evento lo **reintrodurrebbe** nello strato di persistenza.

Con i nanosecondi e un offset a minuti interi il round-trip funziona
(`2026-10-21T12:00:00.000000001-03:00` torna identico): il difetto riguarda
**solo** i secondi dell'offset, che è proprio ciò che `FixedOffset` ammette.

La forma proposta li distingue (**[ESEGUITO]**):

```text
forma esatta di a: 2026-10-21T12:00:00.000000000OFF1
forma esatta di b: 2026-10-21T12:00:00.000000000OFF2
differiscono?      true
```

Da qui **PER-09**: l'istante si salva in due campi dichiarati, orario locale e
offset **in secondi**, e **non** come stringa RFC3339.

### 3.5 S3 — assenze, chiavi duplicate, testi ostili

**[ESEGUITO]**, con `serde_json 1.0.154`:

| Caso | Esito osservato | Conseguenza |
|---|---|---|
| `Option::None` con `skip_serializing_if` | campo **omesso** | il writer omette, non scrive `null` |
| campo **omesso** in lettura | `None` | — |
| campo **`null`** in lettura | `None` | omesso e `null` sono **indistinguibili** per il lettore: limite dichiarato, PER-10 |
| stringa vuota | `Some("")`, distinta da `None` | assenza, zero e stringa vuota restano tre cose |
| chiave duplicata su una **struct** | **rifiutata**: `duplicate field 'a'` | lo schema usa struct, non mappe, dove la duplicazione conta |
| chiave duplicata su una **mappa** | **accettata**, vince l'ultima | PER-12 vieta le mappe nei punti sensibili |
| campo **sconosciuto** senza `deny_unknown_fields` | **accettato in silenzio** | PER-11 impone `deny_unknown_fields` su ogni struct dell'archivio |
| JSON **troncato** | rifiutato: `EOF while parsing a value` | — |
| testo `=1+1 SUCO "MARCA"` e `00012345678901` | round-trip **identico** | i difetti del workbook non sono difetti del JSON |
| `José` in NFC e in NFD | due JSON **distinti**, entrambi conservati byte per byte | l'archivio conserva gli originali; la NFC resta solo nelle impronte |

Il progetto già usa `deny_unknown_fields` in nove punti, fra cui `DadoBruto`
(**[LETTO_NEL_CODICE]** `modelos.rs:235`) e tutte le struct di `regras.rs`
(contratto C3): PER-11 continua quella disciplina, non la inventa.

### 3.6 S4 — che cosa consente davvero il filesystem di Windows

**[ESEGUITO]** su `%TEMP%`, con `std::fs` e handle aperti da Rust:

| Caso | Esito osservato |
|---|---|
| `rename` su destinazione **inesistente** | **OK** |
| `rename` su destinazione **esistente e chiusa** | **OK**, contenuto sostituito |
| `rename` su destinazione **aperta in lettura** da un altro handle | **OK** |
| `rename` su destinazione **aperta in scrittura** da un altro handle | **OK** |
| `File::create` su un file aperto in scrittura | **OK, e troncato**: il writer ingenuo distrugge il contenuto |
| `OpenOptions::create_new` ripetuto | primo OK, secondo `AlreadyExists` |
| lettura mentre un altro handle scrive | **OK**: nessun lock obbligatorio, un lettore può vedere un file a metà |
| `remove_file` su un file aperto in lettura | **OK**, e il file **spariva** |
| `sync_all` su una **directory** | **ERRORE `PermissionDenied` (os error 5)** |

Cinque conseguenze, tutte usate nel protocollo di §9:

1. `rename` **è** una sostituzione utilizzabile su Windows: non serve
   cancellare prima. Il mandato chiedeva di dimostrarlo, ed è dimostrato per
   gli handle aperti da Rust.
2. la riuscita è stata provata con handle aperti da Rust, che condivide
   lettura, scrittura e cancellazione. Con un handle che **non** condivide la
   cancellazione la sostituzione viene **rifiutata**: **[RIUSATO]** dall'audit
   del 2026-10-21, che l'ha provato con un handle .NET `FileShare.None`,
   osservando il precedente conservato. Il protocollo tratta quindi il
   fallimento del `rename` come esito **normale e nominato**, non come caso
   impossibile — e il test TP-14 ha una tecnica per provocarlo senza
   installare niente.
3. **"cancella poi rinomina" è vietato**: `remove_file` riesce anche su file
   aperti, e lascia una finestra in cui l'archivio **non esiste**. Non è una
   strategia di sostituzione sicura.
4. `create_new` fornisce un lucchetto **consultivo** utilizzabile, ma un
   lucchetto rimasto da un processo morto resta lì: serve una regola di
   recupero esplicita, mai una cancellazione automatica (§9.4).
5. **non si può sincronizzare la directory**: `sync_all` su una directory è
   negato. Si può sincronizzare il **file** temporaneo prima della
   pubblicazione, ma la durabilità della voce di directory creata dal `rename`
   resta **[NON_VERIFICATO]**. Promettere "durabilità garantita" sarebbe falso:
   §9.3 distingue atomicità, recuperabilità e durabilità invece di confonderle.

### 3.7 S5 — gli esempi di questo documento passano un parser indipendente

**[ESEGUITO]**. I due esempi JSON di §5.2 e §5.3 sono stati estratti dal
Markdown e letti con `ConvertFrom-Json` di PowerShell, cioè uno strumento
**diverso** da `serde_json`:

| Controllo | Esito |
|---|---|
| sintassi dei due esempi | **validi**; chiavi di radice `formato, versao_schema, revisao_arquivo, gramaticas, plano, execucoes, snapshots, eventos` |
| `valor_total_nfe`, `soma_lancamentos`, `componenti[0].valor.valor` | tipo **String** nel parser indipendente, valore `35.00` |
| `articoli_attesi.valor` | `null` |
| `lancamentos` | 0 elementi: l'esempio è davvero il caso senza righe |
| `snapshot.lote` contro `contexto.lote.id` | **uguali**, come impone PER-02 |
| `instante` dell'evento | `local` e `offset_segundos` (intero `-10800`); nessun campo RFC3339 |

**Una trappola trovata proprio qui, e dichiarata.** `ConvertFrom-Json`
**converte** la stringa `"2026-10-21T12:00:00.000000000"` in un
`System.DateTime`, che ha risoluzione di 100 ns: riformattato dà
`2026-10-21T12:00:00.0000000`, con **sette** decimali invece di nove. Il testo
grezzo del file contiene i nove decimali — verificato con una ricerca sul
testo, non sull'oggetto.

Conseguenza per TP-20: il controllo indipendente deve confrontare il **testo
grezzo** dove la precisione conta, perché lo strumento di controllo stesso può
perdere cifre. Uno strumento indipendente non è automaticamente più fedele:
è solo indipendente.

### 3.8 S6 — riproduzione dei riscontri dell'audit, contro i tipi reali

**[ESEGUITO]**. Secondo crate temporaneo, `sonda_codec`, con **dipendenza di
percorso** sul progetto e `serde_json`, compilato `--offline`: legge il
`contexto` dell'esempio di §5.2 con il `Deserialize` **reale** di
`ContextoDocumento` e misura che cosa accetta.

| Ingresso | Esito osservato |
|---|---|
| `"campo_novo": 42` annidato nella riconciliazione | **accettato e scartato**; il modello risulta identico alla base |
| `"valor_total_nfe": 35` (numero JSON) | **accettato**, scala da 2 a 0 |
| `"valor_total_nfe": "0.00000000000000000000000000001"` | **accettato come `0.0000000000000000000000000000`**: un valore diventa **zero** |
| `"valor_total_nfe": "35.000000000000000000000000000001"` | **accettato**, cifre eccedenti eliminate |
| `"stato_documento"` ripetuto nello stesso oggetto | **rifiutato**: `duplicate field` |
| `Dado` con `status: "OK"` e valore assente | **rifiutato** da `Dado::novo` |

E il confronto fra i due parser di `rust_decimal`:

| Testo | `from_str` | `from_str_exact` | `to_string()` uguale al testo? |
|---|---|---|---|
| `35.00`, `0`, `-7.50`, `10.0000`, `0.01`, `MAX` | esatti | esatti | **sì** |
| `0.00000000000000000000000000001` | `0.0000…0` | **errore** | no |
| `35.000000000000000000000000000001` | troncato | **errore** | no |
| `3.5e1` | `35` | **errore** | no |
| `+35.00` | `35.00` | `35.00` | **no** |
| `35.00␠` (spazio finale) | errore | errore | no |

Serve **la coppia** di controlli: `from_str_exact` prende la perdita di cifre e
la notazione scientifica, il confronto del testo riprodotto prende `+35.00`, che
`from_str_exact` accetta. Da qui `PER-20`.

E la riproduzione del pareggio nell'ordine fisico (`R3C3B-04`):

```text
chiave fisica di A e di B con digest assente: identiche
ingresso AB -> prima "nota A"      ingresso BA -> prima "nota B"
con il testo esatto come ultimo livello: chiavi distinte
```

### 3.9 S7 — riproduzione dei due residui del 2026-10-22

**[ESEGUITO]**, `sonda_residui_3c3b.ps1` con il suo log, nella cartella di
§3.1. Sono **modelli dei criteri dichiarati** da questa specifica su file
sintetici: non eseguono il writer, che non esiste.

**Residuo di R3C3B-03**, i cinque passi del recupero della revisione 2 alla
lettera:

```text
1. il chiamante osserva un corrente corrotto e scegli il candidato rev 1 [A]
   impronta dei byte osservati = 795e1e8b...
2. un'altra operazione cooperante pubblica: corrente valido rev 3, [A,B];
   lucchetto libero
3-5. recupero come specificato -> corrente = {"revisao_arquivo":1,"eventos":["A"]}
     prova conservata        = {"revisao_arquivo":3,"eventos":["A","B"]}
     l'evento B e' ancora nel corrente? false
```

Con il confronto proposto: atteso `795e1e8b…`, trovato `b27b2408…`, diversi ⇒
`EstadoMudou`, nessuna sostituzione.

**Residuo di R3C3B-04**, due record della stessa dichiarazione con
`impressao_evento` assente e con il digest corretto:

```text
chiavi della revisione 2 identiche?        true
record serializzati uguali?                false
ingresso [assente, presente] -> file 4ab5d3da...
ingresso [presente, assente] -> file 159dc1a7...

con lo spareggio sul record serializzato:
ingresso [assente, presente] -> file 159dc1a7...   (2 record conservati)
ingresso [presente, assente] -> file 159dc1a7...   (2 record conservati)
```

### 3.10 Evidenze riusate e prove non eseguite

| Evidenza | Fonte | Perché non rieseguita |
|---|---|---|
| 267 test verdi, `fmt`, `clippy -D warnings`, `diff --check` puliti | audit mirato finale 3C.3.a, commit `fe5cafa` e `e6dfca2` | nessun file di codice toccato da questo documento: è solo Markdown |
| R3C3A-01/02/03 chiusi | stesso audit | non riaperti |
| vettori `IMPRESSAO-CONTEUDO/v1`, `EVENTO-EXATO/v1`, D8/D9 | fase 3C.3.a | contratti chiusi e protetti da test |

| Non eseguito | Motivo |
|---|---|
| qualunque riga del writer o del reader | fuori perimetro: la specifica precede l'implementazione |
| prova su un archivio reale | nessuno esiste, e il mandato lo vieta |
| `rename` con destinazione tenuta da un programma esterno | richiederebbe quel programma: §3.6 punto 2, dichiarato `NON_VERIFICATO` |
| comportamento su rete, volumi diversi, FAT32, OneDrive | non provato; §9.6 lo dichiara come vincolo d'uso |
| perdita di alimentazione reale | non simulabile qui; §9.3 dichiara cosa resta incerto |
| suite Rust, `fmt`, `clippy` | nessun codice toccato |

---
## 4. Due soluzioni, e perché il registro dei soli eventi non basta

### 4.1 Confronto

| | **A — registro dei soli eventi** | **B — archivio con snapshot dei contesti** |
|---|---|---|
| contenuto | solo le dichiarazioni di revisione | contesti originali + dichiarazioni + metadati delle esecuzioni + identità del piano |
| chi fornisce i documenti alla rilettura | il chiamante, da fuori | l'archivio |
| riproducibilità | **condizionata**: solo se il chiamante ritrova gli stessi XML, le stesse regole e la stessa versione del codice di allocazione | **dichiarata**: i contesti sono nel file |
| se un XML viene spostato o cancellato | gli eventi restano, ma non si applicano a niente: `AlvoAusente` su tutto | la storia si rilegge comunque |
| se le regole YAML cambiano | la riallocazione dà un contenuto diverso, quindi un'impronta diversa, quindi `VersaoSuperada` su tutti gli eventi | gli eventi restano applicati al contenuto che fu esaminato |
| coerenza con D7(a) | **no**: la fonte autorevole resterebbe l'insieme XML + regole + codice, cioè tre cose fuori dall'archivio | **sì** |
| dimensione del file | minima | maggiore: un `ContextoDocumento` completo per documento |

### 4.2 Perché A non è riproducibile, in concreto

L'impronta di contenuto dipende da `conta_id`, `conta_nome` e `grupo_conta`
della **riga**, scritti dalla 3B al momento dell'allocazione
(**[RIUSATO]**, §4.7 della specifica 3C.3). Quindi con la soluzione A basta un
alias nuovo in `regras_classificacao_produtos.yaml` perché la riallocazione
dello stesso XML produca un'impronta diversa e **tutte** le approvazioni
risultino `VersaoSuperada`: corrette come esito, inutili come storia.

Peggio: se l'XML non è più reperibile, A non consente nemmeno di sapere **che
cosa** era stato approvato. Gli eventi citano un'impronta che nessuno può più
ricalcolare.

**Raccomandazione: B**, il minimo coerente con D7(a). Il costo — un contesto
completo per documento — è il prezzo di una fonte autorevole che non dipende da
tre cose esterne. A resta descritta perché il confronto va conservato, non
perché sia ammissibile.

---

## 5. Schema e esempio minimo

### 5.1 Radice

**[PROPOSTA]**. Un documento JSON, UTF-8 **senza BOM**, con terminatori `LF`.

```text
radice := {
  "formato":          "FANTOZZI-ARQUIVO/v1"      // identificatore del formato
  "versao_schema":    1                          // intero, solo questo schema
  "revisao_arquivo":  <u64>                      // pubblicazioni di QUESTO file
  "gramaticas":       { ... }                    // versioni delle grammatiche usate
  "execucoes":        [ execucao, ... ]           // ognuna con il SUO piano
  "snapshots":        [ snapshot, ... ]
  "eventos":          [ evento, ... ]
}
```

**Cinque identificativi di versione, cinque cose diverse** (requisito del
mandato, punto 2):

| Campo | Che cosa conta | Chi lo cambia |
|---|---|---|
| `versao_schema` | la forma di **questo file** | un cambio di schema dell'archivio |
| `gramaticas.*` | le grammatiche di impronta e di evento della 3C.3.a | un `v2` di quelle grammatiche |
| `snapshot.impressao_conteudo` | **quale contenuto** di un documento | un contenuto allocato diverso |
| `execucao.lote.id` | quale esecuzione lo ha prodotto | ogni nuova esecuzione |
| `revisao_arquivo` | quante volte **il file** è stato pubblicato | ogni pubblicazione riuscita |

`revisao_arquivo` **non** è la sequenza storica di un documento: serve al
controllo di concorrenza di §9.5 e a niente altro. La sequenza storica dei
documenti resta `NAO_DISPONIVEL` (D8.5, §11 D17).

### 5.2 Esempio minimo completo

Il caso più piccolo che sia **completo**: un documento `ERRO` **senza
righe**, con la sua riconciliazione integra, e nessun evento. Serve anche come
caso `P-02`.

Valori: la chiave e la data sono quelle del **campione canonico** versionato in
`samples/nfe_exemplo/nfe_oesa_12566_20260505.xml`; il CNPJ dell'azienda, il
lotto, i percorsi e le impronte sono **sintetici**, segnati `<sintetico>`.

```json
{
  "formato": "FANTOZZI-ARQUIVO/v1",
  "versao_schema": 1,
  "revisao_arquivo": 1,
  "gramaticas": {
    "impressao_conteudo": "IMPRESSAO-CONTEUDO/v1",
    "impressao_apresentacao": "IMPRESSAO-APRESENTACAO/v1",
    "evento_exato": "EVENTO-EXATO/v1"
  },
  "execucoes": [
    {
      "lote": {
        "id": "lote-2026-05-A",
        "gerado_em": "2026-05-31",
        "descricao": "fechamento de maio"
      },
      "plano": {
        "versao": "1.0",
        "sha256_arquivo": "c25715f97871cf17868cfe5cc2a31211b80828d6d032dac2a90616baede4e8f5",
        "caminho_declarado": "config/plano_contas_gerencial.yaml"
      },
      "documentos": ["29260512345678000195550010001256612345678901"]
    }
  ],
  "snapshots": [
    {
      "lote": "lote-2026-05-A",
      "chave_origem": "29260512345678000195550010001256612345678901",
      "impressao_conteudo": "<sintetico: 64 hex>",
      "impressao_apresentacao": "<sintetico: 64 hex>",
      "contexto": {
        "chave_origem": "29260512345678000195550010001256612345678901",
        "numero": "12566",
        "serie": "1",
        "data_documento": "2026-05-05",
        "competencia": "2026-05",
        "cnpj_empresa": "55746722000181",
        "nome_empresa": "RESTAURANTE OESA LTDA",
        "arquivo_origem": "samples/nfe_exemplo/nfe_oesa_12566_20260505.xml",
        "alocacao": {
          "lancamentos": [],
          "reconciliacao": {
            "valor_total_nfe": "35.00",
            "soma_lancamentos": "0.00",
            "esito_total": "ERRO",
            "soma_rate_documento": {
              "valor": null,
              "fonte": "nfe:cobr/dup/vDup",
              "status": "N.A.",
              "motivo": "NF-e sem duplicatas"
            },
            "soma_rate_allocate": {
              "valor": null,
              "fonte": "nfe:cobr/dup/vDup",
              "status": "N.A.",
              "motivo": "NF-e sem duplicatas"
            },
            "esito_rate": "N.A.",
            "articoli_allocati": 0,
            "articoli_attesi": {
              "valor": null,
              "fonte": "nfe:det",
              "status": "NAO_DISPONIVEL",
              "motivo": "a fase 2 nao expoe o numero de <det> lidos"
            },
            "articoli_scartati": [
              { "codigo": "SEM_PRODUTOS", "detalhe": "NF-e sem nenhum item <det>/<prod>" }
            ],
            "esito_copertura": "PARCIAL",
            "componenti": [
              {
                "nome": "vProd",
                "valor": {
                  "valor": "35.00",
                  "fonte": "nfe:total/ICMSTot/vProd",
                  "status": "OK"
                },
                "ruolo": "BASE",
                "usada_na_verificacao": true
              }
            ],
            "esito_componenti": "OK",
            "campi_rinviati": [
              {
                "campo": "centro_custo",
                "status": "NAO_DISPONIVEL",
                "fonte": "fase 3B",
                "motivo": "campo nao populado neste bloco"
              }
            ],
            "avisos_documento": [
              { "codigo": "DOCUMENTO_SEM_ITENS", "detalhe": "NF-e sem itens alocaveis" }
            ],
            "stato_documento": "ERRO"
          }
        },
        "avisos_leitura": [
          { "codigo": "SEM_PRODUTOS", "detalhe": "NF-e sem nenhum item <det>/<prod>" }
        ],
        "lote": {
          "id": "lote-2026-05-A",
          "gerado_em": "2026-05-31",
          "descricao": "fechamento de maio"
        }
      }
    }
  ],
  "eventos": []
}
```

Tre cose da notare, tutte **[LETTO_NEL_CODICE]** o **[ESEGUITO]**:

1. l'oggetto `contexto` ha **la stessa forma** della serializzazione di
   `ContextoDocumento`, che deriva `Serialize` e `Deserialize`
   (`prelivro.rs:273`). **Non** viene però letto con quel `Deserialize`: §7.3
   spiega perché non basta, e impone un DTO dedicato anche per i contesti;
2. i monetari sono **stringhe**: `"35.00"`, `"0.00"`. È il comportamento
   provato in §3.3, non una scelta di formattazione;
3. per un `Dado` assente il campo `valor` è **`null` esplicito**, mentre
   `motivo` è omesso quando assente: dipende da `skip_serializing_if` sul solo
   `motivo` (`modelos.rs:127-134`). Il lettore valida comunque tutto
   attraverso `Dado::novo` (`modelos.rs:245-257`).

### 5.3 Un evento, per intero

```json
{
  "alvo": {
    "chave_origem": "29260512345678000195550010001256612345678901",
    "impressao_conteudo": {
      "valor": "<sintetico: 64 hex>",
      "fonte": "revisao:conteudo",
      "status": "OK"
    },
    "origem": "XML_NFE",
    "conta_id": "2.01.004",
    "ordinal": 0
  },
  "operacao": "APROVAR",
  "autor": "luca",
  "instante": {
    "local": "2026-10-21T12:00:00.000000000",
    "offset_segundos": -10800
  },
  "motivo": "visto e aceito",
  "nota": null,
  "sequencia": null,
  "vista": {
    "valor": "<sintetico: 64 hex>",
    "fonte": "revisao:vista",
    "status": "OK"
  },
  "impressao_evento": "<sha256 del texto_exato_evento>"
}
```

`instante` è **due campi**, non una stringa RFC3339: §3.4 dimostra che la
stringa perderebbe i secondi dell'offset. `impressao_evento` è il SHA-256 di
`texto_exato_evento` calcolato alla scrittura: serve a **rilevare
un'alterazione**, non ad autenticare chi ha scritto (PER-13).

---

## 6. Carta dei campi

Ogni dato che l'archivio conserva, con fonte, rappresentazione, assenza
ammessa, validatore. Le righe segnate **escluso** dicono perché.

### 6.1 Radice e metadati

| Campo | Fonte | Rappresentazione | Assenza | Validazione |
|---|---|---|---|---|
| `formato` | costante del writer | stringa | **vietata** | uguale a `FANTOZZI-ARQUIVO/v1`, altrimenti `FormatoDesconhecido` |
| `versao_schema` | costante | intero | vietata | `== 1`; un numero maggiore è `SchemaMaisNovo`, non un tentativo di lettura |
| `revisao_arquivo` | il file stesso | `u64` | vietata | `>= 1`; §9.5 |
| `gramaticas.impressao_conteudo` | `impressao::VERSAO_CONTEUDO` | stringa | vietata | uguale alla costante del crate, altrimenti `GramaticaDesconhecida` |
| `gramaticas.impressao_apresentacao` | `impressao::VERSAO_APRESENTACAO` | stringa | vietata | idem |
| `gramaticas.evento_exato` | `revisao::VERSAO_EVENTO_EXATO` | stringa | vietata | idem |
L'identità del piano **non** sta nella radice: sta in ogni esecuzione (§6.2).
Nella radice avrebbe impedito di archiviare due esecuzioni fatte con piani
diversi senza sovrascrivere un metadato protetto — che `PER-17` vieta.

### 6.2 Esecuzione

| Campo | Fonte | Rappresentazione | Assenza | Validazione |
|---|---|---|---|---|
| `lote.id`, `lote.gerado_em`, `lote.descricao` | `IdentidadeLote` | stringa, `YYYY-MM-DD`, stringa | vietata | `id` non vuoto; `gerado_em` data valida; `descricao` libera |
| `plano.versao` | `PlanoContas`, campo `versao` dello YAML | stringa | vietata | non vuota |
| `plano.sha256_arquivo` | SHA-256 del file YAML, calcolato dal **chiamante** | 64 hex minuscoli | **ammessa** come `null` se il chiamante non la dichiara | se presente, 64 hex; il confronto con il piano corrente è del chiamante |
| `plano.caminho_declarado` | chiamante | stringa o `null` | ammessa | **informativo**: dato d'ambiente, mai usato per caricare niente |
| `documentos` | chiavi dei documenti dell'esecuzione | array di stringhe | array vuoto ammesso | ogni chiave deve avere **esattamente uno** snapshot con quel `lote`; altrimenti `ReferenciaRompida` |

### 6.3 Snapshot

| Campo | Fonte | Rappresentazione | Assenza | Validazione |
|---|---|---|---|---|
| `lote` | `ContextoDocumento::lote.id` | stringa | vietata | **uguale** a `contexto.lote.id`, altrimenti `SnapshotIncoerente` |
| `chave_origem` | `ContextoDocumento` | stringa | vietata | non vuota (C22) e **uguale** a `contexto.chave_origem` |
| `impressao_conteudo` | `impressao::impressao_conteudo` alla scrittura | 64 hex | vietata | **ricalcolata** in lettura e confrontata: PER-13 |
| `impressao_apresentacao` | `impressao::impressao_apresentacao` | 64 hex | ammessa `null` se il piano non era dichiarato | se presente e il piano corrente differisce, D8.3: `APRESENTACAO_MUDADA`, non un errore |
| `contexto` | `ContextoDocumento` integro | oggetto con la **forma** del modello in memoria; il **lettore** usa il DTO restrittivo di §7.3, non il `Deserialize` del tipo | vietata | DTO con `deny_unknown_fields`, decimali esatti, poi i costruttori validati; la coerenza contesto/righe resta imposta da `construir_pre_livro` |

### 6.4 Evento

| Campo | Fonte | Rappresentazione | Assenza | Validazione |
|---|---|---|---|---|
| `alvo.chave_origem` | `ChaveRevisao` | stringa | vietata | non vuota |
| `alvo.impressao_conteudo` | `Dado<String>` | oggetto `Dado` | **ammessa**: `valor: null` con stato e motivo | `Dado::novo`; assente ⇒ `ProvenienciaAusente` alla riapplicazione, **non** un errore di lettura |
| `alvo.origem` | `OrigemLancamento` | `XML_NFE` / `CSV` / `MANUAL` | vietata | enum chiuso |
| `alvo.conta_id` | `ChaveRevisao` | stringa | vietata | non vuota |
| `alvo.ordinal` | `ChaveRevisao` | `u32` | vietata | intero, **non** stringa |
| `operacao` | `OperacaoRevisao` | `APROVAR` / `REVOGAR` / `ANOTAR` | vietata | enum chiuso; un valore nuovo è `EnumDesconhecido` |
| `autor` | dichiarato da chi opera | stringa | vietata | non vuota dopo `trim`: `validar` della 3C.3.a |
| `instante.local` | `DateTime::naive_local` | `YYYY-MM-DDTHH:MM:SS.fffffffff` | vietata | nove decimali obbligatori, per non dipendere dalla scrittura |
| `instante.offset_segundos` | `FixedOffset::local_minus_utc` | intero con segno, in **secondi** | vietata | intervallo ammesso da `FixedOffset`; §3.4 |
| `motivo` | dichiarato | stringa | vietata | non vuoto dopo `trim` |
| `nota` | dichiarata | stringa o `null` | ammessa | obbligatoria per `ANOTAR` |
| `sequencia` | dichiarata | `u64` o `null` | ammessa | un'assenza **non** diventa `0` |
| `vista` | `Dado<String>` | oggetto `Dado` | ammessa | `Dado::novo` |
| `impressao_evento` | SHA-256 di `texto_exato_evento` | 64 hex | ammessa `null` | se presente, **ricalcolata** e confrontata |

### 6.5 Che cosa l'archivio **non** salva, e perché

| Non salvato | Motivo |
|---|---|
| vista revisionata, riepiloghi, indicatori, partizioni, estratto | derivabili: salvarli creerebbe una seconda verità (§1) |
| `status_revisao` proiettato | è un risultato della riapplicazione, non una dichiarazione |
| `IdentidadeLinha` delle righe | derivabile dal contesto più il lotto |
| percorso assoluto del file XML | `arquivo_origem` resta **dentro** il contesto perché è un dato del documento letto, e l'impronta lo esclude. L'archivio non aggiunge altri percorsi |
| utente del sistema operativo, orologio della macchina | non sono dichiarazioni umane: §9.7 |
| workbook o export | derivati, blocco 3C.4 |

---

## 7. Codec e validatori

### 7.1 Regole di rappresentazione

| Tipo | Rappresentazione | Prova |
|---|---|---|
| denaro e ogni `Decimal` | **stringa** decimale esatta, scala conservata | **[ESEGUITO]** §3.3 |
| `NaiveDate` | `YYYY-MM-DD` | serde di `chrono`, già usato nei contesti |
| `Competencia` | `YYYY-MM` | `serde(try_from = "String", into = "String")`, `modelos.rs:354-356` |
| `DateTime<FixedOffset>` | **due campi**: `local` e `offset_segundos` | **[ESEGUITO]** §3.4: la stringa RFC3339 perde i secondi dell'offset |
| enum | etichette `SCREAMING_SNAKE_CASE` già in uso, con `N.A.` per `NaoAplicavel` | `modelos.rs`, `alocacao.rs` |
| `Dado<T>` | oggetto con `valor`, `fonte`, `status`, `motivo` omesso se assente | `modelos.rs:127-134` |
| testo | byte **originali**, nessuna normalizzazione | **[ESEGUITO]** §3.5: NFC e NFD restano distinti |
| identificativi con zeri iniziali | stringa | **[ESEGUITO]** §3.5 |

**La NFC non entra nell'archivio.** Serve alle impronte e alle chiavi canoniche,
dove la 3C.3.a la applica già; riscrivere i record salvati in NFC
cambierebbe i dati, e D8.4(a) dice il contrario: normalizza per calcolare, non
per memorizzare.

### 7.2 Contratti di rappresentazione e di lettura

| ID | Contratto | Ingresso / assenza / conflitto | Esito | Prova |
|---|---|---|---|---|
| **PER-01** | formato, `versao_schema` e le tre grammatiche sono dichiarati nella radice e verificati in lettura | schema o grammatica sconosciuti | `FormatoDesconhecido`, `SchemaMaisNovo`, `GramaticaDesconhecida`: nessuna lettura parziale | TP-11 |
| **PER-02** | l'archivio conserva i `ContextoDocumento` **prima** della proiezione umana, integri: riconciliazione, quattro esiti, avvisi, `Dado`, rate, documenti senza righe, campi rinviati | documento senza righe | snapshot completo, riconciliazione identica alla rilettura | TP-01, TP-02 |
| **PER-03** | l'archivio conserva le **dichiarazioni** degli eventi, non lo stato proiettato né la vista | riga approvata | si rilegge l'evento, e la vista si ricostruisce | TP-01, TP-08 |
| **PER-04** | ogni esecuzione è conservata con la propria `IdentidadeLote`; snapshot distinti per esecuzioni distinte, **anche a parità di impronta** | stesso contenuto in due lotti | due snapshot, nessuna deduplicazione | TP-09 |
| **PER-05** | l'identità del piano dei conti è registrata; il piano **non** viene caricato dall'archivio, lo fornisce il chiamante | piano diverso alla rilettura | `APRESENTACAO_MUDADA` secondo D8.3, decisioni conservate | TP-18 |
| **PER-06** | nessun `f32`/`f64` nel percorso autorevole | — | monetari e decimali come stringhe | TP-04 |
| **PER-07** | un valore monetario che **non** sia una stringa è **rifiutato** | `{"valor": 35}` | `MonetarioNaoTextual`: un numero JSON sarebbe accettato per difetto, e con la scala schiacciata da 2 a 0 (**[ESEGUITO]** §3.8) | TP-04 |
| **PER-08** | scala, segno e dominio dei `Decimal` sopravvivono: `0`, `0.01`, negativi, quattro decimali, `MAX` e `MIN`. Nessuna scala 2 imposta a tutti | `10.0000` | riletto `10.0000`, scala 4 | TP-04 |
| **PER-09** | l'istante dell'evento conserva **nanosecondi e offset in secondi**, in due campi dichiarati | offset di 1 e 2 secondi | due eventi distinti, `texto_exato_evento` diverso prima e dopo la lettura | TP-06 |
| **PER-10** | assenza, `null`, stringa vuota e valore dichiarato restano distinti, con una policy **per campo** dichiarata in §7.4: non esiste una regola unica "il writer omette tutto ciò che è assente" | campo omesso contro `null` | entrambi letti come assenza: **limite dichiarato**, §3.5. La stringa vuota resta un valore | TP-03 |
| **PER-11** | ogni struct **del DTO dell'archivio** ha `deny_unknown_fields`; i tipi del modello non vengono usati come lettori (§7.3) | campo sconosciuto, anche annidato | `CampoDesconhecido` con il percorso. Senza il DTO verrebbe **accettato e scartato** (**[ESEGUITO]** §3.8) | TP-11 |
| **PER-12** | lo schema usa **struct** in ogni punto, mai mappe libere, e **non** passa per un albero di valori generico | chiave JSON duplicata, anche annidata | rifiutata da serde con `duplicate field` (**[ESEGUITO]** §3.8). Un albero `serde_json::Value` avrebbe **perso** la duplicazione, collassandola prima del controllo | TP-11 |
| **PER-13** | le impronte salvate vengono **ricalcolate** e confrontate; nessun aggiornamento automatico dell'hash per accettare un contenuto alterato | impronta o contenuto alterati | `ImpressaoDivergente`, nessuna riparazione. Un digest rileva incoerenza, **non** autentica l'autore | TP-12 |
| **PER-14** | ogni riferimento fra esecuzione, snapshot ed evento è verificato | `documentos` cita una chiave senza snapshot | `ReferenciaRompida` | TP-11 |
| **PER-15** | la costruzione del modello passa per i validatori esistenti: `Dado::novo`, `Competencia::new`, gli enum chiusi. Il DTO **non** costruisce stati incoerenti: li rifiuta | `Dado` con stato `OK` e valore assente | errore di `Dado::novo` propagato: controllo positivo **[ESEGUITO]** §3.8 | TP-03 |
| **PER-20** | ogni `Decimal` dell'archivio è una stringa convertita con `Decimal::from_str_exact`, **e** il testo riprodotto da `to_string()` deve coincidere con il testo letto | `"0.00000000000000000000000000001"`, `"3.5e1"`, `"+35.00"`, `"35.00 "` | `DecimalNaoExato` con il percorso: nessun arrotondamento correttivo, nessuno zero al posto di un valore. Vale per **tutti** i decimali, anche dentro `Dado`, rate, quantità e prezzi unitari; nessuna scala imposta | TP-04 |
| **PER-21** | lo strato nuovo ha DTO propri per **tutto** l'albero dell'archivio, contesti compresi; i tipi auditati non vengono modificati e non vengono usati come lettori. La conversione DTO → modello passa per i costruttori validati | nuovo campo nel modello non riportato nel DTO | il test di parità TP-21 fallisce: un campo che il DTO non conosce sarebbe un dato perso al salvataggio | TP-21 |
| **PER-16** | nessun importo illeggibile convertito in zero, nessuna riparazione automatica dell'archivio | JSON troncato, enum sconosciuto | errore nominato; la policy degli ingressi grezzi è D12, un'altra cosa | TP-11 |

---

### 7.3 Il lettore: perché i `Deserialize` esistenti non bastano

Riscontro **R3C3B-01** dell'audit del 2026-10-21, accolto. La revisione 1
diceva due cose incompatibili: «il contesto è esattamente la serializzazione
dei tipi esistenti, non serve un DTO» **e** «`deny_unknown_fields` su ogni
struct, lettura esatta dei decimali». Le garanzie non vengono dai
deserializzatori esistenti. Riprodotto contro i **tipi reali**
(**[ESEGUITO]** §3.8):

| Ingresso | `ContextoDocumento::deserialize` |
|---|---|
| `"campo_novo": 42` annidato nella riconciliazione | **accettato e scartato**: il modello risulta identico alla base |
| `"valor_total_nfe": 35` (numero) | **accettato**, scala da 2 a 0 |
| `"valor_total_nfe": "0.00000000000000000000000000001"` | **accettato come `0.0000000000000000000000000000`**: un valore diventa **zero** |
| `"valor_total_nfe": "35.000000000000000000000000000001"` | **accettato** con le cifre eccedenti eliminate |
| chiave duplicata annidata | **rifiutata**: `duplicate field` |
| `Dado` con `status: "OK"` e valore assente | **rifiutato** da `Dado::novo` |

Due conclusioni, e una va contro l'intuizione:

1. **richiedere una stringa non basta.** Il visitor di `rust_decimal` usa
   `Decimal::from_str`, che **arrotonda**. Serve `from_str_exact`, che rifiuta,
   più il confronto del testo riprodotto, perché `from_str_exact` accetta
   comunque `"+35.00"` (**[ESEGUITO]** §3.8). I due controlli insieme sono
   totali: uno prende la perdita di cifre, l'altro le forme non canoniche;
2. **un albero di valori generico peggiorerebbe le cose.** Le chiavi duplicate
   sono rifiutate **perché** si deserializza in struct: un passaggio
   intermedio per `serde_json::Value` le avrebbe collassate prima di qualunque
   controllo. Quindi la barriera non è un albero da ispezionare, sono le
   struct.

**Lettore proposto**, `PER-11`, `PER-20`, `PER-21`:

```text
 testo JSON
     |
     v  (1) serde_json -> DTO dell'archivio: struct annidate, tutte con
     |      deny_unknown_fields. Qui cadono campi sconosciuti (con il percorso)
     |      e chiavi duplicate. Ogni decimale e' tipizzato `String`.
     |
     v  (2) validazione esatta dei decimali: from_str_exact + to_string() == testo
     |      per ogni campo decimale dichiarato, compresi quelli dentro `Dado`,
     |      le rate, le quantita' e i prezzi unitari.
     |
     v  (3) costruzione del modello attraverso i costruttori validati:
     |      Dado::novo, Competencia::new, enum chiusi.
     |
     v  Arquivo, strutturalmente valido
```

I tre passaggi hanno responsabilità separate e nessuno copre il buco di un
altro: (1) la **forma**, (2) la **rappresentazione**, (3) gli **invarianti**.
Nessuno dei tre tocca i moduli auditati.

**Costo dichiarato.** Il DTO replica la forma di circa otto tipi
(`ContextoDocumento`, `AlocacaoNfe`, `Lancamento`, `Reconciliacao`,
`Componente`, `CampoRinviato`, `Duplicata`, `Aviso`, più `Dado` nelle sue
istanze). È duplicazione di **forma**, non di logica: nessuna regola di
dominio viene riscritta. Il rischio è che il DTO resti indietro quando il
modello cambia, e per questo `TP-21` verifica la parità con un'istanza
completamente popolata: un campo che il DTO non conosce si perderebbe nel
round-trip e il test fallisce.

### 7.4 Assenza: policy per campo

La revisione 1 prometteva «il writer omette i campi assenti». Non è vero per i
`Dado`: `valor` è serializzato come `null` esplicito e solo `motivo` viene
omesso, perché `skip_serializing_if` sta sul solo `motivo`
(**[LETTO_NEL_CODICE]** `modelos.rs:127-134`). Rettifica dopo l'audit:

| Campo | Assente si scrive | Letto da |
|---|---|---|
| `Dado::valor` | **`null` esplicito** | `null` oppure campo omesso: entrambi assenza |
| `Dado::motivo` | **omesso** | omesso oppure `null`: entrambi assenza |
| `evento.nota`, `evento.sequencia`, `evento.impressao_evento` | `null` esplicito, perché la loro assenza è un'informazione | `null` oppure omesso |
| `snapshot.impressao_apresentacao` | `null` esplicito | idem |
| `execucao.plano.sha256_arquivo`, `caminho_declarado` | `null` esplicito | idem |

Regola unica che resta vera: **omesso e `null` significano la stessa cosa in
lettura**, e nessuno dei due diventa mai uno zero, una stringa vuota o un
valore di default. Il writer è deterministico: dato lo stesso modello scrive
sempre la stessa forma, come richiede `TP-17`.

---

## 8. Identità, versioni, storico

### 8.1 Riuso, senza cambiare scopo

| Identità | Dove è definita | Uso nell'archivio |
|---|---|---|
| `ChaveRevisao` (senza lotto) | `revisao.rs:90` | bersaglio dell'evento, come è |
| `IdentidadeLinha` (con lotto) | `prelivro.rs:320` | **non** salvata: derivabile |
| `impressao_conteudo` | `IMPRESSAO-CONTEUDO/v1` | quale contenuto è stato esaminato |
| `impressao_apresentacao` | `IMPRESSAO-APRESENTACAO/v1` | quali nomi erano mostrati |
| `texto_exato_evento` | `EVENTO-EXATO/v1` | identità esatta dell'evento, e base di `impressao_evento` |
| `hash_registro` (C23) | `alocacao.rs` | **mai** chiave univoca, **mai** criterio di scarto: resta un campo del `Lancamento` dentro il contesto |

### 8.2 I casi dello storico

| Caso | Comportamento proposto |
|---|---|
| **reingresso identico** della stessa esecuzione | stesso `lote` e stesso contenuto: lo snapshot esiste già. Si verifica l'uguaglianza del contesto; se differisce, `SnapshotConflitante` invece di sovrascrivere |
| **nuovo contenuto, stessa chiave, stesso lotto** | conflitto: due contenuti per la stessa `(lote, chave)` non sono una storia, sono un errore del chiamante. `SnapshotConflitante` |
| **stesso contenuto in un altro lotto** | **due** snapshot, uno per esecuzione (PER-04). Gli eventi del primo si riapplicano al secondo, perché la corrispondenza è su `ChaveRevisao`, che non contiene il lotto (D8.1) |
| **nuovo contenuto in un lotto nuovo** | snapshot nuovo; gli eventi vecchi restano, citano l'impronta vecchia e risultano `VersaoSuperada` alla riapplicazione. **Nessun trasferimento** dell'approvazione, nemmeno a parità di totale o di hash C23 |
| **note diverse sullo stesso bersaglio** | due eventi, entrambi conservati |
| **stessa dichiarazione ripresentata** | l'archivio **conserva** entrambe le occorrenze; è la riapplicazione a dichiararle `Repeticao` e a contarle. Cancellare una dichiarazione sul disco sarebbe una perdita, non una deduplicazione |

L'ultimo punto è la differenza fra memoria e disco che il mandato chiede di non
confondere: `aplicar_revisao` collassa le ripetizioni **nella vista**, con
`repeticoes` che le conta; l'archivio non ha il diritto di dimenticarle.

### 8.3 Append-only logico

**[PROPOSTA]** `PER-17`: lo storico è **logicamente append-only**. Una
pubblicazione può riscrivere fisicamente il file, ma non può:

- rimuovere uno snapshot presente nel file che sta sostituendo;
- cambiare il `contexto` di uno snapshot esistente;
- cambiare i campi dichiarati di un evento esistente;
- **ridurre la molteplicità** degli eventi: se il file contiene due
  dichiarazioni identiche, il candidato deve contenerne due. Il confronto è su
  **multinsieme**, non su insieme — altrimenti la deduplicazione che la vista
  fa legittimamente in memoria diventerebbe una perdita sul disco (§8.2);
- rimuovere un'esecuzione o cambiarne i metadati, compresa l'identità del piano
  che ha usato.

Metadati protetti, per essere espliciti: `execucoes` (con il loro `plano`),
`snapshots`, `eventos` e la loro molteplicità. `revisao_arquivo` è l'unico
campo che **deve** cambiare, e lo assegna il pubblicatore (§9.1).

**Il confronto va fatto contro il file che si sta sostituendo, non contro la
base che il chiamante dice di aver letto.** Riscontro **R3C3B-02**, accolto:

```text
base A letta dal chiamante:  revisao 5, eventi [A]
file B nella destinazione:   revisao 5, eventi [B]
candidato derivato da A:     revisao 6, eventi [A,C]

append-only rispetto ad A           -> passa
confronto del solo contatore con B  -> passa, 5 == 5
append-only rispetto a B            -> FALLISCE: B scompare
```

Due archivi diversi possono condividere il contatore, quindi un `u64` non
vincola la pubblicazione allo stato preciso letto. Non serve uno scrittore
ostile: basta consegnare alla funzione un candidato preparato dalla base
sbagliata. Il protocollo di §9.1 rilegge e valida il file **dentro il
lucchetto** e confronta `verificar_acrescimo` contro **quel** contenuto, e la
base del chiamante diventa un `TokenBase` che include l'impronta del contenuto
letto (§10.2).

### 8.4 Conflitti e ordine

| Elemento | Ordine |
|---|---|
| eventi **nella vista** | ordine canonico a sei livelli della 3C.3.a (F3C-44): invariato, e l'archivio non lo cambia |
| eventi **nel file** | ordinati per `(instante assoluto, offset_segundos, chave_origem, conta_id, ordinal, operacao, texto_exato_evento, record serializzato)` |
| snapshot nel file | ordinati per `(lote, chave_origem)` |
| esecuzioni nel file | ordinate per `lote.id` |
| `execucao.documentos` | ordinati per `chave_origem` |
| `reconciliacao.componenti`, `campi_rinviati`, `articoli_scartati`, `avisos_documento`, `lancamento.duplicatas` | **ordine memorizzato**: sono dati del documento, e riordinarli cambierebbe il contenuto conservato |
| `avisos_leitura` | ordine memorizzato nel **record**; l'ordinamento canonico resta quello dell'impronta, che non riscrive il record |

L'ultimo livello della chiave degli eventi è il **testo esatto**, non
`impressao_evento`. Riscontro **R3C3B-04**, accolto: il digest è ammesso `null`
(§6.4), e due eventi con lo stesso istante e lo stesso bersaglio ma **note
diverse** e digest assente avevano chiavi fisiche **identiche**. Riprodotto
(**[ESEGUITO]** §3.8):

```text
chiave fisica di A e di B: identiche
ingresso AB -> prima "nota A"
ingresso BA -> prima "nota B"
con il testo esatto come ultimo livello: chiavi distinte
```

Un ordinamento stabile avrebbe conservato due ordini fisici diversi per lo
stesso contenuto, e uno instabile non avrebbe comunque una regola. Il testo
esatto è sempre calcolabile, è già iniettivo sui dati dichiarati (F3C-46) e non
dipende da un campo opzionale.

**Ma il testo esatto non basta per il file**, riscontro **R3C3B-04 residuo**
del 2026-10-22, accolto. Il record archivistico porta un campo che la
dichiarazione **non** ha: `impressao_evento`, ammesso `null` (§6.4). Due record
della **stessa** dichiarazione, uno senza digest e uno con il digest corretto,
sono entrambi ammessi dalla specifica, coincidono su **tutti** i livelli — testo
esatto compreso — e producono due file diversi secondo l'ordine d'ingresso.
Riprodotto (**[ESEGUITO]** §3.9):

```text
chiavi della revisione 2 identiche?                    true
i record serializzati sono uguali?                     false
ingresso [assente, presente] -> file 4ab5d3da...
ingresso [presente, assente] -> file 159dc1a7...
con lo spareggio sul record: 159dc1a7... in entrambi, 2 record conservati
```

Quindi l'ultimo livello è il **record archivistico interamente serializzato**.
Operativamente: si serializza **ogni** record in modo indipendente, si ordina
per la chiave con il suo testo come ultimo livello, poi si uniscono i testi —
nessuna circolarità, perché il testo di un record non dipende dalla posizione
degli altri.

Due proprietà che questo garantisce:

1. l'ordine è **totale sui record**, non solo sulle dichiarazioni: due record
   distinti hanno testi distinti, altrimenti sarebbero lo stesso record;
2. **copre i campi futuri**: qualunque campo aggiunto a `EventoArquivado`
   entra automaticamente nello spareggio, perché entra nel suo testo. È la
   stessa lezione di F3C-46, applicata al record invece che all'evento.

**Due livelli che restano distinti**, e non vanno confusi:

| Livello | Che cosa ordina | Dove è definito |
|---|---|---|
| identità e ordine della **dichiarazione umana** | la vista, la traccia, le decisioni | 3C.3.a, F3C-44: **invariato** |
| byte del **record archivistico** | il file su disco | §8.4, e comprende i metadati che solo l'archivio ha |

Il pareggio **non** si risolve scartando un record, normalizzando gli
originali o riempiendo in silenzio un digest assente: §7.4 prescrive `null`
quando è assente, e `PER-17` vieta di ridurre la molteplicità. Entrambe le
occorrenze restano nel file.

L'ordine **fisico** del file e l'ordine **canonico** della vista sono due cose
separate, come chiede il caso P-17: il primo serve a rendere i file
confrontabili, il secondo a rendere la vista riproducibile. Nessuna politica
«vince l'ultimo file»: §9.5.

---
## 9. Salvataggio, errore, recupero, concorrenza

### 9.1 Il protocollo, passo per passo

**[PROPOSTA]**. Tutti i file nella **stessa directory**, quindi sullo stesso
volume: è il presupposto del `rename` (§9.6).

```text
 0. PREPARA    costruisci l'archivio in memoria e valida la struttura
               (PER-01..PER-21). NON serializzare ancora: il numero di
               revisione non e' noto. Un errore qui non ha toccato nessun file.
 1. LUCCHETTO  crea `arquivo.lock` con create_new, scrivendoci i metadati
               DICHIARATI dal chiamante. Se esiste -> ArquivoOcupado, con il
               contenuto del lucchetto nel messaggio.
 2. RILEGGI    leggi e valida il file corrente. Da qui in avanti il confronto
               e' contro QUESTO contenuto, non contro la base del chiamante:
               a) destinazione assente  -> ramo di INIZIALIZZAZIONE (9.1.1)
               b) presente e valido     -> confronta il TokenBase dichiarato
                  (revisione + impronta del contenuto). Diverso ->
                  BaseDesatualizada
               c) presente e NON valido -> ArquivoCorrenteInvalido: non si
                  pubblica sopra un file che non si sa leggere. Il recupero
                  e' una chiamata diversa (9.4)
 3. ACCRESCIMENTO verificar_acrescimo(file_corrente, candidato): multinsieme
               degli eventi, snapshot, esecuzioni e loro metadati (PER-17).
               Fallisce -> HistoriaTruncada
 4. NUMERA     revisao_arquivo = corrente + 1, con checked_add.
               Esaurito -> ContadorEsgotado. Solo ora si serializza, cosi' il
               testo pubblicato contiene il numero verificato sotto lucchetto
 5. TEMPORARIO scrivi `arquivo.json.novo.<revisao>` con create_new, write_all,
               sync_all. Il nome porta la revisione e NON viene troncato: se
               esiste un residuo -> ResiduoPresente, esplicito (9.3)
 6. GENERAZIONE copia `arquivo.json` in `arquivo.json.ger<revisao_base>` con
               create_new e sincronizza. L'originale resta al suo posto.
               Nel ramo di inizializzazione questo passo NON esiste
 7. PUBBLICA   fs::rename(temporaneo, `arquivo.json`)      <== PUNTO DI COMMIT
 8. RILASCIA   rimuovi `arquivo.lock`. Un fallimento qui NON annulla la
               pubblicazione: PublicacaoComLucchettoResiduo (9.3)
```

Il **punto di commit** è il passo 7, e uno solo. Prima di quel passo
`arquivo.json` non è mai stato aperto in scrittura: nessun passo precedente
può danneggiarlo.

Tre cose sono cambiate rispetto alla revisione 1, per il riscontro
**R3C3B-02** e **R3C3B-03**: il confronto dell'accrescimento avviene contro il
**file riletto** e non contro la base del chiamante; la **serializzazione**
avviene dopo l'assegnazione del numero, dentro il lucchetto; il temporaneo usa
`create_new` e un nome che porta la revisione, così non può troncare un
candidato conservato per il recupero.

#### 9.1.1 Inizializzazione: la prima pubblicazione

La revisione 1 affermava che «esiste sempre una generazione precedente». Non è
vero per il primo archivio, e il protocollo rileggeva e copiava sempre un file
che non esiste. **[PROPOSTA]** `PER-22`:

| Elemento | Nel ramo di inizializzazione |
|---|---|
| base dichiarata | **`None`**: il chiamante dichiara di aspettarsi una destinazione vuota. Nessun numero inventato |
| destinazione già presente | `DestinoJaExistente`: non si inizializza sopra qualcosa |
| passo 3, accrescimento | non si applica: non c'è un file da non troncare |
| passo 4, numerazione | `revisao_arquivo = 1`, con fonte dichiarata: è la prima pubblicazione di **questo file**, non la versione di un documento |
| passo 6, generazione | **non esiste**: non c'è niente da conservare |
| recuperabilità | **nessuna generazione** dopo la prima pubblicazione. È un limite dichiarato, non una promessa mancata |

### 9.2 Perché non "cancella poi rinomina"

**[ESEGUITO]** §3.6: `remove_file` riesce anche su un file **aperto**, e il
file spariva subito. Una strategia "cancella l'originale, poi rinomina il
nuovo" lascerebbe quindi una finestra in cui **l'archivio non esiste affatto**:
un'interruzione lì non lascia né il vecchio né il nuovo. È vietata.

Non serve nemmeno: **[ESEGUITO]** `fs::rename` su una destinazione esistente
riesce su Windows, compreso quando un altro handle aperto da Rust la tiene in
lettura o in scrittura. Il mandato chiedeva di dimostrarlo invece di dedurlo
dal nome della funzione, ed è dimostrato — con il limite di §9.6 punto 2.

### 9.3 Atomicità, recuperabilità, durabilità: tre cose diverse

| Proprietà | Che cosa si può affermare | Prova |
|---|---|---|
| **atomicità** della sostituzione | il `rename` pubblica il file **intero** o non lo pubblica: nessun lettore vede metà del nuovo contenuto *per effetto del rename* | **[ESEGUITO]** §3.6; `MoveFileEx` con sostituzione |
| **integrità** del contenuto pubblicato | il temporaneo è stato scritto e **sincronizzato** prima del commit, quindi ciò che viene pubblicato è completo | **[ESEGUITO]** `sync_all` sul file riesce |
| **recuperabilità** | **dalla seconda pubblicazione in poi** esiste una generazione precedente validabile, e il candidato si sceglie in modo esplicito. Dopo la **prima** pubblicazione non esiste: §9.1.1 | §9.4 |
| **durabilità** della voce di directory creata dal `rename` | **[NON_VERIFICATO]**: su Windows `sync_all` su una **directory** è negato (`PermissionDenied`, os error 5), quindi non si può forzare la sincronizzazione della directory. Dopo una perdita di alimentazione subito dopo il commit, non è dimostrabile quale nome sopravviva | **[ESEGUITO]** §3.6 punto 9 |

Gli esiti per fase, **[PROPOSTA]**:

| Momento del guasto | Che cosa resta | Esito dichiarato |
|---|---|---|
| passi 0–4 | `arquivo.json` **intatto**: non è stato aperto in scrittura | errore nominato, nessuna modifica |
| passo 5 o 6 | `arquivo.json` intatto; restano `*.novo.<rev>` e forse `*.ger<n>` | errore nominato; i resti hanno nomi che portano la revisione, e **non** vengono cancellati in automatico |
| **passo 7** | **incerto**: o il vecchio o il nuovo, **entrambi completi e validi** | `PublicacaoIncerta`: il chiamante rilegge e decide. Non si afferma che il precedente sia sempre intatto, perché a quel punto non è dimostrabile |
| dopo il passo 7, prima dell'8 | nuovo **pubblicato**, lucchetto rimasto | `PublicacaoComLucchettoResiduo`: la pubblicazione **è valida**, e lo sblocco è un'azione esplicita (§9.4). Trattarla come un fallimento farebbe ripubblicare un contenuto già pubblicato |
| perdita di alimentazione | il contenuto scritto e sincronizzato non è lacerato, ma **quali nomi** sopravvivano — il corrente, il temporaneo, la generazione appena copiata — è **[NON_VERIFICATO]** | il recupero elenca i candidati e li valida; nessuna affermazione su quale debba esserci |

Questa tabella è l'unica promessa che il protocollo può mantenere. Dire "ogni
errore lascia il precedente invariato" sarebbe falso al passo 7, ed è
esattamente il punto 21 del mandato. E nemmeno la **recuperabilità** dopo una
perdita di alimentazione è dimostrata: la copia della generazione al passo 6
viene sincronizzata, ma la sua voce di directory ricade nello stesso limite del
commit, perché su Windows la directory non si può sincronizzare.

### 9.4 Generazione precedente e recupero

**[PROPOSTA]** `PER-18`:

- a ogni pubblicazione il passo 4 crea `arquivo.json.ger<revisao_base>`;
- **nessuna generazione viene cancellata** dalla libreria: la potatura è una
  decisione a sé (D16);
- il recupero **non è automatico**: `geracoes()` elenca i candidati — il file
  corrente, le generazioni, un eventuale `*.novo` rimasto — e per ognuno
  restituisce esito di validazione, `revisao_arquivo` e conteggi. **È il
  chiamante a scegliere**;
- un file corrente corrotto **non** fa caricare in silenzio una generazione
  vecchia: `ler()` restituisce l'errore del file corrente, e il recupero è una
  chiamata diversa, con fonte ed esito dichiarati;
- un lucchetto rimasto non viene **mai** cancellato automaticamente: contiene
  l'identità dichiarata di chi l'ha preso e l'istante dichiarato — entrambi
  **parametri** di `publicar`, non dedotti dall'ambiente — e la sua rimozione è
  un'azione esplicita del chiamante.

### 9.4.1 Le transizioni del ciclo di vita

Riscontro **R3C3B-03**, accolto: elencare i candidati non è ancora un ciclo
operativo. Ogni stato della destinazione ha una transizione dichiarata,
**[PROPOSTA]** `PER-23`:

| Stato della destinazione | `ler` | `publicar(base)` | Operazione che risolve |
|---|---|---|---|
| **assente** | `DestinoAusente` | ammessa **solo** con `base = None`: inizializzazione (§9.1.1) | — |
| corrente **valido** | `Leitura` con `TokenBase` | ammessa con il token corrispondente | — |
| corrente **non valido** | errore del corrente, **mai** un ripiego su una generazione | rifiutata: `ArquivoCorrenteInvalido` | `recuperar`: §9.4.2 |
| **temporaneo residuo** presente | ignorato: `ler` legge solo il corrente | rifiutata: `ResiduoPresente`, con il nome del residuo | il chiamante lo esamina con `candidatos` e poi lo promuove o lo sposta: nessuna cancellazione dalla libreria |
| **generazione** già presente con lo stesso numero | ignorata da `ler` | rifiutata: `GeracaoJaPresente` | come sopra |
| **lucchetto rimasto** | `ler` funziona: leggere non richiede il lucchetto | rifiutata: `ArquivoOcupado` con i metadati dichiarati del lucchetto | `liberar(destino, confirmacao)`: esplicita |
| pubblicazione con esito **incerto** | `ler` dice quale contenuto c'è ora | il chiamante rilegge, confronta il `TokenBase` e decide se ripubblicare | `candidatos` per vedere tutte le generazioni |

Il principio dietro la tabella: **la libreria non ripara niente da sola**.
Preferisce fermarsi con un errore che nomina il residuo, perché cancellare un
candidato è esattamente perdere la prova di recupero che §9.3 promette di
conservare.

### 9.4.2 Recupero esplicito di un candidato

```text
recuperar(destino, candidato_scelto, esperado: &EstadoObservado, lucchetto)
```

**[PROPOSTA]** `PER-24`. Il recupero è una funzione a sé, non un ramo di
`publicar`, proprio perché le precondizioni di `publicar` qui non valgono:

1. prende il lucchetto come il passo 1 di §9.1;
2. **osserva lo stato effettivo della destinazione e lo confronta con
   `esperado`**. Diverso ⇒ `EstadoMudou { esperado, encontrado }`, e **niente
   viene toccato**: né il corrente conservato, né il candidato promosso. Il
   chiamante deve scegliere di nuovo, sullo stato nuovo;
3. **valida** il candidato scelto: se non passa `verificar_estrutura`,
   `CandidatoInvalido` e niente viene toccato;
4. se il corrente esiste — anche non valido — lo **conserva** come
   `arquivo.json.corrupto<revisao o istante dichiarato>`: non viene mai
   cancellato, perché un file corrotto è una prova;
5. pubblica il candidato con lo stesso `rename` del passo 7;
6. rilascia il lucchetto, con lo stesso esito dichiarato del passo 8.

**Perché il passo 2 esiste**, riscontro **R3C3B-03 residuo** del 2026-10-22,
accolto. Il lucchetto garantisce la mutua esclusione **al momento del commit**,
non copre il tempo fra l'**osservazione** del chiamante e l'acquisizione del
lucchetto. Riprodotto (**[ESEGUITO]** §3.9):

```text
1. il chiamante osserva un corrente corrotto e scegli il candidato rev 1 [A]
2. un'altra operazione COOPERANTE pubblica: corrente valido, rev 3, [A,B];
   il lucchetto torna libero
3. il recupero prende regolarmente il lucchetto e promuove la rev 1
   -> B non e' piu' nel corrente
```

I byte di `[A,B]` restano come prova, quindi **non** è una perdita fisica
irreversibile: è un **rollback non confermato**, su uno stato diverso da quello
su cui la decisione era stata presa. Con il confronto del passo 2 la sequenza
si ferma: impronte diverse, `EstadoMudou`, nessuna sostituzione
(**[ESEGUITO]** §3.9).

Il controllo **non vieta** un rollback intenzionale: se il chiamante vuole
tornare alla revisione 1 sapendo che il corrente è la 3, osserva la 3, la
dichiara come `esperado` e il recupero procede. Impedisce soltanto che il
**bersaglio cambi senza conferma**.

Lo stato atteso deve coprire anche i casi in cui un `TokenBase` non esiste,
perché il file non si riesce a deserializzare o non c'è affatto:

```text
EstadoObservado :=
    DestinoAusente
  | Corrente { sha256_bytes: String, token: Option<TokenBase> }
```

L'impronta dei **byte** funziona anche su un file illeggibile, ed è ciò che
rende confrontabile uno stato corrotto; `token` è presente solo quando il
corrente era deserializzabile. `candidatos()` restituisce anche questo stato,
così il chiamante lo ripassa a `recuperar` senza doverlo ricostruire.

Il numero di revisione del candidato **non** viene riscritto: il recupero
ripristina uno stato che è esistito, non ne fabbrica uno nuovo. La
pubblicazione successiva riprenderà da quel numero, e il salto eventuale nella
sequenza è un fatto osservabile, non un buco da nascondere.

### 9.5 Concorrenza: lucchetto consultivo più base dichiarata

Il problema vero non è la scrittura simultanea: è **leggi, modifica, scrivi**.
Due chiamanti che leggono la stessa base e pubblicano entrambi perderebbero gli
eventi del primo.

Due meccanismi insieme, **[PROPOSTA]** `PER-19`:

1. **lucchetto consultivo**: `arquivo.lock` creato con `create_new`.
   **[ESEGUITO]** §3.6: la seconda creazione fallisce con `AlreadyExists`, quindi
   funziona come esclusione fra processi che lo rispettano;
2. **base dichiarata**: il chiamante passa il `TokenBase` — revisione **e
   impronta del contenuto letto**, §8.3 — da cui è
   partito; il passo 2 la riconfronta **dentro** il lucchetto. Diversa ⇒
   `BaseDesatualizada`, e il chiamante rilegge e riapplica.

Perché servono entrambi, e che cosa resta scoperto:

| Scenario | Esito |
|---|---|
| due scrittori che rispettano il lucchetto | il secondo attende o riceve `ArquivoOcupado`; nessuna perdita |
| uno scrittore che ignora il lucchetto, ma dichiara la base | il controllo del passo 2 lo blocca **se** la pubblicazione concorrente è già avvenuta |
| due scrittori che ignorano **entrambi** il lucchetto | **[NON_VERIFICATO]** e non eliminabile da questa libreria: fra il controllo del passo 2 e il `rename` del passo 7 esiste una finestra. `std` non offre lock obbligatori, e §3.6 mostra che Windows non li impone |
| un lettore mentre uno scrittore pubblica | il lettore vede il vecchio **o** il nuovo, perché il commit è un `rename`. **[ESEGUITO]**: la lettura concorrente è permessa, quindi leggere il **temporaneo** sarebbe sbagliato — e nessuno lo fa |

La libreria **non** assume un unico scrittore: dichiara il vincolo
("rispettare il lucchetto") e lo rende verificabile con un errore nominato.
Un controllo seguito da scrittura senza lucchetto non eliminerebbe la corsa, e
questa specifica non pretende il contrario.

### 9.6 Vincoli d'uso dichiarati

| Vincolo | Stato |
|---|---|
| temporaneo, generazione e destinazione nella **stessa directory** | requisito: un `rename` fra volumi diversi non è una sostituzione atomica |
| destinazione tenuta da un handle **senza condivisione della cancellazione** | **[RIUSATO]**: l'audit del 2026-10-21 l'ha provato con un handle .NET `FileShare.None` — la sostituzione viene **rifiutata** e il precedente resta. Il `rename` fallito è quindi un esito reale e nominato, e la tecnica è disponibile per il test TP-14 senza installare niente |
| volumi di rete, FAT32, cartelle sincronizzate come OneDrive | **[NON_VERIFICATO]**: nessuna prova. Uso sconsigliato finché non provato |
| percorso dell'archivio | sempre dal **chiamante**: la libreria non sceglie cartelle, non legge variabili d'ambiente, non crea percorsi impliciti |

### 9.7 Metadati: quali sono dichiarati e quali sono tecnici

| Dato | Provenienza |
|---|---|
| autore, istante, motivo, nota, sequenza dell'evento | **dichiarati** da chi opera, come in 3C.3.a |
| `IdentidadeLote` | dichiarata dal chiamante |
| percorso dell'archivio, nome del file | dal chiamante |
| `revisao_arquivo` | tecnico, della libreria |
| data di modifica del file, utente del processo | **non usati**: non sono dichiarazioni, e nessun campo dell'archivio li contiene |

I metadati tecnici di una scrittura **non** sostituiscono quelli umani: un
`revisao_arquivo` che cresce non dice chi ha approvato niente.

---

## 10. API proposte

Firme indicative. Due moduli nuovi: i moduli già auditati **non si toccano**.

### 10.1 `src/arquivo.rs` — codec e validazione, puro

```rust
pub const FORMATO: &str = "FANTOZZI-ARQUIVO/v1";
pub const VERSAO_SCHEMA: u32 = 1;

pub struct Arquivo {
    pub revisao_arquivo: u64,
    pub execucoes: Vec<Execucao>,
    pub snapshots: Vec<Snapshot>,
    pub eventos: Vec<EventoArquivado>,
}

pub struct Execucao {
    pub lote: IdentidadeLote,
    pub plano: IdentidadePlano,          // il piano di QUESTA esecuzione
    pub documentos: Vec<String>,         // ordinati per chave_origem
}

pub struct IdentidadePlano {
    pub versao: String,
    pub sha256_arquivo: Option<String>,
    pub caminho_declarado: Option<String>,
}

pub struct Snapshot {
    pub lote: String,
    pub chave_origem: String,
    pub impressao_conteudo: String,
    pub impressao_apresentacao: Option<String>,
    pub contexto: ContextoDocumento,          // modello in memoria; letto dal DTO di §7.3
}

/// DTO dell'evento: esiste perche' `EventoRevisao` ha solo `Serialize`
/// (`revisao.rs:134`) e perche' il `Serialize` di `chrono` perde i secondi
/// dell'offset (§3.4). Converte in entrambe le direzioni.
pub struct EventoArquivado { /* campi di §6.4 */ }

impl EventoArquivado {
    pub fn de_evento(evento: &EventoRevisao) -> Self;
    pub fn para_evento(&self) -> Result<EventoRevisao, ErroArquivo>;
}

/// Serializza con l'ordine fisico di §8.4. Il numero di revisione e' un
/// parametro: lo assegna il pubblicatore sotto lucchetto (§9.1 passo 4).
pub fn serializar(arquivo: &Arquivo, revisao: u64) -> Result<String, ErroArquivo>;

/// Lettore restrittivo di §7.3: DTO con `deny_unknown_fields`, validazione
/// esatta dei decimali, poi i costruttori validati del modello.
pub fn desserializar(texto: &str) -> Result<Arquivo, ErroArquivo>;

/// Validazione che **non** richiede il piano: riferimenti, coerenza
/// snapshot/contesto e **impronta di contenuto** ricalcolata (PER-13, PER-14).
/// La usano `ler` e `publicar`.
pub fn verificar_estrutura(arquivo: &Arquivo) -> Result<(), ErroArquivo>;

/// Confronto della **presentazione**, che richiede il piano corrente. Una
/// differenza **non** e' un errore: e' `APRESENTACAO_MUDADA` secondo D8.3.
/// Chiamata a parte, dal chiamante che ha il piano in mano.
pub fn verificar_apresentacao(
    arquivo: &Arquivo,
    plano: &PlanoContas,
) -> Result<Vec<Divergencia>, ErroArquivo>;

/// Append-only (PER-17): confronta il **file che si sta sostituendo** con il
/// candidato. Il multinsieme degli eventi non puo' ridursi.
pub fn verificar_acrescimo(corrente: &Arquivo, candidato: &Arquivo) -> Result<(), ErroArquivo>;

/// Ingressi per la riapplicazione: contesti di un lotto ed eventi.
pub fn contextos_do_lote(arquivo: &Arquivo, lote: &str) -> Result<Vec<ContextoDocumento>, ErroArquivo>;
pub fn eventos(arquivo: &Arquivo) -> Result<Vec<EventoRevisao>, ErroArquivo>;

pub enum ErroArquivo {
    FormatoDesconhecido { encontrado: String },
    SchemaMaisNovo { encontrado: u32, suportado: u32 },
    GramaticaDesconhecida { campo: &'static str, encontrada: String, suportada: String },
    CampoDesconhecido { caminho: String, campo: String },
    MonetarioNaoTextual { caminho: String },
    ImpressaoDivergente { chave_origem: String, salva: String, recalculada: String },
    ReferenciaRompida { de: String, para: String },
    SnapshotIncoerente { chave_origem: String, campo: &'static str },
    SnapshotConflitante { lote: String, chave_origem: String },
    HistoriaTruncada { o_que: String },
    Modelo(String),                 // errore di un validatore esistente
    Json { linha: usize, coluna: usize, detalhe: String },
}
```

### 10.2 `src/arquivo_io.rs` — lo strato IO

```rust
pub struct Destino { pub diretorio: PathBuf, pub nome: String }   // dal chiamante

/// Che cosa il chiamante ha letto. **Non** solo il contatore: due archivi
/// diversi possono condividerlo (§8.3, R3C3B-02).
pub struct TokenBase {
    pub revisao: u64,
    pub sha256_conteudo: String,       // impronta dei byte letti
}

pub struct Leitura { pub arquivo: Arquivo, pub base: TokenBase }

/// Metadati del lucchetto: **dichiarati dal chiamante**, non dedotti.
pub struct MetadadosLucchetto {
    pub operador: String,              // non vuoto
    pub instante: DateTime<FixedOffset>,
    pub motivo: String,
}

pub struct Publicacao {
    pub revisao_publicada: u64,
    pub geracao_anterior: Option<PathBuf>,   // None alla prima pubblicazione
    pub lucchetto_residuo: bool,             // true se il passo 8 e' fallito
}

pub struct CandidatoRecuperacao {
    pub caminho: PathBuf,
    pub papel: PapelCandidato,        // Corrente | Geracao | TemporarioRestante
    pub revisao_arquivo: Option<u64>,
    pub validacao: Result<(), ErroArquivo>,
    pub snapshots: usize,
    pub eventos: usize,
}

pub struct Candidatos {
    pub estado: EstadoObservado,      // da ripassare a `recuperar`
    pub candidatos: Vec<CandidatoRecuperacao>,
}

pub fn ler(destino: &Destino) -> Result<Leitura, ErroIo>;

/// Esegue i passi 1..8 di §9.1. `base = None` significa **inizializzazione**
/// (§9.1.1): nessun numero inventato e nessuna generazione da copiare.
pub fn publicar(
    destino: &Destino,
    arquivo: &Arquivo,
    base: Option<&TokenBase>,
    lucchetto: &MetadadosLucchetto,
) -> Result<Publicacao, ErroIo>;

/// Elenca e valida i candidati; non carica niente in automatico (PER-18).
pub fn candidatos(destino: &Destino) -> Result<Candidatos, ErroIo>;

/// Recupero **esplicito** di §9.4.2: promuove un candidato validato, conserva
/// il corrente come prova e non riscrive il numero di revisione.
/// Stato della destinazione come il chiamante l'ha **osservato**. Copre anche
/// la destinazione assente e il corrente illeggibile, dove un `TokenBase` non
/// esiste (§9.4.2).
pub enum EstadoObservado {
    DestinoAusente,
    Corrente { sha256_bytes: String, token: Option<TokenBase> },
}

/// Osserva la destinazione senza interpretarla: funziona anche su un file che
/// non si riesce a deserializzare.
pub fn observar(destino: &Destino) -> Result<EstadoObservado, ErroIo>;

pub fn recuperar(
    destino: &Destino,
    candidato: &Path,
    esperado: &EstadoObservado,
    lucchetto: &MetadadosLucchetto,
) -> Result<Publicacao, ErroIo>;

/// Sblocco esplicito: non cancella mai un lucchetto senza che il chiamante
/// confermi quale lucchetto sta rimuovendo.
pub fn liberar(destino: &Destino, confirmacao: &MetadadosLucchetto) -> Result<(), ErroIo>;

pub enum ErroIo {
    DestinoAusente { caminho: PathBuf },
    DestinoJaExistente { caminho: PathBuf },
    ArquivoCorrenteInvalido { caminho: PathBuf, causa: ErroArquivo },
    ResiduoPresente { caminho: PathBuf },
    GeracaoJaPresente { caminho: PathBuf },
    ArquivoOcupado { lucchetto: PathBuf, conteudo_declarado: String },
    BaseDesatualizada { esperada: TokenBase, encontrada: TokenBase },
    EstadoMudou { esperado: EstadoObservado, encontrado: EstadoObservado },
    HistoriaTruncada { detalhe: String },
    ContadorEsgotado { revisao_corrente: u64 },
    CandidatoInvalido { caminho: PathBuf, causa: ErroArquivo },
    PublicacaoIncerta { detalhe: String, candidatos: Vec<PathBuf> },
    Conteudo(ErroArquivo),
    Io { operacao: &'static str, caminho: PathBuf, erro: std::io::Error },
}
```

Dove avviene **quale** validazione, per non lasciare in giro un archivio
«grezzo» (punto 5 del riscontro R3C3B-02):

| Funzione | Valida | Non valida |
|---|---|---|
| `desserializar` | forma, rappresentazione, invarianti del modello (§7.3) | riferimenti incrociati e impronte |
| `ler` | `desserializar` **più** `verificar_estrutura` | la presentazione: non riceve il piano |
| `publicar` | come `ler` sul file corrente, più `verificar_acrescimo` | la presentazione |
| `verificar_apresentacao` | il confronto con il piano corrente | — |

Non esiste un percorso che restituisca un `Arquivo` non validato: `ler` o
restituisce un archivio strutturalmente valido o un errore. La presentazione è
l'unica verifica separata, perché il suo esito **non è un errore**.

### 10.3 File da toccare, e quelli da non toccare

| File | Intervento |
|---|---|
| `src/arquivo.rs` | **nuovo** |
| `src/arquivo_io.rs` | **nuovo** |
| `src/lib.rs` | due righe di esportazione |
| `Cargo.toml`, `Cargo.lock` | **aggiunta di `serde_json = "1.0.154"`, da autorizzare** nel mandato implementativo (§3.2) |
| `tests/persistencia_entre_processos.rs` | **nuovo** test di integrazione, nominato: §12.2 |
| `README.md` | stato, API, limiti, conteggio |
| `src/revisao.rs`, `src/impressao.rs`, `src/prelivro.rs`, `src/alocacao.rs`, `src/dinheiro.rs`, `src/modelos.rs`, `src/nfe.rs`, `src/regras.rs`, `src/main.rs` | **non toccati** |

Il punto 25 del mandato è rispettato alla lettera: i DTO stanno nei moduli
nuovi, e in particolare **D8.5 non viene cambiata** in `revisao.rs` per far
contento il writer (§11, D17).

---

## 11. Decisioni: approvate il 2026-10-22

Quattro, tutte **approvate da Luca il 2026-10-22 nell'opzione (a)**, dopo il
superamento dell'audit tecnico della specifica. Le opzioni scartate restano
scritte: una decisione senza le alternative non e' verificabile. Nessuna di
queste riapre D7, D8, D9, D10 o D13.

### D14 — granularità del file — **APPROVATA (a)**

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** un **solo** file JSON per archivio | protocollo di pubblicazione semplice e atomico con un `rename`; il file cresce con lo storico | **raccomandata** per il primo lotto: una sola sostituzione atomica, nessun impegno su più file da mantenere coerenti |
| (b) una directory con un file per esecuzione più un indice | file piccoli, ma la pubblicazione tocca **due** file e l'atomicità di un `rename` non copre più l'insieme: servirebbe un protocollo a due fasi | rinviata: utile se il file di (a) diventasse ingestibile |
| (c) un file per esecuzione **senza** indice | niente da aggiornare in comune, ma nessun punto unico di verità e nessun `revisao_arquivo` globale | sconsigliata: perde il controllo di concorrenza |

Scegliendo (a) la migrazione verso (b) resta possibile: `versao_schema` sale a 2
e il lettore vecchio rifiuta con `SchemaMaisNovo` invece di indovinare.

### D15 — politica di concorrenza — **APPROVATA (a)**

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** lucchetto consultivo **più** base dichiarata (§9.5) | blocca i casi reali e nomina l'errore; la finestra fra controllo e commit resta dichiarata | **raccomandata** |
| (b) solo base dichiarata | nessun file di lucchetto da gestire, ma la corsa fra controllo e `rename` resta aperta anche fra processi cooperanti | sconsigliata |
| (c) solo lucchetto | chi lo ignora sovrascrive senza accorgersene | sconsigliata |

### D16 — potatura delle generazioni — **APPROVATA (a): nessuna potatura**

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** nessuna potatura automatica | lo storico fisico non si perde mai; la cartella cresce | **raccomandata** per il primo lotto |
| (b) tenere le ultime N | cartella sotto controllo, ma la libreria cancella storia | richiede una decisione esplicita su N e su chi autorizza |

### D17 — numerazione storica dei documenti — **APPROVATA (a): rinviata**

`VersaoDocumento::sequencia` è `NAO_DISPONIVEL` per D8.5. L'archivio **potrebbe**
derivarne una, contando gli snapshot distinti per `chave_origem`.

| Opzione | Effetto | Giudizio |
|---|---|---|
| **(a)** **rinviare**: la sequenza resta assente anche con l'archivio | nessun numero senza una regola approvata; `D8.5` resta com'è, `revisao.rs` non si tocca | **raccomandata**: il primo lotto non ne ha bisogno |
| (b) derivarla dall'ordine di comparsa negli snapshot | un numero con fonte, ma dipendente da **quando** un'esecuzione è stata archiviata, non da quando è avvenuta | se serve, va deciso con una regola scritta |

---
## 12. Matrice delle prove

Tutte **PIANIFICATE**: nessuna eseguita. La numerazione `TP-nn` segue i casi
`P-nn` del mandato, così la corrispondenza è diretta.

### 12.1 I diciotto casi richiesti

| Test | Caso | Ingresso minimo | Atteso | Contratto | Blocco |
|---|---|---|---|---|---|
| **TP-01** | P-01 | campione canonico allocato, un `Aprovar`; **destinazione davvero nuova**, quindi inizializzazione con `base = None`; salva, **chiudi il processo**, rileggi in un processo nuovo | `aplicar_revisao` sui dati riletti dà la **stessa** `VistaRevisada`: traccia, esclusioni, riepiloghi, estratto, impronte. `revisao_publicada = 1` e `geracao_anterior = None` | PER-02, PER-03, PER-22 | 3C.3.b.1 |
| **TP-02** | P-02 | documento **senza righe**, con `esito_total` e `stato_documento` `ERRO` come l'esempio di §5.2 | identità, riconciliazione integra e avvisi identici dopo la rilettura; il documento compare (I8) | PER-02 | 3C.3.b.1 |
| **TP-03** | P-03 | `Dado` assente con motivo, `Dado` presente senza motivo, valore zero reale, stringa vuota | quattro esiti **distinti** dopo la rilettura; `Dado::novo` rifiuta uno stato incoerente costruito a mano nel JSON | PER-10, PER-15 | 3C.3.b.1 |
| **TP-04** | P-04 | `Decimal::MAX`, `MIN`, `0.01`, `-7.50`, `10.0000`, quantità a quattro decimali; più sei ingressi ostili: un **numero** al posto della stringa, `"0.00000000000000000000000000001"`, `"35.000000000000000000000000000001"`, `"3.5e1"`, `"+35.00"`, `"35.00 "` | valori e scale identici nei casi legittimi, nessun float; i sei ostili **rifiutati** con `MonetarioNaoTextual` o `DecimalNaoExato`, mai zero e mai arrotondamento correttivo. Da ripetere su un decimale **dentro un `Dado`**, su una rata e su una quantità | PER-06, PER-07, PER-08, PER-20 | 3C.3.b.1 |
| **TP-05** | P-05 | chiave e CNPJ con zeri iniziali, `descricao_itens` in NFC e in NFD, nota che comincia con `=` | byte originali identici dopo la rilettura; nessuna normalizzazione; nessun testo interpretato | PER-08, §7.1 | 3C.3.b.1 |
| **TP-06** | P-06 | due eventi con offset di **1** e **2** secondi; uno con nanosecondi; due con lo stesso istante assoluto e offset diversi | i quattro restano distinti; `texto_exato_evento` **identico prima e dopo** la rilettura; nessuna falsa `Repeticao` | PER-09 | 3C.3.b.1 |
| **TP-07** | P-07 | documento con due righe di uguale `hash_registro` C23, un evento sulla seconda | due bersagli distinti dopo la rilettura, nessuno scarto | PER-02, §8.1 | 3C.3.b.1 |
| **TP-08** | P-08 | `APROVADO` senza provenienza, sanatoria, revoca, ri-approvazione | gli esiti della 3C.3.a si riproducono **identici** dopo la rilettura, estratto compreso | PER-03 | 3C.3.b.1 |
| **TP-09** | P-09 | stesso contenuto in due lotti; poi contenuto nuovo a parità di totale | due snapshot distinti; eventi del primo riapplicati al secondo; **nessun trasferimento** al contenuto corretto | PER-04, §8.2 | 3C.3.b.1 |
| **TP-10** | P-10 | la stessa dichiarazione salvata due volte | **entrambe** presenti nel file; la vista le dichiara `Repeticao` e le conta; nessuna perdita sul disco | §8.2, PER-17 | 3C.3.b.1 |
| **TP-11** | P-11 | un file malformato per **ciascuna** condizione: schema maggiore, grammatica sconosciuta, enum non ammesso, campo obbligatorio assente, JSON troncato, chiave duplicata **annidata**, campo sconosciuto **annidato**, riferimento rotto in `execucao.documentos` | un errore **nominato e distinto** per condizione, con il **percorso** del campo; nessuna lettura parziale, nessuna riparazione. Il conteggio dei file segue le condizioni, non una quota | PER-01, PER-11, PER-12, PER-14, PER-16 | 3C.3.b.1 |
| **TP-12** | P-12 | impronta salvata alterata; contesto alterato lasciando l'impronta; `snapshot.lote` diverso da `contexto.lote.id` | `ImpressaoDivergente` e `SnapshotIncoerente`; **nessun** aggiornamento dell'hash | PER-13 | 3C.3.b.1 |
| **TP-13** | P-13 | errore iniettato fra i passi **5 e 6** di §9.1, per esempio temporaneo non scrivibile o generazione non copiabile | `arquivo.json` **invariato** byte per byte; errore consultabile; i resti hanno nomi che portano la revisione | §9.3 | 3C.3.b.1 |
| **TP-14** | P-14 | destinazione resa non sostituibile; e destinazione tenuta da un altro handle | il `rename` fallito è un esito **nominato**; la generazione precedente resta valida e recuperabile. Il caso dell'handle esterno resta `NON_VERIFICATO` e il test documenta **quale** variante ha provato | §9.2, §9.6 | 3C.3.b.1 |
| **TP-15** | P-15 | il **ciclo completo**: pubblica, lascia i file nello stato di ogni fase (solo `*.novo.<rev>`; `*.novo` più `*.ger`; lucchetto rimasto; corrente corrotto), poi **riparti** da zero come farebbe un processo nuovo, prova una **seconda pubblicazione** e infine un **recupero scelto** | `candidatos()` elenca e **valida** ognuno con ruolo e revisione, e restituisce lo stato osservato; la seconda pubblicazione si **ferma** con `ResiduoPresente` o `ArquivoOcupado` invece di troncare un candidato; `recuperar` promuove quello scelto, conserva il corrotto come prova e non riscrive il numero; `liberar` è l'unico modo di togliere il lucchetto | PER-18, PER-23, PER-24 | 3C.3.b.1 |
| **TP-15b** | P-15 | **la destinazione cambia fra l'osservazione e il recupero**, in tre varianti: corrotto → valido pubblicato da un'altra operazione cooperante; assente → presente; valido → valido con revisione diversa. In tutte, `esperado` è lo stato **osservato prima** | `EstadoMudou` con atteso e trovato; **niente** viene toccato: il corrente non viene conservato né sostituito, il candidato non viene promosso. Poi, osservando lo stato nuovo e dichiarandolo, il **rollback intenzionale** riesce: il controllo non vieta il recupero, vieta che il bersaglio cambi senza conferma | PER-24 | 3C.3.b.1 |
| **TP-16** | P-16 | tre casi: due pubblicazioni dalla **stessa** base; una base presa da **un altro archivio** con la **stessa** `revisao_arquivo`; un candidato che **omette** un evento presente nel corrente, e uno che ne riduce la **molteplicità** da due a uno | il primo `BaseDesatualizada` o `ArquivoOcupado`; il secondo `BaseDesatualizada` sull'**impronta del contenuto**, perché il solo contatore coincide; il terzo e il quarto `HistoriaTruncada`. In nessun caso un evento del corrente scompare | PER-17, PER-19 | 3C.3.b.1 |
| **TP-17** | P-17 | documenti ed eventi permutati in ingresso; due eventi con lo stesso istante e bersaglio, **note diverse** e `impressao_evento` assente; **e** due record della **stessa** dichiarazione, uno con `impressao_evento` assente e uno con il digest **corretto** | la vista e gli errori sono **canonici** e identici; il **file** è **byte per byte lo stesso** nelle due permutazioni in **tutti** i casi, perché l'ultimo livello è il record serializzato; **entrambi** i record restano nel file, nessuno scartato e nessun digest riempito in silenzio. La differenza di `revisao_arquivo` fra pubblicazioni successive è attesa e non riguarda questo confronto | §8.4 | 3C.3.b.1 |
| **TP-18** | P-18 | stesso archivio riletto con un piano in cui **solo** un nome di conto cambia | impronta di **contenuto** identica; impronta di **presentazione** diversa; `APRESENTACAO_MUDADA`; decisioni conservate (D8.3) | PER-05 | 3C.3.b.1 |

### 12.1.1 Prove aggiunte dopo l'audit

| Test | Ingresso minimo | Atteso | Contratto |
|---|---|---|---|
| **TP-21** | un `Arquivo` con **ogni** campo popolato con un valore distinguibile, compresi i campi dei `Dado`, le rate, le quantità e i campi legacy rinviati: serializza, rileggi, confronta il modello | uguaglianza completa. Un campo che il DTO non conoscesse si perderebbe, e il test fallirebbe: è la protezione contro la deriva fra DTO e modello | PER-21 |
| **TP-22** | lo stesso `Arquivo` serializzato **due volte** dalla stessa revisione, con gli array interni permutati dove l'ordine è dichiarato, e con la coppia di record `impressao_evento` assente/presente del `TP-17` | testo **identico**; gli array a ordine memorizzato **non** vengono riordinati, perché riordinarli cambierebbe il contenuto conservato; i due record restano entrambi | §8.4 |

### 12.2 Due prove di metodo, non di contratto

| Test | Che cosa prova | Come, e che cosa **non** prova |
|---|---|---|
| **TP-19** | che la persistenza funziona **fra processi distinti** | il test scrive l'archivio, poi ri-esegue `std::env::current_exe()` con un filtro sul nome del test figlio e il percorso dell'archivio in una variabile d'ambiente. Il figlio rilegge, ricostruisce e confronta. Serve perché una variabile clonata nello stesso processo non prova niente. **Non** prova il comportamento dopo un riavvio della macchina, né su un altro filesystem. Il meccanismo di ri-esecuzione va approvato nel mandato implementativo: evita un binario nuovo e quindi una modifica di `Cargo.toml` |
| **TP-20** | che il **file fisico** è quello che si crede, senza usare il lettore del progetto | un controllo indipendente legge il JSON con uno strumento diverso — `ConvertFrom-Json` di PowerShell è già stato esercitato sugli esempi in §3.7 — e verifica: `formato` e `versao_schema` attesi, ogni valore monetario è una **stringa**, l'istante è in due campi e non in RFC3339, nessun BOM, terminatori `LF`. Dove la **precisione** conta deve confrontare il **testo grezzo**: §3.7 mostra che quel parser converte l'istante in `System.DateTime` e perde due decimali. **Non** prova la semantica: writer e reader del progetto potrebbero condividere un errore di significato che un controllo di forma non vede. Lo dice il test stesso nel suo commento |

### 12.3 Baseline

I **267** test esistenti (**[RIUSATO]**) restano la baseline e **non** vengono
riscritti per accomodare il formato: l'archivio si adatta ai tipi auditati, non
il contrario. Se un test preesistente dovesse cambiare, è il segnale che il
lotto ha superato il suo perimetro.

---

## 13. Primo lotto implementabile — 3C.3.b.1

### 13.1 Dentro

| Elemento | Dettaglio |
|---|---|
| `src/arquivo.rs` | schema di §5, carta dei campi di §6, codec e validatori di §7, append-only di §8.3, ordine fisico di §8.4 |
| `src/arquivo_io.rs` | protocollo di §9.1, generazioni e recupero di §9.4, concorrenza di §9.5 |
| `src/lib.rs` | due righe |
| `tests/persistencia_entre_processos.rs` | TP-19, con il nome dichiarato |
| `Cargo.toml` / `Cargo.lock` | `serde_json = "1.0.154"`, **da autorizzare**: versione già risolta in `Cargo.lock` e presente in cache (§3.2) |
| `README.md` | stato, API pubblica, limiti, conteggio effettivo dei test |
| test | TP-01…TP-22, `TP-15b` compresa, nei moduli nuovi tranne TP-19 e TP-20 |

### 13.2 Fuori, e dichiarato

| Fuori | Dove vive |
|---|---|
| export e import del workbook, round-trip XLSX | 3C.4 |
| adattatori di ingressi grezzi, `INCOMPLETO`, `NAO_RECONHECIDO` | 3C.4, D12 |
| import dello storico Excel, schemi 26/31 | 3C.4, D11 |
| CLI | 3C.5 |
| DRE | 3D |
| autenticazione dell'autore | nessun blocco pianificato: decisione a sé |
| potatura delle generazioni | D16 |
| numerazione storica dei documenti | D17 |
| migrazione verso un archivio a più file | D14(b), con `versao_schema` 2 |

### 13.3 Criterio di accettazione del lotto

1. `TP-19` passa: salva, chiudi, rileggi in un processo nuovo, ricostruisci la
   **stessa** vista a parità di piano;
2. ogni errore di §7 e §9 ha un test che lo nomina;
3. i 267 preesistenti restano verdi senza modifiche;
4. `fmt`, `clippy -D warnings`, `diff --check` senza rilievi;
5. nessun `f32`/`f64`, nessun orologio e nessun percorso implicito nei moduli
   nuovi; l'unico IO sta in `arquivo_io.rs`.

---

## 14. Limiti e rischi residui

| Limite | Stato |
|---|---|
| durabilità dopo perdita di alimentazione | **[NON_VERIFICATO]**: su Windows non si può sincronizzare la directory (§3.6 punto 9). Il contenuto scritto e sincronizzato non è lacerato, ma **quali nomi** sopravvivano — corrente, temporaneo, generazione — non è dimostrabile. Nemmeno la **recuperabilità** è quindi garantita in quel caso: la copia della generazione è sincronizzata come file, non come voce di directory |
| affermazioni assolute sulla sostituzione | **rimosse**: il successo di `rename` e `sync_all` dimostra l'atomicità della sostituzione e l'integrità del contenuto scritto, **non** che dopo qualunque guasto esista sempre un precedente valido |
| `rename` con destinazione tenuta da un **programma esterno** | **[NON_VERIFICATO]**: provato solo con handle aperti da Rust. Il protocollo tratta il fallimento come esito nominato |
| due scrittori che ignorano entrambi il lucchetto | finestra dichiarata fra controllo e commit: `std` non offre lock obbligatori (§9.5) |
| campo omesso contro `null` | **indistinguibili** per il lettore (§3.5): il writer omette, e la differenza non porta significato |
| `-0.00` | salvato come `"0.00"`: il segno dello zero negativo si perde, coerente con `DEC:0:0` |
| numero JSON al posto di una stringa monetaria | accettato da serde per difetto: per questo PER-07 lo **rifiuta** esplicitamente. Se il validatore venisse dimenticato, il difetto tornerebbe silenzioso |
| crescita del file | con D14(a) l'archivio cresce con lo storico; nessuna misura su volumi reali: **[NON_VERIFICATO]** |
| volumi di rete, FAT32, OneDrive | **[NON_VERIFICATO]**, uso sconsigliato |
| `serde_json` | **non** autorizzato: §3.2 |
| autenticazione | assente: l'autore resta una dichiarazione, e un digest rileva alterazioni ma non attesta chi le ha fatte |
| produzione contabile | **no**: questo blocco rende persistente una storia, non la promuove a contabilità |

---

## 15. Stato Git e destino dei file

Controllato prima della consegna:

```text
git status -sb                   ## main...origin/main
                                 ?? MANDATO_CLAUDE_FASE3C3B_PERSISTENZA_SPECIFICA_20261021.md
                                 ?? SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md
git diff --name-status           (vuoto)
git diff --cached --name-status  (vuoto)
```

| File | Origine / mandato | Stato | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3C3B_PERSISTENZA_SPECIFICA_20261021.md` | richiesta «procedi» di Luca, preparato da Codex | mandato, continuità nominata | **registrato** con lo stesso commit documentale |
| `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md` | esecuzione di questo mandato | audit tecnico **superato**, decisioni D14–D17 **approvate** (§17) | **registrata** con il commit documentale autorizzato il 2026-10-22; poi mandato implementativo del lotto `3C.3.b.1` |

**Nessun tracked modificato**, nessuno staged, nessuno stash: nessun file
preesistente del progetto è stato toccato, `README.md` compreso.

Il crate della sonda e i suoi file di prova vivono **fuori** dal repository,
nella cartella temporanea citata in §3.1, e le prove di filesystem in
`%TEMP%\sonda_persistencia_3c3b`: sono output di lavoro, non file di progetto.

**La specifica non implementa e non chiude la persistenza.** D7(a) è adottata e
D14–D17 sono **approvate** (§17), ma i contratti `PER-01…PER-24` restano
**PROPOSTI** e le prove `TP-01…TP-22` più `TP-15b` restano **PIANIFICATE** fino
al mandato implementativo e al suo audit.

---

## 16. Correzione dei riscontri degli audit del 2026-10-21 e 2026-10-22

Quattro riscontri, tutti accolti. Due riprodotti contro i **tipi reali** del
progetto con una sonda a dipendenza di percorso (§3.8), uno come modello del
criterio dichiarato, uno come prova su file sintetici. D7(a) resta adottata e
le parti confermate dall'audit non sono state riscritte.

### R3C3B-01 — i `Deserialize` esistenti non danno la lettura restrittiva promessa

La revisione 1 diceva «il contesto è esattamente la serializzazione dei tipi
esistenti, non serve un DTO» **e** «`deny_unknown_fields` su ogni struct,
lettura esatta dei decimali». Incompatibili. Riprodotto: campo sconosciuto
annidato **accettato**, numero JSON **accettato**, e soprattutto
`"0.00000000000000000000000000001"` **accettato come zero**.

| Intervento | Dove |
|---|---|
| lettore in **tre passaggi** con responsabilità separate — forma, rappresentazione, invarianti — e DTO dedicati per **tutto** l'albero, contesti compresi | nuova §7.3 |
| validazione esatta dei decimali: `from_str_exact` **più** confronto del testo riprodotto. Serve la coppia: il primo prende la perdita di cifre e `3.5e1`, il secondo prende `+35.00` | `PER-20`, §3.8 |
| chiarito che un albero `serde_json::Value` **peggiorerebbe** le cose: le chiavi duplicate sono rifiutate **perché** si deserializza in struct, e un passaggio per `Value` le collasserebbe prima di qualunque controllo | `PER-12`, §7.3 |
| coerenza fra «modelli non modificati» e `deny_unknown_fields`: i tipi auditati non vengono usati come lettori, e il vincolo vive nel DTO | `PER-11`, `PER-21` |
| costo dichiarato: il DTO replica la **forma** di circa otto tipi, non la logica; la deriva è protetta da `TP-21` | §7.3 |
| policy dell'assenza **per campo**, perché `Dado::valor` si scrive `null` e solo `motivo` viene omesso | nuova §7.4, `PER-10` |
| prove estese | `TP-04` con sei ingressi ostili e la ripetizione dentro `Dado`, rate e quantità; `TP-11` con duplicati e campi sconosciuti **annidati** |

### R3C3B-02 — l'append-only non era legato al file sostituito

Un `u64` non vincola la pubblicazione allo stato letto: due archivi diversi
possono condividere il contatore, e il controesempio del rapporto passa tutti i
controlli della revisione 1 mentre perde un evento.

| Intervento | Dove |
|---|---|
| il confronto dell'accrescimento avviene **dentro il lucchetto**, contro il **file riletto e validato**, non contro la base del chiamante | §9.1 passi 2–3 |
| `TokenBase` con revisione **e impronta del contenuto** letto, al posto del solo contatore | §10.2, §8.3 |
| `revisao_arquivo` assegnato dal pubblicatore sotto lucchetto con `checked_add`, e la **serializzazione spostata dopo** l'assegnazione, così il testo pubblicato porta il numero verificato. Esaurimento: `ContadorEsgotado` | §9.1 passi 4–5 |
| append-only sul **multinsieme** degli eventi, più l'elenco esplicito dei metadati protetti | `PER-17` |
| API allineate: `verificar_estrutura` senza piano per `ler` e `publicar`, `verificar_apresentacao` separata perché il suo esito **non è un errore**; nessun percorso restituisce un archivio non validato | §10.1, §10.2 con la tabella «dove avviene quale validazione» |
| difetto di schema trovato di conseguenza: `plano` nella radice avrebbe impedito di archiviare due esecuzioni con piani diversi senza violare `PER-17`. Spostato **dentro l'esecuzione** | §5.1, §5.2, §6.2, §10.1 |
| prove estese | `TP-16` con base di un altro archivio a parità di contatore, candidato che omette un evento e candidato che riduce la molteplicità |

### R3C3B-03 — mancava il ciclo di vita; il nome temporaneo fisso distruggeva un candidato

Tre buchi: la **prima** pubblicazione (nessun file da rileggere né da copiare,
e un `base: u64` obbligatorio), il nome `arquivo.json.novo` **troncato** da una
pubblicazione successiva, e l'assenza dell'operazione che **ripristina** il
candidato scelto.

| Intervento | Dove |
|---|---|
| ramo di **inizializzazione**: `base = None`, `revisao = 1` con fonte dichiarata, nessuna copia di un precedente inesistente, `DestinoJaExistente` se la destinazione c'è. E l'ammissione che dopo la prima pubblicazione **non esiste** una generazione | nuova §9.1.1, `PER-22` |
| temporaneo con `create_new` e nome che porta la revisione: non può troncare un candidato conservato, e un residuo è `ResiduoPresente`, esplicito | §9.1 passi 5–6 |
| tabella delle **transizioni** per ogni stato della destinazione: assente, valido, corrotto, temporaneo residuo, generazione presente, lucchetto rimasto, esito incerto | nuova §9.4.1, `PER-23` |
| `recuperar` come funzione a sé, che valida il candidato, conserva il corrente corrotto come **prova** e **non riscrive** il numero di revisione; `liberar` per lo sblocco esplicito | nuova §9.4.2, `PER-24`, §10.2 |
| esito dopo commit riuscito ma rilascio del lucchetto fallito: `PublicacaoComLucchettoResiduo`, pubblicazione **valida** | §9.3 |
| metadati del lucchetto come **parametro** `MetadadosLucchetto`, coerenti con ciò che la funzione riceve | §10.2, §9.4 |
| affermazioni assolute ridimensionate: la recuperabilità vale **dalla seconda pubblicazione**, e dopo una perdita di alimentazione nemmeno la generazione è garantita | §9.3, §14 |
| prove estese | `TP-01` su destinazione davvero nuova; `TP-15` con riavvio, seconda pubblicazione e recupero scelto |

### R3C3B-04 — l'ordine fisico non era totale

`impressao_evento` è ammesso `null`, quindi due eventi con lo stesso istante e
bersaglio ma **note diverse** avevano chiavi fisiche identiche: riprodotto, con
l'ordine del file che seguiva l'ingresso.

| Intervento | Dove |
|---|---|
| ultimo livello dell'ordine: il **testo esatto** dell'evento, sempre calcolabile e già iniettivo (F3C-46), al posto di un digest opzionale | §8.4 |
| aggiunto `operacao` fra i livelli, e specificato l'ordine di `execucao.documentos` e degli array interni, distinguendo quelli a **ordine memorizzato** — che sono dati del documento e non vanno riordinati | §8.4 |
| distinzione fra determinismo della serializzazione **della stessa revisione** e normale differenza di `revisao_arquivo` fra pubblicazioni successive | `TP-17` |
| prove estese | `TP-17` con la coppia di note e digest assente nelle due permutazioni; `TP-22` nuovo sul determinismo del testo |

### Rettifiche documentali

| Rilievo | Correzione |
|---|---|
| §5.2 diceva `PARCIAL` ma l'esempio ha `ERRO` | prosa e `TP-02` allineati a `ERRO`, che è l'esito corretto per un documento senza righe con `vNF` 35.00 |
| `TP-11` diceva «sei file» ed elencava sette categorie | la copertura segue le **condizioni**, non una quota; aggiunto il riferimento rotto |
| `PER-10` prometteva che il writer omette tutti gli assenti | policy **per campo** in §7.4: `Dado::valor` è `null` esplicito |
| perdita di alimentazione e recuperabilità | dichiarate insieme in §9.3 e §14: il successo di `rename` e `sync_all` non dimostra la sopravvivenza dei nomi |

### Stato dei riscontri

| Riscontro | Stato |
|---|---|
| R3C3B-01 lettura restrittiva | **CORRETTO nella specifica**: §7.3, §7.4, PER-11/12/15/20/21 |
| R3C3B-02 base effettiva dell'append-only | **CORRETTO nella specifica**: §8.3, §9.1, §10, PER-17 |
| R3C3B-03 ciclo di vita e recupero | **CORRETTO nella specifica**: §9.1.1, §9.4.1, §9.4.2, PER-22/23/24 |
| R3C3B-04 ordine fisico totale | **CORRETTO nella specifica**: §8.4 |

«Corretto nella specifica» significa esattamente questo: **nessuna riga di
codice esiste**, i contratti `PER-01…PER-24` restano **PROPOSTI** e tutte le
prove `TP-01…TP-22` restano **PIANIFICATE**. La chiusura dei riscontri spetta
all'audit mirato successivo, non a questo documento.

### Residui dell'audit mirato del 2026-10-22

Due residui, entrambi accolti e riprodotti come modelli dei criteri che questa
specifica dichiara (**[ESEGUITO]** §3.9). R3C3B-01 e R3C3B-02 restano chiusi:
non sono stati riaperti.

#### R3C3B-03, residuo — il recupero non era vincolato allo stato osservato

La prosa mostrava `base_attesa`, la firma la ometteva, e i cinque passi non
confrontavano niente. Il lucchetto garantisce la mutua esclusione **al
commit**, non copre il tempo fra l'osservazione del chiamante e l'acquisizione
del lucchetto: un'altra operazione **cooperante** può pubblicare in quella
finestra, e il recupero sostituisce uno stato diverso da quello su cui era
stato chiesto.

| Intervento | Dove |
|---|---|
| nuovo **passo 2**: osserva lo stato effettivo e confrontalo con `esperado`; diverso ⇒ `EstadoMudou` e **niente** viene toccato, né il corrente conservato né il candidato promosso | §9.4.2 |
| `EstadoObservado` copre anche **destinazione assente** e **corrente illeggibile**, con l'impronta dei **byte**: un `TokenBase` non esiste quando il file non si deserializza | §9.4.2, §10.2 |
| firma allineata alla prosa: `recuperar(destino, candidato, esperado, lucchetto)`; `observar()` esposta; `candidatos()` restituisce anche lo stato, così il chiamante lo ripassa senza ricostruirlo | §10.2 |
| dichiarato che il controllo **non vieta** un rollback intenzionale: chi vuole tornare indietro osserva lo stato nuovo, lo dichiara e procede | §9.4.2 |
| prove estese | `TP-15b` nuovo, con tre transizioni — corrotto → valido, assente → presente, valido → valido con altra revisione — e il rollback intenzionale che riesce dopo una nuova osservazione |

Nota sulla gravità, perché va detta con precisione: i byte precedenti restano
come prova, quindi **non** è una perdita fisica irreversibile. È un rollback
**non confermato**, e il difetto sta nel fatto che il bersaglio della decisione
può cambiare fra la scelta e l'esecuzione.

#### R3C3B-04, residuo — l'ordine era totale sulla dichiarazione, non sul record

`impressao_evento` è un campo del **record archivistico**, non di
`EventoRevisao`, quindi non entra in `texto_exato_evento`. Due record della
stessa dichiarazione, uno con digest assente e uno con il digest **corretto**,
coincidevano su tutti i livelli e producevano due file diversi secondo
l'ingresso.

| Intervento | Dove |
|---|---|
| ultimo livello dell'ordine: il **record archivistico interamente serializzato**, non più il solo testo esatto dell'evento | §8.4 |
| chiarito che non c'è circolarità: ogni record si serializza in modo **indipendente**, poi si ordina per la chiave, poi si uniscono i testi | §8.4 |
| dichiarata la proprietà che conta per il futuro: qualunque campo aggiunto a `EventoArquivado` entra **automaticamente** nello spareggio, perché entra nel testo del record. È F3C-46 applicata al record | §8.4 |
| tabella dei **due livelli distinti**: identità e ordine della dichiarazione umana, invariati dalla 3C.3.a; byte del record archivistico, che comprendono i metadati del solo archivio | §8.4 |
| ribadito che il pareggio **non** si risolve scartando un record, normalizzando gli originali o riempiendo un digest assente | §8.4, §7.4, `PER-17` |
| prove estese | `TP-17` e `TP-22` con la coppia assente/digest corretto nelle due permutazioni, ed entrambe le occorrenze conservate |

#### Allineamenti documentali dello stesso giro

| Rilievo | Correzione |
|---|---|
| §6.3 e il commento di `Snapshot::contexto` richiamavano i `Serialize`/`Deserialize` esistenti | precisato: quella è la **forma** del modello in memoria; il lettore è il DTO restrittivo di §7.3 |
| §9.5 parlava ancora del solo contatore | richiama il `TokenBase` completo, revisione **più** impronta del contenuto |
| numeri di passo rimasti alla numerazione vecchia | aggiornati in §9.4.2, §9.5 e `TP-13`, che ora cita i passi 5 e 6 |
| §13.1 elencava `TP-01…TP-20` | elenca `TP-01…TP-22`, `TP-15b` compresa, senza farne una quota |

#### Stato dei riscontri dopo questa revisione

| Riscontro | Stato |
|---|---|
| R3C3B-01 lettura restrittiva ed esatta | **CHIUSO** dall'audit del 2026-10-22, non riaperto |
| R3C3B-02 pubblicazione dalla base effettiva | **CHIUSO** dall'audit del 2026-10-22, non riaperto |
| R3C3B-03 ciclo di vita | corretto nella revisione 2; **residuo del recupero corretto** in questa revisione |
| R3C3B-04 ordine fisico | corretto nella revisione 2; **residuo del record corretto** in questa revisione |

Vale come prima: «corretto nella specifica» significa che **nessuna riga di
codice esiste**. I contratti `PER-01…PER-24` restano **PROPOSTI** e le prove
`TP-01…TP-22` restano **PIANIFICATE**. La chiusura dei due residui spetta
all'audit mirato successivo.

---

## 17. Decisioni approvate il 2026-10-22

Luca ha approvato **tutte e quattro** le decisioni nell'opzione **(a)**, dopo
il superamento dell'audit tecnico della specifica.

| ID | Decisione approvata | Dove è definita | Contratti che la applicheranno |
|---|---|---|---|
| **D14(a)** | **un solo file JSON** per archivio: un punto di pubblicazione, sostituito atomicamente con un `rename` | §5.1, §9.1 | PER-01, PER-19 |
| **D15(a)** | **lucchetto cooperativo più base completa verificata**: il `TokenBase` porta revisione e impronta del contenuto, e l'accrescimento si confronta con il file riletto dentro il lucchetto | §8.3, §9.1, §9.5 | PER-17, PER-19 |
| **D16(a)** | **nessuna potatura automatica** delle generazioni: la libreria non cancella mai una copia | §9.4 | PER-18 |
| **D17(a)** | numerazione storica dei documenti **rinviata**: `VersaoDocumento::sequencia` resta `NAO_DISPONIVEL`, e `revisao_arquivo` resta una cosa diversa | §5.1, e §4.9 della specifica 3C.3 | **nessun** contratto introduce una numerazione: è questo il contenuto della decisione. `D8.5` resta invariata e `revisao.rs` non si tocca |

### Che cosa questa approvazione è, e che cosa non è

| È | Non è |
|---|---|
| la chiusura delle quattro scelte di disegno dell'archivio | l'approvazione di codice: `arquivo.rs` e `arquivo_io.rs` non esistono |
| la base per scrivere il mandato implementativo del primo lotto | un collaudo: le ventitré prove `TP-01…TP-22` più `TP-15b` sono **pianificate**, nessuna eseguita |
| una decisione sul **formato** e sul **protocollo** | un'autorizzazione ad aggiungere `serde_json` a `Cargo.toml`: va nel mandato implementativo, e la disponibilità offline non è un'autorizzazione |

### I costi che le decisioni portano con sé, dichiarati

| Decisione | Costo accettato |
|---|---|
| D14(a) | l'archivio **cresce** con tutto lo storico in un unico file; nessuna misura su volumi reali (**[NON_VERIFICATO]**). La migrazione verso più file resta possibile con `versao_schema` 2 |
| D15(a) | protegge gli scrittori **cooperanti**; due processi che ignorano entrambi il lucchetto restano una finestra dichiarata e non eliminabile con `std` (§9.5) |
| D16(a) | la cartella dell'archivio **cresce senza limite**: nessuna copia viene mai cancellata dalla libreria. La pulizia resta una decisione umana, da prendere quando servirà |
| D17(a) | chi volesse leggere "versione 3 di questo documento" non lo troverà: esiste l'impronta del contenuto, non un numero. Introdurlo sarà una decisione a sé, con una regola scritta |

### Decisioni che restano fuori

D11 e D12 restano aperte e appartengono a 3C.4. L'autenticazione dell'autore
**non** è stata chiesta né decisa: resta un blocco a sé, e finché non esiste
`autor` è una dichiarazione, non un'identità verificata. Nessuna promozione in
produzione contabile.

### Prossimo passo, non autorizzato da questo documento

Un **mandato implementativo** del primo lotto `3C.3.b.1` nel perimetro di §13,
che dovrà nominare: i file ammessi, l'autorizzazione esplicita per
`serde_json = "1.0.154"`, i contratti `PER-01…PER-24` e le prove
`TP-01…TP-22` più `TP-15b`. L'implementazione, i suoi commit e il suo push
restano da autorizzare separatamente.

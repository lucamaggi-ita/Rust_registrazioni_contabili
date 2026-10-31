# Proposta di mandato — lotto 3C.5.a: uso concreto dalla CLI

| Voce | Valore |
|---|---|
| Data | 2026-10-31 |
| Stato | **AUDIT TECNICO SUPERATO il 2026-10-31** nel perimetro approvato: `R3C5A-I01` e il suo residuo chiusi, nessuna modifica tecnica ulteriore richiesta (§28). **Registrazione Git e pubblicazione da autorizzare**: nessun commit, nessun push |
| Classificazione | `MIRATO`: compone funzioni **esistenti** dietro due comandi. Nessuna regola contabile nuova, nessun cambio di semantica dei dati, writer e nucleo non toccati |
| Obiettivo operativo | ripartire da una **cartella** di NF-e, anche già lette, conservarle nell'archivio autorevole e produrre l'Excel diagnostico |
| Fuori perimetro | DRE (3D), import dello storico Excel (D11(a)), nuove regole contabili o di classificazione, rientro dal file all'archivio, potatura dell'archivio (D16(a)) |

| Revisione | Che cosa cambia |
|---|---|
| 1 | prima stesura: una fattura per volta, `--xml` obbligatorio |
| 2 | la cartella diventa l'unità; `D29` corretta; `R3C5A-01` chiuso |
| 7 (2026-10-31) | **audit tecnico superato**: esito finale, provenienze delle prove tenute distinte, e che cosa l'esito non dichiara (§28) |
| 6 (2026-10-31) | residuo dell'helper archivio chiuso: `.unwrap_or(0)` e l'errore ingoiato del lucchetto sostituiti da un solo punto che distingue presente, assente e `NON_DISPONIVEL`; tre test di unità e `tx65` (§27) |
| 5 (2026-10-31) | audit del percorso operativo **superato**; `R3C5A-I01` corretto con la regressione `tx64`; due correzioni documentali recepite, una delle quali una mia misura sbagliata su `Display` (§26) |
| 4 (2026-10-31) | approvazione di Luca registrata (§22), lotto implementato, verificato e consegnato per l'audit (§23-§25) |
| 3 | audit del 2026-10-31. `R3C5A-02` chiuso togliendo `--manifesto` dalla CLI; `R3C5A-03` chiuso con un **modo** nuovo del lettore, senza toccare le attese della fixture; `R3C5A-04` chiuso con le tabelle complete degli esiti per comando; perimetro ridotto (`novos`, `--adiar-corrigidas`, `--saltar-nao-processaveis` **rinviati**); «rigenerazione dalla cartella attuale» al posto di «aggiornamento incrementale»; esempi in PowerShell; quattro allineamenti recepiti; due decisioni nuove che **non** prendo io, `D33` e `D34` |

## 1. Esiti dei riscontri dell'audit

| Riscontro | Esito | Dove |
|---|---|---|
| `R3C5A-01` lotto immutabile | **chiuso** nella revisione 2 | §4, §7 |
| `R3C5A-02` il manifesto è una seconda scrittura non protetta | **chiuso**: `--manifesto` esce dalla CLI | §6 |
| `R3C5A-03` il lettore è specifico della fixture | **chiuso**: modo nuovo, attese della fixture intatte | §10 |
| `R3C5A-04` esiti di IO non classificati | **chiuso**: una tabella completa per comando | §8 |
| rigenerare ≠ accumulare | **recepito**, con il controesempio di `C` scritto nel documento e una domanda aperta per Luca (`D34`) | §5 |
| `TECNICO_OK` non è universale | **recepito** | §9 |
| impronta diversa ≠ XML corretto | **recepito**, e ne nasce `D33` | §7 |
| `D27` non ha una sede nello schema | **recepito**: etichetta solo nell'uscita della CLI | §12 |
| esempi in PowerShell e creazione delle directory | **recepito** | §3 |

Nessuna di queste correzioni nasce da un difetto dei moduli già auditati.

## 2. I due comandi

| Comando | Scrive | Legge | Unità |
|---|---|---|---|
| `arquivar-nfe` | l'archivio: **una** revisione, **una** esecuzione, un solo file | XML, piano, regole | `--pasta` (forma principale), oppure `--xml` per un file singolo |
| `exportar-diagnostico` | il `.xlsx`: **un** solo file | l'archivio, in sola lettura | un'esecuzione |

Ogni comando scrive **un solo** file. È la ragione per cui sono due, e ora è
vera anche nel secondo: il manifesto non è più un suo output (§6).

I protocolli **non creano directory** — `create_dir_all` compare solo nei
moduli di test di `arquivo_io` e `export_io` — quindi la cartella di
destinazione deve esistere prima, e l'esempio lo mostra.

## 3. Esempio in PowerShell: primo avvio e rilancio

Una riga per comando: in PowerShell il backslash finale **non** continua la
riga.

```powershell
# La directory non la creano i protocolli: la si prepara.
New-Item -ItemType Directory -Force dados | Out-Null

# --- PRIMO AVVIO: cartella di maggio, archivio inesistente ---
cargo run -- arquivar-nfe --pasta dados/entrada/2026-05 --arquivo dados/arquivo.json --inicializar --lote fonte-maio-r1 --lote-data 2026-05-31 --lote-descricao "pasta 2026-05, primeira leitura" --operador "Luca Maggi" --motivo "primeira importacao da pasta de maio"
# STATUS: ARQUIVADO | execucao fonte-maio-r1 | revisao 1
#         3 XML lidos, 3 arquivados, 0 ja' arquivados identicos
#         instante: RELOGIO_DO_SISTEMA 2026-10-31T09:12:44-03:00

cargo run -- exportar-diagnostico --arquivo dados/arquivo.json --lote fonte-maio-r1 --saida dados/diagnostico_maio.xlsx --etiqueta fonte-maio-r1 --operador "Luca Maggi" --motivo "diagnostico de maio"

# --- RILANCIO: e' arrivata la quarta fattura nella stessa cartella ---
# Si rilegge tutta la cartella, e il lotto e' NUOVO: un'esecuzione archiviata
# e' immutabile.
cargo run -- arquivar-nfe --pasta dados/entrada/2026-05 --arquivo dados/arquivo.json --lote fonte-maio-r2 --lote-data 2026-05-31 --lote-descricao "pasta 2026-05, relancamento com a 4a NF-e" --operador "Luca Maggi" --motivo "relancamento da pasta de maio"
# STATUS: ARQUIVADO | execucao fonte-maio-r2 | revisao 2
#         4 XML lidos, 4 arquivados, 3 ja' arquivados identicos em fonte-maio-r1
#         (reuso entre execucoes, nao duplicacao no resultado)

# Rigenerazione dalla cartella attuale, nello stesso percorso: il precedente
# viene conservato come .anterior.<etiqueta>.
cargo run -- exportar-diagnostico --arquivo dados/arquivo.json --lote fonte-maio-r2 --saida dados/diagnostico_maio.xlsx --se-existir substituir --etiqueta fonte-maio-r2 --operador "Luca Maggi" --motivo "diagnostico de maio, relancamento"

# Oppure in una directory nuova per esecuzione, affiancabile alla precedente.
New-Item -ItemType Directory -Force dados/runs/fonte-maio-r2 | Out-Null
cargo run -- exportar-diagnostico --arquivo dados/arquivo.json --lote fonte-maio-r2 --saida dados/runs/fonte-maio-r2/diagnostico.xlsx --etiqueta fonte-maio-r2 --operador "Luca Maggi" --motivo "run de auditoria"
```

## 4. Argomenti

### 4.1 `arquivar-nfe`

| Argomento | Obbligo | Significato |
|---|---|---|
| `--pasta` oppure `--xml` | sì, uno dei due | cartella (ricerca **non** ricorsiva, come il Python: i file direttamente dentro, ordinati per nome) oppure un singolo file |
| `--arquivo` | sì | percorso dell'archivio; si scompone in `Destino { diretorio, nome }` |
| `--inicializar` | solo la prima volta | `publicar(base = None)`. Senza il flag l'archivio deve esistere; con il flag non deve esistere. Nessuna creazione implicita |
| `--lote`, `--lote-data`, `--lote-descricao` | sì | `IdentidadeLote`, **dichiarata**. Un lotto già presente viene rifiutato: un'esecuzione è immutabile |
| `--operador`, `--motivo` | sì | metadati del lucchetto. **Mai** l'utente del sistema operativo |
| `--instante` | no | RFC 3339 con offset. Omesso: orologio della macchina, etichettato `RELOGIO_DO_SISTEMA` **nell'uscita della CLI** (§12) |
| `--plano-contas`, `--regras-produtos` | no | i percorsi di oggi, con gli stessi valori predefiniti |
| `--recusar-nao-ok` | no | rifiuta **prima** di pubblicare se un documento ha stato tecnico `PARCIAL` o `ERRO` (D30) |

### 4.2 `exportar-diagnostico`

| Argomento | Obbligo | Significato |
|---|---|---|
| `--arquivo` | sì | l'archivio, **sola lettura**, protetto come `FonteProtegida` |
| `--lote` | sì | **una** esecuzione. Più lotti insieme sono rifiutati |
| `--saida` | sì | il `.xlsx` |
| `--se-existir` | no | `recusar` (predefinito) o `substituir`, che conserva il precedente |
| `--etiqueta` | no | etichetta del temporaneo; predefinita l'id del lotto. Non letta dall'orologio |
| `--operador`, `--motivo`, `--instante` | sì, sì, no | lucchetto, e `Proveniencia::gerado_em` |
| `--plano-contas` | no | il piano **usato ora**, che l'export confronta con quello archiviato |

Rinviati, perché non servono al percorso richiesto («100 fatture, aggiungo la
101ª, rilancio tutta la cartella»): `--escopo novos`, `--adiar-corrigidas`,
`--saltar-nao-processaveis`, `--manifesto`.

## 5. Rigenerazione dalla cartella attuale, non accumulo

La revisione 2 chiamava «aggiornamento incrementale» la scrittura sullo stesso
percorso e la diceva equivalente al workbook cumulativo del Python. **Non lo
è**, e il controesempio dell'audit lo mostra:

1. primo giro con A, B, C → il risultato contiene A, B, C;
2. `C` viene spostata fuori dalla cartella, arriva `D`, si rilancia su A, B, D;
3. il writer cumulativo Python conserva la riga di `C` e aggiunge `D`: A, B, C, D;
4. qui il risultato della nuova esecuzione è **A, B, D**. `C` resta
   nell'archivio e nell'Excel precedente conservato, ma non nel risultato
   corrente.

Non è una perdita — l'archivio è append-only e `C` resta un fatto archiviato —
e non è un difetto dell'export: è una **differenza funzionale del risultato
corrente**. Quindi entrambe le forme si chiamano **rigenerazione dalla cartella
attuale**: scrivere nello stesso percorso con il precedente conservato, oppure
in una directory nuova. Il nome del file non cambia la semantica.

| | stesso percorso | directory nuova per esecuzione |
|---|---|---|
| archivio | una esecuzione nuova | identico |
| Excel | sostituito, precedente conservato come `.anterior.<etiqueta>` | affiancato |
| copertura | la cartella **al momento del rilancio** | idem |

**`D34`, da decidere e che non decido io:** se una fattura rimossa dalla
cartella deve restare nel risultato corrente. Se sì, serve una selezione
dichiarata — non una somma di esecuzioni e nessuna modifica dello storico — e
diventa un lotto a sé.

## 6. `R3C5A-02`: il manifesto esce dalla CLI

`--manifesto` sarebbe stata una seconda scrittura non coperta da
`FonteProtegida`, che protegge solo il percorso passato a `export_io::escrever`:
un `fs::write` del JSON avrebbe aggirato l'esclusione, avrebbe potuto puntare
sull'archivio o sulla stessa `--saida`, e un suo fallimento dopo un XLSX già
pubblicato non avrebbe avuto un esito dichiarabile.

Il manifesto resta dove serve, nel **percorso di verifica**: lo produce
`tests/cli_percurso_nfe.rs`, dentro l'arena del test, rileggendo con l'API
pubblica l'archivio che la CLI ha scritto e ricostruendo il modello. La
rilettura indipendente resta una prova del lotto, non un secondo output del
comando dell'utente. Nessuna promessa di transazione su due file, perché non
ci sono due file.

## 7. Chiavi ripetute, e che cosa l'impronta dimostra davvero

| Caso | Esito | Perché |
|---|---|---|
| due XML con la stessa chiave **nella stessa esecuzione** | rifiuto, niente pubblicato, i due file nominati | un risultato non può contare una fattura due volte. Lo rileva la barriera esistente, `DocumentoDuplicadoNoLote` |
| chiave già archiviata, **contenuto allocato identico** | **riuso dichiarato**: `JA_ARQUIVADO_EM <lote>`, contato nel riassunto, archiviato nella nuova esecuzione | il modello ammette la stessa chiave in due lotti, e il rilancio sulla cartella è il flusso normale |
| chiave già archiviata, **contenuto allocato diverso** | rifiuto, niente pubblicato, uscita 4 | non è lo stesso fatto archiviato, e il versionamento è di un blocco successivo |

Il messaggio del terzo caso dirà **«stessa chiave, contenuto allocato
diverso»**, senza attribuire una correzione al file XML: `impressao_conteudo` è
lo SHA-256 di un testo canonico che include `conta_id`, `conta_nome`,
`grupo_conta` e `alertas`, cioè gli **effetti delle regole**. Una differenza
può venire dalla fonte **o** dalla configurazione.

**`D33`, decisione nuova che non prendo io.** Qui c'è una conseguenza
operativa da decidere prima di implementare. L'archivio registra l'identità del
**piano** (`IdentidadePlano`: versione, SHA, percorso) ma **non** quella delle
regole di classificazione, che pure entrano nell'impronta. Quindi, dopo un
aggiornamento delle regole, ogni documento già archiviato risulta «contenuto
allocato diverso» e, con la regola sopra, **la cartella non si può più
rilanciare**. Due strade, da scegliere:

| Strada | Costo |
|---|---|
| (a) registrare nell'archivio anche l'identità delle regole, e consentire la differenza quando la configurazione è cambiata, dichiarandola | cambia lo schema dell'archivio: è `STRUTTURALE`, fuori da questo perimetro |
| (b) tenere il rifiuto e dare all'operatore un modo dichiarato di accettare la differenza documento per documento | un flag nuovo, che l'audit chiede di non moltiplicare, ma nessun cambio di schema |

Finché `D33` non è decisa, il lotto resta implementabile **a condizione che le
regole non cambino fra due rilanci**: è un limite da scrivere nel foglio dei
limiti e nell'aiuto del comando, non da scoprire al primo aggiornamento delle
regole.

## 8. Esiti per comando

Contratto `CLI-01`: esiti distinti e determinati, e il riassunto dice sempre
**che cosa è stato pubblicato**. «Niente scritto» non cancella dal rapporto
temporanei o generazioni che i protocolli possono avere conservato.

### 8.1 `arquivar-nfe`

| Codice | Esito | Fatto attestabile |
|---|---|---|
| 0 | esecuzione pubblicata, tutti i documenti `OK` | **archivio corrente pubblicato**, revisione N. Un `lucchetto_residuo` è dichiarato: la pubblicazione **è valida** |
| 1 | configurazione bocciata, argomenti incoerenti, cartella inesistente o senza XML | archivio corrente **non** pubblicato |
| 2 | un XML non elaborabile (`ErroNfe`, `ErroAlocacao`) | archivio corrente **non** pubblicato: un lotto parziale sarebbe un successo parziale silenzioso |
| 3 | due XML con la stessa chiave nella stessa esecuzione | archivio corrente **non** pubblicato |
| 4 | chiave già archiviata con contenuto allocato diverso | archivio corrente **non** pubblicato |
| 5 | pubblicata **con riserva**: almeno un documento con stato tecnico `PARCIAL` o `ERRO` (D30) | **archivio corrente pubblicato**; i documenti non `OK` sono elencati e restano fuori dall'estratto approvato |
| 6 | rifiuto di IO **prima** del commit: `ExecucaoConflitante` (lotto occupato), `DestinoAusente` senza `--inicializar`, `DestinoJaExistente` con `--inicializar`, `ArquivoCorrenteInvalido`, `BaseDesatualizada`, `EstadoMudou`, `ArquivoOcupado`, `ResiduoPresente`, `GeracaoJaPresente`, `ContadorEsgotado`, `Io` | archivio corrente **non** pubblicato; la variante è nominata, e temporanei o generazioni rimasti vengono elencati con il loro percorso |
| 7 | `--recusar-nao-ok` con almeno un documento non `OK` | archivio corrente **non** pubblicato: rifiuto prima della pubblicazione, distinto dal codice 5 |

### 8.2 `exportar-diagnostico`

| Codice | Esito | Fatto attestabile |
|---|---|---|
| 0 | pubblicato | **XLSX finale pubblicato**, con il suo SHA. `lucchetto_residuo` dichiarato se presente |
| 1 | configurazione o argomenti; lotto inesistente nell'archivio; più di un lotto | XLSX finale **non** pubblicato |
| 3 | destinazione già esistente con `--se-existir recusar` (`DestinoJaExistente`) | XLSX finale **non** pubblicato, e il file che c'era resta intatto |
| 4 | destinazione che coincide con l'archivio o con la sua famiglia di nomi (`DestinoCoincideComFonte`) | XLSX finale **non** pubblicato, archivio intatto |
| 6 | altro rifiuto di IO prima del commit: `ExportOcupado`, `ResiduoPresente`, `AnteriorJaPresente`, `PacoteVazio`, `Io`, archivio illeggibile o invalido | XLSX finale **non** pubblicato; residui nominati |

## 9. Esito tecnico e approvazione umana

Contratto `CLI-02`, nella formulazione corretta: **l'importazione non crea
nessuna approvazione umana**. Non «lo stato è `TECNICO_OK`»: senza eventi lo
stato della vista dipende dall'esito tecnico — `revisao.rs:714-716` dà
`TECNICO_OK` per `Ok` e `TECNICO_REVISAR` per `Revisar`.

| Nozione | Chi la produce qui |
|---|---|
| esito **tecnico** della 3B | `alocacao` e la classificazione: `OK` o `REVISAR` |
| **decisione umana** | **nessuno**: i comandi non costruiscono nemmeno un `EventoRevisao` |
| **stato della vista** | derivato: `TECNICO_OK` o `TECNICO_REVISAR`, mai `APROVADO_POR_PESSOA` |

Le prove: `aplicar_revisao` riceve gli eventi **già** nell'archivio, zero alla
prima importazione; `tx50` verifica il caso pulito e `tx53` il caso con un
documento da revisionare, perché il caso pulito da solo non proverebbe la
regola.

## 10. `R3C5A-03`: un modo nuovo del lettore, la fixture intatta

`script/releitura_export.py` ha oggi `--modo tx38` e `--modo export`, e le
attese esplicite di `export` sono cablate sulla fixture: `lote-export-3c41`,
importo `10.00`, due eventi, Alice e Bob, `APROVADO_POR_PESSOA`, più le
divergenze note di quelle celle decisionali. Su un workbook della CLI, senza
approvazioni, quelle attese **non possono** passare: generare un manifesto
diverso non basta, perché le attese esplicite girano sul workbook comunque.

Correzione:

| Punto | Scelta |
|---|---|
| modo nuovo | `--modo percurso-cli`, con il **suo** insieme di attese esplicite e di indicatori, pertinenti alla NF-e reale e all'**assenza** di approvazioni |
| fixture storica | `--modo export` **invariato**: stesse attese, stesse divergenze note. Nessuna deroga globale e nessuna attesa disabilitata per ottenere uscita 0 |
| forma | le costanti diventano insiemi **per modo**; il confronto campo per campo e la verifica effettiva del pacchetto restano |
| divergenze note | nel modo nuovo solo quelle **realmente esercitate** da quell'ingresso |
| impronta del piano | se `D28` viene approvata l'attesa «digest assente» non vale per il modo nuovo: la nuova attesa è il digest **presente e uguale** a quello del piano usato |
| controprova | `script/controprova_rilettura.py` tiene i dodici casi di oggi sulla fixture, **validi e invariati**, e aggiunge per il modo nuovo almeno una mutazione di importo e una di stato: uscita 1 con la cella nominata |

## 11. Contratti nuovi

| Contratto | Regola | Prova |
|---|---|---|
| `CLI-01` | esiti distinti per comando, e il riassunto dichiara che cosa è stato pubblicato | `tx50`–`tx62` |
| `CLI-02` | nessuna approvazione umana creata; lo stato della vista segue l'esito tecnico | `tx50`, `tx53` |
| `CLI-03` | duplicato **nell'esecuzione**: rifiuto che nomina i due file | `tx56` |
| `CLI-04` | chiave già archiviata con contenuto identico: **riuso dichiarato** | `tx51` |
| `CLI-05` | chiave già archiviata con contenuto allocato diverso: rifiuto, messaggio fedele a ciò che è stato confrontato | `tx57` |
| `CLI-06` | nessuna creazione implicita: `--inicializar` assente su archivio mancante e presente su archivio esistente sono **due** rifiuti distinti | `tx54`, `tx55` |
| `CLI-07` | lotto occupato: rifiuto che spiega l'immutabilità dell'esecuzione | `tx52` |
| `CLI-08` | il risultato copre **una** esecuzione, nominata in `proveniencia`; più lotti sono rifiutati | `tx51`, `tx60` |
| `CLI-09` | identità del lotto, operatore e motivo dichiarati; istante etichettato per fonte nell'uscita | `tx50` |

## 12. `D27`: dove vive l'etichetta dell'istante

`Proveniencia` (`export.rs:164-179`) ha `gerado_em` e **nessun** campo per la
fonte; i metadati del lucchetto hanno l'istante ma non una fonte dedicata.
Quindi l'etichetta `RELOGIO_DO_SISTEMA` o `DECLARADO` vive **solo
nell'uscita della CLI**, e il documento non promette altro. Nessun testo libero
dentro un campo data. Portarla nel workbook o nell'archivio sarebbe una
modifica di schema, da nominare come tale: non è composizione, e non è
proposta qui.

## 13. Controlli esistenti riusati

| Controllo | Da | Che cosa impedisce |
|---|---|---|
| `carregar_validado` | `main.rs` | una configurazione che `validar-config` boccia |
| `nfe::ler_nfe` | 3C.2 | più NF-e in un documento (C17), XML malformato |
| `alocacao::alocar_nfe` | 3B | riconciliazione, dominio monetario, identità insufficiente |
| `construir_pre_livro`, via `aplicar_revisao` | 3C.2 | **la barriera**: chiave vuota, duplicato nell'esecuzione, metadati di lotto divergenti, conto fuori piano, scala monetaria, capacità numerica |
| `arquivo::verificar_estrutura`, `verificar_acrescimo` | 3C.3.b | archivio incoerente, lotto ripetuto, perdita di record, base non aggiornata |
| `arquivo_io::publicar` | 3C.3.b | lucchetto, un solo commit, generazione conservata |
| `impressao::impressao_conteudo` | 3C.3.a | riconoscere «già archiviata, identica» in modo esatto |
| `arquivo::verificar_apresentacao` | 3C.4 | una presentazione cambiata che passerebbe muta |
| `export::modelo`, `xlsx::pacote` | 3C.4.1 | modello e formato, 13 fogli compresi `limites` e `proveniencia` |
| `export_io::escrever` | 3C.4.1 | `FonteProtegida`, `SeExistir`, lucchetto, precedente conservato |
| `script/releitura_export.py` | 3C.4.1 | il confronto cella per cella con un lettore che non è il writer |

## 14. Test

`tests/cli_percurso_nfe.rs`, nella forma di `tests/cli_importar_nfe.rs`. Il
numero viene dai casi, non da una quota.

| Test | Che cosa prova |
|---|---|
| `tx50_primeiro_arranque` | cartella con tre NF-e: una esecuzione, tre snapshot, revisione 1, zero eventi, Excel con tre documenti, nessun `APROVADO` |
| `tx51_relancamento_da_pasta` | rilancio con lotto nuovo: uscita 0, **accrescimento** dell'archivio alla revisione 2, i tre documenti riportati come già archiviati identici, e l'Excel della nuova esecuzione conta **tre** documenti, non sei |
| `tx52_lote_ocupado` | lo stesso `--lote` due volte: uscita 6, messaggio sull'immutabilità, archivio **byte-identico** |
| `tx53_documento_a_revisar` | una NF-e che la classificazione manda a `REVISAR`: stato della vista `TECNICO_REVISAR`, nessuna approvazione, `no_extrato_aprovado = NAO` |
| `tx54_sem_inicializar_em_arquivo_ausente` | archivio mancante senza il flag: uscita 6, niente creato |
| `tx55_inicializar_em_arquivo_existente` | flag su archivio esistente: uscita 6, archivio byte-identico |
| `tx56_duplicada_na_execucao` | due file, stessa chiave: uscita 3, non pubblicato |
| `tx57_conteudo_alocado_diverso` | stessa chiave, contenuto diverso: uscita 4, archivio invariato, messaggio che non attribuisce la causa |
| `tx58_nao_ok_arquivado` | documento `PARCIAL`/`ERRO` senza flag: uscita 5, **pubblicato**, documento elencato e fuori dall'estratto approvato |
| `tx59_nao_ok_recusado` | lo stesso con `--recusar-nao-ok`: uscita 7, **non** pubblicato |
| `tx60_mais_de_um_lote` | export con più di un lotto: uscita 1 |
| `tx61_saida_sobre_o_arquivo` | `--saida` sull'archivio o sulla sua famiglia: uscita 4, archivio intatto |
| `tx62_destino_existente` | due export con `recusar`: il secondo esce con 3 e il primo file resta con lo stesso SHA |

Nessuno ridimostra un contratto già coperto dai 389 test esistenti: provano la
composizione, gli esiti e la differenza fra riuso e duplicazione.

## 15. Audit previsto

| Prova | Forma |
|---|---|
| suite completa | `cargo test --locked` |
| forma e analisi statica | `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` |
| rilettura indipendente, modo nuovo | `script/releitura_export.py --modo percurso-cli` sul workbook prodotto **dalla CLI**, con il manifesto costruito dal test |
| rilettura indipendente, fixture | `--modo export` sulla fixture storica, **invariato**, più i dodici casi della controprova: devono restare verdi |
| controprova del modo nuovo | mutazione di importo e mutazione di stato: uscita 1 con la cella nominata |
| conteggi del rilancio | a mano: primo avvio e rilancio, con i conteggi riportati, per dimostrare che un rilancio non gonfia il risultato |
| tabelle degli esiti | a mano, un caso per codice e per comando |
| disciplina Git | `git status -sb`, `git diff --name-status`, `git diff --cached --name-status`, e tabella di classificazione |

## 16. File che il lotto può toccare

| File | Perché |
|---|---|
| `src/main.rs` | i due comandi: argomenti, uscita, codici |
| `src/importacao.rs` (nuovo) | la parte **pura** della composizione: raccolta della cartella, confronto delle impronte, classificazione degli esiti. Separata dall'IO per essere provata da sola |
| `src/lib.rs` | dichiarazione del modulo |
| `tests/cli_percurso_nfe.rs` (nuovo) | i test di percorso, e la costruzione del manifesto nell'arena |
| `script/releitura_export.py` | il modo `percurso-cli`, **senza** toccare le attese di `export` |
| `script/controprova_rilettura.py` | i casi del modo nuovo, **senza** toccare i dodici esistenti |
| `src/arquivo.rs` | **solo** se `D28` è approvata: la visibilità di `sha256_dos_bytes` |
| `README.md`, specifica 3C.4, questo mandato | stato, contratti, appendici |

Nessun altro file. In particolare: nessuna modifica ad `alocacao`, `prelivro`,
`revisao`, `arquivo_io`, `export`, `export_io`, `xlsx`, `nfe`,
`classificacao`, `regras`, alle sei fonti canoniche, e nessuna dipendenza
nuova.

## 17. Decisioni che chiedono approvazione

| # | Decisione | La mia proposta |
|---|---|---|
| D27 | istante | opzionale, con la fonte etichettata **nell'uscita della CLI** e in nessun altro posto |
| D28 | `IdentidadePlano::sha256_arquivo` | valorizzarlo; visibilità circoscritta di `sha256_dos_bytes`, nessuna dipendenza nuova. Comporta l'attesa nuova del modo `percurso-cli` (§10) |
| D29 (corretta) | duplicati | controllo **dentro** l'esecuzione; chiave già archiviata identica = riuso dichiarato; contenuto diverso = rifiuto con messaggio fedele |
| D30 | stato tecnico `PARCIAL`/`ERRO` | conservare il documento con i suoi problemi, uscita 5; `--recusar-nao-ok` rifiuta prima di pubblicare, uscita 7 |
| D31 (ridotta) | scopo dell'esecuzione | **solo** rilettura completa della cartella. `novos` rinviato |
| D32 (rinviata) | risultato fra esecuzioni | fuori: richiede selezione e deduplicazione che l'export non ha |
| **D33 (nuova)** | contenuto allocato diverso quando **la configurazione** è cambiata | serve una scelta fra (a) registrare l'identità delle regole nell'archivio e (b) un modo dichiarato di accettare la differenza. Finché non è decisa, il limite «non cambiare le regole fra due rilanci» va dichiarato |
| **D34 (nuova)** | fattura rimossa dalla cartella | se deve restare nel risultato corrente, serve una selezione dichiarata: lotto a sé |

L'audit non approva per silenzio, e io non implemento nulla prima che `D27`–`D34`
abbiano una risposta.

## 18. Misure da prendere all'inizio

Il campione reale classifica **pulito**: tre articoli, zero da revisionare,
zero avvisi, misurato con `importar-nfe`. Lo `StatoDocumento` dipende però dalla
riconciliazione, che quel comando non mostra: va **misurato** al primo passo, e
`tx50` deve attendersi il valore misurato, non quello sperato. Se risultasse
`PARCIAL`, l'uscita attesa di `tx50` diventa 5 e la cartella per l'uscita 0 va
costruita.

Serve inoltre una cartella di prova con tre chiavi diverse: il repository
versiona **una** NF-e (`samples/nfe_exemplo/`, da non modificare, è l'oracolo
del parser). Le altre le costruisce il test derivandole dalla prima con chiave e
numero cambiati, scrivendo nel test la provenienza della derivazione.

## 19. Rischi

| Rischio | Mitigazione |
|---|---|
| ogni rilancio ripete gli snapshot: 50 fatture rilanciate 12 volte lasciano 600 snapshot | è D16(a): l'archivio cresce senza potatura. Ogni risultato resta per esecuzione, quindi nessun conteggio raddoppia. L'aritmetica va detta a Luca prima, non dopo |
| un aggiornamento delle regole blocca il rilancio della cartella | è `D33`, aperta. Finché non è decisa, il limite va scritto nei `limites` e nell'aiuto del comando |
| qualcuno somma due esecuzioni | più lotti sono rifiutati; `proveniencia` nomina l'esecuzione; `tx51` misura il conteggio dopo un rilancio |
| si crede che il risultato sia «il mese» | il risultato copre un'esecuzione, e l'uscita del comando lo ripete in una riga |
| una fattura rimossa dalla cartella sparisce dal risultato corrente | §5 lo dichiara; `D34` decide se va cambiato |
| gli argomenti obbligatori sono molti | lo sono perché i moduli rifiutano di dedurli. Un file di profilo è una decisione successiva |

## 20. Limiti Excel, conservati

Il workbook della CLI porta lo **stesso** foglio `limites`, invariato:
`EXCEL_NAO_ATESTADO` resta `NON_VERIFICATO`, `ESCAPE_LEITOR` resta
`DIVERGENZA_NOTA`, accanto a `DERIVADO_SEM_RETORNO`,
`AUTOR_NAO_AUTENTICADO`, `NUMERO_NAO_AUTORITATIVO`, `SEM_SEQUENCIA_HISTORICA`
e `DURABILIDADE_ARQUIVO`. `TX-18` resta **parziale** e `TX-19`
**NON_ESEGUITA**: questo lotto non attesta la compatibilità con Microsoft
Excel e non rimuove nessuna divergenza nota del lettore. La conferma visiva dei
tre stati del campione corretto resta quella di
`verifica_visiva_excel_20261030/CONFERMA_VISIVA_R3C41_05.md`.

## 22. Approvazione di Luca — 2026-10-31

> «Approvo il primo lotto con rilettura completa della cartella. Il nuovo
> Excel deve contenere le fatture attualmente presenti; quelle rimosse restano
> nello storico. Per ora confermo il rifiuto esplicito quando una fattura già
> archiviata produce contenuto allocato diverso, senza modificare lo schema
> dell'archivio. Approvo le altre scelte consigliate da Codex nell'ultimo
> rapporto, con D32 rinviata. Registra le decisioni nel mandato, applica gli
> allineamenti tecnici richiesti e procedi con l'implementazione.»

| # | Decisione | Esito | Come è attuata |
|---|---|---|---|
| D27 | istante e sua fonte | **APPROVATA** | `--instante` opzionale; l'etichetta `DECLARADO` o `RELOGIO_DO_SISTEMA` compare **solo** nell'uscita della CLI, perché `Proveniencia` non ha un campo per la fonte |
| D28 | impronta del piano | **APPROVATA** | `IdentidadePlano::sha256_arquivo` riempita; `arquivo::sha256_dos_bytes` resa pubblica, una riga, nessuna dipendenza nuova |
| D29 | duplicati | **APPROVATA** | controllo **dentro** l'esecuzione; chiave già archiviata identica = riuso dichiarato |
| D30 | stato tecnico non `OK` | **APPROVATA** | archiviato con uscita 5; `--recusar-nao-ok` rifiuta prima di pubblicare con uscita 7 |
| D31 | scopo dell'esecuzione | **APPROVATA, ridotta** | solo rilettura completa della cartella. `novos` non implementato |
| D32 | risultato fra esecuzioni | **RINVIATA** | non implementata, e il limite è dichiarato in §8 e nell'uscita del comando |
| D33 | contenuto allocato diverso | **APPROVATA come rifiuto, «per ora»** | rifiuto con uscita 4, senza flag e **senza toccare lo schema** dell'archivio. Ne segue un limite operativo, §24.3 |
| D34 | fattura rimossa dalla cartella | **APPROVATA** | il risultato copre la cartella al momento del rilancio; la fattura rimossa resta nello storico e non entra nell'esecuzione nuova |

## 23. Esito dell'implementazione — 2026-10-31

### 23.1 Che cosa è stato scritto

| File | Contenuto |
|---|---|
| `src/importacao.rs` (nuovo, 330 righe) | la parte **pura**: i tre esiti di un documento, il rapporto, i codici d'uscita come costanti, la costruzione del contesto da una NF-e letta. Nessun IO, nessun orologio |
| `src/main.rs` | i due comandi, i loro argomenti, la raccolta della cartella, le tabelle degli esiti |
| `src/lib.rs` | dichiarazione del modulo |
| `src/arquivo.rs` | **una** riga: la visibilità di `sha256_dos_bytes` (D28) |
| `tests/cli_percurso_nfe.rs` (nuovo, 14 test) | gli esiti e la composizione, più il deposito per la rilettura |
| `script/releitura_export.py` | il modo `percurso-cli` con attese **sue**; quelle della fixture invariate |
| `script/controprova_rilettura.py` | `--modo percurso-cli` con le sue copie alterate; i dodici casi della fixture invariati |
| `README.md` | uso, contratti `CLI-01`–`CLI-09`, conteggi, limiti |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | una frase: dalla 3C.5.a l'impronta del piano è riempita |

Nessun altro file. In particolare: `alocacao`, `prelivro`, `revisao`,
`arquivo_io`, `export`, `export_io`, `xlsx`, `nfe`, `classificacao`, `regras`
non sono stati toccati, nessuna dipendenza è stata aggiunta, e le sei fonti
canoniche di `config/` e `samples/` sono invariate.

### 23.2 Le misure prese, non sperate

| Misura | Valore |
|---|---|
| stato tecnico del campione reale | **`OK`**: `arquivar-nfe` esce con 0 e `STATUS: ARQUIVADO`. `tx50` attende questo valore misurato |
| totale sbilanciato (`vNF` 99.00 contro 35.00 di articoli) | stato `Erro` → uscita **5**, `ARQUIVADO_COM_RESERVA`; con `--recusar-nao-ok` uscita **7** e niente pubblicato |
| rilancio sulla stessa cartella | revisione 2, `arquivo.json.ger1` conservata, tre documenti riportati `JA_ARQUIVADO_IDENTICO`, e l'Excel della nuova esecuzione conta **tre** documenti, non sei |
| campione del percorso CLI | 3 documenti, 9 righe, 105.00 in tutto; `estado_revisao = TECNICO_OK` su tutte le righe, zero approvazioni, `linhas_excluidas_do_extrato = 9` |
| impronta del piano archiviata | presente, 64 esadecimali (D28) |

### 23.3 Verifiche

| Prova | Esito |
|---|---|
| `cargo test --locked` | **404** verdi: 367 lib, 3 binario, 34 integrazione. I 389 precedenti invariati |
| `cargo fmt --check` | pulito |
| `cargo clippy --locked --all-targets -- -D warnings` | pulito |
| `git diff --check` | pulito |
| rilettura, modo `percurso-cli` | **3291** confronti, 1 divergenza nota realmente esercitata (la frase del `limites`), 0 discordanze, uscita 0 |
| rilettura, modo `export` sulla fixture | 411 celle, 998 confronti, 5 divergenze note, 0 discordanze, uscita 0: **invariata** |
| rilettura, modo `tx38` | uscita 0 |
| controprova, fixture | dodici casi, tutti conformi: **invariata** |
| controprova, modo `percurso-cli` | sette casi conformi: baseline 0, senza manifesto 2, quattro mutazioni 1 con la cella nominata, una mascherata 1 |

### 23.4 Differenze rispetto alla proposta

| Punto | Proposta | Fatto | Perché |
|---|---|---|---|
| `tx60` | «più di un lotto rifiutato» | `tx60_lote_inexistente`: lotto inesistente → uscita 1, con l'elenco delle esecuzioni disponibili | `--lote` accetta **un** valore e non esiste modo di nominarne più d'uno: `CLI-08` è garantito dalla costruzione, e il test prova il caso osservabile |
| numero dei test | tredici | **quattordici** | il quattordicesimo è il deposito per la rilettura, che la proposta chiedeva ma non contava |
| messaggi degli errori di IO | non specificato | la CLI usa `Display`: messaggi leggibili, con la variante e il percorso | **la prima versione di questa riga era sbagliata**: avevo scritto che `Display` non esisteva per quei tipi, misurandolo con un `grep` che cercava `impl fmt::Display` e non vedeva `impl std::fmt::Display`. L'audit l'ha corretta: gli implementatori ci sono, in `arquivo_io.rs:246`, `export_io.rs:89` e `xlsx.rs:50`. Usare `{:?}` era una **mia scelta**, non un limite tecnico, e ora la CLI usa `{}` |

## 24. Limiti, dichiarati

### 24.1 Excel

Il workbook della CLI porta lo **stesso** foglio `limites`: `EXCEL_NAO_ATESTADO`
resta `NON_VERIFICATO`, `ESCAPE_LEITOR` resta `DIVERGENZA_NOTA`, accanto a
`DERIVADO_SEM_RETORNO`, `AUTOR_NAO_AUTENTICADO`, `NUMERO_NAO_AUTORITATIVO`,
`SEM_SEQUENCIA_HISTORICA` e `DURABILIDADE_ARQUIVO`. `TX-18` resta **parziale**
e `TX-19` **NON_ESEGUITA**: questo lotto non attesta la compatibilità con
Microsoft Excel. L'uscita del comando ripete in una riga che il derivato non
ha percorso di rientro.

### 24.2 Crescita dell'archivio

Ogni rilancio archivia di nuovo i documenti della cartella: 50 fatture
rilanciate 12 volte lasciano 600 snapshot. È `D16(a)`, già dichiarata —
l'archivio cresce senza potatura — e nessun conteggio raddoppia, perché ogni
risultato copre una esecuzione. L'aritmetica è detta prima, non dopo.

### 24.3 Il limite che nasce da `D33`

Le regole di classificazione entrano nell'impronta di contenuto
(`conta_id`, `conta_nome`, `grupo_conta`, `alertas`), e l'archivio registra
l'identità del **piano** ma non quella delle regole. Quindi, dopo un
aggiornamento delle regole, un rilancio della stessa cartella viene rifiutato
con 4 **per i documenti il cui contenuto allocato cambia davvero**, e il
rifiuto ferma l'intera esecuzione perché un lotto è un solo commit.

Quanti siano quei documenti dipende dalla modifica: una regola nuova che
sposta un articolo di conto cambia l'impronta dei documenti che contengono
quell'articolo, non di tutti. **La prima versione di questo paragrafo diceva
«ogni documento», ed era un'esagerazione**: l'audit l'ha corretta.

È scritto nel README e nel messaggio di rifiuto, che non attribuisce la causa
perché non può: la differenza può venire dalla fonte o dalla configurazione.
Registrare l'identità delle regole nell'archivio sarebbe un **miglioramento
della provenienza**, non la condizione logica del rifiuto, che funziona già:
non è stato fatto qui, e `D33` resta come Luca l'ha approvata.

### 24.4 Altri limiti invariati

Il risultato copre un'esecuzione e non un mese (`D32` rinviata); gli eventi
sono globali all'archivio e puntano alla chiave, quindi una futura
approvazione si applicherebbe a ogni esecuzione che contiene quel documento —
osservazione da tenere per il blocco che introdurrà l'approvazione dalla CLI;
la ricerca nella cartella non è ricorsiva; `--se-existir substituir` conserva
il precedente e non cancella niente.

## 25. Stato Git alla consegna per l'audit

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/importacao.rs` | §23.1, modulo puro della composizione | nuovo, non tracciato, provato | **gruppo codice**, dopo l'audit e l'autorizzazione |
| `tests/cli_percurso_nfe.rs` | §23.1, quattordici test | nuovo, non tracciato, verde | gruppo codice |
| `src/main.rs` | i due comandi | modificato | gruppo codice |
| `src/lib.rs` | dichiarazione del modulo | modificato | gruppo codice |
| `src/arquivo.rs` | D28, visibilità di `sha256_dos_bytes` | modificato, una riga | gruppo codice |
| `script/releitura_export.py` | modo `percurso-cli` | modificato, attese della fixture invariate | gruppo codice |
| `script/controprova_rilettura.py` | `--modo percurso-cli` | modificato, dodici casi della fixture invariati | gruppo codice |
| `README.md` | uso, contratti, conteggi, limiti | modificato | gruppo codice |
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | nota su D28 | modificato, una frase | **gruppo documentale** |
| `PROPOSTA_MANDATO_CLAUDE_FASE3C5A_CLI_IMPORTACAO_20261031.md` | questo mandato, §22–§25 | non tracciato | gruppo documentale |

Nessun file in indice, nessun commit, nessun push, nessun `add`. Niente reset,
restore, checkout, clean, stash, amend o rebase. `main` è allineato a
`origin/main`. Il repository Python è stato solo letto.

Due file del lotto hanno fine riga `LF` nella copia di lavoro — `src/main.rs`,
riscritto da `rustfmt`, e `script/controprova_rilettura.py` — e Git lo segnala
perché `core.autocrlf` è `true`. Il diff che ne risulta è **per riga**, non il
file intero: la normalizzazione di Git lo gestisce come per gli altri file del
repository. Non ho cambiato le fini riga di nessun file: dirlo è più utile che
sistemarlo senza averlo chiesto.

---

## 26. Esito dell'audit del 2026-10-31, e `R3C5A-I01` chiuso

**Il percorso operativo supera le verifiche.** Codex ha misurato in modo
indipendente, con una sonda propria che invoca il binario reale e rilegge gli
XLSX senza gli helper dei test del lotto: primo caricamento, aggiunta della
quarta fattura, rimozione di una fattura dalla cartella, impronta del piano
confrontata con uno SHA-256 calcolato a parte sui byte del YAML, generazione
precedente identica byte per byte, nessuna approvazione automatica. Più i
1504 confronti del modo `percurso-cli` e i sette casi della sua controprova.

Le provenienze restano distinte: la suite di **404**, `clippy` e `tx38` sono
attestazioni della mia consegna, non una seconda suite indipendente.

### 26.1 `R3C5A-I01`: il temporaneo che resta viene nominato

| Voce | Valore |
|---|---|
| Classificazione | MICRO, P2: completamento del contratto `CLI-01`, nessuna garanzia di scrittura nuova |
| Difetto | dopo un errore **successivo** alla creazione del temporaneo, il messaggio nominava il percorso della copia fallita ma **non** il temporaneo rimasto. Chi riprende non sapeva che file cancellare, e un nuovo tentativo con la stessa etichetta falliva con `ResiduoPresente` |
| Correzione | la CLI **guarda** quali residui esistono e li elenca con percorso e dimensione, dicendo che non vengono cancellati da sé. Per l'archivio riusa `arquivo_io::candidatos`, l'API già provata, e tiene i soli `TemporarioRestante`: le generazioni sono conservate per `D16(a)` e non sono residui |
| Niente di nuovo | nessun residuo cancellato, nessun rename cambiato, nessun tentativo automatico, nessuna politica di lucchetto toccata. La ripresa resta una scelta umana |
| Errore originale | stampato **prima** e mai mascherato. Se la verifica dei residui non riesce, l'esito è `NON_DISPONIVEL`, che non si trasforma in «nessun residuo» |

**La regressione**, `tx64_residuo_nomeado_depois_de_erro_de_io`: il protocollo
scrive il temporaneo al passo 4 e rifiuta al passo 5 se la copia del
precedente esiste già, quindi precreare quel file produce un errore dopo la
creazione del temporaneo, in modo deterministico e senza bloccare niente. Il
test verifica uscita **6**, il temporaneo **nominato**, la frase sui residui
non cancellati, e il file finale precedente **byte-identico**.

Non è la prova con un handle esclusivo di un altro processo: quella l'ha
misurata l'audit e resta sua. `TX-16b` su Microsoft Excel **non** è eseguita.

### 26.2 Le due correzioni documentali

| Punto | Che cosa ho corretto |
|---|---|
| `D33` | §24.3 e il README dicevano che dopo un aggiornamento delle regole **ogni** documento viene rifiutato. È un'esagerazione: il rifiuto riguarda i documenti il cui contenuto allocato cambia. E registrare l'identità delle regole è un miglioramento della provenienza, non la condizione logica del rifiuto. `D33` resta come approvata |
| `Display` | §23.4 diceva che `ErroIo`, `ErroExportIo` e `ErroXlsx` non implementano `Display`. **Falso**: `arquivo_io.rs:246`, `export_io.rs:89`, `xlsx.rs:50`. Il mio `grep` cercava `impl fmt::Display` e non vedeva `impl std::fmt::Display`. Era una mia scelta stampare con `{:?}`, e ora la CLI usa `Display` |

### 26.3 Verifiche dopo la correzione

| Prova | Esito |
|---|---|
| `cargo test --locked` | **404** verdi, compreso `tx64` |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti |
| rilettura `percurso-cli` | 1504 celle, 3291 confronti, 1 divergenza nota, 0 discordanze, uscita 0 |
| rilettura `export` e `tx38` sulla fixture | invariate, uscita 0 |
| controprova, fixture e modo nuovo | dodici e sette casi conformi |
| diagnostica a mano | errore dopo il temporaneo: uscita 6, `TEMPORARIO` e `ANTERIOR` nominati con la dimensione, file finale intatto |

Lo stato Git di §25 non cambia: gli stessi dieci file, nessuno staged, nessun
commit, nessun push. Il controllo successivo di Codex resta limitato a questa
diagnostica.

---

## 27. Il residuo dell'helper archivio, chiuso (2026-10-31)

Il controllo mirato ha confermato la diagnostica dell'export, anche con un
handle esclusivo di un altro processo, e ha trovato per ispezione del ramo
d'errore due punti in cui `residuos_do_arquivo` **non** rispettava il
contratto che §26.1 dichiara.

### 27.1 Che cosa era sbagliato

| Punto | Difetto | Perché conta |
|---|---|---|
| dimensione dei temporanei | `std::fs::metadata(...).map(\|d\| d.len()).unwrap_or(0)` | trasformava una misura **non disponibile** in `0 bytes`. Zero byte è una misura reale e possibile: confonderla con l'ignoto è lo zero tecnico che questo progetto vieta, e l'ho scritto proprio nell'helper che doveva dichiarare `NON_DISPONIVEL` |
| lucchetto | `if let Ok(dados) = std::fs::metadata(&lucchetto)` | ingoiava **qualunque** errore. `NotFound` è un'assenza; ogni altro errore non permette di dichiarare il lucchetto assente né la verifica completa |

Il rilievo viene dall'ispezione del ramo d'errore, non da una riproduzione con
permessi o corse del filesystem: nei test ordinari il difetto non si
presentava, e non era un errore contabile.

### 27.2 La correzione

Un solo punto decide i tre esiti, e lo usano entrambi gli helper:

```rust
fn olhar_residuo(papel: &'static str, caminho: PathBuf) -> Result<Option<Residuo>, String> {
    match std::fs::metadata(&caminho) {
        Ok(dados) => Ok(Some(Residuo { papel, caminho, bytes: dados.len() })),
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(erro) => Err(format!("{}: {erro}", caminho.display())),
    }
}
```

Presente con la sua dimensione, assente, oppure verifica impossibile che
risale al chiamante e diventa `NON_DISPONIVEL`. Nessun `unwrap_or`, nessun
errore ingoiato, nessun tipo nuovo e nessuna modifica a `declarar_residuos`,
che già traduceva `Err` senza mascherare l'errore originale.

### 27.3 Le prove

| Prova | Che cosa fissa |
|---|---|
| `residuo_presente_declara_a_dimensao_real` | la dimensione dichiarata è quella reale |
| `zero_bytes_nao_e_ausencia` | un file di **zero byte** è un residuo con misura reale, e un percorso assente dà `None` senza errore: è la distinzione che `.unwrap_or(0)` cancellava |
| `verificacao_impossivel_nao_vira_zero_nem_ausencia` | un nome non valido fa risalire l'errore, che nomina il percorso. Dichiarato `#[cfg(windows)]`: quali caratteri rendano invalido un nome è una proprietà del sistema, e qui è **misurata** — `in*valido?` dà `InvalidFilename`, errore 123, non `NotFound` |
| `tx65_residuo_do_arquivo_nomeado_depois_de_erro_de_io` | il ramo dell'archivio end-to-end: precreando `arquivo.json.ger1` il passo 6 di `publicar` rifiuta dopo aver scritto il temporaneo, e la CLI nomina `arquivo.json.novo.2` con la sua dimensione. L'archivio corrente resta byte-identico |

Misurato a mano sullo stesso caso: uscita **6**, `TEMPORARIO
…\a.json.novo.2 (22943 bytes)`, errore originale stampato prima.

### 27.4 Verifiche

| Prova | Esito |
|---|---|
| `cargo test --locked` | **408** verdi: 367 lib, **6** binario, 35 integrazione |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti |
| rilettura `percurso-cli` | 1504 celle, 3291 confronti, 0 discordanze, uscita 0 |
| rilettura `export`, `tx38`, entrambe le controprove | invariate, uscita 0 |
| sei fonti canoniche | invariate |

Perimetro invariato: nessuna modifica ai protocolli di scrittura, nessuna
cancellazione, nessun tentativo automatico, niente riaperto su `D33`, `D34`,
l'export già verificato o il nucleo monetario. Lo stato Git di §25 non cambia:
gli stessi dieci file, indice vuoto, nessun commit e nessun push.

---

## 28. Esito finale dell'audit tecnico (2026-10-31)

**AUDIT TECNICO SUPERATO nel perimetro approvato del lotto 3C.5.a.**
`R3C5A-I01` e il residuo dell'helper archivio sono chiusi; nessuna ulteriore
modifica tecnica è richiesta. `olhar_residuo` distingue presente, assente e
verifica impossibile, entrambe le funzioni di diagnostica riusano lo stesso
risultato fallibile, e l'errore originale resta stampato prima della
diagnostica.

### 28.1 Le provenienze delle prove, che non si sommano

| Prova | Chi l'ha eseguita, e quando |
|---|---|
| `cargo test --locked --bin …` (6) e `--test cli_percurso_nfe tx6` (6: `tx60`–`tx65`); lettura mirata dei due helper; `cargo fmt --check`; `git diff --check`; confronto SHA dei dieci file | **Codex**, nel controllo finale sulla correzione |
| percorso a cartella con la sonda indipendente, aggiunta della quarta fattura, fattura spostata, digest del piano confrontato con uno SHA calcolato da Python, generazioni e workbook precedenti byte per byte, rifiuti senza pubblicazione, stati separati, 1504 celle e 3291 confronti del modo `percurso-cli`, 411 celle e 998 della fixture, controprove 7 e 12, errore con handle esclusivo | **Codex**, nei controlli precedenti: prove **riusate**, non rieseguite |
| suite completa di **408**, `clippy`, rilettura `tx38` | **Claude**: attestazioni della consegna, **non** una seconda suite indipendente |

Il test sul nome non valido è dichiarato `#[cfg(windows)]` ed è stato eseguito
su questo target: la premessa non si estende ad altri sistemi.

### 28.2 Che cosa questo esito non dichiara

| Voce | Stato |
|---|---|
| registrazione Git e pubblicazione | **non eseguite e non autorizzate** da nessuno degli audit: restano a Luca |
| `TX-18` prova visiva in Excel | **parziale** |
| `TX-19` decodifica di `_xHHHH_` in Excel | **NON_ESEGUITA** |
| fase 3C | **non chiusa** |
| `D32`, versionamento delle regole, rientro dal workbook, DRE, import dello storico, promozione contabile | fuori, invariati |

Le decisioni restano quelle di §22: cartella attuale completa, esecuzioni
immutabili, `D32` rinviata, rifiuto del contenuto allocato diverso secondo
`D33`, fatture rimosse conservate nello storico secondo `D34`.

### 28.3 La fotografia del codice

L'audit ha salvato le impronte dei dieci file in `file_audit_finale_sha256.json`,
fuori dal repository, per confrontare ciò che verrà eventualmente registrato
con ciò che è stato controllato. Da questo momento **il codice non cambia**: le
appendici documentali che seguono — questa compresa — cambiano legittimamente
l'impronta dei soli documenti, e qualunque modifica di codice andrebbe
dichiarata e ricontrollata.

Lo stato Git di §25 resta valido: gli stessi dieci file, tutti con un destino
esplicito, indice vuoto, nessun commit e nessun push.

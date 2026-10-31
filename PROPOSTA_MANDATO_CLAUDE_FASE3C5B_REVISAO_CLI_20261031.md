# Proposta di mandato — lotto 3C.5.b: revisione umana dalla CLI

| Voce | Valore |
|---|---|
| Data | 2026-10-31 |
| Stato | **AUDIT TECNICO SUPERATO il 2026-10-31** nel perimetro del lotto: `R3C5B-I01` e `R3C5B-I02` chiusi, nessuna modifica tecnica ulteriore richiesta (§18). **Registrazione Git e pubblicazione da autorizzare**: nessun commit, nessun push |
| Classificazione | `MIRATO`: espone dalla CLI le operazioni che `revisao.rs` già attua. Nessuna regola contabile nuova, nessuno schema nuovo, nessuna dipendenza nuova |
| Obiettivo | consultare le righe di un'esecuzione, **annotare**, **approvare**, **revocare**, e rigenerare l'Excel |
| Fuori perimetro | DRE (3D), import dello storico (D11), ingresso grezzo (D12), rientro dal workbook, versionamento dei documenti (D8.5), risultato fra esecuzioni (D32), potatura (D16) |

| Revisione | Che cosa cambia |
|---|---|
| 1 | prima stesura: due comandi, filtro per partizione, «in traccia o fra i non applicati» |
| 5 (2026-10-31) | **audit tecnico superato**: esito finale, provenienze delle prove tenute distinte, fotografia confrontata 8 su 8, e che cosa l'esito non dichiara (§18) |
| 4 (2026-10-31) | audit del 2026-10-31: `R3C5B-I01`, il comando generato che non si eseguiva con percorsi ordinari e perdeva il piano selezionato, e `R3C5B-I02`, il nome storico mostrato accanto all'impronta attuale senza l'avviso della vista. Corretti con due regressioni di accettazione (§17) |
| 3 (2026-10-31) | approvazione di Luca registrata (§13), lotto implementato, verificato e consegnato per l'audit (§14-§16) |
| 2 | audit del 2026-10-31. `R3C5B-01`: il filtro guarda la **provenienza umana**, non la partizione, e la consultazione mostra i cinque stati separati. `R3C5B-02`: il candidato si identifica con `texto_exato_evento`, la ripetizione si cerca fra gli eventi **già archiviati**, gli eventi estranei al lotto non bloccano niente, e «applicato» non significa «lo stato è cambiato». Più sei precisazioni tecniche e la motivazione di `M1` riformulata |

## 1. Che cosa esiste già, e che cosa manca

`revisao.rs` attua D8 e D9 per intero: le tre operazioni, l'ordine canonico, i
conflitti, i cinque motivi di non applicazione, la validazione della forma.
`arquivo.rs` conserva le dichiarazioni (`acrescentar_eventos`) e `export.rs`
mostra già `estado_revisao`, `extrato_revisado`, `decisoes`, `aprovado_por` e
`ultima_confirmacao_por`.

Manca **solo la porta**: nessun comando costruisce un `EventoRevisao`, e oggi
gli eventi esistono unicamente nei test.

**Il fatto che decide il disegno.** Un evento cita l'impronta di contenuto che
la persona ha esaminato (`ChaveRevisao::impressao_conteudo`) e, come
informazione, quella di presentazione (`vista`). Il foglio `diagnostico` porta
già le due colonne, e le prende da `vista.versoes` — la **stessa** vista
revisionata che l'export usa, non l'impronta storica dello snapshot. Chi ha
letto l'Excel ha davanti esattamente ciò che l'evento deve citare:

```text
exportar-diagnostico -> la persona legge l'Excel -> declarar-revisao citando
l'impronta che ha visto -> exportar-diagnostico di nuovo
```

## 2. I due comandi

| Comando | Scrive | Legge |
|---|---|---|
| `consultar-linhas` | **niente** | l'archivio e il piano, in sola lettura |
| `declarar-revisao` | l'archivio: **una** revisione, **un** evento | l'archivio e il piano |

La rigenerazione **non** è un comando nuovo: è `exportar-diagnostico` della
3C.5.a con `--se-existir substituir`. Dopo un evento la stessa esecuzione
produce un Excel diverso, perché `aplicar_revisao` applica la decisione.

Un comando per le tre operazioni, con `--operacao` obbligatoria: nominare
l'operazione è ciò che impedisce di approvare per distrazione.

## 3. Esempio in PowerShell

Una riga per comando, e **apici semplici** per i letterali: in PowerShell una
stringa fra apici doppi interpola `$` e il backtick, quindi non è un quoting
sicuro per un motivo scritto da una persona.

```powershell
# 1. Che cosa non ha un'approvazione umana attiva, e che cosa un evento deve citare.
cargo run -- consultar-linhas --arquivo dados/arquivo.json --lote fonte-maio-r2 --so-sem-aprovacao-humana
```

Uscita, per ogni riga: i cinque stati separati e le due impronte **complete**.

```text
LINHA  chave 29260512345678000195550010001256612345678901  conta 2.01.004  ordinal 0  origem XML_NFE
  status_tecnico         REVISAR          estado_revisao   TECNICO_REVISAR
  particao               REVISAR          stato_documento  OK
  no_extrato_revisado    NAO              valor            10.00
  impressao_conteudo     9f2c…  (64 esadecimali, e' questa che l'evento cita)
  impressao_apresentacao a71b…
  decisoes aplicadas     0    anotacoes 0
```

Più il comando pronto da copiare, con le impronte intere. Generarlo è una
**comodità**: non dimostra che l'operatore abbia verificato i dati, e la
dichiarazione resta un suo atto.

```powershell
# 2. L'approvazione. <IMPRESSAO> e <VISTA> sono i 64 esadecimali stampati sopra.
cargo run -- declarar-revisao --arquivo dados/arquivo.json --lote fonte-maio-r2 --chave 29260512345678000195550010001256612345678901 --conta 2.01.004 --ordinal 0 --origem xml-nfe --operacao aprovar --autor 'Luca Maggi' --motivo 'conferido com a nota em papel' --impressao-conteudo <IMPRESSAO> --vista <VISTA>
# STATUS: DECLARADO | evento APLICADO | revisao 3
#         linha 2.01.004/0: TECNICO_REVISAR -> APROVADO_POR_PESSOA

# 3. Un'annotazione: non decide lo stato.
cargo run -- declarar-revisao --arquivo dados/arquivo.json --lote fonte-maio-r2 --chave 2926…901 --conta 2.04.001 --ordinal 0 --origem xml-nfe --operacao anotar --autor 'Luca Maggi' --motivo 'duvida sobre o rateio' --nota 'conferir com o fornecedor antes do fecho' --impressao-conteudo <IMPRESSAO>

# 4. Una revoca.
cargo run -- declarar-revisao --arquivo dados/arquivo.json --lote fonte-maio-r2 --chave 2926…901 --conta 2.01.004 --ordinal 0 --origem xml-nfe --operacao revogar --autor 'Luca Maggi' --motivo 'a nota em papel era de outro mes' --impressao-conteudo <IMPRESSAO>

# 5. Rigenerazione: il comando della 3C.5.a, nessuno nuovo.
cargo run -- exportar-diagnostico --arquivo dados/arquivo.json --lote fonte-maio-r2 --saida dados/diagnostico_maio.xlsx --se-existir substituir --operador 'Luca Maggi' --motivo 'depois das aprovacoes'
```

Le righe con `2926…901` e `<IMPRESSAO>` sono **illustrazioni**, non comandi
eseguibili: i valori veri li stampa la consultazione, interi.

## 4. Argomenti

### 4.1 `consultar-linhas`

| Argomento | Obbligo | Significato |
|---|---|---|
| `--arquivo`, `--lote` | sì | l'archivio in sola lettura, e l'esecuzione da consultare |
| `--chave`, `--conta`, `--ordinal` | no | filtri; senza filtri elenca tutte le righe |
| `--so-sem-aprovacao-humana` | no | solo le righe il cui `estado_revisao` **non** è `APROVADO_POR_PESSOA` |
| `--mostrar-nao-aplicados` | no | elenca anche gli eventi archiviati non applicati, con il motivo |
| `--plano-contas` | no | il piano usato ora, come negli altri comandi |

**`R3C5B-01`: il filtro guarda la provenienza umana, non la partizione.** Una
riga con stato tecnico ereditato `APROVADO` ma senza un evento che lo
giustifichi è `APROVADO_SEM_PROVENIENCIA` (D9.4), e la 3C.2 può assegnarle
partizione `APROVADO`. Filtrare per partizione la nasconderebbe **proprio** a
chi deve sanarla con una decisione esplicita. Il filtro usa quindi
`EstadoRevisao::AprovadoPorPessoa`.

Per lo stesso motivo la consultazione stampa i cinque stati **separati** —
`status_tecnico`, `estado_revisao`, `particao`, `stato_documento`,
`no_extrato_revisado` — e non ne deduce uno dall'altro. Un'approvazione su un
documento `ERRO`, che D9.2 ammette, **non** viene presentata come inclusione
nell'estratto: si stampano l'esclusione e il motivo che la vista già produce.

### 4.2 `declarar-revisao`

| Argomento | Obbligo | Significato |
|---|---|---|
| `--arquivo`, `--lote` | sì | l'archivio, e l'esecuzione su cui si verifica l'effetto |
| `--chave`, `--conta`, `--ordinal`, `--origem` | sì | il bersaglio completo di `ChaveRevisao`. `origem` viene dalla riga **selezionata**, senza valore predefinito arbitrario |
| `--operacao` | sì | `aprovar`, `revogar`, `anotar` |
| `--autor` | sì | attribuisce la dichiarazione; non autentica nessuno |
| `--motivo` | sì | non vuoto |
| `--nota` | solo con `anotar` | obbligatoria per `ANOTAR`, ammessa altrove |
| `--impressao-conteudo` | sì | l'impronta che la persona **ha esaminato**. Non viene sostituita con quella della riga corrente |
| `--vista` | no | l'impronta di presentazione. Omessa: assenza **dichiarata** con il suo motivo, che il `Dado` ammette |
| `--instante`, `--sequencia` | no | istante RFC 3339; omesso, orologio della macchina etichettato come in D27. `--sequencia` ordina due eventi dello stesso istante |
| `--operador` | no | operatore del lucchetto; predefinito `--autor`. Il motivo del lucchetto è derivato e dichiarato |

**Perché l'impronta non viene letta dall'archivio.** Riempirla da sé
renderebbe `VERSAO_SUPERADA` impossibile e la provenienza della decisione una
formalità. La consultazione la stampa pronta da copiare.

## 5. Il controllo che precede la scrittura

`aplicar_revisao` **non** tollera un evento malformato né una coppia
incompatibile: `AutorNaoDeclarado`, `MotivoNaoDeclarado`, `NotaNaoDeclarada`,
`AlvoIncompleto`, `AlvoAmbiguo` e `DecisoesIncompativeis` sono `ErroRevisao`, e
fanno fallire **l'intera** applicazione. L'archivio è append-only e non si
corregge (D16): un evento così, archiviato, renderebbe impossibile ogni export
futuro di quell'archivio.

Quindi `declarar-revisao`, prima di pubblicare:

1. costruisce il candidato e ne calcola l'identità esatta con
   `revisao::texto_exato_evento`, che è la codifica **iniettiva** dei dati
   dichiarati, istante e offset compresi. Non si usano `ChaveEvento`, che non
   è unica per contratto, né un indice, né l'impronta C23;
2. confronta quell'identità con gli eventi **già archiviati**: se esiste un
   testo esatto uguale, la dichiarazione è una `REPETICAO` e si rifiuta
   **prima** di applicare. Cercarla nella traccia dopo l'applicazione
   troverebbe il rappresentante vecchio e accetterebbe la ripetizione nuova,
   perché in caso di duplicato il rappresentante resta applicato e la
   ripetizione va fra i non applicati;
3. applica il candidato **insieme a tutta la storia** degli eventi
   dell'archivio, e classifica **solo l'esito del candidato**, riconoscendolo
   per identità esatta;
4. pubblica se il candidato risulta applicato, conservando nel candidato di
   pubblicazione **l'intera** storia degli eventi;
5. se non è applicato, non scrive niente e nomina il motivo fra i cinque.

**Due cose che questo controllo non fa.** Non pretende che
`eventos_nao_aplicados` sia vuoto: un evento che riguarda un documento
presente solo in un altro lotto diventa `ALVO_AUSENTE` nella vista del lotto
scelto, ed è normale — `sub_arquivo_do_lote` conserva tutti gli eventi, e
nessun evento estraneo viene scartato dal candidato. E non giudica l'esito dal
fatto che lo stato della riga sia cambiato: `ANOTAR`, una conferma e un evento
dichiarato retroattivamente possono applicarsi **senza** cambiare lo stato
corrente.

Una seconda approvazione a istante diverso è una dichiarazione **distinta**,
ammessa dal modello con l'avviso `APROVACAO_REPETIDA`: non è una `REPETICAO`.

## 6. Esiti

| Codice | Esito di `declarar-revisao` | Fatto attestabile |
|---|---|---|
| 0 | candidato applicato e pubblicato | **archivio corrente pubblicato**, revisione N; stato della riga prima → dopo |
| 1 | configurazione, argomenti, lotto o riga inesistenti nell'esecuzione | archivio **non** pubblicato |
| 2 | la barriera di `revisao.rs` rifiuta: forma dell'evento, bersaglio ambiguo, **o** decisioni incompatibili alla stessa coordinata temporale. Causa e, per il conflitto, i **due** eventi nominati | archivio **non** pubblicato; archivio ed export precedenti restano utilizzabili |
| 3 | il candidato **non** sarebbe applicato: `ALVO_AUSENTE`, `VERSAO_SUPERADA`, `PROVENIENCIA_AUSENTE`, `NADA_A_REVOGAR`, `REPETICAO` | archivio **non** pubblicato |
| 6 | rifiuto di IO prima del commit, con i residui nominati come in `R3C5A-I01` | archivio **non** pubblicato |

Una categoria per famiglia, non un codice per variante. `consultar-linhas`
esce con 0, con 1 per configurazione o lotto inesistente, e con 6 se
l'archivio è assente o illeggibile, con la stessa convenzione di IO della
3C.5.a. **Nessuna riga che corrisponda al filtro** non è un errore: è uscita 0
con un elenco vuoto, e non si confonde con un fallimento di lettura.

## 7. Contratti nuovi

| Contratto | Regola | Prova |
|---|---|---|
| `REV-01` | nessun evento viene archiviato se il candidato non si applica al bersaglio scelto | `tx74`, `tx77` |
| `REV-02` | l'impronta esaminata è **dichiarata**, mai dedotta dall'archivio | `tx74` |
| `REV-03` | la consultazione non scrive: archivio byte-identico prima e dopo | `tx70` |
| `REV-04` | il filtro delle righe senza approvazione umana usa `APROVADO_POR_PESSOA`, e i cinque stati si mostrano separati | `tx70b` |
| `REV-05` | `ANOTAR` non decide lo stato; `REVOGAR` senza approvazione applicata non si archivia | `tx72`, `tx73` |
| `REV-06` | un'approvazione vale per **ogni** esecuzione che contiene quel documento con la **stessa** impronta; un'impronta diversa non eredita niente | `tx75` |
| `REV-07` | l'identità del candidato è `texto_exato_evento`; una ripetizione esatta si rifiuta, una dichiarazione distinta si ammette | `tx77` |
| `REV-08` | gli eventi estranei al lotto non impediscono una nuova dichiarazione, e la storia resta intera nell'archivio | `tx78` |
| `REV-09` | una coppia incompatibile non viene mai archiviata: l'archivio resta usabile | `tx79` |

## 8. Test del percorso completo

`tests/cli_revisao_nfe.rs`. Il numero viene dai casi, non da una quota.

| Test | Che cosa prova |
|---|---|
| `tx70_consultar_nao_escreve` | elenca le righe con i cinque stati e le due impronte intere; filtri; archivio byte-identico |
| `tx70b_sem_proveniencia_aparece` | una riga `APROVADO_SEM_PROVENIENCIA` compare fra quelle senza approvazione umana, e una riga approvata con evento attivo **non** compare. La riga si costruisce con l'API pubblica — `alocar_nfe` produce solo `OK` e `REVISAR`, quindi la CLI non può generarla da un XML, e il test lo dichiara |
| `tx71_aprovar_e_regenerar` | approvazione applicata, revisione +1, e l'Excel rigenerato porta `APROVADO_POR_PESSOA`, `no_extrato_revisado = SIM`, una riga in `extrato_revisado` e una in `decisoes` |
| `tx72_revogar` | revoca dopo approvazione → `APROVACAO_REVOGADA`, fuori dall'estratto revisionato; revoca senza approvazione → uscita 3, niente archiviato |
| `tx73_anotar_nao_decide` | annotazione registrata, `estado_revisao` e `particao` invariati, e il candidato è **applicato** anche se lo stato non cambia; senza `--nota` uscita 2 |
| `tx74_impressao_estantia_recusada` | impronta di un'altra versione → uscita 3 con `VERSAO_SUPERADA`, archivio byte-identico |
| `tx75_aprovacao_sobrevive_ao_relancamento` | dopo l'approvazione si rilancia la cartella (3C.5.a): l'export della **nuova** esecuzione mostra già l'approvazione. Con un documento di contenuto diverso, l'approvazione **non** si eredita |
| `tx76_forma_recusada_nao_entra_no_arquivo` | autore o motivo vuoti → uscita 2 e, soprattutto, l'archivio resta usabile: l'export successivo funziona |
| `tx77_repeticao_e_declaracao_distinta` | ripetizione di **tutti** i dati dichiarati, istante compreso → uscita 3 con `REPETICAO`; la stessa approvazione a un istante diverso → applicata, con l'avviso `APROVACAO_REPETIDA` |
| `tx78_evento_estraneo_non_blocca` | un evento storico su un documento di un altro lotto resta fra i non applicati di questa vista, la nuova dichiarazione valida si pubblica, e l'evento storico resta nell'archivio |
| `tx79_decisoes_incompativeis` | approvazione e revoca alla stessa coordinata temporale → uscita 2, i due eventi nominati, archivio ed export precedenti ancora utilizzabili |

## 9. File da toccare

| File | Perché |
|---|---|
| `src/declaracao.rs` (nuovo) | la parte **pura**: costruire l'evento dai campi dichiarati, calcolarne l'identità esatta, cercare la ripetizione fra gli archiviati, classificare l'esito del candidato. Nessun IO |
| `src/main.rs` | i due comandi |
| `src/lib.rs` | dichiarazione del modulo |
| `tests/cli_revisao_nfe.rs` (nuovo) | i test, e il deposito del workbook **con** approvazioni |
| `script/releitura_export.py` | un modo nuovo, `percurso-cli-revisado`: il modo `percurso-cli` attende **zero** approvazioni e non può validare un workbook che ne ha. Le attese di `export` e `percurso-cli` restano intatte |
| `script/controprova_rilettura.py` | i casi del modo nuovo, compresa la mutazione che finge un'approvazione mai dichiarata |
| `README.md`, questo mandato | uso, contratti, limiti |

Nessun'altra modifica. In particolare `revisao::validar` **non** viene esposta:
è privata, e `aplicar_revisao` valida già la forma prima di applicare.
`revisao.rs`, `arquivo.rs`, `arquivo_io.rs`, `export.rs`, `export_io.rs`,
`xlsx.rs`, `prelivro.rs`, `alocacao.rs`, `nfe.rs`, `importacao.rs` e le sei
fonti canoniche restano invariati.

## 10. Le decisioni che chiedono una risposta

| # | Decisione | La mia proposta |
|---|---|---|
| **M1** | un candidato non applicabile al bersaglio scelto va archiviato come dichiarazione o rifiutato? | **rifiutarlo**, uscita 3. La motivazione corretta non è che una dichiarazione non applicabile sia falsa o illecita — una versione vecchia resta conservata nello storico — ma che **questo comando propone di pubblicare soltanto il nuovo evento applicabile al bersaglio selezionato**. Archiviarla comunque è difendibile e vuole un flag dichiarato |
| **M2** | una dichiarazione può coprire più righe in un comando? | **un bersaglio per comando** in questo lotto, con autore, motivo e impronta dichiarati. L'approvazione multipla è rinviata, e il limite va detto chiaramente per 100 fatture; rinviarla **non** rende 3C.5.c obbligatoria prima della prima prova operativa |
| **M3** | un'approvazione vale per ogni esecuzione che contiene quel documento | **comportamento già ereditato** da D8/D9, non una scelta nuova: stessa chiave completa di revisione e stessa impronta ritrovano la decisione fra esecuzioni; un'impronta diversa non la eredita. Da riconoscere perché significa che un rilancio della cartella non chiede di riapprovare |

## 11. Rischi

| Rischio | Mitigazione |
|---|---|
| un evento invalido o una coppia incompatibile archiviati rompono ogni export futuro | §5: forma, conflitti e applicazione si verificano **prima** di pubblicare |
| l'operatore copia l'impronta sbagliata | il candidato non si applica e non viene archiviato: `VERSAO_SUPERADA` è un rifiuto, non un danno |
| 64 caratteri esadecimali a mano | la consultazione stampa il comando completo, con le impronte intere e i letterali fra apici semplici |
| si crede che il comando generato dimostri una verifica | è una comodità: la dichiarazione resta un atto dell'operatore, e l'uscita lo ripete |
| si crede che l'autore sia autenticato | `AUTOR_NAO_AUTENTICADO` è nel foglio `limites`, e l'uscita lo ripete |
| una revisione dell'archivio per evento | è `D16(a)`: nessuna potatura, generazioni conservate. Con molte righe è l'argomento di `M2` |

## 12. Limiti, conservati

`TX-18` resta **parziale** e `TX-19` **NON_ESEGUITA**. Restano
`DERIVADO_SEM_RETORNO` — una modifica al workbook non è una decisione umana e
non rientra —, `NUMERO_NAO_AUTORITATIVO`, `SEM_SEQUENCIA_HISTORICA`,
`DURABILIDADE_ARQUIVO` e la divergenza nota `ESCAPE_LEITOR`.

Il limite di `D33` della 3C.5.a resta nella sua forma corretta: un rilancio
viene rifiutato **per i documenti il cui contenuto allocato confrontato
cambia**, e basta uno perché l'esecuzione si fermi, dato che un lotto è un solo
commit. Non è un divieto assoluto di cambiare le regole. `D32` resta rinviata:
un risultato copre un'esecuzione, non un mese.

## 13. Approvazione di Luca — 2026-10-31

> «Approvo M1: rifiutare la nuova dichiarazione se non è applicabile,
> spiegandone il motivo. Approvo M2: un bersaglio per comando; approvazione
> multipla rinviata. Confermo M3: la decisione resta valida nelle nuove
> esecuzioni che contengono lo stesso documento con identica impronta; non
> passa a contenuti diversi.»

| # | Decisione | Esito | Come è attuata |
|---|---|---|---|
| M1 | candidato non applicabile | **APPROVATA** | uscita 3 con il motivo fra i cinque del modello, e la frase «questo comando pubblica soltanto l'evento applicabile al bersaglio selezionato». Niente viene pubblicato |
| M2 | un bersaglio per comando | **APPROVATA** | `--chave`, `--conta`, `--ordinal`, `--origem` nominano una riga sola. Nessuna opzione di approvazione multipla |
| M3 | la decisione fra esecuzioni | **CONFERMATA** come comportamento ereditato | `tx75` la misura in entrambi i versi: stessa impronta la ritrova, contenuto diverso **non** la eredita |

## 14. Esito dell'implementazione — 2026-10-31

### 14.1 Che cosa è stato scritto

| File | Contenuto |
|---|---|
| `src/declaracao.rs` (nuovo) | la parte **pura**: costruzione del candidato, identità esatta, ricerca della ripetizione fra gli archiviati, classificazione dell'esito, e le righe della consultazione con i cinque stati separati |
| `src/main.rs` | `consultar-linhas` e `declarar-revisao` |
| `src/lib.rs` | dichiarazione del modulo |
| `tests/cli_revisao_nfe.rs` (nuovo, 12 test) | gli undici casi del percorso, più il deposito del workbook revisionato |
| `script/releitura_export.py` | il modo `percurso-cli-revisado`, con attese **sue**; più due correzioni, §14.4 |
| `script/controprova_rilettura.py` | `--modo percurso-cli-revisado` con le sue copie alterate |
| `README.md`, questo mandato | uso, contratti, limiti |

`revisao.rs` **non** è stato toccato: `validar` resta privata, e la barriera è
quella di `aplicar_revisao`. Invariati anche `arquivo.rs`, `arquivo_io.rs`,
`export.rs`, `export_io.rs`, `xlsx.rs`, `prelivro.rs`, `alocacao.rs`,
`nfe.rs`, `importacao.rs` e le sei fonti canoniche. Nessuna dipendenza nuova.

### 14.2 Il percorso, misurato

| Passo | Osservato |
|---|---|
| consultazione | cinque stati separati, due impronte **intere** di 64 esadecimali, il comando pronto da copiare con i letterali fra apici semplici, e il motivo per cui la riga resta fuori dall'estratto |
| approvazione | `TECNICO_OK -> APROVADO_POR_PESSOA`, revisione 2, generazione conservata, `no_extrato_revisado` da `NAO` a `SIM`, una riga in `decisoes` e una in `extrato_revisado` con `aprovado_por` dichiarato |
| annotazione | applicata **senza** cambiare lo stato: `TECNICO_OK -> TECNICO_OK`, e l'uscita lo dice |
| revoca | `APROVACAO_REVOGADA` e fuori dall'estratto revisionato; senza approvazione da revocare, uscita 3 `NADA_A_REVOGAR` |
| impronta stantia | uscita 3 `VERSAO_SUPERADA`, con le **due** impronte nominate, archivio byte-identico |
| ripetizione esatta | uscita 3 `REPETICAO`; la stessa approvazione a un istante diverso è una dichiarazione distinta e si applica |
| coppia incompatibile | uscita 2, niente pubblicato, archivio ed export precedenti ancora utilizzabili |

### 14.3 Verifiche

| Prova | Esito |
|---|---|
| `cargo test --locked` | **420** verdi: 367 lib, 6 binario, 47 integrazione. I 408 precedenti invariati |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti |
| rilettura, modo `percurso-cli-revisado` | **1117** celle, **2478** confronti, 1 divergenza nota realmente esercitata, 0 discordanze, uscita 0 |
| rilettura, modi `export`, `percurso-cli`, `tx38` | **invariati**: 998 confronti e 5 divergenze note sulla fixture, 0 discordanze, uscita 0 |
| controprova, modo revisionato | otto casi conformi: baseline, senza manifesto, cinque mutazioni — compresa un'**approvazione inventata** su una riga che nessuno ha approvato e l'autore sostituito — e una mascherata |
| controprova, fixture e `percurso-cli` | **invariate**: dodici e sette casi conformi |

### 14.4 Due difetti miei, trovati e corretti

| Difetto | Dove | Correzione |
|---|---|---|
| una registrazione di divergenza nota veniva cercata per **nome di cella** senza filtrare il modo: nel modo nuovo una registrazione del modo `export` veniva dichiarata **obsoleta**, e la rilettura usciva con 1 pur avendo 0 discordanze | `script/releitura_export.py`, `Esito.confronta` | la nota vale solo nel **suo** modo, come già faceva `_valuta_eccezione`. È un difetto latente del lotto 3C.5.a: si vedeva solo quando due modi toccano la stessa cella, e qui è successo |
| nell'uso in testa allo script era rimasto un `\n` **letterale** al posto di un ritorno a capo | `script/releitura_export.py`, docstring | riscritto. Era già nel commit `1af8dc7` della 3C.5.a: cosmetico, ma pubblicato |

### 14.5 Differenze rispetto alla revisione 2

| Punto | Proposta | Fatto |
|---|---|---|
| numero dei test | undici | **dodici**: il dodicesimo è il deposito per la rilettura, che la proposta chiedeva in §9 senza contarlo |
| `--ordinal` e `--instante` ripetuti | non previsto | **misurato**: clap rifiuta un'opzione ripetuta — «the argument '--instante <INSTANTE>' cannot be used multiple times». Rafforza `CLI-08` della 3C.5.a, che era garantito dalla costruzione: ora si sa che è un errore esplicito |

## 15. Una nota sull'ambiente, non sul codice

Durante le verifiche la macchina ha esaurito la memoria impegnabile: `clippy`
non è riuscito nemmeno ad avviarsi, con `exit code 0xc000012d`
(`STATUS_COMMIT_LIMIT`), e `rustc` ha prodotto errori interni (ICE) e metadati
invalidi nel target dir. Misurato: **0,8 GiB** di RAM libera su 7,9, e 5,1 GiB
liberi sul disco.

Con `cargo test -j 1` e `cargo clippy -j 1` tutto compila e passa: il difetto
era la compilazione **parallela**, che su questa macchina supera il limite di
commit. Non è un problema del lotto, ma va saputo: le verifiche di questo
mandato sono state eseguite serializzate.

## 16. Stato Git alla consegna per l'audit

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `src/declaracao.rs` | §14.1, modulo puro della dichiarazione | nuovo, non tracciato, provato | **gruppo codice**, dopo l'audit e l'autorizzazione |
| `tests/cli_revisao_nfe.rs` | §14.1, dodici test | nuovo, non tracciato, verde | gruppo codice |
| `src/main.rs` | i due comandi | modificato | gruppo codice |
| `src/lib.rs` | dichiarazione del modulo | modificato | gruppo codice |
| `script/releitura_export.py` | modo `percurso-cli-revisado` e le due correzioni di §14.4 | modificato | gruppo codice |
| `script/controprova_rilettura.py` | `--modo percurso-cli-revisado` | modificato | gruppo codice |
| `README.md` | uso, contratti, conteggi | modificato | gruppo codice |
| `PROPOSTA_MANDATO_CLAUDE_FASE3C5B_REVISAO_CLI_20261031.md` | questo mandato, §13–§16 | non tracciato | **gruppo documentale** |

Nessun file in indice, nessun commit, nessun push, nessun `add`. Niente reset,
restore, checkout, clean, stash, amend o rebase. `main` è allineato a
`origin/main` su `c27ba60`, e i file della 3C.5.a registrati in quel commit
non sono stati toccati — tranne `script/releitura_export.py`, che questo lotto
estende e corregge, come §14.1 e §14.4 dichiarano.

---

## 17. I due riscontri dell'audit, chiusi (2026-10-31)

Il ciclo ordinario era conforme; i due difetti stavano nella **porta
operativa**: il comando da copiare e la presentazione del conto. Entrambi nel
codice di questo lotto, entrambi miei.

### 17.1 `R3C5B-I01`: il comando generato non si eseguiva

| Voce | Valore |
|---|---|
| Difetto | gli argomenti del comando stampato non erano letterali: con un percorso che contiene spazi — `C:\Users\Recanto do Sossego\…` — PowerShell lo spezzava, e l'esecuzione usciva con 2 e `unexpected argument 'do' found`. Inoltre il comando **non** riportava `--plano-contas`: una consultazione fatta con un piano proprio generava una dichiarazione che avrebbe caricato il predefinito |
| Correzione | `literal_ps`: apici semplici con gli apostrofi interni **raddoppiati**, applicata a tutti gli argomenti testuali — archivio, lotto, chiave, conto, impronte — e `--plano-contas` sempre riportato, con il percorso **selezionato** |
| Che cosa non è cambiato | nessun vincolo nuovo sui percorsi, e l'impronta esaminata resta quella mostrata e dichiarata dall'operatore: non viene riletta dall'archivio |

**La regressione**, `tx81_comando_gerado_executa_com_caminho_ostil`: l'arena ha
un percorso con spazi **e un apostrofo**, il piano è una copia propria con
spazi nel nome, e il comando stampato viene **eseguito** in PowerShell
sostituendo il solo lanciatore `cargo run --` con l'eseguibile già compilato.
Il motivo passato porta un apostrofo, e il test verifica che arrivi
all'archivio come **uno** solo: `conferido: l'original em papel`. Non basta
che l'uscita contenga degli apici, e il test non si accontenta di quello.

Dichiarato `#[cfg(windows)]`: invoca `powershell`, che è dove il difetto
viveva.

### 17.2 `R3C5B-I02`: il nome storico accanto all'impronta attuale

| Voce | Valore |
|---|---|
| Difetto | la consultazione mostrava `lancamento.conta_nome`, cioè il nome del momento dell'allocazione, accanto all'impronta di presentazione **corrente**: due cose di due momenti diversi. E non mostrava gli avvisi che la vista già produce, `APRESENTACAO_MUDADA` compreso, mentre l'export li mostra: consultazione e derivato dicevano cose diverse dello stesso contesto |
| Correzione | la riga porta `conta_nome_plano`, il nome autorevole del piano usato ora; il nome archiviato si stampa **etichettato** e solo se differisce. Gli avvisi della vista e del pre-libro si stampano come la vista li produce, con la loro fonte, senza ricostruire nessuna logica |
| Che cosa non è cambiato | nessuna riscrittura del `Lancamento`, nessun cambio dell'impronta di contenuto, nessuna revoca automatica: `D8.3` resta intatta e le decisioni permangono |

**La regressione**, `tx82_nome_do_plano_atual_e_aviso_visivel`: una copia del
piano **fuori** dal repository, con il solo nome del conto `2.01.004`
cambiato. Misurato: il nome corrente compare, quello archiviato compare
etichettato, l'impronta di contenuto **non** cambia, quella di presentazione
cambia, l'approvazione precedente resta applicata e `APRESENTACAO_MUDADA` è
leggibile nella consultazione. L'export con lo stesso piano non fallisce.

**Una cosa che il test mi ha insegnato, e che vale operativamente.**
`APRESENTACAO_MUDADA` nasce dal confronto fra la presentazione **dichiarata**
nell'evento e quella corrente: se chi approva **non** passa `--vista`, quel
confronto non esiste e l'avviso non può comparire. Il comando generato dalla
consultazione la porta sempre, e per questo conviene usarlo: senza `--vista`
il sistema non potrà dirti che la presentazione è cambiata. Sta nel foglio
`limites` come `AUTOR_NAO_AUTENTICADO` sta accanto all'autore: una garanzia
esiste solo se il dato è dichiarato.

### 17.3 Verifiche dopo le correzioni

| Prova | Esito |
|---|---|
| `cargo test --locked` | **422** verdi: 367 lib, 6 binario, 49 integrazione, fra cui i **quattordici** del percorso di revisione |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` | puliti |
| rilettura, quattro modi | `tx38`, `export`, `percurso-cli`, `percurso-cli-revisado`: tutti uscita 0 |
| controprove, tre modi | fixture, `percurso-cli` e `percurso-cli-revisado`: tutte conformi |
| sei fonti canoniche | invariate. La copia del piano con il conto rinominato vive nell'arena del test, mai nel repository |

Perimetro invariato: `revisao.rs`, lo schema dell'archivio, il writer XLSX, il
nucleo monetario, le dipendenze e le decisioni `M1`–`M3` non sono stati
toccati. Le verifiche sono state eseguite **serializzate**, per il limite di
memoria di §15.

Lo stato Git di §16 non cambia: gli stessi otto file, indice vuoto, nessun
commit e nessun push.

---

## 18. Esito finale dell'audit tecnico (2026-10-31)

**AUDIT TECNICO SUPERATO nel perimetro del lotto 3C.5.b.** `R3C5B-I01` e
`R3C5B-I02` sono chiusi, e nessun riscontro bloccante nuovo è emerso. Il
ciclo consultazione → approvazione → annotazione → revoca → export è
conforme, e la precisazione su `--vista` è coerente con il modello: senza una
presentazione dichiarata manca il dato da confrontare con quella corrente.

### 18.1 Le provenienze delle prove, che non si sommano

| Prova | Chi l'ha eseguita, e quando |
|---|---|
| `cargo test --locked -j 1 --test cli_revisao_nfe` (**14** passati, `tx81` e `tx82` comprese); la sonda indipendente rieseguita su un'arena nuova, con uscita 0 e l'approvazione pubblicata; `cargo fmt --check`; `git diff --check`; sei fonti canoniche 6/6; confronto con la fotografia precedente | **Codex**, nel controllo finale |
| riletture dei tre modi del workbook — `percurso-cli-revisado` 1117 celle e 2478 confronti, `percurso-cli` 1504 e 3291, `export` 411 e 998 — e le relative controprove | **Codex**, nel controllo precedente: prove **riusate**, non rieseguite, perché gli script non sono cambiati |
| suite completa di **422** test, `clippy`, rilettura `tx38`, riesecuzione completa degli script | **Claude**: attestazioni della consegna, **non** una seconda suite indipendente |

Codex ha anche verificato che `src/lib.rs` e i **due script Python** sono
byte-identici al primo audit: le correzioni hanno toccato soltanto
`declaracao.rs`, `main.rs`, i test e la documentazione.

### 18.2 La fotografia, confrontata

Prima di scrivere questa appendice ho confrontato gli **otto** file con
`file_audit_finale_sha256.json` del controllo finale: **8 su 8 coincidono**,
mandato compreso.

Da questo momento il codice **non cambia**: questa appendice modifica
legittimamente il solo mandato, e i sette file di codice, script e README
restano quelli fotografati. Qualunque modifica di codice successiva andrebbe
dichiarata e ricontrollata.

### 18.3 Che cosa questo esito non dichiara

| Voce | Stato |
|---|---|
| registrazione Git e pubblicazione | **non eseguite e non autorizzate** da nessuno degli audit: restano a Luca |
| `TX-18` prova visiva in Excel | **parziale** |
| `TX-19` decodifica di `_xHHHH_` in Excel | **NON_ESEGUITA** |
| fase 3C | **non chiusa**. «Audit tecnico superato» non equivale a «fase chiusa in Git» |
| `D32`, approvazione multipla (`M2`), versionamento dei documenti, rientro dal workbook, DRE, import dello storico, promozione contabile | fuori, invariati |

Le decisioni restano quelle di §13: `M1` rifiuto del candidato non applicabile,
`M2` un bersaglio per comando, `M3` la decisione che segue il documento con la
stessa impronta. Il limite di memoria della macchina (§15) resta: le verifiche
vanno eseguite serializzate.

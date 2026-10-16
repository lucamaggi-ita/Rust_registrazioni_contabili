# Specifica fase 3A — registrazioni contabili (`Lancamento`)

Documento di analisi e specifica per la **futura fase 3B**: trasformare una NF-e
già letta e classificata in registrazioni `Lancamento` con riconciliazione.

| Voce | Valore |
|---|---|
| Data | 2026-10-16 |
| Mandato | `MANDATO_CLAUDE_FASE3A_CONTRATTI_20261016.md` |
| Baseline Rust | `3a24ba0f1abb3b71e4bed650bf246c72edb1a485`, suite 113 test (97 lib + 3 bin + 13 CLI), **riusata, non rieseguita**: nessun file di codice è stato toccato |
| Fonte Python canonica | `lucamaggi-ita/Programma_registrazioni-contabili` @ `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` (2026-05-06, "Corrige findings pos-auditoria da Fase 11A") |
| Stato del documento | **revisione 4**: audit tecnico superato (riscontri R3A-01..06, P3, R2-01..03 recepiti) e **le sei decisioni §11 approvate da Luca il 2026-10-16** nella variante (a), con D6 in centesimi. Limite dichiarato: `pytest` originale non eseguito (vedi §2.3) |
| Stato della 3B | **implementata il 2026-10-16** in `src/alocacao.rs` (contratti definitivi `C18-C24` nel README) e **audit tecnico superato** lo stesso giorno: R3B-01, R3B-02 e R3B-03 chiusi dopo tre giri di audit indipendente. Suite **166 test verdi**. Resta solo la **registrazione in Git, da autorizzare**; commit e push non autorizzati |
| Validità di questo documento | resta il documento di **analisi**: gli `F3A-nn` sono gli identificativi di analisi, i contratti implementati vivono nel README come `C18-C24`, con la corrispondenza dichiarata lì |

**Questo documento non implementa niente.** Nessun file in `src/`, `tests/`,
`config/`, nessun comando, nessuna dipendenza, nessun commit sono stati
toccati. Gli identificativi `F3A-nn` sono provvisori: **non** sono contratti
`C18+` implementati, e nessun test qui elencato esiste ancora.

---

## 1. Flusso previsto e perimetro

```text
XML NF-e ──(fase 2, esistente)──> NfeImportada { nfe, avisos }
                                        │
config YAML validata ──> Classificador ─┤
                                        ▼
                         [FASE 3B, da implementare]
                         allocazione per conto + differenza di totale
                         + riparto delle rate + riconciliazione
                                        ▼
                    AlocacaoNfe { lancamentos, riconciliazione, avisos }
                                        │
                     ┌──────────────────┴───────────────────┐
                     ▼                                      ▼
            [3C] pre-libro, revisione,            [3D] aggregazione DRE
                 esportazione                          per competenza
```

Perimetro della 3B: **dalla NF-e letta alle registrazioni in memoria, con
riconciliazione**. Niente scrittura su database, niente Excel, niente DRE,
niente CSV. Confine dettagliato in §7.

---

## 2. Fonti lette e prove

### 2.1 Rust (sola lettura)

| File | Cosa ne viene usato |
|---|---|
[src/modelos.rs](src/modelos.rs) | `Dado<T>` (128-243), `StatusDado` (27-83), `TipoConta` (267-301), `Conta` (302-314), `Competencia` (356-430), `Duplicata` (432-440), `ProdutoNfe` (441-458), `Nfe` (459-496), `Confianca` (497-507), `ResultadoClassificacao` (508-518), `OrigemLancamento` (519-528), `StatusRevisao` (529-539), `Lancamento` (540-575) |
[src/nfe.rs](src/nfe.rs) | dati realmente disponibili dopo la lettura, avvisi, contratti C10/C14/C17 |
[src/classificacao.rs](src/classificacao.rs) | `Classificador::classificar(descricao, plano) -> Result<ResultadoClassificacao, ErroClassificacao>` (151) |
[src/regras.rs](src/regras.rs) | validazione della configurazione, `PlanoContas`, avvisi C13 |
[README.md](README.md), [config/FONTE.md](config/FONTE.md) | contratti vigenti, provenienza, rischi noti |
`config/plano_contas_gerencial.yaml` | 125 conti, 11 centri di costo; esistenza di `2.05.001`, `2.05.002`, `13.01.*` |
`config/mapeamento_dre.yaml` | 26 linee DRE, 105 mappature conto→linea |

**Fatto verificato:** nessuna logica Rust costruisce oggi un `Lancamento`.
`grep -rn "Lancamento" src tests` fuori da `modelos.rs` non restituisce nulla;
l'unico uso è il test di round-trip `modelos.rs:753`. La fase 3B parte da zero
sul lato trasformazione.

### 2.2 Python canonico, a commit fissato

Letti con `git show 5b70f2b:<path>`, **senza** checkout né modifiche nel
progetto Python:

| File | Righe rilevanti | Contenuto |
|---|---|---|
| `src/fantozzi/allocation.py` | 1-337 (intero) | ingresso `allocate_nfe_to_lancamentos` (32-60), raggruppamento (63-86), alert di classificazione (89-95), costruzione (98-128), differenza di totale (131-178), riparto rate (228-302), hash (305-313), arrotondamento (316-317), dedup avvisi (320-337) |
| `src/fantozzi/models.py` | 9-99 | `Conta`, `Duplicata`, `ProdutoNFE`, `NFE`, `ClassificationResult`, `Lancamento` |
| `tests/test_allocation.py` | 1-237 | attese sul campione canonico e due casi sintetici (accessori, sconto) |
| `src/fantozzi/nfe_importer.py` | 66-144 | totali, `valor_tributos_inclusos` come somma di 5 tag, `valor_cobranca` con default zero |
| `src/fantozzi/pre_livro_diario.py` | 21, 78-213, 257 | il pre-libro **legge i lancamentos da un Excel**, non da oggetti: confine 3C |
| `src/fantozzi/dre_gerencial_provisoria.py` | 246 | `dre_linha_id` viene letto dal mapping **in questa fase**, non in allocazione |
| `src/fantozzi/excel_diagnostics.py` | 27, 111-136 | i duplicati di `hash_registro` sono diagnosticati a valle, non bloccati in allocazione |
| `src/fantozzi/excel_writer.py` | 37-45 | `hash_registro`, `dre_linha_*`, `data_pagamento` sono colonne dell'export |
| `tests/test_fase3_end_to_end.py` | 7-42 | ordine reale della pipeline: `parse_nfe` → `allocate_nfe_to_lancamentos` → `write_lancamentos_to_excel` |

Ricerca negativa, importante: `dre_linha`, `data_pagamento` e `centro_custo`
**non compaiono in `allocation.py`**. La ricerca è stata fatta su tutti i
`src/fantozzi/*.py` del commit fissato.

### 2.3 Prove eseguite e non eseguite

| Prova | Stato | Dettaglio |
|---|---|---|
| Lettura della fonte Python a commit fissato | ESEGUITA | `git show`, nessuna scrittura |
| `pytest` sui test Python | **NON_ESEGUITO** | `python` non è nel `PATH` di questa sessione; un runtime Python 3.12.14 **esiste** nell'ambiente dell'applicazione, ma **non ha `pytest` né `PyYAML`** (verificato dall'audit del 2026-10-16), e il mandato vieta di installare dipendenze. Le attese Python citate qui sono **lette dal sorgente dei test**; l'audit le ha poi confermate eseguendo le funzioni originali con un classificatore controllato — **non** è l'esecuzione della suite originale |
| Replica numerica indipendente del riparto | ESEGUITA | probe in `[decimal]` .NET, PowerShell 7.6.7, `MidpointAwayFromZero` (= `ROUND_HALF_UP`); nessun `f64`. Script fuori dal repository: `C:\Users\RECANT~1\AppData\Local\Temp\claude\c--Users-Recanto-do-Sossego-Documents-Luca-GitHub-RUST-Rust-registrazioni-contabili\1898eaaf-4410-4449-8413-33f6928a9c29\scratchpad\probe_riparto_3a.ps1` |
| Validità della replica | **CONFERMATA** | sul campione canonico la probe produce 5.00/10.00/2.50 per ciascuna delle due rate, cioè esattamente le attese di `test_allocation.py:70-89`. La replica riproduce la fonte senza eseguirla |
| Suite Rust | RIUSATA | 113 test della baseline `3a24ba0`; nessun file di codice modificato, quindi non rieseguita |
| Strategia di arrotondamento di `rust_decimal` | VERIFICATA sul sorgente | crate vendorizzato 1.43.0, `src/decimal.rs:1654`: `round_dp` usa `MidpointNearestEven` (Bankers); `MidpointAwayFromZero` esiste (`:153`) e va richiesto esplicitamente |
| Audit indipendente del 2026-10-16 | ESEGUITO da Codex | ha rieseguito la probe PowerShell, verificato gli 11 blob Python contro gli SHA Git, eseguito le **funzioni originali** di `allocation.py` con classificatore controllato (casi A/B/C/E/F coincidenti), e prodotto i controesempi §5.8 (dominio monetario) e §6.2 (payload ripetuto). Ha inoltre verificato sulla CLI corrente che chiave, numero e CNPJ emittente assenti **non** bloccano, che un `<det>` senza `<prod>` viene saltato e che `vProd` a tre decimali è accettato |
| Precondizioni di identità della fase 2 | VERIFICATE da me sul codice dopo l'audit | `nfe.rs:432` (`CHAVE_AUSENTE` è un avviso), `:459` (`CAMPO_IDENTIDADE_VAZIO`), `:663` (`nNF` senza controllo), `analisar_decimal` = `from_str_exact` senza vincolo di scala, `chart_of_accounts.py:43-46` (`KeyError`) |

---

## 3. A — Granularità, identità e carta d'identità dei campi

### 3.1 Granularità

**Una registrazione per (documento, conto)**, non per articolo. Gli articoli
sono classificati uno a uno, poi raggruppati per `conta_id`
(`allocation.py:63-86`); la prova è `test_allocation.py:42-48`: il campione
canonico con 3 articoli su 3 conti diversi produce 3 registrazioni.

A queste si aggiunge **al massimo una** registrazione di differenza di totale
(`allocation.py:131-169`), su `2.05.001` (accessori) oppure `2.05.002`
(sconti), mai entrambe: il segno della differenza ne determina una sola.

Quindi: `numero di registrazioni = conti distinti fra gli articoli + (0 o 1)`.

### 3.2 Identità

| Entità | Campi | Fonte | Nota |
|---|---|---|---|
| Azienda (acquirente) | `cnpj_empresa`, `nome_empresa` | `nfe.cnpj_destinatario`, `nfe.nome_destinatario` | il Python accetta CNPJ **o** CPF nel destinatario (`nfe_importer.py:63`) |
| Fornitore | `cnpj_cpf_favorecido`, `fornecedor_ou_favorecido` | `nfe.cnpj_emitente`, `nfe.nome_emitente` | |
| Documento | `documento` = `nfe.numero`; `chave_origem` = `nfe.chave` (44 cifre) | NF-e | la **chiave** è l'identità forte; `numero` da solo non è unico fra fornitori |
| Riga | `hash_registro` | sha256 di 4 campi, §6.2 | non include la serie né la competenza |
| File | `arquivo_origem` | percorso dell'XML letto | dipende dal percorso locale: non è identità stabile fra macchine |

### 3.3 Contratto campo per campo di `Lancamento`

Legenda stato: **LETTO** = copiato dalla fonte; **CALCOLATO** = derivato da una
formula dichiarata; **COSTANTE** = valore fisso del contratto; **VUOTO_3B** =
non popolato dalla 3B, con motivo.

| Campo (`modelos.rs`) | Fonte | Trasformazione | Assenza alla fonte | Stato 3B |
|---|---|---|---|---|
| `data_documento: NaiveDate` | `nfe.data_emissao` | copia | impossibile: la fase 2 fallisce prima (campo obbligatorio) | LETTO |
| `competencia: Competencia` | `nfe.data_emissao` | `Competencia::from_data` → anno/mese | come sopra | CALCOLATO |
| `cnpj_empresa` | `nfe.cnpj_destinatario` | copia | stringa vuota possibile (né CNPJ né CPF): avviso `CAMPO_IDENTIDADE_VAZIO` in fase 2, **non** errore → §3.4 | LETTO |
| `nome_empresa` | `nfe.nome_destinatario` | copia | come sopra → §3.4 | LETTO |
| `fornecedor_ou_favorecido` | `nfe.nome_emitente` | copia | come sopra → §3.4 | LETTO |
| `cnpj_cpf_favorecido` | `nfe.cnpj_emitente` | copia | come sopra → §3.4 | LETTO |
| `documento` | `nfe.numero` | copia | **nessun controllo in fase 2**: `nfe.rs:663` legge `nNF` e accetta la stringa vuota **senza avviso** → policy 3B in §3.4 | LETTO |
| `tipo_documento: String` | — | `"XML_NFE"` | — | COSTANTE |
| `conta_id`, `conta_nome`, `grupo_conta` | `ResultadoClassificacao::conta` | copia da `Conta` | conto inesistente = errore di configurazione, già bloccato da C6 | LETTO |
| `descricao_itens` | articoli del gruppo | `"{descricao} ({valor:.2})"` uniti da `"; "` | articolo senza descrizione: stringa vuota fra parentesi → **F3A-24** | CALCOLATO |
| `valor: Decimal` | `vProd` degli articoli del gruppo | somma, poi arrotondamento a 2 decimali | nessuna: `vProd` è obbligatorio (C10) | CALCOLATO |
| `duplicatas: Vec<Duplicata>` | `nfe.duplicatas` | riparto della matrice §5.3 | nessuna rata → vettore vuoto, **non** rate a zero | CALCOLATO |
| `origem: OrigemLancamento` | — | `XmlNfe` | — | COSTANTE |
| `arquivo_origem` | `nfe.arquivo_origem` | copia | — | LETTO |
| `chave_origem` | `nfe.chave` | copia | **non bloccante**: senza `Id` la fase 2 emette l'avviso `CHAVE_AUSENTE` (`nfe.rs:432`) e **prosegue** con chiave vuota → policy 3B in §3.4 | LETTO |
| `hash_registro` | 4 campi | sha256, §6.2 | — | CALCOLATO |
| `status_revisao: StatusRevisao` | classificazione + avvisi | §5.5 | — | CALCOLATO |
| `alertas: String` | avvisi del gruppo e del documento | uniti da `"; "`, deduplicati in ordine | nessun avviso → stringa vuota (qui il vuoto **significa** "nessuno") | CALCOLATO |
| `observacoes: String` | — | nessuna fonte NF-e | — | **VUOTO_3B** (campo per annotazione umana, 3C) |
| `centro_custo: String` | — | i 11 centri di costo non sono collegati ai conti: `Conta` non ha il campo | — | **VUOTO_3B**, motivo dichiarato: nessuna fonte di collegamento conto→centro |
| `tipo_lancamento: TipoConta` | `Conta::tipo` | copia | — | LETTO |
| `dre_linha_id`, `dre_linha_nome` | `mapeamento_dre.yaml` | **non popolati in 3B** (§7) | 20 conti su 125 non hanno mappatura | **VUOTO_3B** |
| `data_pagamento: Dado<NaiveDate>` | nessuna | le rate sono **scadenze**, non pagamenti | `Dado::ausente` con stato e motivo, §5.6 | ASSENTE dichiarato |

### 3.4 Identificativi: che cosa la fase 2 garantisce davvero

**Correzione del riscontro R3A-01.** La revisione 1 di questo documento
affermava che chiave, numero e CNPJ emittente assenti fossero già errori di
fase 2. **È falso.** C10 protegge gli **importi** obbligatori, non i campi di
identità. Verificato sul codice corrente:

| Campo | Letto da | Se assente/vuoto | Esito CLI | Prova |
|---|---|---|---|---|
| `chave` | attributo `Id` di `infNFe` | avviso `CHAVE_AUSENTE`, chiave vuota | **exit 0** | `nfe.rs:432`; fixture d'audit `sem_chave.xml` |
| `numero` | `ide/nNF` | **nessun controllo, nessun avviso**, stringa vuota | **exit 0** | `nfe.rs:663` |
| `serie` | `ide/serie` | come `numero` | **exit 0** | `nfe.rs:664` |
| `cnpj_emitente` | `emit/CNPJ` | avviso `CAMPO_IDENTIDADE_VAZIO` | **exit 0** | `nfe.rs:459`; fixture `sem_cnpj_emitente.xml` |
| `nome_emitente` | `emit/xNome` | avviso `CAMPO_IDENTIDADE_VAZIO` | **exit 0** | `nfe.rs:459` |
| `cnpj_destinatario` | `dest/CNPJ`, poi `dest/CPF` | avviso `CAMPO_IDENTIDADE_VAZIO` | **exit 0** | `nfe.rs:444-459` |
| `nome_destinatario` | `dest/xNome` | avviso `CAMPO_IDENTIDADE_VAZIO` | **exit 0** | `nfe.rs:459` |
| `ide`, `emit`, `dest`, `total`, `ICMSTot` (blocchi) | `obrigatorio(...)` | **errore**, nessuna NF-e | exit 1 | `nfe.rs:435-437,467-468` |
| `dhEmi`, `vNF`, `vProd` e altri importi di layout | `analisar_data`, `decimal_obrigatorio` | **errore** con il percorso del campo (C10) | exit 1 | `nfe.rs:440,470-471` |

L'allocatore può quindi ricevere documenti con identità incomplete. Policy 3B
proposta, **per identificativo**, senza riaprire il parser:

| Identificativo | Perché serve alla 3B | Policy proposta | Avviso | Prova da creare |
|---|---|---|---|---|
| `chave` | entra nel payload dell'hash: vuota ⇒ hash non identifica il documento | **errore `IDENTIDADE_INSUFICIENTE`**: senza chiave non si produce una registrazione tracciabile | — | `chave_vazia_nao_produz_lancamento` |
| `numero` | compare in `documento`, usato nella revisione umana | registrazione prodotta, `REVISAR` + avviso | `DOCUMENTO_SEM_NUMERO` | `numero_vazio_marca_revisar` |
| `serie` | non usata dall'allocazione | nessun effetto, nessun avviso nuovo | — | — |
| `cnpj_emitente`, `nome_emitente` | identificano il fornitore nel libro | registrazione prodotta, `REVISAR` + avviso | `FORNECEDOR_SEM_IDENTIDADE` | `fornecedor_sem_identidade_marca_revisar` |
| `cnpj_destinatario`, `nome_destinatario` | identificano l'azienda | registrazione prodotta, `REVISAR` + avviso | `EMPRESA_SEM_IDENTIDADE` | `empresa_sem_identidade_marca_revisar` |

Asimmetria voluta e dichiarata: solo la **chiave** blocca, perché è l'unico di
questi campi da cui dipende l'identità della riga (§6.2). Gli altri producono
registrazioni esplicitamente da revisionare: bloccare l'import per un nome
vuoto sarebbe sproporzionato, lasciarlo passare in silenzio inaccettabile.
Alternativa non scelta: trattare tutti come errori, che renderebbe non
importabile una NF-e altrimenti valida. Decisione **D5**, §11.

### 3.5 Quello che il modello attuale non rappresenta bene

Proposte di evoluzione, **da non implementare in questa fase**:

| Limite | Effetto concreto | Proposta minima |
|---|---|---|
| `Duplicata::valor: Decimal` | con `vNF = 0` il Python scrive rate da `0.00`: uno zero che non è un valore misurato, in conflitto con C2/C10 | `Dado<Decimal>` per il valore ripartito, oppure rifiutare il riparto (decisione **D1**) |
| `Lancamento::alertas: String` | gli avvisi sono una stringa unita: a valle vanno riseparati con `split(';')`, come fa `allocation.py:320-323` | `Vec<String>`, con la stringa prodotta solo in esportazione |
| `dre_linha_id`/`dre_linha_nome: String` | una stringa vuota non distingue "non ancora assegnato" da "nessuna linea applicabile" | `Dado<String>` o `Option`, assegnati in 3D |
| `Dado<T>` non modella la **copertura** | un aggregato `Parcial` dice che manca qualcosa, non *quanto* manca (rischio noto 5 del README) | campo `cobertura { attesi, coperti, esclusi }`, §5.4 |
| `centro_custo: String` | nessuna fonte lo popola: resta vuoto per sempre senza che si veda il perché | `Dado<String>` con stato `NaoDisponivel` e motivo |

---

## 4. Dati realmente disponibili e loro natura

| Dato | Tipo Rust oggi | Natura |
|---|---|---|
| `vProd` per articolo (`ProdutoNfe::valor_total`) | `Decimal` | **letto**, obbligatorio |
| `vNF` (`Nfe::valor_total`) | `Decimal` | **letto**, obbligatorio (C10: assente = errore) |
| `vFrete`, `vSeg`, `vDesc`, `vOutro` | `Decimal` | **letti**; zero significa zero dichiarato |
| `valor_tributos_inclusos` | `Dado<Decimal>` | **aggregato**, può essere `Parcial` (`vFCPST`/`vIPIDevol` condizionali) |
| `valor_cobranca` | `Dado<Decimal>` | **letto o assente** in 4 stati distinti (C10); **non** usato in allocazione, vedi §5.7 |
| rate (`Duplicata`) | `Vec<Duplicata>`, `vencimento: Option` | **lette**; scadenza assente non si inventa |
| conto, confidenza, match | `ResultadoClassificacao` | **calcolati** dalla classificazione |
| avvisi del parser | `Vec<Aviso>` in `NfeImportada` | **letti**, da propagare (§5.5) |

---

## 5. B e C — Trasformazioni, importi, copertura e revisione

### 5.1 Raggruppamento per conto

Formula (`allocation.py:63-86`):

- per ogni articolo, nell'ordine del documento: `conta = classificar(descricao).conta`;
- primo articolo di un conto → crea il gruppo, nell'**ordine di prima comparsa**;
- `gruppo.valor += articolo.valor_total` (nessun arrotondamento intermedio);
- gli avvisi di classificazione si accumulano nel gruppo.

L'ordine è `OrderedDict` per inserimento: **deterministico** e dipendente
dall'ordine degli articoli nell'XML. In Rust non serve una nuova dipendenza:
basta un `Vec<Gruppo>` con ricerca lineare per `conta_id` (i conti per
documento sono pochi), così l'ordine resta quello di comparsa.

### 5.2 Differenza di totale: accessori e sconti

```text
diferenca = round2(vNF - Σ vProd(articoli))

diferenca == 0  → nessuna registrazione aggiuntiva
diferenca  > 0  → registrazione su 2.05.001 "Fretes, Seguros e Outras Despesas sobre Compras", valore = diferenca
diferenca  < 0  → registrazione su 2.05.002 "Descontos Obtidos sobre Compras",   valore = diferenca (negativo)
```

Verifica di spiegazione (`allocation.py:140`, `:156`, `:172-178`):

```text
diferenca > 0 : spiegata  ⇔  diferenca == round2(vFrete + vSeg + vOutro + tributos_inclusos)
diferenca < 0 : spiegata  ⇔  |diferenca| == round2(vDesc)

spiegata     → status_revisao = OK,     alertas = ""
non spiegata → status_revisao = REVISAR, alertas = "VALOR_INCONSISTENTE"
```

**Doppio conteggio.** Il rischio non è teorico: `vFrete`, `vSeg`, `vOutro` e i
tributi inclusi **sono già dentro `vNF`**. Per questo l'unica registrazione
aggiuntiva è la *differenza* `vNF − Σ vProd`, mai la somma delle componenti.
Sommare le componenti **e** la differenza conterebbe due volte. Le componenti
servono solo come **verifica** della differenza, non come importo da registrare.
Invariante che lo protegge: F3A-10.

**Qui c'è un conflitto con C10 (decisione D2).** In Rust
`valor_tributos_inclusos` è un `Dado` che può essere `Parcial`. Il Python somma
`vFCPST`/`vIPIDevol` assenti come zero (`nfe_importer.py:76-84`) e quindi può
dichiarare "spiegata" una differenza che in realtà **non è verificabile**. La
probe lo mostra: con `vNF 112.00`, `vProd 100.00`, `vFrete 10.00`, tributi
`2.00` ma parziali, il Python conclude `spiegato = True`, `status = OK`.

### 5.3 Riparto delle rate sulle registrazioni

Denominatore: `totale = round2(vNF)`. **Non** la somma delle rate, **non** il
valore di fatturazione: C10 resta vigente e l'importo di fatturazione non viene
ricostruito dalle rate.

Matrice `rate × registrazioni` (`allocation.py:273-302`), dove `R` = ultima
riga e `C` = ultima colonna:

```text
cella(r,c) non in R, non in C  =  round2( rata(r) * valore(c) / totale )
cella(r,c) non in C, r in R    =  valore(c) − Σ_{r' < r} cella(r',c)        (resto di colonna)
cella(r,c) c in C              =  rata(r)  − Σ_{c' < c} cella(r,c')         (resto di riga)
```

Il **residuo di arrotondamento** non viene distribuito: finisce nell'ultima
colonna di ogni riga e nell'ultima riga di ogni colonna. Conseguenza voluta:
sia i totali per scadenza sia i totali per registrazione chiudono esattamente
(prova nel caso B di §8).

Precisione: `Decimal` elimina l'errore **binario**, non rende esatta ogni
divisione: `rata * valore / totale` può essere periodica e viene calcolata con
precisione e contesto finiti (in Python il contesto `decimal`, in Rust i 96 bit
di mantissa di `rust_decimal`). Il risultato della divisione è quindi
**approssimato prima** dell'arrotondamento dichiarato. L'arrotondamento a 2
decimali avviene **su ogni cella**, con `ROUND_HALF_UP`. In Rust questo **non**
è il comportamento di default: `round_dp` di `rust_decimal` 1.43.0 usa Bankers
(`decimal.rs:1654`). Serve `round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)` → F3A-21.

Denominatore zero (`allocation.py:237-243`): il Python assegna a **ogni**
registrazione rate di `0.00`, salta la divisione e **ritorna prima** dei
controlli per riga e per colonna. Quello `0.00` non è sempre uno zero tecnico e
non è sempre un errore: le tre situazioni (nessuna rata, tutto realmente zero,
riparto non definito) sono distinte in **D1**, e solo la terza è un problema.
Il ramo è la causa comune dei casi §8D e §8M.

Denominatore incompleto: non esiste in questa trasformazione. `vNF` è
obbligatorio (C10) e la fase 2 fallisce prima nominando il campo; non serve un
fallback.

### 5.4 Copertura e riconciliazione del documento

**Correzione del riscontro R3A-02.** La revisione 1 definiva
`articoli_attesi` come "numero di `<det>` del documento". **Non è
ottenibile**: `NfeImportada` espone solo `nfe` e `avisos`, e `nfe.produtos`
contiene gli articoli **conservati** dal parser. Con un `<det>` senza `<prod>`
il parser lo salta e registra `DET_SEM_PROD` (`nfe.rs:531`): il conteggio
originale non arriva all'allocatore. Usare `produtos.len()` per entrambi i
conteggi farebbe sembrare completa una copertura incompleta; rileggere
`arquivo_origem` romperebbe la purezza della funzione e non funziona per input
da testo.

Rappresentazione scelta: **nessun conteggio inventato**. L'atteso è un `Dado`
che resta assente, e la copertura si deduce dagli **avvisi dichiarati** che la
fase 2 già consegna nello stesso oggetto.

```text
Riconciliazione {
  // --- esito del totale
  valor_total_nfe     : Decimal         // vNF letto
  soma_lancamentos    : Decimal         // Σ valore delle registrazioni
  esito_total         : OK | ERRO

  // --- esito delle rate (indipendente dal totale)
  soma_rate_documento : Dado<Decimal>   // NaoAplicavel se la NF-e non ha rate
  soma_rate_allocate  : Dado<Decimal>   // NaoAplicavel se la NF-e non ha rate
  esito_rate          : OK | ERRO | NaoAplicavel | NON_VERIFICATO

  // --- esito della copertura degli articoli
  articoli_allocati   : usize           // conteggio reale, sempre noto
  articoli_attesi     : Dado<usize>     // NON_DISPONIVEL: il parser non lo espone
  articoli_scartati   : Vec<Aviso>      // DET_SEM_PROD e simili, presi da NfeImportada
  esito_copertura     : OK | PARCIAL | NON_VERIFICATO

  // --- componenti del totale, con stato per ciascuna
  componenti          : Vec<Componente { nome, valor: Dado<Decimal>, ruolo }>
  esito_componenti    : OK | PARCIAL     // PARCIAL se una componente usata in
                                         // una verifica non ha status Ok

  // --- aggregazione dei quattro esiti, regola unica sotto
  stato_documento     : OK | PARCIAL | ERRO

  // --- campi di Lancamento non popolati dalla 3B, resi interrogabili
  campi_rinviati      : Vec<CampoRinviato { campo, status, fonte, motivo }>

  avisos_documento    : Vec<Aviso>      // propagati dal parser + prodotti qui
}
```

#### Tabella degli esiti, per condizione

**Correzione del riscontro R2-02.** Ogni esito si calcola **solo** dalla
propria condizione: nessuna riga assegna a un esito la conseguenza di un'altra
dimensione. Sono **quattro**, non tre: le componenti del totale hanno il
proprio esito, altrimenti D2 non sarebbe rappresentabile.

```text
esito_total      = OK   se round2(Σ valore registrazioni) == round2(vNF)
                 = ERRO altrimenti

esito_rate       = NaoAplicavel  se il documento non ha rate
                 = OK    se Σ rate documento == round2(vNF)
                         e Σ rate di ogni registrazione == suo valore
                         e Σ colonne di ogni rata == rata
                 = ERRO  se una qualunque di quelle tre uguaglianze non vale
                 = NON_VERIFICATO se il riparto non è definito (D1, caso 3)

esito_copertura  = PARCIAL se esiste un avviso di scarto articoli
                           (DET_SEM_PROD, SOMA_ITENS_DIVERGE) o se non ci sono
                           registrazioni
                 = OK      altrimenti, nel limite dichiarato sotto

esito_componenti = PARCIAL se una componente necessaria a una verifica usata
                           (oggi: valor_tributos_inclusos nella spiegazione
                           della differenza, §5.2) ha Dado con status != Ok
                 = OK      altrimenti
```

| Condizione | Effetto, e solo quello |
|---|---|
| Σ registrazioni ≠ `vNF` | `esito_total = ERRO` + `TOTAL_NAO_RECONCILIADO` |
| nessuna rata nel documento | `esito_rate = NaoAplicavel` |
| Σ rate documento ≠ `vNF` (§8C) | `esito_rate = ERRO` + `VALOR_INCONSISTENTE`. `esito_total` **non cambia**: nel caso §8C resta `OK` perché le registrazioni sommano 35.00 |
| una registrazione riceve rate che non sommano al suo valore | `esito_rate = ERRO` + `VALOR_INCONSISTENTE`; `esito_total` calcolato a parte |
| `vNF = 0` con rate non nulle | riparto non definito: `esito_rate = ERRO` o rifiuto secondo **D1** caso 3. `esito_total` resta la sua uguaglianza: con registrazioni che sommano 0.00 è `OK` |
| `vNF = 0`, registrazioni **e** rate tutte `0.00` | caso banale: `esito_total = OK`, `esito_rate = OK` (**D1** caso 2) |
| `vNF = 0` con registrazioni non nulle che si compensano (§8M) | `esito_total = OK` (0.00 = 0.00), **`esito_rate` non verificato**: vedi **D1** caso 3 |
| almeno un `DET_SEM_PROD`/`SOMA_ITENS_DIVERGE` | `esito_copertura = PARCIAL` + avviso del parser propagato |
| nessun avviso di scarto | `esito_copertura = OK`, nel limite dichiarato sotto |
| documento senza registrazioni allocabili | `esito_copertura = PARCIAL` + `DOCUMENTO_SEM_ITENS`. `esito_total` ed `esito_rate` restano le loro uguaglianze: con Σ registrazioni = 0.00 e `vNF` > 0 il totale è `ERRO` **per il suo conto**, e le rate possono esistere |
| componente necessaria `Parcial` (§5.2) | `esito_componenti = PARCIAL` + `AGREGADO_PARCIAL`; se si sceglie **D2(a)**, la registrazione di differenza esce `REVISAR` |

#### Regola di aggregazione, unica e deterministica

```text
stato_documento =
  ERRO     se  esito_total == ERRO  oppure esito_rate == ERRO
  PARCIAL  altrimenti, se  esito_copertura == PARCIAL
                        oppure esito_componenti == PARCIAL
                        oppure esito_rate == NON_VERIFICATO
  OK       altrimenti
```

Precedenza: `ERRO` > `PARCIAL` > `OK`. `NaoAplicavel` nelle rate **non**
declassa niente: un documento senza rate può essere `OK`.

Conseguenza voluta, che allinea §5.4 a **D2(a)**, F3A-07 e all'esempio §8G riga
4: una NF-e con totali e rate che chiudono ma con `valor_tributos_inclusos`
`Parcial` esce **`stato_documento = PARCIAL`**, con `esito_total = OK` ed
`esito_rate = OK`. Nella revisione 2 quella NF-e sarebbe uscita `OK`: era
un'incoerenza fra §5.4 e D2. Se si scegliesse **D2(b)** (parità col Python),
`esito_componenti` resta `OK` e il documento esce `OK`: la regola è la stessa,
cambia solo l'ingresso.

Test pianificato corrispondente (da creare, non esistente):
`documento_parcial_com_componente_necessaria_parcial` — verifica
contemporaneamente `esito_total = OK`, `esito_rate = OK`,
`esito_copertura = OK`, `esito_componenti = PARCIAL`,
`stato_documento = PARCIAL`: i quattro esiti restano distinti e leggibili.

#### Limite dichiarato della copertura

`esito_copertura = OK` significa "**nessun avviso di scarto**", non "tutti i
`<det>` del documento sono stati allocati": l'atteso resta `NON_DISPONIVEL`
perché la fase 2 non lo espone. Non è una percentuale e non è un numero
hardcoded. Per renderlo verificabile davvero servirebbe una **evoluzione
dichiarata**, oggi fuori perimetro: `NfeImportada` dovrebbe portare
`det_letti: usize` accanto a `produtos`. Proposta registrata in §3.5, non
implementata.

#### Campi rinviati, invece di stringhe vuote

Il criterio 5 del blocco 3B vieta vuoti al posto delle assenze, ma
`Lancamento::dre_linha_id`, `dre_linha_nome`, `centro_custo` e `observacoes`
sono `String` e la 3B non può cambiare il modello. Senza una modifica di
modello, l'unico modo di rispettare il criterio è rendere l'assenza
**interrogabile nell'output** invece che nascosta nella stringa vuota:

| Campo | `status` | `fonte` | `motivo` |
|---|---|---|---|
| `dre_linha_id`, `dre_linha_nome` | `NAO_ASSEGNATO` (**stato proprio di `CampoRinviato`**, non una variante di `StatusDado`: oggi `StatusDado` non lo contiene. In implementazione va definito lì o tradotto in uno stato esistente con motivo; non è un motivo per riaprire il modello ora) | `mapeamento_dre.yaml` (non letto in 3B) | assegnazione della linea DRE rinviata alla 3D |
| `centro_custo` | `NAO_DISPONIVEL` | — | nessuna fonte collega conto e centro di costo |
| `observacoes` | `NAO_APLICAVEL` | — | campo per annotazione umana, 3C |
| `data_pagamento` | già `Dado` nel modello | `nfe:cobr/dup/dVenc` | una scadenza non è un pagamento |

Alternativa non scelta, da proporre a parte: portare quei quattro campi a
`Dado<String>` in `Lancamento`, che risolverebbe il problema alla radice ma
cambia il modello e l'export.

### 5.5 Revisione e propagazione

| Ingresso | Avviso | Effetto |
|---|---|---|
| `Confianca::NaoClassificado` | `NAO_CLASSIFICADO` | conto di fallback di revisione (`13.01.001` nel caso dei prodotti), `REVISAR` |
| `Confianca::Ambiguo` | `CLASSIFICACAO_AMBIGUA` | conto a priorità maggiore, `REVISAR` |
| avvisi propri della classificazione | copiati | `REVISAR` |
| differenza di totale non spiegata | `VALOR_INCONSISTENTE` | `REVISAR` solo su quella registrazione |
| rate che non chiudono | `VALOR_INCONSISTENTE` | nel Python su **tutte** le registrazioni (`allocation.py:52-57`) → decisione **D3** |
| avvisi del parser (`FATURA_AUSENTE`, `AGREGADO_PARCIAL`, …) | oggi **non** entrano nel `Lancamento` | proposta: propagarli come avvisi di documento nella riconciliazione, non duplicati su ogni riga |

Regola di stato (`allocation.py:99-103` e `:56-57`): `OK` **solo se** tutte le
classificazioni del gruppo sono `OK` **e** non c'è alcun avviso; qualunque
avviso porta a `REVISAR`. `StatusRevisao::Aprovado` esiste nel modello Rust ma
è uno stato della **revisione umana** (3C): la 3B non lo produce mai.

Quando **fallire** invece di produrre una registrazione da revisione:

| Situazione | Esito proposto |
|---|---|
| `vNF`, `vProd`, `dhEmi`, blocchi `ide`/`emit`/`dest`/`total`/`ICMSTot` assenti | **già errore in fase 2** (C10): la 3B non parte. Questi sono gli unici campi davvero bloccanti |
| `chave`, `numero`, `serie`, CNPJ/nome di emittente e destinatario assenti | **non** bloccati dalla fase 2 (§3.4: avviso o nulla): la 3B li riceve. Policy in **F3A-23a/b**, decisione **D5** |
| conto `2.05.001`/`2.05.002` assente dal piano | **errore** di configurazione, nessun fallback silenzioso (F3A-20) |
| nessun articolo nel documento | nessuna registrazione (il Python ritorna `[]`, `allocation.py:42-43`): proposta di emettere **anche** un avviso di documento, perché una NF-e senza articoli allocati non è un successo |
| `vNF = 0` con rate presenti | decisione **D1** |
| classificazione non conclusa | **registrazione da revisione**, non errore |

### 5.6 Data di pagamento

Le rate hanno `vencimento` (scadenza), che **non** attesta un pagamento. Il
Python non popola mai `data_pagamento` in allocazione (ricerca negativa §2.2):
resta `None` e viene esportata come colonna vuota.

Contratto proposto per la 3B: `data_pagamento = Dado::ausente(status = NaoDisponivel,
fonte = "nfe:cobr/dup/dVenc", motivo = "la NF-e dichiara scadenze, non pagamenti")`.
Quando esisterà una fonte di pagamenti (estratto bancario, CSV), quella fonte
popolerà il campo con stato `Ok`. Mai la scadenza al posto del pagamento.

### 5.7 `valor_cobranca` non entra nell'allocazione

Fatto verificato: `allocation.py` non legge `valor_cobranca` in nessun punto.
Il riparto usa `vNF`. Quindi i quattro stati C10 di `valor_cobranca`
(`N.A.`/`NAO_DISPONIVEL`/`MANCANTE`/zero reale) **non bloccano** la produzione
delle registrazioni: una NF-e senza `<fat>` resta allocabile, con le sue rate
conservate. Questo risponde al punto del mandato: l'assenza di `fat` **non**
deve diventare un blocco dell'intera NF-e, e non lo diventa, perché non è un
ingresso di questa trasformazione.

Proposta: riportare `valor_cobranca` nella riconciliazione come componente
dichiarata (nota / non disponibile / non applicabile), così l'informazione non
si perde senza diventare un prerequisito.

---

### 5.8 Dominio monetario: l'invariante non è gratis

**Correzione del riscontro R3A-05.** La revisione 1 presentava
`Σ registrazioni = vNF` come garanzia ottenuta copiando `allocation.py`. **Non
lo è.** Controesempio eseguito dall'audit sul codice Python originale:

```text
due articoli da 0.005 su due conti diversi, vNF = 0.01, nessuna rata

gruppo 1 = 0.005 → round2 → 0.01
gruppo 2 = 0.005 → round2 → 0.01
diferenca = round2(0.01 − 0.010) = 0.00  → nessuna registrazione di differenza

risultato: Σ registrazioni = 0.02  contro  vNF = 0.01
           entrambe le righe OK, nessun avviso
```

Il parser Rust corrente **accetta** importi con più di due decimali:
`analisar_decimal` è `Decimal::from_str_exact` senza vincolo di scala
(`nfe.rs`, fixture d'audit `vprod_tres_decimais.xml`). Né il parser né i
modelli pubblici impongono le due cifre del layout monetario NF-e.

Dominio dichiarato per la 3B, nella formulazione **approvata il 2026-10-16**:
un importo è nel dominio se il suo **valore** è esprimibile in centesimi, cioè
se è un multiplo esatto di `0.01`. Non è un controllo sul numero di cifre
scritte: gli zeri finali non cambiano il valore e non squalificano l'importo.

| Importo | Nel dominio | Perché |
|---|---|---|
| `10.00` | sì | multiplo esatto di 0.01 |
| `10.0000` | **sì** | stesso valore di 10.00; gli zeri finali sono irrilevanti |
| `10` | sì | stesso valore |
| `0.005` | **no** | mezza frazione di centesimo |
| `12.345` | **no** | frazione di centesimo |

La revisione 1 scriveva "al massimo 2 decimali": letto alla lettera era un
controllo sulla **scala** e avrebbe rifiutato `10.0000`. Formulazione corretta e
verifica esatta in `rust_decimal` 1.43.0: `valor.normalize().scale() <= 2`
(`normalize` toglie gli zeri finali, `decimal.rs:1413`; `scale`, `:836`).
Nessun `f64`, nessuna correzione automatica a `0.01` o `12.35`.

Soggetti al dominio: `vProd` di ogni articolo, `vNF`, le componenti monetarie
del totale (`vFrete`, `vSeg`, `vDesc`, `vOutro`, tributi inclusi) e `vDup` di
ogni rata. **Esclusi**: quantità e valori unitari (`qCom`, `vUnCom`), che non
sono importi da registrare e possono legittimamente avere più decimali.

Due precisazioni che il mandato 3B deve riportare:

1. il controllo si applica agli **ingressi**, non ai valori calcolati: somme di
   gruppo e celle del riparto sono già arrotondate a 2 decimali da F3A-21,
   quindi sono nel dominio per costruzione;
2. un `Dado` **assente** salta il controllo, non lo supera: un
   `valor_tributos_inclusos` con stato diverso da `Ok` non va trattato come
   `0.00` per farlo passare, altrimenti si violerebbe C10 per soddisfare F3A-26.

| Opzione per gli input fuori dominio | Comportamento | Rischio |
|---|---|---|
| **(a) rifiuto nell'allocatore** (raccomandata, F3A-26) | errore `ESCALA_MONETARIA_INVALIDA` che nomina articolo e valore; nessuna registrazione prodotta | diverge dal Python, che accetta e produce righe apparentemente valide |
| (b) riconciliazione `ERRO` | registrazioni prodotte, `esito_total = ERRO`, tutte `REVISAR` | righe esistono: vanno marcate in modo che nessuna sembri contabilizzabile |
| (c) parità col Python | somma divergente senza alcun avviso | **inaccettabile**: viola l'invariante dichiarata in silenzio |

Scelta proposta: **(a)**, perché il difetto è nei dati d'ingresso e un rifiuto
nominato è più onesto di una registrazione "da revisionare" il cui totale non
chiude. Se si preferisce **(b)**, va definito esattamente il ritorno: `esito_total
= ERRO`, ogni riga `REVISAR` con avviso `TOTAL_NAO_RECONCILIADO`, e nessuna
riga esportabile senza revisione. La divergenza dal Python va dichiarata, non
introdotta di nascosto. Decisione **D6**, §11. Non si modifica il parser: il
vincolo vive nell'allocatore, dove serve.

## 6. D — Competenza, identità, ripetibilità

### 6.1 Competenza

Python: `competencia = data_emissao.strftime("%Y-%m")` (`allocation.py:107`),
stringa libera. Rust: `Competencia` è un tipo validato con `ano` fra
`ANO_MIN = 1900` e `ANO_MAX = 2999` (`modelos.rs:364-366`, C8). Divergenza
dichiarata: una data fuori
intervallo diventa **errore** invece di una stringa accettata. La competenza è
quella del **documento** (emissione), non della scadenza e non del pagamento.

### 6.2 Hash e identità di riga

```text
payload = origem | chave_origem | conta_id | valor con esattamente 2 decimali
hash    = sha256(payload codificato UTF-8), esadecimale minuscolo
```

Fonte: `allocation.py:305-313`. Esempio di payload: `XML_NFE|2926...901|2.01.004|10.00`.

Proprietà verificate leggendo la fonte:

| Proprietà | Esito |
|---|---|
| Stabilità fra esecuzioni | **sì**: dipende da 4 valori deterministici, non dall'ordine né dall'orario |
| Reimportazione identica | **stesso hash**: la ripetizione è riconoscibile |
| Documento corretto con valori diversi sullo stesso conto | **hash diverso** |
| Documento corretto che cambia solo gli articoli, non il totale di conto | **stesso hash**: la correzione non si vede |
| Collisione fra registrazioni dello stesso documento | **possibile**, vedi sotto: non è una garanzia |
| Campi **non** inclusi | competenza, data, documento/serie, descrizione, rate, status, avvisi |
| Deduplicazione persistente | **non** in allocazione: i duplicati sono diagnosticati a valle (`excel_diagnostics.py:111-136`) |

**Correzione del riscontro R3A-06.** La revisione 1 affermava che una
collisione dentro lo stesso documento fosse *impossibile*. L'affermazione
assumeva che `2.05.001`/`2.05.002` non possano mai essere il conto scelto dalla
classificazione di un articolo — riserva che **nessuna validazione impone**.
Controesempio eseguito dall'audit con un classificatore controllato:

```text
un articolo da 10.00 classificato su 2.05.001
vNF 20.00, vFrete 10.00  → differenza +10.00, anch'essa su 2.05.001

due registrazioni, stesso conto, stesso valore:
payload identico  → hash identico
745b73187bb8e4b19cb9324fd9ebca8cebe5978b016c567cb00fbfe9bb2ab163
```

Non è una collisione crittografica: sono **due righe diverse con lo stesso
payload di quattro campi**. Nessuna collisione è stata osservata né affermata
sul campione canonico o sulla configurazione canonica corrente: il
controesempio usa un classificatore di prova.

Conseguenze per la 3B, nei limiti dichiarati:

- l'hash è un'**identità di riga ripetibile sotto precondizioni**, non una
  chiave univoca: **non** va usato come criterio automatico di scarto o di
  deduplicazione;
- se due registrazioni dello stesso documento condividono il payload, la 3B
  deve **segnalarlo** (avviso `HASH_DUPLICADO_NO_DOCUMENTO`) senza scartare
  nulla: scartare una riga vera per un'identità debole sarebbe peggio del
  duplicato;
- riservare `2.05.001`/`2.05.002` alle sole registrazioni di differenza sarebbe
  una **scelta separata** sulla configurazione, non un effetto di questa fase;
- nessuno storage, nessun algoritmo di hash nuovo, nessuna regola di
  sostituzione dello storico va inventata ora. "Deduplicazione persistente e
  aggiornamento di registrazioni esistenti" appartiene a una fase con archivio
  e va deciso lì (`DA_PIANIFICARE`).

### 6.3 Ordine deterministico

| Elemento | Ordine |
|---|---|
| Registrazioni | ordine di prima comparsa del conto fra gli articoli; la registrazione di differenza **in coda** (`allocation.py:45-46`) |
| Rate dentro una registrazione | ordine delle rate nel documento (`allocation.py:252-259`) |
| Avvisi | ordine di inserimento, deduplicati conservando il primo (`_unique`, `allocation.py:330-337`) |
| Articoli nella descrizione | ordine del documento (`allocation.py:222-225`) |

---

## 7. E — Confine fra 3B, 3C e 3D

| Fase | Contenuto | Stato |
|---|---|---|
| **3B** | allocazione NF-e → registrazioni in memoria + riconciliazione + elementi da revisione | da implementare, perimetro di questa specifica |
| **3C** | pre-libro, revisione umana, esportazione, deduplicazione con archivio | fuori perimetro. Nel Python il pre-libro **legge un Excel** (`pre_livro_diario.py:257`): la 3C presuppone una persistenza che la 3B non crea |
| **3D** | aggregazione DRE per competenza | fuori perimetro. Nel Python la linea DRE è assegnata in `dre_gerencial_provisoria.py:246` leggendo `mapeamento_dre.yaml` |

`dre_linha_id` e `dre_linha_nome` esistono già in `Lancamento` e **non vanno
riempiti in silenzio**: in 3B restano vuoti, con il motivo dichiarato nella
tabella §3.3.

**Dipendenza minima della 3B dal mapping DRE.** Misura fatta oggi su
`config/mapeamento_dre.yaml`: 26 linee DRE, 105 mappature, su **125 conti del
piano**: **20 conti non hanno alcuna mappatura**. Nessuna mappatura punta a un
conto inesistente.

```text
Conti senza linea DRE (20): 3.02.009, 3.03.003, 3.03.009, 3.03.010, 3.04.011,
4.01.006, 5.02.001, 5.02.002, 5.02.003, 5.02.004, 5.04.004, 5.04.005,
8.03.001, 8.03.002, 11.01.003, 11.01.010, 12.01.001, 12.01.002, 12.01.007,
12.01.009
```

Verificato che i conti usati dall'allocazione nel caso canonico e nei casi di
differenza **sono mappati**: `2.01.002`, `2.01.004`, `2.04.001` → `03.02`/`03.06`;
`2.05.001`, `2.05.002` → `03.01`; tutti i `13.01.*` di revisione → `99.01`.
I 20 conti non mappati sono in gran parte conti di spesa (famiglie 3-8) e
patrimoniali (11-12): rilevanti per un futuro import di spese, non per gli
articoli di una NF-e.

**Correzione del riscontro R3A-03.** La revisione 1 proponeva che la 3B
emettesse un avviso `CONTA_SEM_MAPEAMENTO_DRE`. **Non è producibile**: la firma
`alocar_nfe(importada, classificador, plano)` non riceve il mapping, il file
non è letto dal Rust e `PlanoContas` non lo contiene. Una funzione pura non può
emettere un avviso che dipende da un dato che non ha.

Scelta per il primo blocco: **opzione 1 — rinviare la verifica alla 3D.**

| Opzione | Cosa comporta | Scelta |
|---|---|---|
| 1. Rinviare alla 3D | in 3B i campi DRE restano con stato `NAO_ASSEGNATO` dichiarato in `campi_rinviati` (§5.4); nessun avviso, nessun parametro, nessun lettore nuovo | **scelta**, perimetro minimo |
| 2. Validare già in 3B | servono: un lettore di `mapeamento_dre.yaml`, la validazione del file, un parametro in più nella firma (il caricamento resta **fuori** dalla funzione pura), i test e la configurazione relativi | non scelta ora |

La misura dei 20 conti non mappati resta in questo documento come **dato per la
3D**, non come requisito della 3B. Nessun YAML va modificato. Questa non è una
decisione funzionale approvata da Luca: è il perimetro minimo del primo blocco,
e l'opzione 2 resta disponibile se la 3D la richiederà prima.

---

## 8. Esempi numerici

Tutti calcolati in decimale esatto, `ROUND_HALF_UP`, mai `f64`. Il caso A viene
dal campione canonico e dalle attese del test Python; i casi B-G sono
**SINTETICI**: non sono fonti canoniche e non sostituiscono il campione.

### A — Campione canonico (fonte reale)

Input: `nfe_oesa_12566_20260505.xml`, `vProd = vNF = 35.00`, `<fat>` 35.00, due
rate da 17.50, 3 articoli: 10.00 Coca-Cola → `2.01.004`, 20.00 vino →
`2.01.002`, 5.00 spugna → `2.04.001`.

```text
gruppi (ordine di comparsa): 2.01.004=10.00 ; 2.01.002=20.00 ; 2.04.001=5.00
diferenca = round2(35.00 − 35.00) = 0.00  → nessuna registrazione aggiuntiva
denominatore = 35.00

riparto:
  rata 1 (17.50): 5.00 | 10.00 | 2.50   somma 17.50 = rata     OK
  rata 2 (17.50): 5.00 | 10.00 | 2.50   somma 17.50 = rata     OK
  per registrazione: 10.00 | 20.00 | 5.00  = valore            OK
  totale allocato 35.00 = vNF                                   OK

atteso: 3 registrazioni, status OK, alertas vuoti, competenza 2026-05,
        data_documento 2026-05-05, riconciliazione OK
```

Corrisponde a `test_allocation.py:42-48`, `:51-53`, `:70-89`, `:92-100`,
`:103-105`. Replica indipendente eseguita: identica.

### B — SINTETICO: residuo di arrotondamento da un centesimo

Tre registrazioni 6.67 / 6.67 / 6.66 (totale 20.00), due rate da 10.00:

```text
rata 1: 3.34 | 3.34 | 3.32   somma 10.00  OK
rata 2: 3.33 | 3.33 | 3.34   somma 10.00  OK
colonne: 6.67 | 6.67 | 6.66  = valori     OK
```

`10.00 × 6.67 / 20.00 = 3.335` → `ROUND_HALF_UP` → `3.34` nella prima riga; la
seconda riga assorbe il resto (`3.33`), l'ultima colonna assorbe il resto di
riga. Con Bankers (default di `rust_decimal`) la prima cella darebbe `3.34`
anche qui, ma `2.345`-simili divergerebbero: per questo F3A-21 impone la
strategia esplicita.

### C — SINTETICO: rate che non chiudono il totale

Rate 17.50 + 17.00 = 34.50 contro `vNF` 35.00, registrazioni 10/20/5:

```text
somma rate (34.50) ≠ vNF (35.00) → VALOR_INCONSISTENTE (allocation.py:235)
rata 1: 5.00 | 10.00 | 2.50   somma 17.50  OK
rata 2: 5.00 | 10.00 | 2.00   somma 17.00  OK
colonna 3: 4.50 contro valore 5.00  → VALOR_INCONSISTENTE
```

Nel Python l'avviso finisce su **tutte** le registrazioni e tutte diventano
`REVISAR`, pur avendo rate coerenti due su tre (decisione **D3**). Da notare:
le rate scritte sulla terza registrazione (4.50) **non** sommano al suo valore;
l'avviso è l'unica cosa che lo segnala.

### D — SINTETICO: denominatore zero

`vNF = 0.00`, una rata da 10.00. Python: nessuna divisione, ogni registrazione
riceve rate `0.00` e l'avviso `VALOR_INCONSISTENTE` (perché 10.00 ≠ 0.00).
Alternative in decisione **D1**:

| Opzione | Rate scritte | Rischio |
|---|---|---|
| (a) errore `RIPARTO_IMPOSSIBILE` | nessuna | diverge dal Python |
| (b) rate assenti con motivo | nessuna, `esito_rate = NON_VERIFICATO` | richiede `Dado` in `Duplicata` per portare un valore |
| (c) parità col Python | `0.00` | riscrive a zero una rata dichiarata da 10.00 |

Qui lo `0.00` scritto **è** inventato, perché la rata dichiarata vale 10.00.
Non vale la stessa conclusione quando la rata è davvero zero: vedi i tre casi
di **D1** e il controesempio §8M.

### E — SINTETICO: rata unica su tre registrazioni

Una rata da 35.00, registrazioni 10/20/5: `10.00 | 20.00 | 5.00`, somma 35.00,
tutte le invarianti chiudono. Caso degenere corretto (unica riga = ultima riga).

### F — SINTETICO: tre rate su due registrazioni

Rate 11.11 × 3 (totale 33.33), registrazioni 16.67 / 16.66:

```text
rata 1: 5.56 | 5.55    rata 2: 5.56 | 5.55    rata 3: 5.55 | 5.56
colonne: 16.67 | 16.66  = valori   OK ; righe tutte 11.11   OK
```

### G — SINTETICO: differenze di totale

| Caso | `vNF` | Σ`vProd` | componenti | `diferenca` | conto | spiegata | status |
|---|---|---|---|---|---|---|---|
| accessori | 110.00 | 100.00 | `vFrete` 10.00 | +10.00 | 2.05.001 | sì | OK |
| sconto | 90.00 | 100.00 | `vDesc` 10.00 | −10.00 | 2.05.002 | sì | OK |
| accessori non spiegati | 115.00 | 100.00 | `vFrete` 10.00 | +15.00 | 2.05.001 | **no** | REVISAR + `VALOR_INCONSISTENTE` |
| **tributi parziali** | 112.00 | 100.00 | `vFrete` 10.00 + tributi 2.00 **Parcial** | +12.00 | 2.05.001 | Python: **sì** | Python: OK; proposta: **PARCIAL + REVISAR** (D2) |
| nessuna differenza | 100.00 | 100.00 | — | 0.00 | — | — | nessuna registrazione |

I primi due corrispondono a `test_allocation.py:151-191`.

### H — SINTETICO: classificazione non conclusa

Un articolo "PRODUTO XYZ DESCONHECIDO" da 12.00: conto di fallback
`13.01.001` ("Revisar - Produto Nao Classificado", `tipo: revisao`,
`exige_revisao: true`), `status_revisao = REVISAR`, `alertas` contiene
`NAO_CLASSIFICADO`, `tipo_lancamento = Revisao`. Corrisponde a
`test_allocation.py:108-148`. Un articolo ambiguo produce invece
`CLASSIFICACAO_AMBIGUA` sul conto a priorità maggiore.

### I — SINTETICO: NF-e senza `<fat>` con due rate

`vNF` 35.00, `valor_cobranca` = `NAO_DISPONIVEL`, due rate da 17.50. Il riparto
usa `vNF` e produce gli stessi numeri del caso A. Nessun importo di
fatturazione viene ricostruito dalle rate (C10). La riconciliazione dichiara
`valor_cobranca` come componente **non disponibile**; le registrazioni restano
valide. Con `<vLiq>0.00</vLiq>` dichiarato, lo zero è reale e vale lo stesso
esito.

### J — SINTETICO: reimportazione

Stesso XML importato due volte: stessi gruppi, stessi valori, **stessi
`hash_registro`**. Nessuna scrittura, quindi nessun duplicato creato: la 3B è
una funzione pura. Se invece il documento cambia il valore di un conto, cambia
l'hash di quella riga; se cambia solo la composizione degli articoli a parità
di totale di conto, **l'hash resta identico** e la correzione non si distingue
dall'originale (limite §6.2, decisione **D4**).

### K — Fuori dominio monetario (eseguito dall'audit sul Python originale)

Due articoli da `0.005` su conti diversi, `vNF = 0.01`, nessuna rata:

```text
gruppo 1: 0.005 → 0.01      gruppo 2: 0.005 → 0.01
diferenca = round2(0.01 − 0.010) = 0.00   → nessuna registrazione di differenza
Σ registrazioni = 0.02   ≠   vNF = 0.01
Python: entrambe le righe OK, nessun avviso
```

Atteso con la proposta (a) di §5.8/**D6**: errore `ESCALA_MONETARIA_INVALIDA`
che nomina articolo e valore, nessuna registrazione. Con (b): registrazioni
prodotte, `esito_total = ERRO`, `TOTAL_NAO_RECONCILIADO`, tutte `REVISAR`.

### L — Payload di hash ripetuto (eseguito dall'audit)

Un articolo da `10.00` classificato su `2.05.001`, `vNF = 20.00`,
`vFrete = 10.00` → differenza `+10.00`, anch'essa su `2.05.001`:

```text
riga 1: conta 2.05.001, valor 10.00  → payload XML_NFE|<chiave>|2.05.001|10.00
riga 2: conta 2.05.001, valor 10.00  → payload identico
hash identico: 745b73187bb8e4b19cb9324fd9ebca8cebe5978b016c567cb00fbfe9bb2ab163
```

Atteso: due registrazioni **conservate**, avviso
`HASH_DUPLICADO_NO_DOCUMENTO`, righe `REVISAR` (F3A-27). Nessuno scarto
automatico. Il caso richiede un classificatore che mandi un articolo su
`2.05.001`: non si verifica con la configurazione canonica corrente.

### M — Compensazione a totale netto zero (eseguito dall'audit sul Python originale)

Un articolo da `100.00`, `vDesc = 100.00`, `vNF = 0.00`, **una rata dichiarata
`0.00`**:

```text
gruppo 2.01.004      = 100.00
diferenca            = round2(0.00 − 100.00) = −100.00  → 2.05.002 = −100.00
Σ registrazioni      = 0.00  =  vNF            → esito_total OK
denominatore         = 0.00  → ramo allocation.py:237-243

rate allocate: 2.01.004 → 0.00 ; 2.05.002 → 0.00
Σ rate documento = 0.00 = vNF  → nessun VALOR_INCONSISTENTE

Python: entrambe le righe OK, nessun avviso
violazione silenziosa: Σ rate di 2.01.004 = 0.00 ≠ suo valore 100.00
```

Tutti gli importi sono a due decimali: **D6 non intercetta questo caso**. Lo
zero della rata è reale, e proprio per questo nessun avviso scatta. Atteso con
la proposta **D1(a)**: `RIPARTO_IMPOSSIBILE`, perché il caso banale richiede
che anche le **registrazioni** siano zero. Con **D1(b)**: registrazioni
prodotte, `esito_rate = NON_VERIFICATO`, `duplicatas` vuote con motivo, righe
`REVISAR`. Test pianificato: `total_netto_zero_com_lancamentos_nao_nulos`.

Prova: probe dell'audit `probe_d1_zero_netto.py` / `.json`, sulle funzioni
originali con classificatore controllato; **non** `pytest`, **non** il loader
YAML canonico. Ramo del codice riletto e confermato da me:
`allocation.py:237-243` ritorna prima dei controlli per riga e colonna.

---

## 9. Matrice dei contratti F3A

| ID | Regola | Fonte esatta | Stato | Assenza/errore | Codice futuro | Caso e atteso | Prova |
|---|---|---|---|---|---|---|---|
| F3A-01 | Una registrazione per (documento, conto); articoli classificati singolarmente | `allocation.py:32-60,63-86` | EREDITATO_VERIFICATO | documento senza articoli → nessuna registrazione + avviso proposto | `alocacao.rs` | A → 3 registrazioni | `test_allocation.py:42-48` (NON_ESEGUITO) + §8A |
| F3A-02 | Ordine dei gruppi = prima comparsa del conto; differenza in coda | `allocation.py:66-86,45-46` | EREDITATO_VERIFICATO | — | `alocacao.rs` | A → 2.01.004, 2.01.002, 2.04.001 | da creare |
| F3A-03 | `valor` = Σ `vProd` del gruppo, arrotondato a 2 **alla fine** | `allocation.py:83,118` | EREDITATO_VERIFICATO | `vProd` assente già errore C10; `vProd` con più di 2 decimali → F3A-26 | `alocacao.rs` | A → 10.00/20.00/5.00 | §8A |
| F3A-04 | `descricao_itens` = `"{desc} ({valor})"` uniti da `"; "` | `allocation.py:222-225` | EREDITATO_VERIFICATO | descrizione vuota → F3A-24 | `alocacao.rs` | A → 3 voci | da creare |
| F3A-05 | Differenza di totale: >0 → 2.05.001, <0 → 2.05.002, =0 → nessuna | `allocation.py:131-169` | EREDITATO_VERIFICATO | — | `alocacao.rs` | G | `test_allocation.py:151-191` (NON_ESEGUITO) + §8G |
| F3A-06 | Differenza spiegata ⇔ uguale a accessori identificati (o a `vDesc`) | `allocation.py:140,156,172-178` | EREDITATO_VERIFICATO | — | `alocacao.rs` | G righe 1-3 | §8G |
| F3A-07 | Se una componente della verifica è `Parcial`, la differenza **non** è spiegata: `PARCIAL` + `REVISAR` | contrasto fra `allocation.py:172-178` e C10 | **APPROVATO** 2026-10-16 (**D2(a)**) | — | `alocacao.rs` | G riga 4 | da creare |
| F3A-08 | Riparto rate: celle interne proporzionali, ultima colonna/riga assorbono il resto | `allocation.py:273-302` | EREDITATO_VERIFICATO | — | `alocacao.rs` | A, B, E, F | §8 + probe |
| F3A-09 | Denominatore del riparto = `round2(vNF)`, mai la somma delle rate né `valor_cobranca` | `allocation.py:231-249` | EREDITATO_VERIFICATO + VINCOLO_GIÀ_VIGENTE (C10); denominatore zero **APPROVATO** 2026-10-16 (**D1(a)**) | `vNF` assente già errore C10 | `alocacao.rs` | I | §8I |
| F3A-10 | Invarianti: Σ registrazioni = `vNF`; Σ rate per registrazione = suo valore; Σ colonne per rata = rata. **Vale solo nel dominio monetario a 2 decimali** (F3A-26) **e con denominatore non zero**: fuori dal dominio la viola il controesempio §5.8; con denominatore zero il Python non la verifica affatto (`allocation.py:237-243`, §8M) | `allocation.py:233-270` | EREDITATO_VERIFICATO **nel dominio e con denominatore non zero**; controesempi §5.8 e §8M eseguiti dall'audit | violazione → `esito_rate = ERRO`, oppure `esito_total = ERRO` + `TOTAL_NAO_RECONCILIADO` secondo la dimensione violata (§5.4) | `alocacao.rs` | A OK; C viola sulle rate; §5.8 viola sul totale; §8M non verificata | §8A, §8C, §5.8, §8M |
| F3A-11 | Avvisi del riparto propagati a tutte le registrazioni | `allocation.py:52-57` | EREDITATO_VERIFICATO; **APPROVATO** 2026-10-16 (**D3(a)**) | — | `alocacao.rs` | C → tutte REVISAR | **D3** |
| F3A-12 | `OK` solo se tutte le classificazioni `Ok` e nessun avviso; altrimenti `REVISAR` | `allocation.py:99-103,56-57` | EREDITATO_VERIFICATO | — | `alocacao.rs` | A OK, H REVISAR | `test_allocation.py:103-105` (NON_ESEGUITO) |
| F3A-13a | `NAO_CLASSIFICADO` → conto di **fallback di revisione** + avviso `NAO_CLASSIFICADO` + `REVISAR`, **non** errore | `allocation.py:89-95` | EREDITATO_VERIFICATO | — | `alocacao.rs` | H → `13.01.001` | `test_allocation.py:108-148` (NON_ESEGUITO) |
| F3A-13b | `AMBIGUO` → **il conto vincente per priorità resta invariato**, più avviso `CLASSIFICACAO_AMBIGUA` + `REVISAR`. L'ambiguità **non** dirotta su un conto di revisione | `allocation.py:93-94` (aggiunge solo l'avviso, non cambia `classification.conta`) | EREDITATO_VERIFICATO | — | `alocacao.rs` | "papel manteiga" → `2.02.006` Laticinios e Queijos, `Ambiguo` (CLI corrente) | da creare; la CLI corrente lo mostra già |
| F3A-14 | Competenza = anno/mese della **data di emissione** | `allocation.py:107` | EREDITATO_VERIFICATO; divergenza Rust dichiarata | data fuori 1900..2999 → errore (C8) | `alocacao.rs` | A → 2026-05 | da creare |
| F3A-15 | `hash_registro` = sha256(`origem|chave|conta_id|valor.2f`); **identità di riga sotto precondizioni, non chiave univoca** | `allocation.py:305-313` | EREDITATO_VERIFICATO; parità **APPROVATA** 2026-10-16 (**D4(a)**) | payload ripetuto nello stesso documento → avviso F3A-27, nessuno scarto | `alocacao.rs` | J; controesempio §6.2 | **D4** |
| F3A-16 | `origem` e `tipo_documento` = `XML_NFE` | `allocation.py:113,120` | EREDITATO_VERIFICATO | — | `alocacao.rs` | A | `test_allocation.py:66-67` (NON_ESEGUITO) |
| F3A-17 | `data_pagamento` resta assente con stato e motivo: una scadenza non è un pagamento | ricerca negativa su `src/fantozzi/*.py` + C2 | VINCOLO_GIÀ_VIGENTE + PROPOSTO (stato/motivo) | — | `alocacao.rs` | A → `NAO_DISPONIVEL` | da creare |
| F3A-18 | `dre_linha_*` non popolati in 3B; nessuna linea arbitraria. La verifica del mapping è **rinviata alla 3D** (R3A-03): la 3B dichiara `NAO_ASSEGNATO` in `campi_rinviati` e **non** emette avvisi sul mapping | `allocation.py` (assenza) + `dre_gerencial_provisoria.py:246` | EREDITATO_VERIFICATO (non popolati) + PROPOSTO (stato dichiarato) | 20/125 conti non mappati: dato per la 3D | `alocacao.rs` | §7 | `dre_fica_nao_assegnato_con_motivo` (da creare) |
| F3A-19 | `centro_custo` e `observacoes` vuoti in 3B, con motivo dichiarato | `models.py:94-95`, nessuna fonte | EREDITATO_VERIFICATO | — | — | — | — |
| F3A-20 | `2.05.001`/`2.05.002` risolti con `PlanoContas::por_id` (`regras.rs:209`, `Option`) senza fallback silenzioso; assenti = errore tipizzato | `allocation.py:181-182` + `chart_of_accounts.py:43-46` (`KeyError`) | EREDITATO_VERIFICATO quanto all'esito bloccante; **PROPOSTO** solo per la forma dell'errore | conto assente → errore di configurazione | `alocacao.rs` | config reale: entrambi presenti | verificato oggi sul YAML |
| F3A-21 | Arrotondamento esplicito `MidpointAwayFromZero` a 2 decimali | `allocation.py:316-317` vs `rust_decimal` 1.43.0 `decimal.rs:1654` | **PROPOSTO** | — | `alocacao.rs` | B | verificato sul sorgente del crate |
| F3A-22 | Ordine deterministico di registrazioni, rate, avvisi e articoli; avvisi deduplicati conservando il primo | `allocation.py:320-337,252-259` | EREDITATO_VERIFICATO | — | `alocacao.rs` | A | da creare |
| F3A-23a | `chave` vuota → **errore `IDENTIDADE_INSUFICIENTE`**: senza chiave la riga non è identificabile | `nfe.rs:432` (avviso, non errore) + §6.2 | **APPROVATO** 2026-10-16 (**D5(a)**) | nessuna registrazione prodotta | `alocacao.rs` | `sem_chave.xml` → oggi exit 0 con chiave vuota | `chave_vazia_nao_produz_lancamento` |
| F3A-23b | `numero`, `cnpj`/`nome` emittente e destinatario vuoti → registrazione prodotta, `REVISAR` + avviso dedicato | `nfe.rs:459,663`; `nfe_importer.py:63` | **APPROVATO** 2026-10-16 (**D5(a)**), `serie` esclusa | avvisi `DOCUMENTO_SEM_NUMERO`, `FORNECEDOR_SEM_IDENTIDADE`, `EMPRESA_SEM_IDENTIDADE` | `alocacao.rs` | §3.4 | 3 test, §12 |
| F3A-24 | Articolo senza descrizione: l'articolo **resta** nella registrazione con il suo importo, `descricao_itens` contiene `" (12.00)"`, piu' avviso `ITEM_SEM_DESCRICAO` e `REVISAR`; nessuna descrizione inventata | `allocation.py:222-225` | **EREDITATO_VERIFICATO** (audit 2026-10-16) + policy **APPROVATA** 2026-10-16 | avviso + revisione, l'importo non si perde | `alocacao.rs` | articolo con `xProd` vuoto → `" (12.00)"`, `REVISAR` | `item_sem_descricao_e_conservado_com_aviso` |
| F3A-25 | La 3B è una funzione pura: nessuna scrittura, nessuna deduplicazione persistente | `test_fase3_end_to_end.py:7-42`, `pre_livro_diario.py:257` | EREDITATO_VERIFICATO | — | `alocacao.rs` | J | da creare |
| F3A-26 | **Dominio monetario**: importi con al massimo 2 decimali (`vProd` di articolo, `vNF` e componenti, `vDup`); fuori dominio = rifiuto nominato `ESCALA_MONETARIA_INVALIDA` | controesempio §5.8 eseguito dall'audit sul Python; `nfe.rs` non vincola la scala | **APPROVATO** 2026-10-16 (**D6(a)**), formulazione in centesimi | errore che nomina articolo e valore | `alocacao.rs` | §5.8: due articoli da 0.005 con `vNF` 0.01 | `importo_fora_do_dominio_monetario_e_recusado` |
| F3A-27 | Due registrazioni dello stesso documento con payload di hash identico → avviso `HASH_DUPLICADO_NO_DOCUMENTO`, **nessuno scarto automatico** | controesempio §6.2 eseguito dall'audit | **APPROVATO** 2026-10-16 (parte di **D4(a)**) | avviso + `REVISAR` sulle righe coinvolte | `alocacao.rs` | articolo su `2.05.001` + differenza su `2.05.001`, stesso valore | `payload_duplicado_no_documento_gera_aviso` (da creare) |
| F3A-28 | La riconciliazione separa **quattro esiti** (totale, rate, copertura, componenti) e li aggrega con regola dichiarata; `articoli_attesi` resta `NON_DISPONIVEL` invece di essere inventato | §5.4; `NfeImportada` espone solo `nfe` e `avisos` | **APPROVATO** 2026-10-16 (perimetro 3B: quattro esiti) | ogni condizione ha uno stato nella tabella §5.4 | `alocacao.rs` | `det_sem_prod.xml` → `esito_copertura = PARCIAL` | `copertura_parcial_com_det_sem_prod` (da creare) |

Nessun test di questa matrice esiste: tutti sono **da creare** e nessuno è
stato eseguito. Le prove marcate `(NON_ESEGUITO)` sono attese **lette** nel
sorgente dei test Python, non osservate in esecuzione.

---

## 10. Divergenze Python / Rust proposto

| Tema | Comportamento alla fonte | Prova | Proposta Rust | Ragione | Classificazione | Effetto |
|---|---|---|---|---|---|---|
| Rate con `vNF = 0` | scrive rate `0.00` su ogni registrazione | `allocation.py:237-243` | non scrivere zero: errore o valore assente (**D1**) | C2/C10: assente ≠ zero | **bug rispetto alle regole vigenti** | importi, revisione |
| Differenza con tributi `Parcial` | tratta le componenti assenti come zero e dichiara "spiegata" | `nfe_importer.py:76-84`, `allocation.py:172-178` | `PARCIAL` + `REVISAR` (**D2**, F3A-07) | un aggregato incompleto non può validare un totale | **bug rispetto alle regole vigenti** | revisione, copertura |
| `valor_cobranca` assente | default `Decimal("0")` | `nfe_importer.py:47-51` | già divergente: 4 stati C10 | C10 vigente | differenza intenzionale già implementata | nessuno sull'allocazione (§5.7) |
| Risoluzione dei conti di aggiustamento | accede a `product_classifier._chart` (attributo privato) e `get_by_id` **solleva `KeyError`** se il conto manca | `allocation.py:181-182`, `chart_of_accounts.py:43-46` | `PlanoContas::por_id` (`Option`) + errore tipizzato | il Python **già** blocca: il Rust rende l'errore esplicito e tipizzato, non introduce un blocco nuovo | **parità di esito**, forma diversa (F3A-20) | nessun cambio di comportamento; messaggio migliore |
| Arrotondamento | `ROUND_HALF_UP` esplicito | `allocation.py:316-317` | `round_dp_with_strategy(2, MidpointAwayFromZero)` | il default Rust è Bankers | lacuna tecnica da chiudere | importi al centesimo |
| Competenza | stringa `"%Y-%m"` | `allocation.py:107` | `Competencia` validata 1900..2999 | C8 già vigente | differenza intenzionale | errore su date assurde |
| Avvisi | stringa unita da `"; "` | `allocation.py:326-327` | mantenere la stringa in 3B; `Vec<String>` come evoluzione | parità dimostrabile prima | scelta da approvare | nessuno sugli importi |
| Avvisi del parser | non entrano nel `Lancamento` | `allocation.py` (assenza) | nella riconciliazione come avvisi di documento | non perderli né duplicarli | scelta da approvare | tracciabilità |
| Propagazione globale degli avvisi di riparto | tutte le registrazioni diventano `REVISAR` | `allocation.py:52-57` | **D3** | — | scelta da approvare | revisione |
| Classificazione, alias divergenti, C13, regole per fornitore | come sono | README C13/C16 | **invariati** | fuori mandato | nessuna modifica | nessuno |

---

## 11. Decisioni che cambiano il risultato funzionale

Queste sei hanno effetto sui numeri o sulla revisione.

> **APPROVATE DA LUCA IL 2026-10-16**, tutte nella variante **(a)**, sulla
> scheda di raccomandazione di Codex, con D6 inteso come **importi esprimibili
> in centesimi** (`10.0000` ammesso, `0.005` come totale monetario no; quantità
> e prezzi unitari esclusi) e con le eccezioni di D1 (tre casi) e di `serie`
> come descritte qui sotto.
>
> L'approvazione riguarda **il comportamento**, non l'avvio
> dell'implementazione: la 3B resta da autorizzare a parte, insieme alla
> dipendenza SHA-256 e alle decisioni Git. Le alternative (b), (c), (d) restano
> scritte qui come documentazione delle opzioni **non** scelte, non come
> possibilità ancora aperte.

Le altre proposte della matrice §9 (`PROPOSTO`) **non** sono coperte da questa
approvazione, tranne dove un contratto è la diretta applicazione di una delle
sei decisioni: lo stato di ciascuna è nella matrice.

### D1 — Rate quando il totale della NF-e è zero

Non ogni zero qui è uno zero tecnico, ma **non basta guardare le rate**:
correzione del riscontro R2-03, che ha mostrato un terzo caso. Causa comune:
`allocation.py:237-243` esce dal ramo `expected_total == 0` **prima** dei
controlli per riga e per colonna, quindi in quel ramo le invarianti per
registrazione non vengono mai verificate.

| Caso | Situazione | Che cosa fa il Python | Giudizio |
|---|---|---|---|
| **1** | nessuna rata nel documento | nessun riparto, `duplicatas` vuote | corretto: `esito_rate = NaoAplicavel` |
| **2** | `vNF = 0.00`, **registrazioni e rate tutte `0.00`** | rate `0.00`, nessun avviso | **zero reale** e invarianti rispettate: nulla da correggere |
| **3** | `vNF = 0.00` con **registrazioni non nulle che si compensano** (§8M) oppure con **rate non nulle** | rate riscritte/lasciate a `0.00`; avviso `VALOR_INCONSISTENTE` **solo** se la somma delle rate ≠ 0, quindi nel caso di compensazione **nessun avviso** | **riparto non definito**: ogni registrazione riceve `0.00` benché il suo valore non sia zero. Nel caso di compensazione il Python esce `OK` e silenzioso |

Il caso 3 **non** è intercettato da D6: nel controesempio §8M tutti gli importi
hanno due decimali. E non è intercettato dalla sola somma delle rate: lo zero
della rata può essere reale e il riparto resta comunque non verificato.

La decisione riguarda **solo il caso 3**:

- **(a) errore `RIPARTO_IMPOSSIBILE`** (proposta): con denominatore zero il
  riparto è definito **solo** nel caso banale (caso 2); in tutti gli altri
  nessuna registrazione viene prodotta, e nessun importo dichiarato viene
  riscritto a zero.
- (b) registrazioni prodotte con `esito_rate = NON_VERIFICATO` e righe
  `REVISAR`: il documento resta importabile, ma `duplicatas` **non** può
  portare `0.00` al posto di un valore non verificato senza `Dado<Decimal>` in
  `Duplicata` (evoluzione del modello, oggi fuori perimetro) — quindi in
  pratica richiede che `duplicatas` resti **vuoto** con il motivo dichiarato.
- (c) parità col Python: scrive `0.00` su ogni registrazione e, nel caso di
  compensazione, **senza alcun avviso**.

Criterio di accettazione del caso banale, se si scegliesse (a): accettare
denominatore zero solo quando **tutte** le registrazioni **e** tutte le rate
sono `0.00`; qualunque altra combinazione è riparto non definito. Questa è una
proposta di perimetro, non un algoritmo nuovo.

Effetto: §8D (rate non nulle) e §8M (compensazione, nessun avviso nel Python).
Impedimento a procedere: senza decisione la 3B dovrebbe scegliere fra scrivere
rate `0.00` su registrazioni da 100.00 in silenzio e divergere dalla fonte.

### D2 — Differenza di totale con componenti parziali

- **(a) `PARCIAL` + `REVISAR` con motivo che nomina le componenti assenti**
  (proposta, F3A-07): un aggregato incompleto non può dichiarare spiegata una
  differenza.
- (b) parità col Python: "spiegata", `OK`.
- (c) errore: blocca documenti oggi importabili; sproporzionato.

Dove si vede, se si scegliesse (a): `esito_total` resta `OK` (i totali
chiudono), lo **stato del documento** diventa `PARCIAL`, l'avviso
`AGREGADO_PARCIAL` viaggia in `avisos_documento`, e la componente compare in
`componenti` con il suo `Dado` (`status = Parcial`, motivo che nomina `vFCPST`
e `vIPIDevol`). La registrazione di differenza esce `REVISAR`. Senza questi
posti dichiarati, "PARCIAL + REVISAR" non sarebbe verificabile.

Effetto: §8G riga 4 — la stessa NF-e esce `OK` o `REVISAR` secondo la scelta.

### D3 — Propagazione degli avvisi di riparto

- **(a) parità col Python** (proposta per la prima 3B): tutte le registrazioni
  `REVISAR`; comportamento dimostrabile e confrontabile con lo storico.
- (b) marcare solo le registrazioni non riconciliate, più un avviso di
  documento: più preciso, ma cambia il risultato rispetto allo storico e va
  deciso come cambio di comportamento, non di porting.

Coordinamento con la riconciliazione (§5.4), necessario in **entrambe** le
opzioni: `esito_rate = ERRO` è lo stato **del documento**; quali **righe**
portano `REVISAR` è ciò che questa decisione stabilisce. Con (a) tutte, con (b)
solo quelle la cui somma di rate non torna. Lo stato del documento non cambia
fra le due: cambia solo la marcatura delle righe.

Effetto: §8C — una NF-e con rate incoerenti produce 3 righe `REVISAR` o 1.

### D4 — Campi dell'hash

Il limite dell'hash è **più ampio** di quanto diceva la revisione 1: oltre alle
correzioni a totale invariato (§8J), due righe dello stesso documento possono
condividere il payload (controesempio §6.2, R3A-06).

- **(a) parità col Python** (proposta): 4 campi, stesso payload, gli hash
  restano confrontabili con l'Excel storico; i due limiti vanno registrati e
  il duplicato interno **segnalato** (F3A-27), mai scartato.
- (b) estendere i campi (competenza, documento, descrizione): distingue le
  correzioni a parità di totale e riduce i payload ripetuti, ma rompe la
  confrontabilità con lo storico.

In nessuna delle due l'hash diventa una chiave univoca o un criterio di scarto
automatico. Un'identità persistente vera resta `DA_PIANIFICARE` per la fase con
archivio: qui non si inventa nessuno storage.

Effetto: §8J e §6.2.

### D5 — Identificativi assenti: quali bloccano

La revisione 1 limitava la decisione al destinatario. Dopo R3A-01 riguarda
**tutti** gli identificativi che il parser lascia passare: `chave`, `numero`,
`serie`, `cnpj`/`nome` dell'emittente e del destinatario (tabella §3.4).

- **(a) asimmetrica** (proposta): solo `chave` vuota blocca
  (`IDENTIDADE_INSUFICIENTE`, F3A-23a), perché da essa dipende l'identità della
  riga; `numero` e i quattro campi di identità di emittente e destinatario
  producono registrazioni `REVISAR` con avviso dedicato (F3A-23b); **`serie` è
  l'eccezione dichiarata**: non è usata dall'allocazione, non entra nell'hash e
  non genera avviso (§3.4). Chi volesse anche `serie` fra i campi segnalati sta
  scegliendo un'altra variante, non precisando questa.
- (b) tutti bloccanti: coerente con la severità di C10 sugli importi, ma rende
  non importabile una NF-e altrimenti valida per un nome vuoto.
- (c) tutti non bloccanti: anche la chiave vuota passa; l'hash perde senso
  perché il payload contiene una chiave vuota.
- (d) rendere obbligatori i campi in fase 2: allarga il perimetro di una fase
  già chiusa e audita, e andrebbe deciso come mandato separato.

Effetto: F3A-23a/b. Nessuna NF-e reale nota presenta il caso: decisione
preventiva, ma le fixture d'audit mostrano che il parser lo consente oggi.

### D6 — Dominio monetario e input fuori dominio

Introdotta da R3A-05. **Approvata (a)**: rifiuto nominato
`ESCALA_MONETARIA_INVALIDA` con campo e valore, nessun arrotondamento
correttivo silenzioso. Non scelte: (b) riconciliazione `ERRO` con righe tutte
`REVISAR`, (c) parità col Python, che somma 0.02 contro un `vNF` di 0.01 senza
avvisi.

Dominio **nella formulazione approvata**: importo nel dominio ⇔ il suo
**valore** è multiplo esatto di `0.01`. `10.0000` e `10` sono ammessi quanto
`10.00`; `0.005` e `12.345` sono rifiutati. Verifica esatta:
`valor.normalize().scale() <= 2`. Soggetti: `vProd` di articolo, `vNF`,
componenti monetarie del totale, `vDup`. Esclusi: `qCom` e `vUnCom`. Il
controllo vale sugli **ingressi**; un `Dado` assente lo salta e non viene
sostituito da `0.00` (dettagli e tabella in §5.8).

Effetto: §5.8 e §8K. Con l'approvazione, F3A-10 diventa un invariante
**garantito nel dominio**, perché gli ingressi fuori dominio non arrivano
all'allocazione.

Decisioni **senza** effetto funzionale, prese qui e documentate: nomi dei tipi e
del modulo, uso di `Vec` invece di una mappa ordinata, testo dei messaggi di
avviso. **Non** rientra fra queste la forma della riconciliazione: determina
quali stati e quali coperture sono rappresentabili, quindi è parte del
contratto (F3A-28) e va approvata.

---

## 12. Proposta del primo blocco 3B

Perimetro implementabile subito dopo l'approvazione delle decisioni §11.

**File e logiche**

| File | Contenuto | Nota |
|---|---|---|
| `src/alocacao.rs` (nuovo) | `alocar_nfe(importada: &NfeImportada, classificador: &Classificador, plano: &PlanoContas) -> Result<AlocacaoNfe, ErroAlocacao>`; `AlocacaoNfe { lancamentos: Vec<Lancamento>, reconciliacao: Reconciliacao }`; raggruppamento, differenza, riparto, hash, riconciliazione a quattro esiti. **Nessun parametro di mapping DRE** (R3A-03): la verifica è rinviata alla 3D | gli avvisi del parser arrivano da `importada.avisos` e sono propagati in `reconciliacao.avisos_documento`: non serve un terzo campo in uscita |
| `src/lib.rs` | esporre il modulo | una riga |
| `src/modelos.rs` | solo `Reconciliacao` se si decide di tenerla nei modelli | nessuna modifica ai tipi esistenti |
| `tests/` | regressioni CLI solo quando esisterà un comando; il primo blocco può restare libreria + test unitari | nessun comando CLI in questo blocco |

**Nota sulla dipendenza per l'hash.** Verificato: `Cargo.toml` non contiene
nessuna libreria di hash, quindi `sha2` o equivalente sarebbe **nuova**. Il
divieto di nuove dipendenze vale per **questa fase di analisi**: non crea
un'autorizzazione a installare nulla ora, e non blocca la 3B. La proposta per
la 3B va presentata come un pacchetto unico — perimetro + dipendenza +
aggiornamento di `Cargo.toml`/`Cargo.lock` + test — da autorizzare insieme.
Alternative: (i) `sha2` con quel pacchetto; (ii) rimandare `hash_registro` a un
blocco successivo lasciandolo dichiaratamente assente in `campi_rinviati`;
(iii) un hash diverso già disponibile — **sconsigliata**, romperebbe la parità
di payload con lo storico.

**Test da creare** (nessuno esiste, nessuno è superato)

| Test | Verifica |
|---|---|
| `aloca_campione_canonico_em_tres_lancamentos` | F3A-01, -02, -03: 3 registrazioni, valori 10/20/5, ordine di comparsa |
| `riparto_das_rate_fecha_por_linha_e_por_coluna` | F3A-08, -10: §8A |
| `residuo_de_arredondamento_fica_na_ultima_coluna` | F3A-08, -21: §8B |
| `arredondamento_half_up_nao_bankers` | F3A-21: un valore a metà centesimo |
| `diferenca_positiva_vai_para_conta_de_acessorios` | F3A-05, -06: §8G riga 1 |
| `diferenca_negativa_vai_para_descontos_obtidos` | F3A-05: §8G riga 2 |
| `diferenca_nao_explicada_marca_revisar` | F3A-06: §8G riga 3 |
| `componente_parcial_nao_explica_diferenca` | F3A-07 (D2): §8G riga 4 |
| `rate_que_nao_fecham_geram_valor_inconsistente` | F3A-10, -11: §8C |
| `total_zero_com_rate_nao_vira_zero_tecnico` | F3A-09 (D1): §8D |
| `produto_nao_classificado_vai_para_conta_de_revisao` | F3A-13a: §8H |
| `classificacao_ambigua_marca_revisar_preservando_a_conta` | F3A-13b: la conta vincente non cambia |
| `nfe_sem_fat_continua_alocavel` | F3A-09, C10: §8I |
| `competencia_vem_da_data_de_emissao` | F3A-14 |
| `hash_estavel_na_reimportacao` | F3A-15, -25: §8J |
| `dre_fica_nao_assegnato_con_motivo` | F3A-18: `campi_rinviati` espone `NAO_ASSEGNATO`, nessun avviso sul mapping |
| `campi_rinviati_expone_centro_custo_e_observacoes` | §5.4: nessuna stringa vuota senza stato |
| `chave_vazia_nao_produz_lancamento` | F3A-23a (§3.4) |
| `numero_vazio_marca_revisar` | F3A-23b |
| `fornecedor_sem_identidade_marca_revisar` | F3A-23b |
| `empresa_sem_identidade_marca_revisar` | F3A-23b |
| `importo_fora_do_dominio_monetario_e_recusado` | F3A-26 (§5.8): `0.005` come `vProd` → `ESCALA_MONETARIA_INVALIDA` che nomina campo e valore |
| `zeros_finais_nao_squalificam_o_importo` | F3A-26: `vNF = 10.0000` e `vProd = 10.0000` **accettati**, trattati come `10.00` |
| `preco_unitario_com_mais_decimais_e_ammesso` | F3A-26: `vUnCom = 0.3333` ammesso, perché non è un importo da registrare |
| `dado_assente_non_viene_azzerato_per_passare_il_dominio` | F3A-26 + C10: `valor_tributos_inclusos` con status ≠ `Ok` salta il controllo e **non** diventa `0.00` |
| `payload_duplicado_no_documento_gera_aviso` | F3A-27: due righe con payload identico, conservate, `HASH_DUPLICADO_NO_DOCUMENTO` (§8L) |
| `copertura_parcial_com_det_sem_prod` | F3A-28: `DET_SEM_PROD` → `esito_copertura = PARCIAL`, atteso `NON_DISPONIVEL` |
| `documento_parcial_com_componente_necessaria_parcial` | §5.4 + F3A-07: quattro esiti distinti, `stato_documento = PARCIAL` se **D2(a)** |
| `total_netto_zero_com_lancamentos_nao_nulos` | **D1** caso 3, §8M: nessuna rata `0.00` su una registrazione da 100.00 in silenzio |
| `denominatore_zero_banale_e_accettato` | **D1** caso 2: tutto realmente zero → `esito_total = OK`, `esito_rate = OK` |

**La lista è il criterio, insieme alla matrice:** ogni contratto `F3A-nn` che
nomina un test "da creare" fa parte del criterio di accettazione del blocco,
anche se il nome compare solo nella matrice del §9.
| `data_pagamento_fica_ausente_com_motivo` | F3A-17 |
| `ordem_de_lancamentos_rate_e_avisos_e_deterministica` | F3A-22 |

**Criterio di accettazione del blocco**

1. Tutti i test sopra verdi, più i 113 esistenti **senza regressioni**.
2. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
   `git diff --check` puliti.
3. Campione canonico e cinque YAML con hash SHA-256 invariati.
4. Nessun valore monetario in `f64`; ogni arrotondamento esplicito.
5. Nessuno zero al posto di un'assenza. I quattro campi `String` legacy di
   `Lancamento` (`dre_linha_id`, `dre_linha_nome`, `centro_custo`,
   `observacoes`) **restano vuoti** perché il modello non cambia in questo
   blocco, ma ogni loro assenza è dichiarata e interrogabile in
   `Riconciliazione::campi_rinviati` con stato, fonte e motivo (§5.4). Un campo
   vuoto **senza** la voce corrispondente in `campi_rinviati` fa fallire il
   criterio.
6. Nessuna scrittura su disco o database; la funzione resta pura.
7. Ogni decisione §11 approvata è citata nel test che la protegge.

---

## 13. Limiti di questa specifica

| Limite | Stato |
|---|---|
| Suite `pytest` originale | **NON_ESEGUITO**: `python` non è nel `PATH` di questa sessione; il runtime Python 3.12.14 dell'applicazione **esiste** ma non ha `pytest` né `PyYAML`, e installare è vietato. Le attese sono lette nel sorgente dei test, poi confermate dall'audit eseguendo le **funzioni originali** con classificatore controllato: non è né `pytest`, né il test end-to-end originale, né una prova del loader YAML o di tutte le regole canoniche |
| Comportamento del Python su casi non coperti dai suoi test | **chiuso**: F3A-24 verificato dall'audit (`_describe_items` → `" (12.00)"`) e la sua policy **approvata** il 2026-10-16; F3A-23a/b sono decisioni approvate (D5), non osservazioni della fonte |
| `articoli_attesi` della copertura | **NON_DISPONIVEL**: la fase 2 non espone il numero di `<det>` letti; `esito_copertura = OK` significa "nessun avviso di scarto", non "tutti gli articoli allocati" (§5.4). Chiuderlo richiede un'evoluzione dichiarata di `NfeImportada` |
| Collisioni di payload dell'hash | il controesempio §6.2 usa un **classificatore controllato**: nessuna collisione è stata osservata sul campione o sulla configurazione canonica corrente |
| **Capacità numerica** (riscontro R3B-01 dell'audit 3B, residui A e B compresi) | il dominio di §5.8 vincola i **centesimi**, non la **grandezza**: esistono importi legittimi per D6 il cui prodotto o la cui somma `Decimal` non regge. `Decimal` va in panico sul prodotto del riparto e perde un centesimo in silenzio nella somma, `checked_add` compreso. La 3B calcola quindi in **centesimi interi `i128`**, con divisione e resto `checked` (`i128::MIN / -1` non è rappresentabile) e con il ritorno a `Decimal` che **riduce gli zeri finali** abbassando la scala fino a 0, così un importo intero grande non viene rifiutato per una scala imposta. Quando il risultato non è rappresentabile nemmeno così, restituisce `CapacidadeNumericaExcedida` nominando operazione e valori. È un **limite della rappresentazione dichiarato**, non un tetto di importo approvato: se servisse operare oltre, sarebbe una decisione nuova con la sua fonte e i suoi test |
| Versioni Python più recenti del commit fissato | non lette: fuori baseline per scelta del mandato |
| `mapeamento_dre.yaml` | **NON_LETTO dal codice Rust**: la sua validazione non esiste ancora; qui è solo misurato |
| `csv_columns.yaml` | fuori perimetro: l'import CSV non è parte del primo blocco NF-e |
| Deduplicazione persistente, aggiornamento dello storico, archivio | fuori perimetro: richiedono una fase con persistenza |
| Debiti di dominio già dichiarati (C13, `serde_yaml`, alias divergenti, copertura in `Dado<T>`, XSD) | **non toccati**, non sono prerequisiti di questo lavoro |

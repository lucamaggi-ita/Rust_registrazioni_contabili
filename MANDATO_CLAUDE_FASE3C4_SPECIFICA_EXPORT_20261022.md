# Mandato a Claude — 3C.4, specifica dell'export derivato e degli ingressi

Data: 2026-10-22. Destinatari: Claude (analisi), Codex (audit), Luca (uso e decisioni).
Repository: `Rust_registrazioni_contabili`.
Workspace: `C:\Users\Recanto do Sossego\Documents\Luca\GitHub\RUST\Rust_registrazioni_contabili`.
Metodo obbligatorio: `programmatore-bravo`, proporzionato al lotto.

## 1. Autorizzazione, obiettivo e consegna

Luca ha risposto **«procedi»** alla proposta di preparare il mandato 3C.4, raccomandando D11(a) e D12(a). Questo mandato registra le due scelte come **APPROVATE**. Non richiedere di approvarle nuovamente.

**Esegui ora un lotto di specifica, non di implementazione.** Consegna `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md`, pronta per audit e per un successivo mandato implementativo. Obiettivo concreto: definire il primo export consultabile del pre-libro e della revisione, rigenerabile dall'archivio JSON esistente, e delimitare gli adattatori realmente necessari.

Il passo seguente deve poter produrre un file utile a Luca e poi collegarsi alla CLI. Mantieni la specifica focalizzata: riusa i contratti approvati, cita le prove esistenti e dettaglia soltanto ciò che manca all'export. Nessun nuovo audit globale del porting né riscrittura delle specifiche storiche.

Sono autorizzate letture del progetto, verifiche locali delle dipendenze disponibili e piccole sonde su dati sintetici fuori dal repository. Sono esclusi codice applicativo, modifiche Cargo, installazioni, download di pacchetti, archivi reali, commit e push.

## 2. Baseline e precedenza delle fonti

- HEAD verificato alla preparazione: **`af5683d4522166d5283fdd1f206c91cb3e841dac`**. Due commit del lotto precedente: `9afd626` codice e `af5683d` documenti. Worktree pulito prima di creare questo mandato.
- Baseline tecnica già verificata: **337 test**, 319 lib + 3 binario + 15 integrazione; fmt/clippy/diff check superati. Qui è **RIUSATA**, non rieseguita: solo Markdown e analisi. Se dichiari nuove esecuzioni, eseguile davvero e specifica il perimetro.
- `SPECIFICA_FASE3C_PRELIBRO_REVISIONE_EXPORT.md`: requisiti generali e contratti export/ingressi, compresi F3C-06/10/14–21/24. Le fotografie storiche di stato non riaprono decisioni successive.
- `SPECIFICA_FASE3C3_REVISIONE_VERSIONI.md`: D8/D9 approvate, impronte e revisione implementate.
- `SPECIFICA_FASE3C3B_PERSISTENZA_ARCHIVIO.md`: D7 e D14–D17 approvate, codec e protocollo archivistico implementati e auditati; quattro correzioni chiuse.
- Il codice a questa baseline è la fonte del comportamento implementato: `arquivo`, `arquivo_io`, `prelivro`, `revisao`, `impressao`, `dinheiro`, modelli e piano validato.

Alcune righe del README e della specifica generale conservano stati precedenti («D7/D8/D9 aperte», «manca registrazione»). Indicale come fotografie superate e usa le decisioni successive. Non impiegare questo lotto per una pulizia generale della documentazione.

## 3. Decisioni approvate con questo mandato

| Decisione | Regola approvata | Conseguenza verificabile |
|---|---|---|
| **D11(a)** | **Import dello storico Excel rinviato**; il nuovo flusso non dipende da workbook storici non forniti | il primo export e il futuro flusso CLI funzionano senza importare schemi storici 26/31; nessuna compatibilità con file mai provati viene promessa |
| **D12(a)** | Importo non interpretabile in un ingresso grezzo → riga **`INCOMPLETO`**, testo originale e motivo conservati; i totali interessati sono dichiarati incompleti | nessuno zero tecnico, nessuna esclusione silenziosa e nessun totale completo dichiarato dove manca un importo |

D12 è una decisione funzionale già presa, ma non inventa un nuovo ingresso grezzo nel primo export. `Lancamento::valor` è un `Decimal`: una cella illeggibile non può essere rappresentata riempiendolo di zero. Definisci il confine di un futuro adattatore e la rappresentazione del valore originale/assenza; implementazione e test vanno assegnati al blocco che riceverà davvero quell'ingresso.

Gli stati grezzi sconosciuti e le transizioni aggiuntive vanno ricondotti ai contratti esistenti o marcati come proposte se introducono nuove scelte. D12 non autorizza a cambiare gli enum e le regole di revisione già auditati.

Restano approvate D7, D8/D9, D10(a), D13(a), D14–D17. L'archivio JSON resta autorevole, il workbook è un derivato. Una modifica al workbook non diventa automaticamente una decisione umana né una modifica all'archivio.

## 4. Primo risultato da specificare

Proponi un **primo lotto implementativo di solo export**, con ingresso tipizzato e dati già conservati. Raccomandazione di lavoro: XLSX come primo formato per la consultazione umana; verifica fattibilità e costi prima di fissare dipendenze. La scelta concreta del formato e dello schema deve comparire come proposta verificabile, non come approvazione implicita di una libreria.

Definisci un percorso completo:

```text
archivio JSON letto e validato + lotto scelto + piano validato
  -> contesti originali ed eventi conservati
  -> validazione del pre-libro e applicazione della revisione esistenti
  -> modello dell'export
  -> file derivato
  -> rilettura indipendente e confronto dei contenuti
```

**Mantieni la barriera di `construir_pre_livro`.** Il validatore archivistico non attesta tutti gli invarianti della 3C.2. Riusare `aplicar_revisao` e le sue validazioni è preferibile a duplicare la composizione: dimostra la sequenza effettiva e indica dove ogni controllo avviene.

Specificare almeno:

1. **Diagnostico completo:** righe, documenti anche senza righe, riconciliazione con i quattro esiti, avvisi, esclusioni e riepiloghi con marcatura dei documenti PARCIAL/ERRO. Dichiarare quali informazioni sono colonne, fogli o dettagli collegati, senza copiarle in totali impropri.
2. **Estratto revisionato:** usare la selezione della vista 3C.3.a, con approvazione umana **attiva** e filtro D10(a). Il solo stato tecnico `APROVADO` o l'estratto meno restrittivo della 3C.2 non sono sufficienti. Coprire revoca, sanatoria revocata e ri-approvazione.
3. **Provenienza:** lotto, documento, versione/impronta, identità del piano e stato della revisione consultabili. Non inventare una sequenza storica dei documenti. Distinguere esiti tecnici e decisioni umane.
4. **Schema versionato:** fogli/colonne, tipi, obbligatorietà, assenze e ordinamento dichiarati. Non imporre per inerzia il layout Python a 26 o 31 colonne. Mappare le differenze necessarie e indicare la compatibilità realmente richiesta.
5. **Scrittura:** destinazione esplicita, temporaneo e sostituzione controllata; in caso di errore prima del commit il file preesistente resta intatto. Nessuna modifica all'archivio sorgente durante l'export. Dichiarare il comportamento con output esistente o aperto.

Non commissionare automaticamente due writer. Se proponi anche CSV, spiega l'utilità aggiuntiva e il contratto specifico: quoting e separatori da soli non impediscono a Excel di reinterpretare testi, identificativi o formule.

## 5. Fedeltà dei dati e numeri

Riusa F3C-16/17/18/20/21 e i contratti monetari vigenti. Non cambiare le grammatiche delle impronte per accomodare un formato di presentazione.

- Importi esatti e scale conservati; niente percorso monetario attraverso `f64`, niente correzioni o arrotondamenti nascosti. Quantità/prezzi non vanno forzati ai centesimi.
- CNPJ/CPF, chiavi, numeri documento e conti sono testi con zeri iniziali.
- Testi che iniziano con `=`, `+`, `-`, `@`, virgolette, ritorni a capo e Unicode restano dati. Per XLSX verificare il tipo effettivo della cella e l'assenza di formule, non solo l'aspetto a video. Non modificare silenziosamente il testo originale per renderlo sicuro.
- Assenze, motivi e stati restano distinguibili da zero e stringa vuota; JSON delle rate, scadenze assenti e documenti senza righe non vengono persi.
- Date/istanti devono conservare la precisione e l'offset richiesti dalla revisione. Non assumere che una cella data Excel conservi il dominio di `DateTime<FixedOffset>`.
- I riepiloghi esistenti sono la fonte dei calcoli; ulteriori somme riusano l'aritmetica condivisa. Nessun calcolo contabile duplicato nel writer o delegato a formule Excel senza contratto.
- Colonne numeriche di comodo, formule o perdita della calcolabilità delle colonne testuali sono scelte da rendere esplicite. Non sacrificare l'importo esatto per ottenere una cella sommabile.

La rilettura per il test di fedeltà **non** autorizza l'importazione operativa dello storico o delle modifiche al workbook. Il round-trip di prova deve confrontare valori e metadati, senza reinserirli nell'archivio.

Definisci il determinismo dei contenuti e dell'ordine; distinguilo dall'identità dei byte di uno ZIP XLSX, che può contenere metadati variabili. Non dichiarare identità binaria senza una prova e un contratto sui metadati.

## 6. Dipendenze e prove di fattibilità

Verifica lo stato **attuale** di Cargo e della cache: le vecchie specifiche dichiaravano assenti i crate XLSX, ma è una fotografia storica. Proponi nome, versione, feature minime, API necessarie e costo di risoluzione della sola dipendenza indispensabile.

Con D11(a) non è automaticamente necessario introdurre un lettore XLSX Rust: per il test indipendente valuta gli strumenti già disponibili, per esempio Python/openpyxl, verificando runtime e provenienza degli import. Non installare nulla e non usare il working tree Python corrente come se fosse la fonte canonica fissata.

Se una capacità non si può provare senza una dipendenza non disponibile, marcarla **NON_VERIFICATA** e indicare la prova da eseguire nel mandato implementativo. Documentazione ufficiale e sorgenti locali sono fonti ammesse; leggere una API non equivale a eseguirla.

Nessuna autorizzazione a nuovi crate, aggiornamenti Cargo o download deriva da questo mandato. La proposta implementativa dovrà nominarli esplicitamente insieme ai file da modificare.

## 7. Prove da pianificare e criteri di accettazione

Matrice compatta contratto → caso → atteso → blocco responsabile. Coprire almeno i casi che possono cambiare numeri, perdere righe o attribuire un'approvazione inesistente:

- due righe con hash C23 uguale, nessuna scartata; documenti senza righe presenti;
- azienda/competenza/conto distinti, nomi dal piano e riepiloghi coerenti;
- importi estremi, centesimi, scale differenti, quantità, zero reale e assenza;
- identificativi con zeri iniziali, testi formula e Unicode;
- documento PARCIAL/ERRO, righe approvate/revocate/senza provenienza, esclusioni motivate;
- rate/scadenze e riconciliazione integra, senza somme sostitutive inventate;
- file già esistente, file bloccato, errore prima della sostituzione: sorgente e precedente output invariati;
- rilettura indipendente del file esportato e confronto campo per campo; provenienza e limiti del lettore dichiarati;
- rigenerazione dallo stesso archivio/piano e scelta del lotto esplicita;
- D12: casi e rappresentazione pianificati nel blocco di ingresso effettivo, non test fittizi su un `Decimal` che non può essere illeggibile.

Non è una quota di test. I test restano **PIANIFICATI** fino all'implementazione. Per sola analisi non rieseguire automaticamente i 337 test: riusa la baseline e verifica il perimetro invariato. Le eventuali sonde eseguite vanno separate dai test del prodotto.

## 8. Confini e consegna finale

**File ammessi:** il nuovo `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` e un'eventuale breve appendice di esito a questo mandato. Non modificare altri documenti, sorgenti, test, Cargo, YAML, campioni o repository Python. Sonde sintetiche fuori dal repository, con percorsi dichiarati.

Fuori dal primo lotto proposto: import storico Excel (D11), correzioni provenienti da workbook, CLI 3C.5, DRE 3D, autenticazione, nuove regole contabili, riscrittura dell'archivio e risoluzione dei limiti già dichiarati della persistenza. Non presentarli come prerequisiti impliciti dell'export.

La specifica deve contenere: obiettivo operativo; fonti/decisioni approvate; schema proposto; composizione delle API reali; fedeltà e protocollo IO; dipendenze proposte; matrice delle prove; file del primo lotto implementativo; criteri di accettazione; limiti e scelte ancora realmente necessarie. Privilegiare tabelle e riferimenti puntuali alle specifiche esistenti, evitando di ripubblicarle.

Nel resoconto distinguere **LETTO**, **ESEGUITO**, **RIUSATO**, **PROPOSTO**, **NON_VERIFICATO**. Registrare D11(a)/D12(a) come approvate in forza di questo mandato. Non trasferire l'etichetta di approvazione ai nuovi contratti di formato o alle dipendenze soltanto proposte.

Prima dell'handoff eseguire `git status -sb`, `git diff --name-status`, `git diff --cached --name-status` e `git diff --check`. Classificare ogni file con origine, stato e destino. I nuovi Markdown restano continuità nominata, pronti per audit Codex; commit/push richiedono autorizzazione separata.

**Questo mandato avvia soltanto la specifica 3C.4. Non avvia l'implementazione né chiude la fase 3C.**

---

## 18. Appendice di esito — eseguito da Claude il 2026-10-22

| Voce | Valore |
|---|---|
| Consegna | `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md`, unico file di progetto creato |
| Stato | **SPECIFICA COMPLETA, IN ATTESA DI AUDIT**. Nessun contratto approvato tranne D11(a) e D12(a), registrate dal mandato |
| HEAD alla stesura | `af5683d4522166d5283fdd1f206c91cb3e841dac`, worktree pulito prima di iniziare |
| Baseline dei test | **RIUSATA**, non rieseguita: 337 test del lotto 3C.3.b.1. Questo lotto non tocca codice |
| Codice del progetto | invariato: nessun file sorgente, test, `Cargo.*`, YAML o campione modificato |
| Sonde eseguite | tre, su dati sintetici fuori dal repository, in `...\scratchpad\sonda_3c4\`: XLSX non compresso scritto a mano e riletto; celle numeriche contro celle di testo; convenzione `_xHHHH_` |
| Verifiche di dipendenza | stato reale di `Cargo.toml`, `Cargo.lock` e cache del registro; nessuna installazione, nessun download |
| Decisioni nuove | **proposte** D18-D23, nessuna approvata da questo documento |
| Contratti nuovi | `EXP-01...EXP-29`, tutti proposti; prove `TX-01...TX-28`, tutte pianificate |

### 18.1 Le due misure che hanno cambiato il piano

1. Nessun crate XLSX **e** nessun crate ZIP sono in cache, quindi qualunque
   libreria richiederebbe un download. Ma un XLSX **non compresso** scritto a
   mano si è dimostrato leggibile: la strada senza dipendenze esiste, e la
   proposta D18 la raccomanda.
2. La difesa consueta contro la convenzione `_xHHHH_` **corrompe** il dato con
   il solo lettore eseguibile qui. D22 è quindi una proposta con una prova
   rinviata, non una regola applicata per prudenza.

### 18.2 Limiti del lavoro consegnato

Nessuna riga di writer esiste; le prove `TX-nn` sono pianificate. Il
comportamento di Microsoft Excel resta `NON_VERIFICATO` in quattro punti, con
le prove nominate. La stima di 500-700 righe della strada (a) è una proposta,
non una misura. Nessuna prova su volumi reali.

### 18.3 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4 §1 | nuovo, completo | audit Codex; commit documentale dopo audit e autorizzazione |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex | nuovo, con questa appendice | stesso commit documentale |

Nessun commit, push, amend, rebase, reset, restore, checkout, clean o stash
eseguito. Sonde e script restano fuori dal repository.

---

## 19. Esito dell'audit del 2026-10-23 e correzioni — 2026-10-23

Audit di Codex sulla specifica: **da correggere**, cinque riscontri
`R3C4-01…05`. Tutte le correzioni sono dentro il perimetro di questo mandato:
soltanto `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` e questa appendice.

| Voce | Valore |
|---|---|
| Stato | **SPECIFICA CORRETTA, revisione 2, in attesa di audit mirato** |
| Riscontri | cinque, tutti chiusi: §18 della specifica |
| Metodo | riprodotti **prima** di correggere; le tre riproduzioni che richiedevano il codice reale sono in §5.4 della specifica |
| Sonda | crate **esterno** al repository con dipendenza di percorso al progetto, compilato `--offline`, più due parser XML indipendenti. Nessun file del progetto modificato |
| Contratti nuovi | `EXP-30…EXP-45`, tutti proposti |
| Prove nuove | `TX-29…TX-36`, tutte pianificate; `TX-04`, `TX-28` e i criteri di §13 corretti |
| Decisione nuova | **D24**, lucchetto cooperativo sull'export, proposta |
| Codice del progetto | invariato: nessun sorgente, test, `Cargo.*`, YAML o campione toccato |
| Decisioni | nessuna riaperta: D11(a), D12(a) e le precedenti restano approvate |

### 19.1 Un fatto misurato oltre il rilievo

Il riscontro `R3C4-02` diceva che i testi umani non sono limitati al
repertorio XML. La misura ha trovato di piu': U+0000 **non e'
rappresentabile in XML 1.0 in nessuna forma**, perche' due parser indipendenti
rifiutano anche il riferimento numerico `&#0;` come «reference to invalid
character number». Per questo la correzione non e' una codifica ma un errore di
rappresentabilita' nominato prima del punto di commit (`EXP-33`).

Nel rapporto ho anche corretto una mia imprecisione della revisione 1:
«proteggere corrompe il dato» era una misura su `openpyxl` presentata come un
giudizio sulla codifica. Con un lettore che decodifica, la protezione sarebbe
corretta: §7.4.1 separa ora il dato logico dalla codifica sul file.

### 19.2 Stato Git alla riconsegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4 §1, piu' i cinque riscontri | nuovo, revisione 2, corretto | audit mirato Codex; commit documentale dopo audit e autorizzazione di Luca |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18 e §19 | continuita' nominata | stesso commit documentale |

Nessun commit, push, amend, rebase, reset, restore, checkout, clean o stash
eseguito. Sonde e script restano fuori dal repository.

---

## 20. Esito dell'audit mirato e correzione dei due residui — 2026-10-23

Audit mirato di Codex sulla revisione 2: **tre rilievi chiusi, due residui
circoscritti**. Entrambi corretti qui, dentro il perimetro del mandato:
soltanto `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` e questa appendice.

| Voce | Valore |
|---|---|
| Stato | **SPECIFICA CORRETTA, revisione 3, in attesa del controllo finale mirato** |
| Chiusi dall'audit mirato | `R3C4-02`, `R3C4-04`, `R3C4-05`, come specifica; i test del prodotto restano pianificati |
| Residui corretti | `R3C4-01` autore dell'approvazione attiva; `R3C4-03` confronto dei nomi riservati |
| Metodo | riprodotti **prima** di correggere, con un secondo binario della sonda esterna: §5.5 della specifica |
| Contratti nuovi | `EXP-46...EXP-50`, tutti proposti |
| Prove | `TX-34b` nuova; `TX-21`, `TX-29`, `TX-33`, `TX-34` estesi o corretti |
| Allineamenti accolti | quattro, in §18.2 della specifica |
| Codice del progetto | invariato |
| Decisioni | nessuna riaperta. D18-D24 restano proposte |

### 20.1 I due residui, misurati

**Autore dell'approvazione attiva.** La revisione 2 prendeva l'ultima
approvazione applicata. Il codice dice il contrario: una seconda `Aprovar`
«fica na trilha e nao substitui a primeira», con l'avviso
`APROVACAO_REPETIDA`. Misurato: la regola sbagliata attribuiva a Bob invece che
ad Alice, e a Eve invece che a Dan. Corretto con `EXP-46`, che definisce
l'approvazione attiva come la prima dopo l'ultima revoca efficace, e `EXP-47`,
che dà alle ripetizioni colonne e nome propri.

**Confronto dei nomi riservati.** La revisione 2 attribuiva a
`fs::canonicalize(parent)` l'insensibilità alle maiuscole del nome del file.
Per un nome che non esiste ancora è falso. Misurato: il confronto lessicale
accetta `ARQUIVO.JSON.LOCK`, e la scrittura finisce in `arquivo.json.lock`,
occupando il nome del lucchetto dell'archivio con byte XLSX. Corretto con
`EXP-48`: chiave di confronto insensibile alle maiuscole, famiglia riconosciuta
per prefisso, nomi originali non riscritti, direzione dell'errore per eccesso.

### 20.2 Stato Git alla riconsegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 3 | due residui corretti | controllo finale mirato; decisioni di Luca su D18-D24; poi commit documentale autorizzato |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18, §19 e §20 | continuità nominata | stesso commit documentale |

Nessun commit, push, amend, rebase, reset, restore, checkout, clean o stash
eseguito. Sonde e script restano fuori dal repository.

---

## 21. Esito finale: audit della specifica superato — 2026-10-23

| Voce | Valore |
|---|---|
| Esito di Codex | **AUDIT DELLA SPECIFICA SUPERATO**: i due residui chiusi, `R3C4-02`/`04`/`05` confermati chiusi, nessun nuovo rilievo tecnico bloccante |
| Che cosa attesta | che la specifica è **implementabile** |
| Che cosa non attesta | l'approvazione di D18-D25; l'autorizzazione di una dipendenza; il collaudo del prodotto, che non esiste |
| Stato del documento | revisione 4, con la precisazione recepita |
| Prossimo passo | le decisioni di Luca su D18-D25, poi il mandato implementativo 3C.4.1 |

### 21.1 La precisazione recepita

Il rapporto finale ha corretto una mia imprecisione nella presentazione dei
costi: avevo scritto che il lucchetto «porta con sé la destinazione di zero
byte». Sono due meccanismi distinti:

- la **rivendicazione** di `EXP-43` rende atomico `Recusar` e può lasciare una
  destinazione di zero byte;
- il **lucchetto** di `EXP-44` coordina esecuzioni cooperanti e rende coerente
  la conservazione del precedente con `Substituir`.

Rifiutare D24 **non** elimina il residuo a zero byte. L'alternativa a
`EXP-43`, già descritta in §8.2, ha ora un nome proprio: **D25**. Le due
scelte sono indipendenti e si combinano in quattro modi; §8.2 indica la
combinazione minima coerente con un primo export monoutente senza scegliere al
posto di Luca.

Nessuna di queste proposte introduce account, ruoli o autenticazione: il
perimetro multiutente resta fuori progetto. Non è una riprogettazione e non
richiede un altro ciclo di audit: la seconda strada era già scritta, le
mancava l'etichetta.

### 21.2 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 4 | audit superato; proposte D18-D25 da decidere | registrazione delle scelte; commit documentale dopo autorizzazione di Luca |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18-§21 | continuità nominata | stesso commit documentale |

Nessun commit, push, amend, rebase, reset, restore, checkout, clean o stash
eseguito. Sonde e script restano fuori dal repository. Nessun file del progetto
modificato in questo giro, oltre ai due Markdown.

---

## 22. Decisioni di Luca su D18-D25 — 2026-10-23

Registrate nella specifica in §3.3, con l'effetto su ogni sezione in §18.4.

| Decisione | Esito |
|---|---|
| **D18** | **rust_xlsxwriter**, non il writer scritto a mano. Il mandato implementativo deve indicare versione, funzionalità abilitate e dipendenze aggiunte; restano i contratti di conservazione dei dati e la prova di rilettura indipendente |
| **D19** | approvata: schema `FANTOZZI-EXPORT/v1` |
| **D20** | approvata con la regola di §7.2.1 |
| **D21** | approvata: tutti gli eventi, senza scarti silenziosi |
| **D22** | obiettivo approvato, con **condizione**: codifica verificata con rust_xlsxwriter, nessuna trasformazione manuale senza prova, `_x0041_` identico alla rilettura, compatibilità Excel verificata prima di dichiararla stabile |
| **D23** | approvata: determinismo di contenuti e ordine, non dei byte ZIP |
| **D24** | approvata: lucchetto cooperativo |
| **D25** | **(b)**: `Recusar` con controllo di esistenza, senza rivendicazione |

### 22.1 Che cosa le decisioni hanno cambiato

D18 è la decisione con più conseguenze. Il writer scritto a mano esce dal
documento: la stima di 500-700 righe decade, `src/xlsx.rs` diventa un
adattatore verso la libreria, e la sonda di §5.1 resta come fatto sul formato
senza essere più la giustificazione di una strada. Due conseguenze dichiarate:
l'acquisizione della dipendenza richiede la rete **una volta** - non a ogni
compilazione e mai a tempo di esecuzione - e §10.5 elenca le quattro proprietà della libreria da **misurare**
prima di fidarsi, con `TX-38` messa per prima nell'ordine di lavoro perché è il
passo che può far cadere D18.

D25(b) elimina la rivendicazione del nome finale: `EXP-43b` controlla
l'esistenza invece di crearla, quindi nessuna destinazione di zero byte è
possibile e `EXP-49` decade. Il prezzo è la non atomicità di `Recusar` contro
chi ignora il lucchetto, e `EXP-52` lo scrive nel foglio `limites` del file.

D22 sposta la responsabilità della codifica sulla libreria: `EXP-17` vieta
trasformazioni nostre, `EXP-51` rende verificabile la condizione di Luca, e
`TX-37` la prova. `EXP-33` resta, perché rifiutare un carattere non
rappresentabile non è trasformarlo.

### 22.2 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 5 | audit superato, decisioni registrate, pronta per il mandato implementativo | commit documentale dopo autorizzazione di Luca |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18-§22 | continuità nominata | stesso commit documentale |

Nessun commit, push, amend, rebase, reset, restore, checkout, clean o stash
eseguito. Nessun file di codice, test, Cargo, YAML o campione modificato.
Sonde e script restano fuori dal repository.

---

## 23. Controllo mirato delle decisioni e riscontro R5-01 — 2026-10-23

| Voce | Valore |
|---|---|
| Esito del controllo | le scelte D18-D25 risultano registrate; **un allineamento necessario** prima di commissionare il codice: `R5-01` |
| Stato del documento | specifica a revisione 6 |
| Decisione nuova | **D26, bloccante, non ancora presa**: non registrata come approvata |
| Contratti sospesi | `EXP-07`, `EXP-39`, `EXP-40`, più la prova `TX-04`, in attesa di D26 |
| Codice del progetto | invariato |

### 23.1 R5-01, verificato alla fonte

Il rilievo dice che D18 e il divieto assoluto di `f64` sono incompatibili. **Non
l'ho preso per buono dal verbale**: ho letto la documentazione ufficiale, fonte
ammessa dal mandato §6.

| Fatto letto | Conseguenza |
|---|---|
| `Worksheet::write_number(&mut self, row, col, number: impl Into<f64>)` | ogni cella numerica passa per `f64` |
| `write(..., data: impl IntoExcelData)` accetta `u64`/`i64` con perdita oltre ±999 999 999 999 999 | nessuna via d'uscita per gli interi grandi |
| con la funzionalità `rust_decimal`, `write` accetta un `Decimal` **solo se rappresentabile come `f64`** | nemmeno il supporto decimale evita la conversione |
| `write_string(..., string: impl Into<String>)` non passa per `f64` | la colonna **testuale autorevole** resta al sicuro |

Il rilievo è fondato. `EXP-07` come divieto assoluto, `EXP-39` come identità
dei caratteri del `<v>` ed `EXP-40` non reggono con questa libreria: non sono
proprietà da misurare: è un conflitto che si legge nella firma.

**D26** apre le due strade in §7.2.3 della specifica: (a) un'eccezione nominata
al solo confine di scrittura della cella numerica, con le cinque condizioni del
verbale riportate senza ammorbidirle; (b) rinunciare alla colonna numerica,
rivedendo D20. La scelta è di Luca, e l'eccezione **non** è implicita
nell'approvazione di D18.

Una conseguenza utile: la soglia di 15 cifre significative di D20, scelta prima
di sapere quale libreria si sarebbe usata, coincide con la frontiera che la
documentazione dichiara per `u64`. Nel dominio eleggibile il **valore**
sopravvive al passaggio per `f64`; la **scala** no, ed è ciò che la condizione
4 di D26(a) vieta di promettere.

### 23.2 Le tre precisazioni accolte

| Precisazione | Dove |
|---|---|
| D22: una **codifica interna** diversa con testo riletto identico è conforme; solo un **testo logico** diverso è una prova fallita. `TX-37` confronta il testo letto, non il XML | §7.4.1 |
| l'affermazione sull'offline va circoscritta all'acquisizione: la rete serve una volta, non a ogni compilazione e mai a tempo di esecuzione | §10.3, §17 |
| `EXP-29`: il controllo mirato **consiglia** la prova esterna ripetibile; è una raccomandazione, non un'approvazione, e non introduce un runtime Python nel programma Rust | §10.2 |

### 23.3 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 6 | allineamento recepito; **D26 da decidere** | decisione di Luca, poi commit documentale autorizzato |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18-§23 | continuità nominata | stesso commit documentale |

Nessun commit, push o modifica a codice, test, Cargo, YAML o campioni. Nessun
crate scaricato: la verifica di R5-01 è una **lettura** di documentazione.

---

## 24. D26(a) approvata da Luca — 2026-10-23

| Voce | Valore |
|---|---|
| Decisione | **D26(a)**: `f64` consentito **soltanto** nel confine di scrittura della cella numerica di comodo, con le cinque condizioni del verbale. L'alternativa, rinunciare alla colonna, è stata valutata e scartata |
| Stato del documento | specifica a revisione 7 |
| Decisioni aperte | **nessuna**: D18-D26 sono tutte decise |
| Contratti riallineati | `EXP-07` con una eccezione nominata; `EXP-39` per valore nel dominio eleggibile e non per caratteri; `EXP-40` **decaduto**; `EXP-53` nuovo; `TX-04` sdoppiato |
| Codice del progetto | invariato |

### 24.1 Il riallineamento, nei suoi termini

| Contratto | Prima | Dopo |
|---|---|---|
| `EXP-07` | nessun `f64` in nessun punto del writer | nessun `f64`, **con una sola eccezione nominata**: la scrittura della cella numerica. Il rapporto del lotto deve elencare ogni occorrenza di `f64` nei tre moduli con la sua giustificazione |
| `EXP-39` | cifre di `<v>` identiche carattere per carattere al testo | **uguaglianza del valore** nel dominio eleggibile delle 15 cifre significative; nessuna promessa sulla scala nella cella numerica |
| `EXP-40` | il `<v>` si ottiene dal testo, non da una conversione | **decaduto**: non è ottenibile con questa libreria |
| `EXP-53` | — | **nuovo**: l'eccezione vive in **una sola funzione** di `xlsx.rs`, dichiarata nel codice con il motivo e il rinvio a D26(a) |
| `TX-04` | un confronto | **due**: esatto carattere per carattere sulla colonna testuale autorevole, per valore sulla cella numerica, più l'elenco delle occorrenze di `f64` |

**Che cosa l'eccezione non tocca**, perché è la condizione 2 della decisione:
calcoli, riconciliazioni, archivio, impronte e produzione del testo autorevole
restano senza `f64`. Il nucleo monetario non è stato modificato né messo in
discussione.

### 24.2 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 7 | audit superato, D18-D26 decise, nessuna decisione aperta | commit documentale dopo autorizzazione di Luca |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18-§24 | continuità nominata | stesso commit documentale |

Nessun commit, push o modifica a codice, test, Cargo, YAML o campioni. Nessun
crate scaricato.

---

## 25. EXP-29 approvata e allineamento di §11.1 — 2026-10-23

| Voce | Valore |
|---|---|
| Decisione | **`EXP-29` approvata**: prova indipendente ripetibile con Python e `openpyxl` **già disponibili**, **senza installazioni** |
| Allineamento | §11.1 non rinvia più a `EXP-40`, decaduto; §11.2 nuova descrive il controllo dell'unica funzione autorizzata a usare `f64`, secondo `EXP-53` |
| Stato del documento | specifica a revisione 8 |
| Decisioni aperte | **nessuna**: D18-D26 ed `EXP-29` sono decise |
| Resta da fissare | la versione con le funzionalità di `rust_xlsxwriter`, che il mandato implementativo registra con l'acquisizione |
| Codice del progetto | invariato |

### 25.1 EXP-29, come è registrata

Lo script vive fuori dal crate e usa l'interprete e `openpyxl` dell'utente,
già presenti. Quattro requisiti, in §10.2 della specifica: **ripetibile**, con
percorso e invocazione dichiarati; **esito controllato**, con codice di uscita
diverso da zero in caso di discordanza; **tracciabile**, con versione e
provenienza dell'interprete e del lettore nel rapporto; **confinato**, perché
`openpyxl` non entra in `Cargo.toml`, non viene installato e non introduce un
runtime Python nel programma Rust.

### 25.2 §11.2, il controllo dopo D26(a)

Dopo D26(a) il divieto di `f64` non è più assoluto, quindi non si verifica
cercando «zero occorrenze»: si verifica che le occorrenze siano **esattamente**
quella autorizzata. Quattro passi, obbligo di rapporto e non test superato:
inventario con file e riga di `f64`, `as f64`, `to_f64`, `parse::<f64>`,
`into()` verso `f64` e `write_number` nei tre moduli nuovi; confronto, che deve
dare **una sola** funzione, quella di `xlsx.rs` che scrive la cella numerica;
commento nel codice che dichiara l'eccezione e rinvia a D26(a); e nessuna
occorrenza in `export.rs`, dove il testo autorevole si produce con
`Decimal::to_string()`.

Perché un'ispezione e non un test: un test che leggesse i sorgenti proverebbe
il testo del codice, non il comportamento, e passerebbe anche con una
conversione nascosta dietro un nome diverso. Il comportamento lo copre `TX-04`;
l'ispezione copre ciò che `TX-04` non può vedere, cioè una conversione innocua
oggi in un punto che domani diventa autorevole.

### 25.3 Stato Git alla consegna

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `SPECIFICA_FASE3C4_EXPORT_DERIVATO.md` | mandato 3C.4, revisione 8 | completa, nessuna decisione aperta | **commit documentale unico**, autorizzato da Luca il 2026-10-23, con push ordinario |
| `MANDATO_CLAUDE_FASE3C4_SPECIFICA_EXPORT_20261022.md` | Codex, con le appendici §18-§25 | continuità nominata | stesso commit |

Nessuna modifica a codice, test, `Cargo.*`, YAML o campioni in questo giro.

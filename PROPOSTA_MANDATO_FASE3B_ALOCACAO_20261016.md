# Proposta di mandato — fase 3B: allocazione NF-e in registrazioni

> **APPROVATO DA LUCA IL 2026-10-16**, con tre decisioni aggiuntive registrate
> qui:
>
> 1. **mandato approvato** e implementazione **autorizzata** nel perimetro
>    descritto, mantenendo D1-D6;
> 2. **SHA-256: opzione (i)**, `sha2` da crates.io, con download e aggiornamenti
>    **mirati** di `Cargo.toml` e `Cargo.lock`, senza aggiornamenti
>    generalizzati delle altre dipendenze;
> 3. **F3A-24 approvato**: articolo conservato, importo e riferimenti
>    preservati, avviso `ITEM_SEM_DESCRICAO` e stato `REVISAR`, senza inventare
>    descrizioni.
>
> **Commit e push restano da autorizzare**, dopo l'audit di Codex. Nessun avvio
> di 3C o 3D.
>
> Stato dell'esecuzione: **implementazione completata il 2026-10-16**, esiti nel
> §11. **Audit tecnico superato il 2026-10-16** (tre giri: R3B-01, R3B-02 e
> R3B-03 chiusi). Resta la sola registrazione in Git, da autorizzare.

| Voce | Valore |
|---|---|
| Data | 2026-10-16 |
| Baseline Rust | `3a24ba0f1abb3b71e4bed650bf246c72edb1a485`, suite 113 test verdi |
| Specifica di riferimento | `SPECIFICA_FASE3A_REGISTRAZIONI.md` revisione 4, audit tecnico superato |
| Fonte Python canonica | `Programma_registrazioni-contabili` @ `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` |
| Decisioni di dominio | **D1(a)…D6(a) approvate da Luca il 2026-10-16** |

## 1. Che cosa è approvato e che cosa manca

| Elemento | Stato |
|---|---|
| Comportamento D1-D6 | **APPROVATO** 2026-10-16 (scheda di raccomandazione di Codex, variante (a), D6 in centesimi) |
| Perimetro e contratti della specifica rev. 4 | **APPROVATI** 2026-10-16 con questo documento |
| F3A-24, articolo senza descrizione | **APPROVATO** 2026-10-16: conservato, con avviso e `REVISAR` |
| Avvio dell'implementazione 3B | **AUTORIZZATO** 2026-10-16; eseguito lo stesso giorno |
| Dipendenza SHA-256 e modifica di `Cargo.toml`/`Cargo.lock` | **AUTORIZZATA** 2026-10-16, opzione (i), solo mirata |
| Commit e push | **NON AUTORIZZATI**: Codex audita prima |
| Fasi 3C e 3D | **NON AVVIATE** |

## 2. Decisioni approvate, in forma eseguibile

| ID | Regola approvata | Dove si vede nel risultato |
|---|---|---|
| **D1(a)** | Denominatore zero: riparto definito **solo** se tutte le registrazioni **e** tutte le rate sono `0.00`; altrimenti `RIPARTO_IMPOSSIBILE`, nessuna registrazione parziale. Nessuna rata nel documento ⇒ `esito_rate = NaoAplicavel`, non un rifiuto | errore tipizzato; §8D e §8M della specifica |
| **D2(a)** | Componente necessaria con `Dado` non `Ok` ⇒ `esito_componenti = PARCIAL`, `stato_documento = PARCIAL`, registrazione di differenza `REVISAR`, motivo che nomina le componenti. `esito_total` resta la sua uguaglianza | §5.4, §8G riga 4 |
| **D3(a)** | Riparto rate che non chiude ⇒ `esito_rate = ERRO` e **tutte** le registrazioni `REVISAR`, senza toccare conti, priorità o valori. Gli avvisi informativi del parser **non** diventano errori globali | §8C |
| **D4(a)** | Hash a 4 campi identico al Python; payload ripetuto nello stesso documento ⇒ avviso `HASH_DUPLICADO_NO_DOCUMENTO`, righe conservate e `REVISAR`, **mai** scarto automatico; nessuna deduplicazione persistente | §6.2, §8L |
| **D5(a)** | `chave` vuota ⇒ `IDENTIDADE_INSUFICIENTE`, nessuna registrazione. `numero`, CNPJ/nome di emittente e destinatario vuoti ⇒ registrazioni `REVISAR` con avviso dedicato. `serie`: nessun effetto, nessun avviso | §3.4 |
| **D6(a)** | Importo nel dominio ⇔ valore multiplo esatto di `0.01`: `10.0000` e `10` ammessi, `0.005` e `12.345` rifiutati con `ESCALA_MONETARIA_INVALIDA` che nomina campo e valore. Nessun arrotondamento correttivo. Soggetti: `vProd`, `vNF`, componenti monetarie, `vDup`; esclusi `qCom` e `vUnCom`. Controllo sugli **ingressi**; un `Dado` assente lo salta e non diventa `0.00` | §5.8, §8K |

## 3. Perimetro

**Dentro**

1. Modulo di allocazione: raggruppamento per conto, registrazione di differenza, riparto delle rate, hash, riconciliazione a quattro esiti.
2. Funzione **pura**: nessuna scrittura su disco, nessun database, nessuno stato globale.
3. Validazione del dominio monetario sugli ingressi (D6).
4. Avvisi e stati come da specifica rev. 4; classificazione attuale **preservata**.
5. Test unitari e, se e quando esisterà un comando, regressioni CLI.

**Fuori**

Nessun comando CLI nuovo; nessun export, Excel o CSV; nessun calcolo DRE né lettura di `mapeamento_dre.yaml`; nessun archivio o deduplicazione persistente; nessuna modifica agli YAML canonici, al campione, al parser, alle regole per fornitore, a C13 o agli alias; nessuna validazione XSD; nessuna modifica ai progetti Python; nessuna fase 3C/3D.

**Modello dati:** nessuna modifica ai tipi esistenti. Le evoluzioni proposte
nella specifica (`Dado` in `Duplicata`, `Vec<String>` per `alertas`, `Dado<String>`
per i campi DRE, `det_letti` in `NfeImportada`, copertura in `Dado<T>`) restano
**proposte**, ciascuna da decidere a parte.

## 4. Contratti da implementare

Dalla matrice §9 della specifica: **F3A-01 … F3A-28**, con questi stati.

| Gruppo | Contratti | Stato |
|---|---|---|
| Parità con la fonte, verificata | F3A-01, -02, -03, -04, -05, -06, -08, -12, -13a, -13b, -14, -16, -19, -22, -25 | EREDITATO_VERIFICATO |
| Coperti dalle decisioni approvate | F3A-07 (D2), F3A-09 (D1), F3A-10 (nel dominio), F3A-11 (D3), F3A-15 (D4), F3A-23a/b (D5), F3A-26 (D6), F3A-27 (D4), F3A-28 (quattro esiti) | **APPROVATI** |
| Vincoli già vigenti da non violare | C1, C2, C8, C10, C14, C17 | VINCOLO_GIÀ_VIGENTE |
| Tecnici, da approvare con questo documento | F3A-17 (`data_pagamento` assente con motivo), F3A-18 (DRE `NAO_ASSEGNATO`, verifica rinviata alla 3D), F3A-20 (conti di aggiustamento con errore tipizzato), F3A-21 (arrotondamento `MidpointAwayFromZero`) | PROPOSTI |
| Ancora aperto | **F3A-24**: policy per un articolo senza descrizione. Comportamento della fonte verificato (`" (12.00)"`); la policy proposta è avviso `ITEM_SEM_DESCRICAO` + `REVISAR` | **DA_DECIDERE** |

F3A-24 è l'unica decisione di dominio non coperta dall'approvazione del
2026-10-16: va scelta prima o insieme a questo mandato.

## 5. Progetto tecnico proposto

Dettagli che la scheda di Codex lascia ai tecnici, qui dichiarati per poterli auditare.

| Elemento | Proposta | Motivazione |
|---|---|---|
| File | `src/alocacao.rs`, esposto da `src/lib.rs` | un modulo, nessuna riorganizzazione |
| Ingresso | `alocar_nfe(importada: &NfeImportada, classificador: &Classificador, plano: &PlanoContas) -> Result<AlocacaoNfe, ErroAlocacao>` | nessun parametro di mapping DRE (R3A-03) |
| Uscita | `AlocacaoNfe { lancamentos: Vec<Lancamento>, reconciliacao: Reconciliacao }` | gli avvisi del parser arrivano da `importada.avisos` e sono propagati in `reconciliacao.avisos_documento` |
| Riconciliazione | `esito_total`, `esito_rate`, `esito_copertura`, `esito_componenti`, `stato_documento` con precedenza `ERRO > PARCIAL > OK` | §5.4 della specifica |
| Ordine dei gruppi | `Vec<Gruppo>` con ricerca lineare per `conta_id` | pochi conti per documento; conserva l'ordine di comparsa senza nuove dipendenze |
| Arrotondamento | `round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)` | `round_dp` di `rust_decimal` 1.43.0 usa Bankers (`decimal.rs:1654`): il default divergerebbe dal Python |
| Dominio monetario | `valor.normalize().scale() <= 2` | `normalize` toglie gli zeri finali (`decimal.rs:1413`), quindi `10.0000` passa e `0.005` no |
| Errori | `ErroAlocacao` tipizzato: `IdentidadeInsuficiente`, `RipartoImpossibile`, `EscalaMonetariaInvalida`, `ContaDeAjusteAusente`, `ModeloIncoerente` | ogni errore nomina campo, valore o conto |
| Avvisi | codici stabili: `VALOR_INCONSISTENTE`, `TOTAL_NAO_RECONCILIADO`, `AGREGADO_PARCIAL`, `DOCUMENTO_SEM_NUMERO`, `FORNECEDOR_SEM_IDENTIDADE`, `EMPRESA_SEM_IDENTIDADE`, `DOCUMENTO_SEM_ITENS`, `HASH_DUPLICADO_NO_DOCUMENTO` | citabili nei test e nell'audit |

## 6. SHA-256: vincolo reale, tre opzioni, scelta registrata

> **Scelta di Luca il 2026-10-16: opzione (i).** Eseguita con `cargo add sha2`
> → `sha2 = "0.11.1"`. Il cambiamento è **puramente additivo**: `Cargo.toml`
> una riga in più; `Cargo.lock` solo aggiunte, nessuna riga rimossa e nessuna
> versione di dipendenza esistente modificata. Pacchetti transitivi aggiunti:
> `block-buffer`, `const-oid`, `cpufeatures`, `crypto-common`, `digest`,
> `hybrid-array`, `typenum`. Nessun aggiornamento generalizzato.

Verificato oggi: `Cargo.toml` non contiene librerie di hash, **e la cache locale
di cargo non ne ha nessuna** (95 crate in cache, nessuno fra `sha2`, `digest`,
`blake*`). Quindi aggiungere `sha2` **richiede accesso di rete** a crates.io:
non è solo una riga in `Cargo.toml`.

| Opzione | Cosa comporta | Giudizio |
|---|---|---|
| **(i) `sha2` da crates.io** | `cargo add sha2` con versione fissata, `Cargo.lock` aggiornato, build offline possibile **dopo** il primo fetch. Richiede rete e la tua autorizzazione | **preferita** se la rete è consentita: primitiva crittografica mantenuta da altri |
| (ii) rimandare l'hash | `hash_registro` resta vuoto e dichiarato in `campi_rinviati` con stato e motivo; tutto il resto della 3B procede | praticabile: nessun blocco, ma le righe non sono confrontabili con lo storico Excel |
| (iii) SHA-256 scritto qui | nessuna dipendenza nuova, nessuna rete; ~200 righe di algoritmo FIPS 180-4, validate contro i vettori ufficiali | accettabile ma **sconsigliata**: è codice crittografico da mantenere, e lo scopo qui è solo un'impronta di confronto |

In ogni caso i valori attesi sono già calcolabili e vanno nei test. Payload e
hash del campione canonico, calcolati con la SHA-256 di .NET:

```text
XML_NFE|29260512345678000195550010001256612345678901|2.01.004|10.00
  → 56a719b4f1a1d43673094c8ec65c75a0757fe59d693f48168d229485fb535a53
XML_NFE|29260512345678000195550010001256612345678901|2.01.002|20.00
  → d0438efdb6cd9640c42ae97303654635ecf9df2393b2cd489f49caacc33b1edf
XML_NFE|29260512345678000195550010001256612345678901|2.04.001|5.00
  → 8dcb5b1b8f413b766981dba322ee62c454c83e71cb18f810fc44e79e6fe111c8

vettore di controllo FIPS: sha256("abc")
  → ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
```

Questi valori sono **attesi di test**, non prove di implementazione: nessun
codice Rust li ha ancora prodotti.

## 7. Test di accettazione

Tutti **da creare**; nessuno esiste e nessuno è stato eseguito. L'elenco
completo è in §12 della specifica più i test nominati nella matrice §9: valgono
entrambi. In sintesi, per gruppo:

| Gruppo | Test |
|---|---|
| Allocazione e ordine | `aloca_campione_canonico_em_tres_lancamentos`, `ordem_de_lancamentos_rate_e_avisos_e_deterministica` |
| Riparto e arrotondamento | `riparto_das_rate_fecha_por_linha_e_por_coluna`, `residuo_de_arredondamento_fica_na_ultima_coluna`, `arredondamento_half_up_nao_bankers` |
| Differenza di totale | `diferenca_positiva_vai_para_conta_de_acessorios`, `diferenca_negativa_vai_para_descontos_obtidos`, `diferenca_nao_explicada_marca_revisar`, `componente_parcial_nao_explica_diferenca` |
| Riconciliazione (D2, D3) | `rate_que_nao_fecham_geram_valor_inconsistente`, `documento_parcial_com_componente_necessaria_parcial`, `copertura_parcial_com_det_sem_prod` |
| Denominatore zero (D1) | `denominatore_zero_banale_e_accettato`, `total_zero_com_rate_nao_vira_zero_tecnico`, `total_netto_zero_com_lancamentos_nao_nulos` |
| Dominio monetario (D6) | `importo_fora_do_dominio_monetario_e_recusado`, `zeros_finais_nao_squalificam_o_importo`, `preco_unitario_com_mais_decimais_e_ammesso`, `dado_assente_non_viene_azzerato_per_passare_il_dominio` |
| Identificativi (D5) | `chave_vazia_nao_produz_lancamento`, `numero_vazio_marca_revisar`, `fornecedor_sem_identidade_marca_revisar`, `empresa_sem_identidade_marca_revisar` |
| Hash (D4) | `hash_estavel_na_reimportacao`, `hash_do_campione_canonico_corresponde_ao_payload`, `payload_duplicado_no_documento_gera_aviso` |
| Classificazione | `produto_nao_classificado_vai_para_conta_de_revisao`, `classificacao_ambigua_marca_revisar_preservando_a_conta` |
| Campi rinviati e date | `campi_rinviati_expone_centro_custo_e_observacoes`, `dre_fica_nao_assegnato_con_motivo`, `data_pagamento_fica_ausente_com_motivo`, `competencia_vem_da_data_de_emissao` |
| C10 preservato | `nfe_sem_fat_continua_alocavel` |

**Criteri di accettazione del blocco**

1. Tutti i test sopra verdi, più i **113 esistenti senza regressioni**.
2. `cargo test --locked`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `git diff --check` tutti puliti.
3. Campione canonico e cinque YAML con SHA-256 invariati (6/6).
4. Nessun importo in `f64`; ogni arrotondamento esplicito con strategia dichiarata.
5. Nessuno zero al posto di un'assenza; ogni campo `String` legacy vuoto ha la sua voce in `campi_rinviati` con stato, fonte e motivo.
6. Funzione pura: nessuna scrittura su disco o database.
7. Ogni decisione D1-D6 citata nel test che la protegge.
8. Ogni errore e ogni avviso nuovo coperto da almeno un test che ne verifica il **codice**, non solo il percorso felice.

## 8. Sequenza proposta

| Blocco | Contenuto | Verifica |
|---|---|---|
| 3B.1 | raggruppamento, differenza di totale, dominio monetario, errori tipizzati | test dei gruppi 1, 3, 6 |
| 3B.2 | riparto delle rate, riconciliazione a quattro esiti, D1/D2/D3 | test dei gruppi 2, 4, 5 |
| 3B.3 | identificativi (D5), campi rinviati, date, C10 | test dei gruppi 7, 10, 11 |
| 3B.4 | hash (D4), secondo l'opzione scelta al §6 | test del gruppo 8 + vettori §6 |

Ogni blocco compila e ha i test verdi prima del successivo. Nessun commit
intermedio rotto; nessun commit senza autorizzazione.

## 9. Git

Regole invariate: nessun `git add .`, nessun `reset`/`restore`/`checkout`/`clean`/`stash`/`amend`/`rebase`, nessuna riscrittura della storia, nessun force-push. Staging nominativo, commit tematici, niente commit misti.

| File | Origine/mandato | Stato | Destino |
|---|---|---|---|
| `MANDATO_CLAUDE_FASE3A_CONTRATTI_20261016.md` | richiesta di Luca, mandato Codex | conservato | continuità nominata; registrazione documentale da autorizzare |
| `SPECIFICA_FASE3A_REGISTRAZIONI.md` | analisi Claude, revisione 4 | audit tecnico superato, decisioni approvate | come sopra |
| `PROPOSTA_MANDATO_FASE3B_ALOCACAO_20261016.md` (questo file) | proposta di Claude | da approvare e auditare | come sopra |

## 10. Rischi dichiarati

| Rischio | Mitigazione |
|---|---|
| La parità col Python è stata stabilita **leggendo** la fonte e replicandola, non eseguendo `pytest` | i test della 3B fissano i valori attesi; ogni divergenza futura è una regressione visibile |
| D1(a) e D6(a) rifiutano documenti che il Python accettava | divergenze **approvate** e documentate; i test le proteggono e nominano il motivo |
| L'hash resta un'identità debole | D4(a) lo dichiara: segnalazione sì, scarto no, nessuna deduplicazione |
| `esito_copertura = OK` non prova che tutti i `<det>` siano allocati | limite dichiarato in §5.4; chiuderlo richiede `det_letti` in `NfeImportada`, proposta separata |
| F3A-24 ancora aperto | da decidere prima di chiudere il blocco 3B.1 |

---

**Per approvare serve una decisione esplicita su tre punti**: questo mandato nel
suo complesso, l'opzione SHA-256 del §6, e la policy F3A-24. L'avvio
dell'implementazione, i commit e il push restano autorizzazioni separate.

---

## 11. Esito dell'esecuzione (2026-10-16)

Implementazione completata nello stesso giorno dell'autorizzazione. **In attesa
dell'audit indipendente di Codex**; nessun commit, nessun push.

### Cosa è stato scritto

| File | Natura | Contenuto |
|---|---|---|
| `src/alocacao.rs` | nuovo, ~1.100 righe con i test | `alocar_nfe`, `AlocacaoNfe`, `Reconciliacao` a quattro esiti, `ErroAlocacao` (7 varianti), `Componente`, `CampoRinviato`, modulo `avisos` con gli 11 codici, matrice di riparto, hash, 41 test |
| `src/lib.rs` | 3 righe | esporta `alocacao`, aggiorna la mappa delle fasi |
| `Cargo.toml` / `Cargo.lock` | mirate | `sha2 = "0.11.1"`, solo aggiunte |
| `README.md` | documentazione | fasi 3A/3B/3C/3D, contratti **C18-C24** con la corrispondenza agli `F3A-nn`, `sha2` fra le dipendenze, conteggio test |

**Numerazione dei contratti.** Gli `F3A-nn` della specifica erano provvisori.
Ora che il codice esiste, i contratti implementati prendono i numeri definitivi
`C18…C24` nel README, con la corrispondenza dichiarata riga per riga. Gli
`F3A-nn` restano validi nella specifica come identificativi di analisi.

### Verifiche eseguite

```text
cargo test --locked                                  154 passati, 0 falliti   (prima consegna; dopo l'audit: 163, vedi §12)
                                                     (138 lib + 3 bin + 13 CLI)
cargo fmt --check                                    exit 0
cargo clippy --locked --all-targets -- -D warnings   exit 0
git diff --check                                     exit 0
```

Baseline 113 → 154 nella prima consegna, **163 dopo le correzioni dell'audit**
(§12): 50 test nuovi in tutto, nessuna regressione sui 113 esistenti.

| Criterio §7 | Esito |
|---|---|
| 1. Test nuovi verdi + 113 esistenti senza regressioni | **sì**: 154/154 |
| 2. `test`/`fmt`/`clippy`/`diff --check` puliti | **sì** |
| 3. Sei fonti con SHA-256 invariato | **sì**: 6/6 confrontate col manifesto della salvaguardia |
| 4. Nessun importo in `f64`, arrotondamenti espliciti | **sì**: nessuna occorrenza di `f64`/`f32`; `centavos()` usa `MidpointAwayFromZero` |
| 5. Nessuno zero al posto di un'assenza; campi legacy vuoti dichiarati | **sì**: `campi_rinviati` copre `dre_linha_id`, `dre_linha_nome`, `centro_custo`, `observacoes` |
| 6. Funzione pura | **sì**: nessun `std::fs`, nessun accesso al filesystem nel modulo |
| 7. Ogni decisione D1-D6 citata in un test | **sì**: D1 → 3 test, D2 → 2, D3 → 1, D4 → 3, D5 → 4, D6 → 4 |
| 8. Ogni errore e avviso coperto da un test sul **codice** | **corretto dopo l'audit** (R3B-03, §12): i 7 errori — ora 8 con `CapacidadeNumericaExcedida` — e gli 11 avvisi hanno una regressione, **con l'unica eccezione dichiarata e dimostrata di `ModeloIncoerente`**, non raggiungibile con gli ingressi di questo modulo. L'attestazione "8 su 8" della prima consegna era sbagliata |

### Prove numeriche

| Caso | Atteso | Osservato |
|---|---|---|
| Campione canonico | 3 registrazioni 10/20/5, rate 5/10/2.50 per scadenza, documento `OK` | come atteso |
| Hash del campione | `56a719b4…`, `d0438efd…`, `8dcb5b1b…` | identici, calcolati da `sha2` |
| Vettore FIPS `sha256("abc")` | `ba7816bf…` | identico |
| Residuo di arrotondamento | `3.34\|3.34\|3.32` e `3.33\|3.33\|3.34` | come la probe di fase 3A |
| `2.345` | `2.35` con `ROUND_HALF_UP` | `2.35`; il test prova anche che `round_dp` darebbe `2.34` |
| Totale netto zero (§8M) | `RipartoImpossivel` | errore con il motivo che nomina le somme |
| `0.005` come `vProd` | `EscalaMonetariaInvalida` | errore che nomina campo e valore |
| `10.0000` | accettato come `10.00` | accettato |

### Scostamenti dal mandato

Nessuno nel comportamento. Due note:

1. **README aggiornato** (fasi, contratti C18-C24, dipendenza, conteggio test):
   non era elencato fra i file del §5, ma lasciarlo dichiarare "fase 3
   NAO_INICIADA" e "113 testes" sarebbe stato falso. È documentazione, nessun
   contratto nuovo.
2. **8 test in più** dei 33 pianificati: formato della descrizione, rata unica,
   assenza di differenza, assenza di rate, documento senza articoli, articolo
   senza descrizione (F3A-24), vettore FIPS, conti di aggiustamento nel piano.

### Rischi residui

| Rischio | Stato |
|---|---|
| Parità col Python stabilita leggendo la fonte, non eseguendo `pytest` | invariato e dichiarato; i test fissano ora i valori attesi |
| `esito_copertura = OK` non prova che tutti i `<det>` siano allocati | invariato: richiede `det_letti` in `NfeImportada`, proposta separata |
| `ClassificacaoFalhou` e `ModeloIncoerente` senza test dedicato | dichiarato sopra |
| Nessun comando CLI usa ancora `alocar_nfe` | voluto: il perimetro esclude comandi nuovi. La fase 3B è libreria |

---

## 12. Correzione dei riscontri dell'audit 3B (2026-10-16)

Audit indipendente: **DA_CORREGGERE**, tre riscontri. Tutti e tre riprodotti da
me prima di toccare il codice, poi corretti. Le decisioni D1-D6 non sono state
riaperte.

### R3B-01 — capacità numerica

Riprodotto: `Decimal` **va in panico** sul prodotto del riparto
(`checked_mul` → `None`) e **perde un centesimo in silenzio** nella somma —
`checked_add` compreso, che restituisce `Some(79228162514264337593543950335)`
sommando `0.01` al massimo rappresentabile. Il `checked_*` da solo non
risolveva il caso B, come l'audit aveva avvertito.

Correzione: **tutta l'aritmetica monetaria del modulo passa in centesimi
interi `i128`**, non più in `Decimal`.

| Punto | Prima | Ora |
|---|---|---|
| conversione | — | `em_centesimos`: da mantissa e scala, senza moltiplicare in `Decimal` |
| somme | `Decimal` `+` | `somar_centesimos` con `checked_add` |
| prodotto del riparto | `rate * lanc` → panico | `checked_mul` in `i128`: il caso A ora **riesce** e dà 250000000000000.00 per cella, come la funzione Python |
| divisione e arrotondamento | `round_dp_with_strategy` | resto della divisione intera, `ROUND_HALF_UP` (metà lontano da zero) |
| ritorno a `Decimal` | — | `de_centesimos` con `try_from_i128_with_scale`; se il risultato non è rappresentabile → errore |
| nuovo errore | — | `CapacidadeNumericaExcedida { origem, operacao, detalhe }` |

Nessun tetto di importo inventato, nessun `f64`, nessun arrotondamento
correttivo, nessuna dipendenza nuova. Il limite è **della rappresentazione** ed
è dichiarato come limite tecnico, non come regola di dominio approvata.

Regressioni: `riparto_com_importi_estremi_nao_entra_em_panico` (caso A),
`soma_que_nao_cabe_e_erro_em_vez_de_perder_centavo` (caso B),
`decimal_perde_centavo_na_soma_extrema` e `decimal_estoura_no_produto_do_rateio`
(fissano il motivo della scelta: se qualcuno tornasse a `Decimal`, questi due
test cadono).

### R3B-02 — formato della descrizione

Riprodotto: con `vProd` di scala 0 o 1 la descrizione usciva `(10)` e `(10.0)`,
mentre il Python dà sempre `(10.00)`. Causa: `Display` di `Decimal` senza
precisione, dopo un arrotondamento che non impone la scala.

Correzione: `format!("{} ({:.2})", ...)` in `descrever_item`. Il valore
monetario non cambia. Regressioni: `descricao_usa_sempre_duas_casas_decimais`
sulle quattro rappresentazioni (`10`, `10.0`, `10.00`, `10.0000`) e
`descricao_vazia_mantem_duas_casas_e_o_aviso`.

### R3B-03 — copertura degli errori e attestazione del criterio 8

L'attestazione "8 su 8" della revisione precedente **era sbagliata**: tre errori
erano raggiungibili dall'API pubblica senza regressione. Aggiunte:

| Errore | Regressione | Verifica |
|---|---|---|
| `ClassificacaoFalhou` | `classificacao_com_conta_inexistente_propaga_erro` | variante + `conta_id` + fonte non vuota |
| `CompetenciaInvalida` | `competencia_fora_do_intervalo_propaga_erro` | variante + anno 1800 nel messaggio |
| `ContaDeAjusteAusente` | `conta_de_ajuste_ausente_propaga_erro` | **entrambi i segni**, parametrizzati: `2.05.001` e `2.05.002` |

La prova dei conti di aggiustamento usa una fixture dichiarata,
`tests/fixtures/plano_sem_contas_de_ajuste.yaml`: piano valido nella forma e
volutamente privo dei due conti. **Il piano canonico non è stato toccato**
(hash invariato).

**Eccezione dichiarata e dimostrata: `ModeloIncoerente`.** Non è raggiungibile
con gli ingressi di questo modulo. Dimostrazione: ogni costruzione di `Dado`
qui usa una `fonte` letterale non vuota, un `motivo` letterale non vuoto e uno
`status` scelto in modo coerente con la presenza del valore — le tre condizioni
che `Dado::novo` verifica. Il ramo resta come rete di sicurezza per un bug
futuro di questo modulo, con il messaggio che lo dichiara tale; non viene
forzato un bug artificiale per coprirlo. Quindi: **7 criteri su 8 pieni, il
criterio 8 soddisfatto con questa unica eccezione nominata**, non "8 su 8".

### Verifiche dopo la correzione

```text
cargo test --locked                                  163 passati, 0 falliti
                                                     (147 lib + 3 bin + 13 CLI)
cargo fmt --check                                    exit 0
cargo clippy --locked --all-targets -- -D warnings   exit 0
git diff --check                                     exit 0
sei fonti contro il manifesto della salvaguardia     6/6 invariate
```

Da 154 a 163 in questa correzione, e a **166** dopo l'audit mirato (§13):
regressioni nuove, nessuna regressione sui test
preesistenti. `soma` e `construir_matriz` sono diventate `#[cfg(test)]`,
perché l'aritmetica di produzione non le usa più: `clippy` lo ha segnalato come
codice morto e la marcatura è la correzione onesta, non un `allow`.

### File toccati da questa correzione

| File | Cosa |
|---|---|
| `src/alocacao.rs` | aritmetica in centesimi, nuovo errore, `{:.2}` nella descrizione, 9 regressioni, 2 helper resi test-only |
| `tests/fixtures/plano_sem_contas_de_ajuste.yaml` | nuovo, fixture dichiarata per `ContaDeAjusteAusente` |
| `README.md` | conteggio test, riga della fixture, paragrafo sul limite tecnico di capacità |
| `SPECIFICA_FASE3A_REGISTRAZIONI.md` | limite di capacità numerica fra i limiti dichiarati |
| questo documento | §12 e correzione dell'attestazione del criterio 8 |

---

## 13. Audit mirato, revisione 2 (2026-10-16): chiusura di R3B-01

Esito dell'audit mirato: **R3B-02 e R3B-03 chiusi**; **R3B-01 ancora aperto**
con due residui nei miei helper numerici. Entrambi riprodotti prima della
correzione; toccati **solo** `de_centesimos` e `dividir_meio_para_cima`,
nessuna nuova architettura, dipendenza o decisione.

### Residuo A — la divisione poteva ancora andare in panico

Riprodotto: con `vNF = -0.01`, gruppi `-92233720368547758.08` e
`92233720368547758.07`, rate `184467440737095516.16` e
`-184467440737095516.17`, la prima cella del riparto calcola
`2^64 x (-2^63) = -2^127 = i128::MIN`, che `checked_mul` accetta perché
**è** rappresentabile; poi `i128::MIN / -1` non lo è e l'operatore ordinario
andava in panico (`attempt to divide with overflow`). Avevo controllato somme e
prodotto, non divisione e resto.

Correzione: `checked_div` e `checked_rem`, con lo stesso errore tipizzato.
Nessun filtro di segno introdotto per evitare il caso: gli importi restano
legittimi per D6.

Regressione: `divisao_do_rateio_nao_entra_em_panico_com_minimo`, che verifica
la **variante** `CapacidadeNumericaExcedida` con `operacao = "divisao do
rateio"` e il messaggio, non un panico intercettato.

### Residuo B — falso rifiuto di importi rappresentabili

Riprodotto: `de_centesimos` imponeva la scala 2, quindi un importo intero
grande veniva rifiutato benché `Decimal` lo rappresenti a scala 0. Con
`vNF = 79228162514264337593543950335` l'allocazione falliva con
`CapacidadeNumericaExcedida` su `valor do grupo`: **un falso errore**, perché
il valore d'ingresso prova da sé di essere rappresentabile.

Correzione: la conversione prova la scala 2 e, se la mantissa non ci sta,
**riduce gli zeri finali** abbassando la scala fino a 0. Nessuna cifra diversa
da zero viene eliminata, nessun arrotondamento. Se il centesimo non è
divisibile, il valore non è rappresentabile e l'errore resta.

### Il vecchio "caso B" non provava la causa dichiarata

Rilievo accolto: quel test passava per il falso rifiuto del singolo `MAX`,
prima della somma critica. Non ho forzato un errore artificiale per
conservarlo. Ora:

| Test | Che cosa prova |
|---|---|
| `dois_itens_extremos_em_contas_distintas_preservam_o_centavo` | gli stessi ingressi di prima ora **riescono**: due gruppi (`MAX` e `0.01`) più la differenza di `-0.01` su `2.05.002`, `REVISAR` con `VALOR_INCONSISTENTE`, somma delle registrazioni uguale al totale, `esito_total = OK`. Il centesimo è registrato, non perso |
| `valor_agregado_nao_representavel_e_erro` | errore di rappresentazione **vero**: due articoli sullo stesso conto, valore aggregato `MAX + 0.01`, errore con `operacao = "valor do grupo"` e messaggio "nem reduzindo zeros finais" |
| `round_trip_de_centesimos_respeita_o_dominio_de_decimal` | round-trip esatto di `MAX`, del suo **simmetrico negativo**, di un valore a scala 2 vicino al limite, di `10.00`, `-0.01` e `0.00`; e rifiuto di `MAX + 0.01` in centesimi |
| `riparto_com_importi_estremi_nao_entra_em_panico` | **caso A confermato**: quattro celle da `250000000000000.00` |

Nota su un mio errore in corso d'opera: la prima stesura di
`valor_agregado_nao_representavel_e_erro` usava `MAX + 0.01` come **totale**,
che `Decimal::from_str_exact` rifiuta con `Underflow`. Il test falliva nel
proprio helper, non nel modulo. Corretto: il totale è `MAX` e l'eccedenza sta
nel valore del gruppo.

### Verifiche dopo questa correzione

```text
cargo test --locked                                  166 passati, 0 falliti
                                                     (150 lib + 3 bin + 13 CLI)
cargo fmt --check                                    exit 0
cargo clippy --locked --all-targets -- -D warnings   exit 0
git diff --check                                     exit 0
sei fonti contro il manifesto della salvaguardia     6/6 invariate
```

Da 163 a 166: **3 regressioni nuove** (divisione con `i128::MIN`, valore
aggregato non rappresentabile, round-trip) e una riscritta (il vecchio caso B,
che ora prova la conservazione del centesimo). Nessuna regressione sui test
preesistenti.

### Stato dei riscontri

| Riscontro | Stato |
|---|---|
| R3B-01 | **chiuso**: verbale finale dell'audit del 2026-10-16, dopo la correzione dei due residui |
| R3B-02 | chiuso dall'audit |
| R3B-03 | chiuso dall'audit, con l'eccezione motivata di `ModeloIncoerente` |

Commit e push restano non autorizzati. Nessuna fase 3C/3D avviata.

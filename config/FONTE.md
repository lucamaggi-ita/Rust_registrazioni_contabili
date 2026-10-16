# Fonte dei file di configurazione

Questi YAML **non sono stati scritti qui**: sono copiati dal progetto Python originale.

| Campo | Valore |
|---|---|
| Repository di origine | https://github.com/lucamaggi-ita/Programma_registrazioni-contabili |
| Percorso di origine | `config/` |
| Commit di origine | `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` |
| Data commit origine | 2026-05-06 |
| Data copia | 2026-10-15 |
| Stato | `COPIA_VERSIONATA` |

## Regola anti-divergenza

Esistono ora **due copie** delle stesse regole (Python e Rust). Finche' entrambi i
progetti sono vivi, questa e' una divergenza potenziale, non un fatto risolto.

Contratto operativo:

1. La fonte canonica resta il repository Python fino a migrazione completata.
2. Ogni ri-allineamento aggiorna commit e data in questa tabella.
3. Nessuna modifica di regola va fatta solo qui: prima nel repo canonico, poi copia.
4. Alla fine della migrazione questo file va aggiornato dichiarando Rust come fonte canonica.

## File coperti

- `plano_contas_gerencial.yaml` (125 conte, 11 centri di costo)
- `regras_classificacao_produtos.yaml` (29 regole + 1 regola per fornitore)
- `regras_classificacao_despesas.yaml` (26 regole)
- `mapeamento_dre.yaml` (non letto dal codice Rust: previsto per la fase 3, DRE)
- `csv_columns.yaml` (non letto dal codice Rust: la fase 2 non importa CSV, e la
  fase che lo leggera' non e' ancora definita)

Stato di lettura al 2026-10-16: i primi tre file sono letti e validati; gli
ultimi due sono `NON_LETTO`, copia versionata in attesa della fase che li usa.
Il README li elenca fra i rischi noti per lo stesso motivo.

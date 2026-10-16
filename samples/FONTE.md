# Fonte dei campioni

| Campo | Valore |
|---|---|
| Repository di origine | https://github.com/lucamaggi-ita/Programma_registrazioni-contabili |
| Percorso di origine | `samples/` |
| Commit di origine | `5b70f2b7e8cc39b6aa96f3e8903d2ac5d870c901` |
| Data copia | 2026-10-15 |
| Stato | `COPIA_VERSIONATA` |

## `nfe_exemplo/nfe_oesa_12566_20260505.xml`

NF-e di esempio **gia anonimizzata nella fonte** (CNPJ emittente `12345678000195`,
indirizzi `RUA EXEMPLO`): non contiene dati fiscali reali dell'emittente.

E' il campione su cui gira il test di parsing `nfe::tests`, gli stessi valori
attesi dei test Python `tests/test_nfe_importer.py`. Non modificarlo: cambiarlo
significa cambiare l'oracolo del parser.

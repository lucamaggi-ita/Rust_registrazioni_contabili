#!/usr/bin/env python3
"""Controprova della rilettura indipendente: che cosa deve **fallire**.

Perche' esiste: `releitura_export.py` dichiara alcune divergenze note del
lettore, e un'eccezione concessa troppo facilmente trasforma un file errato in
un limite dichiarato. Riscontro `R3C41-04`, residuo del controllo mirato del
2026-10-30: l'eccezione veniva concessa sul **solo** risultato di `openpyxl`,
senza guardare che cosa il pacchetto rappresenti.

Che cosa fa: prepara copie alterate del workbook depositato dai test, lascia
il manifesto **invariato** ed esegue lo script del progetto senza modificarlo,
confrontando il codice di uscita osservato con quello atteso.

| Famiglia di copie | Atteso |
|---|---|
| baseline, non alterata | 0, con le divergenze note dichiarate |
| baseline senza `--manifesto` | 2, verifica non eseguita |
| sette mutazioni ordinarie di celle | 1, con la cella nominata |
| due alterazioni **mascherate** da una divergenza nota | 1, con la cella nominata |

L'ultima famiglia e' quella del residuo: il testo nel pacchetto viene
riscritto in modo che `openpyxl` restituisca **esattamente** il risultato
della sua anomalia registrata, mentre il testo che il file rappresenta e'
diverso da quello del manifesto. Non afferma che il writer produca quel dato:
dimostra che la verifica lo distingue.

Niente installazioni: `zipfile`, `xml.etree` e `openpyxl` sono gia' presenti.
Lavora in `%TEMP%\\fantozzi_3c41_controprova\\` e non scrive nel repository.

Uso:
    py -I script/controprova_rilettura.py
    py -I script/controprova_rilettura.py --workbook <x.xlsx> --manifesto <m.json>
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
import zipfile

CODICE_DISCORDANZA = 1
CODICE_AMBIENTE = 2

for _flusso in (sys.stdout, sys.stderr):
    try:
        _flusso.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, OSError):
        pass

NS_FOGLIO = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
NS_RELAZIONI = "http://schemas.openxmlformats.org/package/2006/relationships"
NS_R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"

# Mutazioni ordinarie: foglio, riga del workbook, colonna, valore nuovo.
MUTAZIONI = [
    ("totale", "resumos", 2, "valor_total", "99.00"),
    ("totale_numerico", "resumos", 2, "valor_total_numero", 99.0),
    ("chiave", "extrato_revisado", 2, "chave_origem", "2926051234567800019555001000125661234567890X"),
    ("stato", "extrato_revisado", 2, "estado_revisao", "OK"),
    ("autore", "decisoes", 2, "autor", "Eve"),
    ("motivo", "decisoes", 2, "motivo", "visto e aceito"),
    ("indicatore", "indicadores", 11, "valor", "99.00"),
    # `R3C41-05`: la firma esatta del difetto corretto. Un `APROVADO`
    # nello stato tecnico maschererebbe l'esito della 3B.
    ("stato_tecnico", "diagnostico", 2, "status_tecnico", "APROVADO"),
]

# Alterazioni **mascherate**: il pacchetto rappresenta un testo diverso da
# quello del manifesto, ma `openpyxl` restituisce il risultato della divergenza
# registrata per quella cella. Una per ogni eccezione registrata nel modo
# `export`.
#
# `_x005F_` protegge l'underscore: `a_x005F_x000D_b` rappresenta i quindici
# caratteri letterali `a_x000D_b`, non il ritorno a capo del manifesto.
def toglie_un_livello_di_protezione(grezzo: str) -> str:
    """`_x005F_x005F_x005F_` → `_x005F_`: un livello di protezione in meno.

    Serve per la seconda eccezione registrata, quella della frase del limite
    `ESCAPE_LEITOR`. Nel pacchetto corretto il testo logico `_x005F_x0041_` e'
    protetto come `_x005F_x005F_x005F_x0041_`; togliendo un livello il file
    rappresenta `_x0041_`, cioe' **il risultato dell'anomalia del lettore** e
    non il testo del manifesto, mentre `openpyxl` continua a mostrare
    esattamente quel risultato. Se la verifica guardasse solo il lettore, non
    vedrebbe la differenza.
    """
    se_trova = "_x005F_x005F_x005F_"
    if se_trova not in grezzo:
        raise ValueError(
            f"la cella non contiene {se_trova!r}: l'alterazione non e' "
            "applicabile e non viene simulata"
        )
    return grezzo.replace(se_trova, "_x005F_", 1)


MASCHERATE = [
    (
        "mascherata_cr",
        "decisoes",
        2,
        "motivo",
        "visto e aceito a_x005F_x000D_b",
        "decisoes!2:motivo",
    ),
    (
        "mascherata_escape",
        "limites",
        5,
        "descricao",
        toglie_un_livello_di_protezione,
        "limites!5:descricao",
    ),
]

# ---------------------------------------------------------------------------
# Modo `percurso-cli` (fase 3C.5.a)
# ---------------------------------------------------------------------------
#
# Il workbook della CLI non ha ne' decisioni ne' estratto revisionato — e'
# proprio il punto di `CLI-02` — quindi le mutazioni della fixture non sono
# applicabili: le loro celle non esistono. Queste sono le sue.
MUTAZIONI_PERCURSO_CLI = [
    # Un importo.
    ("valor", "diagnostico", 2, "valor", "99.00"),
    ("totale", "resumos", 2, "valor_total", "99.00"),
    # Uno stato, e qui l'alterazione piu' grave di tutte per questo percorso:
    # un'importazione che sembra avere approvato qualcosa.
    ("estado_revisao", "diagnostico", 2, "estado_revisao", "APROVADO_POR_PESSOA"),
    # `R3C41-05`: la fonte dello stato tecnico, di nuovo.
    ("stato_tecnico", "diagnostico", 2, "status_tecnico", "APROVADO"),
]

# Una sola eccezione e' registrata anche per questo modo: la frase del foglio
# `limites`, che ogni export porta identica.
MASCHERATE_PERCURSO_CLI = [
    (
        "mascherata_escape",
        "limites",
        5,
        "descricao",
        toglie_un_livello_di_protezione,
        "limites!5:descricao",
    ),
]


# ---------------------------------------------------------------------------
# Modo `percurso-cli-revisado` (fase 3C.5.b)
# ---------------------------------------------------------------------------
#
# Qui le decisioni **esistono**, quindi le alterazioni che contano sono quelle
# che fingono una decisione mai dichiarata o la attribuiscono a un altro.
MUTAZIONI_PERCURSO_CLI_REVISADO = [
    # Un importo.
    ("valor", "diagnostico", 4, "valor", "99.00"),
    # Un'approvazione **mai dichiarata**, su una riga che nessuno ha approvato.
    ("aprovacao_inventada", "diagnostico", 2, "estado_revisao", "APROVADO_POR_PESSOA"),
    # L'approvazione attribuita a un altro: l'autore e' una dichiarazione, e
    # cambiarlo nel file e' falsificarla.
    ("autor_trocado", "extrato_revisado", 2, "aprovado_por", "Outra Pessoa"),
    ("autor_da_decisao", "decisoes", 2, "autor", "Eve"),
    # La decisione cancellata dall'estratto revisionato.
    ("estado_apagado", "extrato_revisado", 2, "estado_revisao", "TECNICO_OK"),
]

# Come negli altri modi del workbook, la frase del foglio `limites`.
MASCHERATE_PERCURSO_CLI_REVISADO = [
    (
        "mascherata_escape",
        "limites",
        5,
        "descricao",
        toglie_un_livello_di_protezione,
        "limites!5:descricao",
    ),
]


def tabelle_do_modo(modo: str):
    """Mutazioni e mascherate **di questo modo**: le celle di un percorso non
    esistono necessariamente nell'altro."""
    if modo == "percurso-cli":
        return MUTAZIONI_PERCURSO_CLI, MASCHERATE_PERCURSO_CLI
    if modo == "percurso-cli-revisado":
        return (
            MUTAZIONI_PERCURSO_CLI_REVISADO,
            MASCHERATE_PERCURSO_CLI_REVISADO,
        )
    return MUTAZIONI, MASCHERATE


def nome_breve(percorso: pathlib.Path) -> str:
    return percorso.name


def mappa_dei_fogli(z: zipfile.ZipFile) -> dict[str, str]:
    """Nome del foglio → percorso del suo XML, letto dalle relazioni."""
    libro = ET.fromstring(z.read("xl/workbook.xml"))
    relazioni = ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))
    per_id = {
        r.get("Id"): r.get("Target")
        for r in relazioni.findall(f"{{{NS_RELAZIONI}}}Relationship")
    }
    fogli = {}
    for foglio in libro.iter(f"{{{NS_FOGLIO}}}sheet"):
        destinazione = per_id[foglio.get(f"{{{NS_R}}}id")]
        if not destinazione.startswith("/"):
            destinazione = "xl/" + destinazione.lstrip("./")
        fogli[foglio.get("name")] = destinazione.lstrip("/")
    return fogli


def indice_di_colonna(riferimento: str) -> int:
    """`P2` → 16. Le lettere sono in base 26."""
    lettere = re.match(r"([A-Z]+)", riferimento).group(1)
    indice = 0
    for lettera in lettere:
        indice = indice * 26 + (ord(lettera) - ord("A") + 1)
    return indice


def lettere_di_colonna(indice: int) -> str:
    lettere = ""
    while indice:
        indice, resto = divmod(indice - 1, 26)
        lettere = chr(ord("A") + resto) + lettere
    return lettere


def colonna_per_nome(z: zipfile.ZipFile, percorso_foglio: str, nome: str, condivise: list[str]) -> int:
    """Indice della colonna che ha `nome` nell'intestazione del foglio."""
    albero = ET.fromstring(z.read(percorso_foglio))
    for riga in albero.iter(f"{{{NS_FOGLIO}}}row"):
        if riga.get("r") != "1":
            continue
        for cella in riga.iter(f"{{{NS_FOGLIO}}}c"):
            testo = testo_della_cella(cella, condivise)
            if testo == nome:
                return indice_di_colonna(cella.get("r"))
    raise ValueError(f"colonna {nome!r} assente in {percorso_foglio}")


def testo_della_cella(cella: ET.Element, condivise: list[str]) -> str | None:
    tipo = cella.get("t")
    if tipo == "s":
        valore = cella.find(f"{{{NS_FOGLIO}}}v")
        if valore is None or valore.text is None:
            return None
        return condivise[int(valore.text)]
    if tipo == "inlineStr":
        testo = cella.find(f"{{{NS_FOGLIO}}}is/{{{NS_FOGLIO}}}t")
        return "" if testo is None else (testo.text or "")
    return None


def stringhe_condivise(z: zipfile.ZipFile) -> list[str]:
    if "xl/sharedStrings.xml" not in z.namelist():
        return []
    albero = ET.fromstring(z.read("xl/sharedStrings.xml"))
    valori = []
    for si in albero.iter(f"{{{NS_FOGLIO}}}si"):
        testo = si.find(f"{{{NS_FOGLIO}}}t")
        valori.append("" if testo is None or testo.text is None else testo.text)
    return valori


def cella_grezza(
    origine: pathlib.Path,
    percorso_foglio: str,
    riga: int,
    indice_colonna: int,
    condivise: list[str],
) -> str:
    """Il testo **come sta nel pacchetto**, prima di `ST_Xstring`."""
    with zipfile.ZipFile(origine) as z:
        albero = ET.fromstring(z.read(percorso_foglio))
    for riga_xml in albero.iter(f"{{{NS_FOGLIO}}}row"):
        if int(riga_xml.get("r")) != riga:
            continue
        for cella in riga_xml.iter(f"{{{NS_FOGLIO}}}c"):
            if indice_di_colonna(cella.get("r")) != indice_colonna:
                continue
            testo = testo_della_cella(cella, condivise)
            if testo is None:
                raise ValueError(f"la cella {cella.get('r')} non e' testuale")
            return testo
    raise ValueError(f"cella non trovata: riga {riga}, colonna {indice_colonna}")


def altera_in_modo_mascherato(
    origine: pathlib.Path,
    destino: pathlib.Path,
    folha: str,
    riga: int,
    coluna: str,
    codifica_nuova,
) -> str:
    """Riscrive **una** cella puntandola a una stringa condivisa nuova.

    Il writer del progetto non viene toccato: questa e' chirurgia sul
    pacchetto, fatta qui per provare la verifica.
    """
    with zipfile.ZipFile(origine) as z:
        voci = {nome: z.read(nome) for nome in z.namelist()}
        ordine = z.namelist()
        condivise = stringhe_condivise(z)
        fogli = mappa_dei_fogli(z)
        percorso_foglio = fogli[folha]
        indice_colonna = colonna_per_nome(z, percorso_foglio, coluna, condivise)

    # `codifica_nuova` e' un testo, oppure una trasformazione del testo che
    # la cella ha adesso nel pacchetto.
    if callable(codifica_nuova):
        attuale = cella_grezza(
            origine, percorso_foglio, riga, indice_colonna, condivise
        )
        codifica_nuova = codifica_nuova(attuale)

    # 1. una voce nuova in coda alle stringhe condivise
    testo_sst = voci["xl/sharedStrings.xml"].decode("utf-8")
    indice_nuovo = len(condivise)
    voce = f"<si><t>{codifica_nuova}</t></si>"
    assert testo_sst.rstrip().endswith("</sst>")
    testo_sst = testo_sst.rstrip()[: -len("</sst>")] + voce + "</sst>"
    # `count` e `uniqueCount` devono restare coerenti, o il pacchetto e' rotto
    testo_sst = re.sub(
        r'count="(\d+)"',
        lambda m: f'count="{int(m.group(1)) + 1}"',
        testo_sst,
        count=2,
    )
    voci["xl/sharedStrings.xml"] = testo_sst.encode("utf-8")

    # 2. quella sola cella indirizzata alla voce nuova
    riferimento = f"{lettere_di_colonna(indice_colonna)}{riga}"
    testo_foglio = voci[percorso_foglio].decode("utf-8")
    schema = re.compile(
        rf'<c r="{riferimento}"([^>]*)><v>\d+</v></c>'
    )
    sostituito, quante = schema.subn(
        rf'<c r="{riferimento}"\1><v>{indice_nuovo}</v></c>', testo_foglio
    )
    if quante != 1:
        raise ValueError(
            f"cella {folha}!{riferimento} trovata {quante} volte: "
            "l'alterazione non e' stata applicata"
        )
    voci[percorso_foglio] = sostituito.encode("utf-8")

    # 3. ZIP riscritto, con i CRC ricalcolati da `zipfile`
    with zipfile.ZipFile(destino, "w", zipfile.ZIP_DEFLATED) as z:
        for nome in ordine:
            z.writestr(nome, voci[nome])
    return codifica_nuova


def esegui(
    script: pathlib.Path,
    workbook: pathlib.Path,
    manifesto: pathlib.Path | None,
    modo: str = "export",
):
    comando = [sys.executable, "-I", str(script), str(workbook), "--modo", modo]
    if manifesto is not None:
        comando += ["--manifesto", str(manifesto)]
    return subprocess.run(
        comando,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="backslashreplace",
    )


def main() -> int:
    analizzatore = argparse.ArgumentParser(description=__doc__)
    temporanea = pathlib.Path(tempfile.gettempdir()) / "fantozzi_3c41"
    analizzatore.add_argument(
        "--script",
        default=str(pathlib.Path(__file__).with_name("releitura_export.py")),
    )
    analizzatore.add_argument(
        "--workbook", default=str(temporanea / "export_percurso.xlsx")
    )
    analizzatore.add_argument(
        "--manifesto", default=str(temporanea / "export_percurso.manifesto.json")
    )
    analizzatore.add_argument(
        "--modo",
        choices=["export", "percurso-cli", "percurso-cli-revisado"],
        default="export",
        help=(
            "quale percorso si controprova. `export` e' la fixture 3C.4.1 con "
            "le sue due approvazioni; `percurso-cli` e' il workbook che la CLI "
            "produce da una cartella di NF-e, senza nessuna approvazione; "
            "`percurso-cli-revisado` e' lo stesso workbook dopo una "
            "dichiarazione di revisione. Gli ultimi due vogliono --workbook e "
            "--manifesto del loro deposito"
        ),
    )
    argomenti = analizzatore.parse_args()

    script = pathlib.Path(argomenti.script)
    workbook = pathlib.Path(argomenti.workbook)
    manifesto = pathlib.Path(argomenti.manifesto)
    for percorso in (script, workbook, manifesto):
        if not percorso.exists():
            print(
                f"AMBIENTE: {percorso} non esiste. Il workbook e il manifesto "
                "li deposita `cargo test --test export_releitura`.",
                file=sys.stderr,
            )
            return CODICE_AMBIENTE

    try:
        import openpyxl
    except ImportError as errore:
        print(f"AMBIENTE: openpyxl non disponibile: {errore}", file=sys.stderr)
        return CODICE_AMBIENTE

    arena = (
        pathlib.Path(tempfile.gettempdir())
        / f"fantozzi_controprova_{argomenti.modo}"
    )
    mutazioni, mascherate = tabelle_do_modo(argomenti.modo)
    shutil.rmtree(arena, ignore_errors=True)
    arena.mkdir(parents=True)
    print(f"arena  {arena}")
    print(f"script {script}")
    print(f"modo   {argomenti.modo}")

    casi: list[tuple[str, int, str | None, subprocess.CompletedProcess]] = []

    # 1. la baseline, non alterata
    casi.append(
        ("baseline", 0, None, esegui(script, workbook, manifesto, argomenti.modo))
    )

    # 2. la baseline senza manifesto
    casi.append(
        ("senza_manifesto", 2, None, esegui(script, workbook, None, argomenti.modo))
    )

    # 3. le mutazioni ordinarie
    for etichetta, folha, riga, coluna, nuovo in mutazioni:
        copia = arena / f"mutato_{etichetta}.xlsx"
        shutil.copy2(workbook, copia)
        livro = openpyxl.load_workbook(copia)
        folha_obj = livro[folha]
        intestazione = [c.value for c in next(folha_obj.iter_rows(max_row=1))]
        folha_obj.cell(row=riga, column=intestazione.index(coluna) + 1, value=nuovo)
        livro.save(copia)
        livro.close()
        casi.append(
            (
                f"mutazione_{etichetta}",
                1,
                f"{folha}!{riga}:{coluna}",
                esegui(script, copia, manifesto, argomenti.modo),
            )
        )

    # 4. le alterazioni mascherate da una divergenza nota
    for etichetta, folha, riga, coluna, codifica, chiave in mascherate:
        copia = arena / f"{etichetta}.xlsx"
        applicata = altera_in_modo_mascherato(
            workbook, copia, folha, riga, coluna, codifica
        )
        letto = openpyxl.load_workbook(copia)
        folha_obj = letto[folha]
        intestazione = [c.value for c in next(folha_obj.iter_rows(max_row=1))]
        mostrato = folha_obj.cell(
            row=riga, column=intestazione.index(coluna) + 1
        ).value
        letto.close()
        print(f"  {etichetta}")
        print(f"      nel pacchetto   {applicata!r}")
        print(f"      openpyxl mostra {mostrato!r}")
        casi.append(
            (etichetta, 1, chiave, esegui(script, copia, manifesto, argomenti.modo))
        )

    print()
    print(f"{'caso':<28} {'atteso':>6} {'osservato':>9}  esito")
    falliti = []
    for etichetta, atteso, chiave, esito in casi:
        righe = [r for r in esito.stdout.splitlines() if "DISCORDANZA" in r]
        nominata = chiave is None or any(chiave in r for r in righe)
        bene = esito.returncode == atteso and nominata
        print(
            f"{etichetta:<28} {atteso:>6} {esito.returncode:>9}  "
            f"{'ok' if bene else 'NON CONFORME'}"
            + ("" if chiave is None else f"  cella nominata: {nominata}")
        )
        if not bene:
            falliti.append(etichetta)
            for riga in righe[:4]:
                print(f"      {riga[:220]}")
            if not righe:
                for riga in esito.stdout.splitlines()[-6:]:
                    print(f"      {riga[:220]}")

    print()
    if falliti:
        print(f"CONTROPROVA NON SUPERATA: {falliti}")
        return CODICE_DISCORDANZA
    print(
        "CONTROPROVA SUPERATA: la baseline conserva il suo esito con i limiti "
        "dichiarati, e ogni alterazione — anche quella mascherata da una "
        "divergenza nota — viene respinta nominando la cella."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

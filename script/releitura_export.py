#!/usr/bin/env python3
"""Rilettura indipendente di un workbook prodotto dall'export (EXP-29).

Perche' esiste: la prova di fedelta' F3C-21 richiede uno strumento **diverso**
da quello che ha scritto il file. Da Rust non si vede il **tipo effettivo**
della cella, e un confronto del writer con se stesso non proverebbe niente.

Che cosa non e': una dipendenza del progetto. Usa l'interprete e `openpyxl`
gia' presenti sulla macchina, non installa nulla e non entra in `Cargo.toml`.
`cargo test` non lo invoca: va eseguito a parte, con il comando documentato
nel README.

Esito controllato: esce con 0 se ogni confronto torna, con 1 alla prima
discordanza, con 2 se l'ambiente non permette la verifica. Un confronto non
eseguito non risulta superato.

Uso:
    py -I script/releitura_export.py <percorso.xlsx> --modo tx38
    py -I script/releitura_export.py <percorso.xlsx> --modo export \
        --manifesto <percorso.manifesto.json>
    py -I script/releitura_export.py <percorso.xlsx> --modo percurso-cli \n        --manifesto <percorso.manifesto.json>

I modi `export` e `percurso-cli` fanno gli stessi confronti sul file e
hanno attese **diverse**: il primo e' la fixture del percorso 3C.4.1, con
le sue due approvazioni; il secondo e' il workbook che la CLI della fase
3C.5.a produce da una cartella di NF-e reali, dove nessuna approvazione
esiste.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

CODICE_DISCORDANZA = 1
CODICE_AMBIENTE = 2

# La prova stampa code point che una console cp1252 non sa codificare, e
# `py -I` ignora `PYTHONIOENCODING`: lo script si configura da se', perche'
# «ripetibile» significa che non dipende dalla console di chi lo lancia.
for _flusso in (sys.stdout, sys.stderr):
    try:
        _flusso.reconfigure(encoding="utf-8", errors="backslashreplace")
    except (AttributeError, OSError):  # flusso non riconfigurabile: non e' fatale
        pass

# Gli stessi casi di `src/xlsx.rs`, mod tx38::TEXTOS, nello stesso ordine.
# Sono le **attese dichiarate**, non una seconda copia della logica del writer.
TESTI_ATTESI: list[tuple[str, str]] = [
    ("formula_igual", '=1+1 SUCO "MARCA" Açucar'),
    ("mais", "+55 11"),
    ("menos", "-7"),
    ("arroba", "@utente"),
    ("zeros", "00012345678901"),
    ("quebra", "linha1\nlinha2"),
    ("tab", "a\tb"),
    ("nfc", "Açucar"),
    ("nfd", "Açucar"),
    ("escape_literal", "_x0041_"),
    ("escape_protegido", "_x005F_x0041_"),
    ("monetario_texto", "10.0000"),
    ("monetario_escala_zero", "0.00"),
    ("monetario_negativo", "-7.50"),
    ("monetario_longo", "79228162514264337593543950335"),
]

# Testo autorevole e valore atteso della cella numerica di comodo (D20, D26a).
NUMERI_ATTESI: list[tuple[str, float]] = [
    ("10.0000", 10.0),
    ("0.00", 0.0),
    ("-7.50", -7.5),
    ("0.01", 0.01),
    ("12345678901234.56", 12_345_678_901_234.56),
]


# ---------------------------------------------------------------------------
# Il valore che il pacchetto rappresenta (R3C41-04, residuo del 2026-10-30)
# ---------------------------------------------------------------------------

NS_FOGLIO = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
NS_RELAZIONI = "http://schemas.openxmlformats.org/package/2006/relationships"
NS_R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"

_ESCAPE = re.compile(r"_x([0-9A-Fa-f]{4})_")


def decodifica_passata_singola(testo: str) -> str:
    """La decodifica **corretta** della convenzione `ST_Xstring`: una passata.

    Da sinistra a destra, senza sovrapposizioni: ogni `_xHHHH_` diventa il suo
    carattere, e un underscore letterale protetto come `_x005F_` torna
    underscore **senza** che ciò che lo segue venga decodificato una seconda
    volta.

    | Nel pacchetto | Una passata | Significato |
    |---|---|---|
    | `a_x000D_b` | `a` + U+000D + `b` | un ritorno a capo |
    | `a_x005F_x000D_b` | `a_x000D_b` | quindici caratteri letterali |

    È la differenza che distingue un file corretto da uno alterato quando il
    lettore mostra la stessa cosa nei due casi.
    """
    return _ESCAPE.sub(lambda m: chr(int(m.group(1), 16)), testo)


class ValoreNonLeggibile(Exception):
    """Il pacchetto non permette di dire che cosa quella cella rappresenti."""


class Pacote:
    """Che cosa il **file** rappresenta, letto senza passare dal lettore.

    `R3C41-04`, residuo del controllo mirato del 2026-10-30: concedere
    un'eccezione sul solo risultato di `openpyxl` rende indistinguibili un
    difetto del lettore e un file sbagliato. Prima di concedere un'eccezione
    si guarda la codifica della cella nel pacchetto **di questo giro** e si
    decodifica a passata singola.

    Non è un secondo lettore generale: legge le sole celle delle eccezioni
    registrate, con `zipfile` e `xml.etree`, già presenti.
    """

    def __init__(self, caminho: str) -> None:
        self.caminho = caminho
        with zipfile.ZipFile(caminho) as z:
            self._condivise = self._stringhe_condivise(z)
            self._percorsi = self._mappa_dei_fogli(z)
            self._xml = {
                nome: z.read(percorso) for nome, percorso in self._percorsi.items()
            }
        self._celle: dict[str, dict[tuple[int, int], str]] = {}
        self._intestazioni: dict[str, dict[str, int]] = {}

    @staticmethod
    def _stringhe_condivise(z: zipfile.ZipFile) -> list[str]:
        if "xl/sharedStrings.xml" not in z.namelist():
            return []
        albero = ET.fromstring(z.read("xl/sharedStrings.xml"))
        valori = []
        for si in albero.iter(f"{{{NS_FOGLIO}}}si"):
            testo = si.find(f"{{{NS_FOGLIO}}}t")
            valori.append("" if testo is None or testo.text is None else testo.text)
        return valori

    @staticmethod
    def _mappa_dei_fogli(z: zipfile.ZipFile) -> dict[str, str]:
        """Nome del foglio → suo XML, **dalle relazioni** e non dall'ordine."""
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

    @staticmethod
    def _indice_di_colonna(riferimento: str) -> int:
        lettere = re.match(r"([A-Z]+)", riferimento).group(1)
        indice = 0
        for lettera in lettere:
            indice = indice * 26 + (ord(lettera) - ord("A") + 1)
        return indice

    def _testo_grezzo(self, cella: ET.Element) -> str | None:
        """Il testo **come sta nel pacchetto**, prima di `ST_Xstring`."""
        tipo = cella.get("t")
        if tipo == "s":
            valore = cella.find(f"{{{NS_FOGLIO}}}v")
            if valore is None or valore.text is None:
                return None
            indice = int(valore.text)
            if indice >= len(self._condivise):
                raise ValoreNonLeggibile(
                    f"la cella rimanda alla stringa condivisa {indice}, che "
                    f"non esiste: ce ne sono {len(self._condivise)}"
                )
            return self._condivise[indice]
        if tipo == "inlineStr":
            testo = cella.find(f"{{{NS_FOGLIO}}}is/{{{NS_FOGLIO}}}t")
            return "" if testo is None else (testo.text or "")
        return None

    def _celle_del_foglio(self, folha: str) -> dict[tuple[int, int], str]:
        if folha in self._celle:
            return self._celle[folha]
        if folha not in self._xml:
            raise ValoreNonLeggibile(f"foglio {folha!r} assente nel pacchetto")
        mappa: dict[tuple[int, int], str] = {}
        albero = ET.fromstring(self._xml[folha])
        for riga in albero.iter(f"{{{NS_FOGLIO}}}row"):
            numero = int(riga.get("r"))
            for cella in riga.iter(f"{{{NS_FOGLIO}}}c"):
                grezzo = self._testo_grezzo(cella)
                if grezzo is not None:
                    mappa[(numero, self._indice_di_colonna(cella.get("r")))] = grezzo
        self._celle[folha] = mappa
        return mappa

    def _colonna(self, folha: str, nome: str) -> int:
        if folha not in self._intestazioni:
            celle = self._celle_del_foglio(folha)
            self._intestazioni[folha] = {
                testo: colonna
                for (riga, colonna), testo in celle.items()
                if riga == 1
            }
        intestazione = self._intestazioni[folha]
        if nome not in intestazione:
            raise ValoreNonLeggibile(
                f"colonna {nome!r} assente nell'intestazione di {folha}"
            )
        return intestazione[nome]

    def valore_representado(self, cella: dict) -> tuple[str, str]:
        """`(testo grezzo nel pacchetto, testo rappresentato)`."""
        folha = cella["folha"]
        riga = cella["linha"]
        colonna = cella.get("coluna_indice") or self._colonna(folha, cella["coluna"])
        grezzo = self._celle_del_foglio(folha).get((riga, colonna))
        if grezzo is None:
            raise ValoreNonLeggibile(
                f"nessuna cella testuale in {folha}, riga {riga}, colonna {colonna}"
            )
        return grezzo, decodifica_passata_singola(grezzo)

# ---------------------------------------------------------------------------
# Modo `export`: attese del percorso composto
# ---------------------------------------------------------------------------

CHAVE_ATTESA = "29260512345678000195550010001256612345678901"

FORMATO_ATTESO = "FANTOZZI-EXPORT/v1"

# Il motivo dei due eventi del percorso di prova contiene un **ritorno a capo
# nudo**: `R3C41-04`. E' un carattere XML 1.0 valido, quindi il modello non lo
# rifiuta e il writer non lo compensa.
MOTIVO_COM_CR = "visto e aceito a\rb"

# Attese **esplicite**, trascritte a mano dai valori noti del percorso di
# prova: foglio, riga del workbook, colonna, valore atteso, tipo di cella.
# `None` come valore atteso significa «la cella non deve esistere».
#
# Non sostituiscono il confronto campo per campo: lo sorvegliano. Un manifesto
# generato dallo stesso modello seguirebbe una mutazione del modello senza
# accorgersene, mentre queste righe no.
ATTESE_ESPLICITE: list[tuple[str, int, str, object, str]] = [
    # Metadati del derivato.
    ("proveniencia", 2, "formato", FORMATO_ATTESO, "s"),
    ("proveniencia", 2, "versao_schema", "1", "s"),
    ("proveniencia", 2, "lote_id", "lote-export-3c41", "s"),
    ("proveniencia", 2, "plano_coincide", "SIM", "s"),
    ("proveniencia", 2, "apresentacao_divergente", "NAO", "s"),
    ("proveniencia", 2, "eventos_no_arquivo", 2, "n"),
    ("proveniencia", 2, "eventos_aplicados", 2, "n"),
    ("proveniencia", 2, "eventos_nao_aplicados", 0, "n"),
    ("proveniencia", 2, "plano_arquivado_sha256", None, ""),
    # Testo ostile, chiave e assenze dichiarate.
    ("diagnostico", 2, "descricao_itens", '=1+1 SUCO "MARCA" Açucar', "s"),
    ("diagnostico", 2, "cnpj_cpf_favorecido", "00012345678901", "s"),
    ("diagnostico", 2, "chave_origem", CHAVE_ATTESA, "s"),
    ("diagnostico", 2, "valor", "10.00", "s"),
    ("diagnostico", 2, "valor_numero", 10.0, "n"),
    ("diagnostico", 2, "data_pagamento", None, ""),
    ("diagnostico", 2, "data_pagamento_status", "NAO_DISPONIVEL", "s"),
    # `R3C41-05`: tre stati, tre fonti. L'esito **tecnico** della 3B non e' la
    # decisione umana, e un'approvazione non lo riscrive: nel campione il
    # tecnico e' OK, la decisione e' APROVADO_POR_PESSOA e la partizione e'
    # APROVADO. Z2, AA2 e AB2 del print verificato con Luca.
    ("diagnostico", 2, "status_tecnico", "OK", "s"),
    ("diagnostico", 2, "estado_revisao", "APROVADO_POR_PESSOA", "s"),
    ("diagnostico", 2, "particao", "APROVADO", "s"),
    # Totali.
    ("resumos", 2, "valor_total", "10.00", "s"),
    ("resumos", 2, "valor_total_numero", 10.0, "n"),
    ("resumos", 2, "valor_aprovado", "10.00", "s"),
    ("resumos", 2, "valor_revisar", "0.00", "s"),
    ("resumos", 2, "qtd_total", 1, "n"),
    ("resumos", 2, "qtd_aprovado", 1, "n"),
    ("resumos", 2, "documentos_ok", 1, "n"),
    # Chiave, stato, autore e conferma della riga rivista.
    ("extrato_revisado", 2, "chave_origem", CHAVE_ATTESA, "s"),
    ("extrato_revisado", 2, "conta_id", "2.01.004", "s"),
    ("extrato_revisado", 2, "estado_revisao", "APROVADO_POR_PESSOA", "s"),
    ("extrato_revisado", 2, "aprovado_por", "Alice", "s"),
    ("extrato_revisado", 2, "confirmacoes", 1, "n"),
    ("extrato_revisado", 2, "ultima_confirmacao_por", "Bob", "s"),
    ("extrato_revisado", 2, "valor", "10.00", "s"),
    ("extrato_revisado", 2, "valor_numero", 10.0, "n"),
    # Decisioni: autore, operazione, applicazione e **motivo**.
    ("decisoes", 2, "autor", "Alice", "s"),
    ("decisoes", 2, "operacao", "APROVAR", "s"),
    ("decisoes", 2, "aplicado", "SIM", "s"),
    ("decisoes", 2, "motivo", MOTIVO_COM_CR, "s"),
    ("decisoes", 3, "autor", "Bob", "s"),
    ("decisoes", 3, "motivo", MOTIVO_COM_CR, "s"),
    ("decisoes", 3, "repeticoes", 0, "n"),
    # Avvisi: il codice, non il dettaglio con l'impronta corrente.
    ("avisos", 2, "codigo", "APRESENTACAO_MUDADA", "s"),
    ("avisos", 4, "codigo", "APROVACAO_REPETIDA", "s"),
]

# Indicatori attesi per **nome**, non per posizione: testo autorevole e cella
# numerica di comodo.
INDICADORES_ATTESI: dict[str, tuple[str, float]] = {
    "documentos": ("1", 1.0),
    "linhas": ("1", 1.0),
    "linhas_aprovado": ("1", 1.0),
    "linhas_revisar": ("0", 0.0),
    "valor_total": ("10.00", 10.0),
    "valor_aprovado": ("10.00", 10.0),
    "valor_revisar": ("0.00", 0.0),
    "valor_extrato_aprovado": ("10.00", 10.0),
    "linhas_excluidas_do_extrato": ("0", 0.0),
}

# ---------------------------------------------------------------------------
# Modo `percurso-cli` (fase 3C.5.a): attese **sue**, non quelle della fixture
# ---------------------------------------------------------------------------
#
# `R3C5A-03`: le attese di `--modo export` sono cablate sulla fixture del
# percorso 3C.4.1 — lotto `lote-export-3c41`, importo 10.00, due eventi,
# Alice e Bob, `APROVADO_POR_PESSOA`. Un workbook nato dalla CLI su una
# cartella di NF-e reali **non** ha nulla di tutto questo, e generare un
# manifesto diverso non basterebbe: le attese esplicite girano sul workbook
# comunque. Quindi il percorso nuovo ha il suo modo, e quelle della fixture
# restano intatte.
#
# Valori **misurati** sul deposito di `tests/cli_percurso_nfe.rs`
# (`tx63`): tre NF-e derivate dal campione reale, nessuna approvazione.
ATTESE_PERCURSO_CLI: list[tuple[str, int, str, object, str]] = [
    # L'esecuzione che il risultato copre: un lotto, non un mese.
    ("proveniencia", 2, "formato", FORMATO_ATTESO, "s"),
    ("proveniencia", 2, "versao_schema", "1", "s"),
    ("proveniencia", 2, "lote_id", "fonte-maio-r1", "s"),
    ("proveniencia", 2, "lote_gerado_em", "2026-05-31", "s"),
    ("proveniencia", 2, "lote_descricao", "pasta de maio", "s"),
    ("proveniencia", 2, "plano_arquivado_versao", "1.0", "s"),
    ("proveniencia", 2, "plano_coincide", "SIM", "s"),
    ("proveniencia", 2, "apresentacao_divergente", "NAO", "s"),
    # `CLI-02`: l'importazione non crea nessuna decisione umana. Qui si vede
    # nei numeri, non in una promessa.
    ("proveniencia", 2, "eventos_no_arquivo", 0, "n"),
    ("proveniencia", 2, "eventos_aplicados", 0, "n"),
    ("proveniencia", 2, "eventos_nao_aplicados", 0, "n"),
    # I tre stati con le tre fonti distinte (`R3C41-05`), senza approvazioni:
    # il tecnico e' OK, lo stato della vista e' TECNICO_OK — **non**
    # APROVADO_POR_PESSOA — e nulla entra nell'estratto approvato.
    ("diagnostico", 2, "conta_id", "2.01.002", "s"),
    ("diagnostico", 2, "valor", "20.00", "s"),
    ("diagnostico", 2, "status_tecnico", "OK", "s"),
    ("diagnostico", 2, "estado_revisao", "TECNICO_OK", "s"),
    ("diagnostico", 2, "particao", "OK", "s"),
    ("diagnostico", 2, "no_extrato_aprovado", "NAO", "s"),
    ("diagnostico", 2, "no_extrato_revisado", "NAO", "s"),
    ("diagnostico", 5, "conta_id", "2.01.004", "s"),
    ("diagnostico", 5, "valor", "10.00", "s"),
    ("diagnostico", 5, "status_tecnico", "OK", "s"),
    ("diagnostico", 5, "estado_revisao", "TECNICO_OK", "s"),
]

# Tre NF-e da tre articoli: nove righe, 105.00 in tutto, e **niente**
# approvato, quindi tutte le nove righe restano fuori dall'estratto approvato.
INDICADORES_PERCURSO_CLI: dict[str, tuple[str, float]] = {
    "documentos": ("3", 3.0),
    "documentos_ok": ("3", 3.0),
    "documentos_erro": ("0", 0.0),
    "linhas": ("9", 9.0),
    "linhas_ok": ("9", 9.0),
    "linhas_aprovado": ("0", 0.0),
    "linhas_revisar": ("0", 0.0),
    "valor_total": ("105.00", 105.0),
    "valor_ok": ("105.00", 105.0),
    "valor_aprovado": ("0.00", 0.0),
    "valor_extrato_aprovado": ("0.00", 0.0),
    "linhas_excluidas_do_extrato": ("9", 9.0),
}


def attese_do_modo(modo: str) -> list[tuple[str, int, str, object, str]]:
    """Le attese esplicite **di questo modo**, mai quelle di un altro."""
    return ATTESE_PERCURSO_CLI if modo == "percurso-cli" else ATTESE_ESPLICITE


def indicadores_do_modo(modo: str) -> dict[str, tuple[str, float]]:
    return (
        INDICADORES_PERCURSO_CLI if modo == "percurso-cli" else INDICADORES_ATTESI
    )


def _modi_da_nota(nota: dict) -> tuple[str, ...]:
    """I modi a cui una registrazione appartiene.

    Resta una stringa quando la divergenza vale per un solo percorso; una
    tupla dichiara che la **stessa** cella diverge in piu' percorsi, come la
    frase del foglio `limites`, che ogni export porta identica.
    """
    modo = nota["modo"]
    return (modo,) if isinstance(modo, str) else tuple(modo)

# I limiti che il file deve dichiarare di sé, con il loro stato.
LIMITES_ATTESI: dict[str, str] = {
    "DERIVADO_SEM_RETORNO": "DICHIARATO",
    "AUTOR_NAO_AUTENTICADO": "DICHIARATO",
    "NUMERO_NAO_AUTORITATIVO": "DICHIARATO",
    "ESCAPE_LEITOR": "DIVERGENZA_NOTA",
    "EXCEL_NAO_ATESTADO": "NON_VERIFICATO",
    "SEM_SEQUENCIA_HISTORICA": "DICHIARATO",
    "DURABILIDADE_ARQUIVO": "NON_VERIFICATO",
    "EXCLUSAO_FONTE": "NON_VERIFICATO",
    "RECUSAR_NAO_ATOMICO": "DICHIARATO",
}

# Divergenze **note** del lettore, registrate con la decisione del 2026-10-23
# (strada (a)). Ognuna e' identificata da: confronto, testo in ingresso,
# lettore con la sua versione, risultato osservato e perche' non e' un difetto
# del nostro file.
#
# Regole, volute dalla decisione:
#   - una divergenza registrata che si ripresenta **identica** non e' un errore,
#     e non rende TX-37 integralmente superata: viene elencata a parte;
#   - una divergenza **diversa** da quella registrata e' un errore da esaminare;
#   - una divergenza registrata che **non si presenta piu'** rende la
#     registrazione obsoleta, e va aggiornata: anche questo e' un errore, perche'
#     una registrazione stantia e' una falsa dichiarazione.
#   - una registrazione appartiene a un **modo**: `--modo export` non deve
#     pretendere di esercitare una registrazione del modo `tx38`.
# La riga della prova `tx38` si calcola dall'elenco, invece di essere scritta
# a mano: un elenco che cambia non deve spostare silenziosamente la cella su
# cui si verifica il pacchetto.
_RIGA_ESCAPE_PROTEGIDO = (
    next(i for i, (caso, _) in enumerate(TESTI_ATTESI) if caso == "escape_protegido")
    + 2
)

DIVERGENZE_NOTE: dict[str, dict[str, object]] = {
    "caso[escape_protegido].texto": {
        "modo": "tx38",
        "cella": {"folha": "Prova", "linha": _RIGA_ESCAPE_PROTEGIDO, "coluna_indice": 2},
        "ingresso": "_x005F_x0041_",
        "lettore": "openpyxl",
        "versione_misurata": "3.1.5",
        "osservato": "_x0041_",
        "xml_del_writer": "_x005F_x005F_x005F_x0041_",
        "perche": (
            "l'XML del writer e' corretto: una decodifica a passata singola "
            "della convenzione ST_Xstring lo riporta a '_x005F_x0041_'. Questo "
            "lettore decodifica piu' di una volta e perde un livello. Il caso "
            "del testo '_x0041_' torna invece identico"
        ),
        "decisione": "strada (a) del 2026-10-23: limite dichiarato, nessuna "
        "compensazione nel writer e nessun rifiuto del dato",
    },
}

# `R3C41-04`: il ritorno a capo nudo, misurato il 2026-10-30.
#
# Non e' lo stesso caso dell'escape protetto, e non va assorbito in quello: la
# causa e' diversa. Qui il writer **codifica correttamente** il carattere con
# la convenzione `ST_Xstring` del formato, `_x000D_`, verificato leggendo
# `xl/sharedStrings.xml` con `zipfile`; e' il lettore che non decodifica quella
# sequenza e restituisce i sette caratteri letterali. Nel caso dell'escape
# protetto, invece, il writer protegge `_x` e il lettore decodifica piu' di una
# volta.
#
# Conseguenza dichiarata: la fedelta' del **lettore** su un ritorno a capo non
# e' dimostrata, mentre la fedelta' del **file** lo e'. Nessuna compensazione
# nel writer e nessun rifiuto del dato.
NOTA_RITORNO_A_CAPO: dict[str, object] = {
    "modo": "export",
    "ingresso": MOTIVO_COM_CR,
    # La cella si dichiara per ogni confronto registrato, sotto.
    "lettore": "openpyxl",
    "versione_misurata": "3.1.5",
    "osservato": "visto e aceito a_x000D_b",
    "xml_del_writer": "visto e aceito a_x000D_b",
    "perche": (
        "il ritorno a capo e' un carattere XML 1.0 valido, e il writer lo "
        "codifica come _x000D_ secondo ST_Xstring: l'XML prodotto e' corretto. "
        "openpyxl 3.1.5 non decodifica quella sequenza e restituisce i sette "
        "caratteri letterali. E' un limite di fedelta' del lettore, di causa "
        "diversa da quella dell'escape protetto"
    ),
    "decisione": "strada (a) del 2026-10-23, estesa a questo caso il "
    "2026-10-30: limite dichiarato, nessuna compensazione nel writer e nessun "
    "rifiuto del dato",
}

# Una registrazione per **cella**, perche' il registro e' indicizzato dal nome
# del confronto e una registrazione non esercitata e' un errore.
for _chiave_cr, _riga_cr in (
    ("decisoes!2:motivo", 2),
    ("decisoes!3:motivo", 3),
    ("esplicito.decisoes!2:motivo", 2),
    ("esplicito.decisoes!3:motivo", 3),
):
    DIVERGENZE_NOTE[_chiave_cr] = dict(
        NOTA_RITORNO_A_CAPO,
        cella={"folha": "decisoes", "linha": _riga_cr, "coluna": "motivo"},
    )


# `R3C41-04`, misurato il 2026-10-30: la **stessa** divergenza dell'escape
# protetto, in una cella del workbook reale. La descrizione del limite
# `ESCAPE_LEITOR` cita la sequenza protetta, quindi il limite si applica alla
# frase che lo dichiara.
#
# Il testo nel modello vive in `src/export.rs`, `folha_limites`: se quella
# frase cambia, questa registrazione diventa una discordanza da esaminare, ed
# e' voluto. La registrazione e' identificata dall'ingresso.
DIVERGENZE_NOTE["limites!5:descricao"] = {
    # Ogni export porta questa frase identica, quindi la divergenza si
    # presenta in entrambi i percorsi.
    "modo": ("export", "percurso-cli"),
    "cella": {"folha": "limites", "linha": 5, "coluna": "descricao"},
    "ingresso": "Um texto que contem uma sequencia semelhante a _xHHHH_ ja' protegida pode ser mostrado de forma diferente: medido com openpyxl 3.1.5, a entrada _x005F_x0041_ foi relida _x0041_. O XML produzido e' correto; a perda vem do decodificador do leitor.",
    "lettore": "openpyxl",
    "versione_misurata": "3.1.5",
    "osservato": "Um texto que contem uma sequencia semelhante a _xHHHH_ ja' protegida pode ser mostrado de forma diferente: medido com openpyxl 3.1.5, a entrada _x0041_ foi relida _x0041_. O XML produzido e' correto; a perda vem do decodificador do leitor.",
    "xml_del_writer": "la sequenza _x005F_ e' protetta dal writer come "
    "_x005F_x005F_x005F_, e una decodifica a passata singola la riporta "
    "intatta",
    "perche": (
        "stessa causa della registrazione di `caso[escape_protegido].texto`: "
        "il lettore decodifica piu' di una volta e perde un livello. L'XML "
        "prodotto e' corretto"
    ),
    "decisione": "strada (a) del 2026-10-23: limite dichiarato, nessuna "
    "compensazione nel writer e nessuna riscrittura della frase per aggirare "
    "il lettore",
}

class Esito:
    """Raccoglie i confronti: nessuno viene dichiarato superato per default."""

    def __init__(
        self, versione_lettore: str, modo: str, pacote: "Pacote | None" = None
    ) -> None:
        self.versione_lettore = versione_lettore
        self.modo = modo
        # Il pacchetto serve a dire che cosa il **file** rappresenta, prima di
        # concedere un'eccezione. Senza, nessuna eccezione viene concessa.
        self.pacote = pacote
        self.ok: list[str] = []
        self.discordanze: list[str] = []
        self.divergenze_note: list[str] = []
        self.registrazioni_obsolete: list[str] = []
        self.note_viste: set[str] = set()
        self.xml_misurato: dict[str, str] = {}

    def confronta(self, nome: str, atteso: object, trovato: object) -> None:
        nota = DIVERGENZE_NOTE.get(nome)
        if atteso == trovato:
            if nota is not None:
                self.note_viste.add(nome)
                self.registrazioni_obsolete.append(
                    f"{nome}: la divergenza registrata non si presenta piu' "
                    f"(atteso {atteso!r} e trovato {atteso!r}). La "
                    f"registrazione va aggiornata."
                )
            else:
                self.ok.append(nome)
            return

        if nota is not None:
            self._valuta_eccezione(nome, nota, atteso, trovato)
            return

        self.discordanze.append(
            f"{nome}: atteso {atteso!r}, trovato {trovato!r}"
        )

    def _valuta_eccezione(
        self, nome: str, nota: dict, atteso: object, trovato: object
    ) -> None:
        """Un'eccezione si concede al **caso atteso**, non al risultato visto.

        `R3C41-04`, residuo del controllo mirato del 2026-10-30: concedere
        l'eccezione perche' il lettore restituisce il risultato registrato
        rende indistinguibili due cose diverse — un difetto del lettore su un
        file corretto, e un file alterato che per caso produce lo stesso
        risultato. La codifica `_x005F_` basta a costruire il secondo.

        Cinque condizioni, tutte necessarie:

        1. la registrazione appartiene a **questo** modo;
        2. l'**ingresso** registrato e' il valore atteso di questo confronto;
        3. il lettore e la sua **versione** sono quelli su cui la divergenza
           e' stata misurata;
        4. il risultato osservato e' quello registrato;
        5. il **pacchetto di questo giro** rappresenta esattamente il valore
           atteso, con la decodifica a passata singola di `ST_Xstring`.

        Se una manca, il confronto e' una **discordanza**: un controllo non
        eseguito non risulta superato, e un file che rappresenta qualcosa di
        diverso dal manifesto e' un difetto del file, non un limite del
        lettore.
        """
        self.note_viste.add(nome)
        motivi: list[str] = []

        if self.modo not in _modi_da_nota(nota):
            motivi.append(
                f"la registrazione appartiene al modo {nota['modo']!r} e "
                f"questo giro e' {self.modo!r}"
            )
        if nota["ingresso"] != atteso:
            motivi.append(
                f"l'ingresso registrato e' {nota['ingresso']!r}, diverso dal "
                f"valore atteso di questo confronto"
            )
        if self.versione_lettore != nota["versione_misurata"]:
            motivi.append(
                f"la divergenza e' stata misurata su {nota['lettore']} "
                f"{nota['versione_misurata']} e qui c'e' la "
                f"{self.versione_lettore}: va rimisurata prima di essere "
                f"concessa"
            )
        if trovato != nota["osservato"]:
            motivi.append(
                f"era registrato il risultato {nota['osservato']!r}, e questo "
                f"e' **diverso**"
            )

        grezzo = None
        if self.pacote is None:
            motivi.append(
                "il pacchetto non e' stato aperto, quindi il valore "
                "rappresentato dal file non e' stato verificato"
            )
        else:
            try:
                grezzo, rappresentato = self.pacote.valore_representado(nota["cella"])
            except (ValoreNonLeggibile, KeyError, ValueError, ET.ParseError) as errore:
                motivi.append(
                    f"il valore rappresentato dal file non e' leggibile: {errore}"
                )
            else:
                self.xml_misurato[nome] = grezzo
                if rappresentato != atteso:
                    motivi.append(
                        f"il **file** rappresenta {rappresentato!r} — nel "
                        f"pacchetto {grezzo!r} — e non il valore atteso: e' un "
                        f"difetto del file, non del lettore"
                    )

        if motivi:
            self.discordanze.append(
                f"{nome}: atteso {atteso!r}, trovato {trovato!r}. "
                f"Eccezione NON concessa: " + "; ".join(motivi)
            )
            return

        self.divergenze_note.append(
            f"{nome}: ingresso {nota['ingresso']!r}; il pacchetto contiene "
            f"{grezzo!r}, che rappresenta **esattamente** l'ingresso; "
            f"{nota['lettore']} {self.versione_lettore} restituisce "
            f"{trovato!r}"
        )

    def afferma(self, nome: str, condizione: bool, dettaglio: str) -> None:
        if condizione:
            self.ok.append(nome)
        else:
            self.discordanze.append(f"{nome}: {dettaglio}")

    def note_non_esercitate(self) -> list[str]:
        """Registrazioni **di questo modo** che il giro non ha sfiorato."""
        del_modo = {
            nome
            for nome, nota in DIVERGENZE_NOTE.items()
            if self.modo in _modi_da_nota(nota)
        }
        return sorted(del_modo - self.note_viste)


def provenienza(openpyxl_modulo) -> None:
    print("--- provenienza degli strumenti ---")
    print(f"python     {sys.version.split()[0]}  {sys.executable}")
    print(f"openpyxl   {openpyxl_modulo.__version__}  {openpyxl_modulo.__file__}")


def esamina_contenitore(caminho: str, esito: Esito) -> None:
    """Il contenitore, senza passare per il lettore di fogli."""
    print("--- contenitore ---")
    with zipfile.ZipFile(caminho) as z:
        voci = z.infolist()
        compresse = sum(1 for i in voci if i.compress_type != zipfile.ZIP_STORED)
        print(f"voci       {len(voci)}")
        print(f"compresse  {compresse} su {len(voci)}")
        esito.afferma(
            "contenitore_leggibile", bool(voci), "nessuna voce nello ZIP"
        )
        difetto = z.testzip()
        esito.afferma(
            "crc_integro", difetto is None, f"CRC non valido in {difetto}"
        )


def modo_tx38(caminho: str, openpyxl_modulo, esito: Esito) -> None:
    """TX-38 e TX-37 sul workbook minimo."""
    livro = openpyxl_modulo.load_workbook(caminho)
    esito.confronta("fogli", ["Prova", "Numeros"], livro.sheetnames)

    folha = livro["Prova"]
    print("--- testi: tipo effettivo della cella e valore riletto ---")
    for i, (caso, atteso) in enumerate(TESTI_ATTESI, start=2):
        rotulo = folha.cell(row=i, column=1).value
        cella = folha.cell(row=i, column=2)
        esito.confronta(f"caso[{caso}].rotulo", caso, rotulo)
        esito.confronta(f"caso[{caso}].texto", atteso, cella.value)
        esito.confronta(f"caso[{caso}].tipo", "s", cella.data_type)
        print(f"  {caso:<22} tipo={cella.data_type!r} valore={cella.value!r}")

    # Nessuna cella di tipo formula, in nessun foglio.
    formule = [
        f"{nome}!{c.coordinate}"
        for nome in livro.sheetnames
        for riga in livro[nome].iter_rows()
        for c in riga
        if c.data_type == "f"
    ]
    esito.afferma(
        "nessuna_formula", not formule, f"celle di tipo formula: {formule}"
    )

    numeros = livro["Numeros"]
    print("--- numeri: testo autorevole e cella di comodo ---")
    for i, (testo, numero) in enumerate(NUMERI_ATTESI, start=2):
        cella_testo = numeros.cell(row=i, column=1)
        cella_numero = numeros.cell(row=i, column=2)
        # Il testo autorevole si confronta **esattamente**, scala compresa.
        esito.confronta(f"numero[{testo}].texto", testo, cella_testo.value)
        esito.confronta(f"numero[{testo}].texto_tipo", "s", cella_testo.data_type)
        # La cella di comodo si confronta **per valore**: la scala non e'
        # promessa (D26a, condizione 4).
        esito.confronta(f"numero[{testo}].valore", numero, cella_numero.value)
        esito.confronta(f"numero[{testo}].tipo", "n", cella_numero.data_type)
        print(
            f"  testo={cella_testo.value!r:<34} "
            f"numero={cella_numero.value!r} tipo={cella_numero.data_type!r}"
        )
    livro.close()


FOLHAS_ATTESE = [
    "proveniencia",
    "diagnostico",
    "rate",
    "documentos",
    "reconciliacao_componentes",
    "reconciliacao_campos_rinviati",
    "resumos",
    "indicadores",
    "extrato_revisado",
    "exclusoes",
    "decisoes",
    "avisos",
    "limites",
]


def carrega_manifesto(caminho: str) -> dict:
    """Il manifesto del modello, depositato dal test di integrazione.

    `R3C41-04`: senza le attese di **ogni** cella il confronto sarebbe a
    campione, e una mutazione di un totale, di una chiave, di uno stato, di un
    autore o di un motivo potrebbe passare inosservata.
    """
    with open(caminho, "r", encoding="utf-8") as f:
        manifesto = json.load(f)
    if manifesto.get("formato") != FORMATO_ATTESO:
        raise ValueError(
            f"manifesto di un formato inatteso: {manifesto.get('formato')!r}"
        )
    return manifesto


def confronta_cella(esito: "Esito", chiave: str, atteso: dict, cella) -> None:
    """Una cella: valore **e** tipo, con il trattamento esplicito delle assenze."""
    tipo = atteso["tipo"]

    if tipo == "vazia":
        # `Celula::Vazia` e' un'assenza: il writer non scrive la cella, e il
        # lettore restituisce `None`. Non e' lo zero e non e' il testo vuoto.
        esito.afferma(
            f"{chiave}.vazia",
            cella.value is None,
            f"attesa assenza di cella, trovato {cella.value!r} "
            f"(tipo {cella.data_type})",
        )
        return

    if tipo == "texto":
        valore = atteso["valor"]
        if valore == "":
            # Un testo **vuoto** non si distingue da un'assenza alla
            # rilettura: il writer non emette la cella. La differenza vive nel
            # manifesto, e qui si dichiara invece di essere taciuta.
            esito.afferma(
                f"{chiave}.texto_vazio",
                cella.value in (None, ""),
                f"atteso testo vuoto, trovato {cella.value!r}",
            )
            return
        esito.confronta(chiave, valore, cella.value)
        esito.confronta(f"{chiave}.tipo", "s", cella.data_type)
        return

    if tipo == "numero":
        # Il manifesto porta il decimale **esatto**; la cella di comodo e'
        # l'unico punto in cui il valore passa per la virgola mobile, quindi il
        # confronto si fa su `float` del decimale, senza tolleranze: una
        # tolleranza nasconderebbe proprio il difetto che si cerca.
        esito.confronta(chiave, float(atteso["valor"]), cella.value)
        esito.confronta(f"{chiave}.tipo", "n", cella.data_type)
        esito.afferma(
            f"{chiave}.numerico",
            isinstance(cella.value, (int, float)) and not isinstance(cella.value, bool),
            f"attesa una cella numerica, trovato {type(cella.value).__name__}",
        )
        return

    esito.discordanze.append(f"{chiave}: tipo di cella non previsto {tipo!r}")


def confronto_campo_per_campo(livro, manifesto: dict, esito: "Esito") -> None:
    """Ogni foglio, ogni intestazione, ogni cella del manifesto."""
    nomi_attesi = [f["nome"] for f in manifesto["folhas"]]
    esito.confronta("folhas", nomi_attesi, livro.sheetnames)

    celle = 0
    for descricao in manifesto["folhas"]:
        nome = descricao["nome"]
        if nome not in livro.sheetnames:
            esito.discordanze.append(f"{nome}: foglio assente nel workbook")
            continue
        folha = livro[nome]
        cabecalho = descricao["cabecalho"]
        linhas = descricao["linhas"]

        letto = [c.value for c in next(folha.iter_rows(min_row=1, max_row=1))]
        esito.confronta(f"{nome}!cabecalho", cabecalho, letto)
        esito.confronta(f"{nome}!linhas", len(linhas), folha.max_row - 1)
        esito.confronta(f"{nome}!colunas", len(cabecalho), folha.max_column)

        for i, linha in enumerate(linhas):
            esito.afferma(
                f"{nome}!linha[{i}].largura",
                len(linha) == len(cabecalho),
                f"la riga ha {len(linha)} celle e l'intestazione "
                f"{len(cabecalho)}",
            )
            for j, atteso in enumerate(linha):
                coluna = cabecalho[j] if j < len(cabecalho) else f"coluna{j + 1}"
                chiave = f"{nome}!{i + 2}:{coluna}"
                confronta_cella(
                    esito, chiave, atteso, folha.cell(row=i + 2, column=j + 1)
                )
                celle += 1

    print(f"celle confrontate   {celle}")


def confronto_esplicito(livro, esito: "Esito") -> None:
    """Attese scritte a mano, **indipendenti** dal manifesto.

    Il manifesto nasce dallo stesso modello che genera il file: se il modello
    cambiasse un totale, il manifesto cambierebbe con lui, e il confronto
    campo per campo non se ne accorgerebbe. Queste attese sono invece
    trascritte qui dai valori noti del percorso di prova, e una mutazione di
    un totale, di una chiave, di uno stato, di un autore o di un motivo le fa
    fallire.
    """
    print("--- attese esplicite ---")
    for folha_nome, riga, coluna, atteso, tipo in attese_do_modo(esito.modo):
        folha = livro[folha_nome]
        cabecalho = [c.value for c in next(folha.iter_rows(min_row=1, max_row=1))]
        if coluna not in cabecalho:
            esito.discordanze.append(
                f"esplicito.{folha_nome}.{coluna}: colonna assente"
            )
            continue
        cella = folha.cell(row=riga, column=cabecalho.index(coluna) + 1)
        chiave = f"esplicito.{folha_nome}!{riga}:{coluna}"
        if atteso is None:
            esito.afferma(
                f"{chiave}.ausente",
                cella.value is None,
                f"attesa assenza, trovato {cella.value!r}",
            )
        else:
            esito.confronta(chiave, atteso, cella.value)
            esito.confronta(f"{chiave}.tipo", tipo, cella.data_type)

    # Un indicatore si cerca per **nome**, non per posizione.
    indicadores = livro["indicadores"]
    cab_i = [c.value for c in next(indicadores.iter_rows(min_row=1, max_row=1))]
    trovati = {
        indicadores.cell(row=r, column=cab_i.index("nome") + 1).value: (
            indicadores.cell(row=r, column=cab_i.index("valor") + 1).value,
            indicadores.cell(row=r, column=cab_i.index("valor_numero") + 1).value,
        )
        for r in range(2, indicadores.max_row + 1)
    }
    for nome, (texto, numero) in indicadores_do_modo(esito.modo).items():
        letto = trovati.get(nome, (None, None))
        esito.confronta(f"esplicito.indicadores[{nome}].valor", texto, letto[0])
        esito.confronta(
            f"esplicito.indicadores[{nome}].valor_numero", numero, letto[1]
        )

    # I limiti dichiarati nel file, con il loro stato.
    limites = livro["limites"]
    codici = {
        limites.cell(row=i, column=1).value: limites.cell(row=i, column=3).value
        for i in range(2, limites.max_row + 1)
    }
    for codigo, stato in LIMITES_ATTESI.items():
        esito.confronta(f"esplicito.limites[{codigo}]", stato, codici.get(codigo))
    esito.afferma(
        "esplicito.limites.completo",
        len(codici) == len(LIMITES_ATTESI),
        f"attesi {len(LIMITES_ATTESI)} limiti, trovati {len(codici)}: "
        f"{sorted(codici)}",
    )


def modo_export(
    caminho: str, manifesto_caminho: str, openpyxl_modulo, esito: "Esito"
) -> None:
    """Il workbook reale dell'export, confrontato campo per campo."""
    manifesto = carrega_manifesto(manifesto_caminho)
    livro = openpyxl_modulo.load_workbook(caminho)

    # Nessuna cella di tipo formula, in nessun foglio: EXP-11, EXP-13.
    formule = [
        f"{nome}!{c.coordinate}"
        for nome in livro.sheetnames
        for riga in livro[nome].iter_rows()
        for c in riga
        if c.data_type == "f"
    ]
    esito.afferma(
        "nessuna_formula", not formule, f"celle di tipo formula: {formule}"
    )

    confronto_campo_per_campo(livro, manifesto, esito)
    confronto_esplicito(livro, esito)
    if esito.modo == "percurso-cli":
        confronti_do_percurso_cli(livro, esito)
    livro.close()


def confronti_do_percurso_cli(livro, esito: "Esito") -> None:
    """Due fatti che il percorso della CLI deve mostrare **nel file**.

    1. `D28`: l'impronta del piano archiviato non e' piu' assente. Si verifica
       la **forma**, 64 esadecimali, e non un valore scritto a mano: cablarlo
       vorrebbe dire copiare qui i byte di una fonte canonica, e un suo
       aggiornamento legittimo farebbe fallire la rilettura.
    2. `CLI-02`: senza approvazioni i fogli delle decisioni, dell'estratto
       revisionato e degli avvisi non hanno nessuna riga di dati. E' il
       controllo piu' diretto che un'importazione non approva niente.
    """
    print("--- confronti del percorso CLI ---")
    prov = livro["proveniencia"]
    cabecalho = [c.value for c in next(prov.iter_rows(min_row=1, max_row=1))]
    impronta = prov.cell(
        row=2, column=cabecalho.index("plano_arquivado_sha256") + 1
    ).value
    esito.afferma(
        "percurso_cli.plano_arquivado_sha256.presente",
        isinstance(impronta, str) and bool(re.fullmatch(r"[0-9a-f]{64}", impronta)),
        f"attesa un'impronta di 64 esadecimali, trovato {impronta!r}",
    )
    for nome in ("decisoes", "extrato_revisado", "avisos"):
        folha = livro[nome]
        # `max_row` conta l'intestazione: una sola riga significa nessun dato.
        esito.afferma(
            f"percurso_cli.{nome}.sem_linhas",
            folha.max_row <= 1,
            f"attese zero righe di dati in {nome}, trovate {folha.max_row - 1}",
        )


def main() -> int:
    analizzatore = argparse.ArgumentParser(description=__doc__)
    analizzatore.add_argument("caminho", help="workbook da rileggere")
    analizzatore.add_argument(
        "--modo",
        choices=["tx38", "export", "percurso-cli"],
        default="tx38",
        help="insieme di confronti",
    )
    analizzatore.add_argument(
        "--manifesto",
        help=(
            "manifesto JSON del modello, depositato accanto al workbook dal "
            "test di integrazione. Obbligatorio con --modo export e con "
            "--modo percurso-cli"
        ),
    )
    argomenti = analizzatore.parse_args()

    if argomenti.modo in ("export", "percurso-cli") and not argomenti.manifesto:
        print(
            f"AMBIENTE: --modo {argomenti.modo} richiede --manifesto: senza "
            "le attese di ogni cella il confronto sarebbe a campione.",
            file=sys.stderr,
        )
        print(
            "La verifica NON e' stata eseguita: non risulta superata.",
            file=sys.stderr,
        )
        return CODICE_AMBIENTE

    try:
        import openpyxl
    except ImportError as errore:
        print(f"AMBIENTE: openpyxl non disponibile: {errore}", file=sys.stderr)
        print("La verifica NON e' stata eseguita: non risulta superata.", file=sys.stderr)
        return CODICE_AMBIENTE

    provenienza(openpyxl)
    esito = Esito(openpyxl.__version__, argomenti.modo)
    try:
        esamina_contenitore(argomenti.caminho, esito)
        # Che cosa il file rappresenta, letto senza passare dal lettore: serve
        # prima di concedere qualunque eccezione.
        esito.pacote = Pacote(argomenti.caminho)
        if argomenti.modo == "tx38":
            modo_tx38(argomenti.caminho, openpyxl, esito)
        elif argomenti.modo in ("export", "percurso-cli"):
            modo_export(
                argomenti.caminho, argomenti.manifesto, openpyxl, esito
            )
    except (
        OSError,
        ValueError,
        KeyError,
        json.JSONDecodeError,
        zipfile.BadZipFile,
        ET.ParseError,
    ) as errore:
        print(f"AMBIENTE: {errore}", file=sys.stderr)
        print(
            "La verifica NON e' stata eseguita: non risulta superata.",
            file=sys.stderr,
        )
        return CODICE_AMBIENTE

    print("--- esito ---")
    print(f"confronti superati  {len(esito.ok)}")
    print(f"divergenze note     {len(esito.divergenze_note)}")
    print(f"discordanze         {len(esito.discordanze)}")

    for nome in esito.divergenze_note:
        print(f"  DIVERGENZA NOTA {nome}")
    for nome in sorted(esito.note_viste):
        nota = DIVERGENZE_NOTE[nome]
        print(f"  REGISTRAZIONE {nome}")
        print(f"    perche'        {nota['perche']}")
        print(f"    decisione      {nota['decisione']}")
        print(f"    xml dichiarato {nota['xml_del_writer']!r}")
        misurato = esito.xml_misurato.get(nome)
        print(
            f"    xml misurato   {misurato!r}"
            if misurato is not None
            else "    xml misurato   NON_VERIFICATO: il pacchetto non e' stato letto"
        )

    for riga in esito.registrazioni_obsolete:
        print(f"  REGISTRAZIONE OBSOLETA {riga}")
    for nome in esito.note_non_esercitate():
        print(f"  REGISTRAZIONE NON ESERCITATA {nome}: il confronto non e' stato eseguito")
    for riga in esito.discordanze:
        print(f"  DISCORDANZA {riga}")

    if esito.discordanze or esito.registrazioni_obsolete:
        return CODICE_DISCORDANZA
    if esito.note_non_esercitate():
        print("Una registrazione non esercitata non si puo' dichiarare valida.")
        return CODICE_DISCORDANZA

    if esito.divergenze_note:
        print(
            "ESITO: tutti i confronti tornano, **tranne** le divergenze note "
            "elencate sopra."
        )
        print(
            "Il confronto NON si dichiara integralmente superato: TX-37 resta "
            "non integralmente superata, la fedelta' del lettore sui casi "
            "elencati resta dichiarata come limite, e TX-19 su Excel resta "
            "NON_ESEGUITA."
        )
        return 0

    print("TUTTI I CONFRONTI TORNANO, nessuna divergenza nota osservata.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

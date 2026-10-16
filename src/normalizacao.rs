//! Normalizzazione del testo prima del match degli alias.
//!
//! Porting di `src/fantozzi/normalization.py`:
//!
//! ```python
//! text = text.lower()
//! text = unicodedata.normalize("NFD", text)
//! text = "".join(ch for ch in text if unicodedata.category(ch) != "Mn")
//! text = re.sub(r'[-/.,;:!()"]', " ", text)
//! text = re.sub(r"\s+", " ", text).strip()
//! ```
//!
//! Contratto C9: il Python applica i quattro passi in modo fisso e **ignora**
//! la sezione `normalizacao:` degli YAML. Qui i passi sono guidati da quei
//! flag, cosi' la configurazione descrive davvero cio' che il codice fa.
//! Nei due file di regole versionati i quattro flag sono tutti `true`, quindi
//! sul dato reale il risultato e' identico al Python.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::regras::Normalizacao;

/// Punteggiatura sostituita da spazio. Stessa classe della regex Python
/// `[-/.,;:!()"]`: cambiare questo insieme cambia i match, quindi e' un
/// contratto, non un dettaglio.
pub const PONTUACAO: [char; 10] = ['-', '/', '.', ',', ';', ':', '!', '(', ')', '"'];

/// Normalizza `texto` secondo la politica dichiarata nello YAML.
pub fn normalizar(texto: &str, politica: Normalizacao) -> String {
    let mut saida = if politica.minusculas {
        texto.to_lowercase()
    } else {
        texto.to_string()
    };

    if politica.remover_acentos {
        // NFD separa la lettera base dai segni diacritici; poi i segni cadono.
        saida = saida.nfd().filter(|c| !is_combining_mark(*c)).collect();
    }

    if politica.remover_pontuacao {
        saida = saida.replace(|c| PONTUACAO.contains(&c), " ");
    }

    if politica.compactar_espacos {
        saida = saida.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    saida
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completa() -> Normalizacao {
        Normalizacao {
            minusculas: true,
            remover_acentos: true,
            remover_pontuacao: true,
            compactar_espacos: true,
        }
    }

    /// Gli stessi casi di `tests/test_normalization.py`.
    #[test]
    fn casos_do_python() {
        let p = completa();
        assert_eq!(normalizar("AGUA SANITARIA", p), "agua sanitaria");
        assert_eq!(normalizar("ÁGUA SANITÁRIA", p), "agua sanitaria");
        assert_eq!(normalizar("COCA-COLA LATA", p), "coca cola lata");
        assert_eq!(normalizar("Rio Sol", p), "rio sol");
        assert_eq!(normalizar("", p), "");
    }

    /// Accenti del portoghese: tutti devono cadere, non solo quelli comuni.
    #[test]
    fn remove_todos_os_acentos_do_portugues() {
        let p = completa();
        assert_eq!(
            normalizar("ÁÀÂÃÄ ÉÊ ÍÏ ÓÔÕÖ ÚÜ Ç Ñ", p),
            "aaaaa ee ii oooo uu c n"
        );
        assert_eq!(
            normalizar("Manutenção de Refrigeração", p),
            "manutencao de refrigeracao"
        );
        assert_eq!(normalizar("PÃO DE QUEIJO", p), "pao de queijo");
    }

    #[test]
    fn compacta_espacos_e_apara_bordas() {
        let p = completa();
        assert_eq!(
            normalizar("  CERVEJA   \t HEINEKEN \n ", p),
            "cerveja heineken"
        );
    }

    #[test]
    fn substitui_toda_a_pontuacao_declarada() {
        let p = completa();
        assert_eq!(
            normalizar("a-b/c.d,e;f:g!h(i)j\"k", p),
            "a b c d e f g h i j k"
        );
    }

    /// L'apostrofo **non** e' nella classe della regex Python: non va rimosso,
    /// altrimenti descrizioni come "D'AGUA" verrebbero divise diversamente.
    #[test]
    fn apostrofo_nao_e_pontuacao_removida() {
        assert_eq!(normalizar("AGUA D'OURO", completa()), "agua d'ouro");
    }

    /// Contratto C9: i flag contano davvero. Se la configurazione dicesse
    /// `remover_acentos: false`, il testo manterrebbe gli accenti.
    #[test]
    fn flags_desligados_mudam_o_resultado() {
        let sem_acentos = Normalizacao {
            remover_acentos: false,
            ..completa()
        };
        assert_eq!(normalizar("ÁGUA", sem_acentos), "água");

        let sem_minusculas = Normalizacao {
            minusculas: false,
            ..completa()
        };
        assert_eq!(normalizar("AGUA", sem_minusculas), "AGUA");

        let sem_pontuacao = Normalizacao {
            remover_pontuacao: false,
            ..completa()
        };
        assert_eq!(normalizar("COCA-COLA", sem_pontuacao), "coca-cola");

        let sem_compactar = Normalizacao {
            compactar_espacos: false,
            ..completa()
        };
        assert_eq!(normalizar("  A  B ", sem_compactar), "  a  b ");
    }

    /// La normalizzazione deve essere idempotente: normalizzare due volte non
    /// cambia il risultato, altrimenti gli alias compilati non sarebbero stabili.
    #[test]
    fn normalizacao_e_idempotente() {
        let p = completa();
        for entrada in ["ÁGUA SANITÁRIA", "COCA-COLA KS 290ML", "  ", "Rio Sol", ""] {
            let uma = normalizar(entrada, p);
            assert_eq!(normalizar(&uma, p), uma, "falhou para {entrada:?}");
        }
    }
}

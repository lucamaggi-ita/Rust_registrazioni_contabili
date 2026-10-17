//! Aritmetica monetaria condivisa: **centesimi interi**, mai `Decimal` nelle
//! somme e mai `f64`.
//!
//! Questo modulo esiste per il riscontro R3C-02 dell'audit della fase 3C: il
//! pre-libro deve sommare con lo stesso criterio esatto dell'allocatore, e due
//! copie dello stesso algoritmo sarebbero due verita' sul denaro. Il codice e'
//! quello gia' auditato della fase 3B, spostato qui senza cambi di
//! comportamento: gli stessi controlli, gli stessi messaggi, gli stessi limiti.
//!
//! Perche' non `Decimal`: con importi legittimi ma estremi `Decimal` va in
//! panico sul prodotto e **perde un centesimo in silenzio** nella somma,
//! `checked_add` compreso. In centesimi interi il calcolo e' esatto; quando il
//! risultato non e' rappresentabile si restituisce [`ErroDinheiro`], che nomina
//! operazione e valori, invece di un risultato parziale.

use rust_decimal::{Decimal, RoundingStrategy};

/// Violazione aritmetica o di dominio monetario.
///
/// Non porta l'origine del documento: la aggiunge il chiamante, che sa a quale
/// NF-e o a quale riepilogo si riferisce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErroDinheiro {
    /// Il valore non e' un multiplo esatto di 0.01 (contratto C21/D6).
    EscalaInvalida { valor: Decimal },
    /// Il calcolo e' esatto in centesimi ma il risultato non e' rappresentabile.
    CapacidadeExcedida {
        operacao: &'static str,
        detalhe: String,
    },
}

/// Arrotondamento monetario a 2 decimali, `ROUND_HALF_UP`.
///
/// `Decimal::round_dp` usa "Bankers Rounding": usarlo divergerebbe dal Python,
/// che quantizza con `ROUND_HALF_UP`.
pub(crate) fn centavos(valor: Decimal) -> Decimal {
    valor.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Un importo e' nel dominio se il suo **valore** e' un multiplo esatto di
/// 0.01. `10.0000` passa, `0.005` no: gli zeri finali non contano.
pub(crate) fn no_dominio(valor: Decimal) -> bool {
    valor.normalize().scale() <= 2
}

fn capacidade(operacao: &'static str, detalhe: impl Into<String>) -> ErroDinheiro {
    ErroDinheiro::CapacidadeExcedida {
        operacao,
        detalhe: detalhe.into(),
    }
}

/// Converte un importo nel suo numero **esatto** di centesimi.
///
/// Richiede che il valore sia nel dominio. La conversione passa da mantissa e
/// scala, senza moltiplicare in `Decimal`: moltiplicare per 100 un importo
/// vicino al massimo rappresentabile andrebbe in overflow.
///
/// `campo` compare solo nel messaggio d'errore: serve a dire *quale* importo
/// era, non cambia il calcolo.
pub(crate) fn em_centesimos(valor: Decimal, campo: &str) -> Result<i128, ErroDinheiro> {
    if !no_dominio(valor) {
        return Err(ErroDinheiro::EscalaInvalida { valor });
    }
    let normalizado = valor.normalize();
    let fator = 10i128.pow(2 - normalizado.scale());
    normalizado
        .mantissa()
        .checked_mul(fator)
        .ok_or_else(|| capacidade("conversao em centavos", format!("{campo} = {valor}")))
}

/// Riporta i centesimi a `Decimal`, o dichiara l'incapacita'.
///
/// Si prova prima la scala 2; se la mantissa non ci sta, si **riducono gli
/// zeri finali** abbassando la scala, fino a 0: un importo intero grande e'
/// rappresentabile a scala 0 anche quando `mantissa x 100` non ci sta piu'.
/// Nessuna cifra diversa da zero viene mai eliminata e non si arrotonda.
pub(crate) fn de_centesimos(
    centesimos: i128,
    operacao: &'static str,
) -> Result<Decimal, ErroDinheiro> {
    let mut mantissa = centesimos;
    let mut escala: u32 = 2;
    loop {
        if let Ok(valor) = Decimal::try_from_i128_with_scale(mantissa, escala) {
            return Ok(valor);
        }
        // Solo zeri finali: il valore resta identico.
        if escala == 0 || mantissa % 10 != 0 {
            break;
        }
        mantissa /= 10;
        escala -= 1;
    }
    Err(capacidade(
        operacao,
        format!(
            "{centesimos} centavos nao sao representaveis em Decimal nem reduzindo \
             zeros finais, e nenhum centavo pode ser descartado"
        ),
    ))
}

/// Somma esatta di centesimi, con `checked_add` su ogni addendo.
pub(crate) fn somar_centesimos(
    valores: impl IntoIterator<Item = i128>,
    operacao: &'static str,
) -> Result<i128, ErroDinheiro> {
    let mut acumulado: i128 = 0;
    for valor in valores {
        acumulado = acumulado
            .checked_add(valor)
            .ok_or_else(|| capacidade(operacao, format!("{acumulado} + {valor}")))?;
    }
    Ok(acumulado)
}

/// Divisione intera con arrotondamento `ROUND_HALF_UP` (meta' lontano da zero).
///
/// Il resto decide: nessun passaggio in virgola mobile, nessun raddoppio del
/// numeratore. Divisione e resto passano per `checked_div`/`checked_rem`:
/// `i128::MIN / -1` non e' rappresentabile e con gli operatori ordinari andava
/// in panico.
pub(crate) fn dividir_meio_para_cima(
    numerador: i128,
    denominador: i128,
) -> Result<i128, ErroDinheiro> {
    if denominador == 0 {
        return Err(capacidade(
            "divisao do rateio",
            "denominador zero nao chega a esta funcao",
        ));
    }
    let excedido = || {
        capacidade(
            "divisao do rateio",
            format!("{numerador} / {denominador} nao e' representavel em i128"),
        )
    };
    let quociente = numerador.checked_div(denominador).ok_or_else(excedido)?;
    let resto = numerador.checked_rem(denominador).ok_or_else(excedido)?;
    if resto == 0 {
        return Ok(quociente);
    }
    let dobro_do_resto = resto
        .checked_abs()
        .and_then(|r| r.checked_mul(2))
        .ok_or_else(|| capacidade("divisao do rateio", format!("resto {resto}")))?;
    if dobro_do_resto < denominador.abs() {
        return Ok(quociente);
    }
    let positivo = (numerador >= 0) == (denominador >= 0);
    let ajuste = if positivo { 1 } else { -1 };
    quociente
        .checked_add(ajuste)
        .ok_or_else(|| capacidade("divisao do rateio", format!("{quociente} + {ajuste}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(bruto: &str) -> Decimal {
        Decimal::from_str_exact(bruto).unwrap()
    }

    #[test]
    fn dominio_aceita_zeros_finais_e_recusa_fracao_de_centavo() {
        for bom in ["10", "10.0", "10.00", "10.0000", "-0.01", "0.00"] {
            assert!(no_dominio(dec(bom)), "{bom}");
        }
        for mau in ["0.005", "12.345", "0.001"] {
            assert!(!no_dominio(dec(mau)), "{mau}");
        }
    }

    #[test]
    fn round_trip_exato_inclusive_no_maximo() {
        for bruto in [
            "79228162514264337593543950335",
            "-79228162514264337593543950335",
            "10.00",
            "-0.01",
            "0.00",
        ] {
            let valor = dec(bruto);
            let cent = em_centesimos(valor, "teste").unwrap();
            assert_eq!(de_centesimos(cent, "round-trip").unwrap(), valor, "{bruto}");
        }
    }

    #[test]
    fn soma_que_nao_cabe_e_erro_nomeado() {
        let maximo = em_centesimos(dec("79228162514264337593543950335"), "teste").unwrap();
        let erro = de_centesimos(maximo + 1, "soma de teste").unwrap_err();
        match erro {
            ErroDinheiro::CapacidadeExcedida { operacao, detalhe } => {
                assert_eq!(operacao, "soma de teste");
                assert!(detalhe.contains("nem reduzindo zeros finais"), "{detalhe}");
            }
            outro => panic!("erro inesperado: {outro:?}"),
        }
    }

    #[test]
    fn divisao_com_minimo_nao_entra_em_panico() {
        let erro = dividir_meio_para_cima(i128::MIN, -1).unwrap_err();
        assert!(matches!(
            erro,
            ErroDinheiro::CapacidadeExcedida {
                operacao: "divisao do rateio",
                ..
            }
        ));
    }

    #[test]
    fn arredondamento_half_up_nao_bankers() {
        assert_eq!(centavos(dec("2.345")), dec("2.35"));
        assert_eq!(centavos(dec("-2.345")), dec("-2.35"));
        assert_eq!(
            dec("2.345").round_dp(2),
            dec("2.34"),
            "o default divergeria"
        );
    }

    #[test]
    fn escala_invalida_e_erro_dedicado() {
        let erro = em_centesimos(dec("0.005"), "det[1]/prod/vProd").unwrap_err();
        assert_eq!(
            erro,
            ErroDinheiro::EscalaInvalida {
                valor: dec("0.005")
            }
        );
    }
}

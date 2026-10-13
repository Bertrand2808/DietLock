//! Parsing of user-typed text, with integers only (no floating point).

use crate::error::{ValidationError, check_range};
use crate::intake::{MACRO_G_MAX, WATER_ML_MAX};
use crate::weigh_in::{WEIGHT_TENTH_KG_MAX, WEIGHT_TENTH_KG_MIN};

fn invalid_number(field: &'static str, input: &str) -> ValidationError {
    ValidationError::InvalidNumber {
        field,
        input: input.to_owned(),
    }
}

/// Non-empty ASCII digits, saturating at `u32::MAX` so that an oversized
/// input reaches the range check (S7) instead of being a format error.
fn parse_digits(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(s.bytes().fold(0u32, |acc, b| {
        acc.saturating_mul(10).saturating_add(u32::from(b - b'0'))
    }))
}

/// Parses `digits[(,|.)digit]` into tenths.
/// No sign, no exponent, no surrounding whitespace, at most one decimal.
fn parse_tenths(field: &'static str, s: &str) -> Result<u32, ValidationError> {
    let err = || invalid_number(field, s);
    let (int, dec) = match s.find([',', '.']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let int = parse_digits(int).ok_or_else(err)?;
    let dec = match dec {
        None => 0,
        Some(d) if d.len() == 1 => parse_digits(d).ok_or_else(err)?,
        Some(_) => return Err(err()),
    };
    Ok(int.saturating_mul(10).saturating_add(dec))
}

/// Narrows a value already checked against a `u16` range.
fn narrow(v: u32) -> u16 {
    u16::try_from(v).expect("value was range-checked against a u16 maximum")
}

/// S6, S7: weight typed in kg (`82,6` or `82.6`), returned in tenths of a kg.
pub fn parse_weight_tenth_kg(s: &str) -> Result<u16, ValidationError> {
    let v = parse_tenths("weight_tenth_kg", s)?;
    check_range(
        "weight_tenth_kg",
        v,
        WEIGHT_TENTH_KG_MIN,
        WEIGHT_TENTH_KG_MAX,
    )?;
    Ok(narrow(v))
}

/// S6, S7: macro typed in whole grams.
pub fn parse_macro_g(field: &'static str, s: &str) -> Result<u16, ValidationError> {
    let v = parse_digits(s).ok_or_else(|| invalid_number(field, s))?;
    check_range(field, v, 0, MACRO_G_MAX)?;
    Ok(narrow(v))
}

/// S6, S7: water typed in litres (`1,5`), returned in mL. An empty field means absent.
pub fn parse_water_ml(s: &str) -> Result<Option<u16>, ValidationError> {
    if s.is_empty() {
        return Ok(None);
    }
    let ml = parse_tenths("water_ml", s)?.saturating_mul(100);
    check_range("water_ml", ml, 0, WATER_ML_MAX)?;
    Ok(Some(narrow(ml)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s6_weight_comma_or_dot() {
        assert_eq!(parse_weight_tenth_kg("82,6"), Ok(826));
        assert_eq!(parse_weight_tenth_kg("82.6"), Ok(826));
        assert_eq!(parse_weight_tenth_kg("82"), Ok(820));
        assert_eq!(parse_weight_tenth_kg("250,0"), Ok(2500));
    }

    #[test]
    fn s6_s7_weight_refused() {
        for s in [
            "82,65", "82,", "abc", "-82,6", ",5", "1e2", "99999", "82,6 ", " 82,6", "250,1",
            "29,9", "", "82,6,1", "+82",
        ] {
            assert!(parse_weight_tenth_kg(s).is_err(), "{s:?} should be refused");
        }
    }

    #[test]
    fn s7_weight_range_error_is_typed() {
        assert!(matches!(
            parse_weight_tenth_kg("250,1"),
            Err(ValidationError::OutOfRange { value: 2501, .. })
        ));
        assert!(matches!(
            parse_weight_tenth_kg("abc"),
            Err(ValidationError::InvalidNumber { .. })
        ));
    }

    #[test]
    fn s7_overflowing_input_is_out_of_range_not_invalid() {
        let huge = "9".repeat(40);
        assert!(matches!(
            parse_weight_tenth_kg("99999"),
            Err(ValidationError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_weight_tenth_kg(&huge),
            Err(ValidationError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_macro_g("protein_g", "99999"),
            Err(ValidationError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_macro_g("protein_g", &huge),
            Err(ValidationError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_water_ml("99999"),
            Err(ValidationError::OutOfRange { .. })
        ));
        assert!(matches!(
            parse_water_ml(&huge),
            Err(ValidationError::OutOfRange { .. })
        ));
    }

    #[test]
    fn s6_s7_macros_whole_grams() {
        assert_eq!(parse_macro_g("protein_g", "150"), Ok(150));
        assert_eq!(parse_macro_g("fat_g", "0"), Ok(0));
        for s in ["1001", "12,5", "-1", "", "abc", "99999"] {
            assert!(
                parse_macro_g("carbs_g", s).is_err(),
                "{s:?} should be refused"
            );
        }
    }

    #[test]
    fn s6_s7_water_litres_to_ml() {
        assert_eq!(parse_water_ml("1,5"), Ok(Some(1500)));
        assert_eq!(parse_water_ml("2"), Ok(Some(2000)));
        assert_eq!(parse_water_ml("10.0"), Ok(Some(10_000)));
        assert_eq!(parse_water_ml("0"), Ok(Some(0)));
        assert_eq!(parse_water_ml(""), Ok(None));
        for s in ["10,1", "1,55", "999", "-1", " ", "1,5 "] {
            assert!(parse_water_ml(s).is_err(), "{s:?} should be refused");
        }
    }
}

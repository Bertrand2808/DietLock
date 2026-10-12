//! Pure business logic of DietLock: no dependency on Tauri, SQLite or the clock.

use std::fmt;

use time::{Date, Month};

/// Application name.
pub fn app_name() -> &'static str {
    "DietLock"
}

/// Minimum accepted weight, in tenths of a kg (30.0 kg).
pub const WEIGHT_TENTH_KG_MIN: u16 = 300;
/// Maximum accepted weight, in tenths of a kg (250.0 kg).
pub const WEIGHT_TENTH_KG_MAX: u16 = 2500;
/// Maximum value of a macro, in grams.
pub const MACRO_G_MAX: u16 = 1000;
/// Maximum water, in millilitres.
pub const WATER_ML_MAX: u16 = 10_000;

/// Input validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Date not in the calendar, or not in the strict `YYYY-MM-DD` format.
    InvalidDate(String),
    /// Value outside its allowed range.
    OutOfRange {
        field: &'static str,
        value: i64,
        min: i64,
        max: i64,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDate(s) => write!(f, "invalid date: {s:?} (expected YYYY-MM-DD)"),
            Self::OutOfRange {
                field,
                value,
                min,
                max,
            } => write!(f, "{field} = {value} out of range [{min}, {max}]"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Calendar day, without time or time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day(Date);

impl Day {
    /// Parses a date in the strict `YYYY-MM-DD` format (4-digit year, zero-padded).
    pub fn parse(s: &str) -> Result<Self, ValidationError> {
        let err = || ValidationError::InvalidDate(s.to_owned());
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(err());
        }
        let num = |r: std::ops::Range<usize>| -> Result<u32, ValidationError> {
            let part = &b[r];
            if !part.iter().all(u8::is_ascii_digit) {
                return Err(err());
            }
            Ok(part.iter().fold(0, |acc, d| acc * 10 + u32::from(d - b'0')))
        };
        let year = num(0..4)?;
        let month = num(5..7)?;
        let day = num(8..10)?;
        if year < 1000 {
            return Err(err());
        }
        let month = u8::try_from(month)
            .ok()
            .and_then(|m| Month::try_from(m).ok())
            .ok_or_else(err)?;
        let day = u8::try_from(day).map_err(|_| err())?;
        let year = i32::try_from(year).map_err(|_| err())?;
        Date::from_calendar_date(year, month, day)
            .map(Self)
            .map_err(|_| err())
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.0.year(),
            u8::from(self.0.month()),
            self.0.day()
        )
    }
}

fn check_range(field: &'static str, value: u16, min: u16, max: u16) -> Result<(), ValidationError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::OutOfRange {
            field,
            value: value.into(),
            min: min.into(),
            max: max.into(),
        })
    }
}

/// Weigh-in of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeighIn {
    date: Day,
    weight_tenth_kg: u16,
    fasted: bool,
}

impl WeighIn {
    pub fn new(date: Day, weight_tenth_kg: u16, fasted: bool) -> Result<Self, ValidationError> {
        check_range(
            "weight_tenth_kg",
            weight_tenth_kg,
            WEIGHT_TENTH_KG_MIN,
            WEIGHT_TENTH_KG_MAX,
        )?;
        Ok(Self {
            date,
            weight_tenth_kg,
            fasted,
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn weight_tenth_kg(&self) -> u16 {
        self.weight_tenth_kg
    }

    pub fn fasted(&self) -> bool {
        self.fasted
    }
}

/// Intake of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intake {
    date: Day,
    protein_g: u16,
    carbs_g: u16,
    fat_g: u16,
    water_ml: Option<u16>,
}

impl Intake {
    pub fn new(
        date: Day,
        protein_g: u16,
        carbs_g: u16,
        fat_g: u16,
        water_ml: Option<u16>,
    ) -> Result<Self, ValidationError> {
        check_range("protein_g", protein_g, 0, MACRO_G_MAX)?;
        check_range("carbs_g", carbs_g, 0, MACRO_G_MAX)?;
        check_range("fat_g", fat_g, 0, MACRO_G_MAX)?;
        if let Some(w) = water_ml {
            check_range("water_ml", w, 0, WATER_ML_MAX)?;
        }
        Ok(Self {
            date,
            protein_g,
            carbs_g,
            fat_g,
            water_ml,
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn protein_g(&self) -> u16 {
        self.protein_g
    }

    pub fn carbs_g(&self) -> u16 {
        self.carbs_g
    }

    pub fn fat_g(&self) -> u16 {
        self.fat_g
    }

    pub fn water_ml(&self) -> Option<u16> {
        self.water_ml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    #[test]
    fn app_name_is_dietlock() {
        assert_eq!(app_name(), "DietLock");
    }

    #[test]
    fn leap_day_2028_is_accepted() {
        assert_eq!(day("2028-02-29").to_string(), "2028-02-29");
    }

    #[test]
    fn invalid_dates_are_refused() {
        for s in [
            "2027-02-29",
            "2026-04-31",
            "2026-10-8",
            "2026-13-01",
            "2026-00-10",
            "2026-10-00",
            "2026/10/08",
            "+026-10-08",
            "2026-10-08 ",
            "",
        ] {
            assert!(Day::parse(s).is_err(), "{s:?} should be refused");
        }
    }

    #[test]
    fn weight_bounds() {
        let d = day("2026-10-12");
        assert!(WeighIn::new(d, 300, true).is_ok());
        assert!(WeighIn::new(d, 2500, false).is_ok());
        assert!(WeighIn::new(d, 299, true).is_err());
        assert!(WeighIn::new(d, 2501, true).is_err());
    }

    #[test]
    fn intake_bounds() {
        let d = day("2026-10-12");
        assert!(Intake::new(d, 0, 1000, 0, Some(10_000)).is_ok());
        assert!(Intake::new(d, 1001, 0, 0, None).is_err());
        assert!(Intake::new(d, 0, 1001, 0, None).is_err());
        assert!(Intake::new(d, 0, 0, 1001, None).is_err());
        assert!(Intake::new(d, 0, 0, 0, Some(10_001)).is_err());
    }
}

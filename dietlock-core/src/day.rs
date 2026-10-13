use std::fmt;

use time::{Date, Month};

use crate::error::ValidationError;

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

    /// The day before, or `None` at the lower bound of the calendar.
    pub fn previous(self) -> Option<Self> {
        self.0.previous_day().map(Self)
    }

    /// The day `days` calendar days earlier, or `None` below the calendar bound.
    pub fn minus_days(self, days: u16) -> Option<Self> {
        self.0
            .checked_sub(time::Duration::days(i64::from(days)))
            .map(Self)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

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
}

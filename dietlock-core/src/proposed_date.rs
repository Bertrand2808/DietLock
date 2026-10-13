//! Date proposed for a new entry. The clock is never read here:
//! the current day and time are passed in by the caller.

use crate::day::Day;
use time::Time;

use crate::error::{ValidationError, check_range};

/// Hour from which D8 applies (inclusive) and before which the intake goes to J-1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolloverHour(u8);

impl RolloverHour {
    pub const MAX: u8 = 11;

    pub fn new(hour: u8) -> Result<Self, ValidationError> {
        check_range("rollover_hour", hour.into(), 0, Self::MAX.into())?;
        Ok(Self(hour))
    }

    pub fn hour(self) -> u8 {
        self.0
    }
}

impl Default for RolloverHour {
    fn default() -> Self {
        Self(4)
    }
}

/// D8 window ends strictly before this hour.
const NOON: u8 = 12;

/// D1: a weigh-in is proposed on the current day.
pub fn proposed_weigh_in_date(today: Day) -> Day {
    today
}

/// D2-D3 and D8: date proposed for an intake.
///
/// `now` is the current local time; a `Time` is always valid, so an
/// impossible hour such as 24:00 cannot reach this function.
pub fn proposed_intake_date(
    today: Day,
    now: Time,
    rollover: RolloverHour,
    yesterday_has_intake: bool,
    today_has_intake: bool,
) -> Day {
    let yesterday = today.previous().unwrap_or(today);
    let hour = now.hour();
    let before_rollover = hour < rollover.hour();
    let d8_applies = hour < NOON && !yesterday_has_intake && !today_has_intake;
    if before_rollover || d8_applies {
        yesterday
    } else {
        today
    }
}

/// D5: refuses a date later than the current day.
pub fn check_not_future(date: Day, today: Day) -> Result<Day, ValidationError> {
    if date > today {
        Err(ValidationError::FutureDate(date.to_string()))
    } else {
        Ok(date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    const TUESDAY: &str = "2026-10-13";
    const MONDAY: &str = "2026-10-12";

    fn time(h: u8, m: u8) -> Time {
        Time::from_hms(h, m, 0).unwrap()
    }

    fn at(h: u8, m: u8, mon: bool, tue: bool) -> Day {
        proposed_intake_date(day(TUESDAY), time(h, m), RolloverHour::default(), mon, tue)
    }

    #[test]
    fn weigh_in_is_today() {
        assert_eq!(proposed_weigh_in_date(day(TUESDAY)), day(TUESDAY));
    }

    #[test]
    fn before_rollover_is_yesterday() {
        assert_eq!(at(3, 59, false, false), day(MONDAY));
        assert_eq!(at(3, 59, true, true), day(MONDAY));
        assert_eq!(at(0, 0, true, false), day(MONDAY));
    }

    #[test]
    fn d8_window() {
        assert_eq!(at(4, 0, false, false), day(MONDAY));
        assert_eq!(at(7, 0, false, false), day(MONDAY));
        assert_eq!(at(7, 0, true, false), day(TUESDAY));
        assert_eq!(at(7, 0, false, true), day(TUESDAY));
        assert_eq!(at(11, 59, false, false), day(MONDAY));
    }

    #[test]
    fn from_noon_is_today() {
        assert_eq!(at(12, 0, false, false), day(TUESDAY));
        assert_eq!(at(12, 1, false, false), day(TUESDAY));
        assert_eq!(at(23, 59, false, false), day(TUESDAY));
    }

    #[test]
    fn impossible_time_cannot_be_built() {
        assert!(Time::from_hms(24, 0, 0).is_err());
        assert!(Time::from_hms(99, 99, 0).is_err());
        assert!(Time::from_hms(12, 60, 0).is_err());
    }

    #[test]
    fn rollover_range() {
        assert_eq!(RolloverHour::default().hour(), 4);
        assert!(RolloverHour::new(0).is_ok());
        assert!(RolloverHour::new(11).is_ok());
        assert!(RolloverHour::new(12).is_err());
    }

    #[test]
    fn rollover_zero_goes_straight_to_d8() {
        let r = RolloverHour::new(0).unwrap();
        let p = proposed_intake_date(day(TUESDAY), time(0, 0), r, false, false);
        assert_eq!(p, day(MONDAY));
    }

    #[test]
    fn future_date_refused() {
        let today = day(TUESDAY);
        assert!(check_not_future(day("2026-10-14"), today).is_err());
        assert_eq!(check_not_future(today, today), Ok(today));
        assert_eq!(check_not_future(day(MONDAY), today), Ok(day(MONDAY)));
    }
}

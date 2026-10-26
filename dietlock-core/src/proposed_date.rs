//! Date proposed for a new entry. The clock is never read here:
//! the current day and time are passed in by the caller.

use crate::day::Day;
use crate::error::{ValidationError, check_range};
use time::Time;

/// Hour before which an intake is attributed to the previous day (D2-D3).
/// The morning D8 window opens at this hour. The evening proposal does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchHour(u8);

impl SwitchHour {
    pub const MAX: u8 = 11;

    pub fn new(hour: u8) -> Result<Self, ValidationError> {
        check_range("switch_hour", hour, 0, Self::MAX)?;
        Ok(Self(hour))
    }

    pub fn hour(self) -> u8 {
        self.0
    }
}

impl Default for SwitchHour {
    fn default() -> Self {
        Self(4)
    }
}

/// The morning D8 window ends strictly before this hour.
const D8_END_HOUR: u8 = 12;

/// From this hour inclusive, D8 proposes yesterday again when it has no intake.
const EVENING_HOUR: u8 = 21;

/// Which of the two days around the current one already hold an intake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntakePresence {
    pub yesterday: bool,
    pub today: bool,
}

/// Date proposed for an intake, and whether D8 chose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProposedIntake {
    pub date: Day,
    /// D8: yesterday was proposed because it was forgotten; the caller
    /// highlights the day's name.
    pub forgotten_yesterday: bool,
}

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
    switch_hour: SwitchHour,
    presence: IntakePresence,
) -> ProposedIntake {
    // At the lower bound of the calendar there is no previous day: keep today.
    let yesterday = today.previous().unwrap_or(today);
    let hour = now.hour();
    if hour < switch_hour.hour() {
        return ProposedIntake {
            date: yesterday,
            forgotten_yesterday: false,
        };
    }
    let forgotten_yesterday = proposes_forgotten_yesterday(hour, presence);
    ProposedIntake {
        date: if forgotten_yesterday {
            yesterday
        } else {
            today
        },
        forgotten_yesterday,
    }
}

/// Morning D8 needs both days empty. From [`EVENING_HOUR`], only yesterday matters:
/// a filled today does not cancel a missing yesterday.
fn proposes_forgotten_yesterday(hour: u8, presence: IntakePresence) -> bool {
    let morning = hour < D8_END_HOUR && !presence.yesterday && !presence.today;
    let evening = hour >= EVENING_HOUR && !presence.yesterday;
    morning || evening
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

    fn proposal(h: u8, m: u8, monday: bool, tuesday: bool) -> ProposedIntake {
        proposed_intake_date(
            day(TUESDAY),
            time(h, m),
            SwitchHour::default(),
            IntakePresence {
                yesterday: monday,
                today: tuesday,
            },
        )
    }

    fn at(h: u8, m: u8, monday: bool, tuesday: bool) -> Day {
        proposal(h, m, monday, tuesday).date
    }

    #[test]
    fn d1_weigh_in_is_today() {
        assert_eq!(proposed_weigh_in_date(day(TUESDAY)), day(TUESDAY));
    }

    #[test]
    fn d2_before_switch_hour_is_yesterday() {
        assert_eq!(at(3, 59, false, false), day(MONDAY));
        assert_eq!(at(3, 59, true, true), day(MONDAY));
        assert_eq!(at(0, 0, true, false), day(MONDAY));
    }

    #[test]
    fn d2_before_switch_hour_is_not_flagged_as_forgotten() {
        assert!(!proposal(3, 59, false, false).forgotten_yesterday);
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
    fn d8_flags_forgotten_yesterday() {
        assert!(proposal(7, 0, false, false).forgotten_yesterday);
        assert!(!proposal(7, 0, true, false).forgotten_yesterday);
        assert!(!proposal(7, 0, false, true).forgotten_yesterday);
        assert!(!proposal(12, 0, false, false).forgotten_yesterday);
    }

    #[test]
    fn d2_from_noon_until_evening_is_today() {
        assert_eq!(at(12, 0, false, false), day(TUESDAY));
        assert_eq!(at(12, 1, false, false), day(TUESDAY));
        assert_eq!(at(20, 59, false, false), day(TUESDAY));
        assert!(!proposal(20, 59, false, false).forgotten_yesterday);
    }

    #[test]
    fn d8_at_21_empty_yesterday_proposes_yesterday() {
        let empty_today = proposal(21, 0, false, false);
        assert_eq!(empty_today.date, day(MONDAY));
        assert!(empty_today.forgotten_yesterday);
        // A filled today does not cancel a missing yesterday.
        let filled_today = proposal(21, 0, false, true);
        assert_eq!(filled_today.date, day(MONDAY));
        assert!(filled_today.forgotten_yesterday);
        assert_eq!(at(23, 59, false, false), day(MONDAY));
    }

    #[test]
    fn d8_at_21_filled_yesterday_proposes_today() {
        let empty_today = proposal(21, 0, true, false);
        assert_eq!(empty_today.date, day(TUESDAY));
        assert!(!empty_today.forgotten_yesterday);
        let filled_today = proposal(21, 0, true, true);
        assert_eq!(filled_today.date, day(TUESDAY));
        assert!(!filled_today.forgotten_yesterday);
    }

    #[test]
    fn d3_switch_hour_does_not_move_the_evening() {
        let switch_hour = SwitchHour::new(11).unwrap();
        let p = proposed_intake_date(
            day(TUESDAY),
            time(21, 0),
            switch_hour,
            IntakePresence {
                yesterday: false,
                today: false,
            },
        );
        assert_eq!(p.date, day(MONDAY));
        assert!(p.forgotten_yesterday);
    }

    #[test]
    fn d3_switch_hour_range() {
        assert_eq!(SwitchHour::default().hour(), 4);
        assert!(SwitchHour::new(0).is_ok());
        assert!(SwitchHour::new(11).is_ok());
        assert!(SwitchHour::new(12).is_err());
    }

    #[test]
    fn d3_switch_hour_zero_goes_straight_to_d8() {
        let r = SwitchHour::new(0).unwrap();
        let presence = IntakePresence {
            yesterday: false,
            today: false,
        };
        let p = proposed_intake_date(day(TUESDAY), time(0, 0), r, presence);
        assert_eq!(p.date, day(MONDAY));
        assert!(p.forgotten_yesterday);
    }

    #[test]
    fn d5_future_date_refused() {
        let today = day(TUESDAY);
        assert!(check_not_future(day("2026-10-14"), today).is_err());
        assert_eq!(check_not_future(today, today), Ok(today));
        assert_eq!(check_not_future(day(MONDAY), today), Ok(day(MONDAY)));
    }
}

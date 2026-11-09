//! Mean energy of the week ending yesterday, and the gap with the week before.
//!
//! Computed on demand and never stored. One rounding rule, the same as the
//! weight trend: a half rounds away from zero. The gap is the difference of
//! the two displayed means, so the figures on the screen add up.
//!
//! Absence is not zero. No intake in the window is `Ok(None)`, not a mean of
//! 0 kcal. No intake in the previous week is a mean whose gap is `None`, not
//! a gap of 0.

use std::collections::HashSet;
use std::fmt;

use crate::day::Day;
use crate::energy::energy_kcal;
use crate::intake::Intake;

/// Calendar days in the window ending yesterday (J−7 to J−1). E1.
pub const ENERGY_WEEK_DAYS: u16 = 7;

/// Two intakes share a date, which S2 forbids: the mean would count the day twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateIntakeDate;

impl fmt::Display for DuplicateIntakeDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "two intakes share the same date")
    }
}

impl std::error::Error for DuplicateIntakeDate {}

/// Integer division rounding a half away from zero. `den` must be positive.
fn div_round_half_away(num: i64, den: i64) -> i64 {
    debug_assert!(den > 0);
    let q = (num.abs() * 2 + den) / (den * 2);
    if num < 0 { -q } else { q }
}

/// Mean of the energies actually entered in one window. E2, E3, E5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnergyMean {
    kcal: u32,
    days: u32,
}

impl EnergyMean {
    /// Whole kcal, half away from zero.
    pub fn kcal(self) -> u32 {
        self.kcal
    }

    /// Intakes inside the window. A day at 0 kcal counts. A day without an intake does not.
    pub fn days(self) -> u32 {
        self.days
    }
}

/// The week ending yesterday. The gap is absent when the previous week has no intake. E4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnergyWeek {
    mean: EnergyMean,
    gap_kcal: Option<i32>,
}

impl EnergyWeek {
    pub fn mean(self) -> EnergyMean {
        self.mean
    }

    /// Signed whole kcal: this week's displayed mean minus the previous week's.
    /// `None` means the previous week has no intake. `Some(0)` is a real tie.
    pub fn gap_kcal(self) -> Option<i32> {
        self.gap_kcal
    }
}

/// Intakes whose date falls in `end` and the [`ENERGY_WEEK_DAYS`]−1 days before it.
fn mean_ending(intakes: &[Intake], end: Day) -> Option<EnergyMean> {
    let from = end.minus_days(ENERGY_WEEK_DAYS - 1)?;
    let mut sum = 0u32;
    let mut days = 0u32;
    for intake in intakes {
        if intake.date() >= from && intake.date() <= end {
            sum += energy_kcal(intake);
            days += 1;
        }
    }
    if days == 0 {
        return None;
    }
    let kcal = div_round_half_away(i64::from(sum), i64::from(days));
    let kcal = u32::try_from(kcal).ok()?;
    Some(EnergyMean { kcal, days })
}

/// Energy of the 7 days ending yesterday, compared with the 7 days before that.
///
/// `Ok(None)` when yesterday's window has no intake (E3): that is not 0 kcal.
/// `Err` if any date appears twice, in or out of the windows (S2).
/// The input can be in any order. Today is outside both windows (E1).
pub fn energy_week(
    intakes: &[Intake],
    today: Day,
) -> Result<Option<EnergyWeek>, DuplicateIntakeDate> {
    let mut seen = HashSet::new();
    if !intakes.iter().all(|i| seen.insert(i.date())) {
        return Err(DuplicateIntakeDate);
    }
    let Some(yesterday) = today.previous() else {
        return Ok(None);
    };
    let Some(mean) = mean_ending(intakes, yesterday) else {
        return Ok(None);
    };
    let gap_kcal = yesterday
        .minus_days(ENERGY_WEEK_DAYS)
        .and_then(|previous_end| {
            let past = mean_ending(intakes, previous_end)?;
            let now = i32::try_from(mean.kcal).ok()?;
            let before = i32::try_from(past.kcal).ok()?;
            Some(now - before)
        });
    Ok(Some(EnergyWeek { mean, gap_kcal }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    /// Macros are tenths of a gram. Protein alone yields `grams * 4` kcal.
    fn protein(date: &str, grams: u16) -> Intake {
        Intake::new(day(date), grams * 10, 0, 0, None).unwrap()
    }

    fn fat_tenths(date: &str, tenths: u16) -> Intake {
        Intake::new(day(date), 0, 0, tenths, None).unwrap()
    }

    fn zero(date: &str) -> Intake {
        Intake::new(day(date), 0, 0, 0, None).unwrap()
    }

    #[test]
    fn e3_no_intake_is_not_a_mean_of_zero() {
        assert_eq!(energy_week(&[], day("2026-11-09")), Ok(None));
        // Today is outside the window, so it does not create a mean.
        let today = protein("2026-11-09", 100);
        assert_eq!(energy_week(&[today], day("2026-11-09")), Ok(None));
    }

    #[test]
    fn e2_zero_kcal_counts_and_a_missing_day_does_not() {
        // Window 11-02..=11-08. 11-05 and 11-08 have no intake. 11-03 is 0 kcal.
        let intakes = [
            protein("2026-11-02", 100), // 400
            zero("2026-11-03"),
            protein("2026-11-04", 50), // 200
        ];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().days(), 3);
        assert_eq!(week.mean().kcal(), 200); // (400 + 0 + 200) / 3 = 200
        assert_eq!(week.gap_kcal(), None);
    }

    #[test]
    fn e1_window_is_the_seven_days_ending_yesterday() {
        // J−7 = 11-02 is inside. J−8 = 11-01 belongs to the previous week.
        // J−15 = 10-25 is outside both. Today = 11-09 is outside both.
        let current = protein("2026-11-02", 300); // 1200
        let previous_end = protein("2026-11-01", 250); // 1000
        let previous_start = protein("2026-10-26", 200); // 800
        let too_old = protein("2026-10-25", 1000); // 4000, must not move the mean
        let today = protein("2026-11-09", 1000);
        let intakes = [today, too_old, previous_start, current, previous_end];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 1200);
        assert_eq!(week.mean().days(), 1);
        assert_eq!(week.gap_kcal(), Some(1200 - 900));
    }

    #[test]
    fn e1_an_intake_on_the_previous_week_only_is_still_no_mean() {
        let previous = protein("2026-11-01", 250);
        assert_eq!(energy_week(&[previous], day("2026-11-09")), Ok(None));
    }

    #[test]
    fn the_screen_fixture_for_9_november_is_plus_120() {
        // Same rows the UI proof stores. Window 11-02..=11-08, previous week 10-26..=11-01.
        let intakes = [
            Intake::new(day("2026-10-26"), 1500, 1500, 0, None).unwrap(),
            Intake::new(day("2026-10-28"), 1500, 1500, 0, None).unwrap(),
            Intake::new(day("2026-10-30"), 1500, 1500, 0, None).unwrap(),
            Intake::new(day("2026-11-02"), 1500, 1500, 0, None).unwrap(),
            Intake::new(day("2026-11-03"), 1500, 2000, 0, None).unwrap(),
            Intake::new(day("2026-11-04"), 2000, 2000, 0, None).unwrap(),
            Intake::new(day("2026-11-06"), 0, 0, 0, None).unwrap(),
            Intake::new(day("2026-11-07"), 2000, 3100, 400, None).unwrap(),
            Intake::new(day("2026-11-09"), 1000, 0, 0, None).unwrap(),
        ];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 1320);
        assert_eq!(week.mean().days(), 5);
        assert_eq!(week.gap_kcal(), Some(120));
    }

    #[test]
    fn e4_no_previous_intake_is_not_a_gap_of_zero() {
        let current = protein("2026-11-08", 100);
        let week = energy_week(&[current], day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 400);
        assert_eq!(week.gap_kcal(), None);
    }

    #[test]
    fn e4_a_real_tie_is_a_gap_of_zero() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-01", 100)];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.gap_kcal(), Some(0));
    }

    #[test]
    fn e5_negative_gap() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-01", 200)];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 400);
        assert_eq!(week.gap_kcal(), Some(-400));
    }

    #[test]
    fn e5_half_rounds_away_from_zero() {
        // 0.1 g of fat is 1 kcal, 0.2 g is 2 kcal (S8). (1 + 2) / 2 = 1.5 → 2.
        let intakes = [fat_tenths("2026-11-07", 1), fat_tenths("2026-11-08", 2)];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 2);
        assert_eq!(week.mean().days(), 2);
    }

    #[test]
    fn e5_gap_uses_the_displayed_means() {
        // Current (1 + 2) / 2 = 1.5 → 2. Previous 2 and 2 → 2.
        // Displayed gap is 0. The raw difference 1.5 − 2 = −0.5 would round to −1.
        let intakes = [
            fat_tenths("2026-11-07", 1),
            fat_tenths("2026-11-08", 2),
            fat_tenths("2026-10-31", 2),
            fat_tenths("2026-11-01", 2),
        ];
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().kcal(), 2);
        assert_eq!(week.gap_kcal(), Some(0));
    }

    #[test]
    fn order_does_not_matter() {
        let mut intakes = [
            protein("2026-11-02", 100),
            protein("2026-11-08", 50),
            protein("2026-10-26", 80),
            protein("2026-01-01", 10),
        ];
        let expected = energy_week(&intakes, day("2026-11-09"));
        intakes.reverse();
        assert_eq!(energy_week(&intakes, day("2026-11-09")), expected);
    }

    #[test]
    fn max_energy_does_not_overflow() {
        let intakes: Vec<_> = (2..=8)
            .map(|day_number| {
                Intake::new(
                    day(&format!("2026-11-{day_number:02}")),
                    10_000,
                    10_000,
                    10_000,
                    None,
                )
                .unwrap()
            })
            .collect();
        let week = energy_week(&intakes, day("2026-11-09")).unwrap().unwrap();
        assert_eq!(week.mean().days(), 7);
        assert_eq!(week.mean().kcal(), 17_000);
    }

    #[test]
    fn duplicate_date_in_the_window_is_an_error() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-08", 50)];
        assert_eq!(
            energy_week(&intakes, day("2026-11-09")),
            Err(DuplicateIntakeDate)
        );
    }

    #[test]
    fn duplicate_date_outside_the_windows_is_an_error() {
        let intakes = [
            protein("2026-11-08", 100),
            protein("2026-09-01", 10),
            protein("2026-09-01", 20),
        ];
        assert_eq!(
            energy_week(&intakes, day("2026-11-09")),
            Err(DuplicateIntakeDate)
        );
    }
}

//! Mean protein of the week ending yesterday, and the gap with the week before.
//!
//! Same window and same absence rules as the energy mean (E1–E5), applied to
//! the protein of each intake. Computed on demand and never stored.

use std::collections::HashSet;

use crate::day::Day;
use crate::energy_week::{DuplicateIntakeDate, ENERGY_WEEK_DAYS, div_round_half_away};
use crate::intake::Intake;

const TENTHS_PER_GRAM: i64 = 10;

fn reject_duplicate_dates(intakes: &[Intake]) -> Result<(), DuplicateIntakeDate> {
    let mut seen = HashSet::new();
    if intakes.iter().all(|intake| seen.insert(intake.date())) {
        Ok(())
    } else {
        Err(DuplicateIntakeDate)
    }
}

// Returns the whole-gram mean and the number of intakes counted.
fn protein_mean_ending(intakes: &[Intake], end: Day) -> Option<(u32, u32)> {
    let from = end.minus_days(ENERGY_WEEK_DAYS - 1)?;
    let (sum_tenths, days) = intakes
        .iter()
        .filter(|intake| intake.date() >= from && intake.date() <= end)
        .fold((0i64, 0i64), |(sum, days), intake| {
            (sum + i64::from(intake.protein_tenth_g()), days + 1)
        });
    if days == 0 {
        return None;
    }
    let mean_g = div_round_half_away(sum_tenths, days * TENTHS_PER_GRAM);
    Some((u32::try_from(mean_g).ok()?, u32::try_from(days).ok()?))
}

/// Mean of one macro over the intakes of a window, in whole grams. M1–M5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroWeek {
    mean_g: u32,
    days: u32,
    gap_g: Option<i32>,
}

impl MacroWeek {
    /// Whole grams per entered day, half away from zero.
    pub fn mean_g(self) -> u32 {
        self.mean_g
    }

    /// Intakes inside the window. A day at 0 g counts. A day without an intake does not.
    pub fn days(self) -> u32 {
        self.days
    }

    /// Signed whole grams: this week's displayed mean minus the previous week's.
    /// `None` means the previous week has no intake. `Some(0)` is a real tie.
    pub fn gap_g(self) -> Option<i32> {
        self.gap_g
    }
}

/// Protein of one day of the window. `None` is a day without intake: a hole, not 0 g.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroDay {
    date: Day,
    protein_tenth_g: Option<u16>,
}

impl MacroDay {
    pub fn date(self) -> Day {
        self.date
    }

    /// Tenths of a gram. `Some(0)` is a day entered at 0 g.
    pub fn protein_tenth_g(self) -> Option<u16> {
        self.protein_tenth_g
    }
}

/// Protein of each of the 7 days ending yesterday, oldest first. M7, M8.
///
/// # Errors
///
/// `Err` if any date appears twice (S2).
pub fn protein_days(intakes: &[Intake], today: Day) -> Result<Vec<MacroDay>, DuplicateIntakeDate> {
    reject_duplicate_dates(intakes)?;
    let Some(yesterday) = today.previous() else {
        return Ok(Vec::new());
    };
    let days = (0..ENERGY_WEEK_DAYS)
        .rev()
        .filter_map(|back| yesterday.minus_days(back))
        .map(|date| MacroDay {
            date,
            protein_tenth_g: intakes
                .iter()
                .find(|intake| intake.date() == date)
                .map(Intake::protein_tenth_g),
        })
        .collect();
    Ok(days)
}

/// Protein of the 7 days ending yesterday, compared with the 7 days before that.
///
/// `Ok(None)` when yesterday's window has no intake: that is not 0 g.
///
/// # Errors
///
/// `Err` if any date appears twice, in or out of the windows (S2).
pub fn protein_week(
    intakes: &[Intake],
    today: Day,
) -> Result<Option<MacroWeek>, DuplicateIntakeDate> {
    reject_duplicate_dates(intakes)?;
    let Some(yesterday) = today.previous() else {
        return Ok(None);
    };
    let Some((mean_g, days)) = protein_mean_ending(intakes, yesterday) else {
        return Ok(None);
    };
    let gap_g = yesterday
        .minus_days(ENERGY_WEEK_DAYS)
        .and_then(|previous_end| protein_mean_ending(intakes, previous_end))
        .and_then(|(previous_mean_g, _)| {
            Some(i32::try_from(mean_g).ok()? - i32::try_from(previous_mean_g).ok()?)
        });
    Ok(Some(MacroWeek {
        mean_g,
        days,
        gap_g,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    /// Protein in tenths of a gram, no carbs and no fat.
    fn protein_tenths(date: &str, tenths: u16) -> Intake {
        Intake::new(day(date), tenths, 0, 0, None).unwrap()
    }

    fn protein(date: &str, grams: u16) -> Intake {
        protein_tenths(date, grams * 10)
    }

    const TODAY: &str = "2026-11-09";

    fn week(intakes: &[Intake]) -> Option<MacroWeek> {
        protein_week(intakes, day(TODAY)).unwrap()
    }

    fn days_of(intakes: &[Intake]) -> Vec<(String, Option<u16>)> {
        protein_days(intakes, day(TODAY))
            .unwrap()
            .into_iter()
            .map(|d| (d.date().to_string(), d.protein_tenth_g()))
            .collect()
    }

    const FIFTEEN: MacroPeriod = MacroPeriod::FifteenDays;

    #[test]
    fn m12_period_lengths_are_seven_and_fifteen_days() {
        assert_eq!(MacroPeriod::SevenDays.days(), 7);
        assert_eq!(FIFTEEN.days(), 15);
    }

    #[test]
    fn m12_fifteen_day_list_runs_from_fifteen_days_before_today_to_yesterday() {
        let listed = protein_days(&[], day(TODAY), FIFTEEN).unwrap();
        assert_eq!(listed.len(), 15);
        assert_eq!(listed[0].date().to_string(), "2026-10-25");
        assert_eq!(listed[14].date().to_string(), "2026-11-08");
    }

    #[test]
    fn m12_fifteen_day_list_keeps_holes_and_zero_days() {
        let intakes = [protein("2026-10-30", 0), protein("2026-11-08", 90)];
        let listed = protein_days(&intakes, day(TODAY), FIFTEEN).unwrap();
        assert_eq!(listed[5].protein_tenth_g(), Some(0));
        assert_eq!(listed[14].protein_tenth_g(), Some(900));
        assert_eq!(listed[6].protein_tenth_g(), None);
    }

    #[test]
    fn m12_an_intake_eight_days_back_counts_in_fifteen_days_only() {
        let intakes = [protein("2026-11-01", 100)];
        assert_eq!(week(&intakes), None);
        let fifteen = protein_mean(&intakes, day(TODAY), FIFTEEN)
            .unwrap()
            .unwrap();
        assert_eq!(fifteen.mean_g(), 100);
        assert_eq!(fifteen.days(), 1);
    }

    #[test]
    fn m12_fifteen_day_window_excludes_today_and_the_sixteenth_day_back() {
        // 10-25 (J−15) is inside. 10-24 (J−16) is the previous period. Today is outside.
        let intakes = [
            protein(TODAY, 900),
            protein("2026-11-08", 160),
            protein("2026-10-25", 140),
            protein("2026-10-24", 100),
            protein("2026-10-10", 100),
            protein("2026-10-09", 900),
        ];
        let result = protein_mean(&intakes, day(TODAY), FIFTEEN)
            .unwrap()
            .unwrap();
        assert_eq!(result.mean_g(), 150);
        assert_eq!(result.days(), 2);
        assert_eq!(result.gap_g(), Some(50));
    }

    #[test]
    fn m12_fifteen_day_gap_is_absent_when_the_previous_period_is_empty() {
        // 10-09 (J−31) is outside the previous fifteen days.
        let intakes = [protein("2026-11-08", 120), protein("2026-10-09", 80)];
        let result = protein_mean(&intakes, day(TODAY), FIFTEEN)
            .unwrap()
            .unwrap();
        assert_eq!(result.gap_g(), None);
    }

    #[test]
    fn m12_fifteen_day_period_with_no_intake_is_not_a_mean_of_zero() {
        assert_eq!(protein_mean(&[], day(TODAY), FIFTEEN), Ok(None));
    }

    #[test]
    fn m7_days_are_the_seven_days_ending_yesterday_oldest_first() {
        let listed: Vec<String> = days_of(&[]).into_iter().map(|(date, _)| date).collect();
        assert_eq!(
            listed,
            [
                "2026-11-02",
                "2026-11-03",
                "2026-11-04",
                "2026-11-05",
                "2026-11-06",
                "2026-11-07",
                "2026-11-08"
            ]
        );
    }

    #[test]
    fn m7_today_and_older_days_are_not_in_the_chart() {
        let intakes = [protein(TODAY, 900), protein("2026-11-01", 800)];
        assert!(days_of(&intakes).iter().all(|(_, value)| value.is_none()));
    }

    #[test]
    fn m8_a_day_without_intake_is_a_hole_not_zero() {
        let found = days_of(&[protein("2026-11-04", 120)]);
        assert_eq!(found[2], ("2026-11-04".to_owned(), Some(1200)));
        assert_eq!(found[0].1, None);
        assert_eq!(found[6].1, None);
    }

    #[test]
    fn m8_a_day_at_zero_grams_is_a_value_of_zero() {
        let found = days_of(&[protein("2026-11-05", 0)]);
        assert_eq!(found[3].1, Some(0));
    }

    #[test]
    fn m10_a_day_keeps_its_tenths_of_a_gram() {
        let found = days_of(&[protein_tenths("2026-11-08", 1425)]);
        assert_eq!(found[6].1, Some(1425));
    }

    #[test]
    fn s2_days_with_two_intakes_on_one_date_is_an_error() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-08", 50)];
        assert_eq!(protein_days(&intakes, day(TODAY)), Err(DuplicateIntakeDate));
    }

    #[test]
    fn m3_empty_window_is_not_a_mean_of_zero() {
        assert_eq!(week(&[]), None);
    }

    #[test]
    fn m1_today_is_outside_the_window() {
        let only_today = protein(TODAY, 150);
        assert_eq!(week(&[only_today]), None);
    }

    #[test]
    fn m3_intake_on_the_previous_week_only_is_still_no_mean() {
        let previous = protein("2026-11-01", 150);
        assert_eq!(week(&[previous]), None);
    }

    #[test]
    fn m2_a_day_at_zero_grams_counts_and_a_missing_day_does_not() {
        // Window 11-02..=11-08. Three entered days, four missing.
        let intakes = [
            protein("2026-11-02", 150),
            protein("2026-11-03", 0),
            protein("2026-11-04", 90),
        ];
        let result = week(&intakes).unwrap();
        assert_eq!(result.days(), 3);
        assert_eq!(result.mean_g(), 80);
    }

    #[test]
    fn m2_a_single_day_at_zero_grams_is_a_mean_of_zero() {
        let result = week(&[protein("2026-11-08", 0)]).unwrap();
        assert_eq!(result.mean_g(), 0);
        assert_eq!(result.days(), 1);
    }

    #[test]
    fn m1_window_runs_from_seven_days_before_today_to_yesterday() {
        // 11-02 (J−7) is inside. 11-01 (J−8) is the previous week. 10-25 (J−15) is outside both.
        let intakes = [
            protein(TODAY, 900),
            protein("2026-11-08", 160),
            protein("2026-11-02", 140),
            protein("2026-11-01", 100),
            protein("2026-10-26", 100),
            protein("2026-10-25", 900),
        ];
        let result = week(&intakes).unwrap();
        assert_eq!(result.mean_g(), 150);
        assert_eq!(result.days(), 2);
        assert_eq!(result.gap_g(), Some(50));
    }

    #[test]
    fn m4_no_previous_intake_is_not_a_gap_of_zero() {
        let result = week(&[protein("2026-11-08", 120)]).unwrap();
        assert_eq!(result.mean_g(), 120);
        assert_eq!(result.gap_g(), None);
    }

    #[test]
    fn m4_a_real_tie_is_a_gap_of_zero() {
        let intakes = [protein("2026-11-08", 120), protein("2026-11-01", 120)];
        assert_eq!(week(&intakes).unwrap().gap_g(), Some(0));
    }

    #[test]
    fn m4_negative_gap_is_signed() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-01", 142)];
        assert_eq!(week(&intakes).unwrap().gap_g(), Some(-42));
    }

    #[test]
    fn m5_half_gram_rounds_away_from_zero() {
        // 100,0 g and 101,0 g: mean 100,5 g, displayed 101.
        let intakes = [
            protein_tenths("2026-11-07", 1000),
            protein_tenths("2026-11-08", 1010),
        ];
        assert_eq!(week(&intakes).unwrap().mean_g(), 101);
    }

    #[test]
    fn m5_a_tenth_of_a_gram_below_the_half_rounds_down() {
        // 100,0 g and 100,9 g: mean 100,45 g, displayed 100.
        let intakes = [
            protein_tenths("2026-11-07", 1000),
            protein_tenths("2026-11-08", 1009),
        ];
        assert_eq!(week(&intakes).unwrap().mean_g(), 100);
    }

    #[test]
    fn m5_gap_subtracts_the_displayed_means() {
        // This week 100,5 → 101. Previous week 101,0 → 101. Displayed gap 0, raw gap −0,5.
        let intakes = [
            protein_tenths("2026-11-07", 1000),
            protein_tenths("2026-11-08", 1010),
            protein_tenths("2026-10-31", 1010),
            protein_tenths("2026-11-01", 1010),
        ];
        let result = week(&intakes).unwrap();
        assert_eq!(result.mean_g(), 101);
        assert_eq!(result.gap_g(), Some(0));
    }

    #[test]
    fn only_protein_feeds_the_mean() {
        let intake = Intake::new(day("2026-11-08"), 1500, 2000, 600, Some(2000)).unwrap();
        assert_eq!(week(&[intake]).unwrap().mean_g(), 150);
    }

    #[test]
    fn order_does_not_matter() {
        let mut intakes = [
            protein("2026-11-02", 100),
            protein("2026-11-08", 50),
            protein("2026-10-26", 80),
            protein("2026-01-01", 10),
        ];
        let expected = protein_week(&intakes, day(TODAY));
        intakes.reverse();
        assert_eq!(protein_week(&intakes, day(TODAY)), expected);
    }

    #[test]
    fn s2_duplicate_date_in_the_window_is_an_error() {
        let intakes = [protein("2026-11-08", 100), protein("2026-11-08", 50)];
        assert_eq!(protein_week(&intakes, day(TODAY)), Err(DuplicateIntakeDate));
    }

    #[test]
    fn s2_duplicate_date_outside_the_windows_is_an_error() {
        let intakes = [
            protein("2026-11-08", 100),
            protein("2026-09-01", 10),
            protein("2026-09-01", 20),
        ];
        assert_eq!(protein_week(&intakes, day(TODAY)), Err(DuplicateIntakeDate));
    }
}

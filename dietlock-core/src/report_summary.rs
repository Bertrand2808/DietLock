//! Means of the 15-day report summary (R11–R13).
//!
//! The window is the 15 calendar days ending today, today included, unlike the
//! energy and macro means, which stop at yesterday. Only days with an intake
//! count, and a day without one is never read as 0 (R12). Rounding follows
//! E5 and M5: a half rounds away from zero. Computed on demand and never stored.

use crate::day::Day;
use crate::energy::energy_kcal;
use crate::energy_week::{DuplicateIntakeDate, div_round_half_away};
use crate::intake::Intake;
use std::collections::HashSet;

/// Days in the report window. R1.
pub const REPORT_DAYS: u16 = 15;

const TENTHS_PER_GRAM: i64 = 10;
const ML_PER_HUNDREDTH_L: i64 = 10;

/// Means of the report window. A `None` mean is an absence, not 0 (R12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportSummary {
    /// Days with an intake. It is the count behind the energy and the macro means.
    pub intake_days: u32,
    /// Whole kcal per counted day (R13).
    pub energy_kcal: Option<u32>,
    /// Whole grams per counted day (R13).
    pub protein_g: Option<u32>,
    pub carbs_g: Option<u32>,
    pub fat_g: Option<u32>,
    /// Days with an intake that has water. Water is optional (S4).
    pub water_days: u32,
    /// Hundredths of a litre per day with water (R13), e.g. 366 for 3,66 L.
    pub water_hundredth_l: Option<u32>,
}

/// Means over the 15 days ending `today`.
///
/// # Errors
///
/// `Err` if any date appears twice (S2).
pub fn report_summary(
    intakes: &[Intake],
    today: Day,
) -> Result<ReportSummary, DuplicateIntakeDate> {
    let mut seen = HashSet::new();
    if !intakes.iter().all(|i| seen.insert(i.date())) {
        return Err(DuplicateIntakeDate);
    }
    let from = today.minus_days(REPORT_DAYS - 1);
    let counted: Vec<&Intake> = intakes
        .iter()
        .filter(|i| i.date() <= today && from.is_none_or(|from| i.date() >= from))
        .collect();
    let days = i64::try_from(counted.len()).unwrap_or(0);
    let macro_mean_g = |tenths: fn(&Intake) -> u16| {
        mean(
            counted.iter().map(|i| i64::from(tenths(i))).sum(),
            days,
            TENTHS_PER_GRAM,
        )
    };
    let water: Vec<i64> = counted
        .iter()
        .filter_map(|i| i.water_ml().map(i64::from))
        .collect();
    let water_days = i64::try_from(water.len()).unwrap_or(0);
    Ok(ReportSummary {
        intake_days: u32::try_from(days).unwrap_or(0),
        energy_kcal: mean(
            counted.iter().map(|i| i64::from(energy_kcal(i))).sum(),
            days,
            1,
        ),
        protein_g: macro_mean_g(Intake::protein_tenth_g),
        carbs_g: macro_mean_g(Intake::carbs_tenth_g),
        fat_g: macro_mean_g(Intake::fat_tenth_g),
        water_days: u32::try_from(water_days).unwrap_or(0),
        water_hundredth_l: mean(water.iter().sum(), water_days, ML_PER_HUNDREDTH_L),
    })
}

/// `sum / (days * unit)` rounded half away from zero, or `None` with no counted day.
fn mean(sum: i64, days: i64, unit: i64) -> Option<u32> {
    if days == 0 {
        return None;
    }
    u32::try_from(div_round_half_away(sum, days * unit)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    const TODAY: &str = "2026-11-24";

    /// Protein, carbs and fat in tenths of a gram, water in ml.
    fn intake(date: &str, p: u16, c: u16, f: u16, water: Option<u16>) -> Intake {
        Intake::new(day(date), p, c, f, water).unwrap()
    }

    fn summary(intakes: &[Intake]) -> ReportSummary {
        report_summary(intakes, day(TODAY)).unwrap()
    }

    #[test]
    fn r12_no_intake_means_no_mean_not_zero() {
        let none = summary(&[]);
        assert_eq!(none.intake_days, 0);
        assert_eq!(none.energy_kcal, None);
        assert_eq!(none.protein_g, None);
        assert_eq!(none.carbs_g, None);
        assert_eq!(none.fat_g, None);
        assert_eq!(none.water_days, 0);
        assert_eq!(none.water_hundredth_l, None);
    }

    #[test]
    fn r12_the_window_is_fifteen_days_and_includes_today() {
        let intakes = [
            intake(TODAY, 1000, 0, 0, None),
            intake("2026-11-10", 2000, 0, 0, None),
            intake("2026-11-09", 9000, 0, 0, None),
        ];
        let s = summary(&intakes);
        assert_eq!(s.intake_days, 2, "J−14 counts, J−15 does not");
        assert_eq!(s.protein_g, Some(150));
    }

    #[test]
    fn r12_a_future_intake_is_outside_the_window() {
        let s = summary(&[intake("2026-11-25", 1000, 0, 0, None)]);
        assert_eq!(s.intake_days, 0);
    }

    #[test]
    fn r12_a_day_entered_at_zero_counts() {
        let intakes = [
            intake("2026-11-23", 0, 0, 0, Some(0)),
            intake("2026-11-22", 1000, 2000, 500, Some(200)),
        ];
        let s = summary(&intakes);
        assert_eq!(s.intake_days, 2);
        assert_eq!(s.protein_g, Some(50));
        assert_eq!(s.energy_kcal, Some(825), "(0 + 1650) / 2");
        assert_eq!(s.water_days, 2);
        assert_eq!(s.water_hundredth_l, Some(10));
    }

    #[test]
    fn r13_energy_mean_is_the_mean_of_whole_day_energies_half_away() {
        // 4 × 10 g = 40 kcal and 4 × 10,1 g = 40,4 → 40 kcal. 40 and 41 average 40,5 → 41.
        let a = intake("2026-11-23", 100, 0, 0, None);
        let b = intake("2026-11-22", 103, 0, 0, None);
        assert_eq!(summary(&[a, b]).energy_kcal, Some(41));
    }

    #[test]
    fn r13_macro_mean_rounds_a_half_gram_away_from_zero() {
        // 10 g and 11 g average 10,5 g, shown as 11 g.
        let intakes = [
            intake("2026-11-23", 100, 0, 0, None),
            intake("2026-11-22", 110, 0, 0, None),
        ];
        assert_eq!(summary(&intakes).protein_g, Some(11));
    }

    #[test]
    fn r11_each_macro_reads_its_own_column() {
        let s = summary(&[intake(TODAY, 1500, 2000, 600, None)]);
        assert_eq!(s.protein_g, Some(150));
        assert_eq!(s.carbs_g, Some(200));
        assert_eq!(s.fat_g, Some(60));
    }

    #[test]
    fn r11_water_counts_only_days_with_water() {
        let intakes = [
            intake("2026-11-23", 0, 0, 0, Some(3000)),
            intake("2026-11-22", 0, 0, 0, None),
            intake("2026-11-21", 0, 0, 0, Some(3320)),
        ];
        let s = summary(&intakes);
        assert_eq!(s.intake_days, 3);
        assert_eq!(s.water_days, 2);
        assert_eq!(s.water_hundredth_l, Some(316), "3,16 L");
    }

    #[test]
    fn s2_two_intakes_on_one_date_are_refused() {
        let twice = [intake(TODAY, 0, 0, 0, None), intake(TODAY, 10, 0, 0, None)];
        assert!(report_summary(&twice, day(TODAY)).is_err());
    }
}

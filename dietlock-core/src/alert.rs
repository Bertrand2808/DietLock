//! Alert: a fasted weigh-in at least 3 kg away from the previous day's trend.
//!
//! Computed on demand from the weigh-ins, never stored (A4).

use crate::trend::{DuplicateDate, trend};
use crate::weigh_in::WeighIn;

/// A2: 3 kg, in hundredths of a kg, the same unit as [`trend`].
pub const ALERT_HUNDREDTH_KG: u32 = 300;

/// A1–A3: the weigh-in alerts when it is fasted and its weight is at least
/// 3 kg above or below the trend of the previous day.
///
/// The previous day's trend window ends the day before, so this weigh-in is
/// not part of the reference (A1). `Err` if `weigh_ins` has a duplicate date,
/// as in [`trend`].
pub fn alert(weigh_ins: &[WeighIn], weigh_in: &WeighIn) -> Result<bool, DuplicateDate> {
    if !weigh_in.fasted() {
        return Ok(false);
    }
    let Some(previous) = weigh_in.date().previous() else {
        return Ok(false);
    };
    let Some(reference) = trend(weigh_ins, previous)? else {
        return Ok(false);
    };
    let weigh_hundredth = u32::from(weigh_in.weight_tenth_kg()) * 10;
    Ok(weigh_hundredth.abs_diff(reference) >= ALERT_HUNDREDTH_KG)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day::Day;
    use crate::tests::day;
    use crate::weigh_in::WeighIn;

    fn previous(day: Day) -> Day {
        day.previous().unwrap()
    }

    fn w(date: &str, tenth: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(date), tenth, fasted).unwrap()
    }

    /// Four fasted weigh-ins at 79.0 kg, so the trend of 15 Oct is 79.00 kg.
    fn steady_week() -> Vec<WeighIn> {
        (12..=15)
            .map(|d| w(&format!("2026-10-{d}"), 790, true))
            .collect()
    }

    #[test]
    fn a1_reference_is_the_previous_days_trend() {
        // 82.0 against 79.0 is exactly 3 kg, so it alerts.
        // A trend that included this weigh-in would be 79.6 and would not.
        let mut v = steady_week();
        let today = w("2026-10-16", 820, true);
        v.push(today);
        assert_eq!(trend(&v, previous(today.date())), Ok(Some(7900)));
        assert_eq!(alert(&v, &today), Ok(true));
    }

    #[test]
    fn a2_three_kilograms_is_the_threshold() {
        let mut above = steady_week();
        let on_threshold = w("2026-10-16", 820, true);
        above.push(on_threshold);
        assert_eq!(alert(&above, &on_threshold), Ok(true));

        let mut below = steady_week();
        let under = w("2026-10-16", 819, true);
        below.push(under);
        assert_eq!(alert(&below, &under), Ok(false));

        let mut down = steady_week();
        let dropped = w("2026-10-16", 760, true);
        down.push(dropped);
        assert_eq!(alert(&down, &dropped), Ok(true));
    }

    #[test]
    fn a3_non_fasted_weigh_in_does_not_alert() {
        let mut v = steady_week();
        let late = w("2026-10-16", 900, false);
        v.push(late);
        assert_eq!(alert(&v, &late), Ok(false));
    }

    #[test]
    fn a3_without_a_previous_trend_there_is_no_alert() {
        let mut v: Vec<_> = (13..=15)
            .map(|d| w(&format!("2026-10-{d}"), 790, true))
            .collect();
        let today = w("2026-10-16", 900, true);
        v.push(today);
        assert_eq!(trend(&v, previous(today.date())), Ok(None));
        assert_eq!(alert(&v, &today), Ok(false));
    }

    #[test]
    fn duplicate_date_is_an_error() {
        let mut v = steady_week();
        v.push(w("2026-10-15", 800, true));
        let today = w("2026-10-16", 820, true);
        v.push(today);
        assert_eq!(alert(&v, &today), Err(DuplicateDate));
    }
}

//! Weight trend and variations, computed on demand and never stored.
//!
//! One rounding rule everywhere: a half rounds away from zero.

use crate::day::Day;
use crate::weigh_in::WeighIn;

/// Number of calendar days in the trend window (J−6 to J).
pub const TREND_WINDOW_DAYS: u16 = 7;
/// Minimum number of fasted weigh-ins in the window for a trend to exist.
pub const TREND_MIN_WEIGH_INS: u32 = 4;

/// Integer division rounding a half away from zero. `den` must be positive.
fn div_round_half_away(num: i64, den: i64) -> i64 {
    debug_assert!(den > 0);
    let q = (num.abs() * 2 + den) / (den * 2);
    if num < 0 { -q } else { q }
}

/// Trend at `day`, in hundredths of a kg: simple mean of the fasted weigh-ins
/// dated from J−6 to J (calendar days), rounded to the hundredth.
/// `None` with fewer than [`TREND_MIN_WEIGH_INS`] such weigh-ins.
/// The input can be in any order.
pub fn trend(weigh_ins: &[WeighIn], day: Day) -> Option<u32> {
    let from = day.minus_days(TREND_WINDOW_DAYS - 1)?;
    let (sum_tenth, count) = weigh_ins
        .iter()
        .filter(|w| w.fasted() && w.date() >= from && w.date() <= day)
        .fold((0u32, 0u32), |(s, c), w| {
            (s + u32::from(w.weight_tenth_kg()), c + 1)
        });
    if count < TREND_MIN_WEIGH_INS {
        return None;
    }
    let mean = div_round_half_away(i64::from(sum_tenth) * 10, i64::from(count));
    u32::try_from(mean).ok()
}

/// Rounds hundredths of a kg to the displayed tenths, half away from zero.
pub fn display_tenth_kg(hundredth_kg: i32) -> i32 {
    // |i32| / 10 always fits back in an i32.
    div_round_half_away(i64::from(hundredth_kg), 10) as i32
}

/// Variation of the trend between J−`days` and J, in tenths of a kg, computed
/// from the displayed values so that what the user reads adds up.
/// `None` if either trend is absent (never 0 in that case).
pub fn variation(weigh_ins: &[WeighIn], day: Day, days: u16) -> Option<i32> {
    let now = trend(weigh_ins, day)?;
    let before = trend(weigh_ins, day.minus_days(days)?)?;
    let shown = |h: u32| i32::try_from(h).ok().map(display_tenth_kg);
    Some(shown(now)? - shown(before)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    fn w(date: &str, tenth: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(date), tenth, fasted).unwrap()
    }

    fn base() -> Vec<WeighIn> {
        vec![
            w("2026-10-07", 800, true),
            w("2026-10-09", 810, true),
            w("2026-10-11", 795, true),
        ]
    }

    #[test]
    fn three_weigh_ins_give_no_trend() {
        assert_eq!(trend(&base(), day("2026-10-12")), None);
    }

    #[test]
    fn fourth_weigh_in_gives_8025_displayed_803() {
        let mut v = base();
        v.push(w("2026-10-12", 805, true));
        assert_eq!(trend(&v, day("2026-10-12")), Some(8025));
        assert_eq!(display_tenth_kg(8025), 803);
        assert_eq!(display_tenth_kg(8024), 802);
    }

    #[test]
    fn negative_display_rounds_away_from_zero() {
        assert_eq!(display_tenth_kg(-125), -13);
        assert_eq!(display_tenth_kg(-124), -12);
        assert_eq!(display_tenth_kg(0), 0);
    }

    #[test]
    fn internal_mean_rounds_half_away_from_zero() {
        // 800+800+801+801+801+801 = 4804 tenths / 6 = 8006.67 -> 8007
        let v: Vec<_> = [800, 800, 801, 801, 801, 801]
            .iter()
            .enumerate()
            .map(|(i, &t)| w(&format!("2026-10-0{}", i + 1), t, true))
            .collect();
        assert_eq!(trend(&v, day("2026-10-07")), Some(8007));
        // 800*3 + 801 = 3201 / 4 = 8002.5 -> 8003
        let v = vec![
            w("2026-10-01", 800, true),
            w("2026-10-02", 800, true),
            w("2026-10-03", 800, true),
            w("2026-10-04", 801, true),
        ];
        assert_eq!(trend(&v, day("2026-10-04")), Some(8003));
    }

    #[test]
    fn double_rounding_is_accepted() {
        // A mean of tenths over 4 to 7 values can never land in [x4.5, x5) hundredths,
        // so the rule is frozen on the rounding chain itself: 80.2496 kg.
        let internal = div_round_half_away(802_496, 100);
        assert_eq!(internal, 8025);
        assert_eq!(display_tenth_kg(internal as i32), 803);
        assert_eq!(div_round_half_away(802_496, 1000), 802);
    }

    #[test]
    fn non_fasted_weigh_in_is_ignored() {
        let mut v = base();
        v.push(w("2026-10-12", 805, false));
        assert_eq!(trend(&v, day("2026-10-12")), None);
        v.push(w("2026-10-10", 805, true));
        assert_eq!(trend(&v, day("2026-10-12")), Some(8025));
    }

    #[test]
    fn weigh_in_at_j_minus_7_is_outside() {
        let mut v = base();
        v.push(w("2026-10-05", 805, true));
        assert_eq!(trend(&v, day("2026-10-12")), None);
        v.push(w("2026-10-06", 805, true)); // J−6 is inside
        assert_eq!(trend(&v, day("2026-10-12")), Some(8025));
    }

    #[test]
    fn future_weigh_in_is_outside() {
        let mut v = base();
        v.push(w("2026-10-13", 805, true));
        assert_eq!(trend(&v, day("2026-10-12")), None);
    }

    #[test]
    fn window_is_calendar_days_not_last_seven_weigh_ins() {
        // Window 10-06..=10-12 with 10-09 missing: 6 weigh-ins inside.
        // The last 7 weigh-ins would also take 10-05 (1000), changing the mean.
        let v = vec![
            w("2026-10-05", 1000, true),
            w("2026-10-06", 800, true),
            w("2026-10-07", 800, true),
            w("2026-10-08", 800, true),
            w("2026-10-10", 800, true),
            w("2026-10-11", 800, true),
            w("2026-10-12", 800, true),
        ];
        assert_eq!(trend(&v, day("2026-10-12")), Some(8000));
    }

    #[test]
    fn order_does_not_matter() {
        let mut v = base();
        v.push(w("2026-10-12", 805, true));
        v.push(w("2026-10-01", 700, true));
        let expected = trend(&v, day("2026-10-12"));
        v.reverse();
        v.swap(0, 3);
        assert_eq!(trend(&v, day("2026-10-12")), expected);
        assert_eq!(expected, Some(8025));
    }

    #[test]
    fn max_weights_do_not_overflow() {
        let v: Vec<_> = (1..=7)
            .map(|i| w(&format!("2026-10-0{i}"), 2500, true))
            .collect();
        assert_eq!(trend(&v, day("2026-10-07")), Some(25000));
    }

    fn week(start_day: u8, tenths: [u16; 4]) -> Vec<WeighIn> {
        tenths
            .iter()
            .enumerate()
            .map(|(i, &t)| w(&format!("2026-10-{:02}", start_day + i as u8), t, true))
            .collect()
    }

    #[test]
    fn variation_uses_displayed_values() {
        // J−7 = 10-05: 791,791,791,792,792 = 3957/5 -> 7914, displayed 791.
        // J = 10-12: 800,810,795,805 -> 8025, displayed 803.
        // 803 − 791 = 12, whereas 8025 − 7914 = 111 hundredths would display 11.
        let mut v = Vec::new();
        v.extend(
            [791, 791, 791, 792, 792]
                .iter()
                .enumerate()
                .map(|(i, &t)| w(&format!("2026-10-0{}", i + 1), t, true)),
        );
        v.extend(week(9, [800, 810, 795, 805]));
        assert_eq!(trend(&v, day("2026-10-05")), Some(7914));
        assert_eq!(trend(&v, day("2026-10-12")), Some(8025));
        assert_eq!(variation(&v, day("2026-10-12"), 7), Some(12));
    }

    #[test]
    fn negative_variation() {
        // J−7 = 10-05: 8125 ; J = 10-12: 8000 -> displayed 813 and 800 -> −13.
        let mut v = week(2, [810, 820, 805, 815]);
        v.extend(week(9, [800, 800, 800, 800]));
        assert_eq!(trend(&v, day("2026-10-05")), Some(8125));
        assert_eq!(variation(&v, day("2026-10-12"), 7), Some(-13));
    }

    #[test]
    fn variation_absent_without_past_trend() {
        let v = week(9, [800, 800, 800, 800]);
        assert_eq!(variation(&v, day("2026-10-12"), 7), None);
        assert_eq!(variation(&v, day("2026-10-12"), 28), None);
    }

    #[test]
    fn variation_28_days() {
        let mut v = week(9, [800, 800, 800, 800]);
        // J−28 = 09-14: four weigh-ins at 82.0 on 09-11..=09-14.
        v.extend(
            week(9, [820, 820, 820, 820])
                .into_iter()
                .map(|x| WeighIn::new(x.date().minus_days(28).unwrap(), 820, true).unwrap()),
        );
        assert_eq!(variation(&v, day("2026-10-12"), 28), Some(-20));
        assert_eq!(variation(&v, day("2026-10-12"), 7), None);
    }

    #[test]
    fn variation_absent_without_current_trend() {
        let v = week(2, [800, 800, 800, 800]);
        assert_eq!(variation(&v, day("2026-10-12"), 7), None);
    }
}

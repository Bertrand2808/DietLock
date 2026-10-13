//! Values sent to the web view, and user-readable error messages.
//! Pure functions, so they are unit-tested without a running app.

use dietlock_core::{
    Day, TREND_WINDOW_DAYS, ValidationError, WEIGHT_TENTH_KG_MAX, WEIGHT_TENTH_KG_MIN, WeighIn,
    display_tenth_kg,
};
use dietlock_store::StoreError;
use serde::Serialize;

/// Number of calendar days shown in the history (J−29 to J).
pub const HISTORY_DAYS: u16 = 30;

// The history must cover the trend window, since the trend is computed on it.
const _: () = assert!(HISTORY_DAYS >= TREND_WINDOW_DAYS);

/// First day of the history ending at `today`; `today` itself at the lower
/// bound of the calendar.
pub fn history_start(today: Day) -> Day {
    today.minus_days(HISTORY_DAYS - 1).unwrap_or(today)
}

/// Tenths of a kg written with a decimal comma, e.g. `82,6`.
pub fn format_tenth_kg(tenth: i32) -> String {
    let sign = if tenth < 0 { "-" } else { "" };
    let abs = tenth.unsigned_abs();
    format!("{sign}{},{}", abs / 10, abs % 10)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeighInView {
    pub date: String,
    pub weight: String,
    pub fasted: bool,
}

impl From<&WeighIn> for WeighInView {
    fn from(w: &WeighIn) -> Self {
        Self {
            date: w.date().to_string(),
            weight: format_tenth_kg(i32::from(w.weight_tenth_kg())),
            fasted: w.fasted(),
        }
    }
}

/// Result of `save_weigh_in` when nothing went wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum SaveOutcome {
    Saved,
    /// The date already holds `old`; nothing was written.
    AlreadyFilled {
        old: WeighInView,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    /// Today, which is also the maximum date of the form.
    pub today: String,
    /// Date proposed for a new weigh-in.
    pub proposed_date: String,
    /// Most recent first.
    pub weigh_ins: Vec<WeighInView>,
    /// Today's trend as displayed, `None` when there is none yet.
    pub trend: Option<String>,
}

impl Listing {
    /// `weigh_ins` in ascending order, as the store returns them;
    /// `trend_hundredth` as returned by `dietlock_core::trend`.
    pub fn new(
        today: Day,
        proposed_date: Day,
        weigh_ins: &[WeighIn],
        trend_hundredth: Option<u32>,
    ) -> Self {
        Self {
            today: today.to_string(),
            proposed_date: proposed_date.to_string(),
            weigh_ins: weigh_ins.iter().rev().map(WeighInView::from).collect(),
            trend: trend_hundredth
                .and_then(|h| i32::try_from(h).ok())
                .map(|h| format_tenth_kg(display_tenth_kg(h))),
        }
    }
}

/// Message shown to the user for a refused input.
pub fn validation_message(e: &ValidationError) -> String {
    match e {
        ValidationError::OutOfRange {
            field: "weight_tenth_kg",
            ..
        } => format!(
            "The weight must be between {} and {} kg.",
            format_tenth_kg(i32::from(WEIGHT_TENTH_KG_MIN)),
            format_tenth_kg(i32::from(WEIGHT_TENTH_KG_MAX)),
        ),
        ValidationError::InvalidNumber {
            field: "weight_tenth_kg",
            input,
        } => format!("{input:?} is not a weight: type kg with at most one decimal, e.g. 82,6."),
        ValidationError::FutureDate(d) => format!("The date {d} is in the future."),
        ValidationError::InvalidDate(d) => format!("The date {d:?} is not valid."),
        other => other.to_string(),
    }
}

/// Message shown to the user for a database failure.
pub fn store_message(e: &StoreError) -> String {
    format!("The database could not be read or written: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dietlock_core::parse_weight_tenth_kg;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    fn weigh(d: &str, w: u16) -> WeighIn {
        WeighIn::new(day(d), w, true).unwrap()
    }

    #[test]
    fn history_is_thirty_calendar_days() {
        assert_eq!(history_start(day("2026-10-13")), day("2026-09-14"));
        assert_eq!(history_start(day("2026-03-01")), day("2026-01-31"));
    }

    #[test]
    fn weight_is_written_with_a_comma() {
        assert_eq!(format_tenth_kg(826), "82,6");
        assert_eq!(format_tenth_kg(800), "80,0");
        assert_eq!(format_tenth_kg(-13), "-1,3");
    }

    #[test]
    fn listing_is_most_recent_first_with_display_trend() {
        let ws = [weigh("2026-10-12", 800), weigh("2026-10-13", 826)];
        let l = Listing::new(day("2026-10-13"), day("2026-10-13"), &ws, Some(8025));
        assert_eq!(l.today, "2026-10-13");
        assert_eq!(l.proposed_date, "2026-10-13");
        assert_eq!(l.weigh_ins[0].date, "2026-10-13");
        assert_eq!(l.weigh_ins[0].weight, "82,6");
        assert_eq!(l.trend.as_deref(), Some("80,3"));
        assert_eq!(
            Listing::new(day("2026-10-13"), day("2026-10-13"), &ws, None).trend,
            None
        );
    }

    #[test]
    fn refused_weights_have_readable_messages() {
        let abc = validation_message(&parse_weight_tenth_kg("abc").unwrap_err());
        assert!(abc.contains("not a weight"), "{abc}");
        let big = validation_message(&parse_weight_tenth_kg("300").unwrap_err());
        assert_eq!(big, "The weight must be between 30,0 and 250,0 kg.");
    }
}

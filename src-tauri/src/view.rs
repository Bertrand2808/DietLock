//! Values sent to the web view, and user-readable error messages.
//! Pure functions, so they are unit-tested without a running app.

use dietlock_core::{
    Day, Intake, MACRO_G_MAX, TREND_WINDOW_DAYS, ValidationError, WEIGHT_TENTH_KG_MAX,
    WEIGHT_TENTH_KG_MIN, WeighIn, display_tenth_kg, energy_kcal,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeView {
    pub date: String,
    pub protein_g: u16,
    pub carbs_g: u16,
    pub fat_g: u16,
    /// Litres with a decimal comma, e.g. `1,5`; `None` when not entered.
    pub water: Option<String>,
    pub energy_kcal: u32,
}

impl From<&Intake> for IntakeView {
    fn from(i: &Intake) -> Self {
        Self {
            date: i.date().to_string(),
            protein_g: i.protein_g(),
            carbs_g: i.carbs_g(),
            fat_g: i.fat_g(),
            // The core stores water in whole tenths of a litre (100 ml).
            water: i.water_ml().map(|ml| format_tenth_kg(i32::from(ml / 100))),
            energy_kcal: energy_kcal(i),
        }
    }
}

/// Result of `save_intake` when nothing went wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum IntakeSaveOutcome {
    Saved,
    /// The date already holds `old`; nothing was written.
    AlreadyFilled {
        old: IntakeView,
    },
}

/// One day of the history; either entry may be missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayView {
    pub date: String,
    pub weigh_in: Option<WeighInView>,
    pub intake: Option<IntakeView>,
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
    /// Every day from today back to the history start, most recent first.
    pub days: Vec<DayView>,
    /// Today's trend as displayed, `None` when there is none yet.
    pub trend: Option<String>,
}

impl Listing {
    /// `weigh_ins` and `intakes` within the history, one per date at most;
    /// `trend_hundredth` as returned by `dietlock_core::trend`.
    pub fn new(
        today: Day,
        proposed_date: Day,
        weigh_ins: &[WeighIn],
        intakes: &[Intake],
        trend_hundredth: Option<u32>,
    ) -> Self {
        let days = (0..HISTORY_DAYS)
            .map_while(|n| today.minus_days(n))
            .map(|d| DayView {
                date: d.to_string(),
                weigh_in: weigh_ins
                    .iter()
                    .find(|w| w.date() == d)
                    .map(WeighInView::from),
                intake: intakes.iter().find(|i| i.date() == d).map(IntakeView::from),
            })
            .collect();
        Self {
            today: today.to_string(),
            proposed_date: proposed_date.to_string(),
            days,
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
        ValidationError::InvalidNumber { field, input } if macro_label(field).is_some() => {
            let label = macro_label(field).unwrap_or(field);
            if input.is_empty() {
                format!("{label} is required.")
            } else {
                format!("{input:?} is not valid for {label}: type whole grams, e.g. 150.")
            }
        }
        ValidationError::OutOfRange { field, .. } if macro_label(field).is_some() => format!(
            "{} must be between 0 and {MACRO_G_MAX} g.",
            macro_label(field).unwrap_or(field)
        ),
        ValidationError::InvalidNumber {
            field: "water_ml",
            input,
        } => format!("{input:?} is not a water amount: type litres, e.g. 1,5, or leave it empty."),
        ValidationError::OutOfRange {
            field: "water_ml", ..
        } => "Water must be between 0 and 10,0 L.".to_owned(),
        ValidationError::FutureDate(d) => format!("The date {d} is in the future."),
        ValidationError::InvalidDate(d) => format!("The date {d:?} is not valid."),
        other => other.to_string(),
    }
}

/// Label of a macro field, `None` for other fields.
fn macro_label(field: &str) -> Option<&'static str> {
    match field {
        "protein_g" => Some("Protein"),
        "carbs_g" => Some("Carbs"),
        "fat_g" => Some("Fat"),
        _ => None,
    }
}

/// Message shown to the user for a database failure.
pub fn store_message(e: &StoreError) -> String {
    format!("The database could not be read or written: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dietlock_core::{parse_macro_g, parse_water_ml, parse_weight_tenth_kg};

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

    fn intake(d: &str, water_ml: Option<u16>) -> Intake {
        Intake::new(day(d), 150, 250, 70, water_ml).unwrap()
    }

    #[test]
    fn listing_is_most_recent_first_with_display_trend() {
        let ws = [weigh("2026-10-12", 800), weigh("2026-10-13", 826)];
        let l = Listing::new(day("2026-10-13"), day("2026-10-13"), &ws, &[], Some(8025));
        assert_eq!(l.today, "2026-10-13");
        assert_eq!(l.proposed_date, "2026-10-13");
        assert_eq!(l.days[0].date, "2026-10-13");
        assert_eq!(l.days[0].weigh_in.as_ref().unwrap().weight, "82,6");
        assert_eq!(l.trend.as_deref(), Some("80,3"));
        assert_eq!(
            Listing::new(day("2026-10-13"), day("2026-10-13"), &ws, &[], None).trend,
            None
        );
    }

    #[test]
    fn listing_has_every_day_with_weigh_in_and_intake_side_by_side() {
        let ws = [weigh("2026-10-11", 800), weigh("2026-10-13", 826)];
        let is = [intake("2026-10-12", Some(1500)), intake("2026-10-13", None)];
        let l = Listing::new(day("2026-10-13"), day("2026-10-13"), &ws, &is, None);
        assert_eq!(l.days.len(), usize::from(HISTORY_DAYS));
        assert_eq!(l.days[29].date, "2026-09-14");
        let (d13, d12, d11) = (&l.days[0], &l.days[1], &l.days[2]);
        assert!(d13.weigh_in.is_some() && d13.intake.is_some());
        assert!(d12.weigh_in.is_none());
        assert_eq!(d12.intake.as_ref().unwrap().energy_kcal, 2230);
        assert!(d11.weigh_in.is_some() && d11.intake.is_none());
        assert!(l.days[3].weigh_in.is_none() && l.days[3].intake.is_none());
    }

    #[test]
    fn intake_view_has_energy_and_optional_water() {
        let v = IntakeView::from(&intake("2026-10-12", Some(1500)));
        assert_eq!(v.energy_kcal, 2230);
        assert_eq!(v.water.as_deref(), Some("1,5"));
        assert_eq!(IntakeView::from(&intake("2026-10-12", None)).water, None);
        let empty = parse_water_ml("").unwrap();
        assert_eq!(empty, None);
    }

    #[test]
    fn refused_macros_and_water_have_readable_messages() {
        let empty = validation_message(&parse_macro_g("protein_g", "").unwrap_err());
        assert_eq!(empty, "Protein is required.");
        let bad = validation_message(&parse_macro_g("fat_g", "12,5").unwrap_err());
        assert!(bad.contains("Fat") && bad.contains("whole grams"), "{bad}");
        let big = validation_message(&parse_macro_g("carbs_g", "1001").unwrap_err());
        assert_eq!(big, "Carbs must be between 0 and 1000 g.");
        let water = validation_message(&parse_water_ml("abc").unwrap_err());
        assert!(water.contains("water"), "{water}");
    }

    #[test]
    fn refused_weights_have_readable_messages() {
        let abc = validation_message(&parse_weight_tenth_kg("abc").unwrap_err());
        assert!(abc.contains("not a weight"), "{abc}");
        let big = validation_message(&parse_weight_tenth_kg("300").unwrap_err());
        assert_eq!(big, "The weight must be between 30,0 and 250,0 kg.");
    }
}

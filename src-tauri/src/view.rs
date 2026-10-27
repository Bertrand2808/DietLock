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

/// `scale` 10 writes tenths (`12,5`) and `scale` 100 writes hundredths (`1,25`).
/// A zero fraction is omitted (`60`, `2`).
fn format_trimmed(value: u32, scale: u32) -> String {
    let whole = value / scale;
    let frac = value % scale;
    if frac == 0 {
        return whole.to_string();
    }
    let width = scale.ilog10() as usize;
    let mut digits = format!("{frac:0width$}");
    while digits.ends_with('0') {
        digits.pop();
    }
    format!("{whole},{digits}")
}

/// Millilitres written as litres. Saved water is a multiple of 10 ml.
fn format_water_l(ml: u16) -> String {
    format_trimmed(u32::from(ml) / 10, 100)
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
    /// Grams with a decimal comma, trailing zero omitted, e.g. `12,5` or `60`.
    pub protein_g: String,
    pub carbs_g: String,
    pub fat_g: String,
    /// Litres with a decimal comma, trailing zeros omitted, e.g. `1,25` or `2`.
    /// `None` when not entered.
    pub water: Option<String>,
    pub energy_kcal: u32,
}

impl From<&Intake> for IntakeView {
    fn from(i: &Intake) -> Self {
        Self {
            date: i.date().to_string(),
            protein_g: format_trimmed(u32::from(i.protein_tenth_g()), 10),
            carbs_g: format_trimmed(u32::from(i.carbs_tenth_g()), 10),
            fat_g: format_trimmed(u32::from(i.fat_tenth_g()), 10),
            water: i.water_ml().map(format_water_l),
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
///
/// Wording lives here: `dietlock-core` keeps its own error text.
pub fn validation_message(e: &ValidationError) -> String {
    match e {
        ValidationError::OutOfRange {
            field: "weight_tenth_kg",
            ..
        } => format!(
            "Le poids doit être compris entre {} et {} kg.",
            format_tenth_kg(i32::from(WEIGHT_TENTH_KG_MIN)),
            format_tenth_kg(i32::from(WEIGHT_TENTH_KG_MAX)),
        ),
        ValidationError::InvalidNumber {
            field: "weight_tenth_kg",
            input,
        } => format!(
            "{input:?} n'est pas un poids : indiquez des kilogrammes avec au plus une décimale, par exemple 82,6."
        ),
        ValidationError::InvalidNumber { field, input } if macro_name(field).is_some() => {
            let name = macro_name(field).unwrap_or(field);
            if input.is_empty() {
                format!("Les {name} sont obligatoires.")
            } else {
                format!(
                    "{input:?} n'est pas valide pour les {name} : indiquez des grammes avec au plus une décimale, par exemple 12,5."
                )
            }
        }
        ValidationError::OutOfRange { field, .. } if macro_name(field).is_some() => {
            format!(
                "Les {} doivent rester entre 0 et {MACRO_G_MAX} g.",
                macro_name(field).unwrap_or(field)
            )
        }
        ValidationError::InvalidNumber {
            field: "water_ml",
            input,
        } => format!(
            "{input:?} n'est pas une quantité d'eau : indiquez des litres avec au plus deux décimales, par exemple 1,25, ou laissez le champ vide."
        ),
        ValidationError::OutOfRange {
            field: "water_ml", ..
        } => "L'eau doit être comprise entre 0 et 10,0 L.".to_owned(),
        ValidationError::FutureDate(d) => format!("La date {d} est dans le futur."),
        ValidationError::InvalidDate(d) => format!("La date {d:?} n'est pas valide."),
        other => format!("La saisie n'est pas acceptée ({other})."),
    }
}

/// Noun of a macro field, without an article. `None` for other fields.
fn macro_name(field: &str) -> Option<&'static str> {
    match field {
        "protein_tenth_g" => Some("protéines"),
        "carbs_tenth_g" => Some("glucides"),
        "fat_tenth_g" => Some("lipides"),
        _ => None,
    }
}

/// Message shown to the user for a database failure.
///
/// The SQLite driver text is kept as the library reports it.
pub fn store_message(e: &StoreError) -> String {
    match e {
        StoreError::Sqlite(err) => {
            format!("La base de données n'a pas pu être lue ou écrite : {err}")
        }
        StoreError::UnsupportedSchemaVersion(version) => {
            format!("La base de données utilise une version inconnue ({version}).")
        }
        StoreError::InvalidRow(_) => "La base de données contient une ligne invalide.".to_owned(),
    }
}

/// Shown when the database lock cannot be taken.
pub fn database_unavailable_message() -> &'static str {
    "La base de données est indisponible, redémarrez l'application."
}

/// Shown when the history contains two weigh-ins on the same day.
pub fn duplicate_date_message() -> &'static str {
    "Deux pesées portent la même date."
}

#[cfg(test)]
mod tests {
    use super::*;
    use dietlock_core::{
        ValidationError, check_not_future, parse_macro_tenth_g, parse_water_ml,
        parse_weight_tenth_kg,
    };
    use dietlock_store::StoreError;

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
        Intake::new(day(d), 1500, 2500, 700, water_ml).unwrap()
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
    fn history_shows_one_decimal_grams_and_two_decimal_litres() {
        let i = Intake::new(
            day("2026-10-12"),
            parse_macro_tenth_g("protein_tenth_g", "12,5").unwrap(),
            parse_macro_tenth_g("carbs_tenth_g", "200.3").unwrap(),
            parse_macro_tenth_g("fat_tenth_g", "60").unwrap(),
            parse_water_ml("1,25").unwrap(),
        )
        .unwrap();
        let v = IntakeView::from(&i);
        assert_eq!(v.protein_g, "12,5");
        assert_eq!(v.carbs_g, "200,3");
        assert_eq!(v.fat_g, "60");
        assert_eq!(v.water.as_deref(), Some("1,25"));
        let fat_only = Intake::new(
            day("2026-10-12"),
            0,
            0,
            parse_macro_tenth_g("fat_tenth_g", "0,5").unwrap(),
            None,
        )
        .unwrap();
        let fat_view = IntakeView::from(&fat_only);
        assert_eq!(fat_view.fat_g, "0,5");
        assert_eq!(fat_view.energy_kcal, 5);
        let sip = Intake::new(day("2026-10-12"), 0, 0, 0, parse_water_ml("0,05").unwrap()).unwrap();
        assert_eq!(IntakeView::from(&sip).water.as_deref(), Some("0,05"));
    }

    #[test]
    fn whole_grams_and_whole_litres_keep_kcal() {
        let i = Intake::new(
            day("2026-10-12"),
            parse_macro_tenth_g("protein_tenth_g", "150").unwrap(),
            parse_macro_tenth_g("carbs_tenth_g", "0").unwrap(),
            parse_macro_tenth_g("fat_tenth_g", "0").unwrap(),
            parse_water_ml("2").unwrap(),
        )
        .unwrap();
        let v = IntakeView::from(&i);
        assert_eq!(v.protein_g, "150");
        assert_eq!(v.carbs_g, "0");
        assert_eq!(v.fat_g, "0");
        assert_eq!(v.water.as_deref(), Some("2"));
        assert_eq!(v.energy_kcal, 600);
    }

    #[test]
    fn weight_that_is_not_a_number_shows_a_french_example() {
        let abc = validation_message(&parse_weight_tenth_kg("abc").unwrap_err());
        assert_eq!(
            abc,
            "\"abc\" n'est pas un poids : indiquez des kilogrammes avec au plus une décimale, par exemple 82,6."
        );
    }

    #[test]
    fn empty_protein_is_required_in_french() {
        let empty = validation_message(&parse_macro_tenth_g("protein_tenth_g", "").unwrap_err());
        assert_eq!(empty, "Les protéines sont obligatoires.");
        assert_eq!(
            validation_message(&parse_macro_tenth_g("carbs_tenth_g", "").unwrap_err()),
            "Les glucides sont obligatoires."
        );
        assert_eq!(
            validation_message(&parse_macro_tenth_g("fat_tenth_g", "").unwrap_err()),
            "Les lipides sont obligatoires."
        );
    }

    #[test]
    fn out_of_bounds_values_and_future_dates_are_refused_in_french() {
        assert_eq!(
            validation_message(&parse_weight_tenth_kg("300").unwrap_err()),
            "Le poids doit être compris entre 30,0 et 250,0 kg."
        );
        assert_eq!(
            validation_message(&parse_macro_tenth_g("carbs_tenth_g", "1001").unwrap_err()),
            "Les glucides doivent rester entre 0 et 1000 g."
        );
        assert_eq!(
            validation_message(&parse_macro_tenth_g("protein_tenth_g", "1001").unwrap_err()),
            "Les protéines doivent rester entre 0 et 1000 g."
        );
        assert_eq!(
            validation_message(&parse_water_ml("11").unwrap_err()),
            "L'eau doit être comprise entre 0 et 10,0 L."
        );
        let future = check_not_future(day("2026-10-14"), day("2026-10-13")).unwrap_err();
        assert_eq!(
            validation_message(&future),
            "La date 2026-10-14 est dans le futur."
        );
        let invalid = Day::parse("2026-13-01").unwrap_err();
        assert_eq!(
            validation_message(&invalid),
            "La date \"2026-13-01\" n'est pas valide."
        );
    }

    #[test]
    fn malformed_macros_and_water_name_the_field_in_french() {
        assert_eq!(
            validation_message(&parse_macro_tenth_g("fat_tenth_g", "12,55").unwrap_err()),
            "\"12,55\" n'est pas valide pour les lipides : indiquez des grammes avec au plus une décimale, par exemple 12,5."
        );
        assert_eq!(
            validation_message(&parse_macro_tenth_g("carbs_tenth_g", "abc").unwrap_err()),
            "\"abc\" n'est pas valide pour les glucides : indiquez des grammes avec au plus une décimale, par exemple 12,5."
        );
        assert_eq!(
            validation_message(&parse_water_ml("1,255").unwrap_err()),
            "\"1,255\" n'est pas une quantité d'eau : indiquez des litres avec au plus deux décimales, par exemple 1,25, ou laissez le champ vide."
        );
    }

    #[test]
    fn database_errors_are_shown_in_french() {
        assert_eq!(
            store_message(&StoreError::UnsupportedSchemaVersion(7)),
            "La base de données utilise une version inconnue (7)."
        );
        assert_eq!(
            store_message(&StoreError::InvalidRow(ValidationError::InvalidDate(
                "x".into()
            ))),
            "La base de données contient une ligne invalide."
        );
        assert_eq!(
            database_unavailable_message(),
            "La base de données est indisponible, redémarrez l'application."
        );
    }

    #[test]
    fn a_duplicated_weigh_in_date_is_explained_in_french() {
        assert_eq!(
            duplicate_date_message(),
            "Deux pesées portent la même date."
        );
    }

    #[test]
    fn an_unknown_field_is_refused_in_french() {
        let msg = validation_message(&ValidationError::OutOfRange {
            field: "unknown",
            value: 1,
            min: 0,
            max: 2,
        });
        assert!(msg.starts_with("La saisie n'est pas acceptée ("), "{msg}");
        assert!(msg.contains("unknown"), "{msg}");
        assert!(msg.ends_with(")."), "{msg}");
    }
}

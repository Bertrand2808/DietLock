//! Values sent to the web view, and user-readable error messages.
//! Pure functions, so they are unit-tested without a running app.

use dietlock_core::{
    Day, Intake, MACRO_G_MAX, ProposedIntake, TREND_WINDOW_DAYS, ValidationError,
    WEIGHT_TENTH_KG_MAX, WEIGHT_TENTH_KG_MIN, WeighIn, alert, display_tenth_kg, energy_kcal,
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
    /// Short French date, e.g. `dim. 25 oct.`.
    pub label: String,
    /// `78,4 kg` when the day has a weigh-in.
    pub weight_label: Option<String>,
    /// Energy, water and a non-fasted note. Macro grams are not included.
    pub detail: Option<String>,
    /// A1–A3: fasted weigh-in at least 3 kg from the previous day's trend.
    pub alert: bool,
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
    /// Big serif line: the trend (`78,5`) or a greeting (`Bonsoir.`). Empty when neither.
    pub primary: String,
    /// `primary` is the trend weight, so the screen adds "kg de tendance".
    pub is_trend: bool,
    /// Line under the headline: kcal, the empty-state invitation, or both.
    pub subtitle: String,
    /// Date proposed for the weigh-in (D1).
    pub weigh_in_date: String,
    /// `Aujourd'hui` or `Hier`.
    pub weigh_in_when: String,
    /// Short French date of the weigh-in, e.g. `lun. 26 oct.`.
    pub weigh_in_label: String,
    /// Date proposed for the intake (D2–D8).
    pub intake_date: String,
    pub intake_when: String,
    pub intake_label: String,
    /// D8 chose yesterday because it has no intake.
    pub forgotten_yesterday: bool,
    /// Every day from today back to the history start, most recent first.
    pub days: Vec<DayView>,
    /// Today's trend as displayed, `None` when there is none yet.
    pub trend: Option<String>,
}

/// `Bonjour.` from 5:00 until 18:00, `Bonsoir.` otherwise.
pub fn greeting(hour: u8) -> &'static str {
    if (5..18).contains(&hour) {
        "Bonjour."
    } else {
        "Bonsoir."
    }
}

/// Whole kcal with a narrow no-break space as the thousands separator.
fn format_kcal(kcal: u32) -> String {
    let digits = kcal.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('\u{202f}');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

/// `dim. 25 oct.`.
pub fn french_date(day: Day) -> String {
    const WEEKDAYS: [&str; 7] = ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."];
    const MONTHS: [&str; 12] = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ];
    let weekday = WEEKDAYS[usize::from(day.weekday_index())];
    let month = MONTHS[usize::from(day.month_number()) - 1];
    format!("{weekday} {} {month}", day.day_number())
}

fn relative_name(date: Day, today: Day) -> &'static str {
    if date == today { "Aujourd'hui" } else { "Hier" }
}

/// Energy and water of the day, plus a note when the weigh-in was not fasted.
/// The macro grams stay out of this line.
fn history_detail(fasted: Option<bool>, intake: Option<&IntakeView>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(i) = intake {
        parts.push(format!("{} kcal", format_kcal(i.energy_kcal)));
        if let Some(water) = &i.water {
            parts.push(format!("eau {water} L"));
        }
    }
    if fasted == Some(false) {
        parts.push("pas à jeun".to_owned());
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn kcal_phrase(today_kcal: Option<u32>, yesterday_kcal: Option<u32>) -> Option<String> {
    match (today_kcal, yesterday_kcal) {
        (Some(kcal), _) => Some(format!("Aujourd'hui · {} kcal", format_kcal(kcal))),
        (None, Some(kcal)) => Some(format!("Hier · {} kcal", format_kcal(kcal))),
        (None, None) => None,
    }
}

impl Listing {
    /// `weigh_ins` may start one trend window before the history, so each
    /// row can be checked for an alert. `intakes` cover the history.
    /// One row per date at most. `trend_hundredth` is `dietlock_core::trend`
    /// of `today`. `hour` is the local hour, used only for the empty greeting.
    pub fn new(
        today: Day,
        hour: u8,
        weigh_in_day: Day,
        proposed_intake: ProposedIntake,
        weigh_ins: &[WeighIn],
        intakes: &[Intake],
        trend_hundredth: Option<u32>,
    ) -> Self {
        let from = history_start(today);
        let has_data = weigh_ins
            .iter()
            .any(|w| w.date() >= from && w.date() <= today)
            || intakes
                .iter()
                .any(|i| i.date() >= from && i.date() <= today);
        let days = (0..HISTORY_DAYS)
            .map_while(|n| today.minus_days(n))
            .map(|d| {
                let weigh_in = weigh_ins.iter().find(|w| w.date() == d);
                let intake = intakes.iter().find(|i| i.date() == d);
                let weigh_view = weigh_in.map(WeighInView::from);
                let intake_view = intake.map(IntakeView::from);
                let alerts = weigh_in.is_some_and(|w| alert(weigh_ins, w).unwrap_or(false));
                DayView {
                    label: french_date(d),
                    weight_label: weigh_view.as_ref().map(|w| format!("{} kg", w.weight)),
                    detail: history_detail(weigh_in.map(|w| w.fasted()), intake_view.as_ref()),
                    alert: alerts,
                    date: d.to_string(),
                    weigh_in: weigh_view,
                    intake: intake_view,
                }
            })
            .collect();
        let trend = trend_hundredth
            .and_then(|h| i32::try_from(h).ok())
            .map(|h| format_tenth_kg(display_tenth_kg(h)));
        let yesterday = today.previous();
        let kcal_on = |day: Day| intakes.iter().find(|i| i.date() == day).map(energy_kcal);
        let today_kcal = kcal_on(today);
        let yesterday_kcal = yesterday.and_then(kcal_on);
        let phrase = kcal_phrase(today_kcal, yesterday_kcal);
        let (primary, is_trend, subtitle) = if let Some(weight) = trend.clone() {
            let subtitle = match phrase {
                Some(kcal) => format!("{kcal} · moyenne à jeun sur 7 jours"),
                None => "Pas encore de saisie · moyenne à jeun sur 7 jours".to_owned(),
            };
            (weight, true, subtitle)
        } else if !has_data {
            (
                greeting(hour).to_owned(),
                false,
                "Pas encore de saisie. La tendance apparaîtra après 4 pesées à jeun.".to_owned(),
            )
        } else {
            let subtitle = match phrase {
                Some(kcal) => format!("{kcal}. La tendance apparaîtra après 4 pesées à jeun."),
                None => {
                    "Pas encore de saisie. La tendance apparaîtra après 4 pesées à jeun.".to_owned()
                }
            };
            (String::new(), false, subtitle)
        };
        Self {
            today: today.to_string(),
            primary,
            is_trend,
            subtitle,
            weigh_in_date: weigh_in_day.to_string(),
            weigh_in_when: relative_name(weigh_in_day, today).to_owned(),
            weigh_in_label: french_date(weigh_in_day),
            intake_date: proposed_intake.date.to_string(),
            intake_when: relative_name(proposed_intake.date, today).to_owned(),
            intake_label: french_date(proposed_intake.date),
            forgotten_yesterday: proposed_intake.forgotten_yesterday,
            days,
            trend,
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
        IntakePresence, SwitchHour, ValidationError, check_not_future, parse_macro_tenth_g,
        parse_water_ml, parse_weight_tenth_kg, proposed_intake_date,
    };
    use dietlock_store::StoreError;
    use time::Time;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    fn weigh(d: &str, w: u16) -> WeighIn {
        WeighIn::new(day(d), w, true).unwrap()
    }

    fn weigh_at(d: &str, w: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(d), w, fasted).unwrap()
    }

    fn home(today: &str, hour: u8, ws: &[WeighIn], is: &[Intake], trend: Option<u32>) -> Listing {
        let today = day(today);
        Listing::new(
            today,
            hour,
            today,
            ProposedIntake {
                date: today,
                forgotten_yesterday: false,
            },
            ws,
            is,
            trend,
        )
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
        let l = home("2026-10-13", 10, &ws, &[], Some(8025));
        assert_eq!(l.today, "2026-10-13");
        assert_eq!(l.weigh_in_date, "2026-10-13");
        assert_eq!(l.weigh_in_when, "Aujourd'hui");
        assert_eq!(l.weigh_in_label, "mar. 13 oct.");
        assert_eq!(l.days[0].date, "2026-10-13");
        assert_eq!(l.days[0].weigh_in.as_ref().unwrap().weight, "82,6");
        assert_eq!(l.trend.as_deref(), Some("80,3"));
        assert!(l.is_trend);
        assert_eq!(l.primary, "80,3");
        assert_eq!(home("2026-10-13", 10, &ws, &[], None).trend, None);
    }

    #[test]
    fn listing_has_every_day_with_weigh_in_and_intake_side_by_side() {
        let ws = [weigh("2026-10-11", 800), weigh("2026-10-13", 826)];
        let is = [intake("2026-10-12", Some(1500)), intake("2026-10-13", None)];
        let l = home("2026-10-13", 10, &ws, &is, None);
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

    fn macros(d: &str, protein: u16, carbs: u16, fat: u16, water: Option<u16>) -> Intake {
        Intake::new(day(d), protein, carbs, fat, water).unwrap()
    }

    #[test]
    fn kcal_line_prefers_today_then_yesterday() {
        let yesterday = macros("2026-10-25", 1650, 1800, 650, Some(2150));
        assert_eq!(energy_kcal(&yesterday), 1965);
        let today = macros("2026-10-26", 1000, 0, 0, None);
        assert_eq!(energy_kcal(&today), 400);
        let hier = home("2026-10-26", 15, &[], &[yesterday], Some(7850));
        assert_eq!(hier.primary, "78,5");
        assert_eq!(
            hier.subtitle,
            "Hier · 1\u{202f}965 kcal · moyenne à jeun sur 7 jours"
        );
        let auj = home(
            "2026-10-26",
            15,
            &[],
            &[macros("2026-10-25", 1650, 1800, 650, Some(2150)), today],
            Some(7850),
        );
        assert_eq!(
            auj.subtitle,
            "Aujourd'hui · 400 kcal · moyenne à jeun sur 7 jours"
        );
        let none = home("2026-10-26", 15, &[], &[], Some(7850));
        assert_eq!(
            none.subtitle,
            "Pas encore de saisie · moyenne à jeun sur 7 jours"
        );
    }

    #[test]
    fn empty_screen_greets_and_invites_an_entry() {
        let evening = home("2026-10-26", 20, &[], &[], None);
        assert_eq!(evening.primary, "Bonsoir.");
        assert!(!evening.is_trend);
        assert_eq!(evening.trend, None);
        assert_eq!(
            evening.subtitle,
            "Pas encore de saisie. La tendance apparaîtra après 4 pesées à jeun."
        );
        assert_eq!(home("2026-10-26", 3, &[], &[], None).primary, "Bonsoir.");
        assert_eq!(home("2026-10-26", 9, &[], &[], None).primary, "Bonjour.");
        assert_eq!(greeting(4), "Bonsoir.");
        assert_eq!(greeting(5), "Bonjour.");
        assert_eq!(greeting(17), "Bonjour.");
        assert_eq!(greeting(18), "Bonsoir.");
    }

    #[test]
    fn at_14h_the_screen_dates_the_intake_yesterday_and_shows_the_flag() {
        let today = day("2026-10-26");
        let proposed = proposed_intake_date(
            today,
            Time::from_hms(14, 0, 0).unwrap(),
            SwitchHour::default(),
            IntakePresence {
                yesterday: false,
                today: true,
            },
        );
        let l = Listing::new(
            today,
            14,
            today,
            proposed,
            &[],
            &[macros("2026-10-26", 1650, 1800, 650, Some(2150))],
            None,
        );
        assert_eq!(proposed.date.to_string(), "2026-10-25");
        assert!(l.forgotten_yesterday);
        assert_eq!(l.weigh_in_when, "Aujourd'hui");
        assert_eq!(l.weigh_in_label, "lun. 26 oct.");
        assert_eq!(l.intake_date, "2026-10-25");
        assert_eq!(l.intake_when, "Hier");
        assert_eq!(l.intake_label, "dim. 25 oct.");
        assert!(l.subtitle.starts_with("Aujourd'hui · 1\u{202f}965 kcal"));
    }

    #[test]
    fn a_filled_yesterday_drops_the_flag_and_proposes_today() {
        let today = day("2026-10-26");
        let proposed = proposed_intake_date(
            today,
            Time::from_hms(14, 0, 0).unwrap(),
            SwitchHour::default(),
            IntakePresence {
                yesterday: true,
                today: true,
            },
        );
        let l = Listing::new(today, 14, today, proposed, &[], &[], None);
        assert!(!proposed.forgotten_yesterday);
        assert_eq!(l.intake_date, "2026-10-26");
        assert_eq!(l.intake_when, "Aujourd'hui");
        assert!(!l.forgotten_yesterday);
    }

    #[test]
    fn history_line_shows_kcal_and_water_without_macro_grams() {
        let intake = macros("2026-10-25", 1650, 1800, 650, Some(2150));
        let l = home(
            "2026-10-26",
            10,
            &[weigh("2026-10-25", 784)],
            &[intake],
            None,
        );
        let row = l.days.iter().find(|d| d.date == "2026-10-25").unwrap();
        assert_eq!(row.weight_label.as_deref(), Some("78,4 kg"));
        assert_eq!(row.label, "dim. 25 oct.");
        let detail = row.detail.as_deref().unwrap();
        assert_eq!(detail, "1\u{202f}965 kcal · eau 2,15 L");
        assert!(!detail.contains("165"), "{detail}");
        assert!(!detail.contains(" g"), "{detail}");
        assert_eq!(row.intake.as_ref().unwrap().protein_g, "165");
        assert!(!row.alert);
    }

    #[test]
    fn a_three_kilogram_gap_is_marked_on_that_day_only() {
        let mut ws: Vec<_> = (12..=15)
            .map(|d| weigh(&format!("2026-10-{d}"), 790))
            .collect();
        ws.push(weigh("2026-10-16", 820));
        ws.push(weigh_at("2026-10-17", 900, false));
        let l = home("2026-10-26", 10, &ws, &[], None);
        let marked = l.days.iter().find(|d| d.date == "2026-10-16").unwrap();
        assert!(marked.alert);
        assert_eq!(marked.weight_label.as_deref(), Some("82,0 kg"));
        assert_eq!(marked.detail, None);
        let late = l.days.iter().find(|d| d.date == "2026-10-17").unwrap();
        assert!(!late.alert);
        assert_eq!(late.detail.as_deref(), Some("pas à jeun"));
        let calm = l.days.iter().find(|d| d.date == "2026-10-15").unwrap();
        assert!(!calm.alert);
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

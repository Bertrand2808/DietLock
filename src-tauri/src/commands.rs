//! IPC commands. Business rules live in `dietlock-core`; this module only
//! reads the clock, locks the store and converts to the view types.

use std::sync::Mutex;

use chrono::Timelike;
use dietlock_core::{
    Day, Intake, IntakePresence, SaveDecision, SwitchHour, TREND_WINDOW_DAYS, WeighIn,
    check_not_future, decide_intake, decide_water, decide_weigh_in, parse_macro_tenth_g,
    parse_water_ml, parse_weight_tenth_kg, proposed_intake_date as core_proposed_intake_date,
    proposed_weigh_in_date, trend,
};
use dietlock_store::{SaveError, Store};
use tauri::State;

use crate::view::{
    self, IntakeSaveOutcome, IntakeView, Listing, SaveOutcome, WeighInView, WeightScreen,
};

/// Current local day, read from the system clock.
fn today() -> Result<Day, String> {
    let s = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    Day::parse(&s).map_err(|e| view::validation_message(&e))
}

/// Current local time of day, to the minute.
fn now() -> Result<time::Time, String> {
    let now = chrono::Local::now();
    // chrono gives hour < 24 and minute < 60, so the conversion cannot fail.
    let to_u8 = |v: u32| u8::try_from(v).map_err(|e| e.to_string());
    time::Time::from_hms(to_u8(now.hour())?, to_u8(now.minute())?, 0).map_err(|e| e.to_string())
}

fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Store>, String> {
    store
        .lock()
        .map_err(|_| view::database_unavailable_message().to_owned())
}

/// Saves a weigh-in typed by the user. The same value writes nothing.
/// A different value on a filled date asks first (S3).
#[tauri::command]
pub fn save_weigh_in(
    store: State<'_, Mutex<Store>>,
    weight_text: String,
    fasted: bool,
    date: String,
    replace: bool,
) -> Result<SaveOutcome, String> {
    let msg = |e: dietlock_core::ValidationError| view::validation_message(&e);
    let date = Day::parse(&date).map_err(msg)?;
    let date = check_not_future(date, today()?).map_err(msg)?;
    let weight = parse_weight_tenth_kg(&weight_text).map_err(msg)?;
    let next = WeighIn::new(date, weight, fasted).map_err(msg)?;
    let mut store = lock(&store)?;
    let existing = store
        .get_weigh_in(date)
        .map_err(|e| view::store_message(&e))?;
    apply_weigh(
        &mut store,
        existing,
        decide_weigh_in(existing, next),
        replace,
    )
}

fn apply_weigh(
    store: &mut Store,
    existing: Option<WeighIn>,
    decision: SaveDecision<WeighIn>,
    replace: bool,
) -> Result<SaveOutcome, String> {
    match decision {
        SaveDecision::Unchanged => Ok(SaveOutcome::Unchanged),
        SaveDecision::Write { confirm: true, .. } if !replace => {
            let old = existing.expect("a weigh-in confirmation only happens when a row is stored");
            Ok(SaveOutcome::AlreadyFilled {
                old: WeighInView::from(&old),
            })
        }
        SaveDecision::Write { value, .. } => match store.save_weigh_in(&value, true) {
            Ok(()) => Ok(SaveOutcome::Saved),
            Err(SaveError::AlreadyFilled(old)) => Ok(SaveOutcome::AlreadyFilled {
                old: WeighInView::from(&old),
            }),
            Err(SaveError::Store(e)) => Err(view::store_message(&e)),
        },
    }
}

/// Saves the intake typed by the user. Blank macros with water update only
/// the water of an existing intake (S3, S4). An empty water text on a full
/// intake saves no water.
#[tauri::command]
pub fn save_intake(
    store: State<'_, Mutex<Store>>,
    protein_text: String,
    carbs_text: String,
    fat_text: String,
    water_text: String,
    date: String,
    replace: bool,
) -> Result<IntakeSaveOutcome, String> {
    let msg = |e: dietlock_core::ValidationError| view::validation_message(&e);
    let date = Day::parse(&date).map_err(msg)?;
    let date = check_not_future(date, today()?).map_err(msg)?;
    let mut store = lock(&store)?;
    let existing = store
        .get_intake(date)
        .map_err(|e| view::store_message(&e))?;
    let blank = |text: &str| text.trim().is_empty();
    let decision = if blank(&protein_text) && blank(&carbs_text) && blank(&fat_text) {
        match parse_water_ml(&water_text).map_err(msg)? {
            None => return Ok(IntakeSaveOutcome::Unchanged),
            Some(ml) => {
                let Some(old) = existing else {
                    return Err(msg(parse_macro_tenth_g("protein_tenth_g", "")
                        .expect_err("an empty macro is not a number")));
                };
                decide_water(old, ml)
            }
        }
    } else {
        let protein = parse_macro_tenth_g("protein_tenth_g", &protein_text).map_err(msg)?;
        let carbs = parse_macro_tenth_g("carbs_tenth_g", &carbs_text).map_err(msg)?;
        let fat = parse_macro_tenth_g("fat_tenth_g", &fat_text).map_err(msg)?;
        let water = parse_water_ml(&water_text).map_err(msg)?;
        let next = Intake::new(date, protein, carbs, fat, water).map_err(msg)?;
        decide_intake(existing, next)
    };
    apply_intake(&mut store, existing, decision, replace)
}

fn apply_intake(
    store: &mut Store,
    existing: Option<Intake>,
    decision: SaveDecision<Intake>,
    replace: bool,
) -> Result<IntakeSaveOutcome, String> {
    match decision {
        SaveDecision::Unchanged => Ok(IntakeSaveOutcome::Unchanged),
        SaveDecision::Write { confirm: true, .. } if !replace => {
            let old = existing.expect("an intake confirmation only happens when a row is stored");
            Ok(IntakeSaveOutcome::AlreadyFilled {
                old: IntakeView::from(&old),
            })
        }
        SaveDecision::Write { value, .. } => match store.save_intake(&value, true) {
            Ok(()) => Ok(IntakeSaveOutcome::Saved),
            Err(SaveError::AlreadyFilled(old)) => Ok(IntakeSaveOutcome::AlreadyFilled {
                old: IntakeView::from(&old),
            }),
            Err(SaveError::Store(e)) => Err(view::store_message(&e)),
        },
    }
}

/// Date proposed for a new intake (D8), from the local time and the intakes
/// of yesterday and today. The switch hour stays at its default, 4 h.
#[tauri::command]
pub fn proposed_intake_date(store: State<'_, Mutex<Store>>) -> Result<String, String> {
    let today = today()?;
    let yesterday = today.previous().unwrap_or(today);
    let store = lock(&store)?;
    let has = |d: Day| {
        store
            .get_intake(d)
            .map(|i| i.is_some())
            .map_err(|e| view::store_message(&e))
    };
    let presence = IntakePresence {
        yesterday: has(yesterday)?,
        today: has(today)?,
    };
    let p = core_proposed_intake_date(today, now()?, SwitchHour::default(), presence);
    Ok(p.date.to_string())
}

/// Home screen: trend, the dates proposed for the weigh-in (D1) and the
/// intake (D2–D8), and the last [`view::HISTORY_DAYS`] days.
///
/// Weigh-ins start one trend window before the history, so an alert on the
/// oldest visible day can still see the previous day's trend (A1). The
/// proposed intake date comes from `dietlock_core`; this function does not
/// reimplement it.
#[tauri::command]
pub fn list_days(store: State<'_, Mutex<Store>>) -> Result<Listing, String> {
    let today = today()?;
    let now = now()?;
    let from = view::history_start(today);
    let weigh_from = from.minus_days(TREND_WINDOW_DAYS).unwrap_or(from);
    let store = lock(&store)?;
    let weigh_ins = store
        .list_weigh_ins(weigh_from, today)
        .map_err(|e| view::store_message(&e))?;
    let intakes = store
        .list_intakes(from, today)
        .map_err(|e| view::store_message(&e))?;
    // The history covers the trend window, and the store holds one row per date.
    let trend = trend(&weigh_ins, today).map_err(|_| view::duplicate_date_message().to_owned())?;
    let yesterday = today.previous().unwrap_or(today);
    let presence = IntakePresence {
        yesterday: intakes.iter().any(|i| i.date() == yesterday),
        today: intakes.iter().any(|i| i.date() == today),
    };
    let proposed = core_proposed_intake_date(today, now, SwitchHour::default(), presence);
    Ok(Listing::new(
        today,
        now.hour(),
        proposed_weigh_in_date(today),
        proposed,
        &weigh_ins,
        &intakes,
        trend,
    ))
}

/// Weight screen: one point per weigh-in and the trend of each day, over 30 days.
/// The 15-day window is the same series, trimmed. The trend is `dietlock_core::trend`.
#[tauri::command]
pub fn weight_curve(store: State<'_, Mutex<Store>>) -> Result<WeightScreen, String> {
    let today = today()?;
    let store = lock(&store)?;
    // Every saved weigh-in. The empty and single notices count the whole history,
    // and a trend at the left edge of the chart reads six days before that edge.
    let from = Day::parse("1000-01-01").map_err(|e| view::validation_message(&e))?;
    let weigh_ins = store
        .list_weigh_ins(from, today)
        .map_err(|e| view::store_message(&e))?;
    WeightScreen::new(today, &weigh_ins).map_err(|_| view::duplicate_date_message().to_owned())
}

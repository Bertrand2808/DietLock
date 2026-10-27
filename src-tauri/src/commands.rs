//! IPC commands. Business rules live in `dietlock-core`; this module only
//! reads the clock, locks the store and converts to the view types.

use std::sync::Mutex;

use chrono::Timelike;
use dietlock_core::{
    Day, Intake, IntakePresence, SwitchHour, WeighIn, check_not_future, parse_macro_tenth_g,
    parse_water_ml, parse_weight_tenth_kg, proposed_weigh_in_date, trend,
};
use dietlock_store::{SaveError, Store};
use tauri::State;

use crate::view::{self, IntakeSaveOutcome, IntakeView, Listing, SaveOutcome, WeighInView};

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

/// Saves a weigh-in typed by the user. With `replace` false and the date
/// already filled, nothing is written and the previous weigh-in is returned
/// so the UI can ask for confirmation (S3).
#[tauri::command]
pub fn save_weigh_in(
    store: State<'_, Mutex<Store>>,
    weight_text: String,
    fasted: bool,
    date: String,
    replace: bool,
) -> Result<SaveOutcome, String> {
    let date = Day::parse(&date).map_err(|e| view::validation_message(&e))?;
    let date = check_not_future(date, today()?).map_err(|e| view::validation_message(&e))?;
    let weight = parse_weight_tenth_kg(&weight_text).map_err(|e| view::validation_message(&e))?;
    let w = WeighIn::new(date, weight, fasted).map_err(|e| view::validation_message(&e))?;
    match lock(&store)?.save_weigh_in(&w, replace) {
        Ok(()) => Ok(SaveOutcome::Saved),
        Err(SaveError::AlreadyFilled(old)) => Ok(SaveOutcome::AlreadyFilled {
            old: WeighInView::from(&old),
        }),
        Err(SaveError::Store(e)) => Err(view::store_message(&e)),
    }
}

/// Saves the macros typed by the user; an empty water text saves no water.
/// Same confirmation pattern as [`save_weigh_in`].
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
    let protein = parse_macro_tenth_g("protein_tenth_g", &protein_text).map_err(msg)?;
    let carbs = parse_macro_tenth_g("carbs_tenth_g", &carbs_text).map_err(msg)?;
    let fat = parse_macro_tenth_g("fat_tenth_g", &fat_text).map_err(msg)?;
    let water = parse_water_ml(&water_text).map_err(msg)?;
    let i = Intake::new(date, protein, carbs, fat, water).map_err(msg)?;
    match lock(&store)?.save_intake(&i, replace) {
        Ok(()) => Ok(IntakeSaveOutcome::Saved),
        Err(SaveError::AlreadyFilled(old)) => Ok(IntakeSaveOutcome::AlreadyFilled {
            old: IntakeView::from(&old),
        }),
        Err(SaveError::Store(e)) => Err(view::store_message(&e)),
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
    let p = dietlock_core::proposed_intake_date(today, now()?, SwitchHour::default(), presence);
    Ok(p.date.to_string())
}

/// Weigh-in and intake of each of the last [`view::HISTORY_DAYS`] days,
/// today's trend and the date proposed for a new weigh-in (D1).
#[tauri::command]
pub fn list_days(store: State<'_, Mutex<Store>>) -> Result<Listing, String> {
    let today = today()?;
    let from = view::history_start(today);
    let store = lock(&store)?;
    let weigh_ins = store
        .list_weigh_ins(from, today)
        .map_err(|e| view::store_message(&e))?;
    let intakes = store
        .list_intakes(from, today)
        .map_err(|e| view::store_message(&e))?;
    // The history covers the trend window, and the store holds one row per date.
    let trend = trend(&weigh_ins, today).map_err(|_| view::duplicate_date_message().to_owned())?;
    Ok(Listing::new(
        today,
        proposed_weigh_in_date(today),
        &weigh_ins,
        &intakes,
        trend,
    ))
}

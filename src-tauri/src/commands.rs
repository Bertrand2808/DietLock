//! IPC commands. Business rules live in `dietlock-core`; this module only
//! reads the clock, locks the store and converts to the view types.

use std::sync::Mutex;

use dietlock_core::{
    Day, WeighIn, check_not_future, parse_weight_tenth_kg, proposed_weigh_in_date, trend,
};
use dietlock_store::{SaveError, Store};
use tauri::State;

use crate::view::{self, Listing, SaveOutcome, WeighInView};

/// Current local day, read from the system clock.
fn today() -> Result<Day, String> {
    let s = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    Day::parse(&s).map_err(|e| view::validation_message(&e))
}

fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Store>, String> {
    store
        .lock()
        .map_err(|_| "The database is unavailable, restart the app.".to_owned())
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

/// Weigh-ins of the last [`view::HISTORY_DAYS`] days, today's trend and the
/// date proposed for a new weigh-in (D1).
#[tauri::command]
pub fn list_weigh_ins(store: State<'_, Mutex<Store>>) -> Result<Listing, String> {
    let today = today()?;
    let from = view::history_start(today);
    let weigh_ins = lock(&store)?
        .list_weigh_ins(from, today)
        .map_err(|e| view::store_message(&e))?;
    // The history covers the trend window, and the store holds one row per date.
    let trend = trend(&weigh_ins, today).map_err(|e| e.to_string())?;
    Ok(Listing::new(
        today,
        proposed_weigh_in_date(today),
        &weigh_ins,
        trend,
    ))
}

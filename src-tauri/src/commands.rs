//! IPC commands. Business rules live in `dietlock-core`; this module only
//! reads the clock, locks the store and converts to the view types.

use std::sync::Mutex;

use chrono::Timelike;
use dietlock_core::{
    ActivityDraft, Day, Intake, IntakePresence, SaveDecision, SwitchHour, TREND_WINDOW_DAYS,
    WeighIn, build_activities, check_not_future, decide_intake, decide_water, decide_weigh_in,
    parse_macro_tenth_g, parse_water_ml, parse_weight_tenth_kg,
    proposed_intake_date as core_proposed_intake_date, proposed_weigh_in_date, trend,
};
use dietlock_store::{SaveError, Store};
use tauri::State;

use crate::view::{
    self, ActivityDraftInput, ActivitySaved, DeleteBlock, DeleteOutcome, DeleteTarget,
    IntakeSaveOutcome, IntakeView, Listing, SaveOutcome, WeighInView, WeightScreen,
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
    // The history starts 29 days back, so it covers both energy weeks (J−14 to yesterday).
    Listing::new(
        today,
        now.hour(),
        proposed_weigh_in_date(today),
        proposed,
        &weigh_ins,
        &intakes,
        trend,
    )
    .map_err(|_| view::duplicate_intake_message().to_owned())
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

/// Deletes one block of a day after confirmation (S10). Without `confirm`,
/// returns the stored block and writes nothing. A missing block returns
/// `NothingToDelete` either way. The trend and the alerts
/// are computed from the stored weigh-ins on every read (T5, A4).
#[tauri::command]
pub fn delete_entry(
    store: State<'_, Mutex<Store>>,
    date: String,
    block: DeleteBlock,
    confirm: bool,
) -> Result<DeleteOutcome, String> {
    let date = Day::parse(&date).map_err(|e| view::validation_message(&e))?;
    let store = lock(&store)?;
    apply_delete(&store, date, block, confirm)
}

fn apply_delete(
    store: &Store,
    date: Day,
    block: DeleteBlock,
    confirm: bool,
) -> Result<DeleteOutcome, String> {
    let fail = |e: &dietlock_store::StoreError| view::store_message(e);
    let target = match block {
        DeleteBlock::WeighIn => store
            .get_weigh_in(date)
            .map_err(|e| fail(&e))?
            .map(|w| DeleteTarget::WeighIn(WeighInView::from(&w))),
        DeleteBlock::Intake => store
            .get_intake(date)
            .map_err(|e| fail(&e))?
            .map(|i| DeleteTarget::Intake(IntakeView::from(&i))),
    };
    let Some(target) = target else {
        return Ok(DeleteOutcome::NothingToDelete);
    };
    if !confirm {
        return Ok(DeleteOutcome::Confirm { target });
    }
    match block {
        DeleteBlock::WeighIn => store.delete_weigh_in(date),
        DeleteBlock::Intake => store.delete_intake(date),
    }
    .map_err(|e| fail(&e))?;
    Ok(DeleteOutcome::Deleted)
}

/// Saves every typed row of the activity form for one date, appending to the
/// activities already stored (AC5). The date and all rows are validated first,
/// so a refusal writes nothing (AC4, D5). Weigh-in and intake are not touched.
#[tauri::command]
pub fn save_activities(
    store: State<'_, Mutex<Store>>,
    date: String,
    drafts: Vec<ActivityDraftInput>,
) -> Result<ActivitySaved, String> {
    let today = today()?;
    let mut store = lock(&store)?;
    apply_save_activities(&mut store, today, &date, drafts)
}

fn apply_save_activities(
    store: &mut Store,
    today: Day,
    date: &str,
    drafts: Vec<ActivityDraftInput>,
) -> Result<ActivitySaved, String> {
    let date = Day::parse(date).map_err(|e| view::validation_message(&e))?;
    let drafts: Vec<ActivityDraft> = drafts.into_iter().map(ActivityDraft::from).collect();
    let activities =
        build_activities(date, today, &drafts).map_err(|e| view::validation_message(&e))?;
    store
        .add_activities(&activities)
        .map_err(|e| view::store_message(&e))?;
    Ok(ActivitySaved {
        count: activities.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dietlock_core::ActivityKind;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    fn input(kind: Option<&str>, note: &str) -> ActivityDraftInput {
        ActivityDraftInput {
            kind: kind.map(str::to_owned),
            note: note.to_owned(),
        }
    }

    fn stored(s: &Store, d: &str) -> Vec<(ActivityKind, Option<String>)> {
        s.list_activities(day(d), day(d))
            .unwrap()
            .iter()
            .map(|a| (a.kind(), a.note().map(str::to_owned)))
            .collect()
    }

    #[test]
    fn ac3_validating_saves_every_typed_row_for_the_chosen_date() {
        let mut s = filled_store();
        let out = apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "2026-10-13",
            vec![
                input(Some("walk"), "chien"),
                input(None, ""),
                input(Some("strength"), ""),
            ],
        )
        .unwrap();
        assert_eq!(out, ActivitySaved { count: 2 });
        assert_eq!(
            stored(&s, "2026-10-13"),
            [
                (ActivityKind::Walk, Some("chien".to_owned())),
                (ActivityKind::Strength, None)
            ]
        );
        assert!(stored(&s, "2026-10-14").is_empty());
    }

    #[test]
    fn ac5_weight_and_macros_of_the_day_are_unchanged() {
        let mut s = filled_store();
        let w = s.get_weigh_in(day("2026-10-13")).unwrap();
        let i = s.get_intake(day("2026-10-13")).unwrap();
        apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "2026-10-13",
            vec![input(Some("run"), "")],
        )
        .unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), w);
        assert_eq!(s.get_intake(day("2026-10-13")).unwrap(), i);
    }

    #[test]
    fn ac4_a_refused_form_writes_nothing_and_answers_in_french() {
        let mut s = filled_store();
        let err = apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "2026-10-13",
            vec![input(Some("run"), ""), input(None, "oublié")],
        )
        .unwrap_err();
        assert_eq!(err, "Choisis un type pour chaque activité.");
        let err =
            apply_save_activities(&mut s, day("2026-10-14"), "2026-10-13", vec![]).unwrap_err();
        assert_eq!(err, "Choisis au moins un type d'activité.");
        assert!(stored(&s, "2026-10-13").is_empty());
    }

    #[test]
    fn d5_a_future_date_is_refused_and_today_is_accepted() {
        let mut s = filled_store();
        let err = apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "2026-10-15",
            vec![input(Some("run"), "")],
        )
        .unwrap_err();
        assert_eq!(
            err,
            "jeu. 15 oct. est dans le futur, choisis aujourd'hui ou avant."
        );
        assert!(stored(&s, "2026-10-15").is_empty());
        apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "2026-10-14",
            vec![input(Some("run"), "")],
        )
        .unwrap();
        assert_eq!(stored(&s, "2026-10-14").len(), 1);
    }

    #[test]
    fn an_invalid_date_text_is_refused_in_french() {
        let mut s = filled_store();
        let err = apply_save_activities(
            &mut s,
            day("2026-10-14"),
            "demain",
            vec![input(Some("run"), "")],
        )
        .unwrap_err();
        assert_eq!(err, "La date \"demain\" n'est pas valide.");
    }

    #[test]
    fn ac5_saving_again_for_the_same_date_appends() {
        let mut s = filled_store();
        for kind in ["walk", "bike"] {
            apply_save_activities(
                &mut s,
                day("2026-10-14"),
                "2026-10-14",
                vec![input(Some(kind), "")],
            )
            .unwrap();
        }
        let kinds: Vec<ActivityKind> = stored(&s, "2026-10-14")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(kinds, [ActivityKind::Walk, ActivityKind::Bike]);
    }

    fn filled_store() -> Store {
        let mut s = Store::open_in_memory().unwrap();
        for d in ["2026-10-13", "2026-10-14"] {
            s.save_weigh_in(&WeighIn::new(day(d), 826, true).unwrap(), false)
                .unwrap();
            s.save_intake(
                &Intake::new(day(d), 1500, 2000, 700, Some(2000)).unwrap(),
                false,
            )
            .unwrap();
        }
        s
    }

    #[test]
    fn s10_asking_to_delete_shows_the_stored_weigh_in_and_writes_nothing() {
        let s = filled_store();
        let out = apply_delete(&s, day("2026-10-13"), DeleteBlock::WeighIn, false).unwrap();
        let DeleteOutcome::Confirm {
            target: DeleteTarget::WeighIn(w),
        } = out
        else {
            panic!("expected a weigh-in confirmation, got {out:?}");
        };
        assert_eq!((w.date.as_str(), w.weight.as_str()), ("2026-10-13", "82,6"));
        assert!(s.get_weigh_in(day("2026-10-13")).unwrap().is_some());
    }

    #[test]
    fn s10_asking_to_delete_shows_macros_and_water_together() {
        let s = filled_store();
        let out = apply_delete(&s, day("2026-10-13"), DeleteBlock::Intake, false).unwrap();
        let DeleteOutcome::Confirm {
            target: DeleteTarget::Intake(i),
        } = out
        else {
            panic!("expected an intake confirmation, got {out:?}");
        };
        assert_eq!(i.protein_g, "150");
        assert_eq!(i.water.as_deref(), Some("2"));
        assert!(s.get_intake(day("2026-10-13")).unwrap().is_some());
    }

    #[test]
    fn s10_confirmed_weigh_in_deletion_leaves_the_intake_and_other_dates() {
        let s = filled_store();
        let out = apply_delete(&s, day("2026-10-13"), DeleteBlock::WeighIn, true).unwrap();
        assert_eq!(out, DeleteOutcome::Deleted);
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), None);
        assert!(s.get_intake(day("2026-10-13")).unwrap().is_some());
        assert!(s.get_weigh_in(day("2026-10-14")).unwrap().is_some());
    }

    #[test]
    fn s10_confirmed_intake_deletion_removes_water_too_and_keeps_the_weigh_in() {
        let s = filled_store();
        let out = apply_delete(&s, day("2026-10-14"), DeleteBlock::Intake, true).unwrap();
        assert_eq!(out, DeleteOutcome::Deleted);
        assert_eq!(s.get_intake(day("2026-10-14")).unwrap(), None);
        assert!(s.get_weigh_in(day("2026-10-14")).unwrap().is_some());
        assert!(s.get_intake(day("2026-10-13")).unwrap().is_some());
    }

    #[test]
    fn s10_deleting_a_missing_block_reports_it_and_asks_nothing() {
        let s = filled_store();
        for confirm in [false, true] {
            let out = apply_delete(&s, day("2026-10-20"), DeleteBlock::WeighIn, confirm).unwrap();
            assert_eq!(out, DeleteOutcome::NothingToDelete);
        }
    }

    #[test]
    fn t5_a_deleted_weigh_in_leaves_the_trend_window() {
        let mut s = Store::open_in_memory().unwrap();
        for (d, w) in [
            ("2026-10-10", 800),
            ("2026-10-11", 800),
            ("2026-10-12", 800),
            ("2026-10-13", 900),
        ] {
            s.save_weigh_in(&WeighIn::new(day(d), w, true).unwrap(), false)
                .unwrap();
        }
        let list = |s: &Store| {
            s.list_weigh_ins(day("2026-10-01"), day("2026-10-13"))
                .unwrap()
        };
        assert_eq!(trend(&list(&s), day("2026-10-13")).unwrap(), Some(8250));
        apply_delete(&s, day("2026-10-13"), DeleteBlock::WeighIn, true).unwrap();
        assert_eq!(trend(&list(&s), day("2026-10-13")).unwrap(), None);
    }
}

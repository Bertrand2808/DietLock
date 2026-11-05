//! What one save does with a weigh-in or an intake (S1, S3).

use crate::intake::Intake;
use crate::weigh_in::WeighIn;

/// Decision for a single block. `confirm` is true only when the write replaces
/// a value the user already entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveDecision<T> {
    /// The typed value matches the stored entry.
    Unchanged,
    /// Persist `value`. Ask first when `confirm` is set.
    Write { value: T, confirm: bool },
}

pub fn decide_weigh_in(existing: Option<WeighIn>, next: WeighIn) -> SaveDecision<WeighIn> {
    match existing {
        Some(old) if old == next => SaveDecision::Unchanged,
        Some(_) => SaveDecision::Write {
            value: next,
            confirm: true,
        },
        None => SaveDecision::Write {
            value: next,
            confirm: false,
        },
    }
}

pub fn decide_intake(existing: Option<Intake>, next: Intake) -> SaveDecision<Intake> {
    let Some(old) = existing else {
        return SaveDecision::Write {
            value: next,
            confirm: false,
        };
    };
    if old == next {
        return SaveDecision::Unchanged;
    }
    let same_macros = old.protein_tenth_g() == next.protein_tenth_g()
        && old.carbs_tenth_g() == next.carbs_tenth_g()
        && old.fat_tenth_g() == next.fat_tenth_g();
    let adding_water = same_macros && old.water_ml().is_none() && next.water_ml().is_some();
    SaveDecision::Write {
        value: next,
        confirm: !adding_water,
    }
}

pub fn decide_water(existing: Intake, water_ml: u16) -> SaveDecision<Intake> {
    if existing.water_ml() == Some(water_ml) {
        return SaveDecision::Unchanged;
    }
    let value = Intake::new(
        existing.date(),
        existing.protein_tenth_g(),
        existing.carbs_tenth_g(),
        existing.fat_tenth_g(),
        Some(water_ml),
    )
    .expect("stored macros and a parsed water amount stay inside their ranges");
    SaveDecision::Write {
        value,
        confirm: existing.water_ml().is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    fn weigh(d: &str, tenth: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(d), tenth, fasted).unwrap()
    }

    fn intake(d: &str, water: Option<u16>) -> Intake {
        Intake::new(day(d), 1460, 1900, 940, water).unwrap()
    }

    #[test]
    fn the_same_weigh_in_is_not_a_replacement() {
        let stored = weigh("2026-11-05", 1079, true);
        assert_eq!(
            decide_weigh_in(Some(stored), stored),
            SaveDecision::Unchanged
        );
    }

    #[test]
    fn a_different_weight_still_asks() {
        let stored = weigh("2026-11-05", 1079, true);
        let next = weigh("2026-11-05", 1081, true);
        assert_eq!(
            decide_weigh_in(Some(stored), next),
            SaveDecision::Write {
                value: next,
                confirm: true
            }
        );
    }

    #[test]
    fn changing_only_the_fasted_flag_asks() {
        let stored = weigh("2026-11-05", 1079, true);
        let next = weigh("2026-11-05", 1079, false);
        assert_eq!(
            decide_weigh_in(Some(stored), next),
            SaveDecision::Write {
                value: next,
                confirm: true
            }
        );
    }

    #[test]
    fn the_first_weigh_in_does_not_ask() {
        let next = weigh("2026-11-05", 1079, true);
        assert_eq!(
            decide_weigh_in(None, next),
            SaveDecision::Write {
                value: next,
                confirm: false
            }
        );
    }

    #[test]
    fn the_same_intake_is_not_a_replacement() {
        let stored = intake("2026-11-03", Some(3100));
        assert_eq!(decide_intake(Some(stored), stored), SaveDecision::Unchanged);
    }

    #[test]
    fn different_macros_still_ask() {
        let stored = intake("2026-11-03", Some(3100));
        let next = Intake::new(day("2026-11-03"), 1500, 1900, 940, Some(3100)).unwrap();
        assert_eq!(
            decide_intake(Some(stored), next),
            SaveDecision::Write {
                value: next,
                confirm: true
            }
        );
    }

    #[test]
    fn adding_water_beside_the_same_macros_does_not_ask() {
        let stored = intake("2026-11-03", None);
        let next = intake("2026-11-03", Some(3100));
        assert_eq!(
            decide_intake(Some(stored), next),
            SaveDecision::Write {
                value: next,
                confirm: false
            }
        );
    }

    #[test]
    fn clearing_entered_water_still_asks() {
        let stored = intake("2026-11-03", Some(3100));
        let next = intake("2026-11-03", None);
        assert_eq!(
            decide_intake(Some(stored), next),
            SaveDecision::Write {
                value: next,
                confirm: true
            }
        );
    }

    #[test]
    fn the_first_intake_does_not_ask() {
        let next = intake("2026-11-03", Some(3100));
        assert_eq!(
            decide_intake(None, next),
            SaveDecision::Write {
                value: next,
                confirm: false
            }
        );
    }

    #[test]
    fn water_alone_keeps_the_macros_and_does_not_ask_when_none_was_stored() {
        let stored = intake("2026-11-03", None);
        let next = intake("2026-11-03", Some(3100));
        assert_eq!(
            decide_water(stored, 3100),
            SaveDecision::Write {
                value: next,
                confirm: false
            }
        );
    }

    #[test]
    fn replacing_stored_water_asks_and_keeps_the_macros() {
        let stored = intake("2026-11-03", Some(3100));
        let next = intake("2026-11-03", Some(2000));
        assert_eq!(
            decide_water(stored, 2000),
            SaveDecision::Write {
                value: next,
                confirm: true
            }
        );
    }

    #[test]
    fn the_same_water_is_unchanged() {
        let stored = intake("2026-11-03", Some(3100));
        assert_eq!(decide_water(stored, 3100), SaveDecision::Unchanged);
    }
}

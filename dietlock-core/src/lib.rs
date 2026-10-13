//! Pure business logic of DietLock: no dependency on Tauri, SQLite or the clock.

mod day;
mod energy;
mod error;
mod input;
mod intake;
mod proposed_date;
mod weigh_in;

pub use day::Day;
pub use energy::energy_kcal;
pub use error::ValidationError;
pub use input::{parse_macro_g, parse_water_ml, parse_weight_tenth_kg};
pub use intake::{Intake, MACRO_G_MAX, WATER_ML_MAX};
pub use proposed_date::{
    RolloverHour, check_not_future, proposed_intake_date, proposed_weigh_in_date,
};
pub use weigh_in::{WEIGHT_TENTH_KG_MAX, WEIGHT_TENTH_KG_MIN, WeighIn};

/// Application name.
pub fn app_name() -> &'static str {
    "DietLock"
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    #[test]
    fn app_name_is_dietlock() {
        assert_eq!(app_name(), "DietLock");
    }
}

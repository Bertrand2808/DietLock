//! Pure business logic of DietLock: no dependency on Tauri, SQLite or the clock.

mod day;
mod error;
mod intake;
mod weigh_in;

pub use day::Day;
pub use error::ValidationError;
pub use intake::{Intake, MACRO_G_MAX, WATER_ML_MAX};
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

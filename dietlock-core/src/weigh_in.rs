use crate::day::Day;
use crate::error::{ValidationError, check_range};

/// Minimum accepted weight, in tenths of a kg (30.0 kg).
pub const WEIGHT_TENTH_KG_MIN: u16 = 300;
/// Maximum accepted weight, in tenths of a kg (250.0 kg).
pub const WEIGHT_TENTH_KG_MAX: u16 = 2500;

/// Weigh-in of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeighIn {
    date: Day,
    weight_tenth_kg: u16,
    fasted: bool,
}

impl WeighIn {
    pub fn new(date: Day, weight_tenth_kg: u16, fasted: bool) -> Result<Self, ValidationError> {
        check_range(
            "weight_tenth_kg",
            weight_tenth_kg,
            WEIGHT_TENTH_KG_MIN,
            WEIGHT_TENTH_KG_MAX,
        )?;
        Ok(Self {
            date,
            weight_tenth_kg,
            fasted,
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn weight_tenth_kg(&self) -> u16 {
        self.weight_tenth_kg
    }

    pub fn fasted(&self) -> bool {
        self.fasted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    #[test]
    fn weight_bounds() {
        let d = day("2026-10-12");
        assert!(WeighIn::new(d, 300, true).is_ok());
        assert!(WeighIn::new(d, 2500, false).is_ok());
        assert!(WeighIn::new(d, 299, true).is_err());
        assert!(WeighIn::new(d, 2501, true).is_err());
    }
}

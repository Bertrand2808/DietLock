use crate::day::Day;
use crate::error::{ValidationError, check_range};

/// Maximum value of a macro, in grams. Stored values use [`MACRO_TENTH_G_MAX`].
pub const MACRO_G_MAX: u16 = 1000;
/// Maximum value of a macro, in tenths of a gram (1_000.0 g).
pub const MACRO_TENTH_G_MAX: u16 = MACRO_G_MAX * 10;
/// Maximum water, in millilitres.
pub const WATER_ML_MAX: u16 = 10_000;

/// Intake of a day. Macros are tenths of a gram. Water is millilitres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intake {
    date: Day,
    protein_tenth_g: u16,
    carbs_tenth_g: u16,
    fat_tenth_g: u16,
    water_ml: Option<u16>,
}

impl Intake {
    pub fn new(
        date: Day,
        protein_tenth_g: u16,
        carbs_tenth_g: u16,
        fat_tenth_g: u16,
        water_ml: Option<u16>,
    ) -> Result<Self, ValidationError> {
        check_range("protein_tenth_g", protein_tenth_g, 0, MACRO_TENTH_G_MAX)?;
        check_range("carbs_tenth_g", carbs_tenth_g, 0, MACRO_TENTH_G_MAX)?;
        check_range("fat_tenth_g", fat_tenth_g, 0, MACRO_TENTH_G_MAX)?;
        if let Some(w) = water_ml {
            check_range("water_ml", w, 0, WATER_ML_MAX)?;
        }
        Ok(Self {
            date,
            protein_tenth_g,
            carbs_tenth_g,
            fat_tenth_g,
            water_ml,
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn protein_tenth_g(&self) -> u16 {
        self.protein_tenth_g
    }

    pub fn carbs_tenth_g(&self) -> u16 {
        self.carbs_tenth_g
    }

    pub fn fat_tenth_g(&self) -> u16 {
        self.fat_tenth_g
    }

    pub fn water_ml(&self) -> Option<u16> {
        self.water_ml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    #[test]
    fn intake_bounds() {
        let d = day("2026-10-12");
        assert!(Intake::new(d, 0, 10_000, 0, Some(10_000)).is_ok());
        assert!(Intake::new(d, 10_001, 0, 0, None).is_err());
        assert!(Intake::new(d, 0, 10_001, 0, None).is_err());
        assert!(Intake::new(d, 0, 0, 10_001, None).is_err());
        assert!(Intake::new(d, 0, 0, 0, Some(10_001)).is_err());
    }
}

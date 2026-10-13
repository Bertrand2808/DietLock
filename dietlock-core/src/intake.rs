use crate::day::Day;
use crate::error::{ValidationError, check_range};

/// Maximum value of a macro, in grams.
pub const MACRO_G_MAX: u16 = 1000;
/// Maximum water, in millilitres.
pub const WATER_ML_MAX: u16 = 10_000;

/// Intake of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intake {
    date: Day,
    protein_g: u16,
    carbs_g: u16,
    fat_g: u16,
    water_ml: Option<u16>,
}

impl Intake {
    pub fn new(
        date: Day,
        protein_g: u16,
        carbs_g: u16,
        fat_g: u16,
        water_ml: Option<u16>,
    ) -> Result<Self, ValidationError> {
        check_range("protein_g", protein_g, 0, MACRO_G_MAX)?;
        check_range("carbs_g", carbs_g, 0, MACRO_G_MAX)?;
        check_range("fat_g", fat_g, 0, MACRO_G_MAX)?;
        if let Some(w) = water_ml {
            check_range("water_ml", w, 0, WATER_ML_MAX)?;
        }
        Ok(Self {
            date,
            protein_g,
            carbs_g,
            fat_g,
            water_ml,
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn protein_g(&self) -> u16 {
        self.protein_g
    }

    pub fn carbs_g(&self) -> u16 {
        self.carbs_g
    }

    pub fn fat_g(&self) -> u16 {
        self.fat_g
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
        assert!(Intake::new(d, 0, 1000, 0, Some(10_000)).is_ok());
        assert!(Intake::new(d, 1001, 0, 0, None).is_err());
        assert!(Intake::new(d, 0, 1001, 0, None).is_err());
        assert!(Intake::new(d, 0, 0, 1001, None).is_err());
        assert!(Intake::new(d, 0, 0, 0, Some(10_001)).is_err());
    }
}

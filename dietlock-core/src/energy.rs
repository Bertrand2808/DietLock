use crate::intake::Intake;

/// Energy of an intake, in kcal: 4 per g of protein and carbs, 9 per g of fat.
/// Water is not counted. Never stored: always recomputed from the macros.
pub fn energy_kcal(intake: &Intake) -> u32 {
    4 * u32::from(intake.protein_g())
        + 4 * u32::from(intake.carbs_g())
        + 9 * u32::from(intake.fat_g())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    #[test]
    fn energy_of_macros() {
        let d = day("2026-10-13");
        let i = Intake::new(d, 150, 250, 70, Some(1500)).unwrap();
        assert_eq!(energy_kcal(&i), 2230);
        let zero = Intake::new(d, 0, 0, 0, None).unwrap();
        assert_eq!(energy_kcal(&zero), 0);
    }

    #[test]
    fn energy_max_fits() {
        let i = Intake::new(day("2026-10-13"), 1000, 1000, 1000, None).unwrap();
        assert_eq!(energy_kcal(&i), 17_000);
    }
}

use crate::intake::Intake;

/// S8: energy of an intake, in kcal. 4 per gram of protein and carbs, 9 per gram of fat.
/// Macros are tenths of a gram, so the total is divided by 10 and rounded half away
/// from zero. The inputs are never negative, which makes that the same as half up.
/// Water is not counted. Never stored: always recomputed from the macros.
pub fn energy_kcal(intake: &Intake) -> u32 {
    let tenths = 4 * u32::from(intake.protein_tenth_g())
        + 4 * u32::from(intake.carbs_tenth_g())
        + 9 * u32::from(intake.fat_tenth_g());
    (tenths + 5) / 10
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::parse_macro_tenth_g;
    use crate::tests::day;

    #[test]
    fn s8_half_gram_of_fat_is_5_kcal() {
        let d = day("2026-10-13");
        let fat = parse_macro_tenth_g("fat_tenth_g", "0,5").unwrap();
        let i = Intake::new(d, 0, 0, fat, None).unwrap();
        assert_eq!(energy_kcal(&i), 5);
    }

    #[test]
    fn s8_energy_of_macros() {
        let d = day("2026-10-13");
        let i = Intake::new(d, 1500, 2500, 700, Some(1500)).unwrap();
        assert_eq!(energy_kcal(&i), 2230);
        let zero = Intake::new(d, 0, 0, 0, None).unwrap();
        assert_eq!(energy_kcal(&zero), 0);
    }

    #[test]
    fn s8_energy_max_fits() {
        let i = Intake::new(day("2026-10-13"), 10_000, 10_000, 10_000, None).unwrap();
        assert_eq!(energy_kcal(&i), 17_000);
    }
}

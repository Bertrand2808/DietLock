use dietlock_core::{Day, Intake};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{SaveError, StoreError};

type RawIntake = (String, u16, u16, u16, Option<u16>);

fn raw_intake(r: &Row<'_>) -> rusqlite::Result<RawIntake> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
}

fn to_intake((d, p, c, f, w): RawIntake) -> Result<Intake, StoreError> {
    let day = Day::parse(&d).map_err(StoreError::InvalidRow)?;
    Intake::new(day, p, c, f, w).map_err(StoreError::InvalidRow)
}

pub(crate) fn get_intake(conn: &Connection, date: Day) -> Result<Option<Intake>, StoreError> {
    conn.query_row(
        "SELECT date, protein_g, carbs_g, fat_g, water_ml FROM intake WHERE date = ?1",
        [date.to_string()],
        raw_intake,
    )
    .optional()?
    .map(to_intake)
    .transpose()
}

/// Intakes from `from` to `to` inclusive, in ascending date order.
pub(crate) fn list_intakes(
    conn: &Connection,
    from: Day,
    to: Day,
) -> Result<Vec<Intake>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT date, protein_g, carbs_g, fat_g, water_ml FROM intake
         WHERE date BETWEEN ?1 AND ?2 ORDER BY date",
    )?;
    let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_intake)?;
    rows.map(|r| to_intake(r?)).collect()
}

/// Saves an intake. If the date is already filled and `replace` is
/// false, returns [`SaveError::AlreadyFilled`] with the previous intake.
pub(crate) fn save_intake(
    conn: &Connection,
    i: &Intake,
    replace: bool,
) -> Result<(), SaveError<Intake>> {
    if !replace && let Some(old) = get_intake(conn, i.date())? {
        return Err(SaveError::AlreadyFilled(old));
    }
    conn.execute(
        "INSERT OR REPLACE INTO intake (date, protein_g, carbs_g, fat_g, water_ml)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            i.date().to_string(),
            i.protein_g(),
            i.carbs_g(),
            i.fat_g(),
            i.water_ml()
        ],
    )?;
    Ok(())
}

/// Deletes the day's intake; no-op if there is none.
pub(crate) fn delete_intake(conn: &Connection, date: Day) -> Result<(), StoreError> {
    conn.execute("DELETE FROM intake WHERE date = ?1", [date.to_string()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;
    use crate::tests::{day, intake, store};

    #[test]
    fn second_intake_needs_replace() {
        let mut s: Store = store();
        let first = intake("2026-10-12", Some(1500));
        let second = intake("2026-10-12", None);
        s.save_intake(&first, false).unwrap();
        match s.save_intake(&second, false) {
            Err(SaveError::AlreadyFilled(old)) => assert_eq!(old, first),
            other => panic!("expected AlreadyFilled, got {other:?}"),
        }
        s.save_intake(&second, true).unwrap();
        assert_eq!(s.get_intake(day("2026-10-12")).unwrap(), Some(second));
    }

    #[test]
    fn intake_without_water_reads_none() {
        let mut s: Store = store();
        s.save_intake(&intake("2026-10-12", None), false).unwrap();
        let got = s.get_intake(day("2026-10-12")).unwrap().unwrap();
        assert_eq!(got.water_ml(), None);
        let raw: Option<i64> = s
            .conn()
            .query_row("SELECT water_ml FROM intake", [], |r| r.get(0))
            .unwrap();
        assert_eq!(raw, None);
    }
}

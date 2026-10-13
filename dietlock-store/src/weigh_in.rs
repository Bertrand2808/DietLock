use dietlock_core::{Day, WeighIn};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{SaveError, StoreError};

type RawWeighIn = (String, u16, bool);

fn raw_weigh_in(r: &Row<'_>) -> rusqlite::Result<RawWeighIn> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
}

fn to_weigh_in((d, w, f): RawWeighIn) -> Result<WeighIn, StoreError> {
    let day = Day::parse(&d).map_err(StoreError::InvalidRow)?;
    WeighIn::new(day, w, f).map_err(StoreError::InvalidRow)
}

pub(crate) fn get_weigh_in(conn: &Connection, date: Day) -> Result<Option<WeighIn>, StoreError> {
    conn.query_row(
        "SELECT date, weight_tenth_kg, fasted FROM weigh_in WHERE date = ?1",
        [date.to_string()],
        raw_weigh_in,
    )
    .optional()?
    .map(to_weigh_in)
    .transpose()
}

/// Weigh-ins from `from` to `to` inclusive, in ascending date order.
pub(crate) fn list_weigh_ins(
    conn: &Connection,
    from: Day,
    to: Day,
) -> Result<Vec<WeighIn>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT date, weight_tenth_kg, fasted FROM weigh_in
         WHERE date BETWEEN ?1 AND ?2 ORDER BY date",
    )?;
    let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_weigh_in)?;
    rows.map(|r| to_weigh_in(r?)).collect()
}

/// Saves a weigh-in. If the date is already filled and `replace` is false,
/// returns [`SaveError::AlreadyFilled`] with the previous weigh-in.
pub(crate) fn save_weigh_in(
    conn: &Connection,
    w: &WeighIn,
    replace: bool,
) -> Result<(), SaveError<WeighIn>> {
    if !replace && let Some(old) = get_weigh_in(conn, w.date())? {
        return Err(SaveError::AlreadyFilled(old));
    }
    conn.execute(
        "INSERT OR REPLACE INTO weigh_in (date, weight_tenth_kg, fasted) VALUES (?1, ?2, ?3)",
        params![w.date().to_string(), w.weight_tenth_kg(), w.fasted()],
    )?;
    Ok(())
}

/// Deletes the day's weigh-in; no-op if there is none.
pub(crate) fn delete_weigh_in(conn: &Connection, date: Day) -> Result<(), StoreError> {
    conn.execute("DELETE FROM weigh_in WHERE date = ?1", [date.to_string()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{day, store, weigh};

    #[test]
    fn weigh_in_round_trip() {
        let mut s = store();
        s.save_weigh_in(&weigh("2026-10-13", 826, true), false)
            .unwrap();
        let got = s.get_weigh_in(day("2026-10-13")).unwrap().unwrap();
        assert_eq!(got.weight_tenth_kg(), 826);
        assert!(got.fasted());
    }

    #[test]
    fn second_weigh_in_needs_replace() {
        let mut s = store();
        let first = weigh("2026-10-13", 826, true);
        let second = weigh("2026-10-13", 830, false);
        s.save_weigh_in(&first, false).unwrap();
        match s.save_weigh_in(&second, false) {
            Err(SaveError::AlreadyFilled(old)) => assert_eq!(old, first),
            other => panic!("expected AlreadyFilled, got {other:?}"),
        }
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(first));
        s.save_weigh_in(&second, true).unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(second));
    }
}

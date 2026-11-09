//! SQLite persistence for DietLock: weigh-ins and intakes, one row per day.
//!
//! Ranges are checked twice: by the `dietlock-core` types and by
//! `CHECK` constraints in the database.

mod error;
mod intake;
mod schema;
mod weigh_in;

use std::path::Path;

use dietlock_core::{Day, Intake, WeighIn};
use rusqlite::Connection;

pub use error::{SaveError, StoreError};
pub use schema::SCHEMA_VERSION;

/// SQLite-backed store for DietLock data.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens (or creates) the database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }

    /// Opens an in-memory database.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        schema::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Saves a weigh-in. If the date is already filled and `replace` is false,
    /// returns [`SaveError::AlreadyFilled`] with the previous weigh-in.
    pub fn save_weigh_in(&mut self, w: &WeighIn, replace: bool) -> Result<(), SaveError<WeighIn>> {
        let tx = self.conn.transaction()?;
        weigh_in::save_weigh_in(&tx, w, replace)?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_weigh_in(&self, date: Day) -> Result<Option<WeighIn>, StoreError> {
        weigh_in::get_weigh_in(&self.conn, date)
    }

    /// Weigh-ins from `from` to `to` inclusive, in ascending date order.
    pub fn list_weigh_ins(&self, from: Day, to: Day) -> Result<Vec<WeighIn>, StoreError> {
        weigh_in::list_weigh_ins(&self.conn, from, to)
    }

    /// Deletes the day's weigh-in; no-op if there is none.
    pub fn delete_weigh_in(&self, date: Day) -> Result<(), StoreError> {
        weigh_in::delete_weigh_in(&self.conn, date)
    }

    /// Saves an intake. If the date is already filled and `replace` is
    /// false, returns [`SaveError::AlreadyFilled`] with the previous intake.
    pub fn save_intake(&mut self, i: &Intake, replace: bool) -> Result<(), SaveError<Intake>> {
        let tx = self.conn.transaction()?;
        intake::save_intake(&tx, i, replace)?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_intake(&self, date: Day) -> Result<Option<Intake>, StoreError> {
        intake::get_intake(&self.conn, date)
    }

    /// Intakes from `from` to `to` inclusive, in ascending date order.
    pub fn list_intakes(&self, from: Day, to: Day) -> Result<Vec<Intake>, StoreError> {
        intake::list_intakes(&self.conn, from, to)
    }

    /// Deletes the day's intake; no-op if there is none.
    pub fn delete_intake(&self, date: Day) -> Result<(), StoreError> {
        intake::delete_intake(&self.conn, date)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    pub(crate) fn store() -> Store {
        Store::open_in_memory().unwrap()
    }

    pub(crate) fn weigh(d: &str, w: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(d), w, fasted).unwrap()
    }

    pub(crate) fn intake(d: &str, water: Option<u16>) -> Intake {
        Intake::new(day(d), 1500, 2000, 700, water).unwrap()
    }

    #[test]
    fn delete_empty_date_is_noop() {
        let mut s = store();
        let w = weigh("2026-10-13", 826, true);
        let i = intake("2026-10-13", Some(2000));
        s.save_weigh_in(&w, false).unwrap();
        s.save_intake(&i, false).unwrap();
        s.delete_weigh_in(day("2026-10-14")).unwrap();
        s.delete_intake(day("2026-10-14")).unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(w));
        assert_eq!(s.get_intake(day("2026-10-13")).unwrap(), Some(i));
        s.delete_weigh_in(day("2026-10-13")).unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), None);
        assert_eq!(s.get_intake(day("2026-10-13")).unwrap(), Some(i));
    }

    #[test]
    fn weigh_in_and_intake_are_independent() {
        let mut s = store();
        s.save_weigh_in(&weigh("2026-10-13", 826, true), false)
            .unwrap();
        assert_eq!(s.get_intake(day("2026-10-13")).unwrap(), None);
        s.save_intake(&intake("2026-10-14", None), false).unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-14")).unwrap(), None);
        s.save_intake(&intake("2026-10-13", None), false).unwrap();
        assert!(s.get_weigh_in(day("2026-10-13")).unwrap().is_some());
    }

    #[test]
    fn list_is_inclusive_and_ordered() {
        let mut s = store();
        for d in ["2026-10-15", "2026-10-11", "2026-10-13", "2026-10-12"] {
            s.save_weigh_in(&weigh(d, 800, true), false).unwrap();
            s.save_intake(&intake(d, None), false).unwrap();
        }
        let dates: Vec<String> = s
            .list_weigh_ins(day("2026-10-12"), day("2026-10-15"))
            .unwrap()
            .iter()
            .map(|w| w.date().to_string())
            .collect();
        assert_eq!(dates, ["2026-10-12", "2026-10-13", "2026-10-15"]);
        let dates: Vec<String> = s
            .list_intakes(day("2026-10-11"), day("2026-10-13"))
            .unwrap()
            .iter()
            .map(|i| i.date().to_string())
            .collect();
        assert_eq!(dates, ["2026-10-11", "2026-10-12", "2026-10-13"]);
    }

    #[test]
    fn list_with_from_after_to_is_empty() {
        let mut s = store();
        for d in ["2026-10-12", "2026-10-13", "2026-10-14"] {
            s.save_weigh_in(&weigh(d, 800, true), false).unwrap();
            s.save_intake(&intake(d, None), false).unwrap();
        }
        assert!(
            s.list_weigh_ins(day("2026-10-14"), day("2026-10-12"))
                .unwrap()
                .is_empty()
        );
        assert!(
            s.list_intakes(day("2026-10-14"), day("2026-10-12"))
                .unwrap()
                .is_empty()
        );
    }
}

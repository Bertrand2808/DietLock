//! Persistance SQLite de DietLock : pesées et alimentations, une ligne par jour.
//!
//! Les plages sont vérifiées deux fois : par les types de `dietlock-core` et par des
//! contraintes `CHECK` dans la base.

use std::fmt;
use std::path::Path;

use dietlock_core::{Day, Intake, ValidationError, WeighIn};
use rusqlite::{Connection, OptionalExtension, Row, params};

/// Version du schéma, stockée dans `PRAGMA user_version`.
pub const SCHEMA_VERSION: i32 = 1;

const SCHEMA_V1: &str = "
CREATE TABLE weigh_in (
    date            TEXT    PRIMARY KEY NOT NULL CHECK (date(date) IS date),
    weight_tenth_kg INTEGER NOT NULL CHECK (weight_tenth_kg BETWEEN 300 AND 2500),
    fasted          INTEGER NOT NULL CHECK (fasted IN (0, 1))
) STRICT;
CREATE TABLE intake (
    date      TEXT    PRIMARY KEY NOT NULL CHECK (date(date) IS date),
    protein_g INTEGER NOT NULL CHECK (protein_g BETWEEN 0 AND 1000),
    carbs_g   INTEGER NOT NULL CHECK (carbs_g BETWEEN 0 AND 1000),
    fat_g     INTEGER NOT NULL CHECK (fat_g BETWEEN 0 AND 1000),
    water_ml  INTEGER CHECK (water_ml IS NULL OR water_ml BETWEEN 0 AND 10000)
) STRICT;
PRAGMA user_version = 1;
";

/// Erreur d'accès à la base.
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    /// Schéma plus récent que cette version de l'application.
    UnsupportedSchemaVersion(i32),
    /// Ligne lue en base qui ne passe pas la validation des types.
    InvalidRow(ValidationError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "erreur SQLite : {e}"),
            Self::UnsupportedSchemaVersion(v) => write!(f, "version de schéma inconnue : {v}"),
            Self::InvalidRow(e) => write!(f, "ligne invalide en base : {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// Erreur d'enregistrement.
#[derive(Debug)]
pub enum SaveError<T> {
    /// La date est déjà remplie ; contient la valeur existante.
    AlreadyFilled(T),
    Store(StoreError),
}

impl<T: fmt::Debug> fmt::Display for SaveError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyFilled(old) => write!(f, "déjà rempli : {old:?}"),
            Self::Store(e) => e.fmt(f),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for SaveError<T> {}

impl<T> From<StoreError> for SaveError<T> {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl<T> From<rusqlite::Error> for SaveError<T> {
    fn from(e: rusqlite::Error) -> Self {
        Self::Store(e.into())
    }
}

/// Base DietLock.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Ouvre (ou crée) la base au chemin donné.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }

    /// Ouvre une base en mémoire.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        let tx = conn.transaction()?;
        let version: i32 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match version {
            0 => tx.execute_batch(SCHEMA_V1)?,
            SCHEMA_VERSION => {}
            v => return Err(StoreError::UnsupportedSchemaVersion(v)),
        }
        tx.commit()?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Enregistre une pesée. Si la date est déjà remplie et que `replace` est faux,
    /// renvoie [`SaveError::AlreadyFilled`] avec l'ancienne pesée.
    pub fn save_weigh_in(&mut self, w: &WeighIn, replace: bool) -> Result<(), SaveError<WeighIn>> {
        let tx = self.conn.transaction()?;
        if !replace && let Some(old) = get_weigh_in(&tx, w.date())? {
            return Err(SaveError::AlreadyFilled(old));
        }
        tx.execute(
            "INSERT OR REPLACE INTO weigh_in (date, weight_tenth_kg, fasted) VALUES (?1, ?2, ?3)",
            params![w.date().to_string(), w.weight_tenth_kg(), w.fasted()],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_weigh_in(&self, date: Day) -> Result<Option<WeighIn>, StoreError> {
        get_weigh_in(&self.conn, date)
    }

    /// Pesées de `from` à `to` inclus, par date croissante.
    pub fn list_weigh_ins(&self, from: Day, to: Day) -> Result<Vec<WeighIn>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT date, weight_tenth_kg, fasted FROM weigh_in
             WHERE date BETWEEN ?1 AND ?2 ORDER BY date",
        )?;
        let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_weigh_in)?;
        rows.map(|r| to_weigh_in(r?)).collect()
    }

    /// Supprime la pesée du jour ; sans effet si elle n'existe pas.
    pub fn delete_weigh_in(&self, date: Day) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM weigh_in WHERE date = ?1", [date.to_string()])?;
        Ok(())
    }

    /// Enregistre une alimentation. Si la date est déjà remplie et que `replace` est
    /// faux, renvoie [`SaveError::AlreadyFilled`] avec l'ancienne alimentation.
    pub fn save_intake(&mut self, i: &Intake, replace: bool) -> Result<(), SaveError<Intake>> {
        let tx = self.conn.transaction()?;
        if !replace && let Some(old) = get_intake(&tx, i.date())? {
            return Err(SaveError::AlreadyFilled(old));
        }
        tx.execute(
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
        tx.commit()?;
        Ok(())
    }

    pub fn get_intake(&self, date: Day) -> Result<Option<Intake>, StoreError> {
        get_intake(&self.conn, date)
    }

    /// Alimentations de `from` à `to` inclus, par date croissante.
    pub fn list_intakes(&self, from: Day, to: Day) -> Result<Vec<Intake>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT date, protein_g, carbs_g, fat_g, water_ml FROM intake
             WHERE date BETWEEN ?1 AND ?2 ORDER BY date",
        )?;
        let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_intake)?;
        rows.map(|r| to_intake(r?)).collect()
    }

    /// Supprime l'alimentation du jour ; sans effet si elle n'existe pas.
    pub fn delete_intake(&self, date: Day) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM intake WHERE date = ?1", [date.to_string()])?;
        Ok(())
    }
}

type RawWeighIn = (String, u16, bool);
type RawIntake = (String, u16, u16, u16, Option<u16>);

fn raw_weigh_in(r: &Row<'_>) -> rusqlite::Result<RawWeighIn> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
}

fn raw_intake(r: &Row<'_>) -> rusqlite::Result<RawIntake> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
}

fn to_weigh_in((d, w, f): RawWeighIn) -> Result<WeighIn, StoreError> {
    let day = Day::parse(&d).map_err(StoreError::InvalidRow)?;
    WeighIn::new(day, w, f).map_err(StoreError::InvalidRow)
}

fn to_intake((d, p, c, f, w): RawIntake) -> Result<Intake, StoreError> {
    let day = Day::parse(&d).map_err(StoreError::InvalidRow)?;
    Intake::new(day, p, c, f, w).map_err(StoreError::InvalidRow)
}

fn get_weigh_in(conn: &Connection, date: Day) -> Result<Option<WeighIn>, StoreError> {
    conn.query_row(
        "SELECT date, weight_tenth_kg, fasted FROM weigh_in WHERE date = ?1",
        [date.to_string()],
        raw_weigh_in,
    )
    .optional()?
    .map(to_weigh_in)
    .transpose()
}

fn get_intake(conn: &Connection, date: Day) -> Result<Option<Intake>, StoreError> {
    conn.query_row(
        "SELECT date, protein_g, carbs_g, fat_g, water_ml FROM intake WHERE date = ?1",
        [date.to_string()],
        raw_intake,
    )
    .optional()?
    .map(to_intake)
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> Day {
        Day::parse(s).unwrap()
    }

    fn store() -> Store {
        Store::open_in_memory().unwrap()
    }

    fn weigh(d: &str, w: u16, fasted: bool) -> WeighIn {
        WeighIn::new(day(d), w, fasted).unwrap()
    }

    fn intake(d: &str, water: Option<u16>) -> Intake {
        Intake::new(day(d), 150, 200, 70, water).unwrap()
    }

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
            other => panic!("attendu AlreadyFilled, obtenu {other:?}"),
        }
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(first));
        s.save_weigh_in(&second, true).unwrap();
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(second));
    }

    #[test]
    fn second_intake_needs_replace() {
        let mut s = store();
        let first = intake("2026-10-12", Some(1500));
        let second = intake("2026-10-12", None);
        s.save_intake(&first, false).unwrap();
        match s.save_intake(&second, false) {
            Err(SaveError::AlreadyFilled(old)) => assert_eq!(old, first),
            other => panic!("attendu AlreadyFilled, obtenu {other:?}"),
        }
        s.save_intake(&second, true).unwrap();
        assert_eq!(s.get_intake(day("2026-10-12")).unwrap(), Some(second));
    }

    #[test]
    fn intake_without_water_reads_none() {
        let mut s = store();
        s.save_intake(&intake("2026-10-12", None), false).unwrap();
        let got = s.get_intake(day("2026-10-12")).unwrap().unwrap();
        assert_eq!(got.water_ml(), None);
        let raw: Option<i64> = s
            .conn()
            .query_row("SELECT water_ml FROM intake", [], |r| r.get(0))
            .unwrap();
        assert_eq!(raw, None);
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

    fn sql_refused(s: &Store, sql: &str) {
        let err = s.conn().execute(sql, []).unwrap_err();
        assert!(
            err.to_string().contains("CHECK constraint failed"),
            "{sql} : {err}"
        );
    }

    #[test]
    fn check_constraints_refuse_out_of_range() {
        let s = store();
        let ok_w = "INSERT INTO weigh_in VALUES ('2026-10-13', 826, 1)";
        let ok_i = "INSERT INTO intake VALUES ('2026-10-12', 150, 200, 70, NULL)";
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 299, 1)");
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 2501, 1)");
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 826, 2)");
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 1001, 0, 0, NULL)",
        );
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 0, 1001, 0, NULL)",
        );
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 0, 0, 1001, NULL)",
        );
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 0, 0, 0, 10001)",
        );
        sql_refused(&s, "INSERT INTO intake VALUES ('2026-10-12', 0, 0, 0, -1)");
        for bad in ["2027-02-29", "2026-04-31", "2026-10-8"] {
            sql_refused(
                &s,
                &format!("INSERT INTO weigh_in VALUES ('{bad}', 826, 1)"),
            );
            sql_refused(
                &s,
                &format!("INSERT INTO intake VALUES ('{bad}', 0, 0, 0, NULL)"),
            );
        }
        s.conn().execute(ok_w, []).unwrap();
        s.conn().execute(ok_i, []).unwrap();
        s.conn()
            .execute("INSERT INTO weigh_in VALUES ('2028-02-29', 826, 0)", [])
            .unwrap();
    }

    #[test]
    fn reopening_v1_file_keeps_data() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("dietlock-test-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dietlock.sqlite");
        let w = weigh("2026-10-13", 826, true);
        let i = intake("2026-10-12", Some(1500));
        {
            let mut s = Store::open(&path).unwrap();
            s.save_weigh_in(&w, false).unwrap();
            s.save_intake(&i, false).unwrap();
        }
        let s = Store::open(&path).unwrap();
        let version: i32 = s
            .conn()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(s.get_weigh_in(day("2026-10-13")).unwrap(), Some(w));
        assert_eq!(s.get_intake(day("2026-10-12")).unwrap(), Some(i));
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn newer_schema_is_refused() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 2").unwrap();
        assert!(matches!(
            Store::init(conn),
            Err(StoreError::UnsupportedSchemaVersion(2))
        ));
    }
}

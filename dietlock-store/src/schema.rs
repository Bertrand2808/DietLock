use rusqlite::Connection;

use crate::error::StoreError;

/// Schema version, stored in `PRAGMA user_version`.
pub const SCHEMA_VERSION: i32 = 2;

const WEIGH_IN_TABLE: &str = "
CREATE TABLE weigh_in (
    date            TEXT    PRIMARY KEY NOT NULL CHECK (date(date) IS date),
    weight_tenth_kg INTEGER NOT NULL CHECK (weight_tenth_kg BETWEEN 300 AND 2500),
    fasted          INTEGER NOT NULL CHECK (fasted IN (0, 1))
) STRICT;
";

/// v2 intake table. Macros are tenths of a gram. Water stays millilitres.
fn intake_table(name: &str) -> String {
    format!(
        "CREATE TABLE {name} (
    date            TEXT    PRIMARY KEY NOT NULL CHECK (date(date) IS date),
    protein_tenth_g INTEGER NOT NULL CHECK (protein_tenth_g BETWEEN 0 AND 10000),
    carbs_tenth_g   INTEGER NOT NULL CHECK (carbs_tenth_g BETWEEN 0 AND 10000),
    fat_tenth_g     INTEGER NOT NULL CHECK (fat_tenth_g BETWEEN 0 AND 10000),
    water_ml        INTEGER CHECK (water_ml IS NULL OR water_ml BETWEEN 0 AND 10000)
) STRICT;"
    )
}

/// Creates v2 when the database is empty, scales a v1 file, and refuses
/// any version this application does not know.
pub(crate) fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    let version: i32 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    match version {
        0 => {
            let sql = format!(
                "{WEIGH_IN_TABLE}{}PRAGMA user_version = 2;",
                intake_table("intake")
            );
            tx.execute_batch(&sql)?;
        }
        1 => {
            // v1 stored whole grams. Multiply by 10 so the same amount is tenths of a gram.
            let sql = format!(
                "{}
INSERT INTO intake_v2 (date, protein_tenth_g, carbs_tenth_g, fat_tenth_g, water_ml)
SELECT date, protein_g * 10, carbs_g * 10, fat_g * 10, water_ml FROM intake;
DROP TABLE intake;
ALTER TABLE intake_v2 RENAME TO intake;
PRAGMA user_version = 2;",
                intake_table("intake_v2")
            );
            tx.execute_batch(&sql)?;
        }
        SCHEMA_VERSION => {}
        v => return Err(StoreError::UnsupportedSchemaVersion(v)),
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;
    use crate::tests::{day, intake, store, weigh};

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
        let ok_i = "INSERT INTO intake VALUES ('2026-10-12', 10000, 10000, 10000, 10000)";
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 299, 1)");
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 2501, 1)");
        sql_refused(&s, "INSERT INTO weigh_in VALUES ('2026-10-13', 826, 2)");
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 10001, 0, 0, NULL)",
        );
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 0, 10001, 0, NULL)",
        );
        sql_refused(
            &s,
            "INSERT INTO intake VALUES ('2026-10-12', 0, 0, 10001, NULL)",
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
    fn v1_whole_grams_become_tenths_and_water_stays_ml() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dietlock-v1-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dietlock.sqlite");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "
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
                ",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO intake (date, protein_g, carbs_g, fat_g, water_ml)
                 VALUES ('2026-10-12', 150, 0, 0, 2000)",
                [],
            )
            .unwrap();
        }
        let check = |s: &Store| {
            let version: i32 = s
                .conn()
                .query_row("PRAGMA user_version", [], |r| r.get(0))
                .unwrap();
            assert_eq!(version, 2);
            let i = s.get_intake(day("2026-10-12")).unwrap().unwrap();
            assert_eq!(i.protein_tenth_g(), 1500);
            assert_eq!(i.carbs_tenth_g(), 0);
            assert_eq!(i.fat_tenth_g(), 0);
            assert_eq!(i.water_ml(), Some(2000));
            assert_eq!(dietlock_core::energy_kcal(&i), 600);
            let stored: i64 = s
                .conn()
                .query_row("SELECT * FROM intake", [], |r| r.get(1))
                .unwrap();
            assert_eq!(stored, 1500);
        };
        {
            let s = Store::open(&path).unwrap();
            check(&s);
        }
        let s = Store::open(&path).unwrap();
        check(&s);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn newer_schema_is_refused() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 3").unwrap();
        assert!(matches!(
            Store::init(conn),
            Err(StoreError::UnsupportedSchemaVersion(3))
        ));
    }
}

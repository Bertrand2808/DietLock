use dietlock_core::{Activity, ActivityKind, Day, ValidationError};
use rusqlite::{Connection, Row, params};

use crate::error::StoreError;

type RawActivity = (String, String, Option<String>);

fn raw_activity(r: &Row<'_>) -> rusqlite::Result<RawActivity> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
}

fn to_activity((date, kind, note): RawActivity) -> Result<Activity, StoreError> {
    let day = Day::parse(&date).map_err(StoreError::InvalidRow)?;
    let kind = ActivityKind::from_key(&kind)
        .ok_or_else(|| StoreError::InvalidRow(ValidationError::UnknownActivityType(kind)))?;
    Activity::new(day, kind, note.as_deref().unwrap_or("")).map_err(StoreError::InvalidRow)
}

/// Appends the activities. Existing rows of the same dates stay (AC5).
pub(crate) fn add_activities(conn: &Connection, activities: &[Activity]) -> Result<(), StoreError> {
    let mut stmt = conn.prepare("INSERT INTO activity (date, kind, note) VALUES (?1, ?2, ?3)")?;
    for a in activities {
        stmt.execute(params![a.date().to_string(), a.kind().key(), a.note()])?;
    }
    Ok(())
}

/// Activities from `from` to `to` inclusive, by date then by order of entry.
pub(crate) fn list_activities(
    conn: &Connection,
    from: Day,
    to: Day,
) -> Result<Vec<Activity>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT date, kind, note FROM activity
         WHERE date BETWEEN ?1 AND ?2 ORDER BY date, id",
    )?;
    let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_activity)?;
    rows.map(|r| to_activity(r?)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{day, intake, store, weigh};

    fn act(d: &str, kind: ActivityKind, note: &str) -> Activity {
        Activity::new(day(d), kind, note).unwrap()
    }

    fn all(s: &crate::Store, d: &str) -> Vec<Activity> {
        s.list_activities(day(d), day(d)).unwrap()
    }

    #[test]
    fn ac3_a_batch_is_saved_in_order_for_its_date() {
        let mut s = store();
        let batch = [
            act("2026-11-09", ActivityKind::Walk, "chien"),
            act("2026-11-09", ActivityKind::Strength, ""),
            act("2026-11-09", ActivityKind::Bike, "à jeun"),
        ];
        s.add_activities(&batch).unwrap();
        assert_eq!(all(&s, "2026-11-09"), batch);
    }

    #[test]
    fn ac5_a_second_save_on_the_same_date_appends() {
        let mut s = store();
        let first = act("2026-11-09", ActivityKind::Walk, "");
        let second = act("2026-11-09", ActivityKind::Run, "5 km");
        s.add_activities(std::slice::from_ref(&first)).unwrap();
        s.add_activities(std::slice::from_ref(&second)).unwrap();
        assert_eq!(all(&s, "2026-11-09"), [first, second]);
    }

    #[test]
    fn ac5_identical_activities_are_kept_twice() {
        let mut s = store();
        let a = act("2026-11-09", ActivityKind::Walk, "");
        s.add_activities(&[a.clone(), a.clone()]).unwrap();
        assert_eq!(all(&s, "2026-11-09"), [a.clone(), a]);
    }

    #[test]
    fn ac5_activities_leave_weigh_in_and_intake_untouched() {
        let mut s = store();
        let w = weigh("2026-11-09", 826, true);
        let i = intake("2026-11-09", Some(2000));
        s.save_weigh_in(&w, false).unwrap();
        s.save_intake(&i, false).unwrap();
        s.add_activities(&[act("2026-11-09", ActivityKind::Run, "")])
            .unwrap();
        assert_eq!(s.get_weigh_in(day("2026-11-09")).unwrap(), Some(w));
        assert_eq!(s.get_intake(day("2026-11-09")).unwrap(), Some(i));
    }

    #[test]
    fn ac5_activities_do_not_create_a_weigh_in_or_an_intake() {
        let mut s = store();
        s.add_activities(&[act("2026-11-09", ActivityKind::Run, "")])
            .unwrap();
        assert_eq!(s.get_weigh_in(day("2026-11-09")).unwrap(), None);
        assert_eq!(s.get_intake(day("2026-11-09")).unwrap(), None);
    }

    #[test]
    fn ac6_activities_are_listed_for_the_right_dates_only() {
        let mut s = store();
        s.add_activities(&[
            act("2026-11-10", ActivityKind::Bike, ""),
            act("2026-11-08", ActivityKind::Walk, ""),
            act("2026-11-09", ActivityKind::Run, ""),
            act("2026-11-09", ActivityKind::Other, "x"),
        ])
        .unwrap();
        let kinds = |from: &str, to: &str| -> Vec<ActivityKind> {
            s.list_activities(day(from), day(to))
                .unwrap()
                .iter()
                .map(|a| a.kind())
                .collect()
        };
        assert_eq!(
            kinds("2026-11-08", "2026-11-10"),
            [
                ActivityKind::Walk,
                ActivityKind::Run,
                ActivityKind::Other,
                ActivityKind::Bike
            ]
        );
        assert_eq!(kinds("2026-11-09", "2026-11-09").len(), 2);
        assert!(kinds("2026-11-11", "2026-11-12").is_empty());
        assert!(kinds("2026-11-10", "2026-11-08").is_empty());
    }

    #[test]
    fn ac6_activities_survive_reopening_the_file() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("dietlock-activity-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dietlock.sqlite");
        let batch = [
            act("2026-11-08", ActivityKind::Mobility, "é"),
            act("2026-11-08", ActivityKind::Walk, ""),
        ];
        {
            let mut s = crate::Store::open(&path).unwrap();
            s.add_activities(&batch).unwrap();
        }
        let s = crate::Store::open(&path).unwrap();
        assert_eq!(all(&s, "2026-11-08"), batch);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ac3_a_batch_is_all_or_nothing() {
        let mut s = store();
        s.conn()
            .execute_batch("DROP TABLE activity; CREATE TABLE activity (id INTEGER PRIMARY KEY, date TEXT, kind TEXT NOT NULL CHECK (kind <> 'bike'), note TEXT)")
            .unwrap();
        let err = s.add_activities(&[
            act("2026-11-09", ActivityKind::Walk, ""),
            act("2026-11-09", ActivityKind::Bike, ""),
        ]);
        assert!(err.is_err());
        assert!(all(&s, "2026-11-09").is_empty());
    }

    #[test]
    fn a_stored_unknown_type_is_reported_not_hidden() {
        let s = store();
        s.conn()
            .execute_batch("DROP TABLE activity; CREATE TABLE activity (id INTEGER PRIMARY KEY, date TEXT, kind TEXT, note TEXT); INSERT INTO activity (date, kind) VALUES ('2026-11-09', 'swim')")
            .unwrap();
        assert!(matches!(
            s.list_activities(day("2026-11-09"), day("2026-11-09")),
            Err(StoreError::InvalidRow(ValidationError::UnknownActivityType(k))) if k == "swim"
        ));
    }
}

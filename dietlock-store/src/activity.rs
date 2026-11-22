use dietlock_core::{Activity, ActivityKind, Day, ValidationError};
use rusqlite::{Connection, Row, params};

use crate::error::StoreError;

type RawActivity = (String, String, Option<String>);

fn to_activity((date, kind, note): RawActivity) -> Result<Activity, StoreError> {
    let day = Day::parse(&date).map_err(StoreError::InvalidRow)?;
    let kind = ActivityKind::from_key(&kind)
        .ok_or_else(|| StoreError::InvalidRow(ValidationError::UnknownActivityType(kind)))?;
    Activity::new(day, kind, note.as_deref().unwrap_or("")).map_err(StoreError::InvalidRow)
}

/// An activity already stored. An activity being saved has no id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredActivity {
    id: i64,
    activity: Activity,
}

impl StoredActivity {
    pub fn new(id: i64, activity: Activity) -> Self {
        Self { id, activity }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn activity(&self) -> &Activity {
        &self.activity
    }
}

type RawStored = (i64, String, String, Option<String>);

fn raw_stored(r: &Row<'_>) -> rusqlite::Result<RawStored> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
}

fn to_stored((id, date, kind, note): RawStored) -> Result<StoredActivity, StoreError> {
    Ok(StoredActivity {
        id,
        activity: to_activity((date, kind, note))?,
    })
}

/// Appends the activities. Existing rows of the same dates stay (AC5).
pub(crate) fn add_activities(conn: &Connection, activities: &[Activity]) -> Result<(), StoreError> {
    let mut stmt = conn.prepare("INSERT INTO activity (date, kind, note) VALUES (?1, ?2, ?3)")?;
    for a in activities {
        stmt.execute(params![a.date().to_string(), a.kind().key(), a.note()])?;
    }
    Ok(())
}

/// Stored activities from `from` to `to` inclusive, by date then by id.
pub(crate) fn list_activities(
    conn: &Connection,
    from: Day,
    to: Day,
) -> Result<Vec<StoredActivity>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, date, kind, note FROM activity
         WHERE date BETWEEN ?1 AND ?2 ORDER BY date, id",
    )?;
    let rows = stmt.query_map(params![from.to_string(), to.to_string()], raw_stored)?;
    rows.map(|r| to_stored(r?)).collect()
}

/// Removes the stored activity with this id. An unknown id removes nothing.
pub(crate) fn delete_activity(conn: &Connection, id: i64) -> Result<(), StoreError> {
    conn.execute("DELETE FROM activity WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{day, intake, store, weigh};

    fn act(d: &str, kind: ActivityKind, note: &str) -> Activity {
        Activity::new(day(d), kind, note).unwrap()
    }

    fn all(s: &crate::Store, d: &str) -> Vec<Activity> {
        s.list_activities(day(d), day(d))
            .unwrap()
            .into_iter()
            .map(|row| row.activity().clone())
            .collect()
    }

    fn rows(s: &crate::Store, from: &str, to: &str) -> Vec<StoredActivity> {
        s.list_activities(day(from), day(to)).unwrap()
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
            rows(&s, from, to)
                .iter()
                .map(|row| row.activity().kind())
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

    #[test]
    fn ac7_listed_rows_keep_their_id_ordered_by_date_then_id() {
        let mut s = store();
        s.add_activities(&[act("2026-11-10", ActivityKind::Bike, "")])
            .unwrap();
        s.add_activities(&[
            act("2026-11-08", ActivityKind::Walk, ""),
            act("2026-11-08", ActivityKind::Run, "a"),
        ])
        .unwrap();
        let listed = rows(&s, "2026-11-08", "2026-11-10");
        let ids: Vec<i64> = listed.iter().map(|row| row.id()).collect();
        assert!(ids[0] < ids[1], "{ids:?}");
        assert!(ids[1] > ids[2], "{ids:?}");
        assert_eq!(listed[0].activity().kind(), ActivityKind::Walk);
        assert_eq!(listed[1].activity().note(), Some("a"));
        assert_eq!(listed[2].activity().date(), day("2026-11-10"));
    }

    #[test]
    fn ac9_delete_removes_only_that_row() {
        let mut s = store();
        s.add_activities(&[
            act("2026-11-08", ActivityKind::Walk, "a"),
            act("2026-11-08", ActivityKind::Run, "b"),
            act("2026-11-09", ActivityKind::Bike, "c"),
        ])
        .unwrap();
        let before = rows(&s, "2026-11-08", "2026-11-09");
        s.delete_activity(before[1].id()).unwrap();
        let left = rows(&s, "2026-11-08", "2026-11-09");
        let ids: Vec<i64> = left.iter().map(|row| row.id()).collect();
        assert_eq!(ids, vec![before[0].id(), before[2].id()]);
        assert_eq!(left[0].activity().note(), Some("a"));
        assert_eq!(left[1].activity().note(), Some("c"));
    }

    #[test]
    fn ac9_duplicates_on_the_same_date_are_deleted_one_at_a_time() {
        let mut s = store();
        let a = act("2026-11-09", ActivityKind::Walk, "chien");
        s.add_activities(&[a.clone(), a.clone()]).unwrap();
        let before = rows(&s, "2026-11-09", "2026-11-09");
        assert_eq!(before.len(), 2);
        assert_ne!(before[0].id(), before[1].id());
        s.delete_activity(before[0].id()).unwrap();
        let left = rows(&s, "2026-11-09", "2026-11-09");
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].id(), before[1].id());
        assert_eq!(left[0].activity(), &a);
        s.delete_activity(before[1].id()).unwrap();
        assert!(rows(&s, "2026-11-09", "2026-11-09").is_empty());
    }

    #[test]
    fn ac9_deleting_an_activity_leaves_weigh_in_and_intake() {
        let mut s = store();
        let w = weigh("2026-11-09", 826, true);
        let i = intake("2026-11-09", Some(2000));
        s.save_weigh_in(&w, false).unwrap();
        s.save_intake(&i, false).unwrap();
        s.add_activities(&[act("2026-11-09", ActivityKind::Run, "")])
            .unwrap();
        let id = rows(&s, "2026-11-09", "2026-11-09")[0].id();
        s.delete_activity(id).unwrap();
        assert!(rows(&s, "2026-11-09", "2026-11-09").is_empty());
        assert_eq!(s.get_weigh_in(day("2026-11-09")).unwrap(), Some(w));
        assert_eq!(s.get_intake(day("2026-11-09")).unwrap(), Some(i));
    }

    #[test]
    fn ac9_an_unknown_id_deletes_nothing() {
        let mut s = store();
        s.add_activities(&[act("2026-11-09", ActivityKind::Walk, "")])
            .unwrap();
        let before = rows(&s, "2026-11-09", "2026-11-09");
        s.delete_activity(before[0].id() + 50).unwrap();
        s.delete_activity(0).unwrap();
        assert_eq!(rows(&s, "2026-11-09", "2026-11-09"), before);
    }

    #[test]
    fn ac9_deleting_an_activity_does_not_change_the_schema_version() {
        let s = store();
        let version = |s: &crate::Store| -> i32 {
            s.conn()
                .query_row("PRAGMA user_version", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(version(&s), crate::SCHEMA_VERSION);
        s.delete_activity(1).unwrap();
        assert_eq!(version(&s), crate::SCHEMA_VERSION);
    }

    #[test]
    fn ac11_a_deleted_activity_stays_deleted_after_reopening_the_file() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "dietlock-activity-delete-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dietlock.sqlite");
        let kept = act("2026-11-08", ActivityKind::Walk, "reste");
        let gone = act("2026-11-08", ActivityKind::Run, "partie");
        let w = weigh("2026-11-08", 826, true);
        let kept_id = {
            let mut s = crate::Store::open(&path).unwrap();
            s.save_weigh_in(&w, false).unwrap();
            s.add_activities(&[kept.clone(), gone]).unwrap();
            let listed = rows(&s, "2026-11-08", "2026-11-08");
            assert_eq!(listed.len(), 2);
            s.delete_activity(listed[1].id()).unwrap();
            listed[0].id()
        };
        let s = crate::Store::open(&path).unwrap();
        let listed = rows(&s, "2026-11-08", "2026-11-08");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id(), kept_id);
        assert_eq!(listed[0].activity(), &kept);
        assert_eq!(s.get_weigh_in(day("2026-11-08")).unwrap(), Some(w));
        let version: i32 = s
            .conn()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, crate::SCHEMA_VERSION);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

use crate::day::Day;
use crate::error::{ValidationError, check_range};
use crate::proposed_date::check_not_future;

/// Maximum length of an activity note, in characters.
pub const ACTIVITY_NOTE_MAX_CHARS: u16 = 200;

/// Type of an activity (AC1). The key is the stored and exchanged form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Walk,
    Run,
    Mobility,
    Strength,
    Bike,
    Other,
}

impl ActivityKind {
    /// Every type, in the order the form lists them.
    pub const ALL: [Self; 6] = [
        Self::Walk,
        Self::Run,
        Self::Mobility,
        Self::Strength,
        Self::Bike,
        Self::Other,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Run => "run",
            Self::Mobility => "mobility",
            Self::Strength => "strength",
            Self::Bike => "bike",
            Self::Other => "other",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }
}

/// One physical activity of a day, with an optional note (AC2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    date: Day,
    kind: ActivityKind,
    note: Option<String>,
}

impl Activity {
    /// The note is trimmed. A blank note is no note.
    pub fn new(date: Day, kind: ActivityKind, note: &str) -> Result<Self, ValidationError> {
        let note = note.trim();
        check_range(
            "activity_note",
            i64::try_from(note.chars().count()).unwrap_or(i64::MAX),
            0,
            ACTIVITY_NOTE_MAX_CHARS,
        )?;
        Ok(Self {
            date,
            kind,
            note: (!note.is_empty()).then(|| note.to_owned()),
        })
    }

    pub fn date(&self) -> Day {
        self.date
    }

    pub fn kind(&self) -> ActivityKind {
        self.kind
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// One row of the activity form, as typed. No type means the row has no type yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityDraft {
    pub kind: Option<String>,
    pub note: String,
}

/// Validates every row of the form for one date, all or nothing (AC3, AC4, D5).
/// A row without type and without note is ignored.
pub fn build_activities(
    date: Day,
    today: Day,
    drafts: &[ActivityDraft],
) -> Result<Vec<Activity>, ValidationError> {
    check_not_future(date, today)?;
    let mut activities = Vec::new();
    for draft in drafts {
        let kind = draft.kind.as_deref().filter(|key| !key.is_empty());
        let Some(key) = kind else {
            if draft.note.trim().is_empty() {
                continue;
            }
            return Err(ValidationError::MissingActivityType);
        };
        let kind = ActivityKind::from_key(key)
            .ok_or_else(|| ValidationError::UnknownActivityType(key.to_owned()))?;
        activities.push(Activity::new(date, kind, &draft.note)?);
    }
    if activities.is_empty() {
        return Err(ValidationError::NoActivity);
    }
    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::day;

    fn draft(kind: Option<&str>, note: &str) -> ActivityDraft {
        ActivityDraft {
            kind: kind.map(str::to_owned),
            note: note.to_owned(),
        }
    }

    #[test]
    fn ac4_refusals_have_a_readable_english_description() {
        assert_eq!(
            ValidationError::MissingActivityType.to_string(),
            "activity without a type"
        );
        assert_eq!(
            ValidationError::NoActivity.to_string(),
            "no activity to save"
        );
        assert_eq!(
            ValidationError::UnknownActivityType("swim".to_owned()).to_string(),
            "unknown activity type \"swim\""
        );
    }

    #[test]
    fn ac1_the_six_types_exist_in_the_listed_order() {
        let keys: Vec<&str> = ActivityKind::ALL.iter().map(|k| k.key()).collect();
        assert_eq!(
            keys,
            ["walk", "run", "mobility", "strength", "bike", "other"]
        );
    }

    #[test]
    fn ac1_every_type_round_trips_through_its_key() {
        for kind in ActivityKind::ALL {
            assert_eq!(ActivityKind::from_key(kind.key()), Some(kind));
        }
        assert_eq!(ActivityKind::from_key("swim"), None);
        assert_eq!(ActivityKind::from_key(""), None);
        assert_eq!(ActivityKind::from_key("Walk"), None);
    }

    #[test]
    fn ac2_a_note_is_trimmed_and_a_blank_note_is_none() {
        let d = day("2026-11-09");
        let a = Activity::new(d, ActivityKind::Walk, "  Parc  ").unwrap();
        assert_eq!(a.note(), Some("Parc"));
        assert_eq!(a.date(), d);
        assert_eq!(a.kind(), ActivityKind::Walk);
        assert_eq!(
            Activity::new(d, ActivityKind::Run, "   ").unwrap().note(),
            None
        );
        assert_eq!(
            Activity::new(d, ActivityKind::Run, "").unwrap().note(),
            None
        );
    }

    #[test]
    fn ac2_the_note_limit_counts_characters_not_bytes() {
        let d = day("2026-11-09");
        let max = usize::from(ACTIVITY_NOTE_MAX_CHARS);
        assert!(Activity::new(d, ActivityKind::Other, &"é".repeat(max)).is_ok());
        let err = Activity::new(d, ActivityKind::Other, &"é".repeat(max + 1)).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::OutOfRange {
                field: "activity_note",
                ..
            }
        ));
    }

    #[test]
    fn ac2_the_limit_is_measured_after_trimming() {
        let d = day("2026-11-09");
        let note = format!("  {}  ", "a".repeat(usize::from(ACTIVITY_NOTE_MAX_CHARS)));
        assert!(Activity::new(d, ActivityKind::Other, &note).is_ok());
    }

    #[test]
    fn ac3_several_drafts_become_activities_of_the_chosen_date_in_order() {
        let today = day("2026-11-09");
        let date = day("2026-11-08");
        let got = build_activities(
            date,
            today,
            &[
                draft(Some("walk"), "chien"),
                draft(Some("strength"), ""),
                draft(Some("bike"), "  "),
            ],
        )
        .unwrap();
        let kinds: Vec<ActivityKind> = got.iter().map(|a| a.kind()).collect();
        assert_eq!(
            kinds,
            [
                ActivityKind::Walk,
                ActivityKind::Strength,
                ActivityKind::Bike
            ]
        );
        assert!(got.iter().all(|a| a.date() == date));
        assert_eq!(got[0].note(), Some("chien"));
        assert_eq!(got[1].note(), None);
    }

    #[test]
    fn ac3_a_fully_empty_row_is_ignored() {
        let today = day("2026-11-09");
        let got = build_activities(
            today,
            today,
            &[draft(None, ""), draft(Some("run"), ""), draft(None, "   ")],
        )
        .unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].kind(), ActivityKind::Run);
    }

    #[test]
    fn ac4_a_note_without_a_type_is_refused() {
        let today = day("2026-11-09");
        let err = build_activities(
            today,
            today,
            &[draft(Some("run"), ""), draft(None, "oublié")],
        )
        .unwrap_err();
        assert_eq!(err, ValidationError::MissingActivityType);
        let err = build_activities(today, today, &[draft(Some(""), "oublié")]).unwrap_err();
        assert_eq!(err, ValidationError::MissingActivityType);
    }

    #[test]
    fn ac4_nothing_to_save_is_refused() {
        let today = day("2026-11-09");
        assert_eq!(
            build_activities(today, today, &[]).unwrap_err(),
            ValidationError::NoActivity
        );
        assert_eq!(
            build_activities(today, today, &[draft(None, ""), draft(None, " ")]).unwrap_err(),
            ValidationError::NoActivity
        );
    }

    #[test]
    fn ac4_an_unknown_type_is_refused() {
        let today = day("2026-11-09");
        assert_eq!(
            build_activities(today, today, &[draft(Some("swim"), "")]).unwrap_err(),
            ValidationError::UnknownActivityType("swim".to_owned())
        );
    }

    #[test]
    fn d5_a_future_date_is_refused_before_anything_else() {
        let today = day("2026-11-09");
        let err =
            build_activities(day("2026-11-10"), today, &[draft(Some("run"), "")]).unwrap_err();
        assert_eq!(err, ValidationError::FutureDate("2026-11-10".to_owned()));
        let err = build_activities(day("2026-11-10"), today, &[]).unwrap_err();
        assert_eq!(err, ValidationError::FutureDate("2026-11-10".to_owned()));
        assert!(build_activities(today, today, &[draft(Some("run"), "")]).is_ok());
    }

    #[test]
    fn ac4_one_bad_row_refuses_the_whole_batch() {
        let today = day("2026-11-09");
        let long = "x".repeat(usize::from(ACTIVITY_NOTE_MAX_CHARS) + 1);
        assert!(
            build_activities(
                today,
                today,
                &[draft(Some("run"), ""), draft(Some("walk"), &long)]
            )
            .is_err()
        );
    }
}

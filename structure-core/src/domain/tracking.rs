use serde::Serialize;

use crate::domain::planning::Weight;

/// A performed training session — the logged counterpart to a planned `Workout`.
///
/// `completed_at` is `None` while the session is in progress and set once it is
/// finished; there is no separate status. `planned_workout_id` is `None` for an
/// ad-hoc session and links to the prescription otherwise.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LoggedSession {
    id: i64,
    started_at: String,
    completed_at: Option<String>,
    bodyweight: Option<Weight>,
    note: Option<String>,
    planned_workout_id: Option<i64>,
}

impl LoggedSession {
    pub(crate) fn new(
        id: i64,
        started_at: String,
        completed_at: Option<String>,
        bodyweight: Option<Weight>,
        note: Option<String>,
        planned_workout_id: Option<i64>,
    ) -> LoggedSession {
        LoggedSession {
            id,
            started_at,
            completed_at,
            bodyweight,
            note,
            planned_workout_id,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn started_at(&self) -> &str {
        &self.started_at
    }

    pub fn completed_at(&self) -> Option<&str> {
        self.completed_at.as_deref()
    }

    pub fn bodyweight(&self) -> Option<Weight> {
        self.bodyweight
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn planned_workout_id(&self) -> Option<i64> {
        self.planned_workout_id
    }
}

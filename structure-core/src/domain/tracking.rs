use serde::Serialize;

use crate::domain::planning::{LibraryExercise, Weight};

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

/// A performed exercise within a `LoggedSession` — the logged counterpart to a
/// planned `PlannedExercise`.
///
/// `exercise` is the exercise actually performed. `planned_exercise_id` is
/// `None` for unplanned (extra) work and links to the prescription otherwise; a
/// substitution is detected when `exercise`'s id differs from the linked planned
/// exercise's.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LoggedExercise {
    id: i64,
    exercise: LibraryExercise,
    position: u32,
    planned_exercise_id: Option<i64>,
    note: Option<String>,
}

impl LoggedExercise {
    pub(crate) fn new(
        id: i64,
        exercise: LibraryExercise,
        position: u32,
        planned_exercise_id: Option<i64>,
        note: Option<String>,
    ) -> LoggedExercise {
        LoggedExercise {
            id,
            exercise,
            position,
            planned_exercise_id,
            note,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn exercise(&self) -> &LibraryExercise {
        &self.exercise
    }

    pub fn name(&self) -> &str {
        self.exercise.name()
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn planned_exercise_id(&self) -> Option<i64> {
        self.planned_exercise_id
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

use serde::Serialize;

use crate::domain::planning::{
    ExerciseType, LibraryExercise, Load, SetType, SetValidationError, Weight,
    load_matches_exercise_type,
};

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

/// A performed set within a `LoggedExercise` — the logged counterpart to a
/// planned `Set`.
///
/// `reps` is concrete (a set that was logged was performed; a skipped set has no
/// row at all). `planned_set_id` is `None` for unplanned (extra) sets. Effort is
/// carried on `SetType::Regular`, exactly as in the plan. Drop sets are not yet
/// loggable here — they require segment storage that is deferred.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct LoggedSet {
    id: i64,
    position: u32,
    load: Load,
    reps: u32,
    set_type: SetType,
    planned_set_id: Option<i64>,
}

impl LoggedSet {
    pub(crate) fn new(
        id: i64,
        position: u32,
        exercise_type: ExerciseType,
        load: Load,
        reps: u32,
        set_type: SetType,
        planned_set_id: Option<i64>,
    ) -> Result<LoggedSet, SetValidationError> {
        if !load_matches_exercise_type(exercise_type, load) {
            return Err(SetValidationError::LoadMismatch {
                load,
                exercise_type,
            });
        }
        Ok(LoggedSet {
            id,
            position,
            load,
            reps,
            set_type,
            planned_set_id,
        })
    }

    /// Builds a `LoggedSet` from already-persisted columns without re-checking
    /// the load against the exercise type. Use only on the read path: the
    /// invariant was enforced by `new` on write, so re-validating would only
    /// force reads to join in the exercise type.
    pub(crate) fn new_unchecked(
        id: i64,
        position: u32,
        load: Load,
        reps: u32,
        set_type: SetType,
        planned_set_id: Option<i64>,
    ) -> LoggedSet {
        LoggedSet {
            id,
            position,
            load,
            reps,
            set_type,
            planned_set_id,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn load(&self) -> Load {
        self.load
    }

    pub fn reps(&self) -> u32 {
        self.reps
    }

    pub fn set_type(&self) -> SetType {
        self.set_type
    }

    pub fn planned_set_id(&self) -> Option<i64> {
        self.planned_set_id
    }
}

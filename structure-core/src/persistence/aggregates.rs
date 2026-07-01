use rusqlite::Connection;
use serde::Serialize;

use crate::domain::planning::{LibraryExercise, MesocycleMode, Phase, Set, SetGroup, Weight};
use crate::domain::tracking::LoggedSet;
use crate::persistence::logged_exercises::{self, LoggedExerciseError};
use crate::persistence::logged_sessions::{self, LoggedSessionError};
use crate::persistence::logged_sets::{self, LoggedSetError};
use crate::persistence::mesocycles::{self, MesocycleError};
use crate::persistence::microcycles::{self, MicrocycleError};
use crate::persistence::planned_exercises::{self, PlannedExerciseError};
use crate::persistence::set_groups::{self, SetGroupError};
use crate::persistence::sets::{self, SetError};
use crate::persistence::workouts::{self, WorkoutError};

#[derive(Debug, thiserror::Error)]
pub enum FullMesocycleError {
    #[error(transparent)]
    Mesocycle(#[from] MesocycleError),
    #[error(transparent)]
    Microcycle(#[from] MicrocycleError),
    #[error(transparent)]
    Workout(#[from] WorkoutError),
    #[error(transparent)]
    PlannedExercise(#[from] PlannedExerciseError),
    #[error(transparent)]
    Set(#[from] SetError),
    #[error(transparent)]
    SetGroup(#[from] SetGroupError),
}

#[derive(Serialize)]
pub struct FullMesocycle {
    pub id: i64,
    pub name: String,
    pub mode: MesocycleMode,
    pub microcycles: Vec<FullMicrocycle>,
}

#[derive(Serialize)]
pub struct FullMicrocycle {
    pub id: i64,
    pub position: u32,
    pub phase: Option<Phase>,
    pub workouts: Vec<FullWorkout>,
}

#[derive(Serialize)]
pub struct FullWorkout {
    pub id: i64,
    pub name: String,
    pub position: u32,
    pub planned_exercises: Vec<FullPlannedExercise>,
}

#[derive(Serialize)]
pub struct FullPlannedExercise {
    pub id: i64,
    pub exercise: LibraryExercise,
    pub position: u32,
    pub sets: Vec<Set>,
    pub set_groups: Vec<SetGroup>,
}

pub fn get_full_mesocycle(
    conn: &Connection,
    id: i64,
) -> Result<Option<FullMesocycle>, FullMesocycleError> {
    let Some(mesocycle) = mesocycles::get(conn, id)? else {
        return Ok(None);
    };

    let mut microcycles = Vec::new();
    for microcycle in microcycles::list(conn, id)? {
        let mut workouts = Vec::new();
        for workout in workouts::list(conn, microcycle.id())? {
            let mut planned_exercises = Vec::new();
            for planned in planned_exercises::list(conn, workout.id())? {
                let sets = sets::list(conn, planned.id())?;
                let set_groups = set_groups::list(conn, planned.id())?;
                planned_exercises.push(FullPlannedExercise {
                    id: planned.id(),
                    exercise: planned.exercise().clone(),
                    position: planned.position(),
                    sets,
                    set_groups,
                });
            }
            workouts.push(FullWorkout {
                id: workout.id(),
                name: workout.name().to_string(),
                position: workout.position(),
                planned_exercises,
            });
        }
        microcycles.push(FullMicrocycle {
            id: microcycle.id(),
            position: microcycle.position(),
            phase: microcycle.phase(),
            workouts,
        });
    }

    Ok(Some(FullMesocycle {
        id: mesocycle.id,
        name: mesocycle.name,
        mode: mesocycle.mode,
        microcycles,
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum FullLoggedSessionError {
    #[error(transparent)]
    LoggedSession(#[from] LoggedSessionError),
    #[error(transparent)]
    LoggedExercise(#[from] LoggedExerciseError),
    #[error(transparent)]
    LoggedSet(#[from] LoggedSetError),
}

#[derive(Serialize)]
pub struct FullLoggedSession {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub bodyweight: Option<Weight>,
    pub note: Option<String>,
    pub planned_workout_id: Option<i64>,
    pub exercises: Vec<FullLoggedExercise>,
}

#[derive(Serialize)]
pub struct FullLoggedExercise {
    pub id: i64,
    pub exercise: LibraryExercise,
    pub position: u32,
    pub planned_exercise_id: Option<i64>,
    pub note: Option<String>,
    pub sets: Vec<LoggedSet>,
}

pub fn get_full_logged_session(
    conn: &Connection,
    id: i64,
) -> Result<Option<FullLoggedSession>, FullLoggedSessionError> {
    let Some(session) = logged_sessions::get(conn, id).map_err(LoggedSessionError::from)? else {
        return Ok(None);
    };

    let mut exercises = Vec::new();
    for exercise in logged_exercises::list(conn, session.id())? {
        let sets = logged_sets::list(conn, exercise.id())?;
        exercises.push(FullLoggedExercise {
            id: exercise.id(),
            exercise: exercise.exercise().clone(),
            position: exercise.position(),
            planned_exercise_id: exercise.planned_exercise_id(),
            note: exercise.note().map(str::to_string),
            sets,
        });
    }

    Ok(Some(FullLoggedSession {
        id: session.id(),
        started_at: session.started_at().to_string(),
        completed_at: session.completed_at().map(str::to_string),
        bodyweight: session.bodyweight(),
        note: session.note().map(str::to_string),
        planned_workout_id: session.planned_workout_id(),
        exercises,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::{
        ExerciseType, Intensity, Load, PrescribedSetType, RepTarget, Rir, SetGroupType, SetType,
        Weight, WeightUnit,
    };
    use crate::persistence::{
        connection, library_exercises, logged_exercises, logged_sessions, mesocycles, microcycles,
        planned_exercises, set_groups, workouts,
    };

    const STARTED: &str = "2026-06-26T10:00:00Z";

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("failed to create test database")
    }

    fn weighted(value: f64) -> Load {
        Load::Weighted {
            weight: Some(Weight::new(value, WeightUnit::Kg)),
        }
    }

    #[test]
    fn get_full_mesocycle_exposes_planned_exercise_set_groups() {
        let mut conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let microcycle =
            microcycles::create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = workouts::create(&conn, microcycle.id(), "Push")
            .expect("workout creation should succeed");
        let bench = library_exercises::create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let planned = planned_exercises::create(&conn, workout.id(), bench.id())
            .expect("planned exercise creation should succeed");

        let top_set = SetGroupType::Prescribed {
            set_type: PrescribedSetType::Regular,
            reps: RepTarget::exact(5).unwrap(),
            intensity: Intensity::Rir(Rir::new(2).unwrap()),
        };
        set_groups::create(&mut conn, planned.id(), 3, top_set)
            .expect("set group creation should succeed");
        set_groups::create(&mut conn, planned.id(), 1, SetGroupType::MyorepMatch)
            .expect("set group creation should succeed");

        let full = get_full_mesocycle(&conn, mesocycle.id())
            .expect("query should succeed")
            .expect("mesocycle should exist");

        let planned_view = &full.microcycles[0].workouts[0].planned_exercises[0];
        assert_eq!(planned_view.set_groups.len(), 2);
        assert_eq!(planned_view.set_groups[0].number_of_sets(), 3);
        assert_eq!(
            planned_view.set_groups[1].set_group_type(),
            SetGroupType::MyorepMatch
        );
    }

    #[test]
    fn get_full_logged_session_returns_none_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = get_full_logged_session(&conn, 9999).expect("query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn get_full_logged_session_assembles_the_exercise_and_set_tree() {
        let mut conn = setup_test_db();
        let session = logged_sessions::create(
            &conn,
            STARTED,
            None,
            Some(Weight::new(82.5, WeightUnit::Kg)),
            Some("good session"),
        )
        .expect("session creation should succeed");
        let bench = library_exercises::create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let row = library_exercises::create(&conn, "Row", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let logged_bench = logged_exercises::create(&conn, session.id(), bench.id(), None, None)
            .expect("logged exercise creation should succeed");
        let logged_row = logged_exercises::create(&conn, session.id(), row.id(), None, None)
            .expect("logged exercise creation should succeed");

        logged_sets::create(
            &mut conn,
            logged_bench.id(),
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("set creation should succeed");
        logged_sets::create(
            &mut conn,
            logged_bench.id(),
            weighted(100.0),
            4,
            SetType::Regular { effort: None },
            None,
        )
        .expect("set creation should succeed");
        logged_sets::create(
            &mut conn,
            logged_row.id(),
            weighted(80.0),
            8,
            SetType::Regular { effort: None },
            None,
        )
        .expect("set creation should succeed");

        let full = get_full_logged_session(&conn, session.id())
            .expect("query should succeed")
            .expect("session should exist");

        assert_eq!(full.id, session.id());
        assert_eq!(full.started_at, STARTED);
        assert_eq!(full.note.as_deref(), Some("good session"));
        assert_eq!(full.bodyweight, Some(Weight::new(82.5, WeightUnit::Kg)));

        assert_eq!(full.exercises.len(), 2);
        assert_eq!(full.exercises[0].id, logged_bench.id());
        assert_eq!(full.exercises[0].sets.len(), 2);
        assert_eq!(full.exercises[0].sets[0].reps(), 5);
        assert_eq!(full.exercises[0].sets[1].reps(), 4);
        assert_eq!(full.exercises[1].id, logged_row.id());
        assert_eq!(full.exercises[1].sets.len(), 1);
        assert_eq!(full.exercises[1].sets[0].reps(), 8);
    }
}

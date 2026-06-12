use rusqlite::Connection;
use serde::Serialize;

use crate::domain::planning::{LibraryExercise, MesocycleMode, Set};
use crate::persistence::exercises::{PlannedExerciseError, list_planned_exercises};
use crate::persistence::mesocycles::{MesocycleError, get_mesocycle};
use crate::persistence::microcycles::{MicrocycleError, list_microcycles};
use crate::persistence::sets::{SetError, list_planned_sets};
use crate::persistence::workouts::{WorkoutError, list_workouts};

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
}

pub fn get_full_mesocycle(
    conn: &Connection,
    id: i64,
) -> Result<Option<FullMesocycle>, FullMesocycleError> {
    let Some(mesocycle) = get_mesocycle(conn, id)? else {
        return Ok(None);
    };

    let mut microcycles = Vec::new();
    for microcycle in list_microcycles(conn, id)? {
        let mut workouts = Vec::new();
        for workout in list_workouts(conn, microcycle.id())? {
            let mut planned_exercises = Vec::new();
            for planned in list_planned_exercises(conn, workout.id())? {
                let sets = list_planned_sets(conn, planned.id())?;
                planned_exercises.push(FullPlannedExercise {
                    id: planned.id(),
                    exercise: planned.exercise().clone(),
                    position: planned.position(),
                    sets,
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

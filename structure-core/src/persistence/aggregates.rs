use rusqlite::Connection;
use serde::Serialize;

use crate::domain::planning::{LibraryExercise, MesocycleMode, Set};
use crate::persistence::mesocycles::{self, MesocycleError};
use crate::persistence::microcycles::{self, MicrocycleError};
use crate::persistence::planned_exercises::{self, PlannedExerciseError};
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

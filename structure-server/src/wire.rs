//! Writes `web/src/api/wire.ts` from the Rust types this server serializes and
//! deserializes, so the web client cannot hold a different idea of the contract.
//!
//! The list below is the wire surface: the types a route names, each a root whose
//! fields `export_all` follows into their dependencies.

use structure_core::domain::planning::{
    LibraryExercise, Mesocycle, Microcycle, PlannedExercise, SetGroup, Workout,
};
use structure_core::persistence::aggregates::FullMesocycle;
use structure_core::persistence::mesocycles::MesocycleRow;
use ts_rs::{Config, TS};

use crate::dto::{
    CreateMesocycleRequest, LibraryExerciseRequest, PlannedExerciseRequest, ReorderRequest,
    SetGroupRequest, UpdateMesocycleRequest, UpdatePhaseRequest, WorkoutNameRequest,
};

/// ts-rs renders `i64` as `bigint`, which no JSON response can hold: ids arrive
/// as the numbers `serde_json` writes.
const LARGE_INT: &str = "number";

const OUT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/api");

#[test]
fn writes_the_web_clients_wire_types() {
    let config = Config::new()
        .with_large_int(LARGE_INT)
        .with_out_dir(OUT_DIR);

    export::<MesocycleRow>(&config);
    export::<Mesocycle>(&config);
    export::<FullMesocycle>(&config);
    export::<Microcycle>(&config);
    export::<Workout>(&config);
    export::<PlannedExercise>(&config);
    export::<SetGroup>(&config);
    export::<LibraryExercise>(&config);

    export::<CreateMesocycleRequest>(&config);
    export::<UpdateMesocycleRequest>(&config);
    export::<UpdatePhaseRequest>(&config);
    export::<WorkoutNameRequest>(&config);
    export::<PlannedExerciseRequest>(&config);
    export::<LibraryExerciseRequest>(&config);
    export::<SetGroupRequest>(&config);
    export::<ReorderRequest>(&config);
}

fn export<T: TS + 'static>(config: &Config) {
    T::export_all(config)
        .unwrap_or_else(|error| panic!("exporting {}: {error}", std::any::type_name::<T>()));
}

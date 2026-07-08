// Hand-written mirror of structure-server's wire types. These are assumptions
// about the server's JSON contract; the contract tests (tests/contract) verify
// them against a real server, since typechecking cannot.

export type MesocycleMode = "Algorithmic" | "Manual";

/** A row from `GET /mesocycles` — includes the computed microcycle count. */
export interface MesocycleRow {
  id: number;
  name: string;
  mode: MesocycleMode;
  microcycle_count: number;
}

/** The bare mesocycle returned by `POST`/`PUT /mesocycles`. */
export interface Mesocycle {
  id: number;
  name: string;
  mode: MesocycleMode;
}

export interface CreateMesocycle {
  name: string;
  mode: MesocycleMode;
}

// ---- The full mesocycle tree (GET /mesocycles/{id}/full) ----
//
// Rust enums serialize externally tagged, so a data-carrying variant is an
// object keyed by the variant name (`{ Exact: 5 }`), while a unit variant is a
// bare string (`"MyorepMatch"`). Newtypes serialize as their inner value.

export type Phase = "Accumulation" | "Intensification" | "Deload";

export type ExerciseType =
  | "Bodyweight"
  | "WeightedBodyweight"
  | "AssistedBodyweight"
  | "Weighted";

export type MuscleGroup =
  | "Chest"
  | "Back"
  | "Traps"
  | "Shoulders"
  | "Quads"
  | "Hamstrings"
  | "Glutes"
  | "Biceps"
  | "Triceps"
  | "Calves";

export interface LibraryExercise {
  id: number;
  name: string;
  exercise_type: ExerciseType;
  primary_muscle_group: MuscleGroup;
  secondary_muscle_groups: MuscleGroup[];
}

export type WeightUnit = "Kg" | "Lbs";

export interface Weight {
  value: number;
  unit: WeightUnit;
}

export type RepTarget =
  | { Exact: number }
  | { Range: { min: number; max: number } }
  | { AtLeast: number };

export type Intensity =
  | { Rir: number }
  | { Rpe: number }
  | { PercentOneRepMax: number }
  | { TargetWeight: Weight }
  | { WeightIncrement: Weight };

export type PrescribedSetType = "Regular" | "Myorep" | "Drop";

export type SetGroupType =
  | { Prescribed: { set_type: PrescribedSetType; reps: RepTarget; intensity: Intensity } }
  | "MyorepMatch";

export interface SetGroup {
  id: number;
  position: number;
  number_of_sets: number;
  set_group_type: SetGroupType;
}

export interface FullPlannedExercise {
  id: number;
  exercise: LibraryExercise;
  position: number;
  set_groups: SetGroup[];
}

export interface FullWorkout {
  id: number;
  name: string;
  position: number;
  planned_exercises: FullPlannedExercise[];
}

export interface FullMicrocycle {
  id: number;
  position: number;
  phase: Phase | null;
  workouts: FullWorkout[];
}

export interface FullMesocycle {
  id: number;
  name: string;
  mode: MesocycleMode;
  microcycles: FullMicrocycle[];
}

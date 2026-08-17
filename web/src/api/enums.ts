// Runtime companions to the enum unions in wire.ts, for rendering option lists.
// Those unions are generated from Rust, so a variant added to a backend enum
// reaches this file as a compile error rather than as a silently short list.
import type { ExerciseType, MuscleGroup, Phase } from "./wire";

/**
 * The union's values in the given order. The `Record<T, true>` argument is what
 * makes this exhaustive: adding a variant to the union without listing it here
 * fails to compile, so an option list can't silently fall behind the type.
 */
function valuesOf<T extends string>(members: Record<T, true>): T[] {
  return Object.keys(members) as T[];
}

export const PHASES = valuesOf<Phase>({
  Accumulation: true,
  Intensification: true,
  Deload: true,
});

export const EXERCISE_TYPES = valuesOf<ExerciseType>({
  Weighted: true,
  Bodyweight: true,
  WeightedBodyweight: true,
  AssistedBodyweight: true,
});

export const MUSCLE_GROUPS = valuesOf<MuscleGroup>({
  Chest: true,
  Back: true,
  Traps: true,
  Shoulders: true,
  Quads: true,
  Hamstrings: true,
  Glutes: true,
  Biceps: true,
  Triceps: true,
  Calves: true,
});

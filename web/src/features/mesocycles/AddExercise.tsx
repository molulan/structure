import { useState } from "react";
import type { ExerciseType, FullWorkout, LibraryExercise, MuscleGroup } from "../../api/wire";
import { EXERCISE_TYPES, MUSCLE_GROUPS } from "../../api/enums";
import { describeError } from "../../api/client";
import {
  useAddPlannedExercise,
  useCreateLibraryExercise,
  useLibraryExercises,
} from "./usePlanMutations";
import styles from "./MesocycleGrid.module.css";

interface Props {
  mesocycleId: number;
  workout: FullWorkout;
}

/**
 * Places an exercise into a workout, either picked from the library or created
 * on the spot — the library is the only source of exercises, so a plan built
 * from scratch has to be able to fill it from here.
 */
export function AddExercise({ mesocycleId, workout }: Props) {
  const library = useLibraryExercises();
  const addExercise = useAddPlannedExercise(mesocycleId);
  const [selectedId, setSelectedId] = useState("");
  const [creating, setCreating] = useState(false);

  // The same exercise twice in one workout is a non-goal of the plan model, and
  // two identically-named rows are indistinguishable once placed — so what is
  // already here is not offered again.
  const placed = new Set(workout.planned_exercises.map((planned) => planned.exercise.id));
  const available = (library.data ?? []).filter((exercise) => !placed.has(exercise.id));

  function place(libraryExerciseId: number) {
    addExercise.mutate(
      { workoutId: workout.id, libraryExerciseId },
      { onSuccess: () => setSelectedId("") },
    );
  }

  function onAdd(event: React.FormEvent) {
    event.preventDefault();
    const libraryExerciseId = Number(selectedId);
    if (!libraryExerciseId) return;
    place(libraryExerciseId);
  }

  if (creating) {
    return (
      // Creating and placing are two requests against two resources. The new
      // exercise is handed back here the moment it exists, so a failure to place
      // it leaves the user one click from retrying — rather than stuck in a form
      // whose next submit collides with the name it just took.
      <NewLibraryExercise
        onCreated={(libraryExerciseId) => {
          setCreating(false);
          setSelectedId(String(libraryExerciseId));
          place(libraryExerciseId);
        }}
        onCancel={() => setCreating(false)}
      />
    );
  }

  return (
    <form className={styles.addExercise} onSubmit={onAdd}>
      <select
        className={styles.select}
        aria-label={`Exercise to add to ${workout.name}`}
        value={selectedId}
        onChange={(event) => setSelectedId(event.target.value)}
        disabled={library.isPending}
      >
        {/* Only a library the server confirmed is described here: while pending
            the select is disabled, and a failed load has its own message below.
            A refetch that fails keeps the last good data, so passing `data`
            unguarded would report on a library that was just refused. */}
        <option value="">
          {pickerPlaceholder(library.isSuccess ? library.data : undefined, available.length)}
        </option>
        {available.map((exercise) => (
          <option key={exercise.id} value={exercise.id}>
            {exercise.name}
          </option>
        ))}
      </select>
      <button
        className={styles.addButton}
        type="submit"
        disabled={!selectedId || addExercise.isPending}
      >
        + Exercise
      </button>
      <button className={styles.textButton} type="button" onClick={() => setCreating(true)}>
        New exercise…
      </button>
      {library.isError && <span className={styles.error}>Could not load the exercise library.</span>}
    </form>
  );
}

/**
 * Having nothing to offer has two causes that need different answers. A library
 * with nothing in it is a first-run account, whose only way forward is the
 * "New exercise…" button beside this — telling it "All exercises added" is both
 * false and a signal to look elsewhere. A library that is merely used up means
 * this workout already holds everything there is.
 */
function pickerPlaceholder(
  confirmed: LibraryExercise[] | undefined,
  availableCount: number,
): string {
  if (confirmed === undefined) return "Pick an exercise…";
  if (confirmed.length === 0) return "No exercises yet — create one";
  if (availableCount === 0) return "All exercises added";
  return "Pick an exercise…";
}

/** Adds an exercise to the library and hands its id back to be placed. */
function NewLibraryExercise({
  onCreated,
  onCancel,
}: {
  onCreated: (libraryExerciseId: number) => void;
  onCancel: () => void;
}) {
  const createExercise = useCreateLibraryExercise();
  const [name, setName] = useState("");
  const [exerciseType, setExerciseType] = useState<ExerciseType>("Weighted");
  const [muscleGroup, setMuscleGroup] = useState<MuscleGroup>("Chest");
  const trimmed = name.trim();

  function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!trimmed) return;
    createExercise.mutate(
      {
        name: trimmed,
        exercise_type: exerciseType,
        primary_muscle_group: muscleGroup,
      },
      { onSuccess: (exercise) => onCreated(exercise.id) },
    );
  }

  return (
    <form className={styles.addExercise} onSubmit={onSubmit}>
      <input
        className={styles.input}
        placeholder="Exercise name"
        aria-label="Exercise name"
        value={name}
        autoFocus
        onChange={(event) => setName(event.target.value)}
      />
      <select
        className={styles.select}
        aria-label="Exercise type"
        value={exerciseType}
        onChange={(event) => setExerciseType(event.target.value as ExerciseType)}
      >
        {EXERCISE_TYPES.map((type) => (
          <option key={type} value={type}>
            {type}
          </option>
        ))}
      </select>
      <select
        className={styles.select}
        aria-label="Primary muscle group"
        value={muscleGroup}
        onChange={(event) => setMuscleGroup(event.target.value as MuscleGroup)}
      >
        {MUSCLE_GROUPS.map((group) => (
          <option key={group} value={group}>
            {group}
          </option>
        ))}
      </select>
      <button
        className={styles.addButton}
        type="submit"
        disabled={!trimmed || createExercise.isPending}
      >
        Create and add
      </button>
      <button className={styles.textButton} type="button" onClick={onCancel}>
        Cancel
      </button>
      {/* Every failure shape, not just an HTTP one: offline `fetch` rejects
          with a TypeError, which an ApiError-only guard would swallow. */}
      {createExercise.isError && (
        <span className={styles.error}>{describeError(createExercise.error)}</span>
      )}
    </form>
  );
}

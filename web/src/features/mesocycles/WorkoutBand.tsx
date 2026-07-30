import { useState } from "react";
import type { FullMicrocycle, FullPlannedExercise, FullWorkout } from "../../api/types";
import { describeSetGroup } from "./format";
import { AddExercise } from "./AddExercise";
import {
  useDeletePlannedExercise,
  useDeleteWorkout,
  useRenameWorkout,
} from "./usePlanMutations";
import styles from "./MesocycleGrid.module.css";

interface Props {
  mesocycleId: number;
  workout: FullWorkout;
  microcycles: FullMicrocycle[];
  columnCount: number;
}

/** One workout's row group: its header, its exercise rows, and the add control. */
export function WorkoutBand({ mesocycleId, workout, microcycles, columnCount }: Props) {
  return (
    <tbody>
      <tr>
        {/* Named explicitly so the rename and delete controls inside it stay
            out of the header's accessible name. */}
        <th
          scope="rowgroup"
          colSpan={columnCount}
          className={styles.workoutHead}
          aria-label={workout.name}
        >
          <WorkoutHeading mesocycleId={mesocycleId} workout={workout} />
        </th>
      </tr>

      {workout.planned_exercises.length === 0 ? (
        <tr>
          <td className={styles.bandEmpty} colSpan={columnCount}>
            No exercises yet.
          </td>
        </tr>
      ) : (
        workout.planned_exercises.map((planned) => (
          <ExerciseRow
            key={planned.id}
            mesocycleId={mesocycleId}
            planned={planned}
            microcycles={microcycles}
          />
        ))
      )}

      <tr>
        <td className={styles.addExerciseCell} colSpan={columnCount}>
          <AddExercise mesocycleId={mesocycleId} workout={workout} />
        </td>
      </tr>
    </tbody>
  );
}

function WorkoutHeading({
  mesocycleId,
  workout,
}: {
  mesocycleId: number;
  workout: FullWorkout;
}) {
  const rename = useRenameWorkout(mesocycleId);
  const remove = useDeleteWorkout(mesocycleId);
  const [draftName, setDraftName] = useState<string | null>(null);

  const trimmed = draftName?.trim() ?? "";

  function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!trimmed) return;
    if (trimmed === workout.name) {
      setDraftName(null);
      return;
    }
    rename.mutate({ workoutId: workout.id, name: trimmed }, { onSuccess: () => setDraftName(null) });
  }

  function onDelete() {
    if (window.confirm(`Delete ${workout.name}? Its exercises and their sets go with it.`)) {
      remove.mutate(workout.id);
    }
  }

  if (draftName !== null) {
    return (
      <form className={styles.renameForm} onSubmit={onSubmit}>
        <input
          className={styles.input}
          aria-label="Workout name"
          value={draftName}
          autoFocus
          onChange={(event) => setDraftName(event.target.value)}
          onKeyDown={(event) => event.key === "Escape" && setDraftName(null)}
        />
        <button
          className={styles.addButton}
          type="submit"
          disabled={!trimmed || rename.isPending}
        >
          Save
        </button>
        <button className={styles.textButton} type="button" onClick={() => setDraftName(null)}>
          Cancel
        </button>
      </form>
    );
  }

  return (
    <span className={styles.workoutHeadRow}>
      <h2 className={styles.workoutName}>{workout.name}</h2>{" "}
      {workout.planned_exercises.length > 0 && (
        <span className={styles.workoutMeta}>
          {exerciseCount(workout.planned_exercises.length)}
        </span>
      )}
      <span className={styles.workoutActions}>
        <button
          className={styles.textButton}
          aria-label={`Rename ${workout.name}`}
          onClick={() => setDraftName(workout.name)}
        >
          Rename
        </button>
        <button
          className={styles.textButton}
          aria-label={`Delete ${workout.name}`}
          onClick={onDelete}
          disabled={remove.isPending}
        >
          Delete
        </button>
      </span>
    </span>
  );
}

function ExerciseRow({
  mesocycleId,
  planned,
  microcycles,
}: {
  mesocycleId: number;
  planned: FullPlannedExercise;
  microcycles: FullMicrocycle[];
}) {
  const remove = useDeletePlannedExercise(mesocycleId);
  // Index the cells by week rather than trusting `prescriptions` to be parallel
  // to `microcycles`: a week with nothing prescribed still has to line up under
  // its own column.
  const cells = new Map(planned.prescriptions.map((p) => [p.microcycle_id, p.set_groups]));

  function onRemove() {
    if (window.confirm(`Remove ${planned.exercise.name}? Its sets in every week go with it.`)) {
      remove.mutate(planned.id);
    }
  }

  return (
    <tr>
      <th
        scope="row"
        className={styles.rowHead}
        aria-label={`${planned.exercise.name}, ${planned.exercise.primary_muscle_group}`}
      >
        <span className={styles.exerciseName}>{planned.exercise.name}</span>
        <span className={styles.rowHeadFoot}>
          <span className={styles.muscle}>{planned.exercise.primary_muscle_group}</span>
          <button
            className={styles.iconButton}
            aria-label={`Remove ${planned.exercise.name}`}
            onClick={onRemove}
            disabled={remove.isPending}
          >
            ✕
          </button>
        </span>
      </th>
      {microcycles.map((week) => {
        const setGroups = cells.get(week.id) ?? [];
        return (
          <td key={week.id} className={styles.cell}>
            {setGroups.length === 0 ? (
              // The dash is decoration; the cell's meaning is spelled out for
              // screen readers, which an aria-label on a plain span would not
              // reliably reach.
              <>
                <span className={styles.unprescribed} aria-hidden="true">
                  —
                </span>
                <span className={styles.srOnly}>Not prescribed</span>
              </>
            ) : (
              setGroups.map((group) => (
                <div key={group.id} className={styles.setLine}>
                  {describeSetGroup(group)}
                </div>
              ))
            )}
          </td>
        );
      })}
    </tr>
  );
}

function exerciseCount(count: number): string {
  return `${count} ${count === 1 ? "exercise" : "exercises"}`;
}

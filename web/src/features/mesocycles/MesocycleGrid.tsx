import { useState } from "react";
import type { FullMicrocycle, FullWorkout, Phase } from "../../api/types";
import { PHASES } from "../../api/enums";
import { PlanErrors } from "./PlanErrors";
import { WorkoutBand } from "./WorkoutBand";
import { useAddWeek, useAddWorkout, useDeleteWeek, useSetPhase } from "./usePlanMutations";
import styles from "./MesocycleGrid.module.css";

interface Props {
  mesocycleId: number;
  microcycles: FullMicrocycle[];
  workouts: FullWorkout[];
}

/**
 * The plan grid: weeks across, exercise slots down, each cell the set groups
 * prescribed for that (planned exercise, week). Structure is editable here —
 * weeks, workouts and their exercises; the cells themselves are still read-only.
 */
export function MesocycleGrid({ mesocycleId, microcycles, workouts }: Props) {
  // The label column plus the weeks — or the notice standing in for them, so
  // every row still spans the same number of columns.
  const columnCount = 1 + Math.max(microcycles.length, 1);

  return (
    <PlanErrors>
      <AddWeek mesocycleId={mesocycleId} />

      {/* Focusable so the weeks past the viewport edge can be reached without a
          pointer; a plain overflow container takes no keyboard focus. */}
      <div className={styles.scroll} role="region" aria-label="Plan grid" tabIndex={0}>
        <table className={styles.grid}>
          <thead>
            <tr>
              <td className={styles.corner} />
              {microcycles.length === 0 ? (
                <th scope="col" className={styles.noWeeks}>
                  No weeks yet.
                </th>
              ) : (
                microcycles.map((week) => (
                  <WeekHeader key={week.id} mesocycleId={mesocycleId} week={week} />
                ))
              )}
            </tr>
          </thead>

          {workouts.length === 0 ? (
            <tbody>
              <tr>
                <td className={styles.bandEmpty} colSpan={columnCount}>
                  No workouts yet.
                </td>
              </tr>
            </tbody>
          ) : (
            workouts.map((workout) => (
              <WorkoutBand
                key={workout.id}
                mesocycleId={mesocycleId}
                workout={workout}
                microcycles={microcycles}
                columnCount={columnCount}
              />
            ))
          )}
        </table>
      </div>

      <AddWorkout mesocycleId={mesocycleId} />
    </PlanErrors>
  );
}

function AddWeek({ mesocycleId }: { mesocycleId: number }) {
  const addWeek = useAddWeek(mesocycleId);

  return (
    <div className={styles.toolbar}>
      <button
        className={styles.addButton}
        onClick={() => addWeek.mutate()}
        disabled={addWeek.isPending}
      >
        + Week
      </button>
    </div>
  );
}

function WeekHeader({ mesocycleId, week }: { mesocycleId: number; week: FullMicrocycle }) {
  const setPhase = useSetPhase(mesocycleId);
  const deleteWeek = useDeleteWeek(mesocycleId);
  // Held only while the change is in flight: a controlled select driven purely
  // by server data snaps back to the old phase until the refetch lands.
  const [chosenPhase, setChosenPhase] = useState<string | null>(null);
  const label = `Week ${week.position + 1}`;

  function onDelete() {
    if (window.confirm(`Delete ${label}? Everything prescribed in it is deleted too.`)) {
      deleteWeek.mutate(week.id);
    }
  }

  function onPhaseChange(value: string) {
    setChosenPhase(value);
    setPhase.mutate(
      { microcycleId: week.id, phase: (value || null) as Phase | null },
      // Either way the server's value takes over again — on failure that
      // reverts the control, with the banner saying why.
      { onSuccess: () => setChosenPhase(null), onError: () => setChosenPhase(null) },
    );
  }

  return (
    // Named explicitly: the cell's controls would otherwise be folded into the
    // header's accessible name, and a table header is announced before every
    // cell beneath it.
    <th scope="col" className={styles.weekHead} aria-label={label}>
      <div className={styles.weekHeadRow}>
        <span className={styles.weekLabel}>{label}</span>
        <button
          className={styles.iconButton}
          aria-label={`Delete ${label}`}
          onClick={onDelete}
          disabled={deleteWeek.isPending}
        >
          ✕
        </button>
      </div>
      {/* The select doubles as the phase display, so it carries the phase
          colouring and spells the phase out rather than abbreviating it. */}
      <select
        className={styles.phaseSelect}
        data-phase={chosenPhase ?? week.phase ?? ""}
        aria-label={`Phase for ${label}`}
        value={chosenPhase ?? week.phase ?? ""}
        onChange={(event) => onPhaseChange(event.target.value)}
      >
        <option value="">No phase</option>
        {PHASES.map((phase) => (
          <option key={phase} value={phase}>
            {phase}
          </option>
        ))}
      </select>
    </th>
  );
}

function AddWorkout({ mesocycleId }: { mesocycleId: number }) {
  const addWorkout = useAddWorkout(mesocycleId);
  const [name, setName] = useState("");
  const trimmed = name.trim();

  function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!trimmed) return;
    addWorkout.mutate(trimmed, { onSuccess: () => setName("") });
  }

  return (
    <form className={styles.addWorkout} onSubmit={onSubmit}>
      <input
        className={styles.input}
        placeholder="New workout name"
        aria-label="New workout name"
        value={name}
        onChange={(event) => setName(event.target.value)}
      />
      <button
        className={styles.addButton}
        type="submit"
        disabled={!trimmed || addWorkout.isPending}
      >
        + Workout
      </button>
    </form>
  );
}

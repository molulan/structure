import { useRef, useState } from "react";
import type { FullMicrocycle, FullWorkout, Phase } from "../../api/types";
import { PHASES } from "../../api/enums";
import { GridFrame, gridColumnCount } from "./GridFrame";
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
  const columnCount = gridColumnCount(microcycles);

  return (
    <PlanErrors>
      <AddWeek mesocycleId={mesocycleId} />

      <GridFrame>
        <thead>
          <tr>
            <td className={styles.corner} />
            {microcycles.length === 0 ? (
              <th scope="col" className={styles.noWeeks}>
                No weeks yet.
              </th>
            ) : (
              // Numbered by column, not by stored `position`: deleting a week
              // leaves a gap in positions, and the grid must still read
              // Week 1, 2, 3 across.
              microcycles.map((week, index) => (
                <WeekHeader
                  key={week.id}
                  mesocycleId={mesocycleId}
                  week={week}
                  label={`Week ${index + 1}`}
                />
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
            />
          ))
        )}
      </GridFrame>

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

function WeekHeader({
  mesocycleId,
  week,
  label,
}: {
  mesocycleId: number;
  week: FullMicrocycle;
  label: string;
}) {
  const setPhase = useSetPhase(mesocycleId);
  const deleteWeek = useDeleteWeek(mesocycleId);
  // Held only while the change is in flight: a controlled select driven purely
  // by server data snaps back to the old phase until the refetch lands.
  const [chosenPhase, setChosenPhase] = useState<string | null>(null);
  // Only the newest change may release the held value; an earlier reply
  // landing later would otherwise revert a choice the user has since made.
  const latestChange = useRef(0);

  function onDelete() {
    if (window.confirm(`Delete ${label}? Everything prescribed in it is deleted too.`)) {
      deleteWeek.mutate(week.id);
    }
  }

  function onPhaseChange(value: string) {
    const change = ++latestChange.current;
    setChosenPhase(value);
    setPhase.mutate(
      { microcycleId: week.id, phase: (value || null) as Phase | null },
      // Either way the server's value takes over again — on failure that
      // reverts the control, with the banner saying why.
      {
        onSettled: () => {
          if (latestChange.current === change) setChosenPhase(null);
        },
      },
    );
  }

  // A phase this client doesn't know still has to be displayed as itself
  // rather than falling through to the empty option and reading as unset.
  const shownPhase = chosenPhase ?? week.phase ?? "";
  const unknownPhase = shownPhase !== "" && !PHASES.includes(shownPhase as Phase);

  return (
    // Named explicitly: the cell's controls would otherwise be folded into the
    // header's accessible name, and a table header is announced before every
    // cell beneath it.
    <th
      scope="col"
      className={styles.weekHead}
      aria-label={week.phase ? `${label}, ${week.phase}` : label}
    >
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
        data-phase={shownPhase}
        aria-label={`Phase for ${label}`}
        value={shownPhase}
        onChange={(event) => onPhaseChange(event.target.value)}
      >
        <option value="">No phase</option>
        {PHASES.map((phase) => (
          <option key={phase} value={phase}>
            {phase}
          </option>
        ))}
        {unknownPhase && <option value={shownPhase}>{shownPhase}</option>}
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

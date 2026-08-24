import { createContext, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import type { FullMicrocycle, Phase } from "../../api/wire";
import { PHASES } from "../../api/enums";
import { useDeleteWeek, useSetPhase } from "./usePlanMutations";
import styles from "./MesocycleGrid.module.css";

interface GridScope {
  mesocycleId: number;
  microcycles: FullMicrocycle[];
}

// No default: a row group outside a frame is a mistake, not a degraded state,
// and the alternative — the week list the header used going unnoticed — is what
// this exists to prevent.
const GridScopeContext = createContext<GridScope | null>(null);

/**
 * The weeks the enclosing frame sized its columns from, and the mesocycle its
 * edits belong to. Read rather than passed so a row group cannot lay out against
 * a different week list than the header above it.
 */
export function useGridScope(): GridScope {
  const scope = useContext(GridScopeContext);
  if (scope === null) {
    throw new Error("a plan row group must be rendered inside a GridFrame");
  }
  return scope;
}

/**
 * The grid's scroll container, table, and header row.
 *
 * The header row belongs here rather than with the caller because `.grid` is
 * `table-layout: fixed`: the browser sizes every column from the cells of the
 * *first* row, so the corner cell and the week headers are what give the table
 * its geometry. A band mounted without them lays out to widths the app never
 * has — which is how the dev workbench came to judge long names against a label
 * column that is 190px everywhere except there.
 */
export function GridFrame({
  mesocycleId,
  microcycles,
  children,
}: {
  mesocycleId: number;
  microcycles: FullMicrocycle[];
  children: ReactNode;
}) {
  const scope = useMemo(() => ({ mesocycleId, microcycles }), [mesocycleId, microcycles]);

  return (
    // Focusable so the weeks past the viewport edge can be reached without a
    // pointer; a plain overflow container takes no keyboard focus.
    <div className={styles.scroll} role="region" aria-label="Plan grid" tabIndex={0}>
      <GridScopeContext.Provider value={scope}>
        <table className={styles.grid}>
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

          {children}
        </table>
      </GridScopeContext.Provider>
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

/**
 * How many columns a full-width row spans: the label column plus the weeks, or
 * the notice standing in for them when there are none. Derived by each row that
 * needs it rather than passed down, so no caller can disagree with the header.
 */
export function gridColumnCount(microcycles: FullMicrocycle[]): number {
  return 1 + Math.max(microcycles.length, 1);
}

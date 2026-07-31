import type { ReactNode } from "react";
import type { FullMicrocycle } from "../../api/types";
import styles from "./MesocycleGrid.module.css";

/**
 * The grid's scroll container and table.
 *
 * Shared rather than reproduced: anything mounting a `WorkoutBand` outside
 * `MesocycleGrid` — the dev workbench — has to get the real container, or it
 * reports on a layout the app doesn't have.
 */
export function GridFrame({ children }: { children: ReactNode }) {
  return (
    // Focusable so the weeks past the viewport edge can be reached without a
    // pointer; a plain overflow container takes no keyboard focus.
    <div className={styles.scroll} role="region" aria-label="Plan grid" tabIndex={0}>
      <table className={styles.grid}>{children}</table>
    </div>
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

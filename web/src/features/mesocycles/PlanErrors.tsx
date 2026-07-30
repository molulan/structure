import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { describeError } from "../../api/client";
import styles from "./MesocycleGrid.module.css";

/**
 * One place for the grid's edits to report failure. The controls are scattered
 * across the table — a week header, a band header, a row, the add forms — and
 * an edit that silently does nothing reads as a broken control, so every
 * mutation reports here and the grid shows the last failure once.
 */
interface PlanErrorSink {
  report: (error: unknown) => void;
  /** Called when an edit succeeds, so a stale failure stops contradicting it. */
  clear: () => void;
}

// Reporting is a no-op outside a provider. Components that edit should be
// mounted inside one — including in tests, or their failure paths render
// nothing to assert on.
const PlanErrorContext = createContext<PlanErrorSink>({ report: () => {}, clear: () => {} });

export function usePlanErrorSink(): PlanErrorSink {
  return useContext(PlanErrorContext);
}

export function PlanErrors({ children }: { children: ReactNode }) {
  const [message, setMessage] = useState<string | null>(null);
  const report = useCallback((error: unknown) => setMessage(describeError(error)), []);
  const clear = useCallback(() => setMessage(null), []);
  const sink = useMemo(() => ({ report, clear }), [report, clear]);

  return (
    <PlanErrorContext.Provider value={sink}>
      {message !== null && (
        <p className={styles.errorBanner} role="alert">
          {message}{" "}
          <button className={styles.textButton} onClick={clear}>
            Dismiss
          </button>
        </p>
      )}
      {children}
    </PlanErrorContext.Provider>
  );
}

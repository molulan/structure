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
}

// Reporting is a no-op outside a provider so a component can be mounted on its
// own (in a test, say) without arranging one.
const PlanErrorContext = createContext<PlanErrorSink>({ report: () => {} });

export function usePlanErrorSink(): PlanErrorSink {
  return useContext(PlanErrorContext);
}

export function PlanErrors({ children }: { children: ReactNode }) {
  const [message, setMessage] = useState<string | null>(null);
  const report = useCallback((error: unknown) => setMessage(describeError(error)), []);
  const sink = useMemo(() => ({ report }), [report]);

  return (
    <PlanErrorContext.Provider value={sink}>
      {message !== null && (
        <p className={styles.errorBanner} role="alert">
          {message}{" "}
          <button className={styles.textButton} onClick={() => setMessage(null)}>
            Dismiss
          </button>
        </p>
      )}
      {children}
    </PlanErrorContext.Provider>
  );
}

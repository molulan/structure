import { lazy, Suspense } from "react";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/dev/workbench")({
  component: WorkbenchPage,
});

// Guarded at the module boundary rather than inside the component, so that a
// production build does not merely decline to render the workbench — it has
// never heard of it. Vite replaces `import.meta.env.DEV` with a literal, so this
// branch and the `import()` inside it are eliminated whole and nothing under
// `features/dev/` enters the graph. Guarding inside the component instead leaves
// the module present but unused, where what ships depends on the bundler proving
// each top-level statement pure — and it cannot prove that of `new Proxy(…)`.
const Workbench = import.meta.env.DEV
  ? lazy(() => import("../features/dev/Workbench").then((m) => ({ default: m.Workbench })))
  : null;

function WorkbenchPage() {
  if (!Workbench) {
    return <p>The workbench is a development-only page.</p>;
  }
  return (
    <Suspense fallback={null}>
      <Workbench />
    </Suspense>
  );
}

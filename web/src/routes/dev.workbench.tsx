import { createFileRoute } from "@tanstack/react-router";
import { Workbench } from "../features/dev/Workbench";

export const Route = createFileRoute("/dev/workbench")({
  component: WorkbenchPage,
});

// Code-split like every other route, so a production bundle only pays for the
// fixtures if someone navigates here — and finds a notice rather than them.
function WorkbenchPage() {
  if (!import.meta.env.DEV) {
    return <p>The workbench is a development-only page.</p>;
  }
  return <Workbench />;
}

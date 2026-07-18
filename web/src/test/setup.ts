import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Vitest doesn't auto-run Testing Library's cleanup, so unmount between tests to
// keep the jsdom document from accumulating trees across cases.
afterEach(cleanup);

import { defineConfig } from "vitest/config";

// Component tests: render React components in jsdom with a mocked API boundary,
// to cover the loading/error/empty/data branches that the query hooks drive —
// states that are awkward to force through the real-backend e2e suite. No
// network, no server; the `api` module is mocked per test.
//
// We transform JSX with esbuild's automatic runtime rather than pulling in
// @vitejs/plugin-react (its Plugin type clashes with vitest's bundled vite, and
// tests need no fast-refresh).
export default defineConfig({
  esbuild: { jsx: "automatic" },
  test: {
    include: ["src/**/*.component.test.tsx"],
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    // A `vi.spyOn` restored on the last line of a test is not restored when an
    // assertion above it throws, and `vi.clearAllMocks` only clears call
    // history. One failure would then leave `window.confirm` stubbed for every
    // test after it — passing a later "does it ask before deleting?" against a
    // stub left by an unrelated break. Restoring between tests removes the
    // possibility rather than asking everyone to remember the cleanup line.
    restoreMocks: true,
  },
});

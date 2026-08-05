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
    // Restores spies between tests, so a failure can't leak one into the next.
    // Implementations go too, so arrange mocks per test, never in `beforeAll`.
    restoreMocks: true,
  },
});

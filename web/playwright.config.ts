import { defineConfig } from "@playwright/test";

// e2e boots its own backend (fresh in-memory DB on 3001) and a Vite dev server
// proxying `/api` to it, then drives the real browser end-to-end.
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:5174",
    screenshot: "only-on-failure",
    trace: "on-first-retry",
  },
  webServer: [
    {
      command:
        "cargo run --quiet --manifest-path ../Cargo.toml -p structure-server",
      env: { PORT: "3001", STRUCTURE_DB: ":memory:" },
      url: "http://127.0.0.1:3001/health",
      reuseExistingServer: false,
      timeout: 200_000,
    },
    {
      command: "npm run dev -- --port 5174 --strictPort",
      env: { VITE_PROXY_TARGET: "http://127.0.0.1:3001" },
      url: "http://127.0.0.1:5174",
      reuseExistingServer: false,
      timeout: 120_000,
    },
  ],
});

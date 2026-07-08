import { defineConfig } from "@playwright/test";

// e2e runs on dedicated fixed ports — backend 3001 (fresh in-memory DB) and
// Vite 5174 proxying `/api` to it — kept separate from the dev ports so it
// never touches the real DB. Both start fresh (reuseExistingServer:false); if a
// port is already taken, the run fails loudly rather than reusing a foreign
// server.
export default defineConfig({
  testDir: "./e2e",
  // Serial: all specs share one backend instance (a single in-memory DB), so a
  // test must not assume global emptiness and specs must not run concurrently.
  workers: 1,
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

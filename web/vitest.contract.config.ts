import { defineConfig } from "vitest/config";

// Contract tests: exercise the typed API client against a real, freshly-booted
// structure-server, for what no Rust test reaches — the client's own reading of
// an error body, and the dev seed the workbench renders. What the server does
// with a request is asserted in structure-server/tests instead.
export default defineConfig({
  test: {
    include: ["tests/contract/**/*.test.ts"],
    environment: "node",
    globalSetup: ["./tests/contract/globalSetup.ts"],
    // Booting (and possibly compiling) the server can be slow on a cold cache.
    testTimeout: 30_000,
    hookTimeout: 200_000,
  },
});

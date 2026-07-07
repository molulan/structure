import { defineConfig } from "vitest/config";

// Contract tests: exercise the typed API client against a real, freshly-booted
// structure-server. This is what verifies our hand-written TS types actually
// match the server's wire format (snake_case keys, externally-tagged enums) —
// something typechecking alone cannot catch. The global setup boots the server.
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

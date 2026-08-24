import { defineConfig } from "vitest/config";

// Contract tests: exercise the typed API client against a real, freshly-booted
// structure-server, for what a type cannot carry — status codes, error bodies,
// the trees the server assembles — and for the encodings, which ts-rs
// re-implements rather than calling serde.
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

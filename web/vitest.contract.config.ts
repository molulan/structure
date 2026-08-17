import { defineConfig } from "vitest/config";

// Contract tests: exercise the typed API client against a real, freshly-booted
// structure-server. The wire shapes are generated from Rust into src/api/wire.ts,
// so these cover what a shape cannot state — status codes, error bodies, the
// trees the server assembles — plus the encodings themselves, since ts-rs
// re-implements serde's attributes rather than calling serde. The global setup
// boots the server.
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

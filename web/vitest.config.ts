import { defineConfig } from "vitest/config";

// Unit tests: pure logic, no backend, no DOM. Fast and hermetic.
export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});

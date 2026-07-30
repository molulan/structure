import { vi } from "vitest";
import type { ApiClient } from "../api/client";

/**
 * Every API method as a spy. Component tests mock the leaf `lib/apiClient`
 * module with this, so the hooks above it run for real:
 *
 *   vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
 *
 * The factory has to be imported inside `vi.mock`'s callback because the call
 * is hoisted above the file's imports.
 */
export function mockApiModule(): { api: ApiClient } {
  return {
    api: {
      listMesocycles: vi.fn(),
      createMesocycle: vi.fn(),
      getFullMesocycle: vi.fn(),
      addMicrocycle: vi.fn(),
      deleteMicrocycle: vi.fn(),
      setMicrocyclePhase: vi.fn(),
      addWorkout: vi.fn(),
      renameWorkout: vi.fn(),
      deleteWorkout: vi.fn(),
      addPlannedExercise: vi.fn(),
      deletePlannedExercise: vi.fn(),
      listLibraryExercises: vi.fn(),
      createLibraryExercise: vi.fn(),
    },
  };
}

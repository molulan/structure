import { beforeAll, describe, expect, it } from "vitest";
import { ApiError, createApiClient, type ApiClient } from "../../src/api/client";

// Drives the structure-editing mutations against a real server and reads each
// result back through the grid endpoint, so the request shapes and the
// cascading deletes the UI relies on are verified, not assumed.
let api: ApiClient;

beforeAll(() => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  api = createApiClient(baseUrl);
});

describe("week mutations contract", () => {
  it("adds, phases, and deletes a week", async () => {
    const mesocycle = await api.createMesocycle({ name: "Week Ops", mode: "Manual" });

    const week = await api.addMicrocycle(mesocycle.id);
    expect(week.position).toBe(0);
    expect(week.phase).toBeNull();

    await api.setMicrocyclePhase(week.id, "Deload");
    let full = await api.getFullMesocycle(mesocycle.id);
    expect(full.microcycles.find((m) => m.id === week.id)?.phase).toBe("Deload");

    // `null` clears it again — the UI's "No phase" option.
    await api.setMicrocyclePhase(week.id, null);
    full = await api.getFullMesocycle(mesocycle.id);
    expect(full.microcycles.find((m) => m.id === week.id)?.phase).toBeNull();

    await api.deleteMicrocycle(week.id);
    full = await api.getFullMesocycle(mesocycle.id);
    expect(full.microcycles).toHaveLength(0);
  });
});

describe("workout mutations contract", () => {
  it("adds, renames, and deletes a workout", async () => {
    const mesocycle = await api.createMesocycle({ name: "Workout Ops", mode: "Manual" });

    const workout = await api.addWorkout(mesocycle.id, "Push");
    expect(workout.name).toBe("Push");
    expect(workout.position).toBe(0);

    const renamed = await api.renameWorkout(workout.id, "Push A");
    expect(renamed.name).toBe("Push A");

    let full = await api.getFullMesocycle(mesocycle.id);
    expect(full.workouts.map((w) => w.name)).toEqual(["Push A"]);

    await api.deleteWorkout(workout.id);
    full = await api.getFullMesocycle(mesocycle.id);
    expect(full.workouts).toHaveLength(0);
  });
});

describe("failure reporting contract", () => {
  // The server answers `{"error": "…"}`; the client has to surface the message,
  // not the JSON envelope, since it goes straight on screen.
  it("reports a rejected request as the server's message", async () => {
    await api.createLibraryExercise({
      name: "Contract Duplicate Curl",
      exercise_type: "Weighted",
      primary_muscle_group: "Biceps",
    });

    const conflict = await api
      .createLibraryExercise({
        name: "Contract Duplicate Curl",
        exercise_type: "Weighted",
        primary_muscle_group: "Biceps",
      })
      .then(() => null)
      .catch((error: unknown) => error);

    expect(conflict).toBeInstanceOf(ApiError);
    const error = conflict as ApiError;
    expect(error.status).toBe(409);
    expect(error.message).not.toContain("{");
    expect(error.message.toLowerCase()).toContain("contract duplicate curl");
  });
});

describe("planned exercise mutations contract", () => {
  it("creates a library exercise, places it, and removes it again", async () => {
    const mesocycle = await api.createMesocycle({ name: "Exercise Ops", mode: "Manual" });
    const workout = await api.addWorkout(mesocycle.id, "Pull");

    // Library exercise names are globally unique, so keep this one to itself.
    const exercise = await api.createLibraryExercise({
      name: "Contract Barbell Row",
      exercise_type: "Weighted",
      primary_muscle_group: "Back",
    });
    expect(exercise.secondary_muscle_groups).toEqual([]);
    expect((await api.listLibraryExercises()).some((e) => e.id === exercise.id)).toBe(true);

    const planned = await api.addPlannedExercise(workout.id, exercise.id);
    expect(planned.exercise.name).toBe("Contract Barbell Row");

    let full = await api.getFullMesocycle(mesocycle.id);
    expect(full.workouts[0].planned_exercises.map((p) => p.id)).toEqual([planned.id]);

    await api.deletePlannedExercise(planned.id);
    full = await api.getFullMesocycle(mesocycle.id);
    expect(full.workouts[0].planned_exercises).toHaveLength(0);
  });
});

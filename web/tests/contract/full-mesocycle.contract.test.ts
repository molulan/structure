import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";
import { buildFullMesocycle } from "../support/seed";

// Builds a full mesocycle tree via the API, then asserts GET /mesocycles/{id}/full
// deserializes into our hand-written TS types with every leaf correctly encoded
// (snake_case keys, externally-tagged enums, bare-string unit variants).
let base: string;
let api: ApiClient;

beforeAll(() => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  base = baseUrl;
  api = createApiClient(baseUrl);
});

async function post(path: string, body?: unknown): Promise<{ id: number }> {
  const response = await fetch(`${base}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`POST ${path} → ${response.status}: ${await response.text()}`);
  }
  return response.json();
}

async function setPhase(microcycleId: number, phase: string): Promise<void> {
  const response = await fetch(`${base}/microcycles/${microcycleId}/phase`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ phase }),
  });
  if (!response.ok) {
    throw new Error(`PUT phase → ${response.status}: ${await response.text()}`);
  }
}

describe("full mesocycle contract", () => {
  it("returns the whole nested tree with correctly-encoded leaves", async () => {
    const tree = await buildFullMesocycle(post, "Full Tree");
    await setPhase(tree.microcycleId, "Accumulation");

    const full = await api.getFullMesocycle(tree.mesocycleId);

    expect(full.name).toBe("Full Tree");
    expect(full.mode).toBe("Manual");
    expect(full.microcycles).toHaveLength(1);

    const week = full.microcycles[0];
    expect(week.position).toBe(0);
    expect(week.phase).toBe("Accumulation");
    expect(week.workouts).toHaveLength(1);

    const workout = week.workouts[0];
    expect(workout.name).toBe("Push");
    expect(workout.planned_exercises).toHaveLength(1);

    const pe = workout.planned_exercises[0];
    expect(pe.exercise.name).toBe("Bench Press");
    expect(pe.exercise.exercise_type).toBe("Weighted");
    expect(pe.exercise.primary_muscle_group).toBe("Chest");
    expect([...pe.exercise.secondary_muscle_groups].sort()).toEqual(["Shoulders", "Triceps"]);

    expect(pe.set_groups).toHaveLength(3);
    expect(pe.set_groups[0].set_group_type).toEqual({
      Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } },
    });
    expect(pe.set_groups[1].set_group_type).toEqual({
      Prescribed: { set_type: "Regular", reps: { AtLeast: 12 }, intensity: { Rir: 0 } },
    });
    expect(pe.set_groups[2].set_group_type).toBe("MyorepMatch");
  });

  it("returns a phase of null for a microcycle with no phase set", async () => {
    const mesocycle = await post("/mesocycles", { name: "Phaseless", mode: "Manual" });
    const microcycle = await post(`/mesocycles/${mesocycle.id}/microcycles`);

    const full = await api.getFullMesocycle(mesocycle.id);

    expect(full.microcycles.find((m) => m.id === microcycle.id)?.phase).toBeNull();
  });
});

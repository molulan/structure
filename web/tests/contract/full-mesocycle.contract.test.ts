import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";
import { buildFullMesocycle } from "../support/seed";

// Builds a full mesocycle grid via the API, then asserts GET /mesocycles/{id}/full
// returns it. Two jobs, both still real now that src/api/wire.ts is generated:
// that the grid is assembled from the parts we posted — one cell per week in
// column order, including the week with nothing in it — and that the encodings
// spelled out below are what serde actually writes. ts-rs re-implements serde's
// attributes rather than calling it, so these leaves are what keeps the
// generator honest about externally-tagged enums and bare-string variants.
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
  it("returns the whole grid with correctly-encoded leaves", async () => {
    const tree = await buildFullMesocycle(post, "Full Tree");
    await setPhase(tree.microcycleIds[0], "Accumulation");

    const full = await api.getFullMesocycle(tree.mesocycleId);

    expect(full.name).toBe("Full Tree");
    expect(full.mode).toBe("Manual");

    // Weeks are the columns, workouts the row groups — siblings, not nested.
    expect(full.microcycles.map((m) => m.id)).toEqual(tree.microcycleIds);
    expect(full.microcycles.map((m) => m.position)).toEqual([0, 1, 2]);
    expect(full.microcycles[0].phase).toBe("Accumulation");

    expect(full.workouts).toHaveLength(1);
    const workout = full.workouts[0];
    expect(workout.name).toBe("Push");
    expect(workout.planned_exercises).toHaveLength(1);

    const pe = workout.planned_exercises[0];
    expect(pe.exercise.name).toBe("Bench Press");
    expect(pe.exercise.exercise_type).toBe("Weighted");
    expect(pe.exercise.primary_muscle_group).toBe("Chest");
    expect([...pe.exercise.secondary_muscle_groups].sort()).toEqual(["Shoulders", "Triceps"]);

    // One cell per week, in column order — including the week with nothing in it.
    expect(pe.prescriptions.map((p) => p.microcycle_id)).toEqual(tree.microcycleIds);
    const [first, second, third] = pe.prescriptions;

    expect(first.set_groups).toHaveLength(3);
    expect(first.set_groups[0].set_group_type).toEqual({
      Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } },
    });
    expect(first.set_groups[1].set_group_type).toEqual({
      Prescribed: { set_type: "Regular", reps: { AtLeast: 12 }, intensity: { Rir: 0 } },
    });
    expect(first.set_groups[2].set_group_type).toBe("MyorepMatch");

    expect(second.set_groups).toHaveLength(1);
    expect(second.set_groups[0].number_of_sets).toBe(4);

    expect(third.set_groups).toEqual([]);
  });

  it("returns a phase of null for a microcycle with no phase set", async () => {
    const mesocycle = await post("/mesocycles", { name: "Phaseless", mode: "Manual" });
    const microcycle = await post(`/mesocycles/${mesocycle.id}/microcycles`);

    const full = await api.getFullMesocycle(mesocycle.id);

    expect(full.microcycles.find((m) => m.id === microcycle.id)?.phase).toBeNull();
  });
});

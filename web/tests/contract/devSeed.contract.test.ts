import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";
import { seedDevPlan, SEEDED_EXERCISE_COUNT, SEEDED_WORKOUT_COUNT } from "../../scripts/seedDev";

// The dev stack's seed runs against a real server, so an endpoint or body-shape
// change breaks it — and the symptom would be `npm run app` quietly serving an
// empty app, which is a bad way to find out. Building the block here means the
// same gate that guards the client's types guards the data we develop against.

let api: ApiClient;
let planId: number;

beforeAll(async () => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  api = createApiClient(baseUrl);
  planId = await seedDevPlan(baseUrl);
}, 120_000);

describe("dev seed contract", () => {
  it("builds a four-week block with every workout and exercise", async () => {
    const plan = await api.getFullMesocycle(planId);

    expect(plan.microcycles).toHaveLength(4);
    expect(plan.microcycles.map((m) => m.phase)).toEqual([
      "Accumulation",
      "Accumulation",
      "Intensification",
      "Deload",
    ]);
    expect(plan.workouts).toHaveLength(SEEDED_WORKOUT_COUNT);

    const exercises = plan.workouts.flatMap((w) => w.planned_exercises);
    expect(exercises).toHaveLength(SEEDED_EXERCISE_COUNT);
  });

  it("prescribes cells covering every set-group encoding the grid can render", async () => {
    const plan = await api.getFullMesocycle(planId);
    const groups = plan.workouts
      .flatMap((w) => w.planned_exercises)
      .flatMap((e) => e.prescriptions)
      .flatMap((p) => p.set_groups);

    expect(groups.filter((g) => g.set_group_type === "MyorepMatch").length).toBeGreaterThan(0);

    const prescribed = groups
      .map((g) => (typeof g.set_group_type === "object" ? g.set_group_type.Prescribed : null))
      .filter((p) => p !== null);

    expect(prescribed.some((p) => "Range" in p.reps)).toBe(true);
    expect(prescribed.some((p) => "AtLeast" in p.reps)).toBe(true);
    expect(prescribed.some((p) => "Rir" in p.intensity)).toBe(true);
    expect(prescribed.some((p) => "Rpe" in p.intensity)).toBe(true);
  });

  it("leaves at least one cell unprescribed, so the empty state is on screen too", async () => {
    const plan = await api.getFullMesocycle(planId);
    const cells = plan.workouts
      .flatMap((w) => w.planned_exercises)
      .flatMap((e) => e.prescriptions);

    expect(cells.some((cell) => cell.set_groups.length === 0)).toBe(true);
  });
});

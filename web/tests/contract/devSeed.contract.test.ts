import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";
import {
  FIXTURE_PLAN_NAMES,
  REFERENCE_PLAN_NAME,
  seedDevPlan,
  SEEDED_EXERCISE_COUNT,
  SEEDED_WORKOUT_COUNT,
} from "../../scripts/seedDev";

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

  // These three are defined by what they *lack*, and nothing in the seed states
  // an absence — it simply never makes the call. A server that started handing
  // out a first week with every new mesocycle would empty them of their whole
  // point without one request failing, and they are no longer on the workbench
  // to be noticed missing.
  it("seeds the half-built plans the workbench no longer carries", async () => {
    const list = await api.listMesocycles();

    // Only the plans the seed is responsible for: this suite shares one server,
    // so the list also holds whatever the other contract tests created.
    const names = [REFERENCE_PLAN_NAME, ...Object.values(FIXTURE_PLAN_NAMES)] as string[];
    const seeded = list.filter((plan) => names.includes(plan.name));

    // Deliberately a literal rather than `names.length`: a derived count would
    // absorb a fifth fixture silently, and this is data everyone develops
    // against. It also catches the seed running twice and duplicating itself.
    expect(seeded).toHaveLength(4);

    const idOf = (name: string): number => {
      const plan = list.find((candidate) => candidate.name === name);
      if (!plan) throw new Error(`no seeded plan named "${name}"`);
      return plan.id;
    };

    const empty = await api.getFullMesocycle(idOf(FIXTURE_PLAN_NAMES.empty));
    expect(empty.microcycles).toHaveLength(0);
    expect(empty.workouts).toHaveLength(0);

    const exercisesFirst = await api.getFullMesocycle(idOf(FIXTURE_PLAN_NAMES.exercisesFirst));
    expect(exercisesFirst.microcycles).toHaveLength(0);
    expect(exercisesFirst.workouts.flatMap((w) => w.planned_exercises)).not.toHaveLength(0);

    const weeksFirst = await api.getFullMesocycle(idOf(FIXTURE_PLAN_NAMES.weeksFirst));
    expect(weeksFirst.microcycles).toHaveLength(3);
    expect(weeksFirst.workouts).toHaveLength(1);
    expect(weeksFirst.workouts[0].planned_exercises).toHaveLength(0);
  });

  it("leaves at least one cell unprescribed, so the empty state is on screen too", async () => {
    const plan = await api.getFullMesocycle(planId);
    const cells = plan.workouts
      .flatMap((w) => w.planned_exercises)
      .flatMap((e) => e.prescriptions);

    expect(cells.some((cell) => cell.set_groups.length === 0)).toBe(true);
  });
});

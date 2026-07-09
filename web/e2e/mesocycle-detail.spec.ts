import { test, expect, type APIRequestContext } from "@playwright/test";
import { mkdirSync } from "node:fs";

// The e2e backend has no editing UI yet, so seed a full tree straight through
// its API (same in-memory instance the app reads via the proxy), then verify the
// browser renders it.
const BACKEND = "http://127.0.0.1:3001";

async function seedFullMesocycle(request: APIRequestContext, name: string): Promise<void> {
  const post = async (path: string, data?: object): Promise<{ id: number }> => {
    const response = await request.post(`${BACKEND}${path}`, data ? { data } : {});
    expect(response.ok(), `POST ${path} → ${response.status()}`).toBeTruthy();
    return response.json();
  };

  const mesocycle = await post("/mesocycles", { name, mode: "Manual" });
  const microcycle = await post(`/mesocycles/${mesocycle.id}/microcycles`);
  const workout = await post(`/microcycles/${microcycle.id}/workouts`, { name: "Push" });
  const exercise = await post("/library-exercises", {
    name: "Bench Press",
    exercise_type: "Weighted",
    primary_muscle_group: "Chest",
  });
  const planned = await post(`/workouts/${workout.id}/planned-exercises`, {
    library_exercise_id: exercise.id,
  });
  await post(`/planned-exercises/${planned.id}/set-groups`, {
    number_of_sets: 3,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } },
    },
  });
}

test("opens a mesocycle and renders its full tree", async ({ page, request }) => {
  await seedFullMesocycle(request, "Detail Block");

  await page.goto("/");
  await page.getByRole("button", { name: /Detail Block/ }).click();

  await expect(page.getByRole("heading", { name: "Push" })).toBeVisible();
  await expect(page.getByText("Week 1")).toBeVisible();
  await expect(page.getByText("Bench Press")).toBeVisible();
  await expect(page.getByText("3 × 8–12 @ 2 RIR")).toBeVisible();

  mkdirSync("e2e/screenshots", { recursive: true });
  await page.screenshot({ path: "e2e/screenshots/mesocycle-detail.png", fullPage: true });

  await page.getByRole("button", { name: /All mesocycles/ }).click();
  await expect(page.getByLabel("New mesocycle name")).toBeVisible();
});

test("shows the empty state for a mesocycle with no weeks", async ({ page, request }) => {
  const response = await request.post(`${BACKEND}/mesocycles`, {
    data: { name: "Empty Block", mode: "Manual" },
  });
  expect(response.ok(), `POST /mesocycles → ${response.status()}`).toBeTruthy();

  await page.goto("/");
  await page.getByRole("button", { name: /Empty Block/ }).click();

  await expect(page.getByText("No weeks yet.")).toBeVisible();
});

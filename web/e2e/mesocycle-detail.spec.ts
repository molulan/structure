import { test, expect, type APIRequestContext } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { buildFullMesocycle, type Post } from "../tests/support/seed";

// Seeds through the API (the same in-memory instance the app reads via the
// proxy) and checks what the browser renders. The editing loop is driven
// through the UI instead, in plan-editing.spec.ts.
const BACKEND = "http://127.0.0.1:3001/api";

function apiPost(request: APIRequestContext): Post {
  return async (path, body) => {
    const response = await request.post(`${BACKEND}${path}`, body ? { data: body } : {});
    expect(response.ok(), `POST ${path} → ${response.status()}`).toBeTruthy();
    return response.json();
  };
}

async function setPhase(request: APIRequestContext, microcycleId: number, phase: string) {
  const response = await request.put(`${BACKEND}/microcycles/${microcycleId}/phase`, {
    data: { phase },
  });
  expect(response.ok(), `PUT phase → ${response.status()}`).toBeTruthy();
}

test("opens a mesocycle and renders its grid", async ({ page, request }) => {
  const { mesocycleId, microcycleIds } = await buildFullMesocycle(apiPost(request), "Detail Block");
  await setPhase(request, microcycleIds[0], "Accumulation");
  await setPhase(request, microcycleIds[2], "Deload");

  await page.goto("/");
  await page.getByRole("button", { name: /Detail Block/ }).click();

  await expect(page).toHaveURL(new RegExp(`/mesocycles/${mesocycleId}$`));
  await expect(page.getByRole("heading", { name: "Push" })).toBeVisible();
  await expect(page.getByRole("columnheader", { name: /Week 1/ })).toBeVisible();
  await expect(page.getByRole("columnheader", { name: /Week 3/ })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Phase for Week 1" })).toHaveValue("Accumulation");
  await expect(page.getByRole("combobox", { name: "Phase for Week 3" })).toHaveValue("Deload");
  await expect(page.getByRole("rowheader", { name: /Bench Press/ })).toBeVisible();

  // Week 1's cell carries its three set groups; week 3 was never prescribed.
  const row = page.getByRole("row").filter({ has: page.getByRole("rowheader", { name: /Bench Press/ }) });
  const cells = row.getByRole("cell");
  await expect(cells.nth(0)).toContainText("3×8–12 RIR2");
  await expect(cells.nth(0)).toContainText("2× match");
  await expect(cells.nth(1)).toContainText("4×8 RIR1");
  await expect(cells.nth(2)).toContainText("—");
  await expect(cells.nth(2)).toContainText("Not prescribed");

  mkdirSync("e2e/screenshots", { recursive: true });
  await page.screenshot({ path: "e2e/screenshots/mesocycle-detail.png", fullPage: true });

  await page.getByRole("button", { name: /All mesocycles/ }).click();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByLabel("New mesocycle name")).toBeVisible();
});

test("deep-links to a mesocycle and survives a reload", async ({ page, request }) => {
  // A minimal tree (no library exercise) is enough to prove the URL renders the
  // right detail view — and it sidesteps the shared backend's globally-unique
  // library-exercise names that a second full-tree seed would collide on.
  // No microcycle: structure hangs off the mesocycle, so the workout has to
  // render before any week exists.
  const post = apiPost(request);
  const mesocycle = await post("/mesocycles", { name: "Deep Link Block", mode: "Manual" });
  await post(`/mesocycles/${mesocycle.id}/workouts`, { name: "Deep Squat Day" });

  await page.goto(`/mesocycles/${mesocycle.id}`);
  await expect(page.getByRole("heading", { name: "Deep Squat Day" })).toBeVisible();
  await expect(page.getByText("No weeks yet.")).toBeVisible();

  await page.reload();
  await expect(page.getByRole("heading", { name: "Deep Squat Day" })).toBeVisible();
});

test("shows the empty state for a mesocycle with no weeks", async ({ page, request }) => {
  await apiPost(request)("/mesocycles", { name: "Empty Block", mode: "Manual" });

  await page.goto("/");
  await page.getByRole("button", { name: /Empty Block/ }).click();

  await expect(page.getByText("No weeks yet.")).toBeVisible();
});

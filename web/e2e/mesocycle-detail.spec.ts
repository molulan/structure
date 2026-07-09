import { test, expect, type APIRequestContext } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { buildFullMesocycle, type Post } from "../tests/support/seed";

// The e2e backend has no editing UI yet, so seed through its API (the same
// in-memory instance the app reads via the proxy), then verify the browser
// renders it.
const BACKEND = "http://127.0.0.1:3001";

function apiPost(request: APIRequestContext): Post {
  return async (path, body) => {
    const response = await request.post(`${BACKEND}${path}`, body ? { data: body } : {});
    expect(response.ok(), `POST ${path} → ${response.status()}`).toBeTruthy();
    return response.json();
  };
}

test("opens a mesocycle and renders its full tree", async ({ page, request }) => {
  await buildFullMesocycle(apiPost(request), "Detail Block");

  await page.goto("/");
  await page.getByRole("button", { name: /Detail Block/ }).click();

  await expect(page.getByRole("heading", { name: "Push" })).toBeVisible();
  await expect(page.getByText("Week 1")).toBeVisible();
  await expect(page.getByText("Bench Press")).toBeVisible();
  await expect(page.getByText("3×8–12 RIR2")).toBeVisible();

  mkdirSync("e2e/screenshots", { recursive: true });
  await page.screenshot({ path: "e2e/screenshots/mesocycle-detail.png", fullPage: true });

  await page.getByRole("button", { name: /All mesocycles/ }).click();
  await expect(page.getByLabel("New mesocycle name")).toBeVisible();
});

test("shows the empty state for a mesocycle with no weeks", async ({ page, request }) => {
  await apiPost(request)("/mesocycles", { name: "Empty Block", mode: "Manual" });

  await page.goto("/");
  await page.getByRole("button", { name: /Empty Block/ }).click();

  await expect(page.getByText("No weeks yet.")).toBeVisible();
});

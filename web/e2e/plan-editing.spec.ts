import { test, expect } from "@playwright/test";
import { mkdirSync } from "node:fs";

// Builds a plan through the UI alone — no API seeding — so the whole editing
// loop is covered: create, edit, reload (it persisted), then tear back down.
// Library exercise names are globally unique on the shared backend, so this
// spec's exercise name is its own.
const EXERCISE = "E2E Overhead Press";

test("builds a plan's structure through the grid", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("New mesocycle name").fill("Editing Block");
  await page.getByRole("button", { name: "Create" }).click();
  await page.getByRole("button", { name: /Editing Block/ }).click();

  await expect(page.getByText("No weeks yet.")).toBeVisible();

  await page.getByRole("button", { name: "+ Week" }).click();
  await expect(page.getByRole("columnheader", { name: /Week 1/ })).toBeVisible();

  const phase = page.getByRole("combobox", { name: "Phase for Week 1" });
  await phase.selectOption("Deload");
  await expect(phase).toHaveValue("Deload");

  await page.getByLabel("New workout name").fill("Push Day");
  await page.getByRole("button", { name: "+ Workout" }).click();
  await expect(page.getByRole("heading", { name: "Push Day" })).toBeVisible();
  await expect(page.getByText("No exercises yet.")).toBeVisible();

  await page.getByRole("button", { name: "New exercise…" }).click();
  await page.getByLabel("Exercise name").fill(EXERCISE);
  await page.getByLabel("Primary muscle group").selectOption("Shoulders");
  await page.getByRole("button", { name: "Create and add" }).click();

  const row = page.getByRole("rowheader", { name: new RegExp(EXERCISE) });
  await expect(row).toBeVisible();
  await expect(row).toContainText("Shoulders");
  // Nothing is prescribed yet — cell editing is still to come.
  await expect(page.getByRole("row").filter({ has: row }).getByRole("cell")).toContainText("—");

  await page.reload();
  await expect(page.getByRole("rowheader", { name: new RegExp(EXERCISE) })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Phase for Week 1" })).toHaveValue("Deload");

  mkdirSync("e2e/screenshots", { recursive: true });
  await page.screenshot({ path: "e2e/screenshots/plan-editing.png", fullPage: true });
});

test("takes a plan's structure back apart", async ({ page }) => {
  page.on("dialog", (dialog) => dialog.accept());

  await page.goto("/");
  await page.getByLabel("New mesocycle name").fill("Teardown Block");
  await page.getByRole("button", { name: "Create" }).click();
  await page.getByRole("button", { name: /Teardown Block/ }).click();

  await page.getByRole("button", { name: "+ Week" }).click();
  await page.getByLabel("New workout name").fill("Leg Day");
  await page.getByRole("button", { name: "+ Workout" }).click();
  await expect(page.getByRole("heading", { name: "Leg Day" })).toBeVisible();

  await page.getByRole("button", { name: "Rename Leg Day" }).click();
  const name = page.getByLabel("Workout name", { exact: true });
  await name.fill("Leg Day A");
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByRole("heading", { name: "Leg Day A" })).toBeVisible();

  await page.getByRole("button", { name: "Delete Leg Day A" }).click();
  await expect(page.getByText("No workouts yet.")).toBeVisible();

  await page.getByRole("button", { name: "Delete Week 1" }).click();
  await expect(page.getByText("No weeks yet.")).toBeVisible();
});

import { test, expect } from "@playwright/test";
import { mkdirSync } from "node:fs";

test("create a mesocycle and see it in the list", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByText("No mesocycles yet.")).toBeVisible();

  await page.getByLabel("New mesocycle name").fill("Summer Hypertrophy");
  await page.getByLabel("Mode").selectOption("Algorithmic");
  await page.getByRole("button", { name: "Create" }).click();

  const item = page.getByRole("listitem").filter({ hasText: "Summer Hypertrophy" });
  await expect(item).toBeVisible();
  await expect(item).toContainText("Algorithmic");
  await expect(item).toContainText("0 weeks");

  mkdirSync("e2e/screenshots", { recursive: true });
  await page.screenshot({ path: "e2e/screenshots/mesocycle-created.png", fullPage: true });
});

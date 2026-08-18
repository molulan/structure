import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";

let api: ApiClient;

beforeAll(() => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  api = createApiClient(baseUrl);
});

describe("mesocycles contract", () => {
  it("returns the created mesocycle with the requested fields", async () => {
    const created = await api.createMesocycle({ name: "Contract Block", mode: "Manual" });
    expect(created.id).toBeGreaterThan(0);
    expect(created.name).toBe("Contract Block");
    expect(created.mode).toBe("Manual");
  });

  it("lists a created mesocycle with its computed week count", async () => {
    const created = await api.createMesocycle({ name: "Listed Block", mode: "Algorithmic" });
    const list = await api.listMesocycles();

    expect(Array.isArray(list)).toBe(true);
    const found = list.find((m) => m.id === created.id);
    expect(found).toBeDefined();
    expect(found?.mode).toBe("Algorithmic");
    expect(found?.microcycle_count).toBe(0);
  });

  it("rejects an empty name with 422", async () => {
    await expect(api.createMesocycle({ name: "", mode: "Manual" })).rejects.toMatchObject({
      status: 422,
    });
  });
});

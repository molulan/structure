import { beforeAll, describe, expect, it } from "vitest";
import { createApiClient, type ApiClient } from "../../src/api/client";

// Verifies the typed client against a real server: field names, enum encoding,
// and status codes. If our TS types drift from the server's wire format, these
// fail where typechecking would stay green.
let api: ApiClient;

beforeAll(() => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  api = createApiClient(baseUrl);
});

describe("mesocycles contract", () => {
  it("starts empty", async () => {
    expect(await api.listMesocycles()).toEqual([]);
  });

  it("create returns the bare mesocycle with the requested fields", async () => {
    const created = await api.createMesocycle({ name: "Contract Block", mode: "Manual" });
    expect(created.id).toBeGreaterThan(0);
    expect(created.name).toBe("Contract Block");
    expect(created.mode).toBe("Manual");
  });

  it("a created mesocycle appears in the list with a computed week count", async () => {
    const created = await api.createMesocycle({ name: "Listed Block", mode: "Algorithmic" });
    const list = await api.listMesocycles();

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

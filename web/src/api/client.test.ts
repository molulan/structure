import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, createApiClient } from "./client";

function mockFetch(response: Partial<Response> & { json?: () => Promise<unknown> }) {
  const fn = vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    statusText: "OK",
    json: async () => [],
    text: async () => "",
    ...response,
  });
  vi.stubGlobal("fetch", fn);
  return fn;
}

afterEach(() => vi.unstubAllGlobals());

describe("createApiClient", () => {
  it("prefixes the base URL and requests JSON", async () => {
    const fetchMock = mockFetch({ json: async () => [] });
    await createApiClient("/api").listMesocycles();

    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("/api/mesocycles");
    expect((init.headers as Record<string, string>)["Content-Type"]).toBe("application/json");
  });

  it("POSTs the serialized body on create", async () => {
    const fetchMock = mockFetch({ status: 201, json: async () => ({ id: 1, name: "Blk", mode: "Manual" }) });
    await createApiClient("/api").createMesocycle({ name: "Blk", mode: "Manual" });

    const [, init] = fetchMock.mock.calls[0];
    expect(init.method).toBe("POST");
    expect(JSON.parse(init.body as string)).toEqual({ name: "Blk", mode: "Manual" });
  });

  it("throws ApiError carrying the status and body on a non-2xx response", async () => {
    mockFetch({ ok: false, status: 422, text: async () => "name must not be empty" });

    await expect(createApiClient("/api").createMesocycle({ name: "", mode: "Manual" })).rejects.toMatchObject({
      constructor: ApiError,
      status: 422,
      message: "name must not be empty",
    });
  });
});

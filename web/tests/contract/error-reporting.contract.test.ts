import { beforeAll, describe, expect, it } from "vitest";
import { ApiError, createApiClient, type ApiClient } from "../../src/api/client";

let api: ApiClient;

beforeAll(() => {
  const baseUrl = process.env.STRUCTURE_API_BASE;
  if (!baseUrl) {
    throw new Error("STRUCTURE_API_BASE not set — global setup did not run");
  }
  api = createApiClient(baseUrl);
});

describe("failure reporting contract", () => {
  it("shows the server's message, not the JSON", async () => {
    await api.createLibraryExercise({
      name: "Contract Duplicate Curl",
      exercise_type: "Weighted",
      primary_muscle_group: "Biceps",
    });

    const conflict = await api
      .createLibraryExercise({
        name: "Contract Duplicate Curl",
        exercise_type: "Weighted",
        primary_muscle_group: "Biceps",
      })
      .then(() => null)
      .catch((error: unknown) => error);

    expect(conflict).toBeInstanceOf(ApiError);
    const error = conflict as ApiError;
    expect(error.status).toBe(409);
    expect(error.message).not.toContain("{");
    expect(error.message.toLowerCase()).toContain("contract duplicate curl");
  });
});

import type {
  CreateLibraryExercise,
  CreateMesocycle,
  FullMesocycle,
  LibraryExercise,
  Mesocycle,
  MesocycleRow,
  Microcycle,
  Phase,
  PlannedExercise,
  Workout,
} from "./types";

/** A non-2xx response from the API, carrying the status and the server's message. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

// The server reports failures as `{"error": "…"}`; anything else (a proxy's
// HTML error page, an empty body) is shown as-is rather than guessed at.
function messageFromBody(body: string): string {
  try {
    const parsed: unknown = JSON.parse(body);
    if (parsed && typeof parsed === "object" && "error" in parsed) {
      const { error } = parsed as { error: unknown };
      if (typeof error === "string") return error;
    }
  } catch {
    // not JSON
  }
  return body;
}

/** What to show a user for any thrown value — API failures and transport ones alike. */
export function describeError(error: unknown): string {
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) return error.message;
  return String(error);
}

async function request<T>(
  baseUrl: string,
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  if (!response.ok) {
    const body = await response.text();
    throw new ApiError(response.status, messageFromBody(body) || response.statusText);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}

/**
 * Builds a typed client bound to a base URL. The browser app binds it to `/api`
 * (proxied to the backend); the contract tests bind it to a real server's URL.
 */
export function createApiClient(baseUrl: string) {
  return {
    listMesocycles: () => request<MesocycleRow[]>(baseUrl, "/mesocycles"),
    createMesocycle: (body: CreateMesocycle) =>
      request<Mesocycle>(baseUrl, "/mesocycles", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    getFullMesocycle: (id: number) =>
      request<FullMesocycle>(baseUrl, `/mesocycles/${id}/full`),

    addMicrocycle: (mesocycleId: number) =>
      request<Microcycle>(baseUrl, `/mesocycles/${mesocycleId}/microcycles`, { method: "POST" }),
    deleteMicrocycle: (microcycleId: number) =>
      request<void>(baseUrl, `/microcycles/${microcycleId}`, { method: "DELETE" }),
    // `null` clears the phase.
    setMicrocyclePhase: (microcycleId: number, phase: Phase | null) =>
      request<void>(baseUrl, `/microcycles/${microcycleId}/phase`, {
        method: "PUT",
        body: JSON.stringify({ phase }),
      }),

    addWorkout: (mesocycleId: number, name: string) =>
      request<Workout>(baseUrl, `/mesocycles/${mesocycleId}/workouts`, {
        method: "POST",
        body: JSON.stringify({ name }),
      }),
    renameWorkout: (workoutId: number, name: string) =>
      request<Workout>(baseUrl, `/workouts/${workoutId}`, {
        method: "PUT",
        body: JSON.stringify({ name }),
      }),
    deleteWorkout: (workoutId: number) =>
      request<void>(baseUrl, `/workouts/${workoutId}`, { method: "DELETE" }),

    addPlannedExercise: (workoutId: number, libraryExerciseId: number) =>
      request<PlannedExercise>(baseUrl, `/workouts/${workoutId}/planned-exercises`, {
        method: "POST",
        body: JSON.stringify({ library_exercise_id: libraryExerciseId }),
      }),
    deletePlannedExercise: (plannedExerciseId: number) =>
      request<void>(baseUrl, `/planned-exercises/${plannedExerciseId}`, { method: "DELETE" }),

    listLibraryExercises: () => request<LibraryExercise[]>(baseUrl, "/library-exercises"),
    createLibraryExercise: (body: CreateLibraryExercise) =>
      request<LibraryExercise>(baseUrl, "/library-exercises", {
        method: "POST",
        body: JSON.stringify(body),
      }),
  };
}

export type ApiClient = ReturnType<typeof createApiClient>;

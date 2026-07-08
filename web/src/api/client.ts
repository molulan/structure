import type { CreateMesocycle, FullMesocycle, Mesocycle, MesocycleRow } from "./types";

/** A non-2xx response from the API, carrying the status and the server's body. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
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
    throw new ApiError(response.status, (await response.text()) || response.statusText);
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
  };
}

export type ApiClient = ReturnType<typeof createApiClient>;

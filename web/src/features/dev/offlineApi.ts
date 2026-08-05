import type { ApiClient } from "../../api/client";

/**
 * A client that cannot reach the backend.
 *
 * Every stage mounts the real controls, so each one is a click away from a
 * request. Substituting the client is what makes that safe, and it is the only
 * thing that does: an id no mesocycle has still leaves `createLibraryExercise`
 * writing a real row, and still leaves a delete landing on whatever `dev.db`
 * happens to hold under a fixture's id.
 */
export const offlineApi = new Proxy({} as ApiClient, {
  get(_target, property) {
    // Only methods are answered. Returning a function for every name would make
    // this thenable, and an awaited client resolves to something unrecognisable
    // instead of failing.
    if (typeof property !== "string" || property === "then") return undefined;
    return () =>
      Promise.reject(new Error(`The workbench has no backend, so ${property} was not sent.`));
  },
});

import { createContext, useContext, type ReactNode } from "react";
import type { ApiClient } from "../api/client";
import { api } from "./apiClient";

const ApiContext = createContext<ApiClient>(api);

/**
 * Substitutes the API client for everything below.
 *
 * The app itself has one client and never mounts this. The dev workbench does:
 * it renders the real controls in a real browser, where a module mock isn't
 * available, and a control that reached the backend would edit real data.
 */
export function ApiProvider({ client, children }: { client: ApiClient; children: ReactNode }) {
  return <ApiContext.Provider value={client}>{children}</ApiContext.Provider>;
}

/** The client in scope: the app's own, unless a provider has substituted it. */
export function useApi(): ApiClient {
  return useContext(ApiContext);
}

import { createApiClient } from "../api/client";

// In the browser, calls go to `/api`, which Vite proxies to the backend.
export const api = createApiClient(import.meta.env.VITE_API_BASE ?? "/api");

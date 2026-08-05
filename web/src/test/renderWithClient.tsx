import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, type RenderResult } from "@testing-library/react";
import type { ReactElement } from "react";

// Retries off, so an errored query settles immediately instead of retrying on a
// timer during the test.
export function testClient(): QueryClient {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

// The default is a call rather than a shared value, so every render still gets a
// cache of its own. A test that needs one primed — the only way to reach a query
// holding stale data while in error — builds it with `testClient` and passes it.
export function renderWithClient(ui: ReactElement, client = testClient()): RenderResult {
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

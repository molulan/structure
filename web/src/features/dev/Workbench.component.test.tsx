import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
import { Workbench } from "./Workbench";

const mockApi = vi.mocked(api);

describe("Workbench", () => {
  // The page's one load-bearing rule. It has to render with the backend
  // stopped, and `npm run shot` reports any 4xx as a failure — so a stage that
  // reached for the network would break the screenshot loop this page exists to
  // serve, at the moment someone is using it to look at something else.
  it("renders every stage without calling the API", () => {
    render(<Workbench />);

    for (const [name, method] of Object.entries(mockApi)) {
      expect(method, `${name} was called`).not.toHaveBeenCalled();
    }
  });

  it("shows the failure states that seed data cannot reach", () => {
    render(<Workbench />);

    const banners = screen.getAllByRole("alert");
    expect(banners.map((banner) => banner.textContent)).toEqual([
      expect.stringContaining('A workout named "Push" already exists'),
      expect.stringContaining("Failed to fetch"),
    ]);

    // An exhausted library and an empty one are distinct states that the picker
    // currently renders identically; both stages exist to keep that visible.
    expect(screen.getAllByRole("combobox", { name: /Exercise to add to/ }).length).toBeGreaterThan(
      0,
    );
    expect(screen.getAllByText("No exercises yet.")).not.toHaveLength(0);
    expect(screen.getAllByText("No weeks yet.")).not.toHaveLength(0);
    expect(screen.getAllByText("No workouts yet.")).not.toHaveLength(0);
  });
});

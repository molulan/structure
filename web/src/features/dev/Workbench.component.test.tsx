import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
import { Workbench } from "./Workbench";

const mockApi = vi.mocked(api);

// `mockApi` is the app's own client — the singleton every screen outside this
// page uses. Asserting it stays untouched is the real invariant: the workbench
// substitutes `offlineApi` for its stages, and anything reaching past that
// substitution lands here instead.
function expectNothingSent() {
  for (const [name, method] of Object.entries(mockApi)) {
    expect(method, `${name} was called`).not.toHaveBeenCalled();
  }
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Workbench", () => {
  // The page's one load-bearing rule. It has to render with the backend
  // stopped, and `npm run shot` reports any 4xx as a failure — so a stage that
  // reached for the network would break the screenshot loop this page exists to
  // serve, at the moment someone is using it to look at something else.
  it("renders every stage without calling the API", () => {
    render(<Workbench />);

    expectNothingSent();
  });

  // Rendering was never the risky half: every control here is a real one, and a
  // sentinel mesocycle id only ever covered the two mutations that take one.
  it("sends nothing when its live controls are pressed", async () => {
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<Workbench />);

    // Addressed by the workout's own id, so the sentinel never applied to it.
    await userEvent.click(screen.getByRole("button", { name: /^Delete Lower Body/ }));

    // Not scoped to a mesocycle at all: this one wrote a real library row.
    await userEvent.click(screen.getAllByRole("button", { name: "New exercise…" })[0]);
    await userEvent.type(screen.getByLabelText("Exercise name"), "Zercher Squat");
    await userEvent.click(screen.getByRole("button", { name: "Create and add" }));

    expectNothingSent();
    confirm.mockRestore();
  });

  // The page's chrome labels its specimens without ranking above them: a
  // `WorkoutBand` heads itself with an `<h2>`, so a stage `<h3>` around it would
  // invert the outline on the one page meant for judging how these read.
  it("leaves the heading outline to the components under test", () => {
    render(<Workbench />);

    const levels = screen.getAllByRole("heading").map((heading) => Number(heading.tagName[1]));

    expect(levels[0]).toBe(1);
    expect(levels.slice(1)).not.toHaveLength(0);
    expect(levels.slice(1).every((level) => level === 2)).toBe(true);
  });

  it("shows the states a database has no way to hold", () => {
    render(<Workbench />);

    // A rejected edit and a transport failure — no row can express either.
    expect(screen.getAllByRole("alert").map((banner) => banner.textContent)).toEqual([
      expect.stringContaining('A workout named "Push" already exists'),
      expect.stringContaining("Failed to fetch"),
    ]);

    // An empty library can't be seeded beside a plan that references one, so
    // this page is where the two ways of having nothing to offer are read
    // against each other — they must not say the same thing.
    const group = screen.getByRole("region", { name: "AddExercise" });
    const pickers = within(group).getAllByRole("combobox", { name: /Exercise to add to/ });
    const placeholders = pickers.map((picker) => picker.querySelector("option")?.textContent);
    expect(placeholders).toEqual([
      "Pick an exercise…",
      "All exercises added",
      "No exercises yet — create one",
    ]);

    // Adversarial input rather than a state: seeding it would spoil every other
    // screenshot of the demo plan.
    expect(screen.getByRole("heading", { name: /Hamstring and Glute Emphasis/ })).toBeInTheDocument();
  });

  // Dismiss is one of the states these stages exist to show, so the banner has
  // to be restorable — otherwise the first click empties the stage for good.
  it("raises a dismissed banner again", async () => {
    render(<Workbench />);

    const stage = screen.getByRole("article", { name: "Request failed" });
    await userEvent.click(within(stage).getByRole("button", { name: "Dismiss" }));
    expect(within(stage).queryByRole("alert")).not.toBeInTheDocument();

    await userEvent.click(within(stage).getByRole("button", { name: "Raise again" }));
    expect(within(stage).getByRole("alert")).toHaveTextContent(
      'A workout named "Push" already exists',
    );
  });
});

import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { ApiError } from "../../api/client";
import { pushWorkout, weeks } from "../../test/planFixtures";
import { MesocycleGrid } from "./MesocycleGrid";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const MESOCYCLE_ID = 7;

function renderGrid(props: Partial<Parameters<typeof MesocycleGrid>[0]> = {}) {
  return renderWithClient(
    <MesocycleGrid
      mesocycleId={MESOCYCLE_ID}
      microcycles={weeks}
      workouts={[pushWorkout]}
      {...props}
    />,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.listLibraryExercises.mockResolvedValue([]);
});

describe("MesocycleGrid layout", () => {
  // Workouts hang off the mesocycle, not the microcycle, so a week-less
  // mesocycle can still hold structure — it must not be rendered as empty.
  it("still renders the workouts when there are no weeks", () => {
    renderGrid({ microcycles: [] });

    expect(screen.getByText("No weeks yet.")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Push" })).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Bench Press/ })).toBeInTheDocument();
  });

  it("renders a column per week, numbered from one", () => {
    renderGrid();

    const headers = screen.getAllByRole("columnheader");
    expect(headers[0]).toHaveTextContent("Week 1");
    expect(headers[1]).toHaveTextContent("Week 2");
  });

  // Deleting a week leaves a gap in the stored positions, so the label has to
  // come from the column's place in the row, not from `position`.
  it("numbers the columns by position in the grid, not by stored position", () => {
    renderGrid({
      microcycles: [
        { id: 10, position: 0, phase: null },
        { id: 12, position: 2, phase: null },
      ],
    });

    const headers = screen.getAllByRole("columnheader");
    expect(headers[0]).toHaveTextContent("Week 1");
    expect(headers[1]).toHaveTextContent("Week 2");
    expect(screen.getByRole("button", { name: "Delete Week 2" })).toBeInTheDocument();
  });

  it("keeps the phase in the column header's accessible name", () => {
    renderGrid();

    expect(screen.getByRole("columnheader", { name: "Week 1, Accumulation" })).toBeInTheDocument();
  });

  // A phase the backend knows and this client does not must still read as itself.
  it("displays an unrecognised phase rather than showing it as unset", () => {
    renderGrid({ microcycles: [{ id: 10, position: 0, phase: "Taper" as never }] });

    expect(screen.getByRole("combobox", { name: "Phase for Week 1" })).toHaveValue("Taper");
  });

  it("shows each week's phase in its own control", () => {
    renderGrid();

    expect(screen.getByRole("combobox", { name: "Phase for Week 1" })).toHaveValue("Accumulation");
    expect(screen.getByRole("combobox", { name: "Phase for Week 2" })).toHaveValue("");
  });

  it("shows a placeholder for a mesocycle with weeks but no workouts", () => {
    renderGrid({ workouts: [] });

    expect(screen.getByText("No workouts yet.")).toBeInTheDocument();
    expect(screen.getAllByRole("columnheader")).toHaveLength(2);
  });

  it("makes the scrolling grid reachable by keyboard", () => {
    renderGrid();

    expect(screen.getByRole("region", { name: "Plan grid" })).toHaveAttribute("tabindex", "0");
  });
});

describe("MesocycleGrid week editing", () => {
  it("adds a week", async () => {
    mockApi.addMicrocycle.mockResolvedValue({ id: 12, position: 2, phase: null });
    renderGrid();

    await userEvent.click(screen.getByRole("button", { name: "+ Week" }));

    expect(mockApi.addMicrocycle).toHaveBeenCalledWith(MESOCYCLE_ID);
  });

  it("sets and clears a week's phase", async () => {
    mockApi.setMicrocyclePhase.mockResolvedValue(undefined);
    renderGrid();

    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Phase for Week 2" }),
      "Deload",
    );
    expect(mockApi.setMicrocyclePhase).toHaveBeenCalledWith(11, "Deload");

    await userEvent.selectOptions(
      screen.getByRole("combobox", { name: "Phase for Week 1" }),
      "No phase",
    );
    expect(mockApi.setMicrocyclePhase).toHaveBeenCalledWith(10, null);
  });

  // A rejected edit that says nothing reads as a control that ignores input.
  it("reports a rejected edit and keeps it dismissable", async () => {
    mockApi.addMicrocycle.mockRejectedValue(new ApiError(500, "database is locked"));
    renderGrid();

    await userEvent.click(screen.getByRole("button", { name: "+ Week" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("database is locked");

    await userEvent.click(within(alert).getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("clears a stale failure once an edit succeeds", async () => {
    mockApi.addMicrocycle.mockRejectedValueOnce(new ApiError(500, "database is locked"));
    renderGrid();

    await userEvent.click(screen.getByRole("button", { name: "+ Week" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();

    mockApi.addMicrocycle.mockResolvedValue({ id: 12, position: 2, phase: null });
    await userEvent.click(screen.getByRole("button", { name: "+ Week" }));

    await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
  });

  it("holds the chosen phase until the server answers", async () => {
    let settle: (() => void) | undefined;
    mockApi.setMicrocyclePhase.mockReturnValue(
      new Promise<void>((resolve) => {
        settle = () => resolve();
      }),
    );
    renderGrid();

    const phase = screen.getByRole("combobox", { name: "Phase for Week 2" });
    await userEvent.selectOptions(phase, "Deload");

    // The server data still says "no phase"; the control must not snap back.
    expect(phase).toHaveValue("Deload");
    settle?.();
  });

  it("deletes a week only once the destruction is confirmed", async () => {
    mockApi.deleteMicrocycle.mockResolvedValue(undefined);
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
    renderGrid();

    await userEvent.click(screen.getByRole("button", { name: "Delete Week 1" }));
    expect(mockApi.deleteMicrocycle).not.toHaveBeenCalled();

    confirm.mockReturnValue(true);
    await userEvent.click(screen.getByRole("button", { name: "Delete Week 1" }));
    expect(mockApi.deleteMicrocycle).toHaveBeenCalledWith(10);

  });
});

describe("MesocycleGrid workout adding", () => {
  it("adds a workout and clears the field", async () => {
    mockApi.addWorkout.mockResolvedValue({ id: 101, name: "Pull", position: 1 });
    renderGrid();

    const field = screen.getByLabelText("New workout name");
    await userEvent.type(field, "Pull");
    await userEvent.click(screen.getByRole("button", { name: "+ Workout" }));

    expect(mockApi.addWorkout).toHaveBeenCalledWith(MESOCYCLE_ID, "Pull");
    await waitFor(() => expect(field).toHaveValue(""));
  });

  // A submit that silently does nothing is indistinguishable from a dead button.
  it("cannot be submitted with a blank workout name", async () => {
    renderGrid();

    await userEvent.type(screen.getByLabelText("New workout name"), "   ");

    expect(screen.getByRole("button", { name: "+ Workout" })).toBeDisabled();
    expect(mockApi.addWorkout).not.toHaveBeenCalled();
  });
});

describe("MesocycleGrid cells", () => {
  it("puts each week's set groups in that week's cell, and a dash where nothing is prescribed", () => {
    renderGrid();

    const row = screen.getByRole("rowheader", { name: /Bench Press/ }).closest("tr");
    const cells = within(row!).getAllByRole("cell");
    expect(cells[0]).toHaveTextContent("3×8–12 RIR2");
    expect(cells[0]).toHaveTextContent("2× match");
    expect(cells[1]).toHaveTextContent("—");
    expect(within(cells[1]).getByText("Not prescribed")).toBeInTheDocument();
  });
});

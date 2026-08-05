import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { pushWorkout, weeks } from "../../fixtures/plan";
import type { FullWorkout } from "../../api/types";
import { ApiError } from "../../api/client";
import { GridFrame } from "./GridFrame";
import { PlanErrors } from "./PlanErrors";
import { WorkoutBand } from "./WorkoutBand";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const MESOCYCLE_ID = 7;

// A tbody needs a table around it to render at all — the app's own, not one
// built here. A hand-made `<table>` has no header row, which is what sizes the
// columns, so these tests would keep passing against a shape the app dropped.
function renderBand(workout: FullWorkout = pushWorkout) {
  return renderWithClient(
    <PlanErrors>
      <GridFrame mesocycleId={MESOCYCLE_ID} microcycles={weeks}>
        <WorkoutBand mesocycleId={MESOCYCLE_ID} workout={workout} microcycles={weeks} />
      </GridFrame>
    </PlanErrors>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.listLibraryExercises.mockResolvedValue([]);
});

describe("WorkoutBand", () => {
  it("heads its row group with the workout name and exercise count", () => {
    renderBand();

    const band = screen.getByRole("rowheader", { name: /Push/ });
    expect(within(band).getByRole("heading", { name: "Push" })).toBeInTheDocument();
    expect(band).toHaveTextContent("1 exercise");
  });

  it("places cells by microcycle id rather than by prescription order", () => {
    const reversed: FullWorkout = {
      ...pushWorkout,
      planned_exercises: [
        {
          ...pushWorkout.planned_exercises[0],
          prescriptions: [...pushWorkout.planned_exercises[0].prescriptions].reverse(),
        },
      ],
    };
    renderBand(reversed);

    const row = screen.getByRole("rowheader", { name: /Bench Press/ }).closest("tr");
    const cells = within(row!).getAllByRole("cell");
    expect(cells[0]).toHaveTextContent("3×8–12 RIR2");
    expect(cells[1]).toHaveTextContent("—");
  });

  it("renames a workout", async () => {
    mockApi.renameWorkout.mockResolvedValue({ id: 100, name: "Push A", position: 0 });
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    const field = screen.getByLabelText("Workout name");
    await userEvent.clear(field);
    await userEvent.type(field, "Push A");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(mockApi.renameWorkout).toHaveBeenCalledWith(100, "Push A");
  });

  it("leaves the name alone when the rename is cancelled", async () => {
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    await userEvent.type(screen.getByLabelText("Workout name"), " A");
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(mockApi.renameWorkout).not.toHaveBeenCalled();
    expect(screen.getByRole("heading", { name: "Push" })).toBeInTheDocument();
  });

  it("abandons the rename on Escape", async () => {
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    await userEvent.type(screen.getByLabelText("Workout name"), "{Escape}");

    expect(mockApi.renameWorkout).not.toHaveBeenCalled();
    expect(screen.getByRole("heading", { name: "Push" })).toBeInTheDocument();
  });

  it("skips the request when the name is unchanged", async () => {
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(mockApi.renameWorkout).not.toHaveBeenCalled();
    expect(screen.getByRole("heading", { name: "Push" })).toBeInTheDocument();
  });

  it("cannot save an emptied name", async () => {
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    await userEvent.clear(screen.getByLabelText("Workout name"));

    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(mockApi.renameWorkout).not.toHaveBeenCalled();
  });

  it("deletes a workout only once the destruction is confirmed", async () => {
    mockApi.deleteWorkout.mockResolvedValue(undefined);
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Delete Push" }));
    expect(mockApi.deleteWorkout).not.toHaveBeenCalled();

    confirm.mockReturnValue(true);
    await userEvent.click(screen.getByRole("button", { name: "Delete Push" }));
    expect(mockApi.deleteWorkout).toHaveBeenCalledWith(100);

  });

  it("removes an exercise only once the destruction is confirmed", async () => {
    mockApi.deletePlannedExercise.mockResolvedValue(undefined);
    vi.spyOn(window, "confirm").mockReturnValue(true);
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Remove Bench Press" }));

    expect(mockApi.deletePlannedExercise).toHaveBeenCalledWith(1000);
  });

  it("reports a rejected rename", async () => {
    mockApi.renameWorkout.mockRejectedValue(new ApiError(409, "a workout named Pull already exists"));
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Rename Push" }));
    const field = screen.getByLabelText("Workout name");
    await userEvent.clear(field);
    await userEvent.type(field, "Pull");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "a workout named Pull already exists",
    );
  });

  it("reports a rejected removal", async () => {
    mockApi.deletePlannedExercise.mockRejectedValue(new ApiError(500, "database is locked"));
    vi.spyOn(window, "confirm").mockReturnValue(true);
    renderBand();

    await userEvent.click(screen.getByRole("button", { name: "Remove Bench Press" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("database is locked");
  });

  it("keeps the row as wide as the header when there are no weeks", () => {
    renderWithClient(
      <PlanErrors>
        <GridFrame mesocycleId={MESOCYCLE_ID} microcycles={[]}>
          <WorkoutBand mesocycleId={MESOCYCLE_ID} workout={pushWorkout} microcycles={[]} />
        </GridFrame>
      </PlanErrors>,
    );

    const row = screen.getByRole("rowheader", { name: /Bench Press/ }).closest("tr");
    expect(within(row!).getAllByRole("cell")).toHaveLength(1);
  });

  it("shows a placeholder for a workout with no exercises", () => {
    renderBand({ ...pushWorkout, planned_exercises: [] });

    expect(screen.getByText("No exercises yet.")).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Push/ })).not.toHaveTextContent("exercises");
  });
});

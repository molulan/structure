import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { benchPress, pushWorkout } from "../../test/planFixtures";
import { ApiError } from "../../api/client";
import { AddExercise } from "./AddExercise";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const MESOCYCLE_ID = 7;

function renderAddExercise() {
  return renderWithClient(<AddExercise mesocycleId={MESOCYCLE_ID} workout={pushWorkout} />);
}

async function createSquat() {
  await userEvent.click(await screen.findByRole("button", { name: "New exercise…" }));
  await userEvent.type(screen.getByLabelText("Exercise name"), "Squat");
  await userEvent.selectOptions(screen.getByLabelText("Primary muscle group"), "Quads");
  await userEvent.click(screen.getByRole("button", { name: "Create and add" }));
}

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.listLibraryExercises.mockResolvedValue([benchPress]);
});

describe("AddExercise", () => {
  it("places a library exercise into the workout", async () => {
    mockApi.addPlannedExercise.mockResolvedValue({ id: 1001, exercise: benchPress, position: 1 });
    renderAddExercise();

    // The picker is disabled and optionless until the library query settles.
    await screen.findByRole("option", { name: "Bench Press" });
    const picker = screen.getByRole("combobox", { name: "Exercise to add to Push" });
    await userEvent.selectOptions(picker, "Bench Press");
    await userEvent.click(screen.getByRole("button", { name: "+ Exercise" }));

    expect(mockApi.addPlannedExercise).toHaveBeenCalledWith(100, 1);
    await waitFor(() => expect(picker).toHaveValue(""));
  });

  it("cannot be submitted until an exercise is picked", async () => {
    renderAddExercise();

    expect(await screen.findByRole("button", { name: "+ Exercise" })).toBeDisabled();
  });

  it("creates a library exercise and places it in one step", async () => {
    const squat = { ...benchPress, id: 2, name: "Squat", primary_muscle_group: "Quads" as const };
    mockApi.createLibraryExercise.mockResolvedValue(squat);
    mockApi.addPlannedExercise.mockResolvedValue({ id: 1002, exercise: squat, position: 1 });
    renderAddExercise();

    await createSquat();

    expect(mockApi.createLibraryExercise).toHaveBeenCalledWith({
      name: "Squat",
      exercise_type: "Weighted",
      primary_muscle_group: "Quads",
    });
    await waitFor(() => expect(mockApi.addPlannedExercise).toHaveBeenCalledWith(100, 2));
    // Back to the picker once it lands.
    await screen.findByRole("button", { name: "+ Exercise" });
  });

  // The exercise exists in the library the moment the first request succeeds;
  // retrying the whole form would collide with the name it just took.
  it("leaves a created-but-unplaced exercise ready to retry from the picker", async () => {
    const squat = { ...benchPress, id: 2, name: "Squat", primary_muscle_group: "Quads" as const };
    mockApi.createLibraryExercise.mockResolvedValue(squat);
    mockApi.addPlannedExercise.mockRejectedValue(new ApiError(500, "boom"));
    // The library gains the new exercise once it has been created, which is
    // what lets the picker hold on to it after the placement fails.
    mockApi.listLibraryExercises.mockImplementation(async () =>
      mockApi.createLibraryExercise.mock.calls.length > 0 ? [benchPress, squat] : [benchPress],
    );
    renderAddExercise();

    await createSquat();

    const picker = await screen.findByRole("combobox", { name: "Exercise to add to Push" });
    await waitFor(() => expect(picker).toHaveValue("2"));

    mockApi.addPlannedExercise.mockResolvedValue({ id: 1002, exercise: squat, position: 1 });
    await userEvent.click(screen.getByRole("button", { name: "+ Exercise" }));

    expect(mockApi.createLibraryExercise).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(mockApi.addPlannedExercise).toHaveBeenCalledTimes(2));
  });

  it("keeps the entry on screen and reports why when the library rejects it", async () => {
    mockApi.createLibraryExercise.mockRejectedValue(
      new ApiError(409, "an exercise named 'Bench Press' already exists"),
    );
    renderAddExercise();

    await userEvent.click(await screen.findByRole("button", { name: "New exercise…" }));
    await userEvent.type(screen.getByLabelText("Exercise name"), "Bench Press");
    await userEvent.click(screen.getByRole("button", { name: "Create and add" }));

    expect(
      await screen.findByText("an exercise named 'Bench Press' already exists"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Exercise name")).toHaveValue("Bench Press");
    expect(mockApi.addPlannedExercise).not.toHaveBeenCalled();
  });

  // fetch rejects with a TypeError when it cannot reach the server at all.
  it("reports a transport failure, not just an HTTP one", async () => {
    mockApi.createLibraryExercise.mockRejectedValue(new TypeError("Failed to fetch"));
    renderAddExercise();

    await userEvent.click(await screen.findByRole("button", { name: "New exercise…" }));
    await userEvent.type(screen.getByLabelText("Exercise name"), "Squat");
    await userEvent.click(screen.getByRole("button", { name: "Create and add" }));

    expect(await screen.findByText("Failed to fetch")).toBeInTheDocument();
  });

  it("reports a library that will not load", async () => {
    mockApi.listLibraryExercises.mockRejectedValue(new ApiError(500, "boom"));
    renderAddExercise();

    expect(await screen.findByText("Could not load the exercise library.")).toBeInTheDocument();
  });
});

import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient, testClient } from "../../test/renderWithClient";
import { libraryExercisesKey } from "./usePlanMutations";
import { benchPress, pushWorkout, squat } from "../../fixtures/plan";
import { ApiError } from "../../api/client";
import { PlanErrors } from "./PlanErrors";
import { AddExercise } from "./AddExercise";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const MESOCYCLE_ID = 7;

function renderAddExercise() {
  return renderWithClient(
    <PlanErrors>
      <AddExercise mesocycleId={MESOCYCLE_ID} workout={pushWorkout} />
    </PlanErrors>,
  );
}

async function createDeadlift() {
  await userEvent.click(await screen.findByRole("button", { name: "New exercise…" }));
  await userEvent.type(screen.getByLabelText("Exercise name"), "Deadlift");
  await userEvent.selectOptions(screen.getByLabelText("Primary muscle group"), "Hamstrings");
  await userEvent.click(screen.getByRole("button", { name: "Create and add" }));
}

const deadlift = {
  id: 3,
  name: "Deadlift",
  exercise_type: "Weighted" as const,
  primary_muscle_group: "Hamstrings" as const,
  secondary_muscle_groups: [],
};

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.listLibraryExercises.mockResolvedValue([benchPress, squat]);
});

describe("AddExercise", () => {
  it("places a library exercise into the workout", async () => {
    mockApi.addPlannedExercise.mockResolvedValue({ id: 1001, exercise: squat, position: 1 });
    renderAddExercise();

    // The picker is disabled and optionless until the library query settles.
    await screen.findByRole("option", { name: "Squat" });
    const picker = screen.getByRole("combobox", { name: "Exercise to add to Push" });
    await userEvent.selectOptions(picker, "Squat");
    await userEvent.click(screen.getByRole("button", { name: "+ Exercise" }));

    expect(mockApi.addPlannedExercise).toHaveBeenCalledWith(100, 2);
    await waitFor(() => expect(picker).toHaveValue(""));
  });

  // Two identical rows are indistinguishable once placed — including to the
  // "Remove X" buttons, which would share an accessible name.
  it("does not offer an exercise the workout already has", async () => {
    renderAddExercise();

    await screen.findByRole("option", { name: "Squat" });
    expect(screen.queryByRole("option", { name: "Bench Press" })).not.toBeInTheDocument();
  });

  // A first-run account has nowhere to go but "New exercise…", so the picker
  // must not report the state that means the opposite.
  it("tells an empty library apart from a used-up one", async () => {
    mockApi.listLibraryExercises.mockResolvedValue([]);
    renderAddExercise();

    expect(await screen.findByRole("option", { name: "No exercises yet — create one" })).toBeInTheDocument();
  });

  it("says the library is used up when the workout already holds all of it", async () => {
    mockApi.listLibraryExercises.mockResolvedValue([benchPress]);
    renderAddExercise();

    expect(await screen.findByRole("option", { name: "All exercises added" })).toBeInTheDocument();
  });

  it("cannot be submitted until an exercise is picked", async () => {
    renderAddExercise();

    expect(await screen.findByRole("button", { name: "+ Exercise" })).toBeDisabled();
  });

  it("creates a library exercise and places it in one step", async () => {
    mockApi.createLibraryExercise.mockResolvedValue(deadlift);
    mockApi.addPlannedExercise.mockResolvedValue({ id: 1002, exercise: deadlift, position: 1 });
    renderAddExercise();

    await createDeadlift();

    expect(mockApi.createLibraryExercise).toHaveBeenCalledWith({
      name: "Deadlift",
      exercise_type: "Weighted",
      primary_muscle_group: "Hamstrings",
    });
    await waitFor(() => expect(mockApi.addPlannedExercise).toHaveBeenCalledWith(100, 3));
    // Back to the picker once it lands.
    await screen.findByRole("button", { name: "+ Exercise" });
  });

  // The exercise exists in the library the moment the first request succeeds;
  // retrying the whole form would collide with the name it just took.
  it("leaves a created-but-unplaced exercise ready to retry from the picker", async () => {
    mockApi.createLibraryExercise.mockResolvedValue(deadlift);
    mockApi.addPlannedExercise.mockRejectedValue(new ApiError(500, "boom"));
    // The library gains the new exercise once it has been created, which is
    // what lets the picker hold on to it after the placement fails.
    mockApi.listLibraryExercises.mockImplementation(async () =>
      mockApi.createLibraryExercise.mock.calls.length > 0
        ? [benchPress, squat, deadlift]
        : [benchPress, squat],
    );
    renderAddExercise();

    await createDeadlift();

    const picker = await screen.findByRole("combobox", { name: "Exercise to add to Push" });
    await waitFor(() => expect(picker).toHaveValue("3"));

    mockApi.addPlannedExercise.mockResolvedValue({ id: 1002, exercise: deadlift, position: 1 });
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
    await userEvent.type(screen.getByLabelText("Exercise name"), "Deadlift");
    await userEvent.click(screen.getByRole("button", { name: "Create and add" }));

    expect(await screen.findByText("Failed to fetch")).toBeInTheDocument();
  });

  // Only reachable with data already cached: a refetch that fails keeps the last
  // good data, so `data` is populated while the query sits in error. A fresh
  // client cannot produce that, which is why this one is primed by hand.
  it("does not claim the library is used up when the load failed", async () => {
    const client = testClient();
    client.setQueryData(libraryExercisesKey, [benchPress]);
    mockApi.listLibraryExercises.mockRejectedValue(new ApiError(500, "boom"));

    renderWithClient(
      <PlanErrors>
        <AddExercise mesocycleId={MESOCYCLE_ID} workout={pushWorkout} />
      </PlanErrors>,
      client,
    );

    // The seed is stale, so mounting refetches; awaiting the error line is what
    // proves the failed-refetch state has actually been reached.
    expect(await screen.findByText("Could not load the exercise library.")).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Pick an exercise…" })).toBeInTheDocument();
  });

  it("reports a library that will not load", async () => {
    mockApi.listLibraryExercises.mockRejectedValue(new ApiError(500, "boom"));
    renderAddExercise();

    expect(await screen.findByText("Could not load the exercise library.")).toBeInTheDocument();
  });
});

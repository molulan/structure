import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { ApiError } from "../../api/client";
import type { FullMesocycle } from "../../api/types";
import { MesocycleDetail } from "./MesocycleDetail";

vi.mock("../../lib/apiClient", async () => (await import("../../test/apiMock")).mockApiModule());
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

// The grid's own rendering is covered by MesocycleGrid.component.test.tsx; this
// fixture is only rich enough to prove the detail view hands it the data.
const fullGrid: FullMesocycle = {
  id: 1,
  name: "Hypertrophy Block",
  mode: "Manual",
  microcycles: [{ id: 10, position: 0, phase: "Accumulation" }],
  workouts: [
    {
      id: 100,
      name: "Push",
      position: 0,
      planned_exercises: [
        {
          id: 1000,
          position: 0,
          exercise: {
            id: 1,
            name: "Bench Press",
            exercise_type: "Weighted",
            primary_muscle_group: "Chest",
            secondary_muscle_groups: ["Triceps", "Shoulders"],
          },
          prescriptions: [
            {
              microcycle_id: 10,
              set_groups: [
                {
                  id: 1,
                  position: 0,
                  number_of_sets: 3,
                  set_group_type: {
                    Prescribed: {
                      set_type: "Regular",
                      reps: { Range: { min: 8, max: 12 } },
                      intensity: { Rir: 2 },
                    },
                  },
                },
              ],
            },
          ],
        },
      ],
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.listLibraryExercises.mockResolvedValue([]);
});

describe("MesocycleDetail", () => {
  it("shows a loading state while the query is pending", () => {
    mockApi.getFullMesocycle.mockReturnValue(new Promise(() => {}));
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);
    expect(screen.getByText("Loading…")).toBeInTheDocument();
  });

  it("shows an error state when the query rejects", async () => {
    mockApi.getFullMesocycle.mockRejectedValue(new ApiError(404, "gone"));
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);
    expect(await screen.findByText("Could not load this mesocycle.")).toBeInTheDocument();
  });

  it("shows the empty state for a mesocycle with no weeks", async () => {
    mockApi.getFullMesocycle.mockResolvedValue({ ...fullGrid, microcycles: [], workouts: [] });
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);

    expect(await screen.findByText("Hypertrophy Block")).toBeInTheDocument();
    expect(screen.getByText("No weeks yet.")).toBeInTheDocument();
  });

  it("renders the mesocycle header and its grid", async () => {
    mockApi.getFullMesocycle.mockResolvedValue(fullGrid);
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);

    expect(await screen.findByRole("heading", { name: "Hypertrophy Block", level: 1 })).toBeInTheDocument();
    expect(screen.getByText("Manual")).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Week 1/ })).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Bench Press/ })).toBeInTheDocument();
    expect(screen.getByText("3×8–12 RIR2")).toBeInTheDocument();
  });

  // The mutations invalidate this view's query; this is the one place both
  // halves are mounted together, so it is where the refetch can be observed.
  it("refetches the grid after an edit lands", async () => {
    mockApi.getFullMesocycle.mockResolvedValue(fullGrid);
    mockApi.addMicrocycle.mockResolvedValue({ id: 11, position: 1, phase: null });
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);

    await userEvent.click(await screen.findByRole("button", { name: "+ Week" }));

    await waitFor(() => expect(mockApi.getFullMesocycle).toHaveBeenCalledTimes(2));
  });

  it("calls onBack when the back button is clicked", async () => {
    mockApi.getFullMesocycle.mockResolvedValue(fullGrid);
    const onBack = vi.fn();
    renderWithClient(<MesocycleDetail id={1} onBack={onBack} />);

    await userEvent.click(await screen.findByRole("button", { name: /All mesocycles/ }));
    expect(onBack).toHaveBeenCalledOnce();
  });
});

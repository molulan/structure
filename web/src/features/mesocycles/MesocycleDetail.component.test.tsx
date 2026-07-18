import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { ApiError } from "../../api/client";
import type { FullMesocycle } from "../../api/types";
import { MesocycleDetail } from "./MesocycleDetail";

vi.mock("../../lib/apiClient", () => ({
  api: {
    listMesocycles: vi.fn(),
    createMesocycle: vi.fn(),
    getFullMesocycle: vi.fn(),
  },
}));
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const fullTree: FullMesocycle = {
  id: 1,
  name: "Hypertrophy Block",
  mode: "Manual",
  microcycles: [
    {
      id: 10,
      position: 0,
      phase: "Accumulation",
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
    mockApi.getFullMesocycle.mockResolvedValue({ ...fullTree, microcycles: [] });
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);

    expect(await screen.findByText("Hypertrophy Block")).toBeInTheDocument();
    expect(screen.getByText("No weeks yet.")).toBeInTheDocument();
  });

  it("renders the full nested tree with formatted set groups", async () => {
    mockApi.getFullMesocycle.mockResolvedValue(fullTree);
    renderWithClient(<MesocycleDetail id={1} onBack={() => {}} />);

    expect(await screen.findByRole("heading", { name: "Push" })).toBeInTheDocument();
    expect(screen.getByText("Week 1")).toBeInTheDocument();
    expect(screen.getByText("Accumulation")).toBeInTheDocument();
    expect(screen.getByText("Bench Press")).toBeInTheDocument();
    expect(screen.getByText("Chest")).toBeInTheDocument();
    expect(screen.getByText("3×8–12 RIR2")).toBeInTheDocument();
  });

  it("calls onBack when the back button is clicked", async () => {
    mockApi.getFullMesocycle.mockResolvedValue(fullTree);
    const onBack = vi.fn();
    renderWithClient(<MesocycleDetail id={1} onBack={onBack} />);

    await userEvent.click(await screen.findByRole("button", { name: /All mesocycles/ }));
    expect(onBack).toHaveBeenCalledOnce();
  });
});

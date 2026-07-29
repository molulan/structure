import { describe, expect, it } from "vitest";
import { render, screen, within } from "@testing-library/react";
import type { FullMicrocycle, FullWorkout } from "../../api/types";
import { MesocycleGrid } from "./MesocycleGrid";

const weeks: FullMicrocycle[] = [
  { id: 10, position: 0, phase: "Accumulation" },
  { id: 11, position: 1, phase: null },
];

const workouts: FullWorkout[] = [
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
              { id: 2, position: 1, number_of_sets: 2, set_group_type: "MyorepMatch" },
            ],
          },
          // Week 2 is left unprescribed — the grid still has to place the cell.
          { microcycle_id: 11, set_groups: [] },
        ],
      },
    ],
  },
];

/** The cells of the row whose header names `exercise`, in column order. */
function cellsOf(exercise: string): HTMLElement[] {
  const row = screen.getByRole("rowheader", { name: new RegExp(exercise) }).closest("tr");
  if (!row) throw new Error(`no row for ${exercise}`);
  return within(row).getAllByRole("cell");
}

describe("MesocycleGrid", () => {
  it("shows the empty state when there are no weeks", () => {
    render(<MesocycleGrid microcycles={[]} workouts={workouts} />);

    expect(screen.getByText("No weeks yet.")).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });

  it("renders a column per week, numbered from one, with its phase badge", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    const headers = screen.getAllByRole("columnheader");
    expect(headers[0]).toHaveTextContent("Week 1");
    expect(headers[0]).toHaveTextContent("ACCUM");
    expect(headers[1]).toHaveTextContent("Week 2");
    expect(headers[1]).not.toHaveTextContent("ACCUM");
  });

  it("groups exercise rows under their workout", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    expect(screen.getByRole("columnheader", { name: /Push/ })).toHaveTextContent("1 exercise");
    expect(screen.getByRole("rowheader", { name: /Bench Press/ })).toHaveTextContent("Chest");
  });

  it("puts each week's set groups in that week's cell, and a dash where nothing is prescribed", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    const [week1, week2] = cellsOf("Bench Press");
    expect(week1).toHaveTextContent("3×8–12 RIR2");
    expect(week1).toHaveTextContent("2× match");
    expect(week2).toHaveTextContent("—");
  });

  it("places cells by microcycle id rather than by prescription order", () => {
    const reversed: FullWorkout[] = [
      {
        ...workouts[0],
        planned_exercises: [
          {
            ...workouts[0].planned_exercises[0],
            prescriptions: [...workouts[0].planned_exercises[0].prescriptions].reverse(),
          },
        ],
      },
    ];
    render(<MesocycleGrid microcycles={weeks} workouts={reversed} />);

    const [week1, week2] = cellsOf("Bench Press");
    expect(week1).toHaveTextContent("3×8–12 RIR2");
    expect(week2).toHaveTextContent("—");
  });

  it("shows a placeholder for a mesocycle with weeks but no workouts", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={[]} />);

    expect(screen.getByText("No workouts yet.")).toBeInTheDocument();
    expect(screen.getAllByRole("columnheader")).toHaveLength(2);
  });

  it("shows a placeholder for a workout with no exercises", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={[{ ...workouts[0], planned_exercises: [] }]} />);

    expect(screen.getByText("No exercises yet.")).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Push/ })).not.toHaveTextContent("exercises");
  });
});

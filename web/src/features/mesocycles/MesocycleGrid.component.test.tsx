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
  // Workouts hang off the mesocycle, not the microcycle, so a week-less
  // mesocycle can still hold structure — it must not be rendered as empty.
  it("still renders the workouts when there are no weeks", () => {
    render(<MesocycleGrid microcycles={[]} workouts={workouts} />);

    expect(screen.getByText("No weeks yet.")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Push" })).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Bench Press/ })).toBeInTheDocument();
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

    // The band heads its row group, not the week columns, and stays a heading
    // so the workouts are reachable by heading navigation.
    const band = screen.getByRole("rowheader", { name: /Push/ });
    expect(band).toHaveTextContent("1 exercise");
    expect(within(band).getByRole("heading", { name: "Push" })).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Bench Press/ })).toHaveTextContent("Chest");
  });

  it("puts each week's set groups in that week's cell, and a dash where nothing is prescribed", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    const [week1, week2] = cellsOf("Bench Press");
    expect(week1).toHaveTextContent("3×8–12 RIR2");
    expect(week1).toHaveTextContent("2× match");
    // The dash is decorative; the cell's meaning is spelled out for screen readers.
    expect(week2).toHaveTextContent("—");
    expect(within(week2).getByText("Not prescribed")).toBeInTheDocument();
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

  it("announces the full phase name rather than the abbreviation", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    const header = screen.getAllByRole("columnheader")[0];
    expect(within(header).getByText("Accumulation")).toBeInTheDocument();
  });

  it("spells out the abbreviations of the phases in use, and only those", () => {
    render(
      <MesocycleGrid
        microcycles={[...weeks, { id: 12, position: 2, phase: "Deload" }]}
        workouts={workouts}
      />,
    );

    const legend = screen.getByRole("region", { name: "Phase key" });
    expect(legend).toHaveTextContent("ACCUM Accumulation");
    expect(legend).toHaveTextContent("DELOAD Deload");
    expect(legend).not.toHaveTextContent("Intensification");
  });

  it("omits the legend when no week has a phase", () => {
    render(<MesocycleGrid microcycles={[{ id: 10, position: 0, phase: null }]} workouts={workouts} />);

    expect(screen.queryByRole("region", { name: "Phase key" })).not.toBeInTheDocument();
  });

  it("makes the scrolling grid reachable by keyboard", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={workouts} />);

    expect(screen.getByRole("region", { name: "Plan grid" })).toHaveAttribute("tabindex", "0");
  });

  it("shows a placeholder for a mesocycle with weeks but no workouts", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={[]} />);

    expect(screen.getByText("No workouts yet.")).toBeInTheDocument();
    expect(screen.getAllByRole("columnheader")).toHaveLength(2);
  });

  it("shows a placeholder for a workout with no exercises", () => {
    render(<MesocycleGrid microcycles={weeks} workouts={[{ ...workouts[0], planned_exercises: [] }]} />);

    expect(screen.getByText("No exercises yet.")).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Push/ })).not.toHaveTextContent("exercises");
  });
});

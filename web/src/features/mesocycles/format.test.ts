import { describe, expect, it } from "vitest";
import type { Intensity, RepTarget, SetGroup } from "../../api/types";
import { describeSetGroup, formatIntensity, formatReps } from "./format";

describe("formatReps", () => {
  it("renders an exact count as the bare number", () => {
    expect(formatReps({ Exact: 5 })).toBe("5");
  });
  it("renders a range with an en dash", () => {
    expect(formatReps({ Range: { min: 8, max: 12 } })).toBe("8–12");
  });
  it("renders an AMRAP target with a trailing plus", () => {
    expect(formatReps({ AtLeast: 12 })).toBe("12+");
  });
  it("degrades an unrecognized variant to a placeholder instead of throwing", () => {
    expect(formatReps({ Unknown: 1 } as unknown as RepTarget)).toBe("?");
  });
});

describe("formatIntensity", () => {
  it("renders RIR label-attached", () => {
    expect(formatIntensity({ Rir: 2 })).toBe("RIR2");
  });
  it("renders RPE label-attached", () => {
    expect(formatIntensity({ Rpe: 8 })).toBe("RPE8");
  });
  it("renders a percent of 1RM", () => {
    expect(formatIntensity({ PercentOneRepMax: 80 })).toBe("80%");
  });
  it("renders a target weight with its unit and no space", () => {
    expect(formatIntensity({ TargetWeight: { value: 60, unit: "Kg" } })).toBe("60kg");
  });
  it("renders a weight increment with a leading plus", () => {
    expect(formatIntensity({ WeightIncrement: { value: 2.5, unit: "Lbs" } })).toBe("+2.5lb");
  });
  it("degrades an unrecognized variant to a placeholder instead of throwing", () => {
    expect(formatIntensity({ Unknown: 1 } as unknown as Intensity)).toBe("?");
  });
});

describe("describeSetGroup", () => {
  const group = (id: number, set_group_type: SetGroup["set_group_type"], number_of_sets = 3): SetGroup => ({
    id,
    position: 0,
    number_of_sets,
    set_group_type,
  });

  it("joins reps and RIR with a space", () => {
    expect(
      describeSetGroup(
        group(1, { Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } } }),
      ),
    ).toBe("3×8–12 RIR2");
  });

  it("joins reps and an absolute target weight with ×", () => {
    expect(
      describeSetGroup(
        group(2, { Prescribed: { set_type: "Regular", reps: { Exact: 8 }, intensity: { TargetWeight: { value: 60, unit: "Kg" } } } }),
      ),
    ).toBe("3×8×60kg");
  });

  it("renders an AMRAP target", () => {
    expect(
      describeSetGroup(
        group(3, { Prescribed: { set_type: "Regular", reps: { AtLeast: 12 }, intensity: { Rir: 0 } } }, 1),
      ),
    ).toBe("1×12+ RIR0");
  });

  it("appends the set-type name and the load for a myorep group", () => {
    expect(
      describeSetGroup(
        group(4, { Prescribed: { set_type: "Myorep", reps: { Exact: 10 }, intensity: { PercentOneRepMax: 80 } } }, 1),
      ),
    ).toBe("1×10×80% myorep");
  });

  it("appends the set-type name and the load for a dropset", () => {
    expect(
      describeSetGroup(
        group(5, { Prescribed: { set_type: "Drop", reps: { Exact: 8 }, intensity: { TargetWeight: { value: 40, unit: "Kg" } } } }, 2),
      ),
    ).toBe("2×8×40kg dropset");
  });

  it("summarizes a myorep-match group", () => {
    expect(describeSetGroup(group(6, "MyorepMatch", 2))).toBe("2× match");
  });

  it("degrades an unrecognized set-group type to a placeholder instead of throwing", () => {
    expect(
      describeSetGroup(group(7, "SomethingNew" as unknown as SetGroup["set_group_type"], 3)),
    ).toBe("3× ?");
  });
});

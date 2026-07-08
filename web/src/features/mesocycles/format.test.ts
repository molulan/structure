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
  it("renders RIR", () => {
    expect(formatIntensity({ Rir: 2 })).toBe("2 RIR");
  });
  it("renders RPE", () => {
    expect(formatIntensity({ Rpe: 8 })).toBe("RPE 8");
  });
  it("renders a percent of 1RM", () => {
    expect(formatIntensity({ PercentOneRepMax: 75 })).toBe("75% 1RM");
  });
  it("renders a target weight with its unit", () => {
    expect(formatIntensity({ TargetWeight: { value: 60, unit: "Kg" } })).toBe("60 kg");
  });
  it("renders a weight increment with a leading plus", () => {
    expect(formatIntensity({ WeightIncrement: { value: 2.5, unit: "Lbs" } })).toBe("+2.5 lb");
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

  it("summarizes a regular prescribed group without naming the type", () => {
    expect(
      describeSetGroup(
        group(1, { Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } } }),
      ),
    ).toBe("3 × 8–12 @ 2 RIR");
  });

  it("names a non-regular set type", () => {
    expect(
      describeSetGroup(
        group(2, { Prescribed: { set_type: "Myorep", reps: { Exact: 10 }, intensity: { Rpe: 9 } } }, 1),
      ),
    ).toBe("1 × myorep 10 @ RPE 9");
  });

  it("summarizes a myorep-match group", () => {
    expect(describeSetGroup(group(3, "MyorepMatch", 2))).toBe("2 × myorep match");
  });

  it("degrades an unrecognized set-group type to a placeholder instead of throwing", () => {
    expect(
      describeSetGroup(group(4, "SomethingNew" as unknown as SetGroup["set_group_type"], 3)),
    ).toBe("3 × ?");
  });
});

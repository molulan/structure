import type { Intensity, RepTarget, SetGroup, Weight } from "../../api/types";

// Shown in place of a value whose variant isn't one we recognize (e.g. the
// backend grew a new enum variant the client doesn't know yet). Degrading to
// this beats throwing, which would blank the whole detail render.
const UNKNOWN = "?";

export function formatReps(reps: RepTarget): string {
  if ("Exact" in reps) return String(reps.Exact);
  if ("AtLeast" in reps) return `${reps.AtLeast}+`;
  if ("Range" in reps) return `${reps.Range.min}–${reps.Range.max}`;
  return UNKNOWN;
}

function formatWeight(weight: Weight): string {
  return `${weight.value} ${weight.unit === "Kg" ? "kg" : "lb"}`;
}

export function formatIntensity(intensity: Intensity): string {
  if ("Rir" in intensity) return `${intensity.Rir} RIR`;
  if ("Rpe" in intensity) return `RPE ${intensity.Rpe}`;
  if ("PercentOneRepMax" in intensity) return `${intensity.PercentOneRepMax}% 1RM`;
  if ("TargetWeight" in intensity) return formatWeight(intensity.TargetWeight);
  if ("WeightIncrement" in intensity) return `+${formatWeight(intensity.WeightIncrement)}`;
  return UNKNOWN;
}

/** A one-line human summary of a set group, e.g. `3 × 8–12 @ 2 RIR`. */
export function describeSetGroup(group: SetGroup): string {
  const sets = group.number_of_sets;
  const type = group.set_group_type;
  if (type === "MyorepMatch") {
    return `${sets} × myorep match`;
  }
  if (typeof type === "object" && "Prescribed" in type) {
    const { set_type, reps, intensity } = type.Prescribed;
    const kind = set_type === "Regular" ? "" : ` ${set_type.toLowerCase()}`;
    return `${sets} ×${kind} ${formatReps(reps)} @ ${formatIntensity(intensity)}`;
  }
  return `${sets} × ${UNKNOWN}`;
}

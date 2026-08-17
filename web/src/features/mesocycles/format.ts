import type { Intensity, RepTarget, SetGroup, Weight } from "../../api/wire";

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
  return `${weight.value}${weight.unit === "Kg" ? "kg" : "lb"}`;
}

/** The bare intensity token, e.g. `RIR2`, `RPE8`, `80%`, `60kg`, `+2.5kg`. */
export function formatIntensity(intensity: Intensity): string {
  if ("Rir" in intensity) return `RIR${intensity.Rir}`;
  if ("Rpe" in intensity) return `RPE${intensity.Rpe}`;
  if ("PercentOneRepMax" in intensity) return `${intensity.PercentOneRepMax}%`;
  if ("TargetWeight" in intensity) return formatWeight(intensity.TargetWeight);
  if ("WeightIncrement" in intensity) {
    const w = intensity.WeightIncrement;
    return `${w.value >= 0 ? "+" : ""}${formatWeight(w)}`;
  }
  return UNKNOWN;
}

// A failure-based set group (myorep/drop) can't carry RIR/RPE, so it shows its
// weight-resolving intensity joined with ×, mirroring the prototype's specOf.
function loadSuffix(intensity: Intensity): string {
  const isWeight =
    "PercentOneRepMax" in intensity || "TargetWeight" in intensity || "WeightIncrement" in intensity;
  return isWeight ? `×${formatIntensity(intensity)}` : "";
}

/**
 * A one-line spec for a set group, in the prototype's `specOf` format:
 * `3×8–12 RIR2`, `3×8 80%`, `3×8×60kg`, `3×12+ RIR0`, `2× match`,
 * `3×8×60kg myorep`, `3×8×60kg dropset`.
 */
export function describeSetGroup(group: SetGroup): string {
  const count = group.number_of_sets;
  const type = group.set_group_type;
  if (type === "MyorepMatch") {
    return `${count}× match`;
  }
  if (typeof type === "object" && "Prescribed" in type) {
    const { set_type, reps, intensity } = type.Prescribed;
    const base = `${count}×${formatReps(reps)}`;
    if (set_type === "Myorep") return `${base}${loadSuffix(intensity)} myorep`;
    if (set_type === "Drop") return `${base}${loadSuffix(intensity)} dropset`;
    // Regular: an absolute target weight joins with ×, every other intensity with a space.
    const join = "TargetWeight" in intensity ? "×" : " ";
    return `${base}${join}${formatIntensity(intensity)}`;
  }
  return `${count}× ${UNKNOWN}`;
}

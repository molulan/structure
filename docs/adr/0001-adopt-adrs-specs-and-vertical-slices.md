# 1. Adopt ADRs, spec-driven development and vertical slices

Date: 2026-11-15

## Status

Accepted

## Context

AI lets me generate code very fast, but my review process has stayed the
same, and it has become the bottleneck. Reviewing AI-generated code
line by line feels like proof-reading, which I don't find interesting.
I want a review process that fits AI-native software engineering.

Spec-driven development looks like a way there. I have always liked
the idea of test-driven development, and this feels like its AI-native
version: a specification written in plain language becomes the source
of truth.

Vertical slice architecture suits spec-driven development: the spec
defining a feature can live right next to the feature. It should also
help review by narrowing the code a change touches. Each feature can
be changed and reviewed on its own, without holding the complexity of
the whole codebase in my head. When shared code is touched, I know to
review extra carefully, because that change can break other features.

Keeping ADRs makes me reflect on the tools and practices I use and want
to use. The reflection itself helps me learn, and documented decisions,
good and bad, are easier to review and remember later.

## Decision

1. **Keep Architecture Decision Records.** Significant decisions are
   recorded in `docs/adr/` as numbered records stating context,
   decision and consequences. An accepted ADR is not edited; a later
   ADR supersedes it.
2. **Build and adopt a spec-driven development approach.** A feature is
   specified before it is built, and the specification is what the
   implementation is checked against.
3. **Create and adopt a vertical slice architecture.** Code is organised
   by feature rather than by layer, so that each feature lives in one
   place.

## Consequences

- The specific review process is still to be figured out, but the hope
  is that it will be faster while still giving confidence in the code.
- A change to one feature can be reviewed on its own. A change to
  shared code is the signal for a careful review.
- Decisions and the reasoning behind them are written down, so they can
  be revisited and learned from.
- Knowledge gets duplicated, in code and in documentation, and
  duplicates can drift. If each feature is independent, this may not
  be a problem: drift only matters when duplicated things are supposed
  to stay the same.

The details are left to later ADRs: the spec format and workflow, the
slice layout and what stays shared between slices, and how existing
code moves over.

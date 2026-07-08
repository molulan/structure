# mesocycle_builder — design reference (not shipped)

This directory holds the original **Claude-Design prototype** of the mesocycle
builder, in Claude's design-container format (`mesocycle_builder.dc.html` +
`support.js` runtime, plus a `.thumbnail`).

It is **reference material, not application code**:

- Nothing under `src/` imports it; it is not part of the build or the bundle.
- It is excluded from linting and typechecking.
- It runs standalone with its own in-memory state and does **not** talk to the
  backend.

It is kept as the **visual and interaction spec** for the builder UI that the
real SPA (in `web/src/`) is being rebuilt to match, PR by PR. Treat it as a
picture of the intended behaviour, not as a source of truth for code.

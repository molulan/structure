# structure-web

The web frontend for Structure — a Vite + React + TypeScript SPA that talks to
`structure-server` over HTTP. Currently a scaffold: a mesocycle list/create
screen proving the end-to-end stack. The full mesocycle builder is rebuilt from
`mesocycle_builder/` (the Claude-Design prototype, kept as the interaction spec)
in later PRs.

## Stack

- **Vite + React + TypeScript**
- **TanStack Query** — server data, caching, and mutations
- **CSS Modules + design tokens** (`src/styles/tokens.css`) — the palette is
  provisional (inherited from the prototype); a deliberate design pass comes later

## Run it

```bash
npm install
npm run app        # → http://127.0.0.1:5173
```

That is the whole thing: the Axum backend on `:3000` against a persistent
`dev.db` at the repo root, that database seeded with a four-week block if it is
empty, and Vite in front of it. It prints the app URL and a link straight to the
seeded plan, and Ctrl-C stops both halves.

`dev.db` is gitignored and yours to mess up — whatever you build by hand is still
there next time. `npm run app:reset` deletes it and reseeds. The seed itself is
`scripts/seedDev.ts`; extend it when a screen needs data the block doesn't have.

Two behaviours worth knowing:

- If something is **already serving `:3000`**, that backend is reused and *never
  seeded* — it may well be the real `structure.db`, and writing a demo block into
  it isn't the dev stack's business. The banner says so when this happens.
- Ports are overridable (`STRUCTURE_DEV_API_PORT`, `STRUCTURE_DEV_WEB_PORT`) for
  running a second stack alongside the first; the Vite proxy follows the backend.

The lower-level pieces are still there if you want them: `npm run dev` runs Vite
alone against a backend you started yourself (`cargo run -p structure-server`),
with `VITE_PROXY_TARGET` to point it at a non-default port.

## Look at it

```bash
npm run shot -- /mesocycles/1          # → .shots/mesocycles-1.png
npm run shot -- / /mesocycles/1        # several routes at once
npm run shot -- /mesocycles/1 --viewport=430x900 --viewport-only
```

Screenshots the stack that's *already running*, so an edit is on film as fast as
Vite can hot-reload it — where `test:e2e`, the only other thing that photographs
the app, pays a fresh cargo build and browser launch every run. It exits non-zero
on console errors, uncaught exceptions and 4xx/5xx responses, so a screenshot
that looks right but logged a warning still reports a problem. Output lands in
`.shots/` (gitignored).

## Verify

One gate runs the whole verification stack:

```bash
npm run verify     # typecheck + lint + unit + contract + e2e
```

- **typecheck / lint** — `tsc --noEmit`, ESLint.
- **unit** (`test:unit`) — pure logic, no backend (e.g. the API client).
- **contract** (`test:contract`) — boots a real `structure-server`
  (`STRUCTURE_DB=:memory:` on a free port) and drives the typed client against
  it, verifying our TS types match the server's wire format. Requires `cargo`.
- **e2e** (`test:e2e`) — Playwright boots the backend + Vite dev server and
  drives a real browser, writing a screenshot to `e2e/screenshots/`. First run
  needs the browser: `npx playwright install chromium`.

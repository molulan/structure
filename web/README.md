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

It builds four plans. One is a realistic four-week block — the reference, and the
one the banner links to. The other three are plans caught mid-build, named for
what they are because they are fixtures:

| Plan | Shape |
|---|---|
| `Upper/Lower Hypertrophy` | four weeks, four workouts, every set-group encoding |
| `Empty — no weeks or workouts` | nothing yet: both empty notices at once |
| `Draft — exercises first` | two workouts with exercises, no weeks |
| `Draft — weeks first` | three weeks, one workout still empty |

Degenerate states belong here rather than on the workbench whenever a database
can hold them, because seeded they stay **clickable** — you can press "+ Week" on
an empty plan and watch the first column appear. `devSeed.contract.test.ts` pins
their shape, since each is defined by what it lacks and the seed states that
nowhere: it simply never makes the call.

The reference block is only ever built into an empty database, but the three
fixture plans are topped up on every start — they belong to the seed rather than
to you, so a `dev.db` from before they existed gains them without your hand-built
data being reset away. Delete one and it comes back next boot; that is the trade.

Three behaviours worth knowing:

- If something is **already serving `:3000`**, that backend is reused and *never
  seeded* — it may well be the real `structure.db`, and writing a demo block into
  it isn't the dev stack's business. The banner says so when this happens. A
  backend the stack starts itself announces its own bind before anything is
  seeded, so a stranger that took the port while cargo compiled can't be mistaken
  for ours.
- Something **already serving `:5173`** is an error rather than a reuse. It
  proxies `/api` to a backend this stack knows nothing about, so carrying on
  would advertise a URL whose edits land in someone else's database.
- Ports are overridable (`STRUCTURE_DEV_API_PORT`, `STRUCTURE_DEV_WEB_PORT`) for
  running a second stack alongside the first; the Vite proxy follows the backend,
  and the database is named after the backend port (`dev-3001.db`) so the two
  stacks never share one SQLite file.

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

### Driving it interactively

`.mcp.json` at the repo root configures a headless [Playwright MCP][mcp] server,
which gives an agent a real browser to drive against the running stack — click,
hover, fill a form, read the console and network log — rather than one still
frame at a time. Claude Code picks it up automatically and asks to approve the
project-scoped server the first time; the version is pinned so a session doesn't
change behaviour under you.

`npm run shot` is still the right first reach for "does this look right". The MCP
server earns its keep on interaction: what a dropdown does when open, what a cell
looks like mid-edit, which request a click actually fired.

[mcp]: https://github.com/microsoft/playwright-mcp

### The workbench

```bash
npm run shot -- /dev/workbench
```

A development-only route holding the plan states a **database has no way to
hold**. Three kinds qualify, and nothing else belongs there:

1. **States no data can produce** — a rejected edit, a `fetch` that never got a
   status. No row expresses "the request came back 409".
2. **States that contradict the seed's other content** — an empty library can't
   coexist with a plan, since planned exercises reference library rows.
3. **Adversarial input that isn't a state** — a name at length is a stress test,
   and seeding one would degrade every other screenshot.

Anything else that a plan can actually be in goes in `scripts/seedDev.ts`.

Two rules keep the page honest. It **mounts the app's own wrappers** — the real
`GridFrame`, not a copy — so a layout judged there is the layout that ships.
And it **cannot reach the network**: each stage substitutes an API client whose
every method rejects, via `ApiProvider`, so no control can edit real data
whatever id it carries. That isn't a nicety — `npm run shot` fails on any 4xx,
so a stage that fetched would break the screenshot loop for whoever was using it
to look at something else. A component test presses a delete and creates a
library exercise, then asserts the app's real client was never touched.

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

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

## Develop

```bash
npm install
npm run dev        # Vite on :5173, proxying /api → http://127.0.0.1:3000
```

Run the backend separately (from the repo root):

```bash
cargo run -p structure-server   # listens on :3000
```

If you run the backend on a non-default port (`PORT=…`), point the dev proxy at
it too, e.g. `VITE_PROXY_TARGET=http://127.0.0.1:4000 npm run dev` — otherwise
the proxy still targets `:3000` and every `/api` request fails.

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

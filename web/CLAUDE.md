# CLAUDE.md — web frontend

Paths below are relative to `web/`, the npm project these commands run from.

## The API types are generated

`src/api/generated/` is written from the Rust types the server serializes, by the
tests ts-rs generates for every type marked `#[ts(export)]` — so for every type in
it, the frontend's idea of the contract cannot disagree with the server's. It is
gitignored, and `typecheck` rewrites it before it runs — `build` goes through
`typecheck` — so it cannot be stale. `lint` and the unit and component suites don't need it: the app imports
these types with `import type`, which never reaches the filesystem.

Adding an endpoint means marking its type `#[ts(export)]`. Nothing forces that: a
request type nothing imports yet can be left out, and the omission surfaces only
when the frontend first reaches for it.

A type is generated exactly as Rust serializes it: `snake_case` keys, externally
tagged enums (`{ "Rir": 2 }`), bare strings for unit variants, and newtypes as
their inner value. The names are the server's too — `LibraryExerciseRequest`,
not a friendlier local alias.

## Verifying frontend changes

Every change under `web/` ends with a look at the running app, not just green tests. Leave `npm run app` up — it serves a seeded four-week plan, so no screen is empty — and take a `npm run shot -- <route>` after each edit: Vite hot-reloads, so the screenshot is current within a second or two, and the script fails on console errors and 4xx/5xx responses — and on a capture or comparison it could not complete — which a screenshot alone would hide. It keeps each route's last frame in `.shots/prev/` and reports how much of the page moved — `changed 240 pixels (<0.1%) vs a frame 40s old`, with the changed regions in `.shots/diff/`. Read the count before the image: an edit that never reached the screen renders exactly like one that worked, and `identical to the previous frame` is what tells them apart. An edit that changes the page's height reports the size alongside the count for the area both frames share. Any framing but the default — `--viewport` or `--viewport-only` — writes under its own name, so a mobile check doesn't become the baseline the next desktop capture is judged against. Iterate against that rather than against the suites: the pre-commit hook runs `npm run verify:local` — everything `npm run verify` runs bar the production build and the e2e suite, which CI runs — so the gate meets the change at the commit, not between edits. The dev data lives in `scripts/seedDev.ts` — extend it when a screen needs something the block doesn't cover yet, and keep it distinct from `tests/support/seed.ts`, which is shaped for assertions rather than for looking at.

For the times a still frame isn't enough — clicking into a cell, hovering, filling a form, reading the console or network log — the repo-root `.mcp.json` configures a headless Playwright MCP server against the same running stack. Reach for `npm run shot` first: it answers "does this look right" in one command, where the MCP server costs tool-schema overhead in every session. On `browser_take_screenshot`, omit `filename` so output lands in the gitignored `.shots/mcp/`; a relative filename resolves against the repo root instead and leaves junk in the working tree.

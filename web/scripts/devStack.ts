import { spawn, type ChildProcess } from "node:child_process";
import { rmSync } from "node:fs";
import { resolve } from "node:path";
import { seedDevPlan, SEEDED_EXERCISE_COUNT, SEEDED_WORKOUT_COUNT } from "./seedDev";

// One command that leaves the whole app running and worth looking at: the Axum
// backend against a persistent dev database, that database populated if it is
// empty, and Vite in front of it. The point is that opening the app never lands
// on an empty screen — the plan grid only says anything with a plan in it.
//
// Default ports are fixed, unlike the e2e harness's 3001/5174: a stable URL is
// the whole value for a human, and a collision there means the stack is already
// up, which is a reason to reuse rather than fail. The env overrides exist for
// running a second stack alongside the first.

const BACKEND_PORT = Number(process.env.STRUCTURE_DEV_API_PORT ?? 3000);
const WEB_PORT = Number(process.env.STRUCTURE_DEV_WEB_PORT ?? 5173);
const BACKEND_URL = `http://127.0.0.1:${BACKEND_PORT}`;
const WEB_URL = `http://127.0.0.1:${WEB_PORT}`;
const API_URL = `${BACKEND_URL}/api`;

// Relative to web/, where the npm script runs. Absolute because the backend
// resolves STRUCTURE_DB against its own working directory.
const DB_PATH = resolve("..", "dev.db");

const spawned: ChildProcess[] = [];

async function run(): Promise<void> {
  const reset = process.argv.includes("--reset");
  const backendWasUp = await isUp(`${BACKEND_URL}/health`);

  if (reset) {
    if (backendWasUp) {
      throw new Error(
        `a backend is already running on :${BACKEND_PORT} and holds the dev database — ` +
          "stop it first, then re-run with --reset",
      );
    }
    // -wal and -shm may or may not exist depending on how the last run ended.
    for (const suffix of ["", "-wal", "-shm"]) {
      rmSync(`${DB_PATH}${suffix}`, { force: true });
    }
    console.log("reset: deleted dev.db");
  }

  if (backendWasUp) {
    console.log(`backend: reusing the one already on :${BACKEND_PORT}`);
  } else {
    console.log(`backend: starting on :${BACKEND_PORT} (a cold cargo build takes a while)`);
    const backend = start("cargo", ["run", "--quiet", "--manifest-path", "../Cargo.toml", "-p", "structure-server"], {
      PORT: String(BACKEND_PORT),
      STRUCTURE_DB: DB_PATH,
    });
    await waitFor(`${BACKEND_URL}/health`, backend, "structure-server");
  }

  // Only ever seed a database we opened ourselves. A backend that was already up
  // is serving some database we know nothing about — quite possibly the real one
  // — and writing a demo block into that is not ours to do.
  const planId = backendWasUp ? await firstPlanId() : await ensureSeeded();

  if (await isUp(WEB_URL)) {
    console.log(`web: reusing the dev server already on :${WEB_PORT}`);
  } else {
    console.log(`web: starting Vite on :${WEB_PORT}`);
    const web = start("npm", ["run", "dev", "--", "--port", String(WEB_PORT), "--strictPort"], {
      VITE_PROXY_TARGET: BACKEND_URL,
    });
    await waitFor(WEB_URL, web, "vite");
  }

  banner(planId, backendWasUp);
}

async function isUp(url: string): Promise<boolean> {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1_000) })).ok;
  } catch {
    return false;
  }
}

function start(command: string, args: string[], env: Record<string, string>): ChildProcess {
  const child = spawn(command, args, { env: { ...process.env, ...env }, stdio: "inherit" });
  spawned.push(child);
  child.on("exit", (code) => {
    // A child dying is fatal for the stack — half a stack is worse than none,
    // because the surviving half looks like it works.
    if (!shuttingDown) {
      console.error(`\n${command} exited (code ${code}) — shutting the stack down`);
      shutdown(1);
    }
  });
  return child;
}

async function waitFor(url: string, child: ChildProcess, name: string): Promise<void> {
  const deadline = Date.now() + 300_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`${name} exited before it came up`);
    if (await isUp(url)) return;
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`${name} did not come up at ${url} within the timeout`);
}

/**
 * Seeds the dev block when the database holds no mesocycles, and returns the id
 * of the plan the banner should link to. Existing data is left alone — the point
 * of a persistent dev database is that what you built by hand yesterday is still
 * there today; `--reset` is how you ask for a clean one.
 */
async function ensureSeeded(): Promise<number | null> {
  const existing = await plans();
  if (existing.length > 0) {
    console.log(`data: ${existing.length} mesocycle(s) already in dev.db — left as they are`);
    return existing[0].id;
  }
  console.log("data: empty, seeding a four-week block");
  const id = await seedDevPlan(API_URL);
  console.log(
    `data: seeded ${SEEDED_WORKOUT_COUNT} workouts / ${SEEDED_EXERCISE_COUNT} exercises over 4 weeks`,
  );
  return id;
}

async function firstPlanId(): Promise<number | null> {
  const existing = await plans();
  console.log(`data: whatever that backend is serving — ${existing.length} mesocycle(s), not seeded`);
  return existing[0]?.id ?? null;
}

async function plans(): Promise<{ id: number }[]> {
  return (await (await fetch(`${API_URL}/mesocycles`)).json()) as { id: number }[];
}

function banner(planId: number | null, foreignBackend: boolean): void {
  const rule = "─".repeat(52);
  const lines = [
    "",
    rule,
    "  Structure is running",
    "",
    `    app     ${WEB_URL}`,
    ...(planId === null ? [] : [`    plan    ${WEB_URL}/mesocycles/${planId}`]),
    `    api     ${API_URL}`,
    foreignBackend
      ? `    data    the backend already on :${BACKEND_PORT} — not dev.db`
      : "    data    dev.db  (npm run app:reset for a fresh one)",
    "",
    "  Ctrl-C to stop",
    rule,
    "",
  ];
  console.log(lines.join("\n"));
}

let shuttingDown = false;

function shutdown(code: number): void {
  shuttingDown = true;
  for (const child of spawned) child.kill("SIGTERM");
  process.exit(code);
}

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => shutdown(0));
}

try {
  await run();
} catch (error) {
  console.error(`\n${error instanceof Error ? error.message : String(error)}`);
  shutdown(1);
}

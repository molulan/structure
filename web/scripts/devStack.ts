import { spawn, type ChildProcess } from "node:child_process";
import { rmSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { seedDevPlan, SEEDED_EXERCISE_COUNT, SEEDED_WORKOUT_COUNT } from "./seedDev";

// One command that leaves the whole app running and worth looking at: the Axum
// backend against a persistent dev database, that database populated if it is
// empty, and Vite in front of it. The point is that opening the app never lands
// on an empty screen — the plan grid only says anything with a plan in it.
//
// Default ports are fixed, unlike the e2e harness's 3001/5174: a stable URL is
// the whole value for a human. The env overrides exist for running a second
// stack alongside the first, which is why the database is named after the
// backend port — two stacks must not share one SQLite file.

const DEFAULT_API_PORT = 3000;
const BACKEND_PORT = Number(process.env.STRUCTURE_DEV_API_PORT ?? DEFAULT_API_PORT);
const WEB_PORT = Number(process.env.STRUCTURE_DEV_WEB_PORT ?? 5173);
const BACKEND_URL = `http://127.0.0.1:${BACKEND_PORT}`;
const WEB_URL = `http://127.0.0.1:${WEB_PORT}`;
const API_URL = `${BACKEND_URL}/api`;

// Resolved from this file, not the working directory: invoked as anything but
// `npm run app` from web/ — an IDE run configuration, `tsx web/scripts/…` from
// the repo root — a cwd-relative path would put the database outside the
// repository, and `--reset` would delete files there.
const WEB_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const REPO_ROOT = resolve(WEB_DIR, "..");
const DB_PATH = join(
  REPO_ROOT,
  BACKEND_PORT === DEFAULT_API_PORT ? "dev.db" : `dev-${BACKEND_PORT}.db`,
);
const DB_NAME = basename(DB_PATH);

const spawned: ChildProcess[] = [];

async function run(): Promise<void> {
  const reset = process.argv.includes("--reset");
  const backendWasUp = await isUp(`${BACKEND_URL}/health`);

  if (reset) {
    // The database is named after the backend port, so a backend answering that
    // port is the only thing that can hold this file open.
    if (backendWasUp) {
      throw new Error(
        `a backend is already running on :${BACKEND_PORT} and may hold ${DB_NAME} open — ` +
          "stop it first, then re-run with --reset",
      );
    }
    for (const suffix of ["", "-wal", "-shm"]) {
      rmSync(`${DB_PATH}${suffix}`, { force: true });
    }
    console.log(`reset: deleted ${DB_NAME}`);
  }

  // Vite is never reused. Something already on the web port proxies /api to a
  // backend we know nothing about, so the banner would advertise a URL whose
  // edits land in someone else's database.
  if (await isUp(WEB_URL)) {
    throw new Error(
      `something is already serving :${WEB_PORT} — stop it, or pick another port with ` +
        "STRUCTURE_DEV_WEB_PORT",
    );
  }

  if (backendWasUp) {
    console.log(`backend: reusing the one already on :${BACKEND_PORT}`);
  } else {
    console.log(`backend: starting on :${BACKEND_PORT} (a cold cargo build takes a while)`);
    const backend = start(
      "structure-server",
      "cargo",
      ["run", "--quiet", "--manifest-path", join(REPO_ROOT, "Cargo.toml"), "-p", "structure-server"],
      { PORT: String(BACKEND_PORT), STRUCTURE_DB: DB_PATH },
      "pipe",
    );
    // Readiness comes from our own child announcing its bind, not from probing
    // the port: a probe cannot tell our server from one that grabbed the port
    // while cargo was still compiling, and mistaking those seeds a database we
    // do not own.
    await waitForLine(backend, `listening on ${BACKEND_URL}`, "structure-server", 300_000);
  }

  // Only ever seed a database we opened ourselves. A backend that was already up
  // is serving some database we know nothing about — quite possibly the real one
  // — and writing a demo block into that is not ours to do.
  const planId = backendWasUp ? await firstPlanId() : await ensureSeeded();

  console.log(`web: starting Vite on :${WEB_PORT}`);
  const web = start(
    "vite",
    "npm",
    ["run", "dev", "--", "--port", String(WEB_PORT), "--strictPort"],
    { VITE_PROXY_TARGET: BACKEND_URL },
    "inherit",
  );
  await waitForPort(WEB_URL, web, "vite");

  banner(planId, backendWasUp);
}

async function isUp(url: string): Promise<boolean> {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1_000) })).ok;
  } catch {
    return false;
  }
}

/**
 * Spawns a child in its own process group, so shutdown can signal the whole
 * group: `cargo run` forwards nothing to the `structure-server` it spawns, and
 * signalling only the wrapper leaves the server holding the port and the
 * database. `stdin` is ignored rather than inherited — a detached child reading
 * the terminal would take SIGTTIN and stop.
 */
function start(
  name: string,
  command: string,
  args: string[],
  env: Record<string, string>,
  stdout: "pipe" | "inherit",
): ChildProcess {
  const child = spawn(command, args, {
    cwd: WEB_DIR,
    env: { ...process.env, ...env },
    stdio: ["ignore", stdout, "inherit"],
    detached: true,
  });
  spawned.push(child);

  // 'error' fires instead of 'exit' when the command cannot be executed at all,
  // and an unhandled one would throw past the top-level catch.
  child.on("error", (error) => {
    if (shuttingDown) return;
    console.error(`\ncould not start ${name} (${command}): ${error.message}`);
    void shutdown(1);
  });

  child.on("exit", (code) => {
    // A child dying is fatal for the stack — half a stack is worse than none,
    // because the surviving half looks like it works.
    if (shuttingDown) return;
    console.error(`\n${name} exited (code ${code}) — shutting the stack down`);
    void shutdown(1);
  });

  return child;
}

/** Resolves when the child prints `marker`, echoing its output through meanwhile. */
function waitForLine(
  child: ChildProcess,
  marker: string,
  name: string,
  timeoutMs: number,
): Promise<void> {
  if (!child.stdout) throw new Error(`${name} was not started with a readable stdout`);
  const lines = createInterface({ input: child.stdout });

  return new Promise((resolvePromise, reject) => {
    const timer = setTimeout(() => {
      finish(() => reject(new Error(`${name} did not report itself listening within the timeout`)));
    }, timeoutMs);

    const onExit = () =>
      finish(() =>
        reject(
          new Error(
            `${name} exited before it came up — is something else already serving :${BACKEND_PORT}?`,
          ),
        ),
      );

    function finish(settle: () => void): void {
      clearTimeout(timer);
      lines.off("line", onLine);
      child.off("exit", onExit);
      settle();
    }

    function onLine(line: string): void {
      console.log(line);
      if (line.includes(marker)) finish(resolvePromise);
    }

    lines.on("line", onLine);
    child.on("exit", onExit);
  });
}

/**
 * Vite gets a port probe rather than a stdout marker: `--strictPort` makes it
 * exit rather than drift to another port, and the web port was checked free
 * before it started, so anything answering here is ours.
 */
async function waitForPort(url: string, child: ChildProcess, name: string): Promise<void> {
  const deadline = Date.now() + 120_000;
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
    console.log(`data: ${existing.length} mesocycle(s) already in ${DB_NAME} — left as they are`);
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

/**
 * The plan list, with every way a stranger on the port can answer turned into a
 * message that names the real problem. Reusing whatever holds `:3000` means the
 * reply may be an unrelated service's 404 page, and a bare `.json()` would
 * surface that as an unexplained syntax error.
 */
async function plans(): Promise<{ id: number }[]> {
  const url = `${API_URL}/mesocycles`;
  const notOurs = `something is serving :${BACKEND_PORT}, but it does not answer like structure-server`;

  let response: Response;
  try {
    response = await fetch(url);
  } catch (error) {
    throw new Error(`could not reach ${url}: ${error instanceof Error ? error.message : error}`);
  }
  if (!response.ok) {
    throw new Error(`${url} answered ${response.status} — ${notOurs}`);
  }

  const body = await response.text();
  let parsed: unknown;
  try {
    parsed = JSON.parse(body);
  } catch {
    throw new Error(`${url} did not return JSON — ${notOurs}`);
  }
  if (!Array.isArray(parsed)) {
    throw new Error(`${url} did not return a list of mesocycles — ${notOurs}`);
  }
  return parsed as { id: number }[];
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
      ? `    data    the backend already on :${BACKEND_PORT} — not ${DB_NAME}`
      : `    data    ${DB_NAME}  (npm run app:reset for a fresh one)`,
    "",
    "  Ctrl-C to stop",
    rule,
    "",
  ];
  console.log(lines.join("\n"));
}

let shuttingDown = false;

async function shutdown(code: number): Promise<void> {
  if (shuttingDown) return;
  shuttingDown = true;

  signalGroups("SIGTERM");
  await Promise.race([allExited(), new Promise((r) => setTimeout(r, 3_000))]);
  signalGroups("SIGKILL");

  process.exit(code);
}

/**
 * Signals each child's whole process group. `cargo run` is a wrapper: killing it
 * alone leaves the `structure-server` it spawned holding the port and the
 * database, which then poisons every later run.
 */
function signalGroups(signal: "SIGTERM" | "SIGKILL"): void {
  for (const child of spawned) {
    if (child.pid === undefined || child.exitCode !== null || child.signalCode !== null) continue;
    try {
      process.kill(-child.pid, signal);
    } catch {
      // Already gone, or the group outlived its leader — nothing to signal.
    }
  }
}

async function allExited(): Promise<void> {
  while (spawned.some((child) => child.exitCode === null && child.signalCode === null)) {
    await new Promise((r) => setTimeout(r, 100));
  }
}

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => void shutdown(0));
}

try {
  await run();
} catch (error) {
  console.error(`\n${error instanceof Error ? error.message : String(error)}`);
  await shutdown(1);
}

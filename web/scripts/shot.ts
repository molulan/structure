import { chromium, type ConsoleMessage } from "@playwright/test";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";

// A camera pointed at the running dev stack. Because it drives the server that is
// already up, a change is on screen as fast as Vite can hot-reload it — where the
// e2e suite, the only other thing that screenshots the app, pays a fresh cargo
// build and browser launch every run.
//
// It also fails on anything the browser complained about, so a screenshot that
// *looks* right but logged a React key warning or a 500 from /api still reports a
// problem. A silent console is half of "does this work".
//
// Each route's last frame is kept, so a capture can be compared with the one it
// replaced and report how much of the page actually moved.

const WEB_URL = process.env.STRUCTURE_WEB_URL ?? "http://127.0.0.1:5173";
const DEFAULT_VIEWPORT = { width: 1280, height: 900 };
const OUT_DIR = ".shots";
const PREV_DIR = `${OUT_DIR}/prev`;
const DIFF_DIR = `${OUT_DIR}/diff`;

interface Options {
  paths: string[];
  viewport: { width: number; height: number };
  fullPage: boolean;
}

function parseArgs(argv: string[]): Options {
  const paths: string[] = [];
  let viewport = DEFAULT_VIEWPORT;
  let fullPage = true;

  for (const arg of argv) {
    if (arg === "--viewport-only") {
      fullPage = false;
    } else if (arg.startsWith("--viewport=")) {
      const [width, height] = arg.slice("--viewport=".length).split("x").map(Number);
      if (!width || !height) {
        throw new Error(`could not read a WIDTHxHEIGHT out of ${arg}`);
      }
      viewport = { width, height };
    } else if (arg.startsWith("--")) {
      throw new Error(`unknown flag ${arg}`);
    } else {
      paths.push(arg);
    }
  }

  return { paths: paths.length > 0 ? paths : ["/"], viewport, fullPage };
}

/** `/mesocycles/1` → `mesocycles-1`, so the filenames say what they show. */
function slug(path: string): string {
  const trimmed = path.replace(/^\/+|\/+$/g, "");
  return trimmed === "" ? "home" : trimmed.replace(/[^a-zA-Z0-9]+/g, "-");
}

/** First line only — Playwright appends a call log that buries the message. */
function describe(error: unknown): string {
  return (error instanceof Error ? error.message : String(error)).split("\n")[0];
}

/**
 * How old the frame being compared against is.
 *
 * A verdict is only about the last edit if the frame it is measured against is
 * from just before it, and nothing else says which is the case — a week-old
 * baseline reports a week of commits in exactly the same words.
 */
function age(file: string): string {
  const seconds = Math.max(0, Math.round((Date.now() - statSync(file).mtimeMs) / 1000));
  if (seconds < 90) return `${seconds}s old`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 90) return `${minutes}m old`;
  const hours = Math.round(minutes / 60);
  if (hours < 36) return `${hours}h old`;
  return `${Math.round(hours / 24)}d old`;
}

type Comparison =
  | { kind: "identical" }
  | { kind: "changed"; pixels: number; percent: number }
  | { kind: "resized"; from: string; to: string };

/**
 * The frame this route rendered last time, against the one just captured.
 *
 * The count is the point rather than the diff image: an edit that never reached
 * the screen — wrong selector, a class nothing uses, a hot reload that didn't
 * take — renders exactly like one that worked, and only a pixel count tells
 * those apart without someone looking.
 *
 * The only writer of the diff file, so a run that does not reach here leaves
 * none behind.
 */
function compare(previousFile: string, current: Buffer, diffFile: string): Comparison {
  const previous = PNG.sync.read(readFileSync(previousFile));
  const currentPng = PNG.sync.read(current);

  // A full-page capture grows with its content, so two runs can disagree on
  // height. There is no per-pixel correspondence to compare then — the size
  // change is itself the finding.
  if (previous.width !== currentPng.width || previous.height !== currentPng.height) {
    return {
      kind: "resized",
      from: `${previous.width}×${previous.height}`,
      to: `${currentPng.width}×${currentPng.height}`,
    };
  }

  const diff = new PNG({ width: currentPng.width, height: currentPng.height });
  const pixels = pixelmatch(
    previous.data,
    currentPng.data,
    diff.data,
    currentPng.width,
    currentPng.height,
    // How far a pixel's colour may drift before it counts as changed. Stated
    // rather than left implicit because it is the knob to turn for a more or
    // less sensitive diff; antialiased pixels are a separate matter, and
    // pixelmatch already leaves those uncounted unless asked not to.
    { threshold: 0.1 },
  );

  if (pixels === 0) {
    return { kind: "identical" };
  }

  mkdirSync(DIFF_DIR, { recursive: true });
  writeFileSync(diffFile, PNG.sync.write(diff));
  return {
    kind: "changed",
    pixels,
    percent: (pixels / (currentPng.width * currentPng.height)) * 100,
  };
}

async function reachable(url: string): Promise<boolean> {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(2_000) })).ok;
  } catch {
    return false;
  }
}

const { paths, viewport, fullPage } = parseArgs(process.argv.slice(2));

if (!(await reachable(WEB_URL))) {
  console.error(`nothing is serving ${WEB_URL} — start the stack first with: npm run app`);
  process.exit(1);
}

mkdirSync(OUT_DIR, { recursive: true });

let complaints = 0;

// One capture per output name. Two paths that slug alike would have the second
// keep the first's fresh capture as the frame to compare against, so the route
// would be measured against itself and the real previous frame lost.
const shots = new Map<string, string>();
for (const path of paths) {
  const name = slug(path);
  const claimed = shots.get(name);
  if (claimed === undefined) {
    shots.set(name, path);
  } else {
    console.log(`  ! ${path} and ${claimed} are both ${name}.png — skipping ${path}`);
    complaints += 1;
  }
}

const browser = await chromium.launch();

try {
  for (const [name, path] of shots) {
    const context = await browser.newContext({ viewport });
    const page = await context.newPage();
    const problems: string[] = [];

    page.on("console", (message: ConsoleMessage) => {
      if (message.type() === "error" || message.type() === "warning") {
        problems.push(`console.${message.type()}: ${message.text()}`);
      }
    });
    page.on("pageerror", (error) => problems.push(`uncaught: ${error.message}`));
    page.on("response", (response) => {
      if (response.status() >= 400) {
        problems.push(`${response.status()} ${response.url()}`);
      }
    });

    const url = `${WEB_URL}/${path.replace(/^\/+/, "")}`;
    const file = `${OUT_DIR}/${name}.png`;
    const previousFile = `${PREV_DIR}/${name}.png`;
    const diffFile = `${DIFF_DIR}/${name}.png`;

    let captured: Buffer | undefined;
    let settled = true;

    try {
      await page.goto(url, { waitUntil: "load", timeout: 30_000 });

      // Settling is best-effort: a route that polls or refetches never gives
      // 500ms of network silence, and a screenshot of a page rendering fine
      // beats a hard failure. Say so rather than silently capturing early.
      settled = await page
        .waitForLoadState("networkidle", { timeout: 5_000 })
        .then(() => true)
        .catch(() => false);

      // Held in memory rather than written straight through: nothing on disk
      // moves until there is a frame to put in its place, so a capture that
      // fails leaves the last good screenshot where its reader expects it.
      captured = await page.screenshot({ fullPage, timeout: 15_000 });
    } catch (error) {
      // One route that cannot be captured must not abort the others or leave
      // the browser open. Non-app URLs are the usual cause — a bare .svg hangs
      // Playwright's full-page capture — and this is a tool for app routes.
      console.log(`  ! could not capture ${url}: ${describe(error)}`);
      complaints += 1;
    } finally {
      await context.close();
    }

    if (captured !== undefined) {
      if (existsSync(file)) {
        mkdirSync(PREV_DIR, { recursive: true });
        renameSync(file, previousFile);
      } else {
        // Nothing was replaced, so whatever sits in prev/ is not the frame
        // before this one, and a verdict against it would describe some older
        // stretch of edits as if it described this one.
        rmSync(previousFile, { force: true });
      }
      // Cleared for every capture, not only the ones that reach a verdict, so
      // no run can leave a diff behind describing a comparison it never made.
      rmSync(diffFile, { force: true });

      writeFileSync(file, captured);
      console.log(`${file}  ←  ${url}`);
      if (!settled) {
        console.log("  · network never went idle — captured after 5s anyway");
      }

      if (existsSync(previousFile)) {
        const against = age(previousFile);
        try {
          const comparison = compare(previousFile, captured, diffFile);
          if (comparison.kind === "changed") {
            console.log(
              `  · changed ${comparison.percent.toFixed(1)}% of pixels (${comparison.pixels}) vs a frame ${against} → ${diffFile}`,
            );
          } else if (comparison.kind === "identical") {
            console.log(`  · identical to the previous frame (${against})`);
          } else {
            console.log(
              `  · size changed ${comparison.from} → ${comparison.to} vs a frame ${against}, not compared`,
            );
          }
        } catch (error) {
          // A comparison that cannot be made says nothing about the capture,
          // which has already succeeded and is on disk under the name printed
          // above. Reporting this as a capture failure would contradict that.
          console.log(`  ! could not compare with the previous frame: ${describe(error)}`);
          complaints += 1;
        }
      }
    }

    for (const problem of problems) {
      console.log(`  ! ${problem}`);
    }
    complaints += problems.length;
  }
} finally {
  await browser.close();
}

if (complaints > 0) {
  console.error(`\n${complaints} problem(s) above — the screenshot is not the whole story`);
  process.exit(1);
}

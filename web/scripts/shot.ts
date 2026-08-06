import { chromium, type ConsoleMessage } from "@playwright/test";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
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
 */
function compare(previousFile: string, currentFile: string, diffFile: string): Comparison {
  const previous = PNG.sync.read(readFileSync(previousFile));
  const current = PNG.sync.read(readFileSync(currentFile));

  // A full-page capture grows with its content, so two runs can disagree on
  // height. There is no per-pixel correspondence to compare then — the size
  // change is itself the finding.
  if (previous.width !== current.width || previous.height !== current.height) {
    return {
      kind: "resized",
      from: `${previous.width}×${previous.height}`,
      to: `${current.width}×${current.height}`,
    };
  }

  const diff = new PNG({ width: current.width, height: current.height });
  const pixels = pixelmatch(
    previous.data,
    current.data,
    diff.data,
    current.width,
    current.height,
    // Leaves antialiasing uncounted, so a re-render that shifts nothing reads
    // as identical rather than as a few hundred stray pixels.
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
    percent: (pixels / (current.width * current.height)) * 100,
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

const browser = await chromium.launch();
let complaints = 0;

try {
  for (const path of paths) {
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
    try {
      await page.goto(url, { waitUntil: "load", timeout: 30_000 });

      // Settling is best-effort: a route that polls or refetches never gives
      // 500ms of network silence, and a screenshot of a page rendering fine
      // beats a hard failure. Say so rather than silently capturing early.
      const settled = await page
        .waitForLoadState("networkidle", { timeout: 5_000 })
        .then(() => true)
        .catch(() => false);

      const name = slug(path);
      const file = `${OUT_DIR}/${name}.png`;
      const previousFile = `${PREV_DIR}/${name}.png`;
      const diffFile = `${DIFF_DIR}/${name}.png`;

      // Moved aside before the capture overwrites it, so "shoot, change, shoot
      // again" always has a before-frame — without the caller having to
      // remember to keep one, which is the point at which the loop breaks.
      if (existsSync(file)) {
        mkdirSync(PREV_DIR, { recursive: true });
        renameSync(file, previousFile);
      }

      await page.screenshot({ path: file, fullPage, timeout: 15_000 });
      console.log(`${file}  ←  ${url}`);
      if (!settled) {
        console.log("  · network never went idle — captured after 5s anyway");
      }

      if (existsSync(previousFile)) {
        const comparison = compare(previousFile, file, diffFile);
        if (comparison.kind === "changed") {
          console.log(
            `  · changed ${comparison.percent.toFixed(1)}% of pixels (${comparison.pixels}) → ${diffFile}`,
          );
        } else {
          // A diff left from an earlier run would describe a comparison this
          // one didn't make.
          rmSync(diffFile, { force: true });
          console.log(
            comparison.kind === "identical"
              ? "  · identical to the previous frame"
              : `  · size changed ${comparison.from} → ${comparison.to}, not compared`,
          );
        }
      }

      for (const problem of problems) {
        console.log(`  ! ${problem}`);
      }
      complaints += problems.length;
    } catch (error) {
      // One route that cannot be captured must not abort the others or leave
      // the browser open. Non-app URLs are the usual cause — a bare .svg hangs
      // Playwright's full-page capture — and this is a tool for app routes.
      console.log(`  ! could not capture ${url}: ${describe(error)}`);
      complaints += 1;
    } finally {
      await context.close();
    }
  }
} finally {
  await browser.close();
}

if (complaints > 0) {
  console.error(`\n${complaints} browser complaint(s) above — the screenshot is not the whole story`);
  process.exit(1);
}

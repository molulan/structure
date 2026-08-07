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
// replaced and say how much of the page actually moved.

const WEB_URL = process.env.STRUCTURE_WEB_URL ?? "http://127.0.0.1:5173";
const DEFAULT_VIEWPORT = { width: 1280, height: 900 };
const OUT_DIR = ".shots";
const PREV_DIR = `${OUT_DIR}/prev`;
const DIFF_DIR = `${OUT_DIR}/diff`;

// Well below pixelmatch's 0.1 default, which is tuned to swallow rendering noise
// in a regression suite and takes `#ffffff` → `#e6e6e6` with it — a shade change
// anyone can see, reported as no change at all. At 0.02 that reads as changed
// while a 1/255 drift still doesn't.
const COLOUR_TOLERANCE = 0.02;

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

/**
 * A capture taken at anything but the default framing is its own baseline —
 * sharing a name would have a mobile check overwrite the desktop frame, leaving
 * every run after it able to report only that the size had changed. So only the
 * default keeps the bare slug.
 */
function outputName(path: string, { viewport, fullPage }: Options): string {
  const sized =
    viewport.width !== DEFAULT_VIEWPORT.width || viewport.height !== DEFAULT_VIEWPORT.height;
  if (!sized && fullPage) return slug(path);
  return `${slug(path)}@${viewport.width}x${viewport.height}${fullPage ? "" : "-viewport"}`;
}

/** First line only — Playwright appends a call log that buries the message. */
function describe(error: unknown): string {
  return (error instanceof Error ? error.message : String(error)).split("\n")[0];
}

/**
 * A verdict is only about the last edit if the frame behind it is from just
 * before that edit, and nothing else says whether it is — a week-old baseline
 * reports a week of commits in exactly the same words. Rounded down throughout,
 * so the age never claims to be older than it is.
 */
function age(file: string): string {
  const seconds = Math.max(0, Math.floor((Date.now() - statSync(file).mtimeMs) / 1000));
  if (seconds < 90) return `${seconds}s old`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 90) return `${minutes}m old`;
  const hours = Math.floor(minutes / 60);
  if (hours < 36) return `${hours}h old`;
  return `${Math.floor(hours / 24)}d old`;
}

/**
 * Reported to whatever precision keeps it non-zero, because a single-element
 * edit is a few hundred pixels of a million and rounds to `0.0%` — a figure
 * that flatly contradicts the word "changed" next to it.
 */
function share(pixels: number, total: number): string {
  const percent = (pixels / total) * 100;
  if (percent >= 1) return `${percent.toFixed(1)}%`;
  if (percent >= 0.1) return `${percent.toFixed(2)}%`;
  return "<0.1%";
}

interface Comparison {
  resized?: { from: string; to: string };
  pixels: number;
  total: number;
  diff?: string;
  diffError?: string;
}

/** The top-left `width`×`height` of `png` — the area two frames of different heights share. */
function sharedArea(png: PNG, width: number, height: number): PNG {
  if (png.width === width && png.height === height) return png;
  const cropped = new PNG({ width, height });
  PNG.bitblt(png, cropped, 0, 0, width, height, 0, 0);
  return cropped;
}

/**
 * The count is the point rather than the diff image: an edit that never reached
 * the screen — wrong selector, a class nothing uses, a hot reload that didn't
 * take — renders exactly like one that worked, and only a pixel count tells
 * those apart without someone looking.
 *
 * A full-page capture grows with its content, so an edit that adds height would
 * have nothing to compare if the frames had to match exactly. The shared area is
 * compared instead, and the size change reported alongside it.
 */
function compare(previousBytes: Buffer, current: Buffer, diffFile: string): Comparison {
  const previous = PNG.sync.read(previousBytes);
  const currentPng = PNG.sync.read(current);

  const resized =
    previous.width !== currentPng.width || previous.height !== currentPng.height
      ? {
          from: `${previous.width}×${previous.height}`,
          to: `${currentPng.width}×${currentPng.height}`,
        }
      : undefined;

  const width = Math.min(previous.width, currentPng.width);
  const height = Math.min(previous.height, currentPng.height);
  const total = width * height;
  const diff = new PNG({ width, height });
  const pixels = pixelmatch(
    sharedArea(previous, width, height).data,
    sharedArea(currentPng, width, height).data,
    diff.data,
    width,
    height,
    // Antialiased pixels count. They are excluded by default to absorb renderer
    // noise, but this only runs when the two files differ byte for byte, so a
    // difference confined to them is a real one — and dropping it would report
    // a font-weight or radius change as no change at all.
    { threshold: COLOUR_TOLERANCE, includeAA: true },
  );

  if (pixels === 0) {
    return { resized, pixels, total };
  }

  try {
    mkdirSync(DIFF_DIR, { recursive: true });
    writeFileSync(diffFile, PNG.sync.write(diff));
    return { resized, pixels, total, diff: diffFile };
  } catch (error) {
    // The count survives a diff that could not be saved: it is the signal, and
    // the image only shows where.
    return { resized, pixels, total, diffError: describe(error) };
  }
}

async function reachable(url: string): Promise<boolean> {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(2_000) })).ok;
  } catch {
    return false;
  }
}

const options = parseArgs(process.argv.slice(2));
const { viewport, fullPage } = options;

if (!(await reachable(WEB_URL))) {
  console.error(`nothing is serving ${WEB_URL} — start the stack first with: npm run app`);
  process.exit(1);
}

mkdirSync(OUT_DIR, { recursive: true });

// Every distinct route is visited, because a route that is never loaded has its
// console and failed responses go unchecked. Only the repeats of one route are
// dropped, and two routes whose names would collide are told apart, so no
// capture can land on another's baseline.
const shots: Array<{ name: string; path: string }> = [];
const visited = new Set<string>();
const taken = new Set<string>();
for (const path of options.paths) {
  const route = path.replace(/^\/+|\/+$/g, "");
  if (visited.has(route)) continue;
  visited.add(route);

  const wanted = outputName(path, options);
  let name = wanted;
  for (let n = 2; taken.has(name); n++) name = `${wanted}-${n}`;
  taken.add(name);
  shots.push({ name, path });
}

const browser = await chromium.launch();
let complaints = 0;

try {
  for (const { name, path } of shots) {
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
    const pendingFile = `${file}.pending`;
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
      // Ahead of the writing, so every line below it is attributable to this
      // route even when the writing is what failed.
      console.log(`${file}  ←  ${url}`);
      if (!settled) {
        console.log("  · network never went idle — captured after 5s anyway");
      }

      let rotated = false;
      try {
        // Written aside and moved into place, so a write that fails partway
        // leaves the frame its reader has open untouched.
        writeFileSync(pendingFile, captured);
        if (existsSync(file)) {
          mkdirSync(PREV_DIR, { recursive: true });
          renameSync(file, previousFile);
          rotated = true;
        } else {
          // Nothing was replaced, so whatever sits in prev/ is not the frame
          // before this one, and a verdict against it would describe some
          // older stretch of edits as if it described this one.
          rmSync(previousFile, { force: true });
        }
        // Unconditional, so no run can leave a diff behind describing a
        // comparison it never made.
        rmSync(diffFile, { force: true });
        renameSync(pendingFile, file);

        if (existsSync(previousFile)) {
          try {
            const against = age(previousFile);
            const previousBytes = readFileSync(previousFile);
            const comparison: Comparison = captured.equals(previousBytes)
              ? { pixels: 0, total: 0 }
              : compare(previousBytes, captured, diffFile);
            const { resized, pixels } = comparison;

            if (resized && pixels > 0) {
              console.log(
                `  · size changed ${resized.from} → ${resized.to}; ${pixels} pixels (${share(pixels, comparison.total)}) of the shared area differ, vs a frame ${against}${comparison.diff ? ` → ${comparison.diff}` : ""}`,
              );
            } else if (resized) {
              console.log(
                `  · size changed ${resized.from} → ${resized.to}; the shared area is identical, vs a frame ${against}`,
              );
            } else if (pixels > 0) {
              console.log(
                `  · changed ${pixels} pixels (${share(pixels, comparison.total)}) vs a frame ${against}${comparison.diff ? ` → ${comparison.diff}` : ""}`,
              );
            } else {
              console.log(`  · identical to the previous frame (${against})`);
            }

            if (comparison.diffError !== undefined) {
              console.log(`  ! could not write ${diffFile}: ${comparison.diffError}`);
              complaints += 1;
            }
          } catch (error) {
            // A comparison that cannot be made says nothing about the capture,
            // which has already succeeded and is on disk under the name printed
            // above. Reporting this as a capture failure would contradict that.
            console.log(`  ! could not compare with the previous frame: ${describe(error)}`);
            complaints += 1;
          }
        } else {
          // Silence here would read the same as a comparison that ran and found
          // nothing, which is the opposite conclusion.
          console.log("  · no previous frame to compare with");
        }
      } catch (error) {
        // The capture is already lost; at least leave the frame that was there
        // before, rather than a route with no screenshot at all.
        if (rotated && !existsSync(file) && existsSync(previousFile)) {
          try {
            renameSync(previousFile, file);
          } catch {
            console.log(`  ! ${file} could not be put back after the failure below`);
          }
        }
        rmSync(pendingFile, { force: true });
        console.log(`  ! could not store ${file}: ${describe(error)}`);
        complaints += 1;
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

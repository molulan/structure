import { chromium, type ConsoleMessage } from "@playwright/test";
import { mkdirSync } from "node:fs";

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

for (const path of paths) {
  const page = await (await browser.newContext({ viewport })).newPage();
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
  await page.goto(url, { waitUntil: "networkidle" });

  const file = `${OUT_DIR}/${slug(path)}.png`;
  await page.screenshot({ path: file, fullPage });
  console.log(`${file}  ←  ${url}`);

  for (const problem of problems) {
    console.log(`  ! ${problem}`);
  }
  complaints += problems.length;

  await page.context().close();
}

await browser.close();

if (complaints > 0) {
  console.error(`\n${complaints} browser complaint(s) above — the screenshot is not the whole story`);
  process.exit(1);
}

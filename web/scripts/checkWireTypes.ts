import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// src/api/wire.ts is committed so the frontend typechecks without a cargo run,
// which leaves it free to fall behind the Rust types. Comparing contents rather
// than git lets a regenerated but uncommitted tree pass.
//
// The file is removed before the generator runs, so an absent one afterwards
// says the generator never ran: `cargo test <filter>` exits 0 when the filter
// matches nothing, which would otherwise make an ignored or moved export test
// look exactly like success.

const WEB_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const BINDINGS = join(WEB_DIR, "src/api/wire.ts");
const GENERATOR = "cargo test --manifest-path ../Cargo.toml -p structure-server --lib wire";

const before = existsSync(BINDINGS) ? readFileSync(BINDINGS, "utf8") : null;

rmSync(BINDINGS, { force: true });

const [command, ...args] = GENERATOR.split(" ");
const generated = spawnSync(command, args, { cwd: WEB_DIR, stdio: "inherit" });
if (generated.status !== 0) {
  if (before !== null) writeFileSync(BINDINGS, before);
  fail(`${GENERATOR} failed — the bindings could not be regenerated.`);
}

if (!existsSync(BINDINGS)) {
  if (before !== null) writeFileSync(BINDINGS, before);
  fail(
    "src/api/wire.ts was not written. The export test in structure-server/src/wire.rs " +
      `did not run — check that it still matches the filter in \`${GENERATOR}\`.`,
  );
}

if (before === null) {
  fail("src/api/wire.ts was missing. It has just been generated — commit it.");
}

if (readFileSync(BINDINGS, "utf8") !== before) {
  fail(
    "src/api/wire.ts was out of date with the Rust wire types.\n" +
      "It has just been regenerated in place — review the diff and commit it.",
  );
}

function fail(message: string): never {
  console.error(`\n${message}`);
  process.exit(1);
}

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// src/api/wire.ts is committed so the frontend typechecks without a cargo run,
// which leaves it free to fall behind the Rust types. Comparing contents rather
// than git lets a regenerated but uncommitted tree pass.

const WEB_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const BINDINGS = join(WEB_DIR, "src/api/wire.ts");
const GENERATOR = "cargo test --manifest-path ../Cargo.toml -p structure-server --lib wire";

const before = readFileSync(BINDINGS, "utf8");

const [command, ...args] = GENERATOR.split(" ");
const generated = spawnSync(command, args, { cwd: WEB_DIR, stdio: "inherit" });
if (generated.status !== 0) {
  console.error(`\n${GENERATOR} failed — the bindings could not be regenerated.`);
  process.exit(1);
}

if (readFileSync(BINDINGS, "utf8") !== before) {
  console.error(
    `\nsrc/api/wire.ts was out of date with the Rust wire types.\n` +
      `It has just been regenerated in place — review the diff and commit it.`,
  );
  process.exit(1);
}

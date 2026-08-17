import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// src/api/wire.ts is written by a Rust test from the types the server serializes,
// and committed so the frontend typechecks without a cargo run. Committing a
// generated file is the one thing that can drift, so this regenerates it and
// fails if the contents moved: a Rust wire type changed without its bindings.
//
// The comparison is against the file's contents rather than against git, so a
// working tree that has legitimately regenerated but not yet committed passes.

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

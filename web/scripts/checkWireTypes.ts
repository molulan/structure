import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// src/api/wire.ts is committed so the frontend typechecks without a cargo run,
// which leaves it free to fall behind the Rust types. Comparing contents rather
// than git lets a regenerated but uncommitted tree pass.
//
// The export goes to a temp directory: the committed file is then never in a
// state anyone could be interrupted in, and an export that produced nothing
// there says the generator did not run — `cargo test <filter>` exits 0 when the
// filter matches nothing, which would otherwise look exactly like success.

const WEB_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const BINDINGS = join(WEB_DIR, "src/api/wire.ts");
const GENERATOR = "cargo test --manifest-path ../Cargo.toml -p structure-server --lib wire";

const outDir = mkdtempSync(join(tmpdir(), "structure-wire-"));

try {
  const [command, ...args] = GENERATOR.split(" ");
  const generated = spawnSync(command, args, {
    cwd: WEB_DIR,
    stdio: "inherit",
    env: { ...process.env, STRUCTURE_WIRE_OUT_DIR: outDir },
  });
  if (generated.status !== 0) {
    fail(`${GENERATOR} failed — the bindings could not be regenerated.`);
  }

  const exported = join(outDir, "wire.ts");
  if (!existsSync(exported)) {
    fail(
      "no bindings were exported. The export test in structure-server/src/wire.rs " +
        `did not run — check that it still matches the filter in \`${GENERATOR}\`.`,
    );
  }

  const fresh = readFileSync(exported, "utf8");
  if (!existsSync(BINDINGS)) {
    writeFileSync(BINDINGS, fresh);
    fail("src/api/wire.ts was missing. It has just been written — commit it.");
  }

  if (readFileSync(BINDINGS, "utf8") !== fresh) {
    writeFileSync(BINDINGS, fresh);
    fail(
      "src/api/wire.ts was out of date with the Rust wire types.\n" +
        "It has just been regenerated in place — review the diff and commit it.",
    );
  }
} catch (error) {
  console.error(`\n${error instanceof Error ? error.message : String(error)}`);
  process.exitCode = 1;
} finally {
  rmSync(outDir, { recursive: true, force: true });
}

// Thrown rather than exited, so the temp directory is still cleaned up on the
// way out: `process.exit` would skip the `finally` above.
function fail(message: string): never {
  throw new Error(message);
}

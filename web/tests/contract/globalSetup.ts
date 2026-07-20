import { spawn, type ChildProcess } from "node:child_process";
import { createServer } from "node:net";

/**
 * Boots a real structure-server on a free port with a fresh in-memory DB, waits
 * for it to answer /health, and exposes its base URL to the tests via
 * STRUCTURE_API_BASE. Torn down when the suite finishes.
 */
export default async function setup() {
  const port = await freePort();
  const baseUrl = `http://127.0.0.1:${port}`;

  const server = spawn(
    "cargo",
    ["run", "--quiet", "--manifest-path", "../Cargo.toml", "-p", "structure-server"],
    {
      env: { ...process.env, PORT: String(port), STRUCTURE_DB: ":memory:" },
      stdio: "inherit",
    },
  );

  await waitForHealth(`${baseUrl}/health`, server);
  // The API is served under `/api`; `/health` stays at the root.
  process.env.STRUCTURE_API_BASE = `${baseUrl}/api`;

  return async () => {
    server.kill("SIGTERM");
  };
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = createServer();
    srv.on("error", reject);
    srv.listen(0, "127.0.0.1", () => {
      const address = srv.address();
      if (typeof address === "object" && address) {
        const { port } = address;
        srv.close(() => resolve(port));
      } else {
        srv.close(() => reject(new Error("could not determine a free port")));
      }
    });
  });
}

async function waitForHealth(url: string, server: ChildProcess): Promise<void> {
  const deadline = Date.now() + 180_000;
  let exitCode: number | null | undefined; // undefined while the process is still running
  const onExit = (code: number | null) => {
    exitCode = code ?? null;
  };
  server.on("exit", onExit);
  try {
    while (Date.now() < deadline) {
      if (exitCode !== undefined) {
        throw new Error(`structure-server exited (code ${exitCode}) before becoming healthy`);
      }
      try {
        const response = await fetch(url);
        if (response.ok) return;
      } catch {
        // not up yet
      }
      await new Promise((r) => setTimeout(r, 250));
    }
    server.kill("SIGKILL");
    throw new Error(`structure-server did not become healthy within the timeout at ${url}`);
  } finally {
    server.off("exit", onExit);
  }
}

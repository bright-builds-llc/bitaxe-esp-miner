// Shared real-process fixtures: private roots, a fake `just` on PATH and detached stand-in owners.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createServer } from "node:net";
import { mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isLive, liveIdentity, nodeBinary, waitFor } from "./host.mjs";

const fakeJust = fileURLToPath(new URL("./fake-just.test-helper.mjs", import.meta.url));

/** A 0700 parent with a 0700 private root inside it, plus a fake `just` whose behavior `scenario` sets. */
export async function workspaceFixture(scenario = {}) {
  const base = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "hardware-operator-")));
  const parent = resolve(base, "parent"), root = resolve(parent, "attempt-001"), bin = resolve(base, "bin"), fake = resolve(base, "fake");
  for (const directory of [parent, root, bin, fake]) await mkdir(directory, { mode: 0o700 });
  await writeFile(resolve(fake, "scenario.json"), JSON.stringify(scenario));
  // The real Node binary: production strips the rules_js launcher environment from nested `just` calls.
  await writeFile(resolve(bin, "just"), `#!/bin/sh\nexec '${nodeBinary()}' '${fakeJust}' "$@"\n`, { mode: 0o700 });
  const env = { ...process.env, PATH: `${bin}:/usr/bin:/bin`, BUILD_WORKSPACE_DIRECTORY: base, FAKE_JUST_DIR: fake };
  return { base, parent, root, env, operations: { env, ownerStopMs: 5000, collectionBeginGraceMs: 200, serveReadyMs: 5000 },
    calls: async () => (await readFile(resolve(fake, "calls.jsonl"), "utf8").catch(() => "")).trim().split("\n").filter(Boolean).map((line) => JSON.parse(line)) };
}

/** A detached stand-in server process with its own process group, as `serve` owners run. */
export async function detachedOwner() {
  const child = spawn("/bin/sleep", ["300"], { detached: true, stdio: "ignore" });
  child.unref();
  return waitFor(() => liveIdentity(child.pid), { timeoutMs: 3000, intervalMs: 50 });
}

/** A real port number with no listener, as a stopped owner leaves behind. */
export async function closedPort() {
  const server = createServer().listen(0, "127.0.0.1");
  await once(server, "listening");
  const { port } = server.address();
  await new Promise((done) => server.close(done));
  return port;
}

export async function writeServerOwner(directory, owner, maybePort = null) {
  const port = maybePort ?? await closedPort();
  await mkdir(directory, { recursive: true, mode: 0o700 });
  await writeFile(resolve(directory, "server-owner.json"), `${JSON.stringify({ owner, port })}\n`, { mode: 0o600 });
}

export async function writeCollection(directory, { startedAtUnixMs = Date.now(), finished = true } = {}) {
  await writeFile(resolve(directory, "collection-begin.json"), JSON.stringify({ schema: "str005-recovery-collection-v1", startedAtUnixMs }), { mode: 0o600 });
  if (finished) await writeFile(resolve(directory, "finished.json"), "{}", { mode: 0o600 });
}

export async function kill(identity) {
  if (identity && await isLive(identity)) process.kill(-identity.pgid, "SIGKILL");
}


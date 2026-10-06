// Repo-owned Noise-serial operator parent, launched only by `just hardware-operator noise-launch`.
// It owns the detached `serve` supervisor and answers one JSON line on stdout per JSON command line
// read from stdin (the held-open command FIFO). Procedure: scripts/str005-noise-serial/README.md.
import { spawn } from "node:child_process";
import { open, realpath } from "node:fs/promises";
import { resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { recordCleanup } from "../str005-noise-serial/cleanup.mjs";
import { loadContext } from "../str005-noise-serial/context.mjs";
import { digest, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireLsofAbsent, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { baseline, readJournal } from "../str005-noise-serial/journal.mjs";
import { installCandidate, observeOwnedExit } from "../str005-noise-serial/operator.mjs";
import { GATE_PORT, operatorDirectory, parseCommand } from "./noise-protocol.mjs";

const SUPERVISOR_STOP_MS = 5000;
const pause = (milliseconds) => new Promise((done) => setTimeout(done, milliseconds));
const output = (value) => process.stdout.write(`${JSON.stringify(value)}\n`);
const fault = (error) => ({
  event: "operator_error",
  code: typeof error?.code === "string" && /^[a-zA-Z0-9_]+$/u.test(error.code) ? error.code
    : typeof error?.message === "string" && /^[a-z][a-z0-9_]*$/u.test(error.message) ? error.message : "operator_failed",
});

function privateRootArgument(argv) {
  const values = argv.filter((value) => value.startsWith("--private-root=")).map((value) => value.slice("--private-root=".length));
  if (argv.length !== 1 || values.length !== 1 || !values[0].startsWith("/")) throw Error("noise_parent_arguments");
  return resolve(values[0]);
}

async function startSupervisor(repository, root) {
  requireLsofAbsent(["-nP", `-iTCP:${GATE_PORT}`, "-sTCP:LISTEN", "-t"]);
  const node = await realpath(process.execPath);
  const stdout = await open(`${root}.serve.stdout.log`, "wx", 0o600), stderr = await open(`${root}.serve.stderr.log`, "wx", 0o600);
  const child = spawn(node, [resolve(repository, "scripts/str005-noise-serial/main.mjs"), "serve", "--private-root", root, "--port", String(GATE_PORT)], {
    cwd: repository, detached: true, stdio: ["ignore", stdout.fd, stderr.fd],
    env: { PATH: process.env.PATH, HOME: process.env.HOME, LANG: "C", LC_ALL: "C", ...(process.env.TZ ? { TZ: process.env.TZ } : {}) },
  });
  const state = { exited: false };
  child.once("close", () => { state.exited = true; });
  child.once("error", (error) => output(fault(error)));
  await stdout.close(); await stderr.close();
  return { child, state };
}

async function awaitOwnership(root, child, state) {
  let owner;
  for (let index = 0; index < 30 && !owner && !state.exited; index++) {
    owner = (await processSnapshot()).find((row) => row.pid === child.pid && row.pgid === child.pid);
    if (!owner) await pause(100);
  }
  if (!owner || state.exited) throw Error("supervisor_start_failed");
  return owner;
}

async function awaitServer(root, child, state, owner) {
  let server;
  for (let index = 0; index < 300 && !server && !state.exited; index++) {
    // A SyntaxError is a record the supervisor is still writing.
    try { server = (await proof(root, "server-owner.json")).value; } catch (error) { if (error.code !== "ENOENT" && !(error instanceof SyntaxError)) throw error; }
    if (!server) await pause(100);
  }
  if (!server || !sameProcess(server.owner, owner) || state.exited) { child.kill("SIGTERM"); throw Error("supervisor_readiness_failed"); }
}

/** Everything before readiness; a failure here stops the supervisor and this parent so nothing lingers. */
async function startup(root, operator) {
  const repository = fileURLToPath(new URL("../../", import.meta.url));
  const context = await loadContext(root), contextSha256 = digest(JSON.stringify(context));
  let maybeChild = null;
  try {
    const { child, state } = await startSupervisor(repository, root);
    maybeChild = child;
    const owner = await awaitOwnership(root, child, state);
    const exitObserver = observeOwnedExit(child, contextSha256, owner, "supervisor");
    await writeNew(resolve(operator, "supervisor-root.json"), owner);
    await awaitServer(root, child, state, owner);
    return { context, contextSha256, child, state, exitObserver };
  } catch (error) {
    if (maybeChild && maybeChild.exitCode === null && maybeChild.signalCode === null) maybeChild.kill("SIGTERM");
    throw error;
  }
}

async function main() {
  const root = privateRootArgument(process.argv.slice(2)), operator = operatorDirectory(root);
  const { context, contextSha256, child, state, exitObserver } = await startup(root, operator);
  output({ event: "supervisor_ready", context_sha256: contextSha256 });
  let maybeBrowser = null, maybeSupervisor = null;
  const lines = createInterface({ input: process.stdin, terminal: false });
  for await (const line of lines) {
    try {
      const command = parseCommand(line);
      if (command.action === "install") {
        const result = await installCandidate(root, command.index);
        output({ event: "installation_reviewed", index: command.index, reviewed: result.installation_reviewed ?? result.reviewed ?? true });
      } else if (command.action === "status") {
        const rows = await readJournal(root, context), last = rows.at(-1)?.state;
        output({ event: "state", rows: rows.length, status: last?.status, connected: last?.connected, running: last?.running,
          failure: last?.failure ?? null, baselineConfirmed: last?.deviceBaselineConfirmed });
      } else if (command.action === "browser-closed") {
        const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state, true);
        maybeBrowser = { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
          lastSequence: last.sequence, lastStateSha256: digest(JSON.stringify(last)), observedAtUnixMs: Date.now() };
        await writeNew(resolve(operator, "browser.json"), maybeBrowser); output({ event: "browser_closure_recorded" });
      } else if (command.action === "stop") {
        exitObserver.markStopRequested(); child.kill("SIGTERM");
        let timer;
        try {
          maybeSupervisor = await Promise.race([exitObserver.receipt(),
            new Promise((_, reject) => { timer = setTimeout(() => reject(Error("supervisor_cleanup_timeout")), SUPERVISOR_STOP_MS); })]);
        } finally { clearTimeout(timer); }
        await writeNew(resolve(operator, "supervisor-exit.json"), maybeSupervisor); output({ event: "supervisor_stopped", code: maybeSupervisor.code });
      } else if (command.action === "cleanup") {
        if (!maybeBrowser || !maybeSupervisor) throw Error("observations_missing");
        await recordCleanup(root, context, { browser: maybeBrowser, supervisor: maybeSupervisor }); output({ event: "cleanup_recorded" });
      } else if (command.action === "exit") {
        if (!state.exited) throw Error("supervisor_still_live");
        // Closing readline alone keeps the FIFO read end open while the holder lives, so the parent would never exit.
        output({ event: "parent_exiting" }); lines.close(); process.stdin.destroy(); break;
      }
    } catch (error) {
      const value = fault(error);
      await writeNew(resolve(operator, `error-${Date.now()}.json`), value); output(value);
    }
  }
}

// Exit explicitly: a pre-ready failure must not keep this process alive behind a supervisor handle.
main().catch((error) => { output(fault(error)); process.exit(1); });

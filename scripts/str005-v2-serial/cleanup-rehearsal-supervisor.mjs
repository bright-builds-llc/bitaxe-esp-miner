// Test-only child. No Serial API or hardware command exists in this process.
import { spawn, execFileSync } from "node:child_process";
import { once } from "node:events";
import { tmpdir } from "node:os";
import { createSupervisor } from "./server.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";

if (!process.send) throw Error("rehearsal_ipc_required");
let sequence = 0, maybeServer, stopping = false, maybeClosing;
const pending = new Map();
const request = (method, args) => new Promise((done, reject) => {
  const id = ++sequence; pending.set(id, { done, reject }); process.send({ kind: "prerequisite", id, method, args });
});
process.on("message", message => {
  if (message.kind !== "prerequisite-result") return;
  const entry = pending.get(message.id); if (!entry) return;
  pending.delete(message.id);
  if (message.error) entry.reject(Object.assign(Error(message.error), { code: message.error })); else entry.done(message.value);
});
async function close(code) {
  if (maybeClosing) return maybeClosing;
  stopping = true;
  // A killed test runner cannot leave a helper awaiting unfinished IPC forever.
  // This is failed test-only teardown, never a successful qualification exit.
  setTimeout(() => process.exit(1), 6000).unref();
  maybeClosing = (async () => {
    try { if (maybeServer) await maybeServer.closeQualificationResources(); }
    catch { code = 1; }
    process.exit(code);
  })();
  return maybeClosing;
}
process.once("SIGTERM", () => { void close(0); });
process.once("disconnect", () => { void close(1); });
const watchdog = setTimeout(() => { void close(1); }, 120000); watchdog.unref();
try {
  const [input] = await once(process, "message");
  if (input.kind !== "initialize") throw Error("rehearsal_initialize");
  const { options, context } = input;
  await syntheticRoot(options.privateRoot, context);
  const operations = {
    cleanPushed() {}, ignored() {}, hostPlatform: "darwin",
    git: path => path === context.firmware_root ? context.firmware_commit : context.gate_commit,
    nativeSourceFiles: context.native_source_files, nativeAuditorSources: context.native_auditor_sources,
    inspectNative: value => request("inspectNative", [value]),
    inspectPredecessor: (path, scope) => request("inspectPredecessor", [path, scope]),
    inspectPermissionClosure: path => request("inspectPermissionClosure", [path]),
    inspectChannelSuccessor: path => request("inspectChannelSuccessor", [path]),
    checkCurrentSuccessorOwnership: value => request("checkCurrentSuccessorOwnership", [value]),
    processSnapshot,
    execFileSync(program, args, configuration) {
      // The only simulated OS resource is the nonexistent physical Serial device.
      if (program === "/usr/sbin/lsof" && args[0] === "-t" && /^\/dev\/(?:cu|tty)\.synthetic$/u.test(args[1]))
        throw Object.assign(Error("synthetic_serial_absence"), { status: 1, signal: null, stdout: "", stderr: "" });
      return execFileSync(program, args, configuration);
    },
    spawn(program, args, configuration) {
      // Execute the exact source-bound test fixture bytes with an interpreter.
      // Additional IPC exists only in this helper and closes on parent death.
      return spawn(process.execPath, [program, ...args], {
        ...configuration, stdio: [...configuration.stdio, "ipc"], env: { ...configuration.env, TMPDIR: tmpdir(),
          ...(process.env.TZ === undefined ? {} : { TZ: process.env.TZ }), ...nodeRuntimeEnvironment() },
      });
    },
  };
  maybeServer = await createSupervisor(options, operations);
  if (stopping) await close(1);
  maybeServer.listen(0, "127.0.0.1"); await once(maybeServer, "listening"); await maybeServer.qualificationReady;
  process.send({ kind: "ready" });
} catch (error) {
  const code = /^(?:v2|noise|iterative)_[a-z_]+$/u.test(error?.code) ? error.code : "rehearsal_supervisor_failed";
  if (process.connected) process.send({ kind: "failed", code });
  await close(1);
}

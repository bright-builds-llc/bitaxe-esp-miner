// Test-only subprocess bridge. The whitelist contains cold synthetic prerequisites;
// process identity, sockets, HTTP, exits and listener inventory are never proxied.
import { spawn, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";

const METHODS = new Set(["inspectNative", "inspectPredecessor", "inspectPermissionClosure", "inspectChannelSuccessor", "checkCurrentSuccessorOwnership"]);
export const wait = ms => new Promise(done => setTimeout(done, ms));
export async function bound(promise, milliseconds = 30000) {
  let timer;
  try { return await Promise.race([promise, new Promise((_done, reject) => { timer = setTimeout(() => reject(Error("rehearsal_deadline")), milliseconds); })]); }
  finally { clearTimeout(timer); }
}
export function realHostOperations(cold) {
  return { ...cold, processSnapshot, spawnSync: undefined,
    execFileSync(program, args, options) {
      if (program === "/usr/sbin/lsof" && args[0] === "-t" && /^\/dev\/(?:cu|tty)\.synthetic$/u.test(args[1]))
        throw Object.assign(Error("synthetic_serial_absence"), { status: 1, signal: null, stdout: "", stderr: "" });
      return execFileSync(program, args, options);
    } };
}
export async function supervisor(f) {
  await syntheticRoot(f.root, f.context);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./cleanup-rehearsal-supervisor.mjs", import.meta.url))], {
    detached: true, stdio: ["ignore", "pipe", "pipe", "ipc"],
    env: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C", TMPDIR: tmpdir(),
      // ps lstart is rendered in the observer timezone; both owners must agree.
      ...(process.env.TZ === undefined ? {} : { TZ: process.env.TZ }), ...nodeRuntimeEnvironment() },
  });
  const exited = new Promise(done => child.once("exit", (code, signal) => done({ code, signal })));
  const closed = new Promise(done => child.once("close", (code, signal) => done({ code, signal })));
  let outputBytes = 0;
  for (const stream of [child.stdout, child.stderr]) stream.on("data", bytes => { outputBytes += bytes.length; bytes.fill(0); });
  const ready = new Promise((done, reject) => {
    child.once("error", () => reject(Error("rehearsal_spawn_failed")));
    child.once("close", () => reject(Error("rehearsal_supervisor_early_close")));
    child.on("message", async message => {
      if (message.kind === "ready") { done(); return; }
      if (message.kind === "failed") { reject(Error(/^(?:v2|noise|iterative)_[a-z_]+$/u.test(message.code) ? message.code : "rehearsal_supervisor_failed")); return; }
      if (message.kind !== "prerequisite" || !METHODS.has(message.method)) { reject(Error("rehearsal_prerequisite_invalid")); return; }
      try {
        const operation = f.operations[message.method]; if (typeof operation !== "function") throw Error("rehearsal_prerequisite_missing");
        const value = await operation(...message.args, f.operations);
        if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, value });
      } catch (error) {
        const code = /^(?:v2|noise|iterative)_[a-z_]+$/u.test(error?.code) ? error.code : "rehearsal_prerequisite_failed";
        if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, error: code });
      }
    });
  });
  child.send({ kind: "initialize", options: f.options, context: f.context });
  try { await bound(ready, 45000); }
  catch (error) {
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
    try { await bound(closed, 7000); }
    catch { child.kill("SIGKILL"); await bound(closed, 3000); }
    throw error;
  }
  const saved = (await proof(f.root, "server-owner.json")).value;
  if (saved.owner.pid !== child.pid || saved.owner.pgid !== child.pid) throw Error("rehearsal_owner_mismatch");
  let maybeLastFailure = null;
  const request = async (path, input, method = "POST") => {
    let response;
    try { response = await fetch(`${saved.origin}${path}`, { method, redirect: "error", signal: AbortSignal.timeout(30000),
      headers: { Origin: saved.origin, "Content-Type": "application/json" }, ...(method === "POST" ? { body: JSON.stringify(input) } : {}) }); }
    catch { throw Error("rehearsal_http_failed"); }
    const value = await response.json();
    if (!response.ok) { maybeLastFailure ??= /^(?:v2|noise|iterative)_[a-z_]+$/u.test(value.error) ? value.error : "rehearsal_http_rejected";
      throw Object.assign(Error(maybeLastFailure), { code: maybeLastFailure }); }
    return value;
  };
  return { child, owner: saved.owner, request, closed, exited, lastFailure: () => maybeLastFailure, outputBytes: () => outputBytes,
    async stop() {
      if (child.exitCode !== null || child.signalCode !== null) { child.stdout.destroy(); child.stderr.destroy(); return exited; }
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
      try { return await bound(closed, 7000); }
      catch { child.kill("SIGKILL"); return bound(closed, 3000); }
    } };
}

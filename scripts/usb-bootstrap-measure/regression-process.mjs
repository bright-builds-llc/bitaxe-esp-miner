import { actualNodePath } from "./node-executable.mjs";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { setTimeout as pause } from "node:timers/promises";
import { parseProcessRows, runDiagnostic, processOwnership, cleanupOwned, sameProcess } from "../host-stalls/capture.mjs";
const failure = code => Object.assign(Error(code), { code });
/** Execute fixed caller-selected regression commands behind an observed owner; never acquire hardware. */
export async function runRegression(program, args, options, operations = {}) {
  if (!Number.isSafeInteger(options.timeout) || options.timeout < 1000 || !Number.isSafeInteger(options.maxBuffer) || options.maxBuffer <= 0 || options.maxBuffer > 65536)
    throw failure("bootstrap_reader_runner_bounds");
  const started = performance.now(), deadline = started + options.timeout;
  const reserve = Math.min(20000, Math.max(750, options.timeout / 3)), executionEnd = deadline - reserve;
  const ownerNode = await actualNodePath(options.nodePath ? { environment: {}, executable: options.nodePath } : {});
  const child = spawn(ownerNode, [fileURLToPath(new URL("./regression-process-child.mjs", import.meta.url))],
    { cwd: options.cwd, env: options.env, detached: true, stdio: ["ignore", "pipe", "pipe", "ipc"] });
  const chunks = { stdout: [], stderr: [] }; let retained = 0, maybeError, maybeResult, exited = false;
  const note = code => { maybeError ??= failure(code); };
  for (const stream of ["stdout", "stderr"]) child[stream].on("data", chunk => {
    const room = options.maxBuffer - retained, kept = chunk.subarray(0, Math.max(0, room));
    if (kept.length) chunks[stream].push(Buffer.from(kept)); retained += kept.length;
    if (kept.length !== chunk.length) note("bootstrap_reader_runner_output");
  });
  child.once("error", () => note("bootstrap_reader_runner_spawn"));
  child.once("exit", () => { exited = true; if (!maybeResult) note("bootstrap_reader_runner_owner_exit"); });
  child.once("message", value => { maybeResult = value; if (value.failed) note("bootstrap_reader_runner_spawn"); });
  let known = [], admitted = false;
  const snapshot = async () => {
    const remaining = deadline - performance.now();
    if (remaining <= 550) throw failure("bootstrap_reader_runner_cleanup_timeout");
    if (operations.snapshot) return operations.snapshot();
    const result = await runDiagnostic(["/bin/ps", "-axo", "pid=,ppid=,pgid=,lstart=,state=,%cpu="], Math.min(2500, remaining - 500), 524288);
    if (result.error || result.exitCode !== 0 || result.timedOut || result.truncated || result.unreaped) throw failure("bootstrap_reader_runner_snapshot");
    return parseProcessRows(result.stdout);
  };
  try {
    while (!maybeError && performance.now() < executionEnd) {
      const completedBeforeSnapshot = maybeResult !== undefined;
      const rows = await snapshot();
      const ownership = processOwnership(rows, child.pid, known, !admitted && !exited);
      for (const row of ownership.owned) if (!known.some(item => sameProcess(item, row))) known.push(row);
      if (!admitted) {
        if (ownership.owned.some(row => row.pid === child.pid && row.pgid === child.pid)) {
          admitted = true; child.send({ program, args, cwd: options.cwd, env: options.env }, error => { if (error) note("bootstrap_reader_runner_dispatch"); });
        }
      } else if (maybeResult && completedBeforeSnapshot) {
        if (ownership.owned.some(row => row.pid !== child.pid && !row.state.startsWith("Z"))) note("bootstrap_reader_runner_descendants");
        break;
      }
      if (known.length > 128) { note("bootstrap_reader_runner_ownership_bound"); break; }
      await pause(20);
    }
    if (!maybeResult && !maybeError) note("bootstrap_reader_runner_timeout");
  } catch (error) { maybeError ??= error; }
  try {
    const cleanup = await cleanupOwned(child.pid, known, snapshot);
    while (!exited && performance.now() < deadline) await pause(5);
    const rows = await snapshot(), ownership = processOwnership(rows, child.pid, known);
    const complete = cleanup.complete && exited && ownership.owned.length === 0 && ownership.unanchored.length === 0;
    if (!complete) throw failure("bootstrap_reader_runner_cleanup_unconfirmed");
    return { status: maybeResult?.status ?? null, signal: maybeResult?.signal ?? null,
      stdout: Buffer.concat(chunks.stdout), stderr: Buffer.concat(chunks.stderr), ...(maybeError ? { error: maybeError } : {}) };
  } finally {
    child.stdout.destroy(); child.stderr.destroy(); if (child.connected) child.disconnect();
  }
}

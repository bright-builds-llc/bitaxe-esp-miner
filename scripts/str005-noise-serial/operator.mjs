import { spawn } from "node:child_process";
import { once } from "node:events";
import { open, readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { loadContext } from "./context.mjs";
import { processSnapshot, sameProcess, requireGone } from "./host-resources.mjs";
import { processOwnership } from "../host-stalls/capture.mjs";
import { readJournal, baseline } from "./journal.mjs";
import { check, digest, proof, writeNew } from "./files.mjs";
import { nodeRuntimeEnvironment } from "./node-runtime.mjs";
const pause = (ms) => new Promise((done) => setTimeout(done, ms));
export const monotonicHostMs = () => Number(process.hrtime.bigint() / 1_000_000n);
async function request(root, path, input) {
  const server = (await proof(root, "server-owner.json")).value;
  check(server.origin === `http://127.0.0.1:${server.port}`, "noise_operator_origin");
  const response = await fetch(`${server.origin}${path}`, { method: "POST", headers: { Origin: server.origin, "Content-Type": "application/json" }, body: JSON.stringify(input) });
  const value = await response.json(); check(response.ok, "noise_operator_request_rejected"); return value;
}
/** Owned command/observer producer; no arbitrary program, arguments, port or manifest input. */
export async function observeCommand(root, context, mode, index, operations = {}) {
  const monotonicBegan = monotonicHostMs(), commandDeadline = monotonicBegan + 300000;
  const prefix = resolve(root, `install-${index}${mode === "detect" ? ".detect" : ""}`);
  const stdout = await open(`${prefix}.stdout.log`, "wx", 0o600), stderr = await open(`${prefix}.stderr.log`, "wx", 0o600);
  let child, ready, exited;
  try {
    child = (operations.spawn ?? spawn)(process.execPath, [operations.childProgram ?? fileURLToPath(new URL("./operator-child.mjs", import.meta.url)), root, mode, String(index)], {
      detached: true, stdio: ["ignore", stdout.fd, stderr.fd, "ipc"],
      env: { PATH: operations.path ?? process.env.PATH ?? "/usr/bin:/bin", HOME: homedir(), LANG: "C", LC_ALL: "C", RUST_BACKTRACE: "0",
        ...(process.env.TZ === undefined ? {} : { TZ: process.env.TZ }), ...nodeRuntimeEnvironment() },
    });
    let resolveReady, resolveExit, readyReceived = false;
    ready = new Promise((done) => { resolveReady = done; });
    exited = new Promise((done) => { resolveExit = done; });
    child.on("error", () => { if (!readyReceived) { readyReceived = true; resolveReady({ error: "noise_command_spawn_failed" }); } });
    child.on("message", (message) => { if (!readyReceived) { readyReceived = true; resolveReady({ message }); } });
    child.once("close", (code, signal) => {
      if (!readyReceived) { readyReceived = true; resolveReady({ error: "noise_command_early_close" }); }
      resolveExit([code, signal]);
    });
  } finally { await stdout.close(); await stderr.close(); }
  const began = Date.now();
  const bound = (promise, milliseconds) => new Promise((resolveBound, rejectBound) => {
    const timer = setTimeout(() => resolveBound(null), milliseconds);
    promise.then((value) => { clearTimeout(timer); resolveBound(value); }, (error) => { clearTimeout(timer); rejectBound(error); });
  });
  let observerFailed;
  const observerFailure = new Promise((done) => { observerFailed = done; });
  let known = [], seen = [], rootObserved = false, observations = 0, completed = false, failure = null;
  const observing = (async () => {
    while (!completed && monotonicHostMs() < commandDeadline) {
      const rows = await processSnapshot(), current = rows.find((row) => row.pid === child.pid && row.pgid === child.pid);
      if (!rootObserved) {
        check(current, "noise_command_owner_missing"); known = [current];
        await (operations.writeObserverProof ?? writeNew)(`${prefix}.host-root.json`, current);
        await (operations.writeObserverProof ?? writeNew)(`${prefix}.observer-armed.json`, { pid: current.pid, pgid: current.pgid, startedAt: current.startedAt });
        rootObserved = true;
      }
      const owned = processOwnership(rows, child.pid, known).owned;
      for (const row of owned) if (!seen.some((prior) => sameProcess(prior, row))) seen.push(row);
      known = owned; observations++; await pause(250);
    }
  })().catch((error) => { failure = error; observerFailed(null); });
  async function stopOwned() {
    if (child.exitCode !== null || child.signalCode !== null) { await bound(exited, 5000); return; }
    if (!child.pid) { check(await bound(exited, 5000), "noise_command_spawn_cleanup"); return; }
    try { process.kill(-child.pid, "SIGTERM"); } catch (error) { if (error.code !== "ESRCH") throw error; }
    const ending = monotonicHostMs() + 5000;
    while (child.exitCode === null && child.signalCode === null && monotonicHostMs() < ending - 1500) await pause(25);
    if (child.exitCode === null && child.signalCode === null) {
      try { process.kill(-child.pid, "SIGKILL"); } catch (error) { if (error.code !== "ESRCH") throw error; }
    }
    const result = await bound(exited, Math.max(0, ending - monotonicHostMs()));
    check(result, "noise_command_cleanup_pending");
  }
  try {
    const received = await bound(ready, Math.max(0, monotonicBegan + (operations.readyTimeoutMs ?? 5000) - monotonicHostMs()));
    check(received && !received.error, received?.error ?? "noise_command_ready_timeout");
    check(received.message?.ready === true && Object.keys(received.message).length === 1, "noise_command_ready_message");
    while (!rootObserved && !failure) await pause(10);
    if (failure) throw failure;
    let claimSha256 = null;
    if (mode === "flash") {
      const admitted = await request(root, "/install/claim", { index });
      check(admitted.install_claimed && admitted.index === index && admitted.program === "just" &&
        admitted.context_sha256 === digest(JSON.stringify(context)) && /^[a-f0-9]{64}$/u.test(admitted.claim_sha256), "noise_command_claim");
      claimSha256 = admitted.claim_sha256;
    }
    if (failure) throw failure;
    child.send({ kind: "execute", contextSha256: digest(JSON.stringify(context)),
      claimSha256 });
    const result = await bound(Promise.race([exited, observerFailure]), Math.max(0, commandDeadline - monotonicHostMs()));
    check(result && result[0] === 0 && result[1] === null, "noise_command_failed");
  } catch (error) { failure ??= error; await stopOwned(); }
  finally { completed = true; await observing; }
  const finalRows = await processSnapshot(), remaining = seen.filter((row) => finalRows.some((current) => sameProcess(row, current)));
  const observation = { schema: "hello-passive-command-observation-v1", rootObserved, observations,
    started_at_unix_ms: began, finished_at_unix_ms: Date.now(), seen, remaining,
    failures: failure ? [{ category: failure.code ?? "noise_command_failed" }] : [], observer_effects: "process-metadata-only" };
  await writeNew(`${prefix}.observation.json`, observation);
  if (mode === "flash") await writeNew(`${prefix}.exit.json`, { schema: "noise-serial-command-exit-v2", contextSha256: digest(JSON.stringify(context)), index,
    code: child.exitCode, ownerSha256: (await proof(root, `install-${index}.host-root.json`)).sha256,
    observationSha256: (await proof(root, `install-${index}.observation.json`)).sha256 });
  if (failure) throw failure;
  check(remaining.length === 0 && monotonicHostMs() - monotonicBegan <= 300000, "noise_command_cleanup");
}

/** Called only after the same Gate page has published its closed preserved baseline. */
export async function installCandidate(root, index, operations = {}) {
  check(Number.isInteger(index) && index >= 0 && index <= 4, "noise_install_index");
  const context = await loadContext(root, { operations });
  const rows = await readJournal(root, context); baseline(rows.at(-1)?.state, true);
  await observeCommand(root, context, "detect", index, operations);
  await observeCommand(root, context, "flash", index, operations);
  return request(root, "/install/review", { index });
}

/** Parent launcher helper: exit evidence comes from this actual owned child's close event. */
export function observeOwnedExit(child, contextSha256, owner, role) {
  check(role === "supervisor" && child.pid === owner.pid, "noise_exit_role");
  let stopRequestedAtMs = null;
  const finished = new Promise((done) => child.once("close", (code, signal) => done({ code, signal, at: monotonicHostMs() })));
  return {
    markStopRequested() { check(stopRequestedAtMs === null, "noise_duplicate_stop_request"); stopRequestedAtMs = monotonicHostMs(); },
    async receipt() {
      const value = await finished;
      check(value.signal === null && stopRequestedAtMs !== null && value.at >= stopRequestedAtMs && value.at - stopRequestedAtMs <= 5000, "noise_exit_timing");
      return { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner, code: value.code,
        observedAtUnixMs: Date.now(), clock: "node-hrtime-ms-v1", stopRequestedAtMs, exitedAtMs: value.at };
    },
  };
}

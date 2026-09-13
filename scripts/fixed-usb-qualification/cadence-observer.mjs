import { spawn as spawnChild } from "node:child_process";
import { lstat, open } from "node:fs/promises";
import { isIP } from "node:net";
import { resolve } from "node:path";
import { exactObject, fileDigest, protectedPath, QualificationError, requireCondition, writeNew } from "./contract.mjs";

const REASONS = new Set(["requested", "invalid_input", "invalid_frame", "output_failed", "deadline", "transport_failed",
  "counter_overflow", "peer_closed", "clock_invalid", "connect_failed", "input_timeout", "input_closed", "invalid_stop", "input_bound", "input_failed"]);
const integer = (value, max = Number.MAX_SAFE_INTEGER) => Number.isSafeInteger(value) && value >= 0 && value <= max;
function deferred() { let complete; const promise = new Promise(resolvePromise => { complete = resolvePromise; }); return { promise, complete }; }
async function wait(promise, milliseconds) {
  let timer;
  try { return await Promise.race([promise.then(() => true), new Promise(resolveWait => { timer = setTimeout(() => resolveWait(false), milliseconds); })]); }
  finally { clearTimeout(timer); }
}

function checkedEvent(value, previous, connected, terminal) {
  exactObject(value, ["schema", "event", "elapsedMs", "messageCount", "totalBytes", "byteCount", "reason"]);
  requireCondition(value.schema === "cpu0-cadence-observer-v1" && ["connected", "arrival", "closed", "error"].includes(value.event) &&
    integer(value.elapsedMs, 360000) && integer(value.messageCount, 2048) && integer(value.totalBytes, 2048 * 65536) &&
    integer(value.byteCount, 65536) && !terminal, "observer_event");
  const prior = previous ?? { elapsedMs: 0, messageCount: 0, totalBytes: 0 };
  requireCondition(value.elapsedMs >= prior.elapsedMs, "observer_clock");
  if (value.event === "connected") requireCondition(!connected && !previous && value.messageCount === 0 && value.totalBytes === 0 && value.byteCount === 0 && value.reason === null, "observer_connected");
  else if (value.event === "arrival") requireCondition(connected && value.reason === null && value.byteCount > 0 &&
    value.messageCount === prior.messageCount + 1 && value.totalBytes === prior.totalBytes + value.byteCount, "observer_arrival");
  else requireCondition(value.messageCount === prior.messageCount && value.totalBytes === prior.totalBytes && value.byteCount === 0 &&
    REASONS.has(value.reason) && (value.event !== "closed" || (connected && value.reason === "requested")), "observer_terminal");
  return value;
}

/** Owns one passive observer and exports only closed metadata; endpoint binding is checked by the supervisor. */
export function createCadenceObserver(root, context, { spawn = spawnChild, now = Date.now } = {}) {
  let used = false, child, journal, writeQueue = Promise.resolve(), pending = Buffer.alloc(0), bytes = 0, count = 0, journalBytes = 0, lastObserved = 0;
  let connected = false, alive = false, failed = null, terminal = null, previous = null, reaped = false, exitCode = null;
  let startedAtUnixMs = null, connectedAtUnixMs = null, closedAtUnixMs = null, maybeFinish, watchdog;
  const ready = deferred(), closed = deferred();
  const status = () => ({ connected, alive, failed: failed !== null, startedAtUnixMs, connectedAtUnixMs });
  const fail = reason => {
    failed ??= reason;
    ready.complete();
    if (child && !maybeFinish) queueMicrotask(() => { finish().catch(() => { failed ??= "observer_evidence"; }); });
  };
  function terminate(signal) {
    if (reaped || !child || child.exitCode !== null || child.signalCode !== null) return;
    try { if (!child.kill(signal)) failed ??= "observer_cleanup"; }
    catch { failed ??= "observer_cleanup"; }
  }
  function consume(chunk) {
    if (failed) { chunk.fill(0); return; }
    bytes += chunk.length;
    if (bytes > 1048576) { chunk.fill(0); fail("observer_output_bound"); return; }
    const combined = Buffer.concat([pending, chunk]); pending.fill(0); chunk.fill(0); pending = combined;
    while (pending.includes(10)) {
      const end = pending.indexOf(10);
      if (end > 4096 || count >= 2048) { pending.fill(0); pending = Buffer.alloc(0); fail("observer_output_bound"); return; }
      const line = pending.subarray(0, end), tail = Buffer.from(pending.subarray(end + 1));
      let event;
      try { event = checkedEvent(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(line)), previous, connected, terminal); }
      catch { pending.fill(0); pending = Buffer.alloc(0); tail.fill(0); fail("observer_event"); return; }
      pending.fill(0); pending = tail;
      previous = event; count += 1;
      const observedAtUnixMs = now();
      if (!integer(observedAtUnixMs) || (startedAtUnixMs !== null && observedAtUnixMs < startedAtUnixMs) || observedAtUnixMs < lastObserved) { fail("observer_clock"); return; }
      lastObserved = observedAtUnixMs;
      if (event.event === "connected") { connected = true; connectedAtUnixMs = observedAtUnixMs; }
      if (["closed", "error"].includes(event.event)) terminal = event;
      const row = JSON.stringify({ sequence: count, observedAtUnixMs, event }) + "\n";
      journalBytes += Buffer.byteLength(row);
      if (journalBytes > 1048576) { fail("observer_output_bound"); return; }
      writeQueue = writeQueue.then(() => journal.writeFile(row)).then(() => {
        if (event.event === "connected") ready.complete();
      }).catch(() => { fail("observer_evidence"); });
      if (event.event === "error") fail("observer_runtime");
    }
    if (pending.length > 4096) { pending.fill(0); pending = Buffer.alloc(0); fail("observer_output_bound"); }
  }
  async function finishImpl() {
    clearTimeout(watchdog);
    if (child && !reaped) {
      try { child.stdin.end('{"op":"stop"}\n'); } catch { failed ??= "observer_stdin"; }
      if (!await wait(closed.promise, 4000)) {
        failed ??= "observer_stop_timeout"; terminate("SIGTERM");
        if (!await wait(closed.promise, 1000)) { terminate("SIGKILL"); await wait(closed.promise, 1000); }
      }
    }
    if (child && !reaped) {
      failed ??= "observer_cleanup";
      child.stdin.destroy(); child.stdout.destroy(); child.stderr.destroy(); child.unref();
    }
    await writeQueue;
    if (journal) { try { await journal.sync(); await journal.close(); } catch { failed ??= "observer_evidence"; } }
    pending.fill(0); pending = Buffer.alloc(0);
    const passed = connected && reaped && exitCode === 0 && terminal?.event === "closed" && terminal.reason === "requested" && !failed;
    const result = { schema: "worker-cadence-observer-result-v1", connected, closed: reaped, exitCode,
      reason: passed ? "requested" : (failed ?? "observer_incomplete"), cleanupComplete: !child || reaped,
      startedAtUnixMs, connectedAtUnixMs, closedAtUnixMs, messageCount: previous?.messageCount ?? 0,
      totalBytes: previous?.totalBytes ?? 0, eventCount: count,
      journalSha256: journal ? await fileDigest(resolve(root, "cadence-observer.jsonl")) : null };
    if (journal) {
      try { await writeNew(resolve(root, "cadence-observer-result.json"), result); }
      catch { failed ??= "observer_evidence"; throw new QualificationError("observer_evidence"); }
    }
    return result;
  }
  function finish() { return maybeFinish ??= finishImpl(); }
  async function start(endpoint) {
    requireCondition(!used && !maybeFinish, "observer_already_used"); used = true;
    try {
      await protectedPath(root, true);
      exactObject(endpoint, ["ipv4", "httpPort"]);
      const octets = typeof endpoint.ipv4 === "string" ? endpoint.ipv4.split(".").map(Number) : [];
      requireCondition(isIP(endpoint.ipv4) === 4 && (octets[0] === 10 || (octets[0] === 172 && octets[1] >= 16 && octets[1] <= 31) ||
        (octets[0] === 192 && octets[1] === 168)) && integer(endpoint.httpPort, 65535) && endpoint.httpPort > 0, "observer_endpoint");
      const binary = context.cadence_observer;
      exactObject(binary, ["path", "sha256"]);
      const stat = await lstat(binary.path);
      requireCondition(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o111) !== 0 && await fileDigest(binary.path) === binary.sha256, "observer_binary");
      journal = await open(resolve(root, "cadence-observer.jsonl"), "wx", 0o600);
      startedAtUnixMs = now();
      child = spawn(binary.path, [], { stdio: ["pipe", "pipe", "pipe"], env: {} }); alive = true;
      child.stdout.on("data", consume);
      child.stderr.on("data", chunk => { chunk.fill(0); fail("observer_stderr"); });
      child.stdin.on("error", () => fail("observer_stdin"));
      child.stdout.on("error", () => fail("observer_stdout"));
      child.stderr.on("error", () => fail("observer_stderr"));
      child.once("error", () => fail("observer_spawn"));
      child.once("close", code => {
        alive = false; reaped = true; exitCode = Number.isInteger(code) ? code : null; closedAtUnixMs = now();
        if (pending.length || code !== 0 || !terminal || terminal.event !== "closed") fail("observer_exit");
        clearTimeout(watchdog);
        closed.complete(); ready.complete();
      });
      watchdog = setTimeout(() => { fail("observer_deadline"); terminate("SIGKILL"); }, 360000);
      const handoff = Buffer.from(JSON.stringify({ schema: "cpu0-cadence-observer-input-v1", ipv4: endpoint.ipv4,
        port: endpoint.httpPort, observedUnixMs: startedAtUnixMs, expiresUnixMs: startedAtUnixMs + 5000, bindingVerified: true }) + "\n");
      try { child.stdin.write(handoff, () => handoff.fill(0)); }
      catch { handoff.fill(0); throw new QualificationError("observer_stdin"); }
      if (!await wait(ready.promise, 10000)) fail("observer_connect_timeout");
      if (!connected || failed || !alive) { await finish(); throw new QualificationError(failed ?? "observer_connect"); }
      return { observer_connected: true };
    } catch (error) {
      failed ??= error instanceof QualificationError ? error.code : "observer_start";
      await finish();
      throw new QualificationError(failed);
    }
  }
  return { start, finish, status };
}

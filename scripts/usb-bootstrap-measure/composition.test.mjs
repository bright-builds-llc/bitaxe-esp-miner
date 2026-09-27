import assert from "node:assert/strict";
import test from "node:test";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, open, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fixture, writeTestSeam, testOperations } from "./test-fixture.mjs";
import { contextHash } from "./context.mjs";
import { schema, sha256 } from "./values.mjs";
import { exchange } from "../str005-v2-serial/operator-ipc.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { finalize, review } from "./finalize.mjs";
import { inspectCapture } from "./install.mjs";
import { judgeOperator, readStoppedOperator } from "./operator-disposition.mjs";
import { deriveCleanup } from "./cleanup.mjs";
const pause = ms => new Promise(done => setTimeout(done, ms));
async function bound(promise, ms) { let timer; try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(Error("test deadline")), ms); })]); } finally { clearTimeout(timer); } }
async function until(read) { const end = Date.now() + 30000; while (Date.now() < end) { const value = await read(); if (value) return value; await pause(50); } throw Error("test timed out"); }

for (const mode of ["healthy", "failed", "missing"]) test(`real daemon, production install and independent ${mode} measurement review`, { timeout: 90000 }, async t => {
  // Arrange: only hardware/native/publication prerequisites are synthetic.
  const f = await fixture(t); await writeTestSeam(f, mode); const hash = contextHash(f.context), dir = `${f.root}.operator`;
  await mkdir(dir, { mode: 0o700 }); await writeNew(resolve(dir, "start.claim.json"), { schema: schema("operator-start"), contextSha256: hash });
  const out = await open(resolve(dir, "daemon.stdout.log"), "wx", 0o600), err = await open(resolve(dir, "daemon.stderr.log"), "wx", 0o600);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./composition-daemon.mjs", import.meta.url)), f.root], { detached: true, stdio: ["ignore", out.fd, err.fd, "ipc"] });
  // Node's parent-initiated IPC disconnect can leave close counters unbalanced.
  // Actual exit plus the finalizer's kernel absence is authoritative for this daemon.
  const exited = new Promise(done => child.once("exit", (code, signal) => done([code, signal])));
  await out.close(); await err.close();
  t.after(async () => { if (child.exitCode === null && child.signalCode === null) { child.kill("SIGTERM"); try { await bound(exited, 10000); } catch { child.kill("SIGKILL"); await bound(exited, 5000); } } });
  const ready = once(child, "message"); child.send({ schema: schema("operator-bootstrap") }); await bound(ready, 15000); child.disconnect();
  const locator = (await proof(dir, "locator.json")).value;
  const query = maybeRequestId => exchange(locator.socketPath, { schema: schema("operator-query"), contextSha256: hash, maybeRequestId });
  const status = await until(async () => { const value = await query(null); if (value.phase === "failed") throw Error(`daemon failed ${await readFile(resolve(dir, "supervisor.stderr.log"), "utf8")}`); return value.phase === "ready" ? value : null; });
  const post = async (path, value) => {
    const response = await fetch(status.maybeSupervisorOrigin + path, { method: "POST", headers: { Origin: status.maybeSupervisorOrigin, "Content-Type": "application/json" }, body: JSON.stringify(value), signal: AbortSignal.timeout(5000) });
    const result = await response.json(); assert.equal(response.status, 200, JSON.stringify(result)); return result;
  };
  const forbidden = await fetch(status.maybeSupervisorOrigin + "/sign", { method: "POST", headers: { Origin: status.maybeSupervisorOrigin, "Content-Type": "application/json" }, body: "{}" });
  assert.equal(forbidden.status, 404); await forbidden.json();
  const before = f.state(); await post("/record", { state: before }); await post("/accounting", { stage: "before", ledger: f.context.expectedAccounting.ledger, original: f.context.expectedAccounting.original, state: before });
  await post("/record", { state: f.state("before", true) });
  async function request(id, action, payload) {
    const value = { schema: schema("operator-request"), contextSha256: hash, requestId: id.repeat(32), action, payload };
    await exchange(locator.socketPath, value);
    const result = await until(async () => { const result = await query(value.requestId); return result.status === "pending" ? null : result; });
    assert.equal(result.status, "succeeded", JSON.stringify(result)); return result;
  }
  // Act: native effects are replaced by one fixed synthetic child, not a duplicate install implementation.
  await request("1", "install", { index: 0 });
  const after = f.state("candidate"); await post("/record", { state: after }); await post("/accounting", { stage: "after", ledger: f.context.expectedAccounting.ledger, original: f.context.expectedAccounting.original, state: after });
  await post("/record", { state: f.state("candidate", true) });
  const rows = await (await import("./journal.mjs")).readJournal(f.root, f.context), last = rows.at(-1);
  const browserWitness = { schema: schema("browser-closure"), source: "native-ui-observer", contextSha256: hash, closed: true,
    lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: Date.now() };
  await request("2", "finish-cleanup", { browserWitness });
  await exchange(locator.socketPath, { schema: schema("operator-request"), contextSha256: hash, requestId: "3".repeat(32), action: "stop", payload: {} });
  const [code, signal] = await bound(exited, 10000); assert.equal(code, 0); assert.equal(signal, null);
  const ops = await testOperations(f.root);
  if (mode === "healthy") {
    // Independent readers must reject internally changed synthetic evidence before sealing.
    for (const [name, change, inspect] of [
      ["install-0.claim.json", value => { value.beforeStateSha256 = "f".repeat(64); }, () => inspectCapture(f.root, f.context)],
      ["install-0.claim.json", value => { value.argv.push("--factory-reset"); }, () => inspectCapture(f.root, f.context)],
    ]) { const path = resolve(f.root, name), bytes = await readFile(path), value = JSON.parse(bytes); change(value); await writeFile(path, JSON.stringify(value)); await assert.rejects(inspect()); await writeFile(path, bytes); }
    const locatorPath = resolve(dir, "locator.json"), locatorBytes = await readFile(locatorPath);
    for (const change of [v => { v.extra = "private"; }, v => { v.schema = "other-task"; }, v => { v.contextSha256 = "f".repeat(64); }, v => { v.socketPath = "/tmp/unbound/socket"; }]) {
      const value = JSON.parse(locatorBytes); change(value); await writeFile(locatorPath, JSON.stringify(value));
      await assert.rejects(readStoppedOperator(f.root, f.context, { directory: dir, checkKernel: false })); await writeFile(locatorPath, locatorBytes);
    }
    const childExitPath = resolve(dir, "supervisor-close.json"), childExitBytes = await readFile(childExitPath);
    for (const change of [v => { v.extra = "private"; }, v => { v.source = "caller-supplied"; }, v => { v.contextSha256 = "f".repeat(64); }]) {
      const value = JSON.parse(childExitBytes); change(value); await writeFile(childExitPath, JSON.stringify(value));
      assert.equal((await deriveCleanup(f.root, f.context, ops)).complete, false); await writeFile(childExitPath, childExitBytes);
    }
    const phasePath = resolve(dir, "phase-install-0.json"), phaseBytes = await readFile(phasePath), phase = JSON.parse(phaseBytes); phase.requestSha256 = "f".repeat(64);
    await writeFile(phasePath, JSON.stringify(phase)); await assert.rejects(judgeOperator(f.root, f.context, dir)); await writeFile(phasePath, phaseBytes);
    const exitPath = resolve(f.root, "parent-cleanup-supervisor-observation.json"), exitBytes = await readFile(exitPath), changed = JSON.parse(exitBytes); changed.exitedAtMs = changed.stopRequestedAtMs + 5001;
    await writeFile(exitPath, JSON.stringify(changed)); assert.equal((await deriveCleanup(f.root, f.context, ops)).complete, false); await writeFile(exitPath, exitBytes);
  }
  const result = await finalize(f.root, ops), saved = (await proof(f.root, "final-result.json")).value;
  // Assert: failed ordinary capture is never promoted; missing timing remains unverified.
  assert.equal(result.status, mode === "missing" ? "unverified" : "measurement_complete", JSON.stringify(saved));
  assert.equal(saved.capture.exitCode, mode === "failed" ? 1 : 0); assert.equal(saved.capture.qualified, mode !== "failed");
  if (mode === "failed") assert.equal(saved.firstFailure.code, "bootstrap_capture_unqualified");
  assert.deepEqual(await review(f.root, ops), result); assert.equal(result.hardware_qualified, false);
});

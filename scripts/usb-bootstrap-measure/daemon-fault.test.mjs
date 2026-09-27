import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, open, lstat } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fixture, writeTestSeam, testOperations } from "./test-fixture.mjs";
import { contextHash } from "./context.mjs";
import { schema, sha256 } from "./values.mjs";
import { exchange } from "../str005-v2-serial/operator-ipc.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireGone } from "../str005-noise-serial/host-resources.mjs";
import { requireHostStopped } from "./cleanup.mjs";
import { judgeOperator } from "./operator-disposition.mjs";
const pause = ms => new Promise(done => setTimeout(done, ms));
async function bound(promise, ms) { let timer; try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(Error("fault_test_timeout")), ms); })]); } finally { clearTimeout(timer); } }
async function until(read) { const end = Date.now() + 30000; while (Date.now() < end) { const value = await read(); if (value) return value; await pause(30); } throw Error("fault_test_timeout"); }
for (const fault of ["supervisor-receipt", "stop-result"]) test(`actual daemon exits and releases resources when ${fault} publication fails`, { timeout: 90000 }, async t => {
  // Arrange: real daemon/supervisor, synthetic native inputs; no device operation is requested.
  const f = await fixture(t); await writeTestSeam(f); const hash = contextHash(f.context), directory = `${f.root}.operator`;
  await mkdir(directory, { mode: 0o700 }); await writeNew(resolve(directory, "start.claim.json"), { schema: schema("operator-start"), contextSha256: hash });
  const out = await open(resolve(directory, "daemon.stdout.log"), "wx", 0o600), err = await open(resolve(directory, "daemon.stderr.log"), "wx", 0o600);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./composition-daemon.mjs", import.meta.url)), f.root], { detached: true, stdio: ["ignore", out.fd, err.fd, "ipc"] });
  const exited = once(child, "exit"), ready = once(child, "message"); await out.close(); await err.close();
  t.after(async () => { if (child.exitCode === null && child.signalCode === null) { child.kill("SIGTERM"); try { await bound(exited, 10000); } catch { child.kill("SIGKILL"); await exited; } } });
  child.send({ schema: schema("operator-bootstrap") }); await bound(ready, 10000); child.disconnect();
  const locator = (await proof(directory, "locator.json")).value;
  const query = requestId => exchange(locator.socketPath, { schema: schema("operator-query"), contextSha256: hash, maybeRequestId: requestId });
  const status = await until(async () => { const value = await query(null); assert.notEqual(value.phase, "failed"); return value.phase === "ready" ? value : null; });
  const state = f.state("before", true);
  const response = await fetch(`${status.maybeSupervisorOrigin}/record`, { method: "POST", headers: { Origin: status.maybeSupervisorOrigin, "Content-Type": "application/json" }, body: JSON.stringify({ state }) });
  assert.equal(response.status, 200); await response.json();
  const last = (await (await import("./journal.mjs")).readJournal(f.root, f.context)).at(-1);
  const witness = { schema: schema("browser-closure"), source: "native-ui-observer", contextSha256: hash, closed: true, lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: Date.now() };
  if (fault === "supervisor-receipt") await mkdir(resolve(directory, "supervisor-close.json"), { mode: 0o700 });
  const request = (id, action, payload) => ({ schema: schema("operator-request"), contextSha256: hash, requestId: id.repeat(32), action, payload });
  // Act: filesystem failures are real exclusive-write failures, not fake writer return values.
  await exchange(locator.socketPath, request("1", "finish-cleanup", { browserWitness: witness }));
  await until(async () => { const value = await query("1".repeat(32)); return value.status !== "pending" ? value : null; });
  if (fault === "stop-result") await mkdir(resolve(directory, `result-${"2".repeat(32)}.pending.json`), { mode: 0o700 });
  await exchange(locator.socketPath, request("2", "stop", {}));
  const [code, signal] = await bound(exited, 10000);
  // Assert: failure is explicit, no invented successful disposition, kernel ownership is absent.
  assert.equal(code, 1); assert.equal(signal, null);
  await requireGone([locator.owner]); await requireHostStopped(f.root, f.context, await testOperations(f.root));
  await assert.rejects(lstat(locator.socketPath), { code: "ENOENT" });
  const stopped = (await proof(directory, "stopped.json")).value, disposition = (await proof(directory, "disposition.json")).value;
  assert.notEqual(stopped.maybeCode, null); assert.equal(disposition.hostStopped, true); assert.equal(disposition.cleanupRecorded, false);
  await assert.rejects(judgeOperator(f.root, f.context, directory));
  if (fault === "stop-result") await assert.rejects(proof(directory, `result-${"2".repeat(32)}.json`), { code: "ENOENT" });
});

test("locator publication failure releases the pre-ack listener and exits naturally", { timeout: 90000 }, async t => {
  // Arrange: fail the exclusive locator write after the real Unix listener is created.
  const f = await fixture(t); await writeTestSeam(f); const directory = `${f.root}.operator`;
  await mkdir(directory, { mode: 0o700 }); await mkdir(resolve(directory, "locator.json"), { mode: 0o700 });
  const out = await open(resolve(directory, "daemon.stdout.log"), "wx", 0o600), err = await open(resolve(directory, "daemon.stderr.log"), "wx", 0o600);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./composition-daemon.mjs", import.meta.url)), f.root], { detached: true, stdio: ["ignore", out.fd, err.fd, "ipc"] });
  const exited = once(child, "exit"); await out.close(); await err.close();
  t.after(async () => { if (child.exitCode === null && child.signalCode === null) { child.kill("SIGKILL"); await exited; } });
  let acknowledged = false; child.on("message", () => { acknowledged = true; });
  // Act
  child.send({ schema: schema("operator-bootstrap") }); const [code, signal] = await bound(exited, 10000);
  // Assert: no supervisor was launched and no missing receipt is invented.
  assert.equal(code, 1); assert.equal(signal, null); assert.equal(acknowledged, false);
  const { processSnapshot } = await import("../str005-noise-serial/host-resources.mjs");
  assert.equal((await processSnapshot()).some(row => row.pid === child.pid || row.ppid === child.pid || row.pgid === child.pid), false);
  for (const file of ["supervisor-start.json", "stopped.json", "disposition.json"]) await assert.rejects(lstat(resolve(directory, file)), { code: "ENOENT" });
});

import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { request as httpRequest } from "node:http";
import { open, readFile, mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { networkInterfaces } from "node:os";
import { randomBytes } from "node:crypto";
import { contextFixture } from "./context-fixtures.mjs";
import { syntheticDevice } from "./cleanup-rehearsal-device.mjs";
import { realHostOperations, bound, wait } from "./cleanup-rehearsal-runtime.mjs";
import { selectInterface } from "../str005-noise-serial/fixture-owner.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireGone, processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { exchange } from "./operator-ipc.mjs";
import { readJournal } from "./journal.mjs";
import { finalize, review } from "./finalize.mjs";
import { sha256 } from "./values.mjs";
const METHODS = new Set(["bootstrapPins", "currentBootstrapOwnership", "readNoiseAnchorProof", "inspectNative", "inspectPredecessor", "inspectPermissionClosure", "inspectChannelSuccessor", "checkCurrentSuccessorOwnership", "inspectShareSuccessor", "checkCurrentShareSuccessorOwnership"]);
async function eventually(operation, milliseconds = 30000) {
  const deadline = Date.now() + milliseconds;
  while (true) { const result = await operation(); if (result) return result; assert.ok(Date.now() < deadline, "composition deadline"); await wait(30); }
}
function address() {
  const interfaces = networkInterfaces();
  for (const rows of Object.values(interfaces)) for (const row of rows ?? []) {
    if (row.family !== "IPv4" || row.internal) continue;
    try { return selectInterface(row.address, interfaces).address; } catch { /* Inspect next real interface. */ }
  }
  throw Error("rehearsal_private_interface_unavailable");
}
for (const mode of ["pass", "daemon-loss", "daemon-loss-body"]) test(`actual durable daemon and managed supervisor composition: ${mode}`, { timeout: 150000, skip: process.platform !== "darwin" }, async t => {
  const previous = process.env.TMPDIR; process.env.TMPDIR = "/tmp";
  t.after(() => { if (previous === undefined) delete process.env.TMPDIR; else process.env.TMPDIR = previous; });
  const cleanups = [], f = await contextFixture({ after: fn => cleanups.push(fn) }, {
    maybeFixtureBytes: await readFile(fileURLToPath(new URL("./cleanup-rehearsal-fixture.mjs", import.meta.url))),
  });
  const digest = sha256(JSON.stringify(f.context)), directory = `${f.root}.operator`;
  await writeNew(resolve(f.root, "synthetic-rehearsal.json"), { schema: "str005-v2-synthetic-rehearsal-v1", source: "test-only", contextSha256: digest, root: f.root, hardwareEffects: false });
  await writeNew(resolve(f.previous.root, "install-0.claim.json"), { detector: { physical: "c".repeat(64) } });
  await writeNew(resolve(f.previous.root, "sealed-inventory.json"), { files: [{ path: "install-0.claim.json", sha256: (await proof(f.previous.root, "install-0.claim.json")).sha256 }] });
  await mkdir(directory, { mode: 0o700 }); await writeNew(resolve(directory, "start.claim.json"), { schema: "str005-v2-operator-start-v1", contextSha256: digest });
  const output = await open(resolve(directory, "test.stdout"), "wx", 0o600), errors = await open(resolve(directory, "test.stderr"), "wx", 0o600);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./operator-composition-daemon.mjs", import.meta.url)), f.root],
    { detached: true, stdio: ["ignore", output.fd, errors.fd, "ipc"], env: { ...process.env, TMPDIR: "/tmp" } });
  const closed = new Promise(done => child.once("close", (code, signal) => done({ code, signal })));
  const initialized = new Promise((done, reject) => {
    child.on("message", message => {
      if (message.schema === "str005-v2-operator-status-v1" && message.phase === "initializing") done(message);
    });
    child.once("error", reject);
    child.once("close", () => reject(Error("rehearsal_daemon_closed_before_initialization")));
  });
  child.on("message", async message => {
    if (message.kind !== "prerequisite") return;
    if (!METHODS.has(message.method)) throw Error("rehearsal_prerequisite_invalid");
    try { const value = await f.operations[message.method](...message.args, f.operations); if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, value }); }
    catch (error) { if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, error: error.code ?? "v2_rehearsal_failed" }); }
  });
  await output.close(); await errors.close();
  t.after(async () => {
    try {
      if (child.exitCode === null && child.signalCode === null) { child.kill("SIGTERM"); try { await bound(closed, 10000); } catch { child.kill("SIGKILL"); await bound(closed, 3000); } }
    } finally {
      for (const name of ["server-owner.json", "fixture-owner.json"]) {
        let maybeOwner;
        try { maybeOwner = (await proof(f.root, name)).value.owner; } catch (error) { if (error.code !== "ENOENT") throw error; }
        if (maybeOwner && (await processSnapshot()).some(row => sameProcess(row, maybeOwner))) {
          process.kill(-maybeOwner.pgid, "SIGTERM");
          await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, maybeOwner)), 10000);
        }
      }
      for (const cleanup of cleanups) await cleanup();
    }
  });
  // The real acknowledgment is emitted only after locator writeNew has closed.
  // Existence polling can observe a freshly created, not-yet-written JSON file.
  const acknowledgment = await bound(initialized, 90000);
  assert.equal(acknowledgment.contextSha256, digest);
  const locator = (await proof(directory, "locator.json")).value;
  const query = { schema: "str005-v2-operator-query-v1", contextSha256: digest, maybeRequestId: null };
  await eventually(async () => {
    const status = await exchange(locator.socketPath, query);
    if (status.phase === "failed") {
      const logs = [];
      for (const name of ["supervisor.stdout.log", "supervisor.stderr.log", "test.stderr"]) {
        try { logs.push(await readFile(resolve(directory, name), "utf8")); } catch (error) { if (error.code !== "ENOENT") throw error; }
      }
      assert.fail(JSON.stringify(status) + logs.join("\n"));
    }
    return status.phase === "ready";
  });
  const server = (await proof(f.root, "server-owner.json")).value;
  const request = async (path, input, method = "POST") => {
    const response = await fetch(`${server.origin}${path}`, { method, signal: AbortSignal.timeout(15000), headers: { Origin: server.origin, "Content-Type": "application/json" }, ...(method === "POST" ? { body: JSON.stringify(input) } : {}) });
    const value = await response.json(); if (!response.ok) throw Object.assign(Error(value.error), { code: value.error }); return value;
  };
  async function action(action, payload = {}) {
    const requestId = randomBytes(16).toString("hex");
    await exchange(locator.socketPath, { schema: "str005-v2-operator-request-v1", contextSha256: digest, requestId, action, payload });
    if (action === "stop") return;
    await eventually(async () => { const result = await exchange(locator.socketPath, { ...query, maybeRequestId: requestId }); assert.notEqual(result.status, "failed", JSON.stringify(result)); return result.status === "succeeded"; });
  }
  const device = syntheticDevice(f.context, request, address());
  await device.configured(); await device.ready(); await device.coordinator.supervisor.recordAccounting("before-install"); await device.close();
  for (const index of f.context.install_indices) {
    await action("install", { index }); if (index === 0) await device.coordinator.supervisor.configureCandidate(); await device.ready();
    if (index > 0) await device.coordinator.supervisor.recordCycle(index); if (index < 4) await device.close();
  }
  await device.coordinator.supervisor.run();
  if (mode.startsWith("daemon-loss")) {
    let maybeRequest, maybePending;
    if (mode === "daemon-loss-body") {
      maybePending = new Promise((done, reject) => {
        maybeRequest = httpRequest(`${server.origin}/activate`, { method: "POST", headers: { Origin: server.origin, "Content-Length": 2, "Content-Type": "application/json" } }, response => { response.resume(); response.once("end", () => done(response.statusCode)); });
        maybeRequest.on("error", reject); maybeRequest.setTimeout(5000, () => maybeRequest.destroy(Error("rehearsal_request_timeout")));
        maybeRequest.write("{");
      });
      maybePending.catch(() => undefined);
      await wait(100); // The complete JSON body is deliberately unavailable before loss.
    }
    const fixtureOwner = (await proof(f.root, "fixture-owner.json")).value.owner;
    child.kill("SIGKILL"); assert.deepEqual(await bound(closed, 5000), { code: null, signal: "SIGKILL" });
    if (maybeRequest) { maybeRequest.end("}"); await assert.rejects(maybePending, "post-await dispatch must reject parent loss"); }
    else await assert.rejects(request("/activate", {}), "lost parent must close every new effect route");
    await eventually(async () => {
      try { await requireGone([server.owner, fixtureOwner]); return true; } catch { return false; }
    }, 7000);
    await assert.rejects(proof(f.root, "parent-cleanup-supervisor.json"), { code: "ENOENT" });
    return;
  }
  await device.reconnect(); await device.coordinator.supervisor.restoreAndRecord();
  await action("prepare-cleanup"); await device.close();
  const last = (await readJournal(f.root, f.context)).at(-1);
  // The UI/device observations are explicitly synthetic; host observation is real.
  await action("finish-cleanup", { browserWitness: { schema: "noise-serial-browser-closure-v3", source: "native-ui-observer", contextSha256: digest,
    closed: true, lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: Date.now() } });
  await action("stop"); assert.deepEqual(await bound(closed, 10000), { code: 0, signal: null });
  await requireGone([locator.owner, server.owner, (await proof(f.root, "fixture-owner.json")).value.owner]);
  const operations = realHostOperations(f.operations);
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, operations);
  assert.equal(result.status, "passed"); assert.deepEqual(await review(f.root, operations), result);
  assert.equal((await proof(f.root, "parent-cleanup-supervisor.json")).value.code, 0);
  assert.equal((await proof(directory, "disposition.json")).value.cleanupRecorded, true);
});

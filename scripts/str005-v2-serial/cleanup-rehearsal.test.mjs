import assert from "node:assert/strict";
import test from "node:test";
import { readFile, readdir, writeFile, access, mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { networkInterfaces } from "node:os";
import { createServer } from "node:net";
import { contextFixture } from "./context-fixtures.mjs";
import { selectInterface } from "../str005-noise-serial/fixture-owner.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { readJournal } from "./journal.mjs";
import { createCleanupSession } from "./cleanup-session.mjs";
import { inspectCleanup } from "./cleanup.mjs";
import { finalize, review } from "./finalize.mjs";
import { sha256 } from "./values.mjs";
import { requirePoolListenerAbsent, requireGone } from "./host-resources.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { supervisor, realHostOperations, bound, wait } from "./cleanup-rehearsal-runtime.mjs";
import { syntheticDevice } from "./cleanup-rehearsal-device.mjs";
import { simulatedInstall } from "./cleanup-rehearsal-inputs.mjs";

function localAddress() {
  const interfaces = networkInterfaces();
  for (const rows of Object.values(interfaces)) for (const row of rows ?? []) {
    if (row.family !== "IPv4" || row.internal) continue;
    try { return selectInterface(row.address, interfaces).address; } catch { /* Try a different real host interface. */ }
  }
  throw Error("rehearsal_private_interface_unavailable");
}
async function setup(t) {
  const cleanup = [], f = await contextFixture({ after: operation => cleanup.push(operation) }, {
    maybeFixtureBytes: await readFile(fileURLToPath(new URL("./cleanup-rehearsal-fixture.mjs", import.meta.url))),
  });
  let maybeHost;
  t.after(async () => {
    try { if (maybeHost) await maybeHost.stop(); }
    finally { for (const operation of cleanup) await operation(); }
  });
  await writeNew(resolve(f.root, "synthetic-rehearsal.json"), { schema: "str005-v2-synthetic-rehearsal-v1", source: "test-only",
    contextSha256: sha256(JSON.stringify(f.context)), root: f.root, hardwareEffects: false });
  await writeNew(resolve(f.previous.root, "install-0.claim.json"), { detector: { physical: "c".repeat(64) } });
  await writeNew(resolve(f.previous.root, "sealed-inventory.json"), { files: [{ path: "install-0.claim.json",
    sha256: (await proof(f.previous.root, "install-0.claim.json")).sha256 }] });
  const cold = { ...f.operations }, host = await supervisor(f); maybeHost = host;
  const operations = realHostOperations(cold), session = createCleanupSession(f.root, f.context, { child: host.child, owner: host.owner, operations });
  const address = localAddress(), device = syntheticDevice(f.context, host.request, address);
  await device.configured(); await device.ready();
  await device.coordinator.supervisor.recordAccounting("before-install"); await device.close();
  for (const index of f.context.install_indices) {
    await simulatedInstall(f, index); await host.request("/install/review", { index });
    if (index === 0) await device.coordinator.supervisor.configureCandidate();
    await device.ready();
    if (index > 0) await device.coordinator.supervisor.recordCycle(index);
    if (index < 4) await device.close();
  }
  return { ...f, host, operations, device, session, address };
}
async function browserWitness(f) {
  const last = (await readJournal(f.root, f.context)).at(-1);
  // Explicitly synthetic browser observation. Journal/source joins are production;
  // no claim is made that a physical browser or device participated in this test.
  return { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256: sha256(JSON.stringify(f.context)),
    closed: true, lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: Date.now() };
}
async function runtimeEvidence(root) {
  const output = [];
  async function walk(directory) {
    for (const name of await readdir(directory, { withFileTypes: true })) {
      if (["evaluator", "qualified-artifacts", "native", "fixture", "observer"].includes(name.name)) continue;
      const path = resolve(directory, name.name);
      if (name.isDirectory()) await walk(path); else if (name.name.endsWith(".json") || name.name.endsWith(".log")) output.push(await readFile(path, "utf8"));
    }
  }
  await walk(root); return output.join("\n");
}

async function exchanged(f) {
  try { await f.device.coordinator.supervisor.run(); }
  catch (error) { throw Error(f.host.lastFailure() ?? error.message); }
  await f.device.reconnect(); await f.device.coordinator.supervisor.restoreAndRecord();
}
async function absent(path) { await assert.rejects(access(path), { code: "ENOENT" }); }
async function ownersGone(owners) {
  const deadline = Date.now() + 7000;
  for (;;) {
    try { await requireGone(owners); return; }
    catch (error) { if (Date.now() >= deadline) throw error; await wait(25); }
  }
}

test("real host lifecycle in UTC retains the private cleanup capability through finalization and review", { skip: process.platform !== "darwin", timeout: 150000 }, async t => {
  // Arrange: force the canonical runner timezone even in a local non-UTC shell.
  // The actual parent/child ps observations must still identify the same owner.
  const maybeTimezone = process.env.TZ; process.env.TZ = "UTC";
  t.after(() => { if (maybeTimezone === undefined) delete process.env.TZ; else process.env.TZ = maybeTimezone; });
  // Only device/browser/flash/source inputs are synthetic; IPC/HTTP/TCP and ps/lsof are real.
  const f = await setup(t);
  const initial = await readJournal(f.root, f.context);
  assert.equal(initial[0].state.status, "configured"); assert.equal(initial[1].state.serialOwnershipReleased, false);
  assert((await proof(f.root, "accounting-before-install.json")).value.observedSequence > 2);
  // Act
  await exchanged(f);
  assert.deepEqual(await f.session.prepare(), { cleanup_prepared: true });
  await f.device.close();
  const witness = await browserWitness(f), recorded = await f.session.finish(witness);
  assert(recorded.cleanup_recorded && recorded.device_resources_released && recorded.device_baseline_confirmed);
  assert.equal(f.host.outputBytes(), 0);
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  const replay = await review(f.root, f.operations);
  // Assert: these accepted model inputs test host composition, never hardware parity.
  assert.equal(result.status, "passed"); assert.deepEqual(replay, result);
  const cleanup = await inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations);
  assert(cleanup.value.poolListenerAbsent); assert.equal(cleanup.value.supervisorExitCode, 0); assert.equal(cleanup.value.fixtureExitCode, 0);
  const text = await runtimeEvidence(f.root), cleanupText = await runtimeEvidence(`${f.root}.cleanup`);
  assert(!text.includes(f.address)); assert(!text.includes('"fixturePort"')); assert(!text.includes('"localPort"'));
  assert(!cleanupText.includes(String(f.device.privatePort())));
  assert.equal(requirePoolListenerAbsent(f.device.privatePort()), undefined);
});

test("rehearsal entrypoints cannot target a real repository qualification root", async () => {
  // Arrange / Act / Assert: rejection happens before reading device or authority input.
  await assert.rejects(syntheticRoot(resolve("scratch/str005-v2-serial/channel-003")), { code: "v2_rehearsal_root_rejected" });
});


for (const adverse of ["listener_remains", "missing_browser", "changed_fixture_identity"]) {
  test(`real cleanup preserves the first failure: ${adverse}`, { skip: process.platform !== "darwin", timeout: 30000 }, async t => {
    // Arrange: identical production composition; only the named adverse input changes.
    const f = await setup(t); await exchanged(f);
    if (adverse === "changed_fixture_identity") {
      const path = resolve(f.root, "fixture-ready.json"), value = JSON.parse(await readFile(path));
      value.instanceId = Buffer.alloc(16, 9).toString("base64url"); await writeFile(path, `${JSON.stringify(value)}\n`);
      await assert.rejects(f.session.prepare(), { code: "v2_cleanup_instance_changed" });
    } else await f.session.prepare();
    await f.device.close();
    let maybeListener;
    if (adverse === "listener_remains") {
      maybeListener = createServer();
      await new Promise((done, reject) => { maybeListener.once("error", reject); maybeListener.listen(f.device.privatePort(), f.address, done); });
      t.after(() => new Promise(done => maybeListener.close(done)));
    }
    // Act: stop/reap still happens, but no passing cleanup receipt may be created.
    const witness = adverse === "missing_browser" ? null : await browserWitness(f);
    await assert.rejects(f.session.finish(witness));
    const failure = (await proof(f.root, "parent-cleanup-failure.json")).value;
    assert.equal(failure.stage, adverse === "changed_fixture_identity" ? "prepare" : adverse === "missing_browser" ? "browser_witness" : "record");
    if (adverse === "listener_remains") assert.equal(failure.code, "v2_pool_listener_present");
    if (adverse === "changed_fixture_identity") assert.equal(failure.code, "v2_cleanup_instance_changed");
    // Assert: the one-shot failed session cannot be upgraded by another witness.
    await assert.rejects(f.session.finish(await browserWitness(f)), { code: failure.code });
    assert.deepEqual((await proof(f.root, "parent-cleanup-failure.json")).value, failure);
    await absent(`${f.root}.cleanup/receipt.json`);
    assert.equal((await proof(f.root, "parent-cleanup-supervisor.json")).value.code, 0);
    assert.equal((await f.host.closed).code, 0);
    await ownersGone([f.host.owner, (await proof(f.root, "fixture-owner.json")).value.owner]);
  });
}

for (const interruption of ["parent_ipc_disconnect", "supervisor_killed"]) {
  test(`real fixture listener exits after ${interruption}`, { skip: process.platform !== "darwin", timeout: 30000 }, async t => {
    // Arrange: a real listening fixture has not received synthetic device protocol input.
    const f = await setup(t);
    await f.device.coordinator.supervisor.recordAccounting("before");
    const status = await f.device.gate.stratumV2Status("channel", null);
    await f.host.request("/fixture/start", { status });
    const privateContext = await f.host.request("/cleanup/private-context", {});
    const fixtureOwner = (await proof(f.root, "fixture-owner.json")).value.owner;
    assert.throws(() => requirePoolListenerAbsent(privateContext.fixturePort), { code: "v2_pool_listener_present" });
    // Act: emulate the test-runner death route and the supervisor's abrupt death route.
    if (interruption === "parent_ipc_disconnect") f.host.child.disconnect(); else f.host.child.kill("SIGKILL");
    await bound(f.host.exited, 7000);
    // Assert: real ps/lsof, without replacing or fabricating any collector result.
    await ownersGone([f.host.owner, fixtureOwner]);
    assert.equal(requirePoolListenerAbsent(privateContext.fixturePort), undefined);
    await absent(`${f.root}.cleanup/receipt.json`);
  });
}


test("failure receipt write errors cannot prevent owned host cleanup", { skip: process.platform !== "darwin", timeout: 30000 }, async t => {
  // Arrange: real filesystem obstruction, not a mocked writer success/failure.
  const f = await setup(t); await exchanged(f); await f.session.prepare(); await f.device.close();
  await mkdir(resolve(f.root, "parent-cleanup-failure.json"));
  // Act / Assert: missing witness fails and durable evidence remains unproved, but owners exit.
  await assert.rejects(f.session.finish(null), { code: "v2_parent_cleanup_failed" });
  assert.equal((await proof(f.root, "parent-cleanup-supervisor.json")).value.code, 0);
  await ownersGone([f.host.owner, (await proof(f.root, "fixture-owner.json")).value.owner]);
  await absent(`${f.root}.cleanup/receipt.json`);
});

test("timed-out supervisor cleanup reaps both proven owners without a passing receipt", { skip: process.platform !== "darwin", timeout: 30000 }, async t => {
  // Arrange: both real owners are live; the supervisor cannot process graceful stop.
  const f = await setup(t); await f.device.coordinator.supervisor.recordAccounting("before");
  await f.host.request("/fixture/start", { status: await f.device.gate.stratumV2Status("channel", null) });
  const privateContext = await f.host.request("/cleanup/private-context", {});
  const fixtureOwner = (await proof(f.root, "fixture-owner.json")).value.owner;
  await f.session.prepare(); await f.device.close();
  f.host.child.kill("SIGSTOP");
  // Act: the production helper observes its original five-second failure, then
  // performs identity-checked host cleanup. No successful exit is synthesized.
  await assert.rejects(f.session.finish(await browserWitness(f)), { code: "v2_parent_supervisor_timeout" });
  // Assert
  const failure = (await proof(f.root, "parent-cleanup-failure.json")).value;
  assert.equal(failure.stage, "supervisor_stop"); assert.equal(failure.code, "v2_parent_supervisor_timeout");
  await ownersGone([f.host.owner, fixtureOwner]);
  assert.equal(requirePoolListenerAbsent(privateContext.fixturePort), undefined);
  await absent(`${f.root}.cleanup/receipt.json`);
  await absent(resolve(f.root, "parent-cleanup-supervisor.json"));
});

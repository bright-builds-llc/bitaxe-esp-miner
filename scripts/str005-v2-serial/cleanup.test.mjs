import assert from "node:assert/strict";
import { cp, mkdir, readFile, readdir, writeFile, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { verifyCleanupInputs, verifyEffectInputs } from "./context.mjs";
import { contextFixture } from "./context-fixtures.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { createJournal, readJournal } from "./journal.mjs";
import { inspectCleanup, prepareCleanup, requireHostStopped } from "./cleanup.mjs";
import { sha256 } from "./values.mjs";

async function setup(t, scope = "channel") {
  const f = await contextFixture(t, { scope }), contextSha256 = sha256(JSON.stringify(f.context));
  const server = { pid: 11111, pgid: 11111, startedAt: "synthetic-server" }, fixture = { pid: 22222, pgid: 22222, startedAt: "synthetic-fixture" };
  const instance = Buffer.alloc(16, 2).toString("base64url"), supervisorPort = 49319, privatePort = 54321;
  await writeNew(resolve(f.root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256, owner: server,
    origin: `http://127.0.0.1:${supervisorPort}`, port: supervisorPort, atHostMs: 1 });
  await writeNew(resolve(f.root, "fixture-owner.json"), { schema: "str005-v2-fixture-owner-v1", contextSha256, owner: fixture,
    binarySha256: f.context.fixture_sha256, atHostMs: 1 });
  await writeNew(resolve(f.root, "fixture-ready.json"), { schema: "str005-v2-fixture-ready-facts-v1", contextSha256,
    scope, attemptId: f.context.attemptId, instanceId: instance, authorityPublicKeySha256: "a".repeat(64), owner: fixture, readyAtMs: 2 });
  await writeNew(resolve(f.root, "fixture-exit.json"), { schema: "str005-v2-fixture-exit-v1", contextSha256,
    owner: fixture, code: 0, signal: null, atHostMs: 500, stderrBytes: 0, lifetimeMs: 498 });
  await writeNew(resolve(f.root, "fixture-reap.json"), { schema: "str005-v2-fixture-reap-v1", contextSha256,
    kind: "natural_exit", requestedAtHostMs: null, completedAtHostMs: 501, durationMs: null });
  for (const index of [scope === "channel" ? 0 : 1, 4]) await writeNew(resolve(f.root, `install-${index}.claim.json`), { contextSha256, index, detector: { port: `/dev/cu.synthetic${index}` } });
  const journal = await createJournal(f.root, f.context);
  await journal.state("candidate", state(f.context, "candidate", true), 600);
  const device = channelFixture().deviceRecords.at(-1); device.attemptId = f.context.attemptId;
  if (scope === "share") { device.scope = "share"; device.authorityDeadlineDeviceUs = 180001000; device.observationDeadlineDeviceUs = null; }
  await writeNew(resolve(f.root, "device-0001.json"), { schema: "str005-v2-device-record-v1", contextSha256, sequence: 1, atHostMs: 550, record: device });
  let live = true; const calls = [];
  const runtime = { schema: "str005-v2-cleanup-runtime-v1", contextSha256, scope, attemptId: f.context.attemptId, fixtureInstanceId: instance, fixturePort: privatePort };
  const operations = { ...f.operations,
    processSnapshot: async () => live ? [server] : [],
    fetch: async (url, options) => { calls.push({ url, options }); return new Response(JSON.stringify(runtime), { status: 200 }); },
    spawnSync: (_command, args) => { calls.push({ args }); return { status: 0, signal: null, stdout: "p33333\nn*:8080\n", stderr: "" }; },
    execFileSync: (_command, args) => { calls.push({ args }); throw Object.assign(new Error("synthetic absence"), { status: 1, signal: null, stdout: "", stderr: "" }); },
  };
  async function witnesses() {
    const last = (await readJournal(f.root, f.context)).at(-1), now = Date.now();
    return { browser: { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
      lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: now },
    supervisor: { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner: server,
      code: 0, observedAtUnixMs: now, clock: "node-hrtime-ms-v1", stopRequestedAtMs: 1000, exitedAtMs: 1100 } };
  }
  return { ...f, operations, calls, runtime, privatePort, witnesses, close: () => { live = false; } };
}

test("private cleanup preparation records actual derived absence without retaining pool port", async (t) => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations);
  assert.deepEqual(Object.keys(prepared), ["record"]);
  assert.equal(f.calls[0].options.redirect, "error"); assert.equal(f.calls[0].options.headers.Origin, "http://127.0.0.1:49319");
  f.close(); await prepared.record(await f.witnesses());
  const result = await inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations);
  assert.equal(result.value.poolListenerAbsent, true); assert.equal(result.cleanupMs, 100);
  const text = (await Promise.all((await readdir(`${f.root}.cleanup`)).map((name) => readFile(resolve(`${f.root}.cleanup`, name), "utf8")))).join("\n");
  assert(!/\b54321\b/u.test(text)); assert(!/"(?:poolPort|fixturePort|localPort|remotePort)"/u.test(text));
  assert(!/\b54321\b/u.test(JSON.stringify(f.calls)));
  assert(f.calls.some((call) => call.args?.includes("/dev/tty.synthetic0"))); assert(f.calls.some((call) => call.args?.includes("/dev/tty.synthetic4")));
  await assert.rejects(prepared.record(await f.witnesses()), { code: "v2_cleanup_already_recorded" });
});

test("wrong source instance and unavailable live supervisor cannot provide private cleanup context", async (t) => {
  const f = await setup(t); f.runtime.fixtureInstanceId = Buffer.alloc(16, 3).toString("base64url");
  await assert.rejects(prepareCleanup(f.root, f.context, f.operations), { code: "v2_cleanup_instance_changed" });
  f.close(); await assert.rejects(prepareCleanup(f.root, f.context, f.operations), { code: "v2_cleanup_server_not_live" });
});

test("resource still listening prevents evidence creation and port never enters error", async (t) => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations); f.close();
  f.operations.spawnSync = () => ({ status: 0, signal: null, stdout: "p123\nn*:54321\n", stderr: "" });
  await assert.rejects(prepared.record(await f.witnesses()), (error) => error.code === "v2_pool_listener_present" && !error.message.includes("54321"));
  await assert.rejects(readFile(`${f.root}.cleanup/receipt.json`), { code: "ENOENT" });
});

test("actual host cleanup cannot turn missing worker release into positive cleanup", async (t) => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations); f.close();
  const file = resolve(f.root, "device-0001.json"), record = JSON.parse(await readFile(file));
  record.record.resources.workerQuiescent = false; record.record.resources.workerQuiescentAtUs = null; record.record.resources.fenceRetained = true;
  record.record.state = "running"; record.record.outcome = null; record.record.terminalAtDeviceUs = null;
  await writeFile(file, JSON.stringify(record));
  const observed = await prepared.record(await f.witnesses()); assert.equal(observed.device_resources_released, false);
  await assert.rejects(inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, { ...f.operations, checkKernel: false }), { code: "v2_cleanup_incomplete" });
});

test("final cleanup snapshot remains valid while changed live witness is rejected", async (t) => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations); f.close(); await prepared.record(await f.witnesses());
  await mkdir(resolve(f.root, "final-inputs"), { mode: 0o700 });
  await cp(`${f.root}.cleanup`, resolve(f.root, "final-inputs/cleanup"), { recursive: true });
  await inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, { ...f.operations, checkKernel: false, cleanupSnapshot: true });
  const resource = (await proof(`${f.root}.cleanup`, "resources.json")).value; resource.fixtureReadySha256 = "0".repeat(64);
  await writeFile(`${f.root}.cleanup/resources.json`, JSON.stringify(resource));
  await assert.rejects(inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations), { code: "v2_cleanup_witness_changed" });
});

test("unobserved supervisor exit, unreleased process and modified readiness fail closed", async (t) => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations);
  await assert.rejects(requireHostStopped(f.root, f.context, f.operations));
  f.close(); const witness = await f.witnesses(); witness.supervisor.exitedAtMs = 7000;
  await assert.rejects(prepared.record(witness), { code: "v2_supervisor_exit_timing" });
  const ready = (await proof(f.root, "fixture-ready.json")).value; ready.readyAtMs++;
  await writeFile(resolve(f.root, "fixture-ready.json"), JSON.stringify(ready));
  await assert.rejects(prepared.record(await f.witnesses()), { code: "v2_cleanup_source_changed" });
});

test("private cleanup response is bounded and never follows a redirect", async (t) => {
  const f = await setup(t);
  for (const response of [new Response("", { status: 302, headers: { Location: "http://127.0.0.1:1/" } }),
    new Response("x".repeat(4097)), new Response(JSON.stringify({ ...f.runtime, unexpected: "synthetic-private" }))]) {
    f.operations.fetch = async (_url, options) => { assert.equal(options.redirect, "error"); return response; };
    await assert.rejects(prepareCleanup(f.root, f.context, f.operations), { code: "v2_cleanup_private_context_unproved" });
  }
  await assert.rejects(readFile(`${f.root}.cleanup/receipt.json`), { code: "ENOENT" });
});

test("signer closure proofs are bound without retaining secret process output", async (t) => {
  const f = await setup(t, "share"), prepared = await prepareCleanup(f.root, f.context, f.operations); f.close();
  await writeNew(resolve(f.root, "signer-01.exit.json"), { schema: "str005-v2-signer-exit-v1", contextSha256: sha256(JSON.stringify(f.context)), index: 1,
    operation: "public-trust", observation: { pid: 44444, code: 0, signal: null, elapsedMs: 20, stdoutBytes: 123, stderrBytes: 0, overflow: false, inputFailed: false } });
  await prepared.record(await f.witnesses());
  const resource = (await proof(`${f.root}.cleanup`, "resources.json")).value;
  assert.deepEqual(resource.signerExits, [{ path: "signer-01.exit.json", sha256: (await proof(f.root, "signer-01.exit.json")).sha256 }]);
  const changed = (await proof(f.root, "signer-01.exit.json")).value; changed.observation.code = null;
  await writeFile(resolve(f.root, "signer-01.exit.json"), JSON.stringify(changed));
  await assert.rejects(inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations), { code: "v2_signer_close_unproved" });
});

test("actual failed-child closure remains distinct from successful positive cleanup", async t => {
  const f = await setup(t), prepared = await prepareCleanup(f.root, f.context, f.operations); f.close();
  const exit = (await proof(f.root, "fixture-exit.json")).value; exit.code = 1;
  await writeFile(resolve(f.root, "fixture-exit.json"), JSON.stringify(exit));
  await prepared.record(await f.witnesses());
  await assert.rejects(inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations), { code: "v2_cleanup_exit" });
  const actual = await inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, { ...f.operations, requireSuccessfulExit: false });
  assert.equal(actual.value.fixtureExitCode, 1); assert.equal(actual.value.remainingOwnedProcesses, 0);
});

test("independent cleanup rejects a readiness observation outside the actual owner launch bound", async t => {
  const f = await setup(t), ready = (await proof(f.root, "fixture-ready.json")).value;
  ready.readyAtMs = 5002; await writeFile(resolve(f.root, "fixture-ready.json"), JSON.stringify(ready));
  const prepared = await prepareCleanup(f.root, f.context, f.operations); f.close();
  await prepared.record(await f.witnesses());
  await assert.rejects(inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations), { code: "v2_fixture_readiness_deadline" });
});


for (const name of ["failure.json", "parent-cleanup-failure.json"]) {
  test(`cleanup after ${name} can finish while device effects remain forbidden`, async t => {
    // Arrange: preserve the primary observation; cleanup proves resources only.
    const f = await setup(t, "share"), path = resolve(f.root, name), failure = `${JSON.stringify({ synthetic: "preserved failure" })}\n`;
    await f.put(path, failure);
    await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "private_path_exists" });
    // Act.
    const prepared = await prepareCleanup(f.root, f.context, f.operations);
    f.close(); await prepared.record(await f.witnesses());
    const result = await inspectCleanup(f.root, f.context, `${f.root}.cleanup/receipt.json`, f.operations);
    // Assert.
    assert.equal(result.value.poolListenerAbsent, true);
    assert.equal(await readFile(path, "utf8"), failure);
    await assert.rejects(verifyEffectInputs(f.context, f.operations), { code: "private_path_exists" });
  });
}

test("cleanup verification never reopens finalized or sealed results", async t => {
  // Arrange.
  const f = await contextFixture(t);
  // Act / Assert.
  for (const name of ["final-result.json", "sealed-inventory.json", "failed-inventory.json"]) {
    await f.put(resolve(f.root, name), "{}\n");
    await assert.rejects(verifyCleanupInputs(f.context, f.operations), { code: "private_path_exists" });
    await unlink(resolve(f.root, name));
  }
});

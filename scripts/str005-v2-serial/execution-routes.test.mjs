import assert from "node:assert/strict";
import { mkdir, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { preparedFixture } from "./completed-fixture.mjs";
import { createExecutionRoutes, expectedHeartbeatLoss } from "./execution-routes.mjs";
import { inspectExecution } from "./execution-inspect.mjs";
import { readDeviceJournal, readJournal } from "./journal.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { parseStatus } from "./device.mjs";
import { check, PROFILE, sha256 } from "./values.mjs";

async function routesFixture(t) {
  const f = await preparedFixture(t), vector = channelFixture(), hash = sha256(JSON.stringify(f.context));
  for (const record of vector.deviceRecords) record.attemptId = f.context.attemptId;
  const [admitted, final] = vector.deviceRecords;
  const socket = { localIpv4: "192.168.20.4", localPort: 42123, remoteIpv4: "192.168.20.3", remotePort: 33123 };
  const connection = { observedAtUs: 3000, bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: 3,
    poolSessionGeneration: 4, poolTransportEpoch: 5, socket };
  const status = (record = null, fresh = false) => parseStatus({ schema: "worker-stratum-v2-status-v1", scope: "channel", state: record?.state ?? "idle",
    observation: { bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: fresh ? 4 : 3, observedAtUs: record?.observedAtUs ?? 999,
      clockValid: true, stationIpv4: socket.localIpv4, wifiConnected: true, socket: null },
    connection: record?.events.some(event => event.kind === "connected") ? connection : null, record });
  const ready = { schema: "str005-v2-fixture-ready-runtime-v1", scope: "channel", attemptId: f.context.attemptId,
    instanceId: vector.fixtureTerminal.instanceId, listenIpv4: socket.remoteIpv4, listenPort: socket.remotePort, authorityPublicKey: Buffer.from(Array.from({ length: 32 }, (_, index) => index + 51)).toString("base64url") };
  const readyAtMs = ++f.time;
  await writeNew(resolve(f.root, "fixture-ready.json"), { schema: "str005-v2-fixture-ready-facts-v1", contextSha256: hash,
    scope: "channel", attemptId: f.context.attemptId, instanceId: ready.instanceId, authorityPublicKeySha256: "a".repeat(64), owner: {}, readyAtMs });
  await mkdir(resolve(f.root, "fixture-run"), { mode: 0o700 });
  for (const [file, value] of Object.entries({ "job.json": vector.job, "fixture-events.json": vector.fixtureEvents,
    "fixture-terminal.json": vector.fixtureTerminal, "connection-facts.json": vector.fixtureConnectionFacts, "shares.json": vector.fixtureShares }))
    await writeNew(resolve(f.root, "fixture-run", file), value);
  let failed = false, finished = 0;
  const owner = { ready, stratum: { profile: PROFILE, endpoint: `stratum+tcp://${socket.remoteIpv4}:${socket.remotePort}/`,
    authorityPublicKey: ready.authorityPublicKey, userIdentity: "synthetic-worker" },
    alive() { check(!failed, "synthetic_fixture_failed"); }, validateStation(value) { assert.equal(value, socket.localIpv4); },
    requireStartWindow() { check(f.time - readyAtMs <= 10000, "v2_fixture_start_deadline"); },
    async connection() { return { schema: "str005-v2-fixture-connection-runtime-v1", scope: "channel", attemptId: f.context.attemptId,
      instanceId: ready.instanceId, connectionId: vector.job.connectionId, observedAtFixtureUs: 1,
      localIpv4: socket.remoteIpv4, localPort: socket.remotePort, peerIpv4: socket.localIpv4, peerPort: socket.localPort }; },
    async finish() {
      finished++;
      await writeNew(resolve(f.root, "fixture-exit.json"), { atHostMs: ++f.time });
      await writeNew(resolve(f.root, "fixture-reap.json"), { completedAtHostMs: ++f.time });
    },
  };
  const routes = createExecutionRoutes({ root: f.root, context: f.context, contextSha256: hash, now: () => ++f.time,
    ready: () => check(!failed, "synthetic_failed"), verify: async () => {}, journal: f.journal, failures: { failed: () => failed },
    observer: {}, fixture: () => owner, phase: () => "candidate" });
  return { ...f, next: () => ++f.time, routes, status, admitted, final, vector, fail: () => { failed = true; }, finished: () => finished };
}

test("actual execution routes consume one Channel start and emit independently joined cleanup receipts", async t => {
  // Arrange: actual continuity/journal producers plus explicit synthetic native/fixture observations.
  const f = await routesFixture(t), call = (path, input) => f.routes.handle(path, input);
  // Act.
  const start = await call("/start/claim", { status: f.status() });
  assert.equal(start.attemptId, f.context.attemptId);
  await assert.rejects(call("/start/claim", { status: f.status() }), { code: "v2_channel_start_consumed" });
  await f.journal.device(f.status(f.admitted), f.next());
  await f.journal.device(f.status(f.final), f.next());
  await call("/protocol/connection", { status: f.status(f.final) });
  await call("/protocol/complete", {});
  await f.journal.state("candidate", f.state("candidate", true), f.next());
  await f.journal.state("candidate", f.state(), f.next());
  await f.journal.device(f.status(f.final, true), f.next());
  await call("/restoration", { status: f.status(f.final, true), state: f.state() });
  const inspected = await inspectExecution(f.root, f.context, await readJournal(f.root, f.context), await readDeviceJournal(f.root, f.context));
  // Assert.
  assert.equal(inspected.protocolInput.deviceRecords.length, 3);
  assert.equal(f.finished(), 1);
  const serialized = (await Promise.all(["start.claim.json", "connection.json", "job-receipt.json", "protocol-complete.json", "restoration.json"].map(async name => (await proof(f.root, name)).bytes.toString()))).join("\n");
  for (const privateValue of ["192.168.20.", "33123", "42123", "synthetic-worker", start.stratum.authorityPublicKey]) assert.equal(serialized.includes(privateValue), false);
});

test("unrecorded native facts and unfinished resources never produce restoration receipts", async t => {
  // Arrange.
  const f = await routesFixture(t);
  // Act / Assert.
  await assert.rejects(f.routes.handle("/protocol/connection", { status: f.status(f.final) }), { code: "v2_execution_unrecorded_device" });
  await f.journal.device(f.status(f.admitted), f.next());
  await assert.rejects(f.routes.handle("/restoration", { status: f.status(f.admitted), state: f.state() }), { code: "v2_restoration_resource" });
  assert.equal((await readdir(f.root)).includes("restoration.json"), false);
  assert.equal(f.routes.expectedFault({ heartbeatSuppressed: true, running: false, serialFailureCategory: "liveness_lost" }), false);
});

test("failed contexts cannot claim Channel execution even when all cycles exist", async t => {
  // Arrange.
  const f = await routesFixture(t); f.fail();
  // Act / Assert.
  await assert.rejects(f.routes.handle("/start/claim", { status: f.status() }), { code: "synthetic_failed" });
  assert.equal((await readdir(f.root)).includes("start.claim.json"), false);
});


test("expected heartbeat loss does not conceal parser, close, generation or unrelated owner failures", () => {
  // Arrange.
  const state = { heartbeatSuppressed: true, running: false, qualification: { generation: 4 },
    serialFailureCategory: "liveness_lost", failure: "window_control_failed" };
  // Act / Assert.
  assert.equal(expectedHeartbeatLoss(state, 4, true), true);
  assert.equal(expectedHeartbeatLoss(state, 4, false), false);
  for (const patch of [{ failure: "close_failed" }, { failure: "probe_failed" }, { ownerResourceFailure: "stack" }, { serialFailureCategory: "wire_bound" },
    { serialFailureCategory: "timeout" }, { running: true }, { heartbeatSuppressed: false }, { qualification: { generation: 5 } }])
    assert.equal(expectedHeartbeatLoss({ ...state, ...patch }, 4, true), false);
});

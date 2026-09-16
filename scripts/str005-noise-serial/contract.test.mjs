import assert from "node:assert/strict";
import test from "node:test";
import { parseStart } from "./contract.mjs";
import { parseNoiseStatus, validateNoiseProgress } from "./device.mjs";
import { parseFixtureReady, parseFixtureTerminal } from "./fixture.mjs";
import { inspectExchangeShape, parseCleanupReceipt, parseProjection } from "./evidence.mjs";
import { admitted, cleanup, example, projection } from "./fixtures.mjs";

test("matching closed protocol metadata never confers hardware qualification", () => {
  assert.deepEqual(inspectExchangeShape(example()), { schema: "noise-serial-shape-check-v1", protocol_consistent: true, hardware_qualified: false });
});
for (const [name, mutate, code] of [
  ["attempt", (v) => { v.fixtureTerminal.attemptId = "B".repeat(21) + "A"; }, "noise_attempt_join"],
  ["socket", (v) => { v.fixtureTerminal.candidates[0].remotePort++; }, "noise_connection_join"],
  ["authority", (v) => { v.fixtureReady.authorityPublicKey = "B".repeat(42) + "A"; }, "noise_fixture_binding"],
  ["input digest", (v) => { v.device.job.inputSha256 = "b".repeat(64); }, "noise_attempt_join"],
  ["peer EOF", (v) => { v.fixtureTerminal.peerClosed = false; }, "noise_fixture_positive_shape"],
  ["extra bytes", (v) => { v.fixtureTerminal.extraBytesReceived = 1; }, "noise_fixture_positive_shape"],
  ["extra candidate", (v) => { v.fixtureTerminal.expectedPeerConnectionCount = 2; }, "noise_fixture_positive_shape"],
]) test(`protocol consistency rejects contradictory ${name}`, () => {
  const value = example(); mutate(value);
  assert.throws(() => inspectExchangeShape(value), { code });
});
for (const [name, mutate] of [
  ["unknown field", (v) => { v.raw = "secret"; }],
  ["boot mismatch", (v) => { v.observation.bootOrdinal++; }],
  ["observation preceding terminal", (v) => { v.observation.observedAtUs = 9999; }],
  ["missing timing", (v) => { v.job.stages[0].durationUs = null; }],
  ["bad join timing", (v) => { v.job.stages[7].durationUs = 1000; }],
  ["late authority", (v) => { v.job.terminal.decidedAtUs = 120001001; v.observation.observedAtUs = 120001001; }],
  ["duplicate stage", (v) => { v.job.stages[1].stage = v.job.stages[0].stage; }],
  ["act two mismatch", (v) => { v.job.stages[3].bytes = 233; }],
  ["false release", (v) => { v.job.resources.workerState = "running"; }],
]) test(`closed device parser rejects ${name}`, () => {
  const value = example().device; mutate(value);
  assert.throws(() => parseNoiseStatus(value));
});
test("connected but missing address remains observable, not a valid Start address", () => {
  const value = admitted(); value.observation.stationIpv4 = null;
  assert.equal(parseNoiseStatus(value).observation.stationIpv4, null);
  const start = example().start; start.fixtureIpv4 = null;
  assert.throws(() => parseStart(start));
});
test("an active job cannot silently move to a fresh session", () => {
  const value = admitted(); value.observation.transportEpoch++;
  assert.throws(() => parseNoiseStatus(value), { code: "noise_live_binding" });
});
test("retained terminal permits fresh possession without rewriting original epoch", () => {
  const value = example().device; value.observation.transportEpoch++;
  assert.equal(parseNoiseStatus(value).job.transportEpoch, 2);
});
test("history permits append-only progress and rejects rewriting completed operations", () => {
  const before = admitted(), after = example().device;
  validateNoiseProgress(before, after);
  const changed = structuredClone(after); changed.job.stages[0].durationUs = 400;
  assert.throws(() => validateNoiseProgress(after, changed), { code: "noise_stage_changed" });
});
test("history never changes an observed release timestamp", () => {
  const before = example().device, after = structuredClone(before);
  after.job.resources.startedAtUs--; after.job.resources.deadlineAtUs--;
  after.job.stages[7].durationUs++;
  assert.throws(() => validateNoiseProgress(before, after));
});
test("unknown or secret-bearing fixture fields are rejected", () => {
  const ready = example().fixtureReady; ready.authorityPrivateKey = "never-persist";
  assert.throws(() => parseFixtureReady(ready));
  const terminal = example().fixtureTerminal; terminal.raw = "never-persist";
  assert.throws(() => parseFixtureTerminal(terminal));
});
test("cleanup parsing preserves failed exits without treating shape as cleanup proof", () => {
  const value = cleanup(); value.fixtureExitCode = 1;
  assert.equal(parseCleanupReceipt(value).fixtureExitCode, 1);
  value.witnesses.path = "/arbitrary";
  assert.throws(() => parseCleanupReceipt(value));
});
test("projection shape alone is parseable but private fields and wrong bounds fail", () => {
  assert.deepEqual(parseProjection(projection()), projection());
  const secret = projection(); secret.endpoint = "private";
  assert.throws(() => parseProjection(secret));
  const late = projection(); late.timings_ms.diagnostic = 120001;
  assert.throws(() => parseProjection(late));
  const work = projection(); work.counts.new_work = 1;
  assert.throws(() => parseProjection(work));
});

function incomplete() {
  const value = admitted();
  value.state = "terminal"; value.observation.observedAtUs = 7_000_000;
  value.job.firstFailure = { stage: "noise_prepared", category: "authority_lost", detail: "timeout", atUs: 3000 };
  value.job.terminal = { outcome: "incomplete", decidedAtUs: 5_005_001 };
  value.job.resources = { socketState: "open", workerState: "running", volatileInputsDisposed: false,
    startedAtUs: 5000, deadlineAtUs: 5_005_000, releasedAtUs: null, deadlineMet: false,
    failure: { stage: "cleanup", category: "cleanup", detail: "resource_unreleased", atUs: 5_005_001 } };
  return value;
}
test("late resource release fills evidence but never promotes the incomplete terminal", () => {
  const before = incomplete(), after = structuredClone(before);
  after.job.resources.socketState = "closed"; after.job.resources.workerState = "quiescent";
  after.job.resources.volatileInputsDisposed = true; after.job.resources.releasedAtUs = 6_000_500;
  after.job.stages = [
    { stage: "socket_closed", sequence: 1, atUs: 6_000_000, durationUs: 500, bytes: null },
    { stage: "worker_quiescent", sequence: 2, atUs: 6_000_500, durationUs: 5_995_500, bytes: null },
  ];
  validateNoiseProgress(before, after);
  assert.equal(parseNoiseStatus(after).job.terminal.outcome, "incomplete");
  after.job.resources.deadlineMet = true;
  assert.throws(() => validateNoiseProgress(before, after));
});
test("history cannot create owners or protocol progress after a terminal failure", () => {
  const before = incomplete(); before.job.resources.socketState = "not_created";
  const after = structuredClone(before); after.job.resources.socketState = "open";
  assert.throws(() => validateNoiseProgress(before, after), { code: "noise_owner_after_terminal" });
  const protocol = incomplete(); protocol.job.stages = [{ stage: "noise_prepared", sequence: 1, atUs: 4000, durationUs: 100, bytes: null }];
  assert.throws(() => parseNoiseStatus(protocol), { code: "noise_protocol_after_failure" });
});
test("cleanup state and first failure are immutable across repeated observations", () => {
  const before = incomplete(), after = structuredClone(before);
  after.job.firstFailure.atUs = 2999;
  assert.throws(() => validateNoiseProgress(before, after), { code: "noise_first_failure_changed" });
  const clean = example().device, regressed = structuredClone(clean);
  regressed.job.resources.volatileInputsDisposed = false;
  assert.throws(() => validateNoiseProgress(clean, regressed));
});
test("transport timeout category cannot invent absolute authority expiry semantics", () => {
  const value = example().device;
  value.job.stages = [];
  value.job.firstFailure = { stage: "tcp_connected", category: "connect", detail: "timeout", atUs: 2000 };
  value.job.terminal.outcome = "rejected";
  parseNoiseStatus(value);
  value.job.terminal.outcome = "expired";
  parseNoiseStatus(value);
  value.job.firstFailure.category = "authority_lost";
  value.job.terminal.outcome = "rejected";
  assert.throws(() => parseNoiseStatus(value), { code: "noise_terminal_cause" });
});

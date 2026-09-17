// Synthetic failed Share after a genuinely re-evaluated synthetic Channel.
// No proof-of-work predicate is bypassed and this never represents hardware evidence.
import { resolve } from "node:path";
import { preflight, loadContext } from "./context.mjs";
import { preparedFixture, qualification } from "./completed-fixture.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { saveAccounting } from "./journal.mjs";
import { prepareCleanup } from "./cleanup.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { sha256 } from "./values.mjs";

export async function failedShareFixture(t, channel, channelResult) {
  const options = { ...channel.options, scope: "share", privateRoot: resolve(channel.parent, "share-001"),
    predecessorReceipt: resolve(channel.root, "final-result.json") };
  delete options.supersedePermission;
  const predecessor = { root: channel.root, context: channel.context, resultSha256: channelResult.result_sha256,
    sealSha256: channelResult.sealed_inventory_sha256, ledger, original };
  const oldReader = channel.operations.inspectPredecessor;
  const operations = { ...channel.operations, inspectPredecessor: async (path, scope, ops) =>
    scope === "share" ? predecessor : oldReader(path, scope, ops) };
  await preflight(options, operations);
  const context = await loadContext(options.privateRoot, { operations });
  const f = await preparedFixture(t, { ...channel, root: options.privateRoot, options, operations, context });
  const { root, journal } = f, contextSha256 = sha256(JSON.stringify(context));
  const record = structuredClone(channelFixture().deviceRecords[0]);
  Object.assign(record, { scope: "share", attemptId: context.attemptId, authorityDeadlineDeviceUs: null, observationDeadlineDeviceUs: null });
  await writeNew(resolve(root, "device-0001.json"), { schema: "str005-v2-device-record-v1", contextSha256, sequence: 1, atHostMs: ++f.time, record });
  const failed = { ...structuredClone(record), observedAtUs: 2000, state: "terminal", outcome: "rejected", terminalAtDeviceUs: 1900,
    firstFailure: { stage: "authenticated", category: "authentication", atDeviceUs: 1800 },
    resources: { socketClosed: true, workerQuiescent: true, fenceRetained: false, socketClosedAtUs: 1900, workerQuiescentAtUs: 2000 } };
  await writeNew(resolve(root, "device-0002.json"), { schema: "str005-v2-device-record-v1", contextSha256, sequence: 2, atHostMs: ++f.time, record: failed });
  await writeNew(resolve(root, "failure.json"), { schema: "str005-v2-first-failure-v1", contextSha256, code: "v2_device_failure",
    atHostMs: ++f.time, deviceCause: failed.firstFailure, sourceSequence: 2 });
  const instanceId = Buffer.alloc(16, 5).toString("base64url"), fixtureOwner = { pid: 82001, pgid: 82001, startedAt: "synthetic-failed-fixture" };
  const serverOwner = { pid: 82000, pgid: 82000, startedAt: "synthetic-share-server" };
  await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256, owner: serverOwner,
    origin: "http://127.0.0.1:32124", port: 32124, atHostMs: 0 });
  await writeNew(resolve(root, "fixture-owner.json"), { schema: "str005-v2-fixture-owner-v1", contextSha256, owner: fixtureOwner,
    binarySha256: context.fixture_sha256, atHostMs: 1 });
  await writeNew(resolve(root, "fixture-ready.json"), { schema: "str005-v2-fixture-ready-facts-v1", contextSha256, scope: "share", attemptId: context.attemptId,
    instanceId, authorityPublicKeySha256: "a".repeat(64), owner: fixtureOwner, readyAtMs: 2 });
  await f.put(resolve(root, "fixture-run/fixture-terminal.json"), JSON.stringify({ ...channelFixture().fixtureTerminal,
    instanceId, outcome: "unverified", firstFailure: { stage: "setup_received", category: "eof", atFixtureUs: 1000 } }));
  await writeNew(resolve(root, "fixture-exit.json"), { schema: "str005-v2-fixture-exit-v1", contextSha256, code: 1, signal: null,
    atHostMs: f.time, owner: fixtureOwner, stderrBytes: 0, lifetimeMs: f.time - 2 });
  await writeNew(resolve(root, "fixture-reap.json"), { schema: "str005-v2-fixture-reap-v1", contextSha256,
    kind: "natural_exit", requestedAtHostMs: null, completedAtHostMs: ++f.time, durationMs: null });
  const q = { ...qualification(), generation: record.workerGeneration, budget_reserved_ms: 240000, active_limit_ms: 180000,
    attempt: { schema: "worker-qualification-observation-v1", ordinal: 18, purpose: "normal", maximum_active_ms: 180000,
      reserved_ms: 180000, complete: true, active_ms: 0 } };
  f.state = (closed = false) => {
    const value = state(context, "candidate", closed);
    return { ...value, preservation: { ...value.preservation, baseline_id: f.baselineId }, qualification: q,
      authorizationRecovery: { schema: "worker-authorization-recovery-v1", checkpointId: Buffer.alloc(16, 6).toString("base64url"), generation: q.generation, matched: true } };
  };
  await f.record(f.state(true)); const closedSequence = journal.lastState().sequence;
  await f.record(f.state());
  await writeNew(resolve(root, "device-0003.json"), { schema: "str005-v2-device-record-v1", contextSha256, sequence: 3, atHostMs: ++f.time, record: failed });
  await f.record(f.state());
  await writeNew(resolve(root, "restoration.json"), { schema: "str005-v2-restoration-v1", contextSha256, closedSequence,
    observedSequence: journal.lastState().sequence, deviceSequence: 3, state: journal.lastState().state, recordSha256: sha256(JSON.stringify(failed)) });
  await f.record(f.state());
  await saveAccounting(root, context, { stage: "after", state: f.state(),
    ledger: { ...ledger, next_ordinal: 19, last_completed_ordinal: 18, total_charged_ms: 1740000 }, original_budget: original }, journal.lastState());
  await f.record(f.state(true));
  let live = true;
  operations.processSnapshot = async () => live ? [serverOwner] : [];
  operations.spawnSync = () => ({ status: 1, stdout: "", stderr: "", signal: null });
  operations.fetch = async () => new Response(JSON.stringify({ schema: "str005-v2-cleanup-runtime-v1", contextSha256,
    scope: "share", attemptId: context.attemptId, fixtureInstanceId: instanceId, fixturePort: 54321 }));
  const cleanup = await prepareCleanup(root, context, operations); live = false;
  const last = journal.lastState();
  await cleanup.record({ browser: { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
    lastSequence: last.sequence, lastStateSha256: sha256(JSON.stringify(last)), observedAtUnixMs: Date.now() },
  supervisor: { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner: serverOwner, code: 0,
    observedAtUnixMs: Date.now(), clock: "node-hrtime-ms-v1", stopRequestedAtMs: 1000, exitedAtMs: 1001 } });
  return f;
}

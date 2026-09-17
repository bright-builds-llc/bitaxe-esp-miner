// Explicit synthetic sealed v2 failure. Original published validator bytes retain the real historical rejection.
import { mkdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { permissionContextFixture } from "./context-fixtures.mjs";
import { preparedFixture } from "./completed-fixture.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { createReceiptWriter } from "./execution-receipts.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 as digest } from "./values.mjs";
import { finalize, review } from "./finalize.mjs";
import { loadContext } from "./context.mjs";
import { sourceInventory } from "./context-sources.mjs";
import { SUCCESSOR_MODULES } from "./successor-sources.mjs";

export async function successorFixture(t) {
  const f = await permissionContextFixture(t, { beforeLegacySnapshot: async base => {
    await base.put(resolve(base.options.firmwareRoot, "scripts/str005-v2-serial/continuity-judge.mjs"),
      await readFile(new URL("./successor-historical-continuity.fixture.mjs", import.meta.url)));
  } });
  const firstJournal = await createJournal(f.root, f.context), configured = state(f.context, "before");
  delete configured.preservation;
  Object.assign(configured, {status:"configured",connected:false,running:false,deviceBaselineConfirmed:false,deviceRestorationConfirmed:false,deviceLeaseInactive:false,serialOwnershipReleased:true});
  await firstJournal.state("before", configured, 1);
  await firstJournal.state("before", {...configured,serialOwnershipReleased:false}, 2);
  await preparedFixture(t, f);
  const { root, context, journal } = f, contextSha256 = digest(JSON.stringify(context));
  const vector = channelFixture(); vector.context.attemptId = context.attemptId;
  vector.deviceRecords.forEach(record => { record.attemptId = context.attemptId; });
  vector.connectionComparison.attemptId = context.attemptId;
  const instanceId = vector.fixtureTerminal.instanceId;
  const fixtureOwner = { pid: 81001, ppid: 81000, pgid: 81001, startedAt: "synthetic-fixture" }, serverOwner = { pid: 81000, ppid: 91000, pgid: 81000, startedAt: "synthetic-server" };
  await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256, owner: serverOwner,
    origin: "http://127.0.0.1:32123", port: 32123, atHostMs: 0 });
  await writeNew(resolve(root, "fixture-owner.json"), { schema: "str005-v2-fixture-owner-v1", contextSha256, owner: fixtureOwner, atHostMs: ++f.time,
    binarySha256: context.fixture_sha256 });
  const readyAt = ++f.time;
  await writeNew(resolve(root, "fixture-ready.json"), { schema: "str005-v2-fixture-ready-facts-v1", contextSha256, scope: "channel", attemptId: context.attemptId,
    instanceId, authorityPublicKeySha256: "a".repeat(64), owner: fixtureOwner, readyAtMs: readyAt });
  const first = vector.deviceRecords[0];
  await writeNew(resolve(root, "start.claim.json"), { schema: "str005-v2-start-claim-v1", contextSha256, atHostMs: ++f.time,
    observedStateSequence: journal.lastState().sequence, bootOrdinal: first.bootOrdinal, workerGeneration: first.workerGeneration,
    serialTransportEpoch: first.serialTransportEpoch, networkObservedAtDeviceUs: first.admittedAtDeviceUs - 1,
    fixtureReadySha256: (await proof(root, "fixture-ready.json")).sha256, clientSha256: context.client_sha256, attemptId: context.attemptId });
  let deviceSequence = 0;
  const recordDevice = async record => writeNew(resolve(root, `device-${String(++deviceSequence).padStart(4, "0")}.json`), {
    schema: "str005-v2-device-record-v1", contextSha256, sequence: deviceSequence, atHostMs: ++f.time, record });
  for (const record of vector.deviceRecords) await recordDevice(record);
  const final = vector.deviceRecords.at(-1), connection = vector.connectionComparison;
  connection.readinessSha256 = (await proof(root, "fixture-ready.json")).sha256;
  connection.deviceObservationSequence = deviceSequence; connection.comparisonStartedAtMs = ++f.time; connection.comparisonCompletedAtMs = ++f.time;
  const receipt = createReceiptWriter(root, context);
  await receipt("connection.json", "supervisor-ms", connection.comparisonCompletedAtMs, connection);
  await receipt("job-receipt.json", "device-us", final.events.find(event => event.kind === "work_ready").atDeviceUs, vector.job);
  await mkdir(resolve(root, "fixture-run"), { mode: 0o700 });
  for (const [name, value] of Object.entries({ "job.json": vector.job, "fixture-events.json": vector.fixtureEvents,
    "fixture-terminal.json": vector.fixtureTerminal, "connection-facts.json": vector.fixtureConnectionFacts, "shares.json": vector.fixtureShares }))
    await writeNew(resolve(root, "fixture-run", name), value);
  const exitAt = readyAt + vector.fixtureTerminal.elapsedMs + 10; f.time = Math.max(f.time + 1, exitAt);
  await writeNew(resolve(root, "fixture-exit.json"), { schema: "str005-v2-fixture-exit-v1", contextSha256, code: 0, signal: null,
    atHostMs: f.time, owner: fixtureOwner, stderrBytes: 0, lifetimeMs: f.time - readyAt });
  await writeNew(resolve(root, "fixture-reap.json"), { schema: "str005-v2-fixture-reap-v1", contextSha256,
    kind: "natural_exit", requestedAtHostMs: null, completedAtHostMs: ++f.time, durationMs: null });
  await writeNew(resolve(root, "protocol-complete.json"), { schema: "str005-v2-protocol-complete-v1", contextSha256, atHostMs: ++f.time,
    observedStateSequence: journal.lastState().sequence, observedDeviceSequence: deviceSequence,
    fixtureTerminalSha256: (await proof(root, "fixture-run/fixture-terminal.json")).sha256,
    fixtureSharesSha256: (await proof(root, "fixture-run/shares.json")).sha256 });
  await f.record(f.state("candidate", true)); const closedSequence = journal.lastState().sequence;
  await f.record(f.state()); await recordDevice(final); await f.record(f.state());
  await writeNew(resolve(root, "restoration.json"), { schema: "str005-v2-restoration-v1", contextSha256, closedSequence,
    observedSequence: journal.lastState().sequence, deviceSequence, state: journal.lastState().state, recordSha256: digest(JSON.stringify(final)) });
  await f.record(f.state());
  await saveAccounting(root, context, { stage: "after", ledger, original_budget: original, state: f.state() }, journal.lastState());
  await f.record(f.state("candidate", true));
  const last = journal.lastState(), browser = { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
    lastSequence: last.sequence, lastStateSha256: digest(JSON.stringify(last)), observedAtUnixMs: Date.now() };
  const supervisor = { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner: serverOwner, code: 0,
    observedAtUnixMs: Date.now(), clock: "node-hrtime-ms-v1", stopRequestedAtMs: 1000, exitedAtMs: 1009 };
  await mkdir(`${root}.cleanup`, { mode: 0o700 });
  await writeNew(`${root}.cleanup/browser.json`, browser); await writeNew(`${root}.cleanup/supervisor.json`, supervisor);
  const support = "parent-observations/operator", unclaimedOwner = { pid: 82002, ppid: 91001, pgid: 82002, startedAt: "synthetic-unclaimed" };
  const files = {
    "browser.json": browser, "error-1789625890504.json": { event: "operator_error", code: "v2_listener_inventory_shape" },
    "foreground-detection.log": "explicit synthetic detector\n", "initial-detection.log": "explicit synthetic detector\n",
    "launch-001-unclaimed.json": { schema: "str005-v2-unclaimed-launch-observation-v1", source: "parent-observed", contextUnclaimed: true,
      serverOwnerAbsent: true, serverClaimAbsent: true, failureRecordAbsent: true, owner: unclaimedOwner, childExitCode: "not_collected",
      cause: "parent_initialization_wait_30000ms_exhausted", observedAtUnixMs: Date.now() - 1000 },
    "parent-launch-002.mjs": "// Explicit synthetic parent, not an effect producer.\n", "parent.mjs": "// Explicit synthetic initial parent.\n",
    "supervisor-exit-launch-002.json": supervisor, "supervisor-root-launch-002.json": serverOwner, "supervisor-root.json": unclaimedOwner,
  };
  const operatorFiles = [];
  for (const path of Object.keys(files).sort()) {
    const bytes = typeof files[path] === "string" ? files[path] : `${JSON.stringify(files[path], null, 2)}\n`;
    await f.put(resolve(root, support, path), bytes); operatorFiles.push({ path, sha256: digest(bytes), length: Buffer.byteLength(bytes) });
  }
  await writeNew(resolve(root, "parent-observations/parent-observation.json"), { schema: "str005-v2-parent-cleanup-failure-v1", source: "parent-observed",
    earliestCode: "v2_listener_inventory_shape", proofScope: "protocol-and-restoration-observed;complete-host-cleanup-unverified",
    poolPortAbsence: "not_proven", privatePortExported: false, operatorFiles });
  f.operations.processSnapshot = async () => [];
  const sealed = await finalize(root, `${root}.cleanup/receipt.json`, f.operations);
  if (sealed.status !== "unverified" || (await proof(root, "judgment-failure.json")).value.code !== "v2_baseline") throw new Error("synthetic historical rejection was not exercised");
  const required = [...context.native_source_files, ...context.native_auditor_sources, ...SUCCESSOR_MODULES];
  // Current source is corrected; the sealed old evaluator copy remains byte-for-byte historical.
  await f.put(resolve(context.firmware_root, "scripts/str005-v2-serial/continuity-judge.mjs"), await readFile(new URL("./continuity-judge.mjs", import.meta.url)));
  const publishedSources = await sourceInventory(context.firmware_root, required);
  const operations = { ...f.operations,
    inspectHistoricalChannel: async path => ({ context: await loadContext(path, { historical: true, operations: f.operations }), reviewed: await review(path, f.operations) }),
    git: () => "f".repeat(40),
    publishedCheckerPaths: async () => publishedSources.map(row => row.path),
    publishedCheckerSources: async (_repo, _commit, paths) => paths.map(path => structuredClone(publishedSources.find(row => row.path === path))),
  };
  return { ...f, operations, sealed, unclaimedOwner, serverOwner, fixtureOwner };
}

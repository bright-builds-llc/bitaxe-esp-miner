// Explicit synthetic observations exercise the real V2 filesystem reader; never hardware evidence.
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { contextFixture } from "./context-fixtures.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { claimInstall, reviewInstall, claimProbe, completeProbe, recordCycle } from "./install.mjs";
import { sha256 as digest } from "./values.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { channelFixture } from "./protocol-judge.test-helper.mjs";
import { createReceiptWriter } from "./execution-receipts.mjs";
import { prepareCleanup } from "./cleanup.mjs";
export function qualification() {
  return { schema: "worker-qualification-v1", generation: 0, active_ms: 0, generation_elapsed_ms: 0, budget_reserved_ms: 0,
    submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 0, last_valid_heartbeat_ms: 0,
    budget_complete: true, safe_stop_complete: true, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true,
    watchdog_alive: true, mine_on_boot: false, voltage_volts: 5.2, power_watts: 1, chip_temp_celsius: 30, fan_rpm: 3200,
    gate_closed_ms: 0, shutdown_started_ms: 0, safe_stop_stage: "fan_paused", revocation_reason: "none", active_limit_ms: null,
    shutdown_budget_ms: 15550, work_gate_remaining_ms: null };
}
const owner = (pid) => ({ pid, ppid: 1, pgid: pid, startedAt: `synthetic-${pid}`, state: "S", cpuPercent: 0 });
const observation = (person, time) => ({ schema: "hello-passive-command-observation-v1", rootObserved: true,
  observations: 10, started_at_unix_ms: time - 1000, finished_at_unix_ms: time + 1000, seen: [person], remaining: [], failures: [], observer_effects: "process-metadata-only" });
export async function installed(f, index, claimOnly = false) {
  const { root, context, put } = f, unix = 1_800_000_000_000 + index * 10000, person = owner(60000 + index);
  f.operations.unixNow = () => unix;
  f.operations.processSnapshot = async () => [person];
  await put(resolve(root, `install-${index}.detect.stdout.log`), `port: /dev/cu.synthetic\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${"c".repeat(64)}\n`);
  const detection = observation(owner(50000 + index), unix - 1001);
  await writeNew(resolve(root, `install-${index}.detect.observation.json`), detection);
  await writeNew(resolve(root, `install-${index}.host-root.json`), person);
  await writeNew(resolve(root, `install-${index}.observer-armed.json`), { pid: person.pid, pgid: person.pgid, startedAt: person.startedAt });
  const claim = await claimInstall(root, context, index, () => false, f.operations);
  if (claimOnly) return { claim, person };
  await mkdir(resolve(root, `install-${index}`), { mode: 0o700 });
  await writeNew(resolve(root, `install-${index}/flash-command-evidence.json`), {
    command_kind: "flash-monitor", board: "205", flash_status: "completed", capture_mode: "noninteractive", capture_status: "timed_out_after_trusted_output",
    monitor_evidence_status: "trusted", trusted_output: true, commit_ready: true, firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit, trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided", redaction_mode: "commit-redacted", capture_timeout_seconds: 30, manifest_path: "[redacted-path]",
    timestamp: String(Math.floor(unix / 1000)), fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true,
      startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } });
  await put(resolve(root, `install-${index}/flash-monitor.log`), ["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].map(stage =>
    `usb_memory_checkpoint stage=${stage} free_bytes=200000 largest_block_bytes=100000 reserve_bytes=98304 redacted=true`).join("\n"));
  await writeNew(resolve(root, `install-${index}.observation.json`), observation(person, unix));
  await writeNew(resolve(root, `install-${index}.exit.json`), { schema: "noise-serial-command-exit-v2", contextSha256: digest(JSON.stringify(context)), index,
    code: 0, ownerSha256: (await proof(root, `install-${index}.host-root.json`)).sha256,
    observationSha256: (await proof(root, `install-${index}.observation.json`)).sha256 });
  f.operations.processSnapshot = async () => [];
  await reviewInstall(root, context, index, ++f.time, f.operations);
}

function idle(index, scope) {
  return { schema: "worker-stratum-v2-status-v1", scope, state: "idle", record: null, connection: null,
    observation: { bootOrdinal: index + 1, workerGeneration: 0, serialTransportEpoch: index + 10, observedAtUs: 1000,
      clockValid: true, stationIpv4: "192.168.1.10", wifiConnected: true, socket: null } };
}

export async function preparedFixture(t, maybePrepared) {
  const f = maybePrepared ?? await contextFixture(t); f.time = 100;
  const { root, context } = f, contextSha256 = digest(JSON.stringify(context));
  if (context.scope === "channel") {
  await writeNew(resolve(f.previous.root, "install-0.claim.json"), { detector: { physical: "c".repeat(64) } });
  await writeNew(resolve(f.previous.root, "sealed-inventory.json"), { files: [{ path: "install-0.claim.json",
    sha256: (await proof(f.previous.root, "install-0.claim.json")).sha256 }] });
  }
  const journal = await createJournal(root, context);
  f.baselineId = context.scope === "share" ? Buffer.alloc(16, 7).toString("base64url") : state(context).preservation.baseline_id;
  f.state = (phase = "candidate", closed = false) => {
    const value = state(context, phase, closed);
    return { ...value, preservation: { ...value.preservation, baseline_id: f.baselineId },
      ...(phase === "candidate" ? { qualification: qualification() } : {}) };
  };
  f.record = (s, phase = "candidate") => journal.state(phase, s, ++f.time);
  await f.record(f.state("before"), "before");
  await saveAccounting(root, context, { stage: "before-install", ledger, original_budget: original, state: f.state("before") }, journal.lastState());
  await f.record(f.state("before", true), "before");
  if (context.scope === "channel") await installed(f, 0);
  await f.record(f.state());
  for (let index = 1; index <= 4; index++) {
    await f.record(f.state("candidate", true)); await installed(f, index); await f.record(f.state());
    const status = idle(index, context.scope), ticket = await claimProbe(root, context, index, status, ++f.time);
    const probed = { ...f.state(), probe: { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 } };
    await f.record(probed); status.observation.observedAtUs++;
    await completeProbe(root, context, { index, nonce: ticket.probe_nonce, probe: probed.probe, state: probed, status }, ++f.time);
    await recordCycle(root, context, index);
  }
  await f.record(f.state());
  await saveAccounting(root, context, { stage: "before", ledger, original_budget: original, state: f.state() }, journal.lastState());
  return Object.assign(f, { journal });
}

export async function completedFixture(t) {
  const f = await preparedFixture(t);
  const { root, context, journal } = f, contextSha256 = digest(JSON.stringify(context));
  const vector = channelFixture(); vector.context.attemptId = context.attemptId;
  vector.deviceRecords.forEach(record => { record.attemptId = context.attemptId; });
  vector.connectionComparison.attemptId = context.attemptId;
  const instanceId = vector.fixtureTerminal.instanceId;
  const fixtureOwner = { pid: 81001, pgid: 81001, startedAt: "synthetic-fixture" }, serverOwner = { pid: 81000, pgid: 81000, startedAt: "synthetic-server" };
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
  let live = true;
  f.operations.processSnapshot = async () => live ? [serverOwner] : [];
  f.operations.spawnSync = () => ({ status: 1, stdout: "", stderr: "", signal: null });
  f.operations.fetch = async () => new Response(JSON.stringify({ schema: "str005-v2-cleanup-runtime-v1", contextSha256,
    scope: "channel", attemptId: context.attemptId, fixtureInstanceId: instanceId, fixturePort: 54321 }));
  const cleanup = await prepareCleanup(root, context, f.operations); live = false;
  const last = journal.lastState();
  await cleanup.record({ browser: { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
    lastSequence: last.sequence, lastStateSha256: digest(JSON.stringify(last)), observedAtUnixMs: Date.now() },
  supervisor: { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner: serverOwner, code: 0,
    observedAtUnixMs: Date.now(), clock: "node-hrtime-ms-v1", stopRequestedAtMs: 1000, exitedAtMs: 1001 } });
  return f;
}

// Synthetic protected observations for the real independent reader/finalizer.
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fixture, state, ledger, original } from "./test-fixture.mjs";
import { example, admitted } from "./fixtures.mjs";
import { recordState, saveAccounting, recordNoise, readJournal } from "./journal.mjs";
import { claimInstall, reviewInstall, claimProbe, completeProbe, recordCycle } from "./install.mjs";
import { canonical, digest, proof, writeNew } from "./files.mjs";
import { recordCleanup } from "./cleanup.mjs";

function qualification() {
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
function idle(boot = 1, epoch = 2) {
  const v = admitted(); v.schema = "worker-noise-diagnostic-status-v2"; v.state = "idle"; v.job = null;
  v.observation.bootOrdinal = boot; v.observation.transportEpoch = epoch; return v;
}
async function installed(f, index, claimOnly = false) {
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
export async function completedFixture(t) {
  const f = await fixture(t); f.time = 100;
  const { root, context } = f, contextSha256 = digest(JSON.stringify(context));
  f.state = (phase = "candidate", closed = false) => ({ ...state(context, phase, closed), ...(phase === "candidate" ? { qualification: qualification() } : {}) });
  f.record = (s, phase = "candidate") => recordState(root, context, phase, s, ++f.time);
  await writeNew(resolve(root, "server.claim.json"), { schema: "noise-serial-server-claim-v2", contextSha256 });
  await f.record(f.state("before"), "before");
  await saveAccounting(root, context, { stage: "before-install", ledger, original_budget: original, state: f.state("before") });
  await f.record(f.state("before", true), "before");
  await installed(f, 0);
  await f.record(f.state());
  for (let index = 1; index <= 4; index++) {
    await f.record(f.state("candidate", true)); await installed(f, index);
    await f.record(f.state());
    const status = idle(index + 1, index + 10);
    const ticket = await claimProbe(root, context, index, status, ++f.time);
    const withProbe = { ...f.state(), probe: { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 } };
    await f.record(withProbe);
    await completeProbe(root, context, { index, nonce: ticket.probe_nonce, probe: withProbe.probe, state: withProbe, status }, ++f.time);
    await recordCycle(root, context, index);
  }
  await f.record(f.state());
  await saveAccounting(root, context, { stage: "before", ledger, original_budget: original, state: f.state() });
  const vector = example(); vector.start.schema = "worker-noise-diagnostic-start-v2"; vector.start.attemptId = context.attempt_id;
  vector.fixtureReady.attemptId = context.attempt_id; vector.fixtureTerminal.attemptId = context.attempt_id;
  await mkdir(resolve(root, "fixture-run"), { mode: 0o700 });
  await writeNew(resolve(root, "fixture-run/ready.json"), vector.fixtureReady);
  await writeNew(resolve(root, "fixture-run/terminal.json"), vector.fixtureTerminal);
  await writeNew(resolve(root, "fixture-start.claim.json"), { schema: "noise-serial-fixture-claim-v2", contextSha256, atHostMs: ++f.time,
    binarySha256: context.fixture_sha256, selected: { name: "synthetic", address: "192.168.1.20", netmask: "255.255.255.0" }, station: "192.168.1.10" });
  const readyAt = ++f.time;
  await writeNew(resolve(root, "fixture-ready-observation.json"), { contextSha256, atHostMs: readyAt, readySha256: (await proof(root, "fixture-run/ready.json")).sha256 });
  const beforeSequence = (await readJournal(root, context)).at(-1).sequence;
  const network = idle(); network.observation.observedAtUs = vector.start.networkObservedAtUs;
  await writeNew(resolve(root, "start.claim.json"), { schema: "noise-serial-start-claim-v2", contextSha256, atHostMs: ++f.time,
    observedSequence: beforeSequence, inputSha256: digest(canonical(vector.start)), start: vector.start, observation: network });
  const first = admitted(); first.schema = "worker-noise-diagnostic-status-v2"; first.job.attemptId = context.attempt_id; first.job.inputSha256 = digest(canonical(vector.start));
  await recordNoise(root, context, first, ++f.time);
  const accepted = vector.device; accepted.schema = "worker-noise-diagnostic-status-v2";
  accepted.job.attemptId = context.attempt_id; accepted.job.inputSha256 = first.job.inputSha256;
  accepted.job.resources.deadlineAtUs = accepted.job.authorityDeadlineUs + 5000000;
  await recordNoise(root, context, accepted, ++f.time);
  await f.record(f.state());
  const completeSequence = (await readJournal(root, context)).at(-1).sequence;
  await writeNew(resolve(root, "diagnostic-complete.json"), { schema: "noise-serial-complete-v2", contextSha256, atHostMs: ++f.time,
    observedSequence: completeSequence, statusSha256: digest(canonical(accepted)), fixtureSha256: digest(canonical(vector.fixtureTerminal)) });
  const fixturePerson = owner(81001), supervisorPerson = owner(81000);
  await writeNew(resolve(root, "fixture-owner.json"), { schema: "noise-serial-fixture-owner-v2", contextSha256, pid: fixturePerson.pid, group: fixturePerson.pgid,
    owner: fixturePerson, atHostMs: readyAt - 1, binarySha256: context.fixture_sha256 });
  const exitAt = ++f.time;
  await writeNew(resolve(root, "fixture-exit.json"), { schema: "noise-serial-fixture-exit-v2", contextSha256, code: 0, signal: null,
    atHostMs: exitAt, output: { stdoutBytes: 0, stderrBytes: 0, codes: [] }, owner: fixturePerson, lifetimeMs: exitAt - readyAt });
  await writeNew(resolve(root, "fixture-reap.json"), { schema: "noise-serial-fixture-reap-v2", contextSha256,
    startedAtHostMs: exitAt, completedAtHostMs: exitAt + 1, durationMs: 1 });
  await writeNew(resolve(root, "server-owner.json"), { schema: "noise-serial-server-owner-v2", contextSha256, owner: supervisorPerson,
    origin: "http://127.0.0.1:32123", port: 32123, atHostMs: 0 });
  await f.record(f.state("candidate", true)); const closedSequence = (await readJournal(root, context)).at(-1).sequence;
  await f.record(f.state());
  const restoredStatus = structuredClone(accepted); restoredStatus.observation.transportEpoch++;
  await recordNoise(root, context, restoredStatus, ++f.time);
  await f.record(f.state());
  await writeNew(resolve(root, "restoration.json"), { schema: "noise-serial-restoration-v2", contextSha256, closedSequence,
    observedSequence: (await readJournal(root, context)).at(-1).sequence, status: restoredStatus, state: f.state() });
  await f.record(f.state()); await saveAccounting(root, context, { stage: "after", ledger, original_budget: original, state: f.state() });
  await f.record(f.state("candidate", true));
  const last = (await readJournal(root, context)).at(-1);
  await recordCleanup(root, context, {
    browser: { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256, closed: true,
      lastSequence: last.sequence, lastStateSha256: digest(JSON.stringify(last)), observedAtUnixMs: 1000 },
    supervisor: { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256, owner: supervisorPerson,
      code: 0, observedAtUnixMs: 1000, clock: "node-hrtime-ms-v1", stopRequestedAtMs: 100, exitedAtMs: 101 },
  }, f.operations);
  return f;
}

export async function claimedFixture(t) {
  const f = await fixture(t); f.time = 100;
  await recordState(f.root, f.context, "before", state(f.context, "before"), ++f.time);
  await saveAccounting(f.root, f.context, { stage: "before-install", state: state(f.context, "before"), ledger, original_budget: original });
  await recordState(f.root, f.context, "before", state(f.context, "before", true), ++f.time);
  const { claim, person } = await installed(f, 0, true);
  f.operations.pid = person.pid;
  return { ...f, permit: { kind: "execute", contextSha256: claim.context_sha256, claimSha256: claim.claim_sha256 } };
}

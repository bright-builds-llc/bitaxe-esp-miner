import { lstat, readFile, readdir, realpath } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { digest, exactObject, fileDigest, protectedPath, requireCondition as check } from "./contract.mjs";
import { validateCycle } from "./judge.mjs";

const SEAL = "failed-inventory.json";
const forbidden = (name) => /credential|private[-_.]?key|signing[-_.]?key|^(authority|secrets?)(\.|$)|\.(pem|key)$/iu.test(name);
export async function proof(path) {
  check(!forbidden(basename(path)), "forbidden_evidence_kind");
  check((await realpath(path)) === resolve(path), "evidence_alias");
  await protectedPath(path);
  const bytes = await readFile(path);
  return { value: JSON.parse(bytes), sha256: digest(bytes) };
}
export function baseline(state, closed) {
  check(
    state.deviceBaselineConfirmed === true &&
      state.deviceLeaseInactive &&
      !state.running &&
      !state.failure &&
      state.renewalsConfirmed === 0 &&
      state.preservation?.device_identity_match &&
      state.preservation.settings_match &&
      state.preservation.authorization_high_water_match &&
      state.preservation.mine_on_boot === false,
    "cycle_baseline",
  );
  check(
    closed
      ? state.status === "closed" && !state.connected && state.serialOwnershipReleased
      : state.status === "ready" && state.connected && !state.serialOwnershipReleased,
    "cycle_connection",
  );
}
function host(value, cycle, phase) {
  exactObject(
    value,
    [
      "schema",
      "cycle",
      "phase",
      "observed_at_unix_ms",
      "observer",
      "browser_serial_released",
      "usb_holder_count",
      "owned_command_children_remaining",
    ],
    phase === "after_flash" ? ["flash_command_exit_code"] : [],
  );
  check(
    value.schema === "hello-cycle-host-observation-v1" &&
      value.cycle === cycle &&
      value.phase === phase &&
      value.observer === "parent-observed" &&
      Number.isSafeInteger(value.observed_at_unix_ms) &&
      value.observed_at_unix_ms > 0 &&
      value.browser_serial_released &&
      value.usb_holder_count === 0 &&
      value.owned_command_children_remaining === 0 &&
      (phase !== "after_flash" || value.flash_command_exit_code === 0),
    "cycle_host_witness",
  );
}
function processProof(value) {
  check(
    value.schema === "hello-passive-command-observation-v1" &&
      value.rootObserved === true &&
      Number.isSafeInteger(value.observations) &&
      value.observations > 0 &&
      Array.isArray(value.failures) &&
      value.failures.length === 0 &&
      Array.isArray(value.remaining) &&
      value.remaining.length === 0 &&
      value.complete !== false,
    "flash_process_cleanup",
  );
}
async function flash(root, number, context) {
  const saved = await proof(resolve(root, `flash-${number}/flash-command-evidence.json`)),
    f = saved.value,
    a = f.fixed_serial_assessment;
  check(
    f.flash_status === "completed" &&
      f.monitor_evidence_status === "trusted" &&
      f.trusted_output === true &&
      f.commit_ready === true &&
      f.firmware_commit === context.firmware_commit &&
      f.observed_firmware_commit === context.firmware_commit &&
      f.reference_commit === context.reference_commit &&
      f.observed_reference_commit === "Unavailable" &&
      f.trust_basis === "fixed_serial" &&
      f.nvs_seed_status === "not_provided" &&
      f.redaction_mode === "commit-redacted" &&
      f.capture_timeout_seconds === 30 &&
      f.manifest_path === "[redacted-path]" &&
      a?.execution_present === true &&
      a.safe_baseline_confirmed === true &&
      a.startup_complete === true &&
      a.startup_failed === false &&
      a.stable_boot === true &&
      Array.isArray(a.issues) &&
      a.issues.length === 0,
    "flash_identity_startup",
  );
  const observation = await proof(resolve(root, `flash-${number}.observation.json`));
  processProof(observation.value);
  return { saved, observation };
}
async function cycle(root, n, context, records, flashed, previous, previousAudit) {
  const observedInput = await proof(resolve(root, `cycle-${n}.observed-input.json`)),
    official = await proof(resolve(root, `cycle-${n}.json`));
  const report = validateCycle(official.value, context, previous);
  check(isDeepStrictEqual(observedInput.value, report), "official_cycle_input_changed");
  const original = await proof(resolve(root, `cycle-${n}.audit-input.json`)),
    input = original.value;
  exactObject(input, [
    "schema",
    "cycle",
    "before_closed_sequence",
    "after_ready_sequence",
    "before_host_witness",
    "after_host_witness",
    "probe_witness",
    "flash_record",
  ]);
  check(
    input.schema === "hello-cycle-audit-input-v1" &&
      input.cycle === n &&
      input.before_host_witness === resolve(root, `cycle-${n}.before_flash.json`) &&
      input.after_host_witness === resolve(root, `cycle-${n}.after_flash.json`) &&
      input.probe_witness === resolve(root, `cycle-${n}.browser-observation.json`) &&
      input.flash_record === resolve(root, `flash-${n}/flash-command-evidence.json`),
    "cycle_source_paths",
  );
  const b = input.before_closed_sequence,
    a = input.after_ready_sequence;
  check(
    Number.isInteger(b) &&
      Number.isInteger(a) &&
      b > 0 &&
      a > b &&
      a <= records.length &&
      (!previousAudit || previousAudit.after_ready_sequence < b),
    "cycle_journal_order",
  );
  const before = records[b - 1].state,
    after = records[a - 1].state;
  baseline(before, true);
  baseline(after, false);
  check(
    before.preservation.baseline_id === after.preservation.baseline_id && after.preservation.baseline_id === report.baseline_id,
    "cycle_preservation_binding",
  );
  const beforeHost = await proof(input.before_host_witness),
    afterHost = await proof(input.after_host_witness);
  host(beforeHost.value, n, "before_flash");
  host(afterHost.value, n, "after_flash");
  check(beforeHost.value.observed_at_unix_ms < afterHost.value.observed_at_unix_ms, "cycle_host_order");
  const f = flashed.saved.value;
  check(typeof f.timestamp === "string" && /^[0-9]{1,12}$/u.test(f.timestamp), "flash_timestamp");
  const at = Number(f.timestamp) * 1000;
  check(
    Number.isSafeInteger(at) && at + 999 >= beforeHost.value.observed_at_unix_ms && at <= afterHost.value.observed_at_unix_ms,
    "flash_bracketing",
  );
  const probe = await proof(input.probe_witness),
    p = probe.value;
  exactObject(p, [
    "schema",
    "cycle",
    "observer",
    "fresh_connect_completed",
    "probe_completed",
    "after_ready_sequence",
    "observed_at_unix_ms",
    "result",
  ]);
  exactObject(p.result, ["paddingBytes", "requestPayloadBytes", "responsePayloadBytes"]);
  check(
    p.schema === "hello-cycle-browser-observation-v1" &&
      p.cycle === n &&
      p.observer === "parent-observed" &&
      p.fresh_connect_completed &&
      p.probe_completed &&
      p.after_ready_sequence === a &&
      Number.isSafeInteger(p.observed_at_unix_ms) &&
      p.observed_at_unix_ms >= afterHost.value.observed_at_unix_ms &&
      p.result.requestPayloadBytes === 65536 &&
      p.result.responsePayloadBytes === 65536 &&
      isDeepStrictEqual(p.result, after.probe),
    "cycle_fresh_probe",
  );
  for (const phase of ["before_flash", "after_flash"]) {
    const source = (await proof(resolve(root, `cycle-${n}.${phase}-source.json`))).value;
    check(
      Number.isInteger(source.journal_sequence) && source.journal_sequence >= b && source.journal_sequence < a,
      "cycle_host_source_journal",
    );
    baseline(records[source.journal_sequence - 1].state, true);
    check(typeof source.process_observation === "string", "cycle_process_source_path");
    const processPath = resolve(context.firmware_root, source.process_observation);
    check(dirname(processPath) === root, "cycle_process_source_path");
    const process = await proof(processPath);
    processProof(process.value);
    check(
      process.sha256 === source.process_observation_sha256 &&
        source.device_observation_sha256 === (await fileDigest(resolve(root, "detector.device.private.json"))),
      "cycle_host_source_digest",
    );
    if (phase === "after_flash")
      check(processPath === resolve(root, `flash-${n}.observation.json`), "cycle_flash_process_binding");
  }
  const expected = {
    schema: "hello-cycle-observation-audit-v1",
    cycle: n,
    context_sha256: digest(JSON.stringify(context)),
    before_closed_sequence: b,
    after_ready_sequence: a,
    before_state_sha256: digest(JSON.stringify(before)),
    after_state_sha256: digest(JSON.stringify(after)),
    inputs: {
      observations: { path: resolve(root, `cycle-${n}.audit-input.json`), sha256: original.sha256 },
      before_host: { path: input.before_host_witness, sha256: beforeHost.sha256 },
      after_host: { path: input.after_host_witness, sha256: afterHost.sha256 },
      probe: { path: input.probe_witness, sha256: probe.sha256 },
      flash: { path: input.flash_record, sha256: flashed.saved.sha256 },
    },
    report_sha256: digest(JSON.stringify(report)),
    package_reference_bound: true,
    runtime_reference_observed: false,
    receipt_manifest_path_observed: false,
    hardware_execution_claimed_by_helper: false,
  };
  const audit = await proof(resolve(root, `cycle-${n}.observation-audit.json`));
  check(isDeepStrictEqual(audit.value, expected), "cycle_observation_audit_changed");
  return { report, audit: expected };
}
export async function inventory(root, directory = root) {
  await protectedPath(directory, true);
  const rows = [];
  for (const name of (await readdir(directory)).sort()) {
    if (directory === root && name === SEAL) continue;
    check(!forbidden(name), "forbidden_inventory_kind");
    const path = resolve(directory, name),
      info = await lstat(path),
      local = relative(root, path);
    check(!info.isSymbolicLink(), "inventory_alias");
    if (info.isDirectory()) {
      await protectedPath(path, true);
      rows.push({ path: local, type: "directory", mode: 0o700 }, ...(await inventory(root, path)));
    } else {
      await protectedPath(path);
      rows.push({ path: local, type: "file", mode: 0o600, length: info.size, sha256: await fileDigest(path) });
    }
  }
  return rows;
}

export async function observerProof(root, arm) {
  const saved = await proof(resolve(root, "cadence-observer-result.json")),
    o = saved.value;
  exactObject(o, [
    "schema",
    "connected",
    "closed",
    "exitCode",
    "reason",
    "cleanupComplete",
    "startedAtUnixMs",
    "connectedAtUnixMs",
    "closedAtUnixMs",
    "messageCount",
    "totalBytes",
    "eventCount",
    "journalSha256",
  ]);
  check(
    o.schema === "worker-cadence-observer-result-v1" &&
      o.connected === true &&
      o.closed === true &&
      o.exitCode === 0 &&
      o.reason === "requested" &&
      o.cleanupComplete === true &&
      [o.startedAtUnixMs, o.connectedAtUnixMs, o.closedAtUnixMs].every((value) => Number.isSafeInteger(value) && value >= 0) &&
      o.connectedAtUnixMs >= o.startedAtUnixMs &&
      o.closedAtUnixMs >= o.connectedAtUnixMs &&
      o.closedAtUnixMs - o.startedAtUnixMs <= 366000 &&
      o.connectedAtUnixMs <= arm.started_at_unix_ms,
    "healthy_observer_close",
  );
  const path = resolve(root, "cadence-observer.jsonl");
  await protectedPath(path);
  const bytes = await readFile(path);
  check(bytes.length > 0 && bytes.length <= 1048576 && bytes.at(-1) === 10 && digest(bytes) === o.journalSha256, "observer_journal_digest");
  const rows = bytes.toString("utf8").trim().split("\n").map(JSON.parse);
  check(rows.length === o.eventCount && rows.length >= 3 && rows.length <= 2048, "observer_event_count");
  let lastTime = o.startedAtUnixMs,
    lastElapsed = 0,
    messages = 0,
    total = 0;
  for (const [index, row] of rows.entries()) {
    exactObject(row, ["sequence", "observedAtUnixMs", "event"]);
    const e = row.event;
    exactObject(e, ["schema", "event", "elapsedMs", "messageCount", "totalBytes", "byteCount", "reason"]);
    check(
      row.sequence === index + 1 &&
        Number.isSafeInteger(row.observedAtUnixMs) &&
        row.observedAtUnixMs >= lastTime &&
        e.schema === "cpu0-cadence-observer-v1" &&
        Number.isSafeInteger(e.elapsedMs) &&
        e.elapsedMs >= lastElapsed &&
        e.elapsedMs <= 360000,
      "observer_event_shape",
    );
    if (index === 0) check(e.event === "connected" && e.reason === null && e.byteCount === 0, "observer_connected_event");
    else if (index === rows.length - 1)
      check(e.event === "closed" && e.reason === "requested" && e.byteCount === 0, "observer_closed_event");
    else {
      check(
        e.event === "arrival" && e.reason === null && Number.isSafeInteger(e.byteCount) && e.byteCount > 0 && e.byteCount <= 65536,
        "observer_arrival_event",
      );
      messages++;
      total += e.byteCount;
    }
    check(e.messageCount === messages && e.totalBytes === total, "observer_counters");
    lastTime = row.observedAtUnixMs;
    lastElapsed = e.elapsedMs;
  }
  check(
    messages === o.messageCount &&
      total === o.totalBytes &&
      rows[0].observedAtUnixMs === o.connectedAtUnixMs &&
      lastTime <= o.closedAtUnixMs &&
      o.closedAtUnixMs - lastTime <= 6000,
    "observer_terminal_binding",
  );
  check(lastTime - arm.started_at_unix_ms >= 60000, "idle_observer_duration");
  return { sha256: saved.sha256, result: o, terminal_observed_at: lastTime };
}

/** Reconstruct the four official reports from observed journal/flash/probe sources. */
export async function verifyPreparationCycles(root, context, records) {
  const flashes = [];
  for (let n = 0; n <= 4; n++) flashes.push(await flash(root, n, context));
  let previous, previousAudit;
  for (let n = 1; n <= 4; n++) {
    const checked = await cycle(root, n, context, records, flashes[n], previous, previousAudit);
    previous = checked.report;
    previousAudit = checked.audit;
  }
  return { last_ready_sequence: previousAudit.after_ready_sequence, flash_count: 5, cycle_count: 4 };
}

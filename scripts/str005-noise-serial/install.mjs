import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { exactObject, missing } from "../fixed-usb-qualification/contract.mjs";
import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { flashArguments } from "./operator-execution.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { requireNoHolders, processSnapshot, sameProcess } from "./host-resources.mjs";
import { canonical, check, digest, proof, protectedPath, writeNew } from "./files.mjs";

function indexValue(index) { check(Number.isInteger(index) && index >= 0 && index <= 4, "noise_install_index"); }
function processObservation(value) {
  check(value.schema === "hello-passive-command-observation-v1" && value.rootObserved && value.observations > 0 &&
    value.complete !== false && Array.isArray(value.seen) && value.seen.length > 0 &&
    Array.isArray(value.remaining) && value.remaining.length === 0 && Array.isArray(value.failures) && value.failures.length === 0 &&
    Number.isSafeInteger(value.started_at_unix_ms) && Number.isSafeInteger(value.finished_at_unix_ms) &&
    value.finished_at_unix_ms >= value.started_at_unix_ms, "noise_command_observation");
}
async function detector(root, index, operations) {
  const log = resolve(root, `install-${index}.detect.stdout.log`);
  await protectedPath(log);
  const bytes = await readFile(log); check(bytes.length <= 1048576, "noise_detector_bound");
  const lines = [...bytes.toString("utf8").matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  const unique = (key) => { const found = lines.filter((row) => row[1] === key); check(found.length === 1, "noise_detector_ambiguous"); return found[0][2]; };
  const port = unique("port"), profile = unique("usb_profile"), physical = unique("physical_identity_sha256");
  check(profile === "serial_jtag_runtime" && /^[a-f0-9]{64}$/u.test(physical), "noise_detector_identity");
  const observed = await proof(root, `install-${index}.detect.observation.json`); processObservation(observed.value);
  const now = (operations.unixNow ?? Date.now)();
  check(now >= observed.value.finished_at_unix_ms && now - observed.value.finished_at_unix_ms <= 60000, "noise_detector_stale");
  requireNoHolders(port, operations);
  return { port, profile, physical, logSha256: digest(bytes), observationSha256: observed.sha256 };
}
export async function claimInstall(root, context, index, isFailed, operations = {}) {
  indexValue(index); check(!isFailed(), "noise_terminal_failure");
  await missing(resolve(root, `install-${index}`)); await missing(resolve(root, "start.claim.json"));
  const rows = await readJournal(root, context), last = rows.at(-1);
  check(last, "noise_baseline_missing"); baseline(last.state, true);
  const accounting = (await proof(root, "accounting-before-install.json")).value;
  check(last.sequence > accounting.observedSequence && last.state.preservation.baseline_id === accounting.state.preservation.baseline_id,
    "noise_install_baseline_join");
  if (index > 0) {
    await proof(root, `install-${index - 1}.review.json`);
    if (index > 1) await proof(root, `cycle-${index - 1}.json`);
  }
  const detected = await detector(root, index, operations);
  if (index > 0) check(detected.physical === (await proof(root, "install-0.claim.json")).value.detector.physical, "noise_physical_device_changed");
  const armed = await proof(root, `install-${index}.observer-armed.json`), owner = await proof(root, `install-${index}.host-root.json`);
  check(sameProcess(armed.value, owner.value), "noise_install_not_armed");
  const actual = await (operations.processSnapshot ?? processSnapshot)();
  check(actual.some((row) => sameProcess(row, owner.value)), "noise_install_owner_missing");
  const argv = flashArguments(root, context, index, detected.port);
  check(!isFailed(), "noise_terminal_failure");
  const claim = { schema: "noise-serial-install-claim-v2", contextSha256: digest(JSON.stringify(context)),
    index, beforeSequence: last.sequence, beforeStateSha256: digest(canonical(last)), atUnixMs: (operations.unixNow ?? Date.now)(), detector: detected,
    ownerSha256: owner.sha256, armedSha256: armed.sha256, argv };
  await writeNew(resolve(root, `install-${index}.claim.json`), claim);
  check(!isFailed(), "noise_terminal_failure");
  return { install_claimed: true, index, program: "just", argv, context_sha256: digest(JSON.stringify(context)),
    claim_sha256: digest(`${JSON.stringify(claim, null, 2)}\n`) };
}
export async function inspectInstall(root, context, index) {
  indexValue(index);
  const claim = (await proof(root, `install-${index}.claim.json`)).value;
  check(claim.schema === "noise-serial-install-claim-v2" && claim.contextSha256 === digest(JSON.stringify(context)) && claim.index === index, "noise_install_claim");
  const artifact = await proof(root, `install-${index}/flash-command-evidence.json`), f = artifact.value, assessment = f.fixed_serial_assessment;
  check(f.command_kind === "flash-monitor" && f.board === "205" && f.flash_status === "completed" && f.capture_mode === "noninteractive" &&
    ["completed", "timed_out_after_trusted_output"].includes(f.capture_status) && f.monitor_evidence_status === "trusted" &&
    f.trusted_output === true && f.commit_ready === true && f.firmware_commit === context.firmware_commit &&
    f.observed_firmware_commit === context.firmware_commit && f.reference_commit === context.reference_commit &&
    f.trust_basis === "fixed_serial" && f.nvs_seed_status === "not_provided" && f.redaction_mode === "commit-redacted" &&
    f.capture_timeout_seconds === 30 && f.manifest_path === "[redacted-path]" && assessment?.execution_present === true &&
    assessment.safe_baseline_confirmed === true && assessment.startup_complete === true && assessment.startup_failed === false &&
    assessment.stable_boot === true && assessment.retained_failure_history !== true && Array.isArray(assessment.issues) && assessment.issues.length === 0, "noise_install_unqualified");
  const capturePath = resolve(root, `install-${index}/flash-monitor.log`);
  await protectedPath(capturePath);
  const capture = await readFile(capturePath);
  check(capture.length <= 8_388_608 && (f.monitor_log_sha256 === undefined || f.monitor_log_sha256 === digest(capture)), "noise_startup_capture");
  const memory = [];
  const text = capture.toString("utf8").replace(/\u001b\[[0-?]*[ -/]*[@-~]/gu, "");
  for (const match of text.matchAll(/usb_memory_checkpoint stage=(worker_owner_prepare|usb_install|usb_installed|statistics_start|statistics_started|wifi_driver_prepare|wifi_driver_prepared) free_bytes=(\d{1,10}) largest_block_bytes=(\d{1,10}) reserve_bytes=(\d{1,10}) redacted=true/gu)) {
    const row = { stage: match[1], freeBytes: Number(match[2]), largestBlockBytes: Number(match[3]), reserveBytes: Number(match[4]) };
    check([row.freeBytes, row.largestBlockBytes, row.reserveBytes].every((v) => Number.isInteger(v) && v >= 0 && v <= 0xffffffff) &&
      row.reserveBytes === 98304, "noise_startup_heap_shape");
    if (!memory.some((prior) => JSON.stringify(prior) === JSON.stringify(row))) memory.push(row);
  }
  check(["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].every((stage) => memory.some((row) => row.stage === stage)) && memory.length <= 128,
    "noise_fresh_startup_heap_missing");
  const observed = await proof(root, `install-${index}.observation.json`); processObservation(observed.value);
  const owner = await proof(root, `install-${index}.host-root.json`), armed = await proof(root, `install-${index}.observer-armed.json`);
  check(owner.sha256 === claim.ownerSha256 && armed.sha256 === claim.armedSha256 && sameProcess(owner.value, armed.value) &&
    observed.value.seen.some((row) => sameProcess(row, owner.value)) && observed.value.started_at_unix_ms <= claim.atUnixMs &&
    observed.value.finished_at_unix_ms >= claim.atUnixMs, "noise_install_observer_binding");
  check(typeof f.timestamp === "string" && /^[0-9]+$/u.test(f.timestamp) && Number(f.timestamp) * 1000 + 999 >= claim.atUnixMs &&
    Number(f.timestamp) * 1000 <= observed.value.finished_at_unix_ms, "noise_install_chronology");
  const exit = (await proof(root, `install-${index}.exit.json`)).value;
  exactObject(exit, ["schema", "contextSha256", "index", "code", "ownerSha256", "observationSha256"]);
  check(exit.schema === "noise-serial-command-exit-v2" && exit.contextSha256 === claim.contextSha256 && exit.index === index && exit.code === 0 &&
    exit.ownerSha256 === owner.sha256 && exit.observationSha256 === observed.sha256, "noise_install_exit");
  return { claim, flashSha256: artifact.sha256, observationSha256: observed.sha256, owners: observed.value.seen,
    startupCaptureSha256: digest(capture), startupMemory: memory };
}
export async function reviewInstall(root, context, index, now, operations = {}) {
  const reviewed = await inspectInstall(root, context, index);
  const { requireGone } = await import("./host-resources.mjs");
  await requireGone(reviewed.owners, operations); requireNoHolders(reviewed.claim.detector.port, operations);
  const value = { schema: "noise-serial-install-review-v2", contextSha256: digest(JSON.stringify(context)), index,
    beforeSequence: reviewed.claim.beforeSequence, atHostMs: now, flashSha256: reviewed.flashSha256, observationSha256: reviewed.observationSha256,
    startupCaptureSha256: reviewed.startupCaptureSha256, startupMemory: reviewed.startupMemory };
  await writeNew(resolve(root, `install-${index}.review.json`), value);
  return { install_verified: true, index };
}
export async function recordCycle(root, context, index) {
  check(Number.isInteger(index) && index >= 1 && index <= 4, "noise_cycle_index");
  const review = (await proof(root, `install-${index}.review.json`)).value;
  const rows = await readJournal(root, context), before = rows[review.beforeSequence - 1], after = rows.at(-1);
  check(after.sequence > before.sequence && after.atHostMs > review.atHostMs, "noise_cycle_order");
  const probe = await inspectProbe(root, context, index, rows);
  check(probe.beforeSequence > before.sequence && probe.afterSequence === after.sequence, "noise_cycle_fresh_probe");
  baseline(before.state, true); baseline(after.state, false);
  check(after.state.probe?.requestPayloadBytes === 65536 && after.state.probe?.responsePayloadBytes === 65536 &&
    after.state.preservation.baseline_id === before.state.preservation.baseline_id, "noise_probe_required");
  const previous = index > 1 ? (await proof(root, `cycle-${index - 1}.json`)).value.report : undefined;
  const report = validateCycle({ schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit,
    app_elf_sha256: context.app_elf_sha256, baseline_id: after.state.preservation.baseline_id,
    browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
    device_identity_match: true, settings_match: true, authorization_high_water_match: true,
    probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false }, context, previous);
  await writeNew(resolve(root, `cycle-${index}.json`), { schema: "noise-serial-cycle-v2", contextSha256: digest(JSON.stringify(context)),
    beforeSequence: before.sequence, afterSequence: after.sequence, installReviewSha256: (await proof(root, `install-${index}.review.json`)).sha256, report });
  return { cycle_verified: true, index };
}

export async function claimProbe(root, context, index, status, now) {
  check(Number.isInteger(index) && index >= 1 && index <= 4, "noise_probe_index");
  const { parseNoiseStatusV2 } = await import("./device-v2.mjs");
  const { nonce } = await import("../fixed-usb-qualification/contract.mjs");
  parseNoiseStatusV2(status); check(status.state === "idle", "noise_probe_idle");
  const review = (await proof(root, `install-${index}.review.json`)).value;
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last.state, false);
  check(last.sequence > review.beforeSequence && last.atHostMs > review.atHostMs, "noise_probe_before_reconnect");
  const value = { schema: "noise-serial-probe-claim-v2", contextSha256: digest(JSON.stringify(context)), index,
    nonce: nonce(), beforeSequence: last.sequence, atHostMs: now, connection: status.observation,
    clientSha256: context.client_sha256 };
  await writeNew(resolve(root, `cycle-${index}.probe-claim.json`), value);
  return { probe_nonce: value.nonce };
}
export async function completeProbe(root, context, input, now) {
  exactObject(input, ["index", "nonce", "probe", "status", "state"]);
  const { parseNoiseStatusV2 } = await import("./device-v2.mjs"); parseNoiseStatusV2(input.status);
  const claim = await proof(root, `cycle-${input.index}.probe-claim.json`), c = claim.value;
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last.state, false);
  check(input.nonce === c.nonce && last.sequence > c.beforeSequence && now >= c.atHostMs && now - c.atHostMs <= 60000 &&
    input.status.state === "idle" && input.status.observation.bootOrdinal === c.connection.bootOrdinal &&
    input.status.observation.transportEpoch === c.connection.transportEpoch &&
    JSON.stringify(input.state) === JSON.stringify(last.state) && JSON.stringify(input.probe) === JSON.stringify(last.state.probe) &&
    input.probe.requestPayloadBytes === 65536 && input.probe.responsePayloadBytes === 65536, "noise_probe_join");
  await writeNew(resolve(root, `cycle-${input.index}.probe.json`), { schema: "noise-serial-probe-v2", contextSha256: digest(JSON.stringify(context)),
    index: input.index, claimSha256: claim.sha256, beforeSequence: c.beforeSequence, afterSequence: last.sequence,
    atHostMs: now, nonce: input.nonce, probe: input.probe, connection: input.status.observation, clientSha256: context.client_sha256 });
  return { probe_recorded: true };
}
export async function inspectProbe(root, context, index, rows) {
  const claim = await proof(root, `cycle-${index}.probe-claim.json`), result = await proof(root, `cycle-${index}.probe.json`);
  const c = claim.value, p = result.value;
  check(p.schema === "noise-serial-probe-v2" && c.schema === "noise-serial-probe-claim-v2" && p.contextSha256 === digest(JSON.stringify(context)) &&
    c.contextSha256 === p.contextSha256 && p.index === index && c.index === index && p.claimSha256 === claim.sha256 &&
    p.nonce === c.nonce && p.beforeSequence === c.beforeSequence && p.afterSequence > p.beforeSequence &&
    p.atHostMs >= c.atHostMs && p.atHostMs - c.atHostMs <= 60000 && p.clientSha256 === context.client_sha256 &&
    c.clientSha256 === context.client_sha256 && p.connection.bootOrdinal === c.connection.bootOrdinal &&
    p.connection.transportEpoch === c.connection.transportEpoch &&
    JSON.stringify(rows[p.afterSequence - 1]?.state.probe) === JSON.stringify(p.probe) &&
    p.probe.requestPayloadBytes === 65536 && p.probe.responsePayloadBytes === 65536, "noise_probe_evidence");
  return p;
}

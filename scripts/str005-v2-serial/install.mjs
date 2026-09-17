import { resolve } from "node:path";
import { missing, nonce } from "../fixed-usb-qualification/contract.mjs";
import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { inspectInstall, readFreshDetector } from "../str005-noise-serial/install.mjs";
import { flashArguments } from "../str005-noise-serial/operator-execution.mjs";
import { processSnapshot, requireGone, requireNoHolders, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { canonical, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { parseStatus } from "./device.mjs";
import { bytes, check, digest, object, sha256, uint } from "./values.mjs";

function admittedIndex(context, index) { check(Number.isInteger(index) && context.install_indices.includes(index), "v2_install_index"); }
async function predecessorPhysical(context) {
  const root = context.predecessor.root, sealed = (await proof(root, "sealed-inventory.json")).value;
  const claim = await proof(root, "install-0.claim.json");
  check(sealed.files.some((file) => file.path === "install-0.claim.json" && file.sha256 === claim.sha256), "v2_predecessor_physical_proof");
  return claim.value.detector.physical;
}

/** Installation facts reuse qualified continuity schemas; they make no Noise-protocol claim. */
export async function claimInstall(root, context, index, failed, operations = {}) {
  admittedIndex(context, index); check(!failed(), "v2_terminal_failure");
  await missing(resolve(root, `install-${index}`)); await missing(resolve(root, "start.claim.json"));
  await missing(resolve(root, "issuance.claim.json"));
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state, true);
  check(last.state.renewalsConfirmed === 0, "v2_install_after_work");
  const accounting = (await proof(root, "accounting-before-install.json")).value;
  check(last.sequence > accounting.observedSequence && last.state.preservation.baseline_id === accounting.state.preservation.baseline_id,
    "v2_install_baseline");
  const position = context.install_indices.indexOf(index), first = context.install_indices[0];
  if (position > 0) {
    const prior = context.install_indices[position - 1]; await proof(root, `install-${prior}.review.json`);
    if (prior > 0) await proof(root, `cycle-${prior}.json`);
  }
  const detected = await readFreshDetector(root, index, operations);
  const physical = position > 0 ? (await proof(root, `install-${first}.claim.json`)).value.detector.physical : await predecessorPhysical(context);
  check(detected.physical === physical, "v2_physical_device_changed");
  const owner = await proof(root, `install-${index}.host-root.json`), armed = await proof(root, `install-${index}.observer-armed.json`);
  check(sameProcess(owner.value, armed.value), "v2_install_not_armed");
  const processes = await (operations.processSnapshot ?? processSnapshot)();
  check(processes.some((row) => sameProcess(row, owner.value)), "v2_install_owner_missing");
  const argv = flashArguments(root, context, index, detected.port);
  check(!failed(), "v2_terminal_failure");
  const claim = { schema: "noise-serial-install-claim-v2", contextSha256: sha256(JSON.stringify(context)), index,
    beforeSequence: last.sequence, beforeStateSha256: sha256(canonical(last)), atUnixMs: (operations.unixNow ?? Date.now)(),
    detector: detected, ownerSha256: owner.sha256, armedSha256: armed.sha256, argv };
  await writeNew(resolve(root, `install-${index}.claim.json`), claim);
  check(!failed(), "v2_terminal_failure");
  return { install_claimed: true, index, program: "just", argv, context_sha256: claim.contextSha256,
    claim_sha256: sha256(`${JSON.stringify(claim, null, 2)}\n`) };
}
export async function reviewInstall(root, context, index, now, operations = {}) {
  admittedIndex(context, index);
  const reviewed = await inspectInstall(root, context, index);
  await requireGone(reviewed.owners, operations); requireNoHolders(reviewed.claim.detector.port, operations);
  await writeNew(resolve(root, `install-${index}.review.json`), { schema: "str005-v2-install-review-v1",
    contextSha256: sha256(JSON.stringify(context)), index, beforeSequence: reviewed.claim.beforeSequence, atHostMs: now,
    flashSha256: reviewed.flashSha256, observationSha256: reviewed.observationSha256,
    startupCaptureSha256: reviewed.startupCaptureSha256, startupMemory: reviewed.startupMemory });
  return { install_verified: true, index };
}

function probeObservation(status, scope) {
  const parsed = parseStatus(status); check(parsed.state === "idle" && parsed.scope === scope, "v2_probe_idle");
  const { bootOrdinal, workerGeneration, serialTransportEpoch, observedAtUs, clockValid } = parsed.observation;
  return { bootOrdinal, workerGeneration, serialTransportEpoch, observedAtUs, clockValid };
}
function validateProbeObservation(value) {
  object(value, ["bootOrdinal", "workerGeneration", "serialTransportEpoch", "observedAtUs", "clockValid"]);
  for (const key of ["bootOrdinal", "workerGeneration", "serialTransportEpoch", "observedAtUs"]) uint(value[key]);
  check(value.bootOrdinal > 0 && value.clockValid === true, "v2_probe_clock");
}
export async function claimProbe(root, context, index, status, now) {
  check(Number.isInteger(index) && index >= 1 && index <= 4, "v2_probe_index");
  const observation = probeObservation(status, context.scope), review = (await proof(root, `install-${index}.review.json`)).value;
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state);
  check(last.phase === "candidate" && last.sequence > review.beforeSequence && last.atHostMs > review.atHostMs, "v2_probe_reconnect");
  const value = { schema: "str005-v2-probe-claim-v1", contextSha256: sha256(JSON.stringify(context)), index,
    nonce: nonce(), beforeSequence: last.sequence, atHostMs: now, observation, clientSha256: context.client_sha256 };
  await writeNew(resolve(root, `cycle-${index}.probe-claim.json`), value);
  return { probe_nonce: value.nonce };
}
export async function completeProbe(root, context, input, now) {
  object(input, ["index", "nonce", "probe", "status", "state"]);
  check(Number.isInteger(input.index) && input.index >= 1 && input.index <= 4, "v2_probe_index");
  const observation = probeObservation(input.status, context.scope), claim = await proof(root, `cycle-${input.index}.probe-claim.json`), c = claim.value;
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state);
  check(input.nonce === c.nonce && last.sequence > c.beforeSequence && now >= c.atHostMs && now - c.atHostMs <= 60000 &&
    observation.observedAtUs > c.observation.observedAtUs &&
    ["bootOrdinal", "workerGeneration", "serialTransportEpoch"].every((key) => observation[key] === c.observation[key]) &&
    canonical(input.state) === canonical(last.state) && canonical(input.probe) === canonical(last.state.probe) &&
    input.probe.requestPayloadBytes === 65536 && input.probe.responsePayloadBytes === 65536, "v2_probe_join");
  await writeNew(resolve(root, `cycle-${input.index}.probe.json`), { schema: "str005-v2-probe-v1",
    contextSha256: sha256(JSON.stringify(context)), index: input.index, claimSha256: claim.sha256, beforeSequence: c.beforeSequence,
    afterSequence: last.sequence, atHostMs: now, nonce: input.nonce, probe: input.probe, observation, clientSha256: context.client_sha256 });
  return { probe_recorded: true };
}
export async function inspectProbe(root, context, index, rows) {
  const claim = await proof(root, `cycle-${index}.probe-claim.json`), result = await proof(root, `cycle-${index}.probe.json`);
  const c = claim.value, p = result.value;
  object(c, ["schema", "contextSha256", "index", "nonce", "beforeSequence", "atHostMs", "observation", "clientSha256"]);
  object(p, ["schema", "contextSha256", "index", "claimSha256", "beforeSequence", "afterSequence", "atHostMs", "nonce", "probe", "observation", "clientSha256"]);
  validateProbeObservation(c.observation); validateProbeObservation(p.observation); bytes(c.nonce, 16); digest(c.clientSha256);
  check(c.schema === "str005-v2-probe-claim-v1" && p.schema === "str005-v2-probe-v1" && c.contextSha256 === sha256(JSON.stringify(context)) &&
    p.contextSha256 === c.contextSha256 && p.index === index && c.index === index && p.claimSha256 === claim.sha256 &&
    p.nonce === c.nonce && p.beforeSequence === c.beforeSequence && p.afterSequence > p.beforeSequence &&
    p.atHostMs >= c.atHostMs && p.atHostMs - c.atHostMs <= 60000 && p.clientSha256 === context.client_sha256 &&
    p.observation.observedAtUs > c.observation.observedAtUs &&
    c.clientSha256 === context.client_sha256 && ["bootOrdinal", "workerGeneration", "serialTransportEpoch"].every((key) => p.observation[key] === c.observation[key]) &&
    canonical(rows[p.afterSequence - 1]?.state.probe) === canonical(p.probe) && p.probe.requestPayloadBytes === 65536 &&
    p.probe.responsePayloadBytes === 65536, "v2_probe_evidence");
  return p;
}
export async function recordCycle(root, context, index) {
  check(Number.isInteger(index) && index >= 1 && index <= 4, "v2_cycle_index");
  const review = (await proof(root, `install-${index}.review.json`)).value, rows = await readJournal(root, context);
  const before = rows[review.beforeSequence - 1], after = rows.at(-1), probe = await inspectProbe(root, context, index, rows);
  check(after.sequence > before.sequence && after.atHostMs > review.atHostMs && probe.beforeSequence > before.sequence &&
    probe.afterSequence === after.sequence, "v2_cycle_order");
  baseline(before.state, true); baseline(after.state);
  check(after.state.preservation.baseline_id === before.state.preservation.baseline_id, "v2_cycle_baseline");
  const previous = index > 1 ? (await proof(root, `cycle-${index - 1}.json`)).value.report : undefined;
  const report = validateCycle({ schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit,
    app_elf_sha256: context.app_elf_sha256, baseline_id: after.state.preservation.baseline_id, browser_released: true,
    flash_success: true, runtime_identity_match: true, cleanup_complete: true, device_identity_match: true, settings_match: true,
    authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false }, context, previous);
  await writeNew(resolve(root, `cycle-${index}.json`), { schema: "str005-v2-cycle-v1", contextSha256: sha256(JSON.stringify(context)),
    beforeSequence: before.sequence, afterSequence: after.sequence, installReviewSha256: (await proof(root, `install-${index}.review.json`)).sha256, report });
  return { cycle_verified: true, index };
}

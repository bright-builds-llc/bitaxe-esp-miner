import { basename, resolve } from "node:path";
import { readFile, readdir } from "node:fs/promises";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, protectedPath, verifyInventory } from "../str005-noise-serial/files.mjs";
import { inspectInstall, validateCommandObservation } from "../str005-noise-serial/install.mjs";
import { flashArguments } from "../str005-noise-serial/operator-execution.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { readAccounting } from "./accounting-judge.mjs";
import { inspectInstallOwnershipEvidence } from "./install-successor-ownership.mjs";
import { recheckFailedEvidence } from "./successor-evidence.mjs";
import { check, object, sha256, uint } from "./values.mjs";

export const INSTALL_FAILED_CONTEXT = "031552692b777c324b5633dfb5b1cbbc4625da4cdea1adedcd8c1c3ee0cbd1b1";
export const INSTALL_FAILED_RESULT = "cf219da3a6b6830f1818f03eeb7e3d88b0b912d589a99e6a4a1f8b8b0a02ad54";
export const INSTALL_FAILED_SEAL = "5941810323f06e9bdf86d79bc8c9f4aed6ce3a2f105716153e68b101e7f79bf3";
const CONTEXT_FILE = "f1b6dfbcfeac92246b98f1a6ac5fb1e5f2f249c7df33e1e2fe615468e2467b29";
const INSTALLED = "0d2b6d061edc7aab8957c17764ce77ee82f313c1";
const ELF = "8e766fbadca0678e4db534acb067e7621d7fed29e1a749ce9d806c275559d178";
const ROOT_FILES = new Set(["accounting-before-install.json", "artifact-snapshot.json", "context.json", "failure.json", "final-result.json",
  "host-correction.json", "permission-correction.json", "preflight-inventory.json", "sealed-inventory.json", "judgment-failure.json",
  "parent-cleanup-failure.json", "parent-cleanup-supervisor-observation.json", "parent-cleanup-supervisor.json", "parent-install-permissions.json",
  "server-owner.json", "server.claim.json", ...Array.from({ length: 9 }, (_, i) => `state-${String(i + 1).padStart(4, "0")}.json`),
  ...["claim.json", "detect.host-root.json", "detect.observation.json", "detect.observer-armed.json", "detect.stderr.log", "detect.stdout.log",
    "exit.json", "host-root.json", "observation.json", "observer-armed.json", "stderr.log", "stdout.log"].map(name => `install-0.${name}`)]);
const DIRECTORIES = new Set(["evaluator", "fixture", "install-0", "native", "observer", "parent-observations", "qualified-artifacts"]);

async function pinnedHistory(root, pins) {
  // No ancestry or recorded operational path is interpreted before these immutable byte anchors.
  check(pins.context.sha256 === CONTEXT_FILE && pins.result.sha256 === INSTALL_FAILED_RESULT && pins.seal.sha256 === INSTALL_FAILED_SEAL,
    "v2_install_successor_anchor");
  const context = pins.context.value.context;
  check(sha256(JSON.stringify(context)) === INSTALL_FAILED_CONTEXT && context.firmware_commit === INSTALLED && context.app_elf_sha256 === ELF &&
    context.gate_commit === "e20c0fd52d2216596f904992ffa54fda33be9025", "v2_install_successor_pair");
  const { review } = await import("./finalize.mjs");
  return { context, reviewed: await review(root) };
}
function disposition(context, pins, reviewed, hash) {
  check(context.schema === "str005-v2-serial-context-v3" && context.scope === "channel" && context.hostOrdinal === 3 &&
    pins.context.value.sha256 === hash && canonical(pins.context.value.context) === canonical(context) &&
    reviewed.status === "unverified" && reviewed.outcome === "stop_evidence_incomplete" && reviewed.scope === "channel" &&
    reviewed.hardware_qualified === false && reviewed.result_sha256 === pins.result.sha256 && reviewed.sealed_inventory_sha256 === pins.seal.sha256,
    "v2_install_successor_history");
  const value = pins.result.value;
  object(value, ["schema", "contextSha256", "status", "outcome", "firstFailure", "inputs", "scope"]);
  check(value.schema === "str005-v2-serial-result-v1" && value.contextSha256 === hash && value.status === "unverified" &&
    value.outcome === "stop_evidence_incomplete" && value.scope === "channel" && canonical(value.firstFailure) === canonical({
      source: "supervisor", code: "v2_operation_failed", cause: null, sourceSequence: null, observedAtHostMs: 703528,
      availableCauses: { device: null, fixture: null }, ordering: "recorded-first-observation" }), "v2_install_successor_disposition");
}
async function noLaterEvidence(root) {
  for (const entry of await readdir(root, { withFileTypes: true }))
    check(entry.isDirectory() ? DIRECTORIES.has(entry.name) : entry.isFile() && ROOT_FILES.has(entry.name), "v2_install_successor_unexpected_evidence");
  check(canonical((await readdir(resolve(root, "install-0"))).sort()) === canonical(["flash-command-evidence.json", "flash-monitor.log"]),
    "v2_install_successor_install_inventory");
  await missing(resolve(root, "projection.json"));
}
async function failureEvidence(root, hash) {
  const failure = await proof(root, "failure.json"), f = failure.value;
  object(f, ["schema", "contextSha256", "code", "atHostMs", "deviceCause", "sourceSequence"]);
  check(f.schema === "str005-v2-first-failure-v1" && f.contextSha256 === hash && f.code === "v2_operation_failed" && f.atHostMs === 703528 &&
    f.deviceCause === null && f.sourceSequence === null, "v2_install_successor_first_failure");
  const judgment = (await proof(root, "judgment-failure.json")).value;
  check(canonical(judgment) === canonical({ schema: "str005-v2-judgment-failure-v1", code: "v2_recorded_failure", nativeCode: null }),
    "v2_install_successor_judgment");
  return failure;
}
async function permissionEvidence(root, hash, failure, files) {
  const p = (await proof(root, "parent-install-permissions.json")).value;
  object(p, ["schema", "source", "contextSha256", "installation", "originalFailureSha256", "readOnlyDiagnosis", "permissionRepairOnly", "acceptanceRestored", "entries"]);
  check(p.schema === "str005-v2-parent-install-permissions-v1" && p.source === "parent-observed" && p.contextSha256 === hash && p.installation === 0 &&
    p.originalFailureSha256 === failure.sha256 && p.readOnlyDiagnosis === "private_path_policy" && p.permissionRepairOnly === true && p.acceptanceRestored === false &&
    Array.isArray(p.entries) && p.entries.length === 3, "v2_install_successor_permission_provenance");
  check(canonical(p.entries[0]) === canonical({ path: "install-0", modeBefore: "0o755", kind: "directory" }), "v2_install_successor_original_modes");
  for (const [index, name] of ["flash-command-evidence.json", "flash-monitor.log"].entries()) {
    const entry = p.entries[index + 1], path = `install-0/${name}`;
    object(entry, ["path", "modeBefore", "kind", "sha256", "length"]); uint(entry.length);
    const sealed = files.filter(file => file.path === path);
    check(entry.path === path && entry.modeBefore === "0o644" && entry.kind === "file" && sealed.length === 1 &&
      entry.sha256 === sealed[0].sha256 && entry.length === sealed[0].length, "v2_install_successor_permission_bytes");
  }
}
function initialJournal(states, accounting, claim) {
  const statuses = ["configured", "configured", "ready", "ready", "closing", "closing", "closed", "closing", "closed"];
  check(states.length === 9 && accounting.observedSequence === 4 && claim.beforeSequence === 7, "v2_install_successor_initial_sequence");
  const initial = states[3], closed = states[6]; baseline(initial.state); baseline(closed.state, true);
  check(canonical(initial.state) === canonical(accounting.state) && claim.beforeStateSha256 === sha256(canonical(closed)) &&
    closed.atHostMs < 703528 && states[7].atHostMs > 703528, "v2_install_successor_initial_join");
  for (const [index, row] of states.entries()) {
    const s = row.state;
    check(row.phase === "before" && s.status === statuses[index] && !s.running && !s.heartbeatSuppressed && s.renewalsConfirmed === 0 &&
      !s.failure && !s.serialFailureCategory && !s.ownerResourceFailure, "v2_install_successor_no_work");
    if (index >= 2) {
      check(s.preservation?.baseline_id === initial.state.preservation.baseline_id && s.preservation.device_identity_match &&
        s.preservation.settings_match && s.preservation.authorization_high_water_match && !s.preservation.mine_on_boot && s.qualification &&
        ["generation", "work_dispatched", "submitted", "accepted", "rejected", "nonce_work_correlations"].every(key => s.qualification[key] === 0) &&
        s.qualification.budget_reserved_ms === initial.state.qualification.budget_reserved_ms, "v2_install_successor_counter_changed");
    }
    if (index >= 4) check(!s.connected, "v2_install_successor_postinstall_admission");
  }
}
async function detectorJoin(root, context, installed) {
  const claim = installed.claim, detector = claim.detector;
  check(canonical(claim.argv) === canonical(flashArguments(root, context, 0, detector.port)), "v2_install_successor_arguments");
  const path = resolve(root, "install-0.detect.stdout.log"); await protectedPath(path);
  const data = await readFile(path); check(data.length <= 1048576 && sha256(data) === detector.logSha256, "v2_install_successor_detector_bytes");
  const lines = [...data.toString("utf8").matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  for (const [key, expected] of [["port", detector.port], ["usb_profile", "serial_jtag_runtime"], ["physical_identity_sha256", detector.physical]]) {
    const matches = lines.filter(row => row[1] === key);
    check(matches.length === 1 && matches[0][2] === expected, "v2_install_successor_detector_identity");
  }
  const observed = await proof(root, "install-0.detect.observation.json"); validateCommandObservation(observed.value);
  check(observed.sha256 === detector.observationSha256 && claim.atUnixMs >= observed.value.finished_at_unix_ms &&
    claim.atUnixMs - observed.value.finished_at_unix_ms <= 60000, "v2_install_successor_detector_freshness");
  const ancestorClaim = await proof(context.predecessor.root, "install-0.claim.json"), ancestorSeal = (await proof(context.predecessor.root, "sealed-inventory.json")).value;
  check(ancestorSeal.files.some(row => row.path === "install-0.claim.json" && row.sha256 === ancestorClaim.sha256) &&
    detector.physical === ancestorClaim.value.detector.physical, "v2_install_successor_physical_ancestry");
}

/** Exact available facts after a healthy write; never a repaired baseline or historical acceptance. */
export async function inspectFailedInstallChannel(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root) && basename(root) === "channel-003", "v2_install_successor_root");
  await privateRoot(root);
  const pins = { context: await proof(root, "context.json"), result: await proof(root, "final-result.json"), seal: await proof(root, "sealed-inventory.json") };
  const { context, reviewed } = await (operations.inspectHistoricalInstallChannel ?? pinnedHistory)(root, pins);
  const hash = sha256(JSON.stringify(context)); disposition(context, pins, reviewed, hash);
  check(root === resolve(context.firmware_root, "scratch/str005-v2-serial/channel-003"), "v2_install_successor_namespace");
  const seal = pins.seal.value; object(seal, ["schema", "contextSha256", "files"]);
  check(seal.schema === "str005-v2-serial-seal-v1" && seal.contextSha256 === hash, "v2_install_successor_seal");
  await verifyInventory(root, seal.files, new Set(["sealed-inventory.json", "projection.json"]));
  await noLaterEvidence(root);
  const failure = await failureEvidence(root, hash); await permissionEvidence(root, hash, failure, seal.files);
  const states = await readJournal(root, context), accounting = await readAccounting(root, context, states, "before-install");
  const installed = await inspectInstall(root, context, 0); initialJournal(states, accounting, installed.claim);
  await detectorJoin(root, context, installed);
  const ownership = await inspectInstallOwnershipEvidence(root, context, states, installed);
  const observed = { root, context, contextSha256: hash, resultSha256: pins.result.sha256, sealSha256: pins.seal.sha256, inspectedInputs: seal.files,
    beforeSource: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 }, predecessor: structuredClone(context.predecessor),
    initialAccounting: { path: "accounting-before-install.json", sha256: (await proof(root, "accounting-before-install.json")).sha256,
      observedSequence: accounting.observedSequence, ledger: accounting.ledger, original: accounting.original_budget }, afterBaselineObserved: false, ownership };
  await recheckFailedEvidence(observed); return observed;
}

import { basename, dirname, resolve } from "node:path";
import { readdir } from "node:fs/promises";
import { canonical, privateRoot, proof, verifyInventory } from "../str005-noise-serial/files.mjs";
import { inspectInstall } from "../str005-noise-serial/install.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { readAccounting } from "./accounting-judge.mjs";
import { recheckFailedEvidence } from "./successor-evidence.mjs";
import { inspectShareOwnershipEvidence } from "./share-successor-ownership.mjs";
import { check, object, sha256 } from "./values.mjs";

export const SHARE_FAILED_CONTEXT = "e96e658911fd438e222ba7d0088a8d8720e6542c4a93ed7a43914cea92a7b79e";
export const SHARE_FAILED_RESULT = "da7d9aa759a071febd83ec062fb046f360affe6c790dbc33d27ee65cc506c994";
export const SHARE_FAILED_SEAL = "0449a33a703c9ccab27c92ed573156912449d9f67feabd8fb59d5c36aabc8aa1";
export const SHARE_CHANNEL_RESULT = "68997b3cb71933a8aae0715f2f659e0d0c5253cfc9f33e95437572efc449b907";
export const SHARE_CHANNEL_SEAL = "16fd7dddce80530c3b2f1b4b0b94ce6592a51c99ee5008b75bc4c602ab95beea";
const CONTEXT_FILE = "1b605cafa53546010dd7232e1e5d90ef6894744988bad55dcddeddd5c2fe9806";
const ROOT_FILES = new Set(["accounting-before-install.json", "artifact-snapshot.json", "context.json", "final-result.json",
  "host-correction.json", "permission-correction.json", "preflight-inventory.json", "sealed-inventory.json", "judgment-failure.json",
  "server-owner.json", "server.claim.json", "signer-01.exit.json", ...Array.from({ length: 7 }, (_, i) => `state-${String(i + 1).padStart(4, "0")}.json`),
  ...["claim.json", "detect.host-root.json", "detect.observation.json", "detect.observer-armed.json", "detect.stderr.log", "detect.stdout.log",
    "exit.json", "host-root.json", "observation.json", "observer-armed.json", "stderr.log", "stdout.log", "review.json"].map(name => `install-1.${name}`)]);
const DIRECTORIES = new Set(["evaluator", "fixture", "install-1", "native", "observer", "parent-observations", "qualified-artifacts"]);

async function pinnedHistory(root, pins) {
  check(pins.context.sha256 === CONTEXT_FILE && pins.result.sha256 === SHARE_FAILED_RESULT && pins.seal.sha256 === SHARE_FAILED_SEAL,
    "v2_share_successor_anchor");
  check((await proof(dirname(root), "qualification-ordinal-18.json")).sha256 ===
    "a75ffc361c41209981ed90230c6e8945da0ba00e36d6d584f5d69e42c89fe13b", "v2_share_successor_ordinal_marker");
  const context = pins.context.value.context;
  check(sha256(JSON.stringify(context)) === SHARE_FAILED_CONTEXT && context.firmware_commit === "0a9b29bf96b4d4797b46128f0d3850755f5637f2" &&
    context.app_elf_sha256 === "c24bcfee9a2feb9f3932ce7b7f1f81cc012360420b47329f4f5c11bded218d52" &&
    context.predecessor.resultSha256 === SHARE_CHANNEL_RESULT && context.predecessor.sealSha256 === SHARE_CHANNEL_SEAL,
  "v2_share_successor_pair");
  return { context, reviewed: await (await import("./finalize.mjs")).review(root) };
}
export async function inspectShareNoLaterEvidence(root) {
  for (const entry of await readdir(root, { withFileTypes: true }))
    check(entry.isDirectory() ? DIRECTORIES.has(entry.name) : entry.isFile() && ROOT_FILES.has(entry.name), "v2_share_successor_unexpected_evidence");
  check(canonical((await readdir(resolve(root, "install-1"))).sort()) === canonical(["flash-command-evidence.json", "flash-monitor.log"]),
    "v2_share_successor_install_inventory");
}
export function validateShareSigner(value, hash) {
  object(value, ["schema", "contextSha256", "index", "operation", "observation"]);
  const o = value.observation;
  object(o, ["pid", "code", "signal", "elapsedMs", "stdoutBytes", "stderrBytes", "overflow", "inputFailed"]);
  check(value.schema === "str005-v2-signer-exit-v1" && value.contextSha256 === hash && value.index === 1 && value.operation === "public-trust" &&
    Number.isSafeInteger(o.pid) && o.pid > 0 && o.code === 0 && o.signal === null && o.elapsedMs === 83 && o.stdoutBytes === 712 &&
    o.stderrBytes === 0 && o.overflow === false && o.inputFailed === false, "v2_share_successor_signer");
}
export function validateShareInitialJournal(states, accounting, claim) {
  const statuses = ["configured", "configured", "ready", "ready", "closing", "closing", "closed"];
  check(states.length === 7 && accounting.observedSequence === 4 && claim.beforeSequence === 7, "v2_share_successor_sequence");
  const initial = states[3], closed = states[6]; baseline(initial.state); baseline(closed.state, true);
  check(canonical(initial.state) === canonical(accounting.state) && claim.beforeStateSha256 === sha256(canonical(closed)), "v2_share_successor_initial_join");
  for (const [index, row] of states.entries()) {
    const s = row.state;
    check(row.phase === "before" && s.status === statuses[index] && !s.running && !s.heartbeatSuppressed && s.renewalsConfirmed === 0 &&
      !s.failure && !s.serialFailureCategory && !s.ownerResourceFailure, "v2_share_successor_no_work");
    if (index >= 2) check(s.preservation?.baseline_id === initial.state.preservation.baseline_id && s.preservation.device_identity_match &&
      s.preservation.settings_match && s.preservation.authorization_high_water_match && !s.preservation.mine_on_boot && s.qualification &&
      ["generation", "work_dispatched", "submitted", "accepted", "rejected", "nonce_work_correlations"].every(key => s.qualification[key] === 0) &&
      s.qualification.budget_reserved_ms === initial.state.qualification.budget_reserved_ms, "v2_share_successor_counter_changed");
  }
}
/** Exact interruption facts, never an accepted share or a reconstructed restoration. */
export async function inspectFailedShare(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root) && basename(root) === "share-001", "v2_share_successor_root");
  await privateRoot(root);
  const pins = { context: await proof(root, "context.json"), result: await proof(root, "final-result.json"), seal: await proof(root, "sealed-inventory.json") };
  const { context, reviewed } = await (operations.inspectHistoricalShare ?? pinnedHistory)(root, pins), hash = sha256(JSON.stringify(context));
  check(context.schema === "str005-v2-serial-context-v4" && context.scope === "share" && context.hostOrdinal === 1 && pins.context.value.sha256 === hash &&
    reviewed.status === "unverified" && reviewed.outcome === "stop_evidence_incomplete" && reviewed.scope === "share" && !reviewed.hardware_qualified &&
    reviewed.result_sha256 === pins.result.sha256 && reviewed.sealed_inventory_sha256 === pins.seal.sha256, "v2_share_successor_history");
  check(root === resolve(context.firmware_root, "scratch/str005-v2-serial/share-001"), "v2_share_successor_namespace");
  check(canonical(pins.result.value.firstFailure) === canonical({ source: "judge", code: "v2_evidence_incomplete", cause: null, sourceSequence: null,
    observedAtHostMs: null, availableCauses: { device: null, fixture: null }, ordering: "no-producer-cause" }), "v2_share_successor_disposition");
  const seal = pins.seal.value; object(seal, ["schema", "contextSha256", "files"]);
  check(seal.schema === "str005-v2-serial-seal-v1" && seal.contextSha256 === hash, "v2_share_successor_seal");
  await verifyInventory(root, seal.files, new Set(["sealed-inventory.json"])); await inspectShareNoLaterEvidence(root);
  validateShareSigner((await proof(root, "signer-01.exit.json")).value, hash);
  const states = await readJournal(root, context), accounting = await readAccounting(root, context, states, "before-install");
  const installed = await inspectInstall(root, context, 1); validateShareInitialJournal(states, accounting, installed.claim);
  const review = (await proof(root, "install-1.review.json")).value;
  check(review.contextSha256 === hash && review.index === 1 && review.beforeSequence === installed.claim.beforeSequence &&
    review.flashSha256 === installed.flashSha256 && review.observationSha256 === installed.observationSha256, "v2_share_successor_install_review");
  const marker = await proof(dirname(root), "qualification-ordinal-18.json");
  check(canonical(marker.value) === canonical({ schema: "str005-v2-serial-assignment-v1", root, scope: "share", context_sha256: hash }),
    "v2_share_successor_ordinal_marker");
  const ownership = await inspectShareOwnershipEvidence(root, context, states, installed);
  const observed = { root, context, contextSha256: hash, resultSha256: pins.result.sha256, sealSha256: pins.seal.sha256, inspectedInputs: seal.files,
    beforeSource: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 }, predecessor: structuredClone(context.predecessor),
    initialAccounting: { path: "accounting-before-install.json", sha256: (await proof(root, "accounting-before-install.json")).sha256,
      observedSequence: accounting.observedSequence, ledger: accounting.ledger, original: accounting.original_budget }, afterBaselineObserved: false,
    ordinalMarker: { path: "qualification-ordinal-18.json", sha256: marker.sha256, length: marker.bytes.length }, ownership };
  await recheckFailedEvidence(observed); return observed;
}

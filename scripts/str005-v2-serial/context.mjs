import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE, PAGE, canonicalBase64, cleanPushed, fileDigest, ignored, missing, nonce, packageSnapshot } from "../fixed-usb-qualification/contract.mjs";
import { validateAttempt } from "../fixed-usb-qualification/iterative-contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { attemptName, requireActiveTask } from "./contract.mjs";
import { inspectSources, nativeInterface, readContractBinding, requireNativeReadiness, sourceInventory } from "./context-sources.mjs";
import { inspectPredecessor, requirePredecessorBinding } from "./predecessor.mjs";
import { createSnapshot, verifySnapshot } from "./snapshot.mjs";
import { check, object, SCOPES, sha256 } from "./values.mjs";

export const SCHEMA = "str005-v2-serial-context-v1";
const hostMarker = (root, scope, ordinal) => resolve(dirname(root), `${scope}-ordinal-${ordinal}.json`);
const qualificationMarker = (root, ordinal) => resolve(dirname(root), `qualification-ordinal-${ordinal}.json`);
async function predecessorFor(path, scope, operations) { return (operations.inspectPredecessor ?? inspectPredecessor)(path, scope, operations); }
function validatePolicy(context) {
  const keys = ["schema", "manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments", "contracts", "contractSha256",
    "firmware_root", "gate_root", "firmware_commit", "gate_commit", "manifest", "cadence_observer", "observer_build_receipt_sha256", "fixture_binary", "fixture_sha256", "fixture_build_receipt_sha256",
    "gate_bundle_sha256", "gate_page_relative_path", "gate_page_sha256", "trust_sha256", "sdkconfig_sha256", "evaluator", "native_source_files",
    "native_auditor_sources", "client_sha256", "operator_sha256", "scope", "attemptId", "hostOrdinal", "predecessor", "before_source", "original_campaign_id", "install_indices",
    ...(Object.hasOwn(context ?? {}, "native_readiness") ? ["native_readiness"] : []),
    ...(context?.scope === "share" ? ["qualificationAttempt", "expectedLedgerBefore"] : [])];
  object(context, keys);
  check(context?.schema === SCHEMA && SCOPES.includes(context.scope) && canonicalBase64(context.attemptId, 16) &&
    context.install_indices?.join(",") === (context.scope === "channel" ? "0,1,2,3,4" : "1,2,3,4"), "v2_context_policy");
  // This implementation admits the initial run; later runs need concrete verified-progress evidence, not a numeric policy cap.
  check(context.hostOrdinal === 1, "v2_retry_progress_unverified");
  if (context.scope === "channel") check(!Object.hasOwn(context, "qualificationAttempt") && !Object.hasOwn(context, "expectedLedgerBefore"), "v2_channel_no_allowance");
  else {
    validateAttempt(context.qualificationAttempt);
    check(context.qualificationAttempt.id === context.attemptId && context.qualificationAttempt.ordinal === 18 &&
      context.qualificationAttempt.purpose === "normal" && context.qualificationAttempt.maximumActiveMilliseconds === 180000 &&
      canonical(context.expectedLedgerBefore) === canonical({ schema: "worker-qualification-ledger-v1", next_ordinal: 18, last_completed_ordinal: 17, total_charged_ms: 1560000, pending: false }),
      "v2_qualification_policy");
  }
}
/** Read-only prerequisites precede both exclusive assignments; failures never refund host claims. */
export async function preflight(options, operations = {}) {
  check(options.authorityDirectory === undefined && options.poolCredentials === undefined && options.wifiCredentials === undefined, "v2_preflight_credentials_forbidden");
  const root = resolve(options.privateRoot), parent = await privateRoot(dirname(root));
  const hostOrdinal = attemptName(root, options.scope); await missing(root);
  check(hostOrdinal === 1, "v2_retry_progress_unverified");
  const native = await nativeInterface(operations), source = await inspectSources(options, native, operations);
  check(parent === resolve(source.firmware_root, "scratch/str005-v2-serial"), "v2_namespace");
  (operations.ignored ?? ignored)(source.firmware_root, root);
  const predecessor = await predecessorFor(resolve(options.predecessorReceipt), options.scope, operations);
  check(predecessor.root !== root, "v2_predecessor_cycle");
  const attemptId = nonce();
  const context = { schema: SCHEMA, ...source, scope: options.scope, attemptId, hostOrdinal,
    predecessor: { root: predecessor.root, resultSha256: predecessor.resultSha256, sealSha256: predecessor.sealSha256 },
    before_source: { firmware_commit: predecessor.context.firmware_commit, app_elf_sha256: predecessor.context.app_elf_sha256 },
    original_campaign_id: predecessor.context.original_campaign_id,
    install_indices: options.scope === "channel" ? [0, 1, 2, 3, 4] : [1, 2, 3, 4],
    ...(options.scope === "share" ? { qualificationAttempt: { schema: "worker-qualification-attempt-v1", id: attemptId, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 },
      expectedLedgerBefore: predecessor.ledger } : {}) };
  validatePolicy(context); requirePredecessorBinding(context, predecessor);
  check(canonicalBase64(context.original_campaign_id, 16), "v2_original_campaign");
  context.native_readiness = await native.inspect({ firmwareRoot: context.firmware_root, manifestPath: context.manifest,
    expectedSourceCommit: context.firmware_commit, expectedElfSha256: context.app_elf_sha256 });
  requireNativeReadiness(context, context.native_readiness);
  // Recheck publication, task and generated inputs after the auditor, before assignment.
  check(canonical(await inspectSources(options, native, operations)) === canonical(source), "v2_source_changed_during_preflight");
  const marker = { schema: "str005-v2-serial-assignment-v1", root, scope: context.scope, context_sha256: sha256(JSON.stringify(context)) };
  await missing(hostMarker(root, context.scope, hostOrdinal));
  if (context.scope === "share") await missing(qualificationMarker(root, context.qualificationAttempt.ordinal));
  await writeNew(hostMarker(root, context.scope, hostOrdinal), marker);
  if (context.scope === "share") await writeNew(qualificationMarker(root, context.qualificationAttempt.ordinal), marker);
  await (operations.beforeCreate ?? (() => {}))();
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: marker.context_sha256 });
  await createSnapshot(root, context);
  return { ready: true, scope: context.scope, context_sha256: marker.context_sha256, device_effects: false, hardware_qualified: false };
}
export async function loadContext(root, { historical = false, operations = {} } = {}) {
  root = await privateRoot(root);
  const stored = (await proof(root, "context.json")).value; object(stored, ["context", "sha256"]);
  const context = stored.context; validatePolicy(context);
  check(stored.sha256 === sha256(JSON.stringify(context)) && attemptName(root, context.scope) === context.hostOrdinal, "v2_context_integrity");
  const paths = [hostMarker(root, context.scope, context.hostOrdinal)];
  if (context.scope === "share") paths.push(qualificationMarker(root, context.qualificationAttempt.ordinal));
  for (const path of paths) {
    const marker = (await proof(dirname(root), path)).value;
    object(marker, ["schema", "root", "scope", "context_sha256"]);
    check(marker.schema === "str005-v2-serial-assignment-v1" && marker.root === root && marker.scope === context.scope && marker.context_sha256 === stored.sha256,
      "v2_assignment_changed");
  }
  await verifySnapshot(root, context);
  check(context.predecessor.root !== root, "v2_predecessor_cycle");
  const predecessor = await predecessorFor(resolve(context.predecessor.root, "final-result.json"), context.scope, operations);
  requirePredecessorBinding(context, predecessor);
  if (!historical) {
    for (const name of ["final-result.json", "sealed-inventory.json", "failed-inventory.json", "failure.json"]) await missing(resolve(root, name));
    const native = await nativeInterface(operations);
    const current = await inspectSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root, manifest: context.manifest, fixtureBinary: context.fixture_binary }, native, operations);
    for (const key of Object.keys(current)) check(canonical(current[key]) === canonical(context[key]), "v2_live_source_drift");
  }
  return context;
}
/** Bounded gate after full serve admission, not repeated native disassembly or ancestor review. */
export async function verifyEffectInputs(context, operations = {}) {
  validatePolicy(context);
  (operations.cleanPushed ?? cleanPushed)(context.firmware_root, context.firmware_commit);
  (operations.cleanPushed ?? cleanPushed)(context.gate_root, context.gate_commit);
  requireActiveTask(await readFile(resolve(context.firmware_root, "TASKS.md"), "utf8"));
  check(canonical(await readContractBinding(context.firmware_root)) === canonical({ contracts: context.contracts, contractSha256: context.contractSha256 }), "v2_contract_changed");
  const packaged = await packageSnapshot(context.firmware_root, context.manifest, context.firmware_commit);
  for (const key of Object.keys(packaged)) check(canonical(packaged[key]) === canonical(context[key]), "v2_effect_package_changed");
  check(await fileDigest(resolve(context.gate_root, BUNDLE)) === context.gate_bundle_sha256 && await fileDigest(resolve(context.gate_root, PAGE)) === context.gate_page_sha256 &&
    await fileDigest(context.fixture_binary) === context.fixture_sha256 && await fileDigest(context.cadence_observer.path) === context.cadence_observer.sha256 &&
    await fileDigest(resolve(dirname(context.cadence_observer.path), "v2-observer-build-identity.json")) === context.observer_build_receipt_sha256 &&
    await fileDigest(resolve(dirname(context.fixture_binary), "v2-serial-build-identity.json")) === context.fixture_build_receipt_sha256 &&
    await fileDigest(resolve(dirname(context.manifest), "bitaxe-firmware.sdkconfig")) === context.sdkconfig_sha256, "v2_effect_artifact_changed");
  const current = await sourceInventory(context.firmware_root, [...context.native_source_files, ...context.native_auditor_sources]);
  check(canonical(current) === canonical(context.evaluator), "v2_effect_source_changed");
}
export async function recheckNative(context, operations = {}) {
  const native = await nativeInterface(operations);
  const observed = await native.inspect({ firmwareRoot: context.firmware_root, manifestPath: context.manifest,
    expectedSourceCommit: context.firmware_commit, expectedElfSha256: context.app_elf_sha256 });
  requireNativeReadiness(context, observed);
  check(canonical(observed) === canonical(context.native_readiness), "v2_native_changed");
  return observed;
}

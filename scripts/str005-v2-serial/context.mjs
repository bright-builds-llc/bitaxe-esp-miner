import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { BUNDLE, PAGE, canonicalBase64, cleanPushed, fileDigest, ignored, missing, nonce, packageSnapshot } from "../fixed-usb-qualification/contract.mjs";
import { validateAttempt } from "../fixed-usb-qualification/iterative-contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { attemptName, requireActiveTask } from "./contract.mjs";
import { inspectSources, nativeInterface, readContractBinding, requireNativeReadiness, sourceInventory } from "./context-sources.mjs";
import { inspectPredecessor, requirePredecessorBinding } from "./predecessor.mjs";
import { createSnapshot, verifySnapshot } from "./snapshot.mjs";
import { check, digest, object, SCOPES, sha256, CONTRACT_PATH, CONTRACT_SHA256, AMENDMENT_PATH, AMENDMENT_SHA256,
  PERMISSION_AMENDMENT_PATH, PERMISSION_AMENDMENT_SHA256 } from "./values.mjs";
import { checkPermissionCorrection, validatePermissionCorrection } from "./permission-correction.mjs";

export const LEGACY_SCHEMA = "str005-v2-serial-context-v1";
export const SCHEMA = "str005-v2-serial-context-v2";
const hostMarker = (root, scope, ordinal) => resolve(dirname(root), `${scope}-ordinal-${ordinal}.json`);
const qualificationMarker = (root, ordinal) => resolve(dirname(root), `qualification-ordinal-${ordinal}.json`);
async function predecessorFor(path, scope, operations) { return (operations.inspectPredecessor ?? inspectPredecessor)(path, scope, operations); }
function validatePolicy(context) {
  const keys = ["schema", "manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments", "contracts", "contractSha256",
    "firmware_root", "gate_root", "firmware_commit", "gate_commit", "manifest", "cadence_observer", "observer_build_receipt_sha256", "fixture_binary", "fixture_sha256", "fixture_build_receipt_sha256",
    "gate_bundle_sha256", "gate_page_relative_path", "gate_page_sha256", "trust_sha256", "sdkconfig_sha256", "evaluator", "native_source_files",
    "native_auditor_sources", "client_sha256", "operator_sha256", "scope", "attemptId", "hostOrdinal", "predecessor", "before_source", "original_campaign_id", "install_indices",
    ...(context?.schema === SCHEMA ? ["permissionSupersession"] : []),
    ...(Object.hasOwn(context ?? {}, "native_readiness") ? ["native_readiness"] : []),
    ...(context?.scope === "share" ? ["qualificationAttempt", "expectedLedgerBefore"] : [])];
  object(context, keys);
  check([LEGACY_SCHEMA, SCHEMA].includes(context?.schema) && SCOPES.includes(context.scope) && canonicalBase64(context.attemptId, 16) &&
    context.install_indices?.join(",") === (context.scope === "channel" ? "0,1,2,3,4" : "1,2,3,4"), "v2_context_policy");
  // The original closed ordinal-one shape remains historical evidence only.
  if (context.schema === LEGACY_SCHEMA) check(context.hostOrdinal === 1, "v2_retry_progress_unverified");
  else {
    check(context.hostOrdinal === (context.scope === "channel" ? 2 : 1), "v2_retry_progress_unverified");
    if (context.scope === "share") check(context.permissionSupersession === null, "v2_permission_scope");
    else permissionMetadata(context.permissionSupersession);
    const expected = { base: { path: CONTRACT_PATH, sha256: CONTRACT_SHA256 }, amendment: { path: AMENDMENT_PATH, sha256: AMENDMENT_SHA256 },
      permission: { path: PERMISSION_AMENDMENT_PATH, sha256: PERMISSION_AMENDMENT_SHA256 } };
    check(canonical(context.contracts) === canonical(expected) && context.contractSha256 === sha256(canonical(expected)), "v2_contract_changed");
  }
  if (context.scope === "channel") check(!Object.hasOwn(context, "qualificationAttempt") && !Object.hasOwn(context, "expectedLedgerBefore"), "v2_channel_no_allowance");
  else {
    validateAttempt(context.qualificationAttempt);
    check(context.qualificationAttempt.id === context.attemptId && context.qualificationAttempt.ordinal === 18 &&
      context.qualificationAttempt.purpose === "normal" && context.qualificationAttempt.maximumActiveMilliseconds === 180000 &&
      canonical(context.expectedLedgerBefore) === canonical({ schema: "worker-qualification-ledger-v1", next_ordinal: 18, last_completed_ordinal: 17, total_charged_ms: 1560000, pending: false }),
      "v2_qualification_policy");
  }
}
function permissionMetadata(value) {
  object(value, ["failedRoot", "failedContextSha256", "closurePath", "closureSha256"]);
  digest(value.failedContextSha256); digest(value.closureSha256);
  check(typeof value.failedRoot === "string" && value.failedRoot === resolve(value.failedRoot) && basename(value.failedRoot) === "channel-001" &&
    value.closurePath === `${value.failedRoot}.permission-closure.json`, "v2_permission_path");
}
async function reviewedPermission(path, operations) {
  const inspect = operations.inspectPermissionClosure ?? (await import("./permission-closure.mjs")).inspectPermissionClosure;
  return inspect(path, operations);
}
function permissionBinding(context, closure) {
  const expectedRoot = resolve(context.firmware_root, "scratch/str005-v2-serial/channel-001");
  check(closure.classification === "closed_no_device_admission" && closure.hardware_qualified === false &&
    closure.root === expectedRoot && closure.closurePath === `${expectedRoot}.permission-closure.json` &&
    closure.context.schema === LEGACY_SCHEMA && closure.context.scope === "channel" && closure.context.hostOrdinal === 1 &&
    closure.contextSha256 === sha256(JSON.stringify(closure.context)) &&
    canonical(closure.predecessor) === canonical(context.predecessor) &&
    canonical(closure.context.predecessor) === canonical(context.predecessor), "v2_permission_predecessor");
  check(context.firmware_commit !== closure.context.firmware_commit && context.gate_commit !== closure.context.gate_commit &&
    context.gate_bundle_sha256 !== closure.context.gate_bundle_sha256, "v2_permission_pair_unchanged");
  const binding = { failedRoot: closure.root, failedContextSha256: closure.contextSha256,
    closurePath: closure.closurePath, closureSha256: closure.closureSha256 };
  permissionMetadata(binding);
  check(canonical(binding) === canonical(context.permissionSupersession), "v2_permission_closure_changed");
}
async function verifyPermissionPin(context) {
  const pin = context.permissionSupersession;
  await privateRoot(dirname(pin.closurePath)); await privateRoot(pin.failedRoot);
  await missing(`${pin.closurePath}.pending`);
  const [closure, failed] = await Promise.all([
    proof(dirname(pin.closurePath), pin.closurePath), proof(pin.failedRoot, "context.json"),
  ]);
  object(failed.value, ["context", "sha256"]);
  check(Array.isArray(closure.value.inputs?.attempt), "v2_permission_closure_changed");
  const contextFiles = closure.value.inputs.attempt.filter(row => row.path === "context.json");
  check(contextFiles.length === 1 && contextFiles[0].sha256 === failed.sha256 &&
    contextFiles[0].length === failed.bytes.length, "v2_permission_closure_changed");
  // The full classifier ran before assignment and serving. This only detects
  // changed pinned inputs; it never supplies a substitute historical verdict.
  check(closure.sha256 === pin.closureSha256 && closure.value.schema === "str005-v2-permission-closure-v1" &&
    closure.value.failedRoot === pin.failedRoot && closure.value.failedContextSha256 === pin.failedContextSha256 &&
    failed.value.sha256 === pin.failedContextSha256 && sha256(JSON.stringify(failed.value.context)) === pin.failedContextSha256,
  "v2_permission_closure_changed");
}
function requireNewOrdinal(scope, ordinal, maybeClosure) {
  check(scope === "channel" ? ordinal === 2 && typeof maybeClosure === "string" && maybeClosure === resolve(maybeClosure) :
    scope === "share" && ordinal === 1 && maybeClosure === undefined, "v2_permission_successor_required");
}

/** Read-only prerequisites precede both exclusive assignments; failures never refund host claims. */
export async function preflight(options, operations = {}) {
  check(options.authorityDirectory === undefined && options.poolCredentials === undefined && options.wifiCredentials === undefined, "v2_preflight_credentials_forbidden");
  const root = resolve(options.privateRoot), parent = await privateRoot(dirname(root));
  const hostOrdinal = attemptName(root, options.scope); await missing(root);
  requireNewOrdinal(options.scope, hostOrdinal, options.supersedePermission);
  const native = await nativeInterface(operations), source = await inspectSources(options, native, operations);
  check(parent === resolve(source.firmware_root, "scratch/str005-v2-serial"), "v2_namespace");
  (operations.ignored ?? ignored)(source.firmware_root, root);
  const predecessor = await predecessorFor(resolve(options.predecessorReceipt), options.scope, operations);
  check(predecessor.root !== root, "v2_predecessor_cycle");
  const maybeClosure = options.scope === "channel" ? await reviewedPermission(options.supersedePermission, operations) : null;
  const attemptId = nonce();
  const context = { schema: SCHEMA, ...source, scope: options.scope, attemptId, hostOrdinal,
    predecessor: { root: predecessor.root, resultSha256: predecessor.resultSha256, sealSha256: predecessor.sealSha256 },
    before_source: { firmware_commit: predecessor.context.firmware_commit, app_elf_sha256: predecessor.context.app_elf_sha256 },
    original_campaign_id: predecessor.context.original_campaign_id,
    permissionSupersession: maybeClosure ? { failedRoot: maybeClosure.root, failedContextSha256: maybeClosure.contextSha256,
      closurePath: maybeClosure.closurePath, closureSha256: maybeClosure.closureSha256 } : null,
    install_indices: options.scope === "channel" ? [0, 1, 2, 3, 4] : [1, 2, 3, 4],
    ...(options.scope === "share" ? { qualificationAttempt: { schema: "worker-qualification-attempt-v1", id: attemptId, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 },
      expectedLedgerBefore: predecessor.ledger } : {}) };
  validatePolicy(context); requirePredecessorBinding(context, predecessor);
  if (maybeClosure) permissionBinding(context, maybeClosure);
  else check(predecessor.context.schema === SCHEMA && predecessor.context.hostOrdinal === 2 && predecessor.context.permissionSupersession !== null,
    "v2_channel_successor_required");
  check(canonicalBase64(context.original_campaign_id, 16), "v2_original_campaign");
  context.native_readiness = await native.inspect({ firmwareRoot: context.firmware_root, manifestPath: context.manifest,
    expectedSourceCommit: context.firmware_commit, expectedElfSha256: context.app_elf_sha256 });
  requireNativeReadiness(context, context.native_readiness);
  const correction = context.scope === "channel" ? await checkPermissionCorrection(context, operations) :
    validatePermissionCorrection((await proof(predecessor.root, "permission-correction.json")).value, context);
  validatePermissionCorrection(correction, context);
  // Recheck publication, task and generated inputs after the auditor, before assignment.
  check(canonical(await inspectSources(options, native, operations)) === canonical(source), "v2_source_changed_during_preflight");
  if (maybeClosure) permissionBinding(context, await reviewedPermission(options.supersedePermission, operations));
  const marker = { schema: "str005-v2-serial-assignment-v1", root, scope: context.scope, context_sha256: sha256(JSON.stringify(context)) };
  await missing(hostMarker(root, context.scope, hostOrdinal));
  if (context.scope === "share") await missing(qualificationMarker(root, context.qualificationAttempt.ordinal));
  await writeNew(hostMarker(root, context.scope, hostOrdinal), marker);
  if (context.scope === "share") await writeNew(qualificationMarker(root, context.qualificationAttempt.ordinal), marker);
  await (operations.beforeCreate ?? (() => {}))();
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: marker.context_sha256 });
  await createSnapshot(root, context, { permissionCorrection: correction });
  return { ready: true, scope: context.scope, context_sha256: marker.context_sha256, device_effects: false, hardware_qualified: false };
}
export async function loadContext(root, { historical = false, operations = {} } = {}) {
  root = await privateRoot(root);
  const stored = (await proof(root, "context.json")).value; object(stored, ["context", "sha256"]);
  const context = stored.context; validatePolicy(context);
  if (!historical) check(context.schema === SCHEMA, "v2_legacy_context_read_only");
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
  if (context.schema === SCHEMA) {
    if (context.scope === "channel") permissionBinding(context, await reviewedPermission(context.permissionSupersession.closurePath, operations));
    else check(predecessor.context.schema === SCHEMA && predecessor.context.hostOrdinal === 2 && predecessor.context.permissionSupersession !== null,
      "v2_channel_successor_required");
  }
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
  validatePolicy(context); check(context.schema === SCHEMA, "v2_legacy_context_read_only");
  if (context.scope === "channel") await verifyPermissionPin(context);
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

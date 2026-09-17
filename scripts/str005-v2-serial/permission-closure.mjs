import { execFileSync } from "node:child_process";
import { link, readFile, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, git, missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireGone, requireLsofAbsent, requireNoHolders } from "../str005-noise-serial/host-resources.mjs";
import { checkedOwner } from "./host-resources.mjs";
import { checkPermissionCorrection, validatePermissionCorrection, CORRECTION_CHECKER } from "./permission-correction.mjs";
import { closurePath, inspectPermissionInputs, PERMISSION_PATH, PERMISSION_SHA256 } from "./permission-closure-inputs.mjs";
import { check, object, sha256, uint } from "./values.mjs";

const SCHEMA = "str005-v2-permission-closure-v1";
const PRODUCERS = ["scripts/str005-v2-serial/permission-closure.mjs", "scripts/str005-v2-serial/permission-closure-inputs.mjs"];
const HERE = dirname(fileURLToPath(import.meta.url));
const NONCLAIMS = ["permission-selection-outcome", "fresh-device-identity", "fresh-settings", "fresh-baseline", "fresh-accounting",
  "device-charge-or-refund", "channel-acceptance", "mining-acceptance", "continuation-authority"];

async function publishedBytes(root, commit, path, operations) {
  if (operations.readPublishedSource) return operations.readPublishedSource(root, commit, path);
  try { return execFileSync("git", ["-C", root, "show", `${commit}:${path}`], { maxBuffer: 1048576, timeout: 10000, stdio: ["ignore", "pipe", "pipe"] }); }
  catch { check(false, "v2_permission_published_source"); }
}
async function checkCreator(value, oldContext, operations) {
  object(value, ["correction", "validatorSources"]);
  const correction = value.correction;
  check(/^[a-f0-9]{40}$/u.test(correction.firmwareCommit) && /^[a-f0-9]{40}$/u.test(correction.gateCommit) &&
    correction.firmwareCommit !== oldContext.firmware_commit && correction.gateCommit !== oldContext.gate_commit &&
    correction.gateBundleSha256 !== oldContext.gate_bundle_sha256,
  "v2_permission_correction_pair");
  validatePermissionCorrection(correction, { firmware_commit: correction.firmwareCommit, gate_commit: correction.gateCommit,
    gate_bundle_sha256: correction.gateBundleSha256, evaluator: [{ path: CORRECTION_CHECKER, sha256: correction.checkerSha256 }] });
  check(Array.isArray(value.validatorSources) && value.validatorSources.length === PRODUCERS.length, "v2_permission_validator_sources");
  for (const [index, row] of value.validatorSources.entries()) {
    object(row, ["path", "sha256", "length"]);
    check(row.path === PRODUCERS[index], "v2_permission_validator_sources");
    const bytes = await publishedBytes(oldContext.firmware_root, correction.firmwareCommit, row.path, operations);
    check(row.sha256 === sha256(bytes) && row.length === bytes.length, "v2_permission_validator_changed");
  }
  check(sha256(await publishedBytes(oldContext.firmware_root, correction.firmwareCommit, CORRECTION_CHECKER, operations)) === correction.checkerSha256 &&
    sha256(await publishedBytes(oldContext.firmware_root, correction.firmwareCommit, PERMISSION_PATH, operations)) === PERMISSION_SHA256,
  "v2_permission_amendment_binding");
  const pins = (await publishedBytes(oldContext.firmware_root, correction.firmwareCommit, "MODULE.bazel", operations)).toString("utf8");
  const matches = [...pins.matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(matches.length === 1 && matches[0][1] === correction.gateCommit, "v2_permission_published_gate");
}
async function creator(context, operations) {
  const source = { firmware_root: context.firmware_root, gate_root: context.gate_root,
    firmware_commit: (operations.git ?? git)(context.firmware_root, ["rev-parse", "HEAD"]),
    gate_commit: (operations.git ?? git)(context.gate_root, ["rev-parse", "HEAD"]),
    gate_bundle_sha256: sha256(await readFile(resolve(context.gate_root, BUNDLE))) };
  const correction = await checkPermissionCorrection(source, operations), validatorSources = [];
  for (const path of PRODUCERS) {
    const bytes = await readFile(resolve(context.firmware_root, path));
    check(sha256(bytes) === sha256(await readFile(resolve(HERE, path.split("/").at(-1)))), "v2_permission_running_validator_changed");
    validatorSources.push({ path, sha256: sha256(bytes), length: bytes.length });
  }
  const result = { correction, validatorSources }; await checkCreator(result, context, operations); return result;
}
function evidenceFacts(observed) {
  return { scope: "channel", failedHostOrdinal: 1, boundary: "before_native_open", stateCount: 6,
    detectorInspectionObserved: true, deviceBaseline: "not_collected", deviceAccounting: "not_collected",
    browserClosureSource: "separate_parent_cua_observation", savedOperatorBrowserBranchUsed: false,
    supervisorExitCode: 0, supervisorStopToCloseMs: observed.exited.exitedAtMs - observed.exited.stopRequestedAtMs };
}
function result(observed, path, hash) {
  return { root: observed.root, closurePath: path, closureSha256: hash, context: observed.context, contextSha256: observed.contextSha256,
    predecessor: structuredClone(observed.context.predecessor), classification: "closed_no_device_admission", status: "unverified", hardware_qualified: false };
}
async function freshAbsence(observed, operations) {
  await requireGone([observed.server.owner], operations);
  requireLsofAbsent(["-nP", `-iTCP:${observed.server.port}`, "-sTCP:LISTEN", "-t"], operations);
  requireNoHolders(observed.serialPort, operations);
  const observedAtUnixMs = (operations.now ?? Date.now)(); uint(observedAtUnixMs);
  check(observedAtUnixMs >= observed.resources.observedAtUnixMs, "v2_permission_collection_clock");
  return { schema: "str005-v2-permission-resource-check-v1", source: "closure-collector", observedAtUnixMs,
    owner: checkedOwner(observed.server.owner), supervisorAbsent: true, supervisorListenerAbsent: true, detectorSerialPairAbsent: true };
}
async function publishClosure(path, value, operations) {
  const pending = `${path}.pending`;
  await writeNew(pending, value);
  await (operations.beforeClosurePublish ?? (() => {}))();
  // The visible receipt appears only after complete write+sync. Neither link can overwrite evidence.
  await link(pending, path); await unlink(pending);
}

/** The only mutation is one exclusive protected sibling; original evidence and markers are never rewritten. */
export async function closePermission(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_permission_root");
  await privateRoot(dirname(root)); const path = closurePath(root); await missing(path); await missing(`${path}.pending`);
  const observed = await inspectPermissionInputs(root, operations), createdBy = await creator(observed.context, operations);
  const resources = await freshAbsence(observed, operations);
  const repeated = await inspectPermissionInputs(root, operations);
  check(canonical(repeated.inputs) === canonical(observed.inputs), "v2_permission_input_drift");
  const value = { schema: SCHEMA, amendmentSha256: PERMISSION_SHA256, failedRoot: root, failedContextSha256: observed.contextSha256,
    status: "unverified", classification: "closed_no_device_admission", hardwareQualified: false,
    firstObservation: observed.failure, facts: evidenceFacts(observed), nonClaims: NONCLAIMS, inputs: observed.inputs, createdBy, resources };
  await publishClosure(path, value, operations);
  return inspectPermissionClosure(path, operations);
}

/** Historical review checks retained observations, not today's ownership of reusable ports. */
export async function inspectPermissionClosure(path, operations = {}) {
  check(typeof path === "string" && path === resolve(path) && path.endsWith(".permission-closure.json"), "v2_permission_closure_path");
  const root = path.slice(0, -".permission-closure.json".length);
  await privateRoot(dirname(root)); check(path === closurePath(root), "v2_permission_closure_path");
  await missing(`${path}.pending`);
  const stored = await proof(dirname(path), path), value = stored.value;
  object(value, ["schema", "amendmentSha256", "failedRoot", "failedContextSha256", "status", "classification", "hardwareQualified", "firstObservation",
    "facts", "nonClaims", "inputs", "createdBy", "resources"]);
  const observed = await inspectPermissionInputs(root, operations);
  check(value.schema === SCHEMA && value.amendmentSha256 === PERMISSION_SHA256 && value.failedRoot === root &&
    value.failedContextSha256 === observed.contextSha256 && value.status === "unverified" && value.classification === "closed_no_device_admission" &&
    value.hardwareQualified === false && canonical(value.firstObservation) === canonical(observed.failure) &&
    canonical(value.facts) === canonical(evidenceFacts(observed)) && canonical(value.nonClaims) === canonical(NONCLAIMS) &&
    canonical(value.inputs) === canonical(observed.inputs), "v2_permission_closure_binding");
  await checkCreator(value.createdBy, observed.context, operations);
  const resource = value.resources;
  object(resource, ["schema", "source", "observedAtUnixMs", "owner", "supervisorAbsent", "supervisorListenerAbsent", "detectorSerialPairAbsent"]);
  uint(resource.observedAtUnixMs);
  check(resource.schema === "str005-v2-permission-resource-check-v1" && resource.source === "closure-collector" &&
    resource.observedAtUnixMs >= observed.resources.observedAtUnixMs && canonical(resource.owner) === canonical(checkedOwner(observed.server.owner)) &&
    resource.supervisorAbsent === true && resource.supervisorListenerAbsent === true && resource.detectorSerialPairAbsent === true,
  "v2_permission_closure_resources");
  check(canonical((await inspectPermissionInputs(root, operations)).inputs) === canonical(value.inputs) &&
    (await proof(dirname(path), path)).sha256 === stored.sha256, "v2_permission_review_drift");
  return result(observed, path, stored.sha256);
}
export async function reviewPermission(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root), "v2_permission_root");
  return inspectPermissionClosure(closurePath(root), operations);
}

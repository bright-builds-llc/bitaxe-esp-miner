import { resolve } from "node:path";
import { canonical } from "../str005-noise-serial/files.mjs";
import { PREFLIGHT_AMENDMENT, check, object } from "./values.mjs";
import { hex } from "./shapes.mjs";
export function binding(inspected) {
  return { failedRoot: inspected.root, failedContextSha256: inspected.contextSha256, assignmentSha256: inspected.assignment.sha256,
    closurePath: inspected.closurePath, closureSha256: inspected.closureSha256 };
}
export function validateBinding(context) {
  check(canonical(context.preflightAmendment) === canonical(PREFLIGHT_AMENDMENT), "bootstrap_preflight_amendment");
  const value = context.preflightSupersession; object(value, ["failedRoot", "failedContextSha256", "assignmentSha256", "closurePath", "closureSha256"]);
  for (const key of ["failedContextSha256", "assignmentSha256", "closureSha256"]) hex(value[key]);
  check(value.failedRoot === resolve(context.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-001") && value.closurePath === `${value.failedRoot}.preflight-closure.json`, "bootstrap_preflight_supersession");
}
export function requireSupersession(context, inspected) {
  validateBinding(context);
  check(canonical(context.preflightSupersession) === canonical(binding(inspected)) && inspected.effectsObserved === false && inspected.classification === "interrupted_before_effects" &&
    inspected.checkerIdentity.commit === context.package.firmware_commit && canonical(inspected.checkerIdentity.sources) === canonical(context.sourceInventory) &&
    inspected.context.package.firmware_commit !== context.package.firmware_commit && inspected.context.package.manifest_sha256 !== context.package.manifest_sha256 &&
    canonical(inspected.context.sourceInventory) !== canonical(context.sourceInventory) && canonical(inspected.context.beforeSource) === canonical(context.beforeSource) &&
    canonical(inspected.context.originalCampaign) === canonical(context.originalCampaign) && canonical(inspected.context.expectedAccounting) === canonical(context.expectedAccounting), "bootstrap_preflight_supersession");
}
export async function inspectBoundClosure(path, operations, deep) {
  return (operations.inspectPreflightClosure ?? (await import("./preflight-closure.mjs")).inspectPreflightClosure)(path, operations, { deep });
}
export async function recheckBinding(context, operations = {}, { deep = false } = {}) {
  const value = await inspectBoundClosure(context.preflightSupersession.closurePath, operations, deep); requireSupersession(context, value); return value;
}

import { resolve } from "node:path";
import { canonical, privateRoot, proof } from "../str005-noise-serial/files.mjs";
import { BOOTSTRAP_BEFORE, check, object, sha256 } from "./values.mjs";
import { requireIdleLedger, requireExhaustedOriginal } from "../fixed-usb-qualification/iterative-contract.mjs";
export const BOOTSTRAP_PINS = Object.freeze({ failedContextSha256: "dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a", failedResultSha256: "1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e", failedSealSha256: "3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342", acceptedContextSha256: "804f022dbdf4bb1f1f2298558486a5eb659c84fd851ae32a9e5a97cf1c542c78", acceptedResultSha256: "be8339ce97e8e91bf5194076181caa11e9e9407892bf6ab376f8a06fdfbd34c8", acceptedSealSha256: "b9f85846b4c7875f4908cd44dc38067a11b7ac84a99c3c9b57391544ddc7a424" });
export { BOOTSTRAP_BEFORE } from "./values.mjs";
export const bootstrapBinding = firmwareRoot => ({ failedRoot: resolve(firmwareRoot, "scratch/str005-v2-serial/channel-005"), acceptedRoot: resolve(firmwareRoot, "scratch/usb-bootstrap-measure/attempt-004"), ...BOOTSTRAP_PINS });
export function bootstrapMetadata(value, firmwareRoot) {
  object(value, ["failedRoot", "acceptedRoot", ...Object.keys(BOOTSTRAP_PINS)]);
  check(canonical(value) === canonical(bootstrapBinding(firmwareRoot)), "v2_bootstrap_binding");
}
async function anchored(root, prefix) {
  await privateRoot(root);
  const [stored, result, seal] = await Promise.all(["context.json", "final-result.json", "sealed-inventory.json"].map(name => proof(root, name)));
  check(stored.value.sha256 === BOOTSTRAP_PINS[`${prefix}ContextSha256`] && sha256(JSON.stringify(stored.value.context)) === stored.value.sha256 && result.sha256 === BOOTSTRAP_PINS[`${prefix}ResultSha256`] && seal.sha256 === BOOTSTRAP_PINS[`${prefix}SealSha256`], "v2_bootstrap_anchor");
  return { context: stored.value.context, result: result.value, seal: seal.value };
}
async function sealedFile(root, seal, name) {
  const file = await proof(root, name), rows = seal.files.filter(row => row.path === name);
  check(rows.length === 1 && rows[0].sha256 === file.sha256 && rows[0].length === file.bytes.length, "v2_bootstrap_inventory"); return file.value;
}
export async function bootstrapPins(firmwareRoot) {
  const binding = bootstrapBinding(firmwareRoot), failed = await anchored(binding.failedRoot, "failed"), accepted = await anchored(binding.acceptedRoot, "accepted");
  check(failed.context.schema === "str005-v2-serial-context-v5" && failed.context.scope === "channel" && failed.context.hostOrdinal === 5 && failed.result.status === "unverified" &&
    accepted.context.schema === "usb-bootstrap-measure-context-v4" && accepted.result.status === "measurement_complete" && accepted.result.correction.accepted === true &&
    Object.values(accepted.result.correction.checks).every(value => value === true) && accepted.result.cleanup.complete === true && accepted.result.restoration.confirmed === true && accepted.result.capture.qualified === true && accepted.result.capture.exitCode === 0 &&
    accepted.result.hardware_qualified === false && accepted.result.mining_authorized === false && accepted.result.qualification_credit === "none", "v2_bootstrap_outcome");
  const accounting = await sealedFile(binding.acceptedRoot, accepted.seal, "accounting-after.json");
  await sealedFile(binding.acceptedRoot, accepted.seal, "install-0.claim.json");
  requireIdleLedger(accounting.ledger, 18, 1560000); requireExhaustedOriginal(accounting.original);
  check(canonical({ firmware_commit: accepted.context.package.firmware_commit, app_elf_sha256: accepted.context.package.app_elf_sha256 }) === canonical(BOOTSTRAP_BEFORE) &&
    accepted.context.originalCampaign.id === failed.context.original_campaign_id && accounting.state.deviceRestorationConfirmed === true, "v2_bootstrap_baseline");
  return { binding, failed: failed.context, accepted: accepted.context, shareSupersession: failed.context.shareSupersession, beforeSource: BOOTSTRAP_BEFORE, ledger: accounting.ledger, original: accounting.original };
}
export async function inspectBootstrap(firmwareRoot, receipt, operations = {}) {
  check(receipt === resolve(firmwareRoot, "scratch/usb-bootstrap-measure/attempt-004/final-result.json"), "v2_bootstrap_receipt");
  const facts = await bootstrapPins(firmwareRoot);
  await (await import("./finalize.mjs")).review(facts.binding.failedRoot);
  const reviewed = await (await import("../usb-bootstrap-measure/finalize.mjs")).review(facts.binding.acceptedRoot);
  check(reviewed.correction_accepted === true, "v2_bootstrap_review");
  await (await import("../usb-bootstrap-measure/restored-predecessor.mjs")).inspectRestoredPredecessor(resolve(firmwareRoot, "scratch/usb-bootstrap-measure/attempt-003"), operations);
  await (await import("../usb-bootstrap-measure/measurement-predecessor.mjs")).inspectMeasurementPredecessor(resolve(firmwareRoot, "scratch/usb-bootstrap-measure/attempt-002"), operations);
  await currentBootstrapOwnership(facts, operations); return facts;
}
export async function currentBootstrapOwnership(facts, operations = {}) {
  const v2 = await import("./cleanup.mjs"), bootstrap = await import("../usb-bootstrap-measure/cleanup.mjs");
  await v2.requireHostStopped(facts.binding.failedRoot, facts.failed, operations);
  await (await import("./operator-evidence.mjs")).requireOperatorStopped(facts.binding.failedRoot, facts.failed, operations);
  await bootstrap.requireHostStopped(facts.binding.acceptedRoot, facts.accepted, operations); await bootstrap.operatorAbsent(facts.binding.acceptedRoot, facts.accepted, operations);
}
export async function requireBootstrapBinding(context, operations = {}) {
  bootstrapMetadata(context.bootstrapCorrection, context.firmware_root);
  const facts = await (operations.bootstrapPins ?? bootstrapPins)(context.firmware_root);
  check(canonical(context.bootstrapCorrection) === canonical(facts.binding) && canonical(context.shareSupersession) === canonical(facts.shareSupersession) && context.original_campaign_id === facts.failed.original_campaign_id &&
    context.firmware_commit !== facts.failed.firmware_commit && context.manifest_sha256 !== facts.failed.manifest_sha256, "v2_bootstrap_pair");
  if (context.scope === "channel") check(canonical(context.before_source) === canonical(facts.beforeSource), "v2_bootstrap_before");
  return facts;
}

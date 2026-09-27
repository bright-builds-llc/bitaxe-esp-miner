import { resolve } from "node:path";
import { canonical, privateRoot, proof } from "../str005-noise-serial/files.mjs";
import { verifyAccounting } from "./journal.mjs";
import { BEFORE_V3, MEASUREMENT_002, check, sha256 } from "./values.mjs";
export async function measurementPins(root) {
  root = await privateRoot(root);
  const [context, result, seal, accounting] = await Promise.all([proof(root, "context.json"), proof(root, "final-result.json"), proof(root, "sealed-inventory.json"), proof(root, "accounting-after.json")]);
  check(context.value.sha256 === MEASUREMENT_002.contextSha256 && sha256(JSON.stringify(context.value.context)) === MEASUREMENT_002.contextSha256 &&
    result.sha256 === MEASUREMENT_002.resultSha256 && seal.sha256 === MEASUREMENT_002.sealSha256, "bootstrap_measurement_predecessor");
  const prior = context.value.context;
  check(prior.schema === "usb-bootstrap-measure-context-v2" && root === resolve(prior.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-002") &&
    prior.package.firmware_commit === BEFORE_V3.firmware_commit && prior.package.app_elf_sha256 === BEFORE_V3.app_elf_sha256 &&
    result.value.status === "measurement_complete" && result.value.hardware_qualified === false && result.value.mining_authorized === false &&
    result.value.qualification_credit === "none" && result.value.restoration.confirmed === true && result.value.cleanup.complete === true,
    "bootstrap_measurement_predecessor");
  const rows = seal.value.files.filter(row => row.path === "accounting-after.json");
  check(rows.length === 1 && rows[0].sha256 === accounting.sha256 && rows[0].length === accounting.bytes.length, "bootstrap_measurement_predecessor");
  return { context: prior, binding: { root, ...MEASUREMENT_002 }, beforeSource: BEFORE_V3,
    originalCampaign: { id: prior.originalCampaign.id, record: { path: "accounting-after.json", sha256: accounting.sha256, length: accounting.bytes.length } },
    expectedAccounting: { ledger: accounting.value.ledger, original: accounting.value.original } };
}
export async function inspectMeasurementPredecessor(root, operations = {}) {
  const prior = await measurementPins(root);
  const reviewed = await (await import("./finalize.mjs")).review(root, operations);
  check(reviewed.status === "measurement_complete" && reviewed.resultSha256 === MEASUREMENT_002.resultSha256 && reviewed.sealSha256 === MEASUREMENT_002.sealSha256,
    "bootstrap_measurement_predecessor");
  await verifyAccounting(root, prior.context); return prior;
}
export async function recheckMeasurement(context, operations = {}) {
  const prior = await (operations.measurementPins ?? measurementPins)(context.predecessor.root);
  check(canonical(prior.binding) === canonical(context.predecessor) && canonical(prior.beforeSource) === canonical(context.beforeSource) &&
    canonical(prior.originalCampaign) === canonical(context.originalCampaign) && canonical(prior.expectedAccounting) === canonical(context.expectedAccounting) &&
    context.package.firmware_commit !== prior.context.package.firmware_commit && context.package.manifest_sha256 !== prior.context.package.manifest_sha256 &&
    canonical(context.sourceInventory) !== canonical(prior.context.sourceInventory), "bootstrap_correction_pair");
  return prior;
}

import { basename, resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, verifyInventory } from "../str005-noise-serial/files.mjs";
import { readDeviceJournal, readJournal, baseline } from "./journal.mjs";
import { judgeAccounting, requireChannelNoWork } from "./accounting-judge.mjs";
import { inspectExecution } from "./execution-inspect.mjs";
import { judgeV2Protocol } from "./protocol-judge.mjs";
import { inspectAvailableContinuity } from "./successor-continuity.mjs";
import { inspectOwnershipEvidence } from "./successor-ownership.mjs";
import { check, object, sha256 } from "./values.mjs";

export const FAILED_RESULT = "675230a92e4f6ebd78ca0cf1748019c364772b5f31742075dda3c3c38af7b4b0";
export const FAILED_SEAL = "a12a3c25cc89c137d12ece4bc80c4896546f7f2d37bfaabf36109004b2b26c80";
export const FAILED_CONTEXT = "b2c889e617a1e1f976c9dfc1fd25a889edb62a515fbcbdc88c7ccf3df2fdb179";
const CONTEXT_FILE = "73e9da9065f00e00223691e45636f247686d4fca9d82840f5b67bd55083b3d60";

async function readPinnedHistory(root, pins) {
  check(pins.result.sha256 === FAILED_RESULT && pins.seal.sha256 === FAILED_SEAL && pins.context.sha256 === CONTEXT_FILE,
    "v2_successor_failure_anchor");
  const { review } = await import("./finalize.mjs");
  const reviewed = await review(root), context = pins.context.value.context;
  check(sha256(JSON.stringify(context)) === FAILED_CONTEXT && context.firmware_commit === "097050c09e9def6a9bf57df254a597a0f287bfa1" &&
    context.app_elf_sha256 === "76639dc29251dcc3001164e159fd2ca8db142796df46c88b07b25345ae9d0498" &&
    context.gate_commit === "e20c0fd52d2216596f904992ffa54fda33be9025", "v2_successor_installed_pair");
  return { context, reviewed };
}

/** Revalidate immutable bytes within a request without replaying already verified ancestry. */
export async function recheckFailedEvidence(observed) {
  const { root } = observed;
  await privateRoot(root);
  const contextFile = observed.inspectedInputs.filter(row => row.path === "context.json");
  check(contextFile.length === 1, "v2_successor_context_member");
  const [context, result, seal] = await Promise.all([proof(root, "context.json"), proof(root, "final-result.json"), proof(root, "sealed-inventory.json")]);
  check(context.sha256 === contextFile[0].sha256 && context.bytes.length === contextFile[0].length &&
    context.value.sha256 === observed.contextSha256 && result.sha256 === observed.resultSha256 && seal.sha256 === observed.sealSha256,
    "v2_successor_evidence_drift");
  await verifyInventory(root, observed.inspectedInputs, new Set(["sealed-inventory.json", "projection.json"]));
  await missing(resolve(root, "projection.json"));
}

/** Pin the one sealed failure before interpreting any ancestry or recorded operational path. */
export async function inspectFailedChannel(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root) && basename(root) === "channel-002", "v2_successor_failed_root");
  await privateRoot(root);
  const pins = { context: await proof(root, "context.json"), result: await proof(root, "final-result.json"), seal: await proof(root, "sealed-inventory.json") };
  const historical = operations.inspectHistoricalChannel ? await operations.inspectHistoricalChannel(root, pins) : await readPinnedHistory(root, pins);
  const { context, reviewed } = historical, contextSha256 = sha256(JSON.stringify(context));
  check(context.schema === "str005-v2-serial-context-v2" && context.scope === "channel" && context.hostOrdinal === 2 &&
    root === resolve(context.firmware_root, "scratch/str005-v2-serial/channel-002") && pins.context.value.sha256 === contextSha256 &&
    canonical(pins.context.value.context) === canonical(context) && reviewed.status === "unverified" && reviewed.outcome === "stop_evidence_incomplete" &&
    reviewed.scope === "channel" && reviewed.hardware_qualified === false && reviewed.result_sha256 === pins.result.sha256 &&
    reviewed.sealed_inventory_sha256 === pins.seal.sha256, "v2_successor_historical_result");
  const seal = pins.seal.value, result = pins.result.value;
  object(seal, ["schema", "contextSha256", "files"]);
  check(seal.schema === "str005-v2-serial-seal-v1" && seal.contextSha256 === contextSha256, "v2_successor_seal");
  await verifyInventory(root, seal.files, new Set(["sealed-inventory.json", "projection.json"]));
  await missing(resolve(root, "projection.json")); await missing(resolve(root, "failure.json"));
  const judgment = (await proof(root, "judgment-failure.json")).value;
  check(judgment.schema === "str005-v2-judgment-failure-v1" && judgment.code === "v2_baseline" && judgment.nativeCode === null &&
    result.status === "unverified" && result.outcome === "stop_evidence_incomplete" && result.contextSha256 === contextSha256 &&
    canonical(result.firstFailure) === canonical({ source: "judge", code: "v2_baseline", cause: null, sourceSequence: null,
      observedAtHostMs: null, availableCauses: { device: null, fixture: null }, ordering: "no-producer-cause" }), "v2_successor_original_disposition");
  const states = await readJournal(root, context), devices = await readDeviceJournal(root, context);
  const execution = await inspectExecution(root, context, states, devices), protocol = judgeV2Protocol(execution.protocolInput);
  const accounting = await judgeAccounting(root, context, states, execution.restoration);
  await inspectAvailableContinuity(root, context, states, accounting);
  await requireChannelNoWork(root, states, accounting.before, accounting.after);
  check(states.every(row => !row.state.failure && !row.state.serialFailureCategory && !row.state.ownerResourceFailure), "v2_successor_conflicting_failure");
  baseline(states.at(-1)?.state, true);
  check(accounting.after.state.deviceRestorationConfirmed === true && states.at(-1).state.deviceRestorationConfirmed === true &&
    accounting.after.state.expectedFirmwareSourceCommit === context.firmware_commit &&
    accounting.after.state.expectedAppElfSha256 === context.app_elf_sha256 && protocol.acceptedProtocol === true &&
    protocol.counts.connections === 1 && protocol.counts.submitted === 0, "v2_successor_restored_image");
  const ownership = await inspectOwnershipEvidence(root, context, states, execution.protocolInput);
  await verifyInventory(root, seal.files, new Set(["sealed-inventory.json", "projection.json"]));
  check((await proof(root, "context.json")).sha256 === pins.context.sha256 && (await proof(root, "final-result.json")).sha256 === pins.result.sha256 &&
    (await proof(root, "sealed-inventory.json")).sha256 === pins.seal.sha256, "v2_successor_evidence_drift");
  return { root, context, contextSha256, resultSha256: pins.result.sha256, sealSha256: pins.seal.sha256, inspectedInputs: seal.files,
    beforeSource: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 },
    predecessor: structuredClone(context.predecessor), ledger: accounting.after.ledger, original: accounting.after.original_budget, ownership };
}

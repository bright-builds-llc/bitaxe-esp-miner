import { validateHostTimingV2 } from "./host-timing-v2.mjs";
import { correctionJudgment } from "./correction-judge.mjs";
import { verifyCaptureExit } from "./capture-exit.mjs";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, inventory, privateRoot, proof, retain, verifyInventory, writeNew } from "../str005-noise-serial/files.mjs";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { contextHash, load } from "./context.mjs";
import { inspectCapture } from "./install.mjs";
import { deriveCleanup, validateCleanup, requireHostStopped, operatorAbsent, maybeProof } from "./cleanup.mjs";
import { verifyAccounting } from "./journal.mjs";
import { judgeOperator } from "./operator-disposition.mjs";
import { CONTEXT_V3, hostTimingFile, check, object, schema, sha256, code } from "./values.mjs";

async function observation(root, context) {
  const capture = await inspectCapture(root, context); await verifyCaptureExit(root, context);
  const { validateHostTiming } = await import("./host-timing.mjs");
  const { parseDeviceObservations } = await import("./device-observations.mjs");
  const host = await proof(root, `install-0/${hostTimingFile(context)}`); (context.schema === CONTEXT_V3 ? validateHostTimingV2 : validateHostTiming)(host.value);
  check(host.value.physicalIdentityDigest === capture.claim.detector.physical, "bootstrap_timing_physical");
  const device = parseDeviceObservations(capture.log, { firmwareCommit: context.package.firmware_commit, appElfSha256: context.package.app_elf_sha256 });
  return { capture, host, device };
}
async function judgment(root, context, inputs) {
  let maybeHost, maybeDevice, maybeFinalState;
  const result = { schema: context.schema === CONTEXT_V3 ? "usb-bootstrap-measure-result-v2" : schema("result"), contextSha256: contextHash(context), status: "unverified", firstFailure: null,
    capture: { exitCode: null, flashVerdictSha256: null, qualified: false }, measurement: { hostTimingSha256: null, deviceObservationsSha256: null, complete: false },
    restoration: { receiptSha256: null, confirmed: false }, cleanup: { receiptSha256: null, complete: false }, inputs,
    hardware_qualified: false, mining_authorized: false, qualification_credit: "none" };
  const first = await maybeProof(root, "failure.json");
  if (first) { const v = first.value; object(v, ["schema", "contextSha256", "source", "stage", "code", "observationSha256"]);
    check(v.schema === schema("failure") && v.contextSha256 === contextHash(context), "bootstrap_failure_changed");
    result.firstFailure = { source: v.source, stage: v.stage, code: v.code, observationSha256: v.observationSha256 }; }
  try { const capture = await inspectCapture(root, context); result.capture = { exitCode: capture.exitCode, flashVerdictSha256: capture.flashVerdictSha256, qualified: capture.qualified }; }
  catch (error) { result.firstFailure ??= { source: "judge", stage: "capture", code: code(error), observationSha256: null }; }
  try {
    const { capture, host, device } = await observation(root, context); maybeHost = host.value; maybeDevice = device;
    result.capture = { exitCode: capture.exitCode, flashVerdictSha256: capture.flashVerdictSha256, qualified: capture.qualified };
    const saved = await proof(root, "device-observations.json"); check(canonical(saved.value) === canonical(device), "bootstrap_device_observations_changed");
    result.measurement = { hostTimingSha256: host.sha256, deviceObservationsSha256: saved.sha256,
      complete: host.value.captureComplete && host.value.cleanupComplete && !host.value.overflow && !host.value.clockDiscontinuity && host.value.missingStages.length === 0 && host.value.readerReopenCount === 0 && device.complete === true };
    const accounting = await verifyAccounting(root, context); maybeFinalState = accounting.last.state;
    if (context.schema === CONTEXT_V3) check(maybeFinalState.deviceRestorationConfirmed === true && host.value.captureDurationMs === 30000, "bootstrap_correction_restoration");
    result.restoration = { receiptSha256: (await proof(root, "restoration.json")).sha256, confirmed: true };
    const cleanup = await proof(root, "cleanup.json");
    check(cleanup.value.schema === schema("cleanup") && cleanup.value.contextSha256 === contextHash(context), "bootstrap_cleanup_changed");
    result.cleanup = { receiptSha256: cleanup.sha256, complete: await validateCleanup(root, context, cleanup.value, resolve(root, "final-inputs/operator")) };
    await judgeOperator(root, context, resolve(root, "final-inputs/operator"));
    const tolerated = !result.firstFailure || result.firstFailure.code === "bootstrap_capture_unqualified";
    if (result.measurement.complete && result.cleanup.complete && tolerated) result.status = "measurement_complete";
  } catch (error) { result.firstFailure ??= { source: "judge", stage: "review", code: code(error), observationSha256: null }; }
  if (context.schema === CONTEXT_V3) result.correction = correctionJudgment(result, maybeHost, maybeDevice, maybeFinalState);
  return result;
}
export async function finalize(root, operations = {}) {
  root = await privateRoot(root); await missing(resolve(root, "final-result.json")); await missing(resolve(root, "sealed-inventory.json"));
  const context = await load(root, { historical: true, operations, ancestry: true });
  await requireHostStopped(root, context, operations); await operatorAbsent(root, context, operations);
  const cleanup = await deriveCleanup(root, context, operations); await writeNew(resolve(root, "cleanup.json"), cleanup);
  try {
    const files = await inventory(`${root}.operator`);
    for (const file of files) await retain(resolve(root, "final-inputs/operator", file.path), await readFile(resolve(`${root}.operator`, file.path)));
    await verifyInventory(`${root}.operator`, files);
  } catch (e) { if (e.code !== "ENOENT") throw e; }
  try { const { device } = await observation(root, context); await writeNew(resolve(root, "device-observations.json"), device); }
  catch (error) { await writeNew(resolve(root, "measurement-failure.json"), { schema: schema("measurement-failure"), contextSha256: contextHash(context), code: code(error) }); }
  await requireHostStopped(root, context, operations); await operatorAbsent(root, context, operations);
  const inputs = await inventory(root), result = await judgment(root, context, inputs); await writeNew(resolve(root, "final-result.json"), result);
  await writeNew(resolve(root, "sealed-inventory.json"), { schema: schema("seal"), contextSha256: contextHash(context), files: await inventory(root) });
  return review(root, operations);
}
export async function review(root, operations = {}) {
  root = await privateRoot(root); const context = await load(root, { historical: true, operations, ancestry: true });
  const stored = await proof(root, "final-result.json"), seal = await proof(root, "sealed-inventory.json"); object(seal.value, ["schema", "contextSha256", "files"]);
  check(seal.value.schema === schema("seal") && seal.value.contextSha256 === contextHash(context), "bootstrap_seal");
  await verifyInventory(root, seal.value.files, new Set(["sealed-inventory.json"]));
  const inputs = await inventory(root, new Set(["final-result.json", "sealed-inventory.json"]));
  check(canonical(await judgment(root, context, inputs)) === canonical(stored.value), "bootstrap_result_changed");
  return { status: stored.value.status, contextSha256: contextHash(context), resultSha256: stored.sha256, sealSha256: seal.sha256,
    hardware_qualified: false, mining_authorized: false, qualification_credit: "none", ...(context.schema === CONTEXT_V3 ? { correction_accepted: stored.value.correction.accepted } : {}) };
}

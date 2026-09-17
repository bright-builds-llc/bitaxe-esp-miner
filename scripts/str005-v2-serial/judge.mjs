import { readdir } from "node:fs/promises";
import { canonical, inventory, proof, verifyInventory } from "../str005-noise-serial/files.mjs";
import { inspectCleanup } from "./cleanup.mjs";
import { readDeviceJournal, readJournal, restoredBaseline } from "./journal.mjs";
import { judgeV2Protocol } from "./protocol-judge.mjs";
import { judgeContinuity } from "./continuity-judge.mjs";
import { judgeAccounting, judgeIssuance, requireChannelNoWork } from "./accounting-judge.mjs";
import { judgeObserver, judgeShareSafety } from "./safety-judge.mjs";
import { check, sha256 } from "./values.mjs";

const FINAL_FILES = new Set(["final-result.json", "sealed-inventory.json", "projection.json", "judgment-failure.json"]);

/** Full filesystem judgment. A structurally valid protocol alone cannot qualify hardware. */
export async function judge(root, context, cleanupPath, operations = {}) {
  const beforeInventory = await inventory(root, FINAL_FILES);
  check(!(await readdir(root)).includes("failure.json"), "v2_recorded_failure");
  const finalNative = (await proof(root, "native/final-readiness.json")).value;
  check(canonical(finalNative) === canonical(context.native_readiness), "v2_native_final_drift");
  const states = await readJournal(root, context), devices = await readDeviceJournal(root, context);
  check(states.length > 0 && devices.length >= 2, "v2_evidence_incomplete");
  const { inspectExecution } = await import("./execution-inspect.mjs");
  const execution = await inspectExecution(root, context, states, devices);
  const accounting = await judgeAccounting(root, context, states, execution.restoration);
  const continuity = await judgeContinuity(root, context, states, accounting.before);
  check(accounting.initial.observedSequence < states.find(row => row.state.status === "closed")?.sequence,
    "v2_initial_accounting_order");
  const baselineId = accounting.initial.state.preservation.baseline_id;
  check(states.every(row => !row.state.preservation || row.state.preservation.baseline_id === baselineId), "v2_private_baseline_changed");
  const protocol = judgeV2Protocol(execution.protocolInput);
  let maybeSafety = null, maybeObserver = null;
  if (context.scope === "channel") await requireChannelNoWork(root, states, accounting.before, accounting.after);
  else {
    await judgeIssuance(root, context, states, accounting.before, devices[0]);
    maybeSafety = judgeShareSafety(context, states, devices, execution, accounting);
    maybeObserver = await judgeObserver(root, context, states, devices, execution.faultConfirmed);
    check(execution.fault.observerTailMs === maybeObserver.tailMs, "v2_fault_observer_join");
  }
  restoredBaseline(states.at(-1).state, context, true);
  check(states.at(-1).phase === "candidate", "v2_final_candidate_required");
  const cleanup = await inspectCleanup(root, context, cleanupPath, { ...operations, requireSuccessfulExit: true });
  const finalDevice = devices.at(-1).record;
  check(finalDevice.resources.socketClosed && finalDevice.resources.workerQuiescent && !finalDevice.resources.fenceRetained,
    "v2_final_resource_release");
  await verifyInventory(root, beforeInventory, FINAL_FILES);
  return { scope: context.scope, contextSha256: sha256(JSON.stringify(context)), protocol, continuity,
    accounting: { before: accounting.before.ledger, after: accounting.after.ledger }, safety: maybeSafety, observer: maybeObserver,
    cleanupMs: cleanup.cleanupMs, poolTupleProof: "source-bound-live-comparison; tuple intentionally not retained" };
}

/** Failed Share publication may release Channel evidence only with independently proved safe restoration. */
export async function inspectRestoredCleanup(root, context, cleanupPath, operations = {}) {
  const states = await readJournal(root, context), devices = await readDeviceJournal(root, context);
  const { inspectRestoration } = await import("./execution-restoration.mjs");
  await inspectRestoration(root, context, states, devices);
  restoredBaseline(states.at(-1)?.state, context, true);
  return inspectCleanup(root, context, cleanupPath, { ...operations, requireSuccessfulExit: false });
}

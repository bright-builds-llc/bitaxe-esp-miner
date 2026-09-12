import { spawn } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { digest, exactObject, fileDigest, missing, protectedPath, QualificationError, readJson, requireCondition, writeNew } from "./contract.mjs";
import { RECOVERY_SCHEMA } from "./recovery-judge.mjs";

export async function validateRecoveryTrace(input, gateRoot, program = "bun") {
  exactObject(input, ["stage", "source", "trace"]);
  requireCondition(["before", "loss", "recovered", "resumed"].includes(input.stage), "recovery_trace_stage");
  requireCondition(["browser", "device"].includes(input.source), "recovery_trace_source");
  return new Promise((resolveResult, reject) => {
    const child = spawn(program, [resolve(dirname(fileURLToPath(import.meta.url)), "recovery-trace-validator.mjs"),
      resolve(gateRoot, "web/worker-serial-trace-export.ts")], { stdio: ["pipe", "pipe", "pipe"] });
    const chunks = []; let size = 0, failed = false;
    const timer = setTimeout(() => { failed = true; child.kill("SIGKILL"); }, 5000);
    child.stdout.on("data", (chunk) => {
      size += chunk.length;
      if (size > 65536) { failed = true; child.kill("SIGKILL"); }
      else chunks.push(chunk);
    });
    child.stderr.on("data", () => { failed = true; });
    child.stdin.on("error", () => { failed = true; child.kill("SIGKILL"); });
    child.once("error", () => { clearTimeout(timer); reject(new QualificationError("recovery_trace_validator_unavailable")); });
    child.once("close", (code) => {
      clearTimeout(timer);
      if (failed || code !== 0) { reject(new QualificationError("recovery_trace_rejected")); return; }
      try { resolveResult(JSON.parse(Buffer.concat(chunks).toString("utf8"))); }
      catch { reject(new QualificationError("recovery_trace_validator_output")); }
    });
    child.stdin.end(JSON.stringify(input));
  });
}

export async function saveRecoveryTrace(root, context, input, sequence, state, validate) {
  requireCondition(context.schema === RECOVERY_SCHEMA && state, "recovery_trace_scope");
  await missing(resolve(root, "result.json"));
  await missing(resolve(root, "sample-seal-intent.json"));
  const trace = await validate(input);
  const stage = input.stage;
  const phase = context.recovery_phase;
  requireCondition(stage === "before" || (phase === "loss" ? ["loss", "recovered"].includes(stage) : stage === "resumed"), "recovery_trace_stage");
  requireCondition(stage === "loss" ? input.source === "browser" && !state.connected && state.serialOwnershipReleased :
    state.connected && !state.running && state.deviceBaselineConfirmed === true, "recovery_trace_state");
  if (stage === "before") {
    await missing(resolve(root, "issued.json"));
    requireCondition(state.qualification?.attempt?.ordinal !== context.qualification_attempt.ordinal, "recovery_trace_before_work");
  }
  if (stage !== "before") requireCondition(state.qualification?.attempt?.ordinal === context.qualification_attempt.ordinal, "recovery_trace_attempt");
  const value = { schema: "fixed-usb-recovery-trace-v1", context_sha256: digest(JSON.stringify(context)),
    stage, source: input.source, after_sequence: sequence, trace };
  await writeNew(resolve(root, `recovery-trace-${stage}-${input.source}.json`), value);
  return { recovery_trace_saved: true, stage };
}

export async function requireRecoveryTraces(root, context, records, fault) {
  if (context.schema !== RECOVERY_SCHEMA) return [];
  const bindings = [], snapshots = new Map();
  requireCondition(Array.isArray(records) && records.length > 0, "recovery_trace_journal_missing");
  const stages = context.recovery_phase === "loss" ? ["before", "loss", "recovered"] : ["before", "resumed"];
  for (const stage of stages) {
    for (const source of stage === "loss" ? ["browser"] : ["browser", "device"]) {
      const file = `recovery-trace-${stage}-${source}.json`;
      const path = resolve(root, file);
      let value;
      try { await protectedPath(path); value = await readJson(path); }
      catch (error) {
        if (error.code === "ENOENT") throw new QualificationError("recovery_trace_missing");
        throw error;
      }
      requireCondition(value.schema === "fixed-usb-recovery-trace-v1" && value.context_sha256 === digest(JSON.stringify(context)) &&
        value.stage === stage && value.source === source, "recovery_trace_binding");
      if (source === "device") {
        requireCondition(value.trace.snapshotAvailable === true, "recovery_trace_unavailable");
        if (stage === "recovered") requireCondition(value.trace.previous !== null &&
          value.trace.previous?.events?.length > 0, "recovery_prior_epoch_missing");
      }
      const maybeState = records.find(record => record.sequence === value.after_sequence)?.state;
      requireCondition(maybeState, "recovery_trace_journal_binding");
      if (stage === "loss") requireCondition(fault && value.after_sequence >= fault.released_sequence &&
        !maybeState.connected && maybeState.serialOwnershipReleased, "recovery_trace_loss_binding");
      else requireCondition(maybeState.connected && !maybeState.running && maybeState.deviceBaselineConfirmed === true &&
        maybeState.deviceLeaseInactive === true, "recovery_trace_baseline_binding");
      if (stage === "before") requireCondition(maybeState.qualification?.attempt?.ordinal !== context.qualification_attempt.ordinal,
        "recovery_trace_before_work");
      else requireCondition(maybeState.qualification?.attempt?.ordinal === context.qualification_attempt.ordinal,
        "recovery_trace_attempt");
      if (["recovered", "resumed"].includes(stage)) requireCondition(maybeState.qualification.safe_stop_complete,
        "recovery_trace_stop_missing");
      snapshots.set(`${stage}-${source}`, value);
      const linkClosedTrace = stage === "recovered" && source === "device" ? {
        link_closed_verified: ["writer_abandoned", "epoch_revoked"].every(stage =>
          value.trace.previous.events.some(event => event.stage === stage && event.epoch === value.trace.previous.epoch)),
      } : {};
      bindings.push({ stage, source, file, sha256: await fileDigest(path), ...linkClosedTrace });
    }
  }
  const maybeFirstWork = records.find(record => record.state.qualification?.attempt?.ordinal === context.qualification_attempt.ordinal &&
    record.state.running && record.state.qualification.work_dispatched > 0);
  requireCondition(maybeFirstWork && ["browser", "device"].every(source => snapshots.get(`before-${source}`).after_sequence < maybeFirstWork.sequence),
    "recovery_trace_order");
  const terminalStage = context.recovery_phase === "loss" ? "recovered" : "resumed";
  requireCondition(["browser", "device"].every(source => snapshots.get(`${terminalStage}-${source}`).after_sequence > maybeFirstWork.sequence),
    "recovery_trace_order");
  if (context.recovery_phase === "loss") {
    const beforeDevice = snapshots.get("before-device"), recoveredDevice = snapshots.get("recovered-device");
    const lossBrowser = snapshots.get("loss-browser"), recoveredBrowser = snapshots.get("recovered-browser");
    requireCondition(lossBrowser.after_sequence < recoveredBrowser.after_sequence &&
      lossBrowser.after_sequence < recoveredDevice.after_sequence &&
      recoveredDevice.trace.previous.epoch === beforeDevice.trace.current.epoch, "recovery_trace_epoch_binding");
    const browserEpoch = snapshots.get("before-browser").trace.events.at(-1)?.epoch;
    requireCondition(browserEpoch > 0 && lossBrowser.trace.events.some(event => event.epoch === browserEpoch && event.stage === "close_completed") &&
      recoveredBrowser.trace.events.at(-1)?.epoch > browserEpoch, "recovery_trace_browser_epoch");
  }
  return bindings;
}

/** Missing baseline diagnostics stop signing before any pool input or allowance is issued. */
export async function requireBeforeRecoveryTraces(root, context, records) {
  if (context.schema !== RECOVERY_SCHEMA) return;
  for (const source of ["browser", "device"]) {
    const path = resolve(root, `recovery-trace-before-${source}.json`);
    let value;
    try { await protectedPath(path); value = await readJson(path); }
    catch (error) {
      if (error.code === "ENOENT") throw new QualificationError("recovery_before_trace_missing");
      throw error;
    }
    const maybeState = records.find(record => record.sequence === value.after_sequence)?.state;
    requireCondition(value.schema === "fixed-usb-recovery-trace-v1" && value.context_sha256 === digest(JSON.stringify(context)) &&
      value.stage === "before" && value.source === source && maybeState?.connected && !maybeState.running &&
      maybeState.deviceBaselineConfirmed === true && maybeState.deviceLeaseInactive &&
      maybeState.qualification?.attempt?.ordinal !== context.qualification_attempt.ordinal, "recovery_before_trace_binding");
    if (source === "device") requireCondition(value.trace.snapshotAvailable === true, "recovery_trace_unavailable");
  }
}

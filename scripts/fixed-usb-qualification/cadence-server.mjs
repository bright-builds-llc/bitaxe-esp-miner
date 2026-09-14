import { appendFile } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { createCadenceObserver } from "./cadence-observer.mjs";
import { CADENCE_PHASES, requireCadenceTask } from "./cadence-contract.mjs";
import { requireCadenceDiagnostics, requireCadencePhase, validateCadenceProbe } from "./cadence-evidence.mjs";
import { canonicalBase64, digest, exactObject, missing, nonce, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateState } from "./judge.mjs";
import { requireSuccessorBaseline } from "./successor.mjs";

export async function createCadenceRoutes(root, context, current, operations = {}) {
  await requireCadenceTask(context.firmware_root);
  const now = operations.now ?? Date.now;
  const observer = await (operations.createCadenceObserver ?? createCadenceObserver)(root, context);
  let endpointChallenge, endpointGeneration, activePhase, phaseIndex = 0, previousProbe, miningMeasurementEnd, observerResult;
  const completed = [];
  const phaseContext = digest(JSON.stringify(context));
  async function idle(state) {
    validateState(state, context);
    requireCondition(state.connected && !state.running && state.deviceLeaseInactive && !state.failure, "cadence_idle_required");
  }
  async function readyToSign() {
    requireCondition(phaseIndex === 2 && activePhase?.phase === "mining" && observer.status().alive &&
      observer.status().connected && !observer.status().failed, "cadence_before_work_required");
    for (const name of ["idle", "usb"]) {
      const saved = await readJson(resolve(root, `cadence-${name}.json`));
      requireCadenceDiagnostics(context, saved.review);
      requireCadencePhase(saved.review, name);
    }
    requireCondition(previousProbe?.ordinal === 12, "cadence_probes_required");
  }
  async function handle(path, input) {
    if (path === "/cadence/status" && input === undefined) return { observer: observer.status(), phase: activePhase?.phase ?? null, faultObservedAtUnixMs: miningMeasurementEnd?.observedAtUnixMs ?? null };
    if (path === "/cadence/observer-context" && input) {
      exactObject(input, []); await idle(current().lastState);
      requireCondition(phaseIndex === 0 && !activePhase && !observer.status().alive, "cadence_observer_order");
      const binding = current().reviewedBinding;
      requireCondition(binding && binding.expires > now() && canonicalBase64(binding.binding, 32), "cadence_authenticated_binding_required");
      endpointChallenge = { nonce: nonce(), expires: now() + 5000, binding: binding.binding };
      return { nonce: endpointChallenge.nonce };
    }
    if (path === "/cadence/observer/start" && input) {
      exactObject(input, ["nonce", "endpoint", "requestedAtUnixMs", "receivedAtUnixMs", "state"]);
      const challenge = endpointChallenge; endpointChallenge = undefined;
      requireCondition(challenge && challenge.nonce === input.nonce && challenge.expires >= now() &&
        Number.isSafeInteger(input.requestedAtUnixMs) && Number.isSafeInteger(input.receivedAtUnixMs) &&
        input.requestedAtUnixMs >= challenge.expires - 5000 && input.receivedAtUnixMs >= input.requestedAtUnixMs &&
        input.receivedAtUnixMs <= now() && now() - input.requestedAtUnixMs <= 5000, "cadence_endpoint_freshness");
      requireCondition(current().reviewedBinding?.binding === challenge.binding && current().reviewedBinding.expires > now(), "cadence_endpoint_binding_changed");
      await idle(input.state); await requireSuccessorBaseline(root, context, input.state);
      requireCondition(isDeepStrictEqual(input.state, current().lastState), "cadence_endpoint_journal_binding");
      const value = input.endpoint;
      exactObject(value, ["schema", "ipv4", "httpPort", "observedAtUs", "bootOrdinal", "generation", "controlSessionBindingSha256"]);
      requireCondition(value.schema === "worker-telemetry-endpoint-v1" && Number.isSafeInteger(value.observedAtUs) && value.observedAtUs >= 0 &&
        Number.isSafeInteger(value.bootOrdinal) && value.bootOrdinal > 0 && Number.isSafeInteger(value.generation) && value.generation > 0 &&
        canonicalBase64(value.controlSessionBindingSha256, 32) && value.controlSessionBindingSha256 === challenge.binding, "cadence_endpoint_binding");
      endpointGeneration = value.generation;
      return observer.start({ ipv4: value.ipv4, httpPort: value.httpPort });
    }
    if (path === "/cadence/phase/start" && input) {
      exactObject(input, ["arm"]); await idle(current().lastState);
      requireCondition(!activePhase && phaseIndex < 3 && observer.status().alive && observer.status().connected && !observer.status().failed,
        "cadence_phase_order");
      const arm = input.arm;
      exactObject(arm, ["schema", "phase", "armedAtUs", "generation"]);
      requireCondition(arm.schema === "worker-telemetry-cadence-arm-v1" && arm.phase === CADENCE_PHASES[phaseIndex] &&
        Number.isSafeInteger(arm.armedAtUs) && arm.armedAtUs >= 0 && Number.isSafeInteger(arm.generation) && arm.generation >= 0,
      "cadence_arm_shape");
      if (arm.phase === "mining") requireCondition(arm.generation === endpointGeneration, "cadence_endpoint_generation_changed");
      await missing(resolve(root, `cadence-${arm.phase}.json`));
      activePhase = { phase: arm.phase, arm, started_sequence: current().sequence, started_at_unix_ms: now() };
      await writeNew(resolve(root, `cadence-${arm.phase}-arm.json`), { context_sha256: phaseContext, ...activePhase });
      return { cadence_phase_started: arm.phase };
    }
    if (path === "/cadence/probe" && input) {
      requireCondition(activePhase?.phase === "usb" && observer.status().alive && !observer.status().failed, "cadence_probe_phase");
      const probe = validateCadenceProbe(input, previousProbe);
      await appendFile(resolve(root, "cadence-probes.jsonl"), JSON.stringify(probe) + "\n", { mode: 0o600 });
      await appendFile(resolve(root, "cadence-probe-witnesses.jsonl"), JSON.stringify({ ordinal: probe.ordinal, observedAtUnixMs: now(), sequence: current().sequence }) + "\n", { mode: 0o600 });
      previousProbe = probe; return { cadence_probe_saved: true };
    }
    if (path === "/cadence/phase/finish" && input) {
      exactObject(input, ["phase", "review"]); await idle(current().lastState);
      requireCadenceDiagnostics(context, input.review);
      const capturedBeforeClose = input.phase === "mining" && miningMeasurementEnd && observerResult?.cleanupComplete === true &&
        observerResult.reason === "requested" && observerResult.exitCode === 0 && observerResult.closedAtUnixMs >= miningMeasurementEnd.observedAtUnixMs + 5000;
      requireCondition(activePhase?.phase === input.phase && ((observer.status().alive && observer.status().connected && !observer.status().failed) || capturedBeforeClose) &&
        current().sequence > activePhase.started_sequence, "cadence_phase_finish_order");
      const phase = requireCadencePhase(input.review, input.phase);
      requireCondition(phase.armedAtUs === activePhase.arm.armedAtUs && phase.generation === activePhase.arm.generation &&
        isDeepStrictEqual(input.review, current().lastState.cadence?.review), "cadence_review_journal_binding");
      for (const previous of completed) requireCondition(isDeepStrictEqual(previous, requireCadencePhase(input.review, previous.phase)), "cadence_frozen_summary_changed");
      if (input.phase === "usb") requireCondition(previousProbe?.ordinal === 12, "cadence_probe_count");
      if (input.phase === "mining") requireCondition(miningMeasurementEnd, "cadence_measurement_end_missing");
      const collectedAt = now();
      await writeNew(resolve(root, `cadence-${input.phase}.json`), { schema: "worker-cadence-phase-v1", context_sha256: phaseContext,
        ...activePhase, finished_sequence: current().sequence,
        measurement_end_sequence: input.phase === "mining" ? miningMeasurementEnd?.sequence : current().sequence,
        finished_at_unix_ms: input.phase === "mining" ? miningMeasurementEnd?.observedAtUnixMs : collectedAt,
        collected_at_unix_ms: collectedAt, review: input.review, observer_connected: true });
      completed.push(phase); activePhase = undefined; phaseIndex += 1;
      return { cadence_phase_saved: input.phase };
    }
    if (path === "/cadence/observer/stop" && input) {
      exactObject(input, []);
      if (miningMeasurementEnd) requireCondition(now() >= miningMeasurementEnd.observedAtUnixMs + 5000, "cadence_fault_observer_tail");
      observerResult = await observer.finish(); return observerResult;
    }
    return undefined;
  }
  async function recordFault() {
    requireCondition(activePhase?.phase === "mining" && !miningMeasurementEnd && observer.status().alive &&
      observer.status().connected && !observer.status().failed, "cadence_fault_observer");
    miningMeasurementEnd = { sequence: current().sequence, observedAtUnixMs: now() };
    await writeNew(resolve(root, "cadence-mining-measurement-end.json"), { context_sha256: phaseContext, ...miningMeasurementEnd });
  }
  async function finish() {
    const result = await observer.finish(), status = observer.status();
    requireCondition(result.cleanupComplete === true && !status.failed &&
      (status.startedAtUnixMs === null || (result.connected === true && result.closed === true && result.exitCode === 0 && result.reason === "requested")),
      "cadence_observer_cleanup_failed");
    return result;
  }
  return { handle, readyToSign, recordFault, finish };
}

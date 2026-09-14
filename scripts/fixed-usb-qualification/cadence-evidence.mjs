import { CADENCE_PHASES, CADENCE_LIVE_STAGES } from "./cadence-contract.mjs";
import { exactObject, requireCondition } from "./contract.mjs";

const integer = value => Number.isSafeInteger(value) && value >= 0;
const COUNTS = ["armedAtUs", "startedAtUs", "endedAtUs", "generation", "maxProbeCount", "firstMaxProbeAtUs", "lastMaxProbeAtUs", "intervalCount", "maximumIntervalUs",
  "maximumExecutionUs", "maximumLiveUs", "maximumLogsUs", "maximumPruneUs", "cpuMismatchCount", "priorityMismatchCount",
  "subscriberMismatchCount", "projectionCount", "unchangedCount", "noSubscriberCount", "projectionFailures",
  "serializationFailures", "queueFailures", "sendFailures", "sendsQueued", "sendsCompleted", "pendingSends", "clockFailures"];
const ZERO = ["cpuMismatchCount", "priorityMismatchCount", "subscriberMismatchCount", "noSubscriberCount",
  "projectionFailures", "serializationFailures", "queueFailures", "sendFailures", "pendingSends", "clockFailures"];

export function requireCadenceDiagnostics(context, review) {
  requireCondition(review?.schema === (context.cadence_diagnostics_version === 2 ? "worker-telemetry-cadence-v2" : "worker-telemetry-cadence-v1"),
    "cadence_diagnostics_identity");
}

export function validateCadenceReview(value) {
  exactObject(value, ["schema", "snapshotAvailable", "droppedObservations", "storageBytes", "phases"]);
  requireCondition(["worker-telemetry-cadence-v1", "worker-telemetry-cadence-v2"].includes(value.schema) && typeof value.snapshotAvailable === "boolean" &&
    integer(value.droppedObservations) && integer(value.storageBytes) && value.storageBytes <= 2048 &&
    Array.isArray(value.phases) && value.phases.length === 3, "cadence_review_shape");
  for (const [index, phase] of value.phases.entries()) {
    const diagnostics = value.schema === "worker-telemetry-cadence-v2";
    exactObject(phase, ["phase", "state", ...COUNTS, "intervalBuckets", "overflow", "passed",
      ...(diagnostics ? ["maximumLiveStagesUs", "worstInterval"] : [])]);
    requireCondition(phase.phase === CADENCE_PHASES[index] && ["empty", "armed", "capturing", "complete"].includes(phase.state) &&
      COUNTS.every(key => integer(phase[key]) && (key.endsWith("Us") || phase[key] <= 0xffffffff)) && typeof phase.overflow === "boolean" && typeof phase.passed === "boolean" &&
      Array.isArray(phase.intervalBuckets) && phase.intervalBuckets.length === 4 && phase.intervalBuckets.every(value => integer(value) && value <= 0xffffffff), "cadence_phase_shape");
    if (diagnostics) {
      exactObject(phase.worstInterval, ["previousExecutionUs", "previousLiveStagesUs", "gapUs"]);
      requireCondition([phase.maximumLiveStagesUs, phase.worstInterval.previousLiveStagesUs].every(values =>
        Array.isArray(values) && values.length === CADENCE_LIVE_STAGES.length && [...values].every(integer)) &&
        integer(phase.worstInterval.previousExecutionUs) && integer(phase.worstInterval.gapUs), "cadence_live_diagnostics_shape");
    }
  }
  return value;
}

export function requireCadencePhase(value, name) {
  validateCadenceReview(value);
  requireCondition(value.snapshotAvailable && value.droppedObservations === 0, "cadence_capture_loss");
  const phase = value.phases[CADENCE_PHASES.indexOf(name)];
  requireCondition(phase && phase.state === "complete" && phase.passed && !phase.overflow &&
    phase.armedAtUs <= phase.startedAtUs && phase.endedAtUs - phase.startedAtUs >= 60000000 &&
    phase.endedAtUs - phase.startedAtUs <= 62000000 && phase.intervalCount >= 60 &&
    phase.intervalBuckets.reduce((a, b) => a + b, 0) === phase.intervalCount &&
    (phase.intervalBuckets[0] + phase.intervalBuckets[1]) * 100 >= phase.intervalCount * 95 &&
    phase.maximumIntervalUs <= 1500000 && phase.maximumExecutionUs <= 500000 &&
    phase.maximumLiveUs <= phase.maximumExecutionUs && phase.maximumLogsUs <= phase.maximumExecutionUs &&
    phase.maximumPruneUs <= phase.maximumExecutionUs && ZERO.every(key => phase[key] === 0) &&
    phase.sendsQueued === phase.sendsCompleted && phase.intervalBuckets[3] === 0,
  "cadence_phase_unqualified");
  if (value.schema === "worker-telemetry-cadence-v2") {
    const witness = phase.worstInterval;
    requireCondition(witness.previousExecutionUs + witness.gapUs === phase.maximumIntervalUs &&
      witness.previousLiveStagesUs.reduce((sum, duration) => sum + duration, 0) <= witness.previousExecutionUs &&
      phase.maximumLiveStagesUs.every(duration => duration <= phase.maximumLiveUs), "cadence_live_diagnostics_incoherent");
  }
  if (name === "usb") requireCondition(phase.maxProbeCount === 12 && phase.firstMaxProbeAtUs >= phase.startedAtUs &&
    phase.lastMaxProbeAtUs >= phase.firstMaxProbeAtUs && phase.lastMaxProbeAtUs - phase.startedAtUs < 60000000,
    "cadence_device_probes");
  else requireCondition(phase.maxProbeCount === 0 && phase.firstMaxProbeAtUs === 0 && phase.lastMaxProbeAtUs === 0,
    "cadence_device_probes");
  return phase;
}

export function validateCadenceBrowser(value) {
  exactObject(value, ["schema", "enabled", "suppressionRequested"], ["firstWorkObservedAtMs", "latestWork", "review"]);
  requireCondition(value.schema === "worker-cadence-browser-v1" && value.enabled === true &&
    typeof value.suppressionRequested === "boolean", "cadence_browser_shape");
  if (value.firstWorkObservedAtMs !== undefined) requireCondition(integer(value.firstWorkObservedAtMs), "cadence_browser_time");
  if (value.latestWork !== undefined) {
    exactObject(value.latestWork, ["atMs", "generation", "workDispatched"]);
    requireCondition(Object.values(value.latestWork).every(integer) && value.latestWork.generation > 0 &&
      value.latestWork.generation <= 0xffffffff && value.latestWork.workDispatched <= 0xffffffff, "cadence_browser_work");
  }
  if (value.review !== undefined) validateCadenceReview(value.review);
}

export function validateCadenceProbe(value, previous) {
  exactObject(value, ["ordinal", "scheduledAtMs", "startedAtMs", "completedAtMs", "requestPayloadBytes", "responsePayloadBytes"]);
  requireCondition(Object.values(value).every(integer) && value.ordinal === (previous?.ordinal ?? 0) + 1 && value.ordinal <= 12 &&
    value.requestPayloadBytes === 65536 && value.responsePayloadBytes === 65536 && value.startedAtMs >= value.scheduledAtMs && value.startedAtMs <= value.scheduledAtMs + 1000 &&
    value.completedAtMs < value.scheduledAtMs + 5000 && value.completedAtMs >= value.startedAtMs && value.completedAtMs - value.startedAtMs <= 5000 &&
    (!previous || (value.scheduledAtMs === previous.scheduledAtMs + 5000 && value.startedAtMs >= previous.completedAtMs)), "cadence_usb_probe");
  return value;
}

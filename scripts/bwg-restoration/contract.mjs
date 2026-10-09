// BWG-007 serial restoration campaign contract (firmware ADR-0035): the task gate, the eight scenarios in
// their fixed order, lease windows, contract caps and the closed vocabularies every record must use.
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";

export const RESTORATION_TASK = "task-bwg007-real-worker-restoration";
export const RESTORATION_TASK_LINE = "BWG-007 serial restoration hardware: enabled.";
/** Fixed origin that holds the Ultra 205 Web Serial grant (AGENTS.md, Persistent Gate Browser Tab). */
export const RESTORATION_PORT = 48765;
export const PAGE = "conformance/bwg-worker-serial-0.2/restoration.html";
export const BUNDLE = "dist/worker-restoration/worker-restoration-page.js";
export const CONTEXT_SCHEMA = "bwg-restoration-context-v1";
export const RESULT_SCHEMA = "bwg-restoration-result-v1";
export const SCENARIO_RESULT_SCHEMA = "bwg-restoration-scenario-result-v1";
export const SUPERVISOR_STATE_SCHEMA = "bwg-restoration-supervisor-state-v1";
export const PROTOCOL_VERSION = "bwg-worker-controller/0.4";
export const ARTIFACT_PROFILE = "bwg-worker-lease-authorization-artifact/0.1";
export const ATTEMPT_PATTERN = /^attempt-[0-9]{3}$/u;

export const SCENARIOS = Object.freeze(["completion", "pause", "cancel", "expiry", "monotonic_uncertainty", "disconnect", "reboot",
  "authorization_negatives"]);

/** Contract caps (ADR-0035 decision 4): per attempt, and re-arms per physical scenario. */
export const MAXIMUM_STARTS = 10;
export const MAXIMUM_RENEWALS = 2;
export const MAXIMUM_REARMS = 2;

/** Unbudgeted Conservative Stratum V1 windows; only the standard window may carry its one renewal. */
export const WINDOWS = Object.freeze({
  standard: Object.freeze({ durationMilliseconds: 60000, renewAfterMilliseconds: 20000 }),
  expiry: Object.freeze({ durationMilliseconds: 30000, renewAfterMilliseconds: 10000 }),
});

/**
 * What each scenario signs and how it must end. `endedBy` names the page journal event that ends the lease
 * (`device` means the device ended it itself and the page only observed baseline).
 */
export const SCENARIO_PLANS = Object.freeze({
  completion: Object.freeze({ window: "standard", renewals: 1, terminal: "challenge_satisfied", endedBy: "restored" }),
  pause: Object.freeze({ window: "standard", renewals: 0, terminal: "paused", endedBy: "paused" }),
  cancel: Object.freeze({ window: "standard", renewals: 0, terminal: "cancelled", endedBy: "cancelled" }),
  expiry: Object.freeze({ window: "expiry", renewals: 0, terminal: "lease_expired", endedBy: "device" }),
  monotonic_uncertainty: Object.freeze({ window: "standard", renewals: 0, terminal: "monotonic_reset", endedBy: "device" }),
  disconnect: Object.freeze({ window: "standard", renewals: 0, terminal: "connectivity_lost", endedBy: "device" }),
  reboot: Object.freeze({ window: "standard", renewals: 0, terminal: "reboot", endedBy: "device" }),
  authorization_negatives: Object.freeze({ window: "standard", renewals: 1, terminal: "control_failed", endedBy: "device" }),
});

/** Physical checkpoints: absence bounds are finite effect windows; human readiness waits have no deadline. */
export const PHYSICAL_PLANS = Object.freeze({
  disconnect: Object.freeze({ kind: "usb_only", minimumAbsenceMs: 5000, removeCheckpoint: "remove_usb", restoreCheckpoint: "restore_usb",
    removal: "Remove only the USB cable from the Ultra 205. Keep barrel power connected. Then wait for the restore instruction.",
    restore: "Reconnect the USB cable to the Ultra 205." }),
  reboot: Object.freeze({ kind: "both_power", minimumAbsenceMs: 10000, removeCheckpoint: "remove_power", restoreCheckpoint: "restore_power",
    removal: "Remove the USB cable and the barrel power from the Ultra 205. Then wait for the restore instruction.",
    restore: "Restore barrel power first, then reconnect the USB cable." }),
});
/** The removal must be observed this soon after the instruction (widened for operator latency, owner 2026-10-09). */
export const REMOVAL_WINDOW_MS = 45000;
/**
 * The removal must also be observed this long before the delivered lease could end. The device accepts the Start
 * only after delivery, so delivery + 60 s is the earliest device deadline; 5 s covers the 2.8 s heartbeat deadline,
 * so the device sees the transport loss while it still holds the lease.
 */
export const REMOVAL_LEASE_HEADROOM_MS = 5000;
/** Bound for the watcher to prove the admitted device present before any removal instruction. */
export const WATCHER_PRESENT_TIMEOUT_MS = 10000;
export const RESTORE_WATCHER_TOKEN = "bwg-restoration-restore-watcher-armed-v1";

/** authorization_negatives timing: N2 waits past the 60 s admission age; N3 needs a fresh admission. */
export const EXPIRED_START_WAIT_MS = 61000;
export const FRESH_POSSESSION_MS = 45000;
/** The device must end a stimulated lease well before the 60 s lease (widened for host latency, owner 2026-10-09). */
export const STIMULUS_OBSERVATION_MS = 20000;
export const EXPIRY_EARLY_TOLERANCE_MS = 3000;

/**
 * The device's non-authoritative admission diagnostic as the restoration page exposes it (firmware
 * `admission_diagnostics.rs`, Gate `state().admission`). Serve uses it only to refuse, never to authorize.
 */
export const ADMISSION_STAGES = Object.freeze(["idle", "admission", "readiness", "preparation", "pool_activation", "active", "cleanup", "complete"]);
export const ADMISSION_FAILURES = Object.freeze(["none", "admission", "readiness", "preparation", "pool_activation", "cleanup"]);
export const ADMISSION_READINESS_MAXIMUM = 63;
/** Stages in which no admission, lease or native shutdown is in progress. */
export const SETTLED_ADMISSION_STAGES = Object.freeze(["idle", "complete"]);

/** Closed Gate restoration-page journal events (web/worker-restoration-journal.ts). */
export const PAGE_JOURNAL_EVENTS = Object.freeze([
  "configured", "admission_failed", "serial_failure", "connect_failed", "connected", "disconnected",
  "start_prepared", "lease_loaded", "lease_started", "lease_start_failed", "renewed", "renew_failed",
  "paused", "cancelled", "restored", "stop_failed", "device_baseline_observed",
  "stimulus_acknowledged", "stimulus_failed", "stimulus_reviewed", "rejection_reviewed", "status_reviewed", "review_failed",
  "replay_accepted", "replay_rejected", "replay_failed",
  "physical_window_begun", "physical_window_armed",
  "status_failed", "closed", "close_failed", "completion_submitted", "completion_failed",
  "admission_observed",
]);
/** Journal events that are failures of the page's own operations. */
export const PAGE_FAILURE_EVENTS = Object.freeze(["admission_failed", "serial_failure", "connect_failed", "lease_start_failed", "renew_failed",
  "stop_failed", "stimulus_failed", "review_failed", "replay_failed", "status_failed", "close_failed", "completion_failed"]);

/** `window.workerRestoration` operations the supervisor client may record. */
export const PAGE_OPERATIONS = Object.freeze(["connect", "reconnect", "prepareStart", "loadScenarioLease", "startScenarioLease", "renewOnce",
  "pause", "cancel", "restoreChallengeSatisfied", "triggerClockDiscontinuity", "clockDiscontinuityStimulusReview",
  "authorizationRejectionReview", "statusReview", "replayArtifact", "beginPhysicalWindow", "armPhysicalWindow", "physicalWindowState", "close",
  "admissionDiagnostic"]);

/** Closed host campaign events; rows carry only these names, scenario names, checkpoints and counts. */
export const HOST_EVENTS = Object.freeze(["campaign_started", "scope_created", "scope_reused", "artifacts_signed", "artifacts_delivered",
  "replay_delivered", "artifact_burned", "operator_ready", "operator_cancelled", "watcher_started", "watcher_stopped", "checkpoint",
  "restore_watcher_armed", "scenario_judged", "campaign_completed", "campaign_failed"]);

export const TOKEN = /^[a-z][a-z0-9_]{0,63}$/u;
export const scenarioIndex = (scenario) => SCENARIOS.indexOf(scenario);
/** reboot and authorization_negatives share one scope and one boot (ADR-0035, N1). */
export const scopeGroup = (scenario) => scenario === "authorization_negatives" ? "reboot" : scenario;

/** The restoration task must be active exactly once and carry the exact enable line in its own block. */
export async function requireRestorationTask(firmwareRoot) {
  const lines = (await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8")).split(/\r?\n/u);
  let section = "", inBlock = false, count = 0, enabled = false;
  for (const line of lines) {
    if (line.startsWith("## ")) { section = line; inBlock = false; }
    if (line.startsWith("### ")) {
      inBlock = section === "## Active" && line.slice(4).split(/\s/u)[0] === RESTORATION_TASK;
      if (inBlock) count += 1;
    }
    if (inBlock && line.trim() === RESTORATION_TASK_LINE) enabled = true;
  }
  requireCondition(count === 1, "restoration_task_ambiguous");
  requireCondition(enabled, "restoration_task_disabled");
}

/** The Gate commit pinned by the firmware's MODULE.bazel Gate archive. */
export async function pinnedGateCommit(firmwareRoot) {
  const text = await readFile(resolve(firmwareRoot, "MODULE.bazel"), "utf8");
  const pins = [...text.matchAll(/strip_prefix = "bitaxe-turnstile-system-([0-9a-f]{40})"/gu)].map((match) => match[1]);
  requireCondition(pins.length === 1, "gate_pin_ambiguous");
  return pins[0];
}

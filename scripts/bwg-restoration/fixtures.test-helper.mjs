// Shared fixtures: page states, journals, records and passing judge inputs for every scenario.
import { spawn } from "node:child_process";
import { mkdtemp, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fileDigest } from "../fixed-usb-qualification/contract.mjs";
import { CONTEXT_SCHEMA } from "./contract.mjs";

export const context = Object.freeze({ schema: CONTEXT_SCHEMA, gate_commit: "c".repeat(40), firmware_commit: "a".repeat(40), app_elf_sha256: "b".repeat(64),
  reference_commit: "d".repeat(40), manifest_sha256: "1".repeat(64), gate_bundle_sha256: "2".repeat(64), gate_page_sha256: "3".repeat(64),
  trust_sha256: "4".repeat(64), physical_identity_sha256: "5".repeat(64), attempt: "attempt-001", firmware_root: "", gate_root: "",
  gate_page_relative_path: "restoration.html", watcher: { path: "/nonexistent", sha256: "6".repeat(64) } });

export async function privateDirectory(prefix = "restoration-") {
  return realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), prefix)));
}

/** Journal entries from `event` or `event:category` strings, numbered from `start + 1`. */
export function journal(events, start = 0) {
  return events.map((item, index) => {
    const [event, category] = item.split(":");
    return { ordinal: start + index + 1, event, ...(category ? { category } : {}) };
  });
}

/** `admission` is omitted unless given, as an older page sends; the current Gate always sends `null` or an observation. */
export function pageState({ entries = [], connected = false, status = "closed", device, highWaterEpoch = 1, leaseActive = false, admission,
  deviceIdentity, poolConfiguration } = {}) {
  return { schema: "worker-restoration-page-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status, connected, leaseActive, leaseLoaded: false, renewalsRemaining: 0, stimulusUsed: false,
    highWaterEpoch, ...(device ? { device } : {}), ...(admission === undefined ? {} : { admission }),
    ...(deviceIdentity === undefined ? {} : { deviceIdentity }), ...(poolConfiguration === undefined ? {} : { poolConfiguration }),
    journal: { entries, dropped: 0 } };
}

export const baseline = (reason) => ({ state: "baseline", restoration: "confirmed", reason });
export const stimulusReview = (state, discontinuitiesDetected) => ({ schema: "worker-clock-discontinuity-stimulus-review-v1", state, offsetMilliseconds: 1000,
  discontinuitiesDetected });
export function rejectionReview({ bootRejections = 0, last = null, advancedThisBoot = false, matches = true, epoch = 1 } = {}) {
  return { schema: "worker-authorization-rejection-review-v1", bootRejections, last,
    highWater: { advancedThisBoot, fingerprintMatchesLatestObservation: matches, fingerprintFirstObservedEpoch: epoch } };
}
export const okRecord = (operation, result, at, state = pageState()) => ({ operation, outcome: "ok", result, state, receivedAtUnixMs: at });

const PHYSICAL_FACTS = { watcherBeforeRemovalInstruction: true, removalObservedByWatcher: true, restoreTokenBeforeRestoreInstruction: true,
  absenceMs: 12000, absenceBoundMet: true, enumerationChanged: true, samePhysicalIdentity: true, stableBeforeReconnect: true, rearms: 0, failure: null };

function physicalJournal(remove, restore, terminal) {
  return journal(["connected", "start_prepared", "lease_loaded:no_renewal", "lease_started", "status_reviewed:mining", `physical_window_begun:${remove}`,
    "serial_failure:port_lost", "disconnected", "physical_window_armed:absence_bounding", `physical_window_armed:${restore}`, "connected",
    `status_reviewed:${terminal}`, "stimulus_reviewed:idle", "rejection_reviewed:none", "closed"]);
}

const leg = (operation, signature, context, replayGuard, ordinal, advancedThisBoot, category = "authentication_failed") => ({
  replay: { operation, outcome: "rejected", category },
  review: rejectionReview({ bootRejections: ordinal, last: { ordinal, operation, signature, context, replayGuard }, advancedThisBoot, epoch: 3 }),
});

/** A passing judge input for each scenario; tests break exactly one fact. */
export function passingInput(scenario) {
  const common = { scenario, segmentStartOrdinal: 0, legs: {}, checkpoint: null, safetySamples: [], carry: { stimulusBaseline: 0 },
    reviews: { stimulus: stimulusReview("idle", 0), rejection: rejectionReview() } };
  const operatorEvents = (stop, loaded = "no_renewal", extra = []) => journal(["connected", "start_prepared", `lease_loaded:${loaded}`, "lease_started", ...extra,
    stop, "stimulus_reviewed:idle", "rejection_reviewed:none", "closed"]);
  const inputs = {
    completion: { records: [], finalState: pageState({ entries: operatorEvents("restored", "one_renewal", ["renewed"]), device: baseline("challenge_satisfied") }) },
    pause: { records: [], finalState: pageState({ entries: operatorEvents("paused"), device: baseline("paused") }) },
    cancel: { records: [], finalState: pageState({ entries: operatorEvents("cancelled"), device: baseline("cancelled") }) },
    expiry: { records: [okRecord("loadScenarioLease", null, 1000), okRecord("startScenarioLease", null, 2000),
      okRecord("statusReview", baseline("lease_expired"), 31500, pageState({ device: baseline("lease_expired") }))],
      finalState: pageState({ entries: journal(["connected", "start_prepared", "lease_loaded:no_renewal", "lease_started", "device_baseline_observed:lease_expired",
        "status_reviewed:lease_expired", "stimulus_reviewed:idle", "rejection_reviewed:none", "closed"]), device: baseline("lease_expired") }) },
    monotonic_uncertainty: {
      records: [okRecord("clockDiscontinuityStimulusReview", stimulusReview("idle", 0), 1000), okRecord("startScenarioLease", null, 2000),
        okRecord("triggerClockDiscontinuity", { schema: "worker-clock-discontinuity-stimulus-v1", offsetMilliseconds: 1000, armedForMilliseconds: 2000 }, 3000),
        okRecord("statusReview", baseline("monotonic_reset"), 5000, pageState({ device: baseline("monotonic_reset") }))],
      reviews: { stimulus: stimulusReview("consumed", 1), rejection: rejectionReview() },
      finalState: pageState({ entries: journal(["connected", "stimulus_reviewed:idle", "start_prepared", "lease_loaded:no_renewal", "lease_started",
        "stimulus_acknowledged", "device_baseline_observed:monotonic_reset", "status_reviewed:monotonic_reset", "stimulus_reviewed:consumed",
        "rejection_reviewed:none", "closed"]), device: baseline("monotonic_reset") }) },
    disconnect: { records: [], checkpoint: { ...PHYSICAL_FACTS }, reviews: { stimulus: stimulusReview("consumed", 1), rejection: rejectionReview() },
      finalState: pageState({ entries: physicalJournal("remove_usb", "restore_usb", "connectivity_lost"), device: baseline("connectivity_lost") }) },
    reboot: { records: [okRecord("statusReview", { state: "mining", restoration: "pending" }, 2000, pageState({ highWaterEpoch: 3 })),
      okRecord("beginPhysicalWindow", { checkpoint: "remove_power" }, 3000, pageState({ highWaterEpoch: 3 }))], checkpoint: { ...PHYSICAL_FACTS },
    finalState: pageState({ entries: physicalJournal("remove_power", "restore_power", "reboot"), device: baseline("reboot") }) },
    authorization_negatives: { records: [], carry: { stimulusBaseline: 0, preRebootEpoch: 3, rebootRejections: 0 },
      legs: { n1: leg("start", "valid", "mismatch", "at_or_below_durable_high_water", 1, false), n2: leg("start", "not_evaluated", "expired", "not_evaluated", 2, false, "admission_required"),
        n3: leg("start", "valid", "mismatch", "fresh", 3, false), n4: leg("renew", "valid", "current", "at_or_below_durable_high_water", 4, true) },
      reviews: { stimulus: stimulusReview("idle", 0), rejection: rejectionReview({ bootRejections: 4, advancedThisBoot: true, epoch: 3,
        last: { ordinal: 4, operation: "renew", signature: "valid", context: "current", replayGuard: "at_or_below_durable_high_water" } }) },
      finalState: pageState({ entries: journal(["connected", "status_reviewed:reboot", "replay_rejected:authentication_failed", "disconnected", "connected", "rejection_reviewed:mismatch",
        "start_prepared", "replay_rejected:admission_required", "disconnected", "connected", "rejection_reviewed:expired", "replay_rejected:authentication_failed",
        "disconnected", "connected", "rejection_reviewed:mismatch", "start_prepared", "lease_loaded:one_renewal", "lease_started", "renewed",
        "replay_rejected:authentication_failed", "disconnected", "connected", "rejection_reviewed:current", "status_reviewed:connectivity_lost",
        "stimulus_reviewed:idle", "rejection_reviewed:current", "closed"]), device: baseline("connectivity_lost") }) },
  };
  return { ...common, ...inputs[scenario] };
}

const HELPER = fileURLToPath(new URL("./fake-watcher.test-helper.mjs", import.meta.url));

/** Real child processes: node runs the fake helper, so no freshly written file is ever exec'd. */
export async function fakeWatcherOperations(root, steps) {
  const control = resolve(root, "control.txt");
  await writeFile(control, "0");
  // Bazel's node may be a symlink, and the watcher owner refuses symlinked binaries.
  const node = await realpath(process.execPath);
  return { control, binary: { path: node, sha256: await fileDigest(node) },
    // The real watcher runs with an empty environment; Bazel's node launcher needs its own variables.
    spawn: (path, args, options) => spawn(path, [HELPER, control, JSON.stringify(steps), ...args], { ...options, env: process.env }) };
}

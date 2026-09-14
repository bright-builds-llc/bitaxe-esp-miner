import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { preminingFixture, artifactFixture, baselineFixture, cycleFixture } from "./cadence-premining-fixtures.mjs";
import { closePremining, preminingClosurePath } from "./cadence-premining.mjs";
import { cadencePreflight } from "./cadence-preflight.mjs";
import { digest, fileDigest, writeNew } from "./contract.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { USB_PREMINING_AUDITOR } from "./cadence-premining-usb.mjs";

function phase(phase, index) {
  return {
    phase,
    state: "empty",
    armedAtUs: 0,
    startedAtUs: 0,
    endedAtUs: 0,
    generation: 0,
    maxProbeCount: 0,
    firstMaxProbeAtUs: 0,
    lastMaxProbeAtUs: 0,
    intervalCount: 0,
    maximumIntervalUs: 0,
    maximumExecutionUs: 0,
    maximumLiveUs: 0,
    maximumLogsUs: 0,
    maximumPruneUs: 0,
    cpuMismatchCount: 0,
    priorityMismatchCount: 0,
    subscriberMismatchCount: 0,
    projectionCount: 0,
    unchangedCount: 0,
    noSubscriberCount: 0,
    projectionFailures: 0,
    serializationFailures: 0,
    queueFailures: 0,
    sendFailures: 0,
    sendsQueued: 0,
    sendsCompleted: 0,
    pendingSends: 0,
    clockFailures: 0,
    intervalBuckets: [0, 0, 0, 0],
    overflow: false,
    passed: false,
  };
}
function reviews() {
  const initial = {
    schema: "worker-telemetry-cadence-v1",
    snapshotAvailable: true,
    droppedObservations: 0,
    storageBytes: 540,
    phases: ["idle", "usb", "mining"].map(phase),
  };
  Object.assign(initial.phases[0], {
    state: "complete",
    armedAtUs: 1000000,
    startedAtUs: 1000000,
    endedAtUs: 61000000,
    intervalCount: 95,
    maximumIntervalUs: 700000,
    maximumExecutionUs: 200000,
    maximumLiveUs: 100000,
    maximumLogsUs: 50000,
    maximumPruneUs: 10000,
    projectionCount: 96,
    sendsQueued: 96,
    sendsCompleted: 96,
    intervalBuckets: [0, 95, 0, 0],
    passed: true,
  });
  const bad = structuredClone(initial);
  bad.droppedObservations = 1;
  bad.phases[0].passed = false;
  Object.assign(bad.phases[1], {
    state: "complete",
    armedAtUs: 62000000,
    startedAtUs: 62000000,
    endedAtUs: 122500547,
    maxProbeCount: 12,
    firstMaxProbeAtUs: 63147059,
    lastMaxProbeAtUs: 117969243,
    intervalCount: 91,
    maximumIntervalUs: 876213,
    maximumExecutionUs: 369667,
    maximumLiveUs: 150000,
    maximumLogsUs: 200000,
    maximumPruneUs: 10000,
    projectionCount: 92,
    sendsQueued: 92,
    sendsCompleted: 90,
    pendingSends: 2,
    intervalBuckets: [0, 85, 6, 0],
  });
  return { initial, bad };
}
async function prepare(t) {
  const f = await preminingFixture(t);
  await closePremining(f.root, f.operations);
  const { sourceRoot, entries, snapshot } = await artifactFixture(f, "8".repeat(40));
  const input = resolve(f.base, "usb-software-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["d".repeat(64)],
  });
  const options = {
    ...f.options,
    privateRoot: resolve(f.base, "attempts/usb-preparation-3"),
    input,
    supersedeUnissued: undefined,
    supersedePremining: preminingClosurePath(f.root),
  };
  f.operations.inspectSources = async () => snapshot;
  await cadencePreflight(options, f.operations);
  const root = options.privateRoot,
    { context } = JSON.parse(await readFile(resolve(root, "context.json"), "utf8"));
  for (const entry of entries) {
    const output = resolve(root, "qualified-artifacts", entry.path);
    await mkdir(dirname(output), { recursive: true, mode: 0o700 });
    await copyFile(resolve(sourceRoot, entry.path), output);
  }
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: entries,
  });
  f.operations.verifyArtifactSnapshot = async (path, value) =>
    [root, f.root].includes(path) ? verifyArtifactSnapshot(path, value) : undefined;
  return { ...f, priorRoot: f.root, root, context, options, snapshot };
}
async function writeCapture(f) {
  const { root, context } = f,
    state = baselineFixture(context),
    records = await cycleFixture(root, context, state),
    { initial, bad } = reviews();
  const ready = (review) => ({
    ...structuredClone(state),
    status: "ready",
    connected: true,
    serialOwnershipReleased: false,
    cadence: { ...state.cadence, review: structuredClone(review) },
  });
  const closed = () => ({ ...structuredClone(state), cadence: { ...state.cadence, review: structuredClone(bad) } });
  records.push(
    { sequence: 9, state: ready(initial) },
    { sequence: 10, state: ready(initial) },
    { sequence: 11, state: ready(bad) },
    { sequence: 12, state: { ...ready(bad), status: "stopping", deviceLeaseInactive: false, deviceBaselineConfirmed: false } },
    { sequence: 13, state: closed() },
    { sequence: 14, state: ready(bad) },
    { sequence: 15, state: ready(bad) },
    { sequence: 16, state: closed() },
  );
  await writeFile(resolve(root, "iterative.samples.jsonl"), records.map((row) => JSON.stringify(row) + "\n").join(""), { mode: 0o600 });
  const hash = digest(JSON.stringify(context));
  const idleArm = {
    context_sha256: hash,
    phase: "idle",
    arm: { schema: "worker-telemetry-cadence-arm-v1", phase: "idle", armedAtUs: 1000000, generation: 0 },
    started_sequence: 8,
    started_at_unix_ms: 100200,
  };
  const usbArm = {
    context_sha256: hash,
    phase: "usb",
    arm: { schema: "worker-telemetry-cadence-arm-v1", phase: "usb", armedAtUs: 62000000, generation: 0 },
    started_sequence: 10,
    started_at_unix_ms: 160500,
  };
  await writeNew(resolve(root, "cadence-idle-arm.json"), idleArm);
  await writeNew(resolve(root, "cadence-usb-arm.json"), usbArm);
  await writeNew(resolve(root, "cadence-idle.json"), {
    schema: "worker-cadence-phase-v1",
    ...idleArm,
    finished_sequence: 9,
    finished_at_unix_ms: 160300,
    collected_at_unix_ms: 160300,
    measurement_end_sequence: 9,
    review: initial,
    observer_connected: true,
  });
  const probes = Array.from({ length: 12 }, (_, index) => ({
    ordinal: index + 1,
    scheduledAtMs: index * 5000,
    startedAtMs: index * 5000,
    completedAtMs: index * 5000 + 500,
    requestPayloadBytes: 65536,
    responsePayloadBytes: 65536,
  }));
  const witnesses = probes.map((probe, index) => ({ ordinal: probe.ordinal, observedAtUnixMs: 161000 + index * 5000, sequence: 10 }));
  for (const [name, rows] of [
    ["cadence-probes.jsonl", probes],
    ["cadence-probe-witnesses.jsonl", witnesses],
  ])
    await writeFile(resolve(root, name), rows.map((row) => JSON.stringify(row) + "\n").join(""), { mode: 0o600 });
  const events = ["connected", "arrival", "closed"].map((event, index) => ({
    sequence: index + 1,
    observedAtUnixMs: [100100, 210100, 226500][index],
    event: {
      schema: "cpu0-cadence-observer-v1",
      event,
      elapsedMs: [0, 110000, 126400][index],
      messageCount: index ? 1 : 0,
      totalBytes: index ? 100 : 0,
      byteCount: index === 1 ? 100 : 0,
      reason: index === 2 ? "requested" : null,
    },
  }));
  const observerBytes = events.map((row) => JSON.stringify(row) + "\n").join("");
  await writeFile(resolve(root, "cadence-observer.jsonl"), observerBytes, { mode: 0o600 });
  await writeNew(resolve(root, "cadence-observer-result.json"), {
    schema: "worker-cadence-observer-result-v1",
    connected: true,
    closed: true,
    exitCode: 0,
    reason: "requested",
    cleanupComplete: true,
    startedAtUnixMs: 100000,
    connectedAtUnixMs: 100100,
    closedAtUnixMs: 226502,
    messageCount: 1,
    totalBytes: 100,
    eventCount: 3,
    journalSha256: digest(observerBytes),
  });
  return { records, initial, bad };
}
async function writeFailure(f, bad) {
  const { root } = f;
  await writeNew(resolve(root, "operator-failure.json"), {
    schema: "cpu0-cadence-usb-failure-v1",
    source: "parent-observed",
    client_failure: { stage: "usb_review", message: "cadence_supervisor_rejected" },
    replayed_validator_failure: "cadence_capture_loss",
    first_rejected_review_sequence: 11,
    first_rejected_review_sha256: digest(JSON.stringify(bad)),
    qualified: false,
    usb_probes_completed: 12,
    mining_started: false,
    allowance_issued: false,
    observations: {
      interval_count: 91,
      within_750ms: 85,
      maximum_interval_us: 876213,
      maximum_execution_us: 369667,
      dropped_observations: 1,
      pending_sends: 2,
    },
    ledger: {
      schema: "worker-qualification-ledger-v1",
      next_ordinal: 16,
      last_completed_ordinal: 15,
      pending: false,
      total_charged_ms: 1200000,
    },
    ledger_source: "fresh_authenticated_workerAcceptance.reviewQualificationAttempts",
    retained_review_after_fresh_reconnect: true,
    final_state: {
      status: "closed",
      connected: false,
      running: false,
      deviceBaselineConfirmed: true,
      deviceLeaseInactive: true,
      serialOwnershipReleased: true,
    },
  });
  await writeNew(resolve(root, "operator-cleanup.json"), {
    schema: "worker-cadence-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
}
async function writeSeal(f, bad) {
  const { root, context } = f,
    hash = (name) => fileDigest(resolve(root, name)),
    reviewHash = digest(JSON.stringify(bad));
  const seal = {
    schema: "cpu0-cadence-usb-failed-inventory-v1",
    outcome: "unverified_usb_cadence_failure",
    qualification_pass: false,
    continuation_authority: false,
    auditor_sha256: USB_PREMINING_AUDITOR,
    context_sha256: digest(JSON.stringify(context)),
    artifact_snapshot_sha256: await hash("artifact-snapshot.json"),
    premining_closure_sha256: context.premining_predecessor.closure_sha256,
    samples_sha256: await hash("iterative.samples.jsonl"),
    operator_failure_sha256: await hash("operator-failure.json"),
    operator_cleanup_sha256: await hash("operator-cleanup.json"),
    idle_phase_sha256: await hash("cadence-idle.json"),
    observer_result_sha256: await hash("cadence-observer-result.json"),
    first_rejected_review: { sequence: 11, sha256: reviewHash },
    later_retained_review: { sequence: 15, sha256: reviewHash, after_released_sequence: 13 },
    usb_arm_sha256: await hash("cadence-usb-arm.json"),
    probes_sha256: await hash("cadence-probes.jsonl"),
    probe_witnesses_sha256: await hash("cadence-probe-witnesses.jsonl"),
    inventory: await inventory(root),
  };
  await writeNew(resolve(root, "failed-inventory.json"), seal);
  Object.defineProperty(f.operations, "expectedUsbPreminingInventorySha256", {
    value: await hash("failed-inventory.json"),
    enumerable: true,
  });
  return seal;
}
export async function usbPreminingFixture(t) {
  const f = await prepare(t),
    capture = await writeCapture(f);
  await writeFailure(f, capture.bad);
  const seal = await writeSeal(f, capture.bad);
  return { ...f, ...capture, seal };
}

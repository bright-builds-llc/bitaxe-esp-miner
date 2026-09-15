import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { cadenceRestartFixture } from "./cadence-restart-supersession-fixtures.mjs";
import { cadencePreflight } from "./cadence-preflight.mjs";
import { baselineFixture, cycleFixture, artifactFixture } from "./cadence-premining-fixtures.mjs";
import { digest, inspectPackage, readJson, writeNew } from "./contract.mjs";
import { inspectCadenceFailureEvidence } from "./cadence-failure-evidence.mjs";
import { inspectCadenceFailure, CADENCE_FAILURE_PRODUCER, CADENCE_FAILURE_SHA256 } from "./cadence-failure.mjs";

function phase(name) {
  const value = {
    phase: name,
    state: "empty",
    intervalBuckets: [0, 0, 0, 0],
    overflow: false,
    passed: false,
    maximumLiveStagesUs: Array(11).fill(0),
    worstInterval: { previousExecutionUs: 0, previousLiveStagesUs: Array(11).fill(0), gapUs: 0 },
  };
  for (const key of [
    "armedAtUs",
    "startedAtUs",
    "endedAtUs",
    "generation",
    "maxProbeCount",
    "firstMaxProbeAtUs",
    "lastMaxProbeAtUs",
    "intervalCount",
    "maximumIntervalUs",
    "maximumExecutionUs",
    "maximumLiveUs",
    "maximumLogsUs",
    "maximumPruneUs",
    "cpuMismatchCount",
    "priorityMismatchCount",
    "subscriberMismatchCount",
    "projectionCount",
    "unchangedCount",
    "noSubscriberCount",
    "projectionFailures",
    "serializationFailures",
    "queueFailures",
    "sendFailures",
    "sendsQueued",
    "sendsCompleted",
    "pendingSends",
    "clockFailures",
  ])
    value[key] = 0;
  return value;
}
function reviews() {
  const idle = {
    schema: "worker-telemetry-cadence-v2",
    snapshotAvailable: true,
    droppedObservations: 0,
    storageBytes: 1284,
    phases: ["idle", "usb", "mining"].map(phase),
  };
  Object.assign(idle.phases[0], {
    state: "complete",
    passed: true,
    armedAtUs: 1000000,
    startedAtUs: 1000000,
    endedAtUs: 61000000,
    intervalCount: 100,
    intervalBuckets: [0, 100, 0, 0],
    maximumIntervalUs: 745891,
    maximumExecutionUs: 238342,
    maximumLiveUs: 200000,
    maximumLogsUs: 5000,
    maximumPruneUs: 5000,
    projectionCount: 100,
    sendsQueued: 100,
    sendsCompleted: 100,
    worstInterval: { previousExecutionUs: 238342, previousLiveStagesUs: Array(11).fill(0), gapUs: 507549 },
  });
  const bad = structuredClone(idle);
  Object.assign(bad.phases[1], {
    state: "complete",
    armedAtUs: 62000000,
    startedAtUs: 62000000,
    endedAtUs: 122600000,
    intervalCount: 95,
    intervalBuckets: [0, 89, 6, 0],
    maximumIntervalUs: 828450,
    maximumExecutionUs: 320724,
    maximumLiveUs: 300000,
    maximumLogsUs: 5000,
    maximumPruneUs: 5000,
    projectionCount: 95,
    sendsQueued: 95,
    sendsCompleted: 95,
    maxProbeCount: 12,
    firstMaxProbeAtUs: 63000000,
    lastMaxProbeAtUs: 118000000,
    worstInterval: { previousExecutionUs: 320724, previousLiveStagesUs: Array(11).fill(0), gapUs: 507726 },
  });
  return { idle, bad };
}
async function snapshot(from, root, context) {
  const prior = await readJson(resolve(from, "artifact-snapshot.json"));
  for (const item of prior.files) {
    const target = resolve(root, "qualified-artifacts", item.path);
    await mkdir(dirname(target), { recursive: true, mode: 0o700 });
    await copyFile(resolve(from, "qualified-artifacts", item.path), target);
  }
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: prior.files,
  });
}
async function capture(f) {
  const { root, context } = f,
    hash = digest(JSON.stringify(context)),
    state = { ...baselineFixture(context), deviceRestorationConfirmed: true },
    records = await cycleFixture(root, context, state, { paddingBytes: 65376 }),
    { idle, bad } = reviews();
  const ready = (review) => ({
    ...structuredClone(state),
    status: "ready",
    connected: true,
    serialOwnershipReleased: false,
    cadence: { ...state.cadence, ...(review ? { review: structuredClone(review) } : {}) },
  });
  records.push(
    { sequence: 9, state: ready() },
    { sequence: 10, state: ready(idle) },
    { sequence: 11, state: ready(idle) },
    { sequence: 12, state: ready(bad) },
    { sequence: 13, state: { ...ready(bad), status: "stopping", deviceLeaseInactive: false, deviceBaselineConfirmed: false } },
    { sequence: 14, state: { ...structuredClone(state), cadence: { ...state.cadence, review: bad } } },
  );
  await writeFile(resolve(root, "iterative.samples.jsonl"), records.map((r) => JSON.stringify(r) + "\n").join(""), { mode: 0o600 });
  const idleArm = {
      context_sha256: hash,
      phase: "idle",
      arm: { schema: "worker-telemetry-cadence-arm-v1", phase: "idle", armedAtUs: 1000000, generation: 0 },
      started_sequence: 9,
      started_at_unix_ms: 100100,
    },
    usbArm = {
      context_sha256: hash,
      phase: "usb",
      arm: { schema: "worker-telemetry-cadence-arm-v1", phase: "usb", armedAtUs: 62000000, generation: 0 },
      started_sequence: 11,
      started_at_unix_ms: 160500,
    };
  await writeNew(resolve(root, "cadence-idle-arm.json"), idleArm);
  await writeNew(resolve(root, "cadence-usb-arm.json"), usbArm);
  await writeNew(resolve(root, "cadence-idle.json"), {
    schema: "worker-cadence-phase-v1",
    ...idleArm,
    finished_sequence: 10,
    finished_at_unix_ms: 160200,
    collected_at_unix_ms: 160200,
    measurement_end_sequence: 10,
    review: idle,
    observer_connected: true,
  });
  const cooling = {
    schema: "worker-iterative-cooling-v1",
    context_sha256: hash,
    proof: {
      schema: "worker-cooling-proof-v1",
      fan_duty_percent: 100,
      fan_rpm: 3000,
      post_command_fan_proven: true,
      asic_effects: false,
      budget_reserved: false,
    },
    restoration: {
      schema: "worker-cooling-baseline-v1",
      fan_duty_percent: 30,
      cooling_proven: true,
      asic_effects: false,
      budget_reserved: false,
    },
    budget_before: f.ledger,
    budget_after: f.ledger,
    state: ready(),
  };
  await writeNew(resolve(root, "cooling.json"), cooling);
  const probes = Array.from({ length: 12 }, (_, i) => ({
    ordinal: i + 1,
    scheduledAtMs: i * 5000,
    startedAtMs: i * 5000,
    completedAtMs: i * 5000 + 500,
    requestPayloadBytes: 65536,
    responsePayloadBytes: 65536,
  }));
  for (const [name, rows] of [
    ["cadence-probes.jsonl", probes],
    ["cadence-probe-witnesses.jsonl", probes.map((p, i) => ({ ordinal: p.ordinal, observedAtUnixMs: 161000 + i * 5000, sequence: 11 }))],
  ])
    await writeFile(resolve(root, name), rows.map((v) => JSON.stringify(v) + "\n").join(""), { mode: 0o600 });
  const events = ["connected", "arrival", "closed"].map((event, i) => ({
    sequence: i + 1,
    observedAtUnixMs: [100000, 210000, 226500][i],
    event: {
      schema: "cpu0-cadence-observer-v1",
      event,
      elapsedMs: [0, 110000, 126500][i],
      messageCount: i ? 1 : 0,
      totalBytes: i ? 100 : 0,
      byteCount: i === 1 ? 100 : 0,
      reason: i === 2 ? "requested" : null,
    },
  }));
  const observerBytes = events.map((v) => JSON.stringify(v) + "\n").join("");
  await writeFile(resolve(root, "cadence-observer.jsonl"), observerBytes, { mode: 0o600 });
  await writeNew(resolve(root, "cadence-observer-result.json"), {
    schema: "worker-cadence-observer-result-v1",
    connected: true,
    closed: true,
    exitCode: 0,
    reason: "requested",
    cleanupComplete: true,
    startedAtUnixMs: 100000,
    connectedAtUnixMs: 100000,
    closedAtUnixMs: 226502,
    messageCount: 1,
    totalBytes: 100,
    eventCount: 3,
    journalSha256: digest(observerBytes),
  });
  await writeNew(resolve(root, "operator-failure.json"), {
    schema: "parent-observed-cadence-failure-v1",
    source: "parent-observed",
    context_sha256: hash,
    browser_error: "cadence_supervisor_rejected",
    browser_error_location: "cadence-client.mjs:31 via runQualification usb phase",
    browser_cleanup_promise: "fulfilled-empty-errors",
    phase: "usb",
    journal_sequence: 14,
    within_750_ms: 89,
    intervals: 95,
    maximum_interval_us: 828450,
    maximum_execution_us: 320724,
    all_twelve_probes_completed: true,
    mining_issued: false,
    mining_consumed: false,
    qualified: false,
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
  await writeNew(resolve(root, "supervisor.observation.json"), {
    schema: "hello-passive-command-observation-v1",
    rootObserved: true,
    observations: 1133,
    seen: [],
    failures: [],
    complete: false,
  });
  return { records, idle, bad };
}
export async function failedCadenceFixture(t) {
  const f = await cadenceRestartFixture(t, { restartAttempt: 4 });
  await cadencePreflight(f.options, f.operations);
  const root = f.options.privateRoot,
    context = (await readJson(resolve(root, "context.json"))).context;
  await snapshot(f.root, root, context);
  const value = { ...f, root, context };
  Object.assign(value, await capture(value));
  const restart = { sha256: context.restart_predecessor.receipt_sha256, value: { receipt: f.prior.receipt } };
  await writeNew(
    resolve(root, "failed-inventory.json"),
    await inspectCadenceFailureEvidence(root, context, restart, CADENCE_FAILURE_PRODUCER),
  );
  return value;
}
export async function cadenceFailureFixture(t) {
  const old = await failedCadenceFixture(t),
    next = "f".repeat(40),
    data = await artifactFixture({ base: old.base, oldContext: old.context }, next),
    manifestPath = resolve(data.sourceRoot, "firmware/bitaxe-ultra205-package.json"),
    manifest = await readJson(manifestPath);
  const elf = manifest.artifacts.find((v) => v.kind === "firmware_elf"),
    newBytes = Buffer.from("synthetic cache corrected ELF");
  await writeFile(resolve(data.sourceRoot, "firmware", elf.path), newBytes);
  elf.sha256 = digest(newBytes);
  manifest.app_elf_sha256 = elf.sha256;
  await writeFile(manifestPath, JSON.stringify(manifest));
  const source = {
    ...old.source,
    ...(await inspectPackage(manifestPath, next, () => resolve(data.sourceRoot, "firmware"))),
    firmware_commit: next,
  };
  const input = resolve(old.base, "cadence-failure-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [CADENCE_FAILURE_SHA256, "1".repeat(64)],
  });
  const options = {
    ...old.options,
    privateRoot: resolve(dirname(old.root), "cadence-preparation-3"),
    input,
    supersedeRestart: undefined,
    supersedeCadenceFailure: old.root,
    firmwareCommit: next,
    manifest: manifestPath,
  };
  const operations = {
    ...old.operations,
    inspectSources: async () => source,
    readCadenceFailure: async (root) => {
      const verified = await inspectCadenceFailure(root, old.context, old.operations);
      return { ...verified, binding: { root, failed_inventory_sha256: CADENCE_FAILURE_SHA256 } };
    },
  };
  return { ...old, old, root: options.privateRoot, options, operations, source };
}

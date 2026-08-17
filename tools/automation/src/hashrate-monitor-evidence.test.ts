import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { toolProgram } from "./cli-tools.js";
import {
  captureHashrateMonitorEvidence,
  HashrateMonitorEvidenceError,
  validateHashrateMonitorTaskAndSources,
  type HashrateMonitorEvidenceOptions,
} from "./hashrate-monitor-evidence.js";
import { createLocalProcessPort } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();
const validatorProgram = toolProgram(
  workspace,
  "crates/bitaxe-automation-contracts/validate_hashrate_monitor_evidence",
);
const sourceDocuments = new Map<string, string>([
  ["crates/bitaxe-core/src/hashrate.rs", [
    "const HASHRATE_REGISTER_UNIT_HASHES: f64 = 1_048_576.0;",
    "const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;",
    "const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;",
  ].join("\n")],
  ["crates/bitaxe-core/src/runtime_health.rs", [
    "Some(\"snapshot_retry_exhausted\")",
    "if maybe_previous.is_some_and(|previous| !latest.is_valid_after(previous)) {",
    "let Some(age_millis) = now_millis.checked_sub(observed_at_millis) else {",
  ].join("\n")],
  ["crates/bitaxe-core/src/runtime_health/wait.rs", [
    "pub enum TaskWatchdogReadOutcome {",
    "pub enum TaskWatchdogWaitState {",
    "pub const fn state_at(self, current_monotonic_millis: u64)",
  ].join("\n")],
  ["crates/bitaxe-stratum/src/v1/state.rs", "pub hashrate_inputs: HashrateInputs"],
  ["crates/bitaxe-stratum/src/v1/production_session/campaign.rs", [
    "Self::Conservative => (400, 1_100, 100)",
    "core_voltage_mv: i64,",
  ].join("\n")],
  ["crates/bitaxe-api/src/mining.rs", [
    "hash_rate: hashrate.current_ghs,",
    "hashrate_monitor: HashrateMonitorWire {",
  ].join("\n")],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub bus_voltage_volts: Observation<f64>,",
    "let min_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);",
    "(min_input_voltage..=max_input_voltage).contains(&bus_voltage_volts)",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "hashRate")]',
    '#[serde(rename = "hashrateMonitor")]',
  ].join("\n")],
  ["crates/bitaxe-api/src/wire/runtime_health.rs", [
    'rename = "taskWatchdogReadOutcome"',
    "task_watchdog_read_outcome: snapshot",
    '#[serde(rename = "taskWatchdogWaitState", default = "invalid_wait_state")]',
    "task_watchdog_wait_state: snapshot.task_watchdog_wait_state().as_str().to_owned(),",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/hashrate.rs", [
    "const HASHRATE_CADENCE_MS: u64 = 1_000;",
    "const BM1366_HASH_DOMAIN_COUNT: usize = 4;",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/asic_worker.rs", [
    "request_hashrate_monitor_register_reads_tx()",
    "emit(AsicWorkerEvent::RegisterRead {",
  ].join("\n")],
  ["firmware/bitaxe/src/runtime_snapshot.rs", "publish_hashrate_snapshot"],
  ["firmware/bitaxe/src/runtime_health_adapter.rs", [
    "let task_watchdog = crate::task_watchdog_observation::coherent_observation();",
    "let current_monotonic_millis = crate::runtime_uptime::millis();",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/owner_loop.rs", [
    "if let Err(error) = adapter.publish_campaign_status",
    "record_owner_phase(TaskWatchdogOwnerPhase::ServicingHashrate)",
  ].join("\n")],
  ["firmware/bitaxe/src/production_mining_session/campaign_status/publication.rs", [
    "CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS: u64 = 1_000",
    "pub(crate) struct CampaignStatusPublicationSchedule {",
  ].join("\n")],
  ["firmware/bitaxe/src/task_watchdog_observation.rs", [
    "const COHERENT_READ_ATTEMPTS: usize = 8;",
    "TaskWatchdogReadOutcome::HistoryPoisoned",
    "TaskWatchdogReadOutcome::RetryExhausted",
    "publication_sequence: AtomicU32,",
    "pub(crate) fn coherent_observation()",
  ].join("\n")],
  ["firmware/bitaxe/sdkconfig.defaults", "CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5"],
  ["crates/bitaxe-safety/src/power.rs", [
    "pub const INPUT_VOLTAGE_NOMINAL_VOLTS: f64 = 5.0;",
    "pub const INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;",
  ].join("\n")],
]);

const okResult = {
  schema: "mining-campaign-result-v15",
  status: "accepted",
  terminal_category: "submit_response_observed",
  stage: "live-share",
  profile: "conservative",
  duration_seconds: 600,
  runtime_identity: "trusted",
  safe_stop: "confirmed",
  usb_cleanup: "ready",
  watchdog_failure: "none",
  watchdog_read_outcome: "stable",
  watchdog_owner_phase: "waiting_inbox",
  watchdog_wait_state: "within_deadline",
  runtime_attestation_parse_failure: "none",
  runtime_attestation_parse_failure_counts: {
    missing_marker: 0,
    malformed_token: 0,
    duplicate_field: 0,
    unknown_field: 0,
    missing_field: 0,
    invalid_field: 0,
    incomplete_readiness: 0,
  },
};

type Fixture = {
  readonly root: string;
  readonly planSha256: string;
  readonly options: HashrateMonitorEvidenceOptions;
};

function sha256(document: string): string {
  return createHash("sha256").update(document).digest("hex");
}

async function writeProtected(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-hashrate-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  for (const [relative, document] of sourceDocuments) {
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${document}\n`);
  }
  const reference = path.join(root, "reference/esp-miner/main/tasks/hashrate_monitor_task.c");
  await mkdir(path.dirname(reference), { recursive: true });
  await writeFile(reference, [
    "#define HASHRATE_UNIT 0x100000uLL",
    "#define POLL_RATE 1000",
    "#define HASHRATE_1M_SIZE (60000 / POLL_RATE)",
    "void update_hash_counter(measurement_t * measurement, uint32_t value, uint64_t time_us)",
    ...Array.from({ length: 7 }, () => "update_hash_counter"),
    "ASIC_read_registers(GLOBAL_STATE);",
  ].join("\n"));
  const counterReference = path.join(root, "reference/esp-miner/components/stratum/utils.c");
  await mkdir(path.dirname(counterReference), { recursive: true });
  await writeFile(counterReference, [
    "#define HASH_CNT_LSB 0x100000000uLL",
    "float hashCounterToGhs(uint64_t duration_us, uint32_t counter)",
  ].join("\n"));
  const deviceReference = path.join(root, "reference/esp-miner/main/device_config.h");
  await mkdir(path.dirname(deviceReference), { recursive: true });
  await writeFile(deviceReference, [
    ".default_voltage_mv = 1200,",
    "FAMILY_ULTRA       = { .id = ULTRA,       .name = \"Ultra\",      .asic = ASIC_BM1366,   .asic_count = 1, .max_power =  25, .power_offset = 5,  .nominal_voltage = 5,",
  ].join("\n"));
  const powerReference = path.join(
    root,
    "reference/esp-miner/main/tasks/power_management_task.c",
  );
  await writeFile(powerReference, [
    "uint16_t voltage = nvs_config_get_u16(NVS_CONFIG_ASIC_VOLTAGE);",
    "VCORE_set_voltage(GLOBAL_STATE, (double) voltage / 1000.0);",
  ].join("\n"));
  const coordinatorReference = path.join(
    root,
    "reference/esp-miner/main/tasks/protocol_coordinator.c",
  );
  await writeFile(coordinatorReference, [
    'xTaskCreateWithCaps(stratum_v1_task, "stratum v1", 8192, (void *)gs, 5,',
  ].join("\n"));
  const planRelative = "docs/parity/work-plans/20260817T082220Z-STAT-001/PLAN.md";
  const plan = "- Parity row: `STAT-001`\n- Active task: `task-parity-stat001-hashrate-monitor`\n";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-stat001-hashrate-monitor | fixture",
    `Plan: \`${planRelative}\`.`,
    "Attempt: `attempt-015`.",
  ].join("\n"));
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  await writeFile(path.join(inputs, "package.json"), JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
  }));
  const wrapper = path.join(root, "scratch/stat001-hashrate-monitor/wrapper-015");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const output of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeProtected(path.join(wrapper, output), "");
  }
  return {
    root,
    planSha256: sha256(plan),
    options: {
      privateRoot: "scratch/stat001-hashrate-monitor/attempt-015",
      packageManifest: "inputs/package.json",
      wifiCredentials: "inputs/wifi.json",
      poolCredentials: "inputs/pool.json",
      detectorOutput: "scratch/stat001-hashrate-monitor/wrapper-015/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json",
      durationSeconds: 600,
      captureTimeoutSeconds: 30,
    },
  };
}

async function childProgram(
  value: Fixture,
  options: Readonly<{
    malformedTransport?: boolean;
    sealedFailure?: boolean;
    watchdogFailure?: string;
    watchdogReadOutcome?: string;
    failureTerminalCategory?: string;
    resultSchema?: string;
    watchdogOwnerPhase?: string;
    watchdogWaitState?: string;
    tamperedSeal?: boolean;
  }> = {},
): Promise<string> {
  const child = path.join(value.root, "child.mjs");
  const failureResult = {
    ...okResult,
    schema: options.resultSchema ?? okResult.schema,
    status: "failed",
    terminal_category: options.failureTerminalCategory
      ?? (options.watchdogFailure === undefined
        ? "runtime_identity_untrusted"
        : "watchdog_unresponsive"),
    watchdog_failure: options.watchdogFailure ?? "none",
    watchdog_read_outcome: options.watchdogReadOutcome ?? "stable",
    watchdog_owner_phase: options.watchdogOwnerPhase ?? "publishing_campaign_status",
    watchdog_wait_state: options.watchdogWaitState ?? "not_waiting",
    runtime_attestation_parse_failure: options.watchdogFailure === undefined
      ? "missing_marker"
      : "none",
    runtime_attestation_parse_failure_counts: {
      ...okResult.runtime_attestation_parse_failure_counts,
      missing_marker: options.watchdogFailure === undefined ? 1 : 0,
    },
    protected_runtime_text: "secret-device-origin private-worker",
  };
  await writeFile(child, `#!${nodeProgram}
import { createHash } from "node:crypto";
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
const digest = (value) => createHash("sha256").update(value).digest("hex");
if (args[0] === "mining-campaign") {
  if (args[args.indexOf("--stage") + 1] !== "live-share" || args[args.indexOf("--profile") + 1] !== "conservative") process.exit(5);
  const root = args[args.indexOf("--evidence-dir") + 1];
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
  const transport = { active_sample_count: 3, positive_coherent_count: 3, distinct_positive_count: 2, warm_rolling_window_count: 2, terminal_zero_confirmed: true };
  const network = JSON.stringify({ schema: "mining-campaign-network-continuity-v9", status: "accepted", watchdog_failure: "none", watchdog_read_outcome: "stable", watchdog_owner_phase: "waiting_inbox", watchdog_wait_state: "within_deadline", required_window_count: 20, covered_window_count: 20, hashrate_monitor: { monitor_cadence_ms: 1000, asic_count: 1, domain_count: 4, http: transport, websocket: ${options.malformedTransport === true ? "{ ...transport, distinct_positive_count: 1 }" : "transport"} } }) + "\\n";
  const result = JSON.stringify(${options.sealedFailure === true
    ? JSON.stringify(failureResult)
    : `{ ...${JSON.stringify(okResult)}, network_continuity_sha256: digest(network) }`}) + "\\n";
  const files = new Map([["campaign-diagnostics.private.json", "{}\\n"], ["campaign-flash.private.json", "{}\\n"], ["campaign-mining-diagnostics.private.json", "{}\\n"], ["campaign-network.private.json", network], ["campaign-observations.private.json", "{}\\n"], ["campaign-result.json", result], ["campaign-result.sha256", ${options.tamperedSeal === true ? '"0".repeat(64)' : "digest(result)"} + "\\n"]]);
  for (const [name, document] of files) { const candidate = path.join(root, name); await writeFile(candidate, document, { mode: 0o600 }); await chmod(candidate, 0o600); }
  ${options.sealedFailure === true ? 'process.stderr.write("secret-child-output private-worker\\n"); process.exitCode = 9;' : ""}
} else if (args[0] === "-C") {
  process.stdout.write(${JSON.stringify(`${referenceCommit}\n`)});
} else if (args[0] === "status") {
  process.stdout.write("");
} else if (args[0] === "rev-parse") {
  process.stdout.write(${JSON.stringify(`${sourceCommit}\n`)});
} else {
  process.exitCode = 2;
}
`);
  await chmod(child, 0o700);
  return child;
}

async function captureError(promise: Promise<unknown>): Promise<HashrateMonitorEvidenceError> {
  try {
    await promise;
    assert.fail("expected hashrate evidence failure");
  } catch (error) {
    assert.ok(error instanceof HashrateMonitorEvidenceError);
    return error;
  }
}

test("admissible conservative campaign and independent validator publish only closed evidence", async () => {
  // Arrange
  const value = await fixture("real-child");
  const child = await childProgram(value);
  try {
    // Act
    const evidence = await captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    );

    // Assert
    assert.equal(evidence.attempt_ordinal, 15);
    assert.equal(evidence.hashrate.http.distinct_positive_count, 2);
    assert.equal(evidence.source.source_path_count, 18);
    assert.equal((await stat(path.join(value.root, value.options.projection))).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(path.join(value.root, value.options.projection), "utf8"),
      /private-port|credential|pool_url|worker|device_url|serial/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("consumed attempt-014 protected root is rejected before capture", async () => {
  // Arrange
  const value = await fixture("consumed-root");
  const child = await childProgram(value);
  const options = {
    ...value.options,
    privateRoot: "scratch/stat001-hashrate-monitor/attempt-014",
  };

  try {
    // Act
    const error = await captureError(captureHashrateMonitorEvidence(
      value.root,
      options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(
      stat(path.join(value.root, "scratch/stat001-hashrate-monitor/attempt-014")),
      { code: "ENOENT" },
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("current immutable task and production/reference sources pass admission", async () => {
  // Arrange
  const root = process.env["RUNFILES_DIR"] === undefined
    ? workspace
    : path.join(process.env["RUNFILES_DIR"], "_main");

  // Act / Assert
  await validateHashrateMonitorTaskAndSources(
    root,
    "da3c3eb4fa4d4a9f949307db2b0e6e905f4e905ad31352a271a0b52ff1096205",
  );
});

test("incomplete transport evidence is rejected before publication", async () => {
  // Arrange
  const value = await fixture("incomplete");
  const child = await childProgram(value, { malformedTransport: true });

  try {
    // Act
    const error = await captureError(captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(path.join(value.root, value.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("sealed non-ready campaign publishes only the closed parse diagnostic", async () => {
  // Arrange
  const value = await fixture("sealed-failure");
  const child = await childProgram(value, { sealedFailure: true });

  try {
    // Act
    const error = await captureError(captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.deepEqual(error.publicValue, {
      stage: "hashrate_monitor_capture",
      projection_published: false,
      runtime_attestation_parse_failure: "missing_marker",
    });
    assert.doesNotMatch(
      JSON.stringify(error.publicValue),
      /secret|device-origin|private-worker/u,
    );
    await assert.rejects(readFile(path.join(value.root, value.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("unsealed watchdog campaign withholds its phase and failure diagnostic", async () => {
  // Arrange
  const value = await fixture("tampered-seal");
  const child = await childProgram(value, {
    sealedFailure: true,
    tamperedSeal: true,
    watchdogFailure: "watchdog_feed_stale",
  });

  try {
    // Act
    const error = await captureError(captureHashrateMonitorEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      validatorProgram,
      value.planSha256,
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.deepEqual(error.publicValue, {
      stage: "hashrate_monitor_capture",
      projection_published: false,
    });
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("every sealed watchdog failure publishes only its closed earliest discriminator", async () => {
  for (const watchdogFailure of [
    "supervisor_unavailable",
    "checkpoint_unhealthy",
    "checkpoint_sequence_missing",
    "watchdog_reason_missing",
    "watchdog_unproved",
    "watchdog_snapshot_retry_exhausted",
    "watchdog_snapshot_history_poisoned",
    "watchdog_read_outcome_unknown",
    "watchdog_invalid_observation",
    "watchdog_subscription_failed",
    "watchdog_feed_failed",
    "watchdog_unsubscription_failed",
    "watchdog_unsubscribed",
    "watchdog_reason_unknown",
    "watchdog_participation_inconsistent",
    "watchdog_feed_sequence_missing",
    "watchdog_feed_age_missing",
    "watchdog_feed_stale",
    "watchdog_owner_phase_unknown",
    "watchdog_wait_state_unknown",
    "http_checkpoint_not_advanced",
    "http_feed_not_advanced",
    "websocket_checkpoint_not_advanced",
    "websocket_feed_not_advanced",
  ] as const) {
    // Arrange
    const value = await fixture(`sealed-watchdog-${watchdogFailure}`);
    const child = await childProgram(value, { sealedFailure: true, watchdogFailure });

    try {
      // Act
      const error = await captureError(captureHashrateMonitorEvidence(
        value.root,
        value.options,
        createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
        child,
        child,
        validatorProgram,
        value.planSha256,
      ));

      // Assert
      assert.equal(error.category, "hardware_blocked");
      assert.deepEqual(error.publicValue, {
        stage: "hashrate_monitor_capture",
        projection_published: false,
        runtime_attestation_parse_failure: "none",
        watchdog_failure: watchdogFailure,
        watchdog_read_outcome: "stable",
        watchdog_owner_phase: "publishing_campaign_status",
        watchdog_wait_state: "not_waiting",
      });
      assert.doesNotMatch(
        JSON.stringify(error.publicValue),
        /secret|device-origin|private-worker/u,
      );
    } finally {
      await rm(value.root, { recursive: true });
    }
  }
});

test("watchdog diagnostic requires the new sealed schema and matching terminal category", async () => {
  for (const [name, options] of [
    ["old-schema", {
      sealedFailure: true,
      watchdogFailure: "http_feed_not_advanced",
      resultSchema: "mining-campaign-result-v14",
    }],
    ["wrong-category", {
      sealedFailure: true,
      watchdogFailure: "http_feed_not_advanced",
      failureTerminalCategory: "network_correlation_failed",
    }],
    ["unknown-label", {
      sealedFailure: true,
      watchdogFailure: "private-sequence-42",
    }],
    ["missing-watchdog-cause", {
      sealedFailure: true,
      watchdogFailure: "none",
    }],
    ["unknown-owner-phase", {
      sealedFailure: true,
      watchdogFailure: "watchdog_feed_stale",
      watchdogOwnerPhase: "private-phase-42",
    }],
    ["unknown-read-outcome", {
      sealedFailure: true,
      watchdogFailure: "watchdog_feed_stale",
      watchdogReadOutcome: "private-read-42",
    }],
    ["unknown-wait-state", {
      sealedFailure: true,
      watchdogFailure: "watchdog_feed_stale",
      watchdogWaitState: "private-wait-42",
    }],
  ] as const) {
    // Arrange
    const value = await fixture(name);
    const child = await childProgram(value, options);

    try {
      // Act
      const error = await captureError(captureHashrateMonitorEvidence(
        value.root,
        value.options,
        createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
        child,
        child,
        validatorProgram,
        value.planSha256,
      ));

      // Assert
      assert.equal(error.category, "hardware_blocked");
      assert.deepEqual(error.publicValue, {
        stage: "hashrate_monitor_capture",
        projection_published: false,
      });
    } finally {
      await rm(value.root, { recursive: true });
    }
  }
});

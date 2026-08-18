import assert from "node:assert/strict";
import { chmod, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  captureHashrateMonitorEvidence,
  HashrateMonitorEvidenceError,
  validateHashrateMonitorTaskAndSources,
} from "./hashrate-monitor-evidence.js";
import {
  fixture,
  nodeProgram,
  okDiagnostics,
  okResult,
  referenceCommit,
  sourceCommit,
  validatorProgram,
  workspace,
  type Fixture,
} from "./hashrate-monitor-evidence.test-support.js";
import { createLocalProcessPort } from "./process.js";

async function childProgram(
  value: Fixture,
  options: Readonly<{
    malformedTransport?: boolean;
    finalTerminalConsumed?: boolean;
    sealedFailure?: boolean;
    watchdogFailure?: string;
    watchdogReadOutcome?: string;
    failureTerminalCategory?: string;
    resultSchema?: string;
    watchdogOwnerPhase?: string;
    watchdogOwnerSubphase?: string;
    watchdogWaitState?: string;
    tamperedSeal?: boolean;
    panicSignature?: string;
    panicTaskFamily?: string;
    panicSignatureCount?: number;
    mixedResetReason?: string;
    tamperedDiagnosticsDigest?: boolean;
  }> = {},
): Promise<string> {
  const child = path.join(value.root, "child.mjs");
  const diagnostics = {
    ...okDiagnostics,
    runtime_attestation_mixed_reset_reason: options.mixedResetReason ?? "none",
    panic_signature: options.panicSignature ?? "none",
    panic_task_family: options.panicTaskFamily ?? "none",
    panic_signature_count: options.panicSignatureCount ?? 0,
  };
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
    watchdog_owner_subphase: options.watchdogOwnerSubphase ?? "unavailable",
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
  const network = JSON.stringify({ schema: "mining-campaign-network-continuity-v12", status: "accepted", correlation_failure: "none", watchdog_failure: "none", watchdog_read_outcome: "stable", watchdog_owner_phase: "waiting_inbox", watchdog_owner_subphase: "unavailable", watchdog_wait_state: "within_deadline", required_window_count: 20, covered_window_count: 20, terminal_settlement: "accepted_after_serial_close", terminal_close_requested: true, terminal_consumed_observed: true, final_terminal_consumed: ${options.finalTerminalConsumed ?? true}, serial_finished_observed: true, hashrate_monitor: { monitor_cadence_ms: 1000, asic_count: 1, domain_count: 4, http: transport, websocket: ${options.malformedTransport === true ? "{ ...transport, distinct_positive_count: 1 }" : "transport"} } }) + "\\n";
  const diagnostics = JSON.stringify(${JSON.stringify(diagnostics)}) + "\\n";
  const resultData = ${JSON.stringify(options.sealedFailure === true ? failureResult : okResult)};
  const result = JSON.stringify({
    ...resultData,
    diagnostics_sha256: ${options.tamperedDiagnosticsDigest === true ? '"0".repeat(64)' : "digest(diagnostics)"},
    ...(${options.sealedFailure === true ? "true" : "false"} ? {} : { network_continuity_sha256: digest(network) }),
  }) + "\\n";
  const files = new Map([["campaign-diagnostics.private.json", diagnostics], ["campaign-flash.private.json", "{}\\n"], ["campaign-mining-diagnostics.private.json", "{}\\n"], ["campaign-network.private.json", network], ["campaign-observations.private.json", "{}\\n"], ["campaign-result.json", result], ["campaign-result.sha256", ${options.tamperedSeal === true ? '"0".repeat(64)' : "digest(result)"} + "\\n"]]);
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
    assert.equal(evidence.attempt_ordinal, 19);
    assert.equal(evidence.hashrate.http.distinct_positive_count, 2);
    assert.equal(evidence.source.source_path_count, 25);
    assert.equal((await stat(path.join(value.root, value.options.projection))).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(path.join(value.root, value.options.projection), "utf8"),
      /private-port|credential|pool_url|worker|device_url|serial/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("accepted campaign with a mixed reset cannot publish", async () => {
  // Arrange
  const value = await fixture("accepted-mixed-reset");
  const child = await childProgram(value, {
    mixedResetReason: "panic",
    panicSignature: "unknown",
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
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(path.join(value.root, value.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("accepted transport evidence without final consumed handoff cannot publish", async () => {
  // Arrange
  const value = await fixture("missing-final-consumed");
  const child = await childProgram(value, { finalTerminalConsumed: false });

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

test("consumed attempt-018 protected root is rejected before capture", async () => {
  // Arrange
  const value = await fixture("consumed-root");
  const child = await childProgram(value);
  const options = {
    ...value.options,
    privateRoot: "scratch/stat001-hashrate-monitor/attempt-018",
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
      stat(path.join(value.root, "scratch/stat001-hashrate-monitor/attempt-018")),
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
    "b9bc554eb3e49c685bcbd7a852a754febf015228df4ae89efe6e6b951eb65e24",
    "archived",
  );
});

test("archived task cannot satisfy the pre-effect active admission", async () => {
  // Arrange
  const value = await fixture("archived-task");
  const taskPath = path.join(value.root, "TASKS.md");
  const archivePath = path.join(value.root, "TASKS.archive.md");
  const taskDocument = await readFile(taskPath, "utf8");
  await writeFile(taskPath, "# Active tasks\n");
  await writeFile(archivePath, taskDocument);

  try {
    // Act / Assert
    await assert.rejects(validateHashrateMonitorTaskAndSources(
      value.root,
      value.planSha256,
    ));
    await validateHashrateMonitorTaskAndSources(
      value.root,
      value.planSha256,
      "archived",
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
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

test("sealed panic campaign publishes only the closed panic tuple", async () => {
  // Arrange
  const value = await fixture("sealed-panic");
  const child = await childProgram(value, {
    sealedFailure: true,
    mixedResetReason: "panic",
    panicSignature: "stack_overflow",
    panicTaskFamily: "production_mining_session",
    panicSignatureCount: 2,
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
      runtime_attestation_parse_failure: "missing_marker",
      panic_signature: "stack_overflow",
      panic_task_family: "production_mining_session",
      panic_signature_count: 2,
    });
    assert.doesNotMatch(
      JSON.stringify(error.publicValue),
      /secret|device-origin|private-worker|address|backtrace|task_name/u,
    );
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("unbound panic diagnostics are withheld", async () => {
  // Arrange
  const value = await fixture("unbound-panic");
  const child = await childProgram(value, {
    sealedFailure: true,
    mixedResetReason: "panic",
    panicSignature: "stack_overflow",
    panicTaskFamily: "production_mining_session",
    panicSignatureCount: 1,
    tamperedDiagnosticsDigest: true,
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
    "watchdog_owner_subphase_unknown",
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
        watchdog_owner_subphase: "unavailable",
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
      resultSchema: "mining-campaign-result-v15",
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
    ["unknown-owner-subphase", {
      sealedFailure: true,
      watchdogFailure: "watchdog_feed_stale",
      watchdogOwnerSubphase: "private-effect-42",
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

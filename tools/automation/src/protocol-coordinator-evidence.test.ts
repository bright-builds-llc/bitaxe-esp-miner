import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";
import {
  projectProtocolCoordinatorEvidence,
  ProtocolCoordinatorEvidenceError,
  type ProtocolCoordinatorSourceValidators,
} from "./protocol-coordinator-evidence.js";

const attemptCommit = "3e0966a140edbff1a14d2a48ca63d140649762c0";
const currentCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const sourcePaths = [
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
  "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
  "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json",
  "docs/parity/evidence/str001-socket/stratum-socket-projection.json",
] as const;
const coordinatorPaths = [
  "crates/bitaxe-core/src/runtime_orchestration.rs",
  "crates/bitaxe-stratum/src/v1/recovery_policy.rs",
  "crates/bitaxe-stratum/src/v1/production_session/runtime.rs",
  "crates/bitaxe-stratum/src/v1/production_session/orchestration.rs",
  "crates/bitaxe-stratum/src/v1/production_session/runtime/asic.rs",
  "firmware/bitaxe/src/production_mining_session.rs",
  "firmware/bitaxe/src/production_mining_session/asic_worker.rs",
] as const;
const validators: ProtocolCoordinatorSourceValidators = {
  initialization: "initialization-validator",
  workSend: "work-send-validator",
  resultParsing: "result-parsing-validator",
  socket: "socket-validator",
  evidence: "evidence-validator",
};

function ok(stdout = ""): ProcessOutcome {
  return { exitCode: 0, stdout, stderr: "", timedOut: false };
}

function digest(document: string): string {
  return createHash("sha256").update(document).digest("hex");
}

function campaignFields(currentSourceCommit: string) {
  return {
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
    campaign_terminal_category: "submit_response_observed",
    submit_outcome: "accepted",
    safety_status: "fresh",
    mine_on_boot_disabled: true,
    safe_stop_confirmed: true,
    lease_cleanup_confirmed: true,
    usb_cleanup_ready: true,
    hardware_rerun_used: false,
    redaction_status: "passed",
  };
}

function sourceDocuments() {
  const initialization = `${JSON.stringify({
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    ...campaignFields("b".repeat(40)),
    initialization: {
      all_preparation_steps_completed: true,
      mining_ready_initialization_completed: true,
      live_initialized_work_observed: true,
    },
  }, null, 2)}\n`;
  const initializationDigest = digest(initialization);
  const workSend = `${JSON.stringify({
    schema_version: "bitaxe-asic-work-send-evidence-v1",
    ...campaignFields("d".repeat(40)),
    source: { initialization_projection_sha256: initializationDigest },
    work_send: {
      production_ready_gate_required: true,
      live_work_observed: true,
      qualified_result_observed: true,
      accepted_submit_observed: true,
    },
  }, null, 2)}\n`;
  const workSendDigest = digest(workSend);
  const resultParsing = `${JSON.stringify({
    schema_version: "bitaxe-asic-result-parsing-evidence-v1",
    ...campaignFields("e".repeat(40)),
    source: { work_send_projection_sha256: workSendDigest },
    result_parsing: {
      job_lookup_validation: true,
      core_validation: true,
      live_qualified_result_observed: true,
      accepted_submit_observed: true,
      correlation_semantics_compatible: true,
    },
  }, null, 2)}\n`;
  const socket = `${JSON.stringify({
    schema_version: "bitaxe-stratum-socket-evidence-v1",
    ...campaignFields("f".repeat(40)),
    source: { initialization_projection_sha256: initializationDigest },
    socket: {
      transport_epoch_isolation: true,
      authorized_session_required_before_submit: true,
      accepted_submit_observed: true,
    },
  }, null, 2)}\n`;
  return {
    documents: [initialization, workSend, resultParsing, socket] as const,
    admittedDigests: {
      initialization: initializationDigest,
      workSend: workSendDigest,
      resultParsing: digest(resultParsing),
      socket: digest(socket),
    },
  };
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-protocol-coordinator-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sources = sourceDocuments();
  for (const [index, sourcePath] of sourcePaths.entries()) {
    const destination = path.join(root, sourcePath);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, sources.documents[index] ?? "");
  }
  const projection = path.join(root,
    "docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json");
  return {
    root,
    projection,
    admittedDigests: sources.admittedDigests,
    options: {
      initializationProjection: path.join(root, sourcePaths[0]),
      workSendProjection: path.join(root, sourcePaths[1]),
      resultParsingProjection: path.join(root, sourcePaths[2]),
      socketProjection: path.join(root, sourcePaths[3]),
      attemptSourceCommit: attemptCommit,
      projection,
    },
  };
}

function coordinatorSource(sourcePath: string): string {
  if (sourcePath.endsWith("runtime_orchestration.rs")) {
    return "pub const PRODUCTION_REREAD_CADENCE_MS: u64 = 1_000;\n";
  }
  if (sourcePath.endsWith("recovery_policy.rs")) {
    return [
      "return Some(ProductionSessionBlocker::OperatorPaused);",
      "return Some(ProductionSessionBlocker::NetworkUnavailable);",
      "return Some(ProductionSessionBlocker::StratumV1Unsupported);",
      "return Some(ProductionSessionBlocker::SafetyPrerequisitesStale);",
      "return Some(ProductionSessionBlocker::CampaignLeaseUnavailable);",
      "return Some(ProductionSessionBlocker::ActuationUnqualified);",
    ].join("\n");
  }
  if (sourcePath.endsWith("production_session/runtime.rs")) {
    return [
      "effects.push(ProductionSessionEffect::PrepareHardware {",
      "if self.hardware_state != MiningHardwareState::Ready {",
      "RecoveryAction::BlockSubmissions,",
      "RecoveryAction::InvalidateWorkAndSubmissions,",
      "RecoveryAction::StopAsicInteraction,",
      "effects.push(ProductionSessionEffect::SafeStopHardware { lease_id });",
    ].join("\n");
  }
  if (sourcePath.endsWith("production_session/orchestration.rs")) {
    return [
      "self.bridge.note_listener_armed();",
      "effects.push(ProductionSessionEffect::DispatchAsic {",
      "runtime.submits.insert(request_id, PendingSubmit { intent });",
      "self.stop_after_first_submit_response(effects)?;",
    ].join("\n");
  }
  if (sourcePath.endsWith("production_session/runtime/asic.rs")) {
    return [
      ".apply_bridge_observation_with_receipt(observation)",
      "BridgeObservationOutcome::SubmitQueued => AsicCorrelation::Correlated,",
    ].join("\n");
  }
  if (sourcePath.endsWith("production_mining_session.rs")) {
    return [
      "const NOTIFICATION_CAPACITY: usize = 16;",
      "let mut session = ProductionMiningSession::new();",
      "while let Some(event) = events.pop_front() {",
      "task_watchdog.feed(crate::runtime_uptime::millis());",
    ].join("\n");
  }
  return [
    "AsicWorkerCommand::Dispatch {",
    "                            generation,",
    "                            valid_jobs,",
    "                            command,",
    "                        } => match executor.maybe_execute(command, &valid_jobs) {",
    "ProductionSessionEffect::DispatchAsic {",
    "                generation,",
    "                valid_jobs,",
    "                command,",
    "            } => Ok(AsicWorkerCommand::Dispatch {",
  ].join("\n");
}

function fakePort(options: {
  readonly moduleDrift?: boolean;
  readonly semanticDrift?: boolean;
  readonly dirty?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
  readonly asicWorkerShape?: "missing" | "duplicate" | "reordered" | "unbound";
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === validators.evidence) {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M coordinator.rs\n" : "");
    if (spec.args[0] === "diff" && options.moduleDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      const sourcePath = coordinatorPaths.find((candidate) => target.endsWith(candidate));
      if (sourcePath !== undefined) {
        let document = coordinatorSource(sourcePath);
        if (sourcePath.endsWith("asic_worker.rs")) {
          const [consumer, mapper] = document.split("ProductionSessionEffect::DispatchAsic {");
          if (options.asicWorkerShape === "missing") document = consumer ?? "";
          if (options.asicWorkerShape === "duplicate") document = `${document}\n${consumer ?? ""}`;
          if (options.asicWorkerShape === "reordered") {
            document = `ProductionSessionEffect::DispatchAsic {${mapper ?? ""}\n${consumer ?? ""}`;
          }
          if (options.asicWorkerShape === "unbound") {
            document = document.replace("executor.maybe_execute(command, &valid_jobs)", "executor.maybe_execute(other, &valid_jobs)");
          }
        }
        if (options.semanticDrift && sourcePath.endsWith("runtime_orchestration.rs")) {
          document = document.replace("PRODUCTION_REREAD_CADENCE_MS: u64 = 1_000", "changed");
        }
        return ok(document);
      }
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<ProtocolCoordinatorEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof ProtocolCoordinatorEvidenceError);
    return error;
  }
}

test("accepted lifecycle emits only closed protocol coordinator evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectProtocolCoordinatorEvidence(
    value.root, value.options, fakePort(), "git", validators, value.admittedDigests,
  );

  // Assert
  assert.equal(evidence.coordinator.readiness_gate_count, 6);
  assert.equal(evidence.coordinator.accepted_submit_observed, true);
  assert.equal(evidence.coordinator.ordered_terminal_safe_stop, true);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /password|credential|device_url|endpoint|response_bytes|local_path|pool_url|wifi_ssid|serial_port/iu);
});

test("malformed or digest-drifted sources withhold public evidence", async () => {
  for (const [name, content] of [["malformed", "not-json\n"], ["drifted", "{}\n"]] as const) {
    // Arrange
    const value = await fixture(name);
    await writeFile(value.options.socketProjection, content);

    // Act
    const error = await captureError(projectProtocolCoordinatorEvidence(
      value.root, value.options, fakePort(), "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("module, semantic, and dirty-path drift withhold public evidence", async () => {
  for (const [name, port] of [
    ["module", fakePort({ moduleDrift: true })],
    ["semantic", fakePort({ semanticDrift: true })],
    ["dirty", fakePort({ dirty: true })],
  ] as const) {
    // Arrange
    const value = await fixture(name);

    // Act
    const error = await captureError(projectProtocolCoordinatorEvidence(
      value.root, value.options, port, "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("ASIC worker dispatch spans must be complete, unique, ordered, and bound", async () => {
  for (const asicWorkerShape of ["missing", "duplicate", "reordered", "unbound"] as const) {
    // Arrange
    const value = await fixture(asicWorkerShape);

    // Act
    const error = await captureError(projectProtocolCoordinatorEvidence(
      value.root, value.options, fakePort({ asicWorkerShape }), "git", validators,
      value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("validator rejection and launch failure preserve typed failure", async () => {
  for (const [name, port, category] of [
    ["validator", fakePort({ validatorFailure: true }), "evidence_invalid"],
    ["launch", fakePort({ launchFailure: true }), "process_failed"],
  ] as const) {
    // Arrange
    const value = await fixture(name);

    // Act
    const error = await captureError(projectProtocolCoordinatorEvidence(
      value.root, value.options, port, "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, category);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("real child validators must accept every source and candidate", async () => {
  // Arrange
  const value = await fixture("real-child");
  const validator = path.join(value.root, "validator-child.sh");
  await writeFile(validator, "#!/bin/sh\ntest -s \"$1\"\n");
  await chmod(validator, 0o700);
  const localPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const gitPort = fakePort();
  const processPort: ProcessPort = {
    loadEspEnvironment: () => localPort.loadEspEnvironment(),
    run: (spec, maybeTimeoutMs) => spec.program === "git-fixture"
      ? gitPort.run(spec, maybeTimeoutMs)
      : localPort.run(spec, maybeTimeoutMs),
  };
  const realValidators: ProtocolCoordinatorSourceValidators = {
    initialization: validator,
    workSend: validator,
    resultParsing: validator,
    socket: validator,
    evidence: validator,
  };

  // Act
  const evidence = await projectProtocolCoordinatorEvidence(
    value.root, value.options, processPort, "git-fixture", realValidators,
    value.admittedDigests,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-protocol-coordinator-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});

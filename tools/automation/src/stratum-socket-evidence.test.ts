import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  projectStratumSocketEvidence,
  StratumSocketEvidenceError,
} from "./stratum-socket-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";

const attemptCommit = "a".repeat(40);
const sourceCommit = "b".repeat(40);
const currentCommit = "c".repeat(40);
const referenceCommit = "d".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const transportSource = `
const COMMAND_CAPACITY: usize = 8;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_millis(50);
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const READ_BUFFER_BYTES: usize = 2 * 1024;
stream.set_nodelay(true)?;
if connection.transport_epoch != transport_epoch {
`;
const ownerSource = `
OwnerInboxMessage::Transport(event) => match event {
ProductionSessionEvent::TransportConnected {
ProductionSessionEvent::TransportFailed {
ProductionSessionEvent::TransportBytes {
ProductionSessionEvent::TransportClosed {
OwnerInboxMessage::Asic(event) => match event {
ProductionSessionEffect::ConnectPool {
ProductionSessionEffect::WritePoolLine {
effect @ (ProductionSessionEffect::ApplyVersionMask
ProductionSessionEffect::ClosePoolConnection {
ProductionSessionEffect::SafeStopHardware {
`;
const lifecycleSource = `
PendingRequestKind::Submit => {
let Some(pending) = maybe_pending else {
current_generation != Some(pending.intent.generation)
SubmitResponseObservation::Response(response)
SubmitClassification::Accepted
PendingRequestKind::Runtime(kind) => {
`;

function sourceEvidence(complete = true) {
  return {
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: sourceCommit,
    reference_commit: referenceCommit,
    source_task_sha256: "e".repeat(64),
    workflow: { schema_version: "bitaxe-workflow-identity-v1", command: "project-asic-initialization-evidence", request_sha256: "f".repeat(64) },
    attempt: { campaign_result_sha256: "0".repeat(64), diagnostics_sha256: "1".repeat(64), observations_sha256: "2".repeat(64), result_seal_valid: true, private_digests_valid: true, protected_modes_valid: true },
    initialization: {
      planned_step_count: 9,
      accepted_preparation_event_count: 18,
      invalid_preparation_event_count: 0,
      terminal_preparation_step: "retain_production_uart",
      terminal_preparation_outcome: "completed",
      all_preparation_steps_completed: complete,
      exactly_one_chip_detected: true,
      mining_ready_initialization_completed: true,
      production_uart_retained: true,
      live_initialized_work_observed: true,
      initialization_paths_unchanged: true,
      compatible_path_count: 7,
    },
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
    serial_outcome_detail: "clean",
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

async function fixture(name: string, complete = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-stratum-socket-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(root,
    "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json");
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  const sourceDocument = `${JSON.stringify(sourceEvidence(complete), null, 2)}\n`;
  await writeFile(sourceProjection, sourceDocument);
  const projection = path.join(root, "docs/parity/evidence/str001-socket/evidence.json");
  return {
    root,
    projection,
    sourceSha256: createHash("sha256").update(sourceDocument).digest("hex"),
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: {
  readonly moduleDrift?: boolean;
  readonly semanticDrift?: boolean;
  readonly dirty?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === "validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M transport.rs\n" : "");
    if (spec.args[0] === "diff" && options.moduleDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      if (target.endsWith("transport.rs")) {
        return ok(options.semanticDrift ? transportSource.replace("stream.set_nodelay(true)?;", "") : transportSource);
      }
      if (target.endsWith("production_mining_session.rs")) {
        return ok(options.semanticDrift && target.startsWith(currentCommit)
          ? ownerSource.replace("ProductionSessionEvent::TransportBytes {", "changed")
          : ownerSource);
      }
      if (target.endsWith("orchestration.rs")) return ok(lifecycleSource);
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<StratumSocketEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof StratumSocketEvidenceError);
    return error;
  }
}

test("accepted session emits only closed Stratum socket evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectStratumSocketEvidence(
    value.root, value.options, fakePort(), "git", "source-validator", "validator",
    value.sourceSha256,
  );

  // Assert
  assert.equal(evidence.socket.connect_timeout_ms, 5000);
  assert.equal(evidence.socket.accepted_submit_observed, true);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /password|credential|device_url|endpoint|response_bytes|local_path|pool_url|wifi_ssid|serial_port/iu);
});

test("malformed or incomplete source withholds public evidence", async () => {
  for (const malformed of [false, true]) {
    // Arrange
    const value = await fixture(malformed ? "malformed" : "incomplete", malformed);
    if (malformed) await writeFile(value.options.sourceProjection, "not-json\n");

    // Act
    const error = await captureError(projectStratumSocketEvidence(
      value.root, value.options, fakePort(), "git", "source-validator", "validator",
      value.sourceSha256,
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
    const error = await captureError(projectStratumSocketEvidence(
      value.root, value.options, port, "git", "source-validator", "validator",
      value.sourceSha256,
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
    const error = await captureError(projectStratumSocketEvidence(
      value.root, value.options, port, "git", "source-validator", "validator",
      value.sourceSha256,
    ));

    // Assert
    assert.equal(error.category, category);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("real child validators must accept source and candidate", async () => {
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

  // Act
  const evidence = await projectStratumSocketEvidence(
    value.root, value.options, processPort, "git-fixture", validator, validator,
    value.sourceSha256,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-stratum-socket-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});

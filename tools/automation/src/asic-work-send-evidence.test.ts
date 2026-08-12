import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicWorkSendEvidenceError,
  projectAsicWorkSendEvidence,
} from "./asic-work-send-evidence.js";
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

const workerSource = `prefix
                        AsicWorkerCommand::Dispatch {
                            generation,
                        } => dispatch(generation),
                        AsicWorkerCommand::Poll {
suffix
`;

const adapterSource = `prefix
            Bm1366ProductionCommand::SendProductionWork(_) => {
                execute_write()?;
            }
            Bm1366ProductionCommand::ReadProductionResult => {
middle
        Bm1366AdapterAction::WriteFrame(frame) => {
            uart.write_frame(frame.as_ref())
        }
        Bm1366AdapterAction::ReadExact { len, timeout_ms } => {
suffix
`;

function sourceEvidence(accepted = true) {
  return {
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: sourceCommit,
    reference_commit: referenceCommit,
    source_task_sha256: "e".repeat(64),
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-initialization-evidence",
      request_sha256: "f".repeat(64),
    },
    attempt: {
      campaign_result_sha256: "0".repeat(64),
      diagnostics_sha256: "1".repeat(64),
      observations_sha256: "2".repeat(64),
      result_seal_valid: true,
      private_digests_valid: true,
      protected_modes_valid: true,
    },
    initialization: {
      planned_step_count: 9,
      accepted_preparation_event_count: 18,
      invalid_preparation_event_count: 0,
      terminal_preparation_step: "retain_production_uart",
      terminal_preparation_outcome: "completed",
      all_preparation_steps_completed: true,
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
    submit_outcome: accepted ? "accepted" : "rejected",
    safety_status: "fresh",
    mine_on_boot_disabled: true,
    safe_stop_confirmed: true,
    lease_cleanup_confirmed: true,
    usb_cleanup_ready: true,
    hardware_rerun_used: false,
    redaction_status: "passed",
  };
}

async function fixture(name: string, accepted = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-work-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(
    root,
    "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
  );
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  await writeFile(sourceProjection, `${JSON.stringify(sourceEvidence(accepted), null, 2)}\n`);
  const projection = path.join(root, "docs/parity/evidence/asic003-work-send/evidence.json");
  return {
    root,
    projection,
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: { readonly spanDrift?: boolean; readonly dirty?: boolean } = {}) {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M work.rs\n" : "");
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      if (target.endsWith("asic_worker.rs")) return ok(workerSource);
      if (target.endsWith("production.rs")) {
        const source = options.spanDrift && target.startsWith(currentCommit)
          ? adapterSource.replace("execute_write()?;", "changed_write()?;")
          : adapterSource;
        return ok(source);
      }
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<AsicWorkSendEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicWorkSendEvidenceError);
    return error;
  }
}

test("accepted live source emits only closed work-send evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectAsicWorkSendEvidence(
    value.root,
    value.options,
    fakePort(),
    "git",
    "source-validator",
    "validator",
  );

  // Assert
  assert.equal(evidence.work_send.frame_length_bytes, 88);
  assert.equal(evidence.work_send.dispatch_spans_unchanged, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(
    await readFile(value.projection, "utf8"),
    /pool|wifi|device_url|endpoint|nonce|difficulty|credential|frame_bytes|local_path/iu,
  );
});

test("nonaccepted source withholds public evidence", async () => {
  // Arrange
  const value = await fixture("nonaccepted", false);

  // Act
  const error = await captureError(projectAsicWorkSendEvidence(
    value.root,
    value.options,
    fakePort(),
    "git",
    "source-validator",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("dispatch span drift withholds public evidence", async () => {
  // Arrange
  const value = await fixture("span-drift");

  // Act
  const error = await captureError(projectAsicWorkSendEvidence(
    value.root,
    value.options,
    fakePort({ spanDrift: true }),
    "git",
    "source-validator",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("dirty work-send paths withhold public evidence", async () => {
  // Arrange
  const value = await fixture("dirty");

  // Act
  const error = await captureError(projectAsicWorkSendEvidence(
    value.root,
    value.options,
    fakePort({ dirty: true }),
    "git",
    "source-validator",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("real child validators must accept both source and candidate files", async () => {
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
  const evidence = await projectAsicWorkSendEvidence(
    value.root,
    value.options,
    processPort,
    "git-fixture",
    validator,
    validator,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-asic-work-send-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});

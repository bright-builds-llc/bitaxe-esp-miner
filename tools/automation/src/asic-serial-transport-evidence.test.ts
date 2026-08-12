import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicSerialTransportEvidenceError,
  projectAsicSerialTransportEvidence,
} from "./asic-serial-transport-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";

const attemptCommit = "a".repeat(40);
const workCommit = "b".repeat(40);
const resultCommit = "c".repeat(40);
const currentCommit = "d".repeat(40);
const referenceCommit = "e".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const uartSource = `pub const UART_INITIAL_BAUD: u32 = 115_200;
pub const UART_TX_PIN: i32 = 17;
pub const UART_RX_PIN: i32 = 18;
pub const WAIT_TX_DONE_TIMEOUT_MS: u32 = 1_000;
const UART_RX_BUFFER_BYTES: usize = UART_BUF_SIZE * 2;
const READ_CHUNK_MAX: usize = 64;
.data_bits(config::DataBits::DataBits8)
.parity_none()
.stop_bits(config::StopBits::STOP1)
.flow_control(config::FlowControl::None)
ensure!(written == frame.len(), "partial BM1366 UART frame write");
let deadline = started + std::time::Duration::from_millis(u64::from(timeout_ms));
let mut scratch = [0_u8; READ_CHUNK_MAX];
Err(error) if is_uart_timeout_error(&error) && buf.is_empty() => 0,
buf.extend_from_slice(&scratch[..read]);
self.clear_rx()?;
`;

const productionSource = `prefix
            Bm1366ProductionCommand::SendProductionWork(_) => {
                for action in actions { execute_adapter_action_on_state(action, &mut state)?; }
                Ok(None)
            }
            Bm1366ProductionCommand::ReadProductionResult => {
suffix
    let maybe_frame = match uart.maybe_try_read_exact(BM1366_RESULT_FRAME_LEN, poll_timeout_ms) {
        Ok(maybe_frame) => maybe_frame,
    };
    let Some(frame) = maybe_frame else {
tail
`;

function workEvidence(accepted = true) {
  return {
    schema_version: "bitaxe-asic-work-send-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: workCommit,
    reference_commit: referenceCommit,
    workflow: { schema_version: "bitaxe-workflow-identity-v1", command: "project-asic-work-send-evidence", request_sha256: "f".repeat(64) },
    source: { initialization_projection_sha256: "0".repeat(64), initialization_projection_current_commit: "1".repeat(40), initialization_projection_valid: true },
    work_send: { payload_length_bytes: 82, frame_length_bytes: 88, job_id_step: 8, job_id_modulus: 128, typed_write_frame_action: true, production_ready_gate_required: true, live_work_observed: true, qualified_result_observed: true, accepted_submit_observed: true, production_uart_retained: true, core_paths_unchanged: true, compatible_core_path_count: 3, dispatch_spans_unchanged: true, uart_write_span_unchanged: true },
    package_admitted: true, runtime_identity: "trusted", runtime_attestation_status: "trusted",
    campaign_terminal_category: "submit_response_observed", submit_outcome: accepted ? "accepted" : "rejected",
    safety_status: "fresh", mine_on_boot_disabled: true, safe_stop_confirmed: true,
    lease_cleanup_confirmed: true, usb_cleanup_ready: true, hardware_rerun_used: false, redaction_status: "passed",
  };
}

function resultEvidence(workDocument: string) {
  return {
    schema_version: "bitaxe-asic-result-parsing-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: resultCommit,
    reference_commit: referenceCommit,
    workflow: { schema_version: "bitaxe-workflow-identity-v1", command: "project-asic-result-parsing-evidence", request_sha256: "2".repeat(64) },
    source: { work_send_projection_sha256: createHash("sha256").update(workDocument).digest("hex"), work_send_projection_current_commit: workCommit, work_send_projection_valid: true },
    result_parsing: { result_frame_length_bytes: 11, strict_length_validation: true, preamble_validation: true, crc_validation: true, job_lookup_validation: true, submit_nonce_little_endian: true, core_validation: true, address_interval_validation: true, version_bits_recovered: true, known_register_classification: true, typed_soft_discard_category_count: 8, soft_discard_continuation: true, live_qualified_result_observed: true, accepted_submit_observed: true, result_transport_module_unchanged: true, parser_spans_unchanged: true, adapter_nonce_span_unchanged: true, worker_nonce_span_unchanged: true, correlation_semantics_compatible: true },
    package_admitted: true, runtime_identity: "trusted", runtime_attestation_status: "trusted",
    campaign_terminal_category: "submit_response_observed", submit_outcome: "accepted",
    safety_status: "fresh", mine_on_boot_disabled: true, safe_stop_confirmed: true,
    lease_cleanup_confirmed: true, usb_cleanup_ready: true, hardware_rerun_used: false, redaction_status: "passed",
  };
}

async function fixture(name: string, accepted = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-transport-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const workSendProjection = path.join(root, "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json");
  const resultParsingProjection = path.join(root, "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json");
  await mkdir(path.dirname(workSendProjection), { recursive: true });
  await mkdir(path.dirname(resultParsingProjection), { recursive: true });
  const workDocument = `${JSON.stringify(workEvidence(accepted), null, 2)}\n`;
  await writeFile(workSendProjection, workDocument);
  await writeFile(resultParsingProjection, `${JSON.stringify(resultEvidence(workDocument), null, 2)}\n`);
  const projection = path.join(root, "docs/parity/evidence/asic005-serial-transport/evidence.json");
  return { root, projection, options: { workSendProjection, resultParsingProjection, attemptSourceCommit: attemptCommit, projection } };
}

function fakePort(options: { readonly moduleDrift?: boolean; readonly spanDrift?: boolean; readonly dirty?: boolean; readonly resultValidatorFailure?: boolean; readonly launchFailure?: boolean } = {}) {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.resultValidatorFailure && spec.program === "result-validator") return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M uart.rs\n" : "");
    if (spec.args[0] === "diff" && options.moduleDrift) return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      if (target.endsWith("uart.rs")) return ok(uartSource);
      if (target.endsWith("production.rs")) return ok(options.spanDrift && target.startsWith(currentCommit) ? productionSource.replace("execute_adapter_action_on_state", "changed_action") : productionSource);
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<AsicSerialTransportEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicSerialTransportEvidenceError);
    return error;
  }
}

test("accepted live sources emit only closed serial-transport evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectAsicSerialTransportEvidence(value.root, value.options, fakePort(), "git", "work-validator", "result-validator", "validator");

  // Assert
  assert.equal(evidence.serial_transport.initial_baud, 115_200);
  assert.equal(evidence.serial_transport.live_result_rx_observed, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"), /password|credential|device_url|endpoint|frame_bytes|local_path|pool_url|wifi_ssid|serial_port/iu);
});

test("source-validator rejection withholds public evidence", async () => {
  // Arrange
  const value = await fixture("validator");

  // Act
  const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, fakePort({ resultValidatorFailure: true }), "git", "work-validator", "result-validator", "validator"));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("incomplete source withholds public evidence", async () => {
  // Arrange
  const value = await fixture("incomplete", false);

  // Act
  const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, fakePort(), "git", "work-validator", "result-validator", "validator"));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("malformed source withholds public evidence", async () => {
  // Arrange
  const value = await fixture("malformed");
  await writeFile(value.options.resultParsingProjection, "not-json\n");

  // Act
  const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, fakePort(), "git", "work-validator", "result-validator", "validator"));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("child launch failure is typed and withholds public evidence", async () => {
  // Arrange
  const value = await fixture("launch-failure");

  // Act
  const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, fakePort({ launchFailure: true }), "git", "work-validator", "result-validator", "validator"));

  // Assert
  assert.equal(error.category, "process_failed");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("module and production span drift withhold public evidence", async () => {
  for (const [name, port] of [["module", fakePort({ moduleDrift: true })], ["span", fakePort({ spanDrift: true })]] as const) {
    // Arrange
    const value = await fixture(name);

    // Act
    const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, port, "git", "work-validator", "result-validator", "validator"));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("dirty transport paths withhold public evidence", async () => {
  // Arrange
  const value = await fixture("dirty");

  // Act
  const error = await captureError(projectAsicSerialTransportEvidence(value.root, value.options, fakePort({ dirty: true }), "git", "work-validator", "result-validator", "validator"));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("real child validators must accept both sources and candidate", async () => {
  // Arrange
  const value = await fixture("real-child");
  const validator = path.join(value.root, "validator-child.sh");
  await writeFile(validator, "#!/bin/sh\ntest -s \"$1\"\n");
  await chmod(validator, 0o700);
  const localPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const gitPort = fakePort();
  const processPort: ProcessPort = {
    loadEspEnvironment: () => localPort.loadEspEnvironment(),
    run: (spec, maybeTimeoutMs) => spec.program === "git-fixture" ? gitPort.run(spec, maybeTimeoutMs) : localPort.run(spec, maybeTimeoutMs),
  };

  // Act
  const evidence = await projectAsicSerialTransportEvidence(value.root, value.options, processPort, "git-fixture", validator, validator, validator);

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-asic-serial-transport-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});

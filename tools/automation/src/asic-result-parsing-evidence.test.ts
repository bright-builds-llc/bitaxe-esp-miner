import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicResultParsingEvidenceError,
  projectAsicResultParsingEvidence,
} from "./asic-result-parsing-evidence.js";
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

const parserSpans = `pub fn parse_bm1366_result_frame(
    bytes: &[u8],
) {
    let frame = ResultFrameBytes::try_from_slice(bytes)?;
}

/// Classifies strict parser failures as a soft discard

fn validate_result_frame(frame: ResultFrameBytes) {
    if actual_preamble != BM1366_RECEIVE_PREAMBLE {
    }
    if crc5(&bytes[2..]) != 0 {
    }
}

fn parse_job_result(frame: ResultFrameBytes) {
    let nonce_bytes = [bytes[2], bytes[3], bytes[4], bytes[5]];
    let submit_nonce = u32::from_le_bytes(nonce_bytes);
    if !valid_jobs.contains(job_id) {
    }
    if core_id >= BM1366_NORMAL_CORE_COUNT {
    }
    let address_interval = valid_address_interval(address_interval)?;
    let asic_index = (u16::from(((nonce_be >> 17) & 0xff) as u8) / address_interval) as u8;
    let version_bits = (u32::from(version_be)) << 13;
}

fn parse_register_read(frame: ResultFrameBytes) {
    let register = Bm1366Register::try_from(bytes[7])?;
}
`;

const currentResultSource = `pub const BM1366_RESULT_FRAME_LEN: usize = 11;
pub enum Bm1366ResultDiscardReason {
    InvalidLength,
    InvalidPreamble,
    InvalidCrc,
    JobLookup,
    Core,
    AddressInterval,
    RegisterResponse,
    ParserInvariant,
}
impl Bm1366ResultDiscardReason {
}
${parserSpans}
fn classify() {
    Err(fault) => Bm1366ProductionResult::Discarded(discard_reason(fault)),
}
`;

const adapterSource = `prefix
        Bm1366ProductionResult::JobNonce(result) => Ok(ProductionReadOutcome::JobNonce(result)),
        Bm1366ProductionResult::RegisterRead(read) => {
suffix
`;

const workerSource = `prefix
                            Ok(ProductionReadOutcome::JobNonce(result)) => {
                                emit(AsicWorkerEvent::Result { generation, result });
                            }
                            Ok(ProductionReadOutcome::Pending) => {
suffix
`;

const correlationSource = `prefix
    pub fn correlate_nonce_result(
        &mut self,
        observation: ProductionNonceObservation,
    ) -> CorrelationOutcome {
        let maybe_record = self.active_work.get_mut(&observation.result.job_id.lookup_key());
        if record.generation != self.generation {
        }
        if !stored_work_context_matches_nonce_result(record, observation.result) {
        }
        let submission = ShareSubmission::from_nonce_result(&record.work, observation.result);
        CorrelationOutcome::SubmitIntent(SubmitIntent {
            submission,
        })
    }
    pub const fn valid_jobs(
suffix
`;

function sourceEvidence(accepted = true) {
  return {
    schema_version: "bitaxe-asic-work-send-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: sourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-work-send-evidence",
      request_sha256: "e".repeat(64),
    },
    source: {
      initialization_projection_sha256: "f".repeat(64),
      initialization_projection_current_commit: "0".repeat(40),
      initialization_projection_valid: true,
    },
    work_send: {
      payload_length_bytes: 82,
      frame_length_bytes: 88,
      job_id_step: 8,
      job_id_modulus: 128,
      typed_write_frame_action: true,
      production_ready_gate_required: true,
      live_work_observed: true,
      qualified_result_observed: true,
      accepted_submit_observed: true,
      production_uart_retained: true,
      core_paths_unchanged: true,
      compatible_core_path_count: 3,
      dispatch_spans_unchanged: true,
      uart_write_span_unchanged: true,
    },
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
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
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-result-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(
    root,
    "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
  );
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  await writeFile(sourceProjection, `${JSON.stringify(sourceEvidence(accepted), null, 2)}\n`);
  const projection = path.join(root, "docs/parity/evidence/asic004-result-parsing/evidence.json");
  return {
    root,
    projection,
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: {
  readonly spanDrift?: boolean;
  readonly dirty?: boolean;
  readonly sourceValidatorFailure?: boolean;
} = {}) {
  return createFakeProcessPort(async (spec) => {
    if (options.sourceValidatorFailure && spec.program === "source-validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M result.rs\n" : "");
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      if (target.endsWith("result.rs")) {
        const source = target.startsWith(attemptCommit) ? parserSpans : currentResultSource;
        return ok(options.spanDrift && target.startsWith(currentCommit)
          ? source.replace("let submit_nonce", "let changed_submit_nonce")
          : source);
      }
      if (target.endsWith("production.rs")) return ok(adapterSource);
      if (target.endsWith("asic_worker.rs")) return ok(workerSource);
      if (target.endsWith("production_work.rs")) return ok(correlationSource);
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<AsicResultParsingEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicResultParsingEvidenceError);
    return error;
  }
}

test("accepted live source emits only closed result-parsing evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectAsicResultParsingEvidence(
    value.root, value.options, fakePort(), "git", "source-validator", "validator",
  );

  // Assert
  assert.equal(evidence.result_parsing.result_frame_length_bytes, 11);
  assert.equal(evidence.result_parsing.typed_soft_discard_category_count, 8);
  assert.equal(
    evidence.source.work_send_projection_sha256,
    createHash("sha256").update(await readFile(value.options.sourceProjection)).digest("hex"),
  );
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(
    await readFile(value.projection, "utf8"),
    /password|credential|device_url|endpoint|frame_bytes|local_path|pool_url|wifi_ssid/iu,
  );
});

test("source-validator rejection withholds public evidence", async () => {
  // Arrange
  const value = await fixture("source-validator");

  // Act
  const error = await captureError(projectAsicResultParsingEvidence(
    value.root,
    value.options,
    fakePort({ sourceValidatorFailure: true }),
    "git",
    "source-validator",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("incomplete source withholds public evidence", async () => {
  // Arrange
  const value = await fixture("incomplete", false);

  // Act
  const error = await captureError(projectAsicResultParsingEvidence(
    value.root, value.options, fakePort(), "git", "source-validator", "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("parser span drift withholds public evidence", async () => {
  // Arrange
  const value = await fixture("span-drift");

  // Act
  const error = await captureError(projectAsicResultParsingEvidence(
    value.root, value.options, fakePort({ spanDrift: true }), "git", "source-validator", "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("dirty result-parsing paths withhold public evidence", async () => {
  // Arrange
  const value = await fixture("dirty");

  // Act
  const error = await captureError(projectAsicResultParsingEvidence(
    value.root, value.options, fakePort({ dirty: true }), "git", "source-validator", "validator",
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
  const evidence = await projectAsicResultParsingEvidence(
    value.root, value.options, processPort, "git-fixture", validator, validator,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-asic-result-parsing-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});

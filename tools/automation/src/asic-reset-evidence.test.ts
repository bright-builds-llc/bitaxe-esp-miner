import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicResetEvidenceError,
  projectAsicResetEvidence,
} from "./asic-reset-evidence.js";
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

const sources = new Map<string, string>([
  ["command.rs", `
pub const RESET_PULSE: Self = Self::ResetPulse {
    low_ms: 100,
    high_ms: 100,
};
pub const HOLD_RESET_LOW: Self = Self::HoldResetLow;
`],
  ["init_plan.rs", `
actions.insert(1, Bm1366AdapterAction::reset_pulse());
pub(crate) fn fail_closed(reason: &'static str, action: FailClosedAction) -> Self {
        let status = AsicInitStatus::FailClosed { reason };
        Self {
            stages: vec![Bm1366InitStage::Preflight],
            actions: vec![
                Bm1366AdapterAction::HoldResetLow,
`],
  ["asic_adapter.rs", `
Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => {
            reset.reset_pulse(*low_ms, *high_ms)?;
Bm1366AdapterAction::HoldResetLow => {
            reset.hold_reset_low()?;
            Ok(ActionOutcome::Continue)
`],
  ["reset.rs", `
pub const RESET_PULSE_LOW_MS: u32 = 100;
pub const RESET_PULSE_HIGH_MS: u32 = 100;
pub fn reset_pulse(&mut self, low_ms: u32, high_ms: u32) {
    self.reset.set_low()?;
        std::thread::sleep(std::time::Duration::from_millis(u64::from(low_ms)));
        self.reset.set_high()?;
        std::thread::sleep(std::time::Duration::from_millis(u64::from(high_ms)));
}
pub fn hold_reset_low(&mut self) -> Result<()> {
`],
  ["mining_actuation.rs", `
pub const fn preparation_plan(profile: MiningHardwareProfile) -> [PreparationStep; 9]
PreparationStep::ResetAndDetectExactlyOneChip,
pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8]
SafeShutdownStep::HoldResetLow,
`],
  ["mining_actuation_adapter.rs", `
skip_reset_pulse: false,
PreparationStep::ResetAndDetectExactlyOneChip =>
SafeShutdownStep::HoldResetLow =>
`],
]);

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
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-reset-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(root,
    "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json");
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  const sourceDocument = `${JSON.stringify(sourceEvidence(complete), null, 2)}\n`;
  await writeFile(sourceProjection, sourceDocument);
  const plan = path.join(root, "docs/parity/work-plans/20260812T185214Z-PWR-001/PLAN.md");
  await mkdir(path.dirname(plan), { recursive: true });
  const planDocument = `# Plan

- Parity row: \`PWR-001\`
- Active task: \`task-parity-pwr001-asic-reset-evidence-audit\`
`;
  await writeFile(plan, planDocument);
  await writeFile(path.join(root, "TASKS.md"), `
### task-parity-pwr001-asic-reset-evidence-audit | 2026-08-12 | Audit

Plan: \`docs/parity/work-plans/20260812T185214Z-PWR-001/PLAN.md\`

- Prove active-low 100 ms/100 ms reset semantics.

Hardware contract: No hardware interaction is permitted or required.

### next-task | later | Other
`);
  const projection = path.join(root,
    "docs/parity/evidence/pwr001-asic-reset/asic-reset-projection.json");
  return {
    root,
    projection,
    sourceSha256: createHash("sha256").update(sourceDocument).digest("hex"),
    planSha256: createHash("sha256").update(planDocument).digest("hex"),
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
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
    if (spec.args[0] === "status") return ok(options.dirty ? " M reset.rs\n" : "");
    if (spec.args[0] === "diff" && options.sourceDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show") {
      const target = spec.args[1] ?? "";
      for (const [suffix, source] of sources) {
        if (target.endsWith(suffix)) {
          return ok(options.semanticDrift && suffix === "reset.rs"
            ? source.replace("RESET_PULSE_LOW_MS: u32 = 100", "RESET_PULSE_LOW_MS: u32 = 99")
            : source);
        }
      }
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<AsicResetEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicResetEvidenceError);
    return error;
  }
}

async function projectFixture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
) {
  return projectAsicResetEvidence(
    value.root, value.options, processPort, "git", "source-validator", "validator",
    value.sourceSha256, value.planSha256,
  );
}

test("complete reset transaction emits only closed row evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.reset.low_duration_ms, 100);
  assert.equal(evidence.reset.exactly_one_chip_detected_after_reset, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /hostname|origin|usbmodem|ssid|password|private\/|scratch\//iu);
});

for (const [name, complete, options, category] of [
  ["incomplete-source", false, {}, "evidence_invalid"],
  ["source-drift", true, { sourceDrift: true }, "evidence_invalid"],
  ["semantic-drift", true, { semanticDrift: true }, "evidence_invalid"],
  ["dirty-source", true, { dirty: true }, "evidence_invalid"],
  ["validator-rejected", true, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", true, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final reset evidence`, async () => {
    // Arrange
    const value = await fixture(name, complete);

    // Act
    const error = await captureError(projectFixture(value, fakePort(options)));

    // Assert
    assert.equal(error.category, category);
    assert.deepEqual(error.publicValue, {
      stage: "sealed_asic_reset_projection",
      hardware_rerun_used: false,
      projection_published: false,
    });
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    await assert.rejects(readFile(`${value.projection}.candidate`, "utf8"), { code: "ENOENT" });
  });
}

test("mutated active task binding is rejected before publication", async () => {
  // Arrange
  const value = await fixture("task-drift");
  await writeFile(path.join(value.root, "TASKS.md"), "### unrelated | task\n");

  // Act
  const error = await captureError(projectFixture(value, fakePort()));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("real child validators must accept source and candidate files", async () => {
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
  const evidence = await projectAsicResetEvidence(
    value.root, value.options, processPort, "git-fixture", validator, validator,
    value.sourceSha256, value.planSha256,
  );

  // Assert
  assert.equal(evidence.source.initialization_projection_valid, true);
  assert.equal(evidence.reset.adapter_semantics_admitted, true);
});

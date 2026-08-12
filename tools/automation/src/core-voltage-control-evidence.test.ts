import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  CoreVoltageControlEvidenceError,
  projectCoreVoltageControlEvidence,
} from "./core-voltage-control-evidence.js";
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

async function repositorySource(relative: string): Promise<string> {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const candidates = [
    ...(maybeRunfiles === undefined ? [] : [path.join(maybeRunfiles, "_main", relative)]),
    path.join(process.cwd(), relative),
    path.resolve(process.cwd(), "..", "..", relative),
  ];
  for (const candidate of candidates) {
    try {
      return await readFile(candidate, "utf8");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  throw new Error("production source file is missing");
}

const sources = new Map<string, string>([
  ["ds4432u.rs", `
Self::Conservative1100Millivolts => 1_100,
Self::Output0 => 0xf8,
let code = core_voltage_code(voltage.millivolts());
bus.write_ds4432u(Ds4432uWriteRegister::Output0, code)
assert_eq!(writer.writes, [(Ds4432uWriteRegister::Output0, 0xe1)]);
`],
  ["mining_actuation.rs", `
pub const CORE_VOLTAGE_STABILIZATION_MS: u16 = 500;
PreparationStep::SetCoreVoltage(profile.core_voltage()),
PreparationStep::WaitForCoreVoltageStabilization500Ms,
PreparationStep::EnableAsic,
SafeShutdownStep::DisableCoreVoltage,
SafeShutdownStep::DisableAsic,
`],
  ["mining_actuation_adapter.rs", `
SafetyActuationCommand::SetCoreVoltage(Self::core_voltage(voltage)?)
thread::sleep(Duration::from_millis(u64::from(
                    CORE_VOLTAGE_STABILIZATION_MS,
                )));
crate::asic_adapter::production::set_asic_power_enabled(true)
SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic =>
crate::asic_adapter::production::set_asic_power_enabled(false)
Pinned upstream VCORE_set_voltage(0) performs no DS4432U write;
`],
  ["safety_adapter.rs", `
SafetyActuationCommand::SetCoreVoltage(voltage) => {
            ds4432u::write_core_voltage(&mut bus, voltage)
        }
Ok(()) => SafetyActuationReply::Applied,
Err(_) => SafetyActuationReply::HardwareWriteFailed,
`],
  ["i2c_bus.rs", `
const DS4432U_I2C_ADDRESS: u8 = 0x48;
self.write_register(DS4432U_I2C_ADDRESS, register.address(), value)
`],
  ["DS4432U.c", `
#define DS4432U_SENSOR_ADDR 0x48
#define DS4432U_OUT0_REG 0xF8
DS4432U_set_current_code(0, reg)
`],
  ["vcore.c", `
gpio_set_level(GPIO_ASIC_ENABLE, core_voltage == 0.0f ? 1 : 0);
if (core_voltage != 0.0f) {
DS4432U_set_voltage(core_voltage)
`],
]);

function sourceEvidence(complete = true) {
  return {
    schema_version: "bitaxe-asic-power-initialization-evidence-v1",
    board: 205,
    attempt_source_commit: attemptCommit,
    current_source_commit: sourceCommit,
    reference_commit: referenceCommit,
    workflow: { schema_version: "bitaxe-workflow-identity-v1", command: "project-asic-power-initialization-evidence", request_sha256: "e".repeat(64) },
    source: { initialization_projection_sha256: "f".repeat(64), initialization_projection_current_commit: sourceCommit, initialization_projection_valid: true, source_task_sha256: "0".repeat(64), plan_sha256: "1".repeat(64) },
    power_initialization: {
      profile: "conservative",
      frequency_mhz: 400,
      core_voltage_command_mv: 1_100,
      fan_duty_command_percent: 100,
      preparation_step_count: 9,
      accepted_preparation_event_count: 18,
      fresh_safety_required_before_effects: true,
      fan_full_commanded_before_voltage: true,
      post_command_nonzero_fan_rpm_required: true,
      core_voltage_stabilization_ms: 500,
      asic_enable_active_low: true,
      reset_and_detect_completed: true,
      exactly_one_chip_detected_after_reset: true,
      mining_ready_initialization_completed: complete,
      production_uart_retained: true,
      accepted_submit_observed: true,
      rollback_step_count: 8,
      rollback_attempts_all_steps: true,
      initial_preparation_failure_primary: true,
      safe_stop_asic_disable_commanded: true,
      unchanged_path_count: 6,
      semantic_path_count: 3,
      source_semantics_admitted: true,
    },
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

async function fixture(name: string, complete = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-core-voltage-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const sourceProjection = path.join(root,
    "docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json");
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  const sourceDocument = `${JSON.stringify(sourceEvidence(complete), null, 2)}\n`;
  await writeFile(sourceProjection, sourceDocument);
  const plan = path.join(root, "docs/parity/work-plans/20260812T212218Z-PWR-003/PLAN.md");
  await mkdir(path.dirname(plan), { recursive: true });
  const planDocument = `# Plan

- Parity row: \`PWR-003\`
- Active task: \`task-parity-pwr003-core-voltage-control-evidence-retry\`
`;
  await writeFile(plan, planDocument);
  await writeFile(path.join(root, "TASKS.md"), `
### task-parity-pwr003-core-voltage-control-evidence-retry | 2026-08-12 | Audit

Plan: \`docs/parity/work-plans/20260812T212218Z-PWR-003/PLAN.md\`

- Prove the DS4432U address/register/code and write route.

This is a software-only evidence audit.

### next-task | later | Other
`);
  const projection = path.join(root,
    "docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json");
  return {
    root,
    projection,
    sourceSha256: createHash("sha256").update(sourceDocument).digest("hex"),
    planSha256: createHash("sha256").update(planDocument).digest("hex"),
    options: { sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function sourceForTarget(target: string): string | undefined {
  for (const [suffix, source] of sources) {
    if (target.endsWith(suffix)) return source;
  }
  return undefined;
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
  readonly semanticDrift?: boolean;
  readonly dirty?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
  readonly productionAdapterSource?: string;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === "validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C" && spec.args[2] === "rev-parse") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M ds4432u.rs\n" : "");
    if (spec.args[0] === "diff" && options.sourceDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    const target = spec.args[0] === "-C" ? spec.args[3] ?? "" : spec.args[1] ?? "";
    if (spec.args[0] === "show" || (spec.args[0] === "-C" && spec.args[2] === "show")) {
      if (options.productionAdapterSource !== undefined
        && target.endsWith("mining_actuation_adapter.rs")) {
        return ok(options.productionAdapterSource);
      }
      const source = sourceForTarget(target);
      if (source !== undefined) {
        return ok(options.semanticDrift && target.endsWith("ds4432u.rs")
          ? source.replace("0xe1", "0xe0")
          : source);
      }
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<CoreVoltageControlEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof CoreVoltageControlEvidenceError);
    return error;
  }
}

async function projectFixture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
) {
  return projectCoreVoltageControlEvidence(
    value.root, value.options, processPort, "git", "source-validator", "validator",
    value.sourceSha256, value.planSha256,
  );
}

test("accepted voltage transaction emits only closed row evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.voltage_control.target_millivolts, 1_100);
  assert.equal(evidence.voltage_control.register_code, 0xe1);
  assert.equal(evidence.voltage_control.active_low_disable, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /hostname|origin|usbmodem|ssid|password|private\/|scratch\//iu);
});

test("production adapter admits the source-shaped stabilization use", async () => {
  // Arrange
  const value = await fixture("production-adapter");
  const productionAdapterSource = await repositorySource(
    "firmware/bitaxe/src/mining_actuation_adapter.rs",
  );
  const ambiguousTokenCount = productionAdapterSource
    .split("CORE_VOLTAGE_STABILIZATION_MS,").length - 1;

  // Act
  const evidence = await projectFixture(value, fakePort({ productionAdapterSource }));

  // Assert
  assert.equal(ambiguousTokenCount, 2);
  assert.equal(evidence.voltage_control.stabilization_millis, 500);
  assert.equal(evidence.voltage_control.stabilization_before_asic_enable, true);
});

for (const [name, complete, options, category] of [
  ["incomplete-source", false, {}, "evidence_invalid"],
  ["source-drift", true, { sourceDrift: true }, "evidence_invalid"],
  ["semantic-drift", true, { semanticDrift: true }, "evidence_invalid"],
  ["dirty-source", true, { dirty: true }, "evidence_invalid"],
  ["validator-rejected", true, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", true, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final core-voltage evidence`, async () => {
    // Arrange
    const value = await fixture(name, complete);

    // Act
    const error = await captureError(projectFixture(value, fakePort(options)));

    // Assert
    assert.equal(error.category, category);
    assert.deepEqual(error.publicValue, {
      stage: "sealed_core_voltage_control_projection",
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
  const validator = "/usr/bin/stat";
  const localPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const gitPort = fakePort();
  const processPort: ProcessPort = {
    loadEspEnvironment: () => localPort.loadEspEnvironment(),
    run: (spec, maybeTimeoutMs) => spec.program === "git-fixture"
      ? gitPort.run(spec, maybeTimeoutMs)
      : localPort.run(spec, maybeTimeoutMs),
  };

  // Act
  const evidence = await projectCoreVoltageControlEvidence(
    value.root, value.options, processPort, "git-fixture", validator, validator,
    value.sourceSha256, value.planSha256,
  );

  // Assert
  assert.equal(evidence.source.power_initialization_projection_valid, true);
  assert.equal(evidence.voltage_control.reference_semantics_admitted, true);
});

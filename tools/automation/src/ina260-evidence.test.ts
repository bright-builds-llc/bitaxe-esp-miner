import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  Ina260EvidenceError,
  projectIna260Evidence,
  type Ina260AdmittedDigests,
} from "./ina260-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";

const attemptCommit = "a".repeat(40);
const currentCommit = "b".repeat(40);
const referenceCommit = "c".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

const sourceDocuments = new Map<string, string>([
  ["crates/bitaxe-safety/src/power.rs", [
    "pub bus_voltage_volts: f64,",
    "pub current_amps: f64,",
    "pub power_watts: f64,",
  ].join("\n")],
  ["crates/bitaxe-safety/src/sensor_acquisition.rs", [
    "let current_ma = f64::from(i16::from_be_bytes(current)) * INA260_CURRENT_MILLIAMPS_PER_BIT;",
    "let bus_voltage_mv = f64::from(u16::from_be_bytes(bus_voltage)) * INA260_BUS_MILLIVOLTS_PER_BIT;",
    "let power_mw = f64::from(u16::from_be_bytes(power)) * INA260_POWER_MILLIWATTS_PER_BIT;",
    "bus_voltage_volts: bus_voltage_mv / 1000.0,",
    "current_amps: current_ma / 1000.0,",
    "power_watts: power_mw / 1000.0,",
  ].join("\n")],
  ["firmware/bitaxe/src/safety_adapter/ina260.rs", [
    "Reads one complete INA260 triple through the closed read-only capability.",
    "bus.read_ina260(Ina260ReadRegister::Current, &mut current)",
    "bus.read_ina260(Ina260ReadRegister::BusVoltage, &mut bus_voltage)",
    "bus.read_ina260(Ina260ReadRegister::Power, &mut power)",
    "AcquisitionOutcome::Success(decode_ina260(current, bus_voltage, power))",
  ].join("\n")],
  ["firmware/bitaxe/src/safety_adapter/i2c_bus.rs", [
    "const INA260_I2C_ADDRESS: u8 = 0x40;",
    "Self::Current => 0x01,",
    "Self::BusVoltage => 0x02,",
    "Self::Power => 0x03,",
    "pub(crate) struct ReadOnlySensorBus",
  ].join("\n")],
  ["firmware/bitaxe/src/operator_sensor_runtime.rs", [
    "power_watts: project_observation(",
    "bus_voltage_volts: project_observation(",
    "current_amps: project_observation(",
  ].join("\n")],
  ["crates/bitaxe-api/src/snapshot.rs", [
    "power_status: (&observations.power_watts).into(),",
    "voltage_status: (&observations.bus_voltage_volts).into(),",
    "current_status: (&observations.current_amps).into(),",
  ].join("\n")],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub power_watts: Observation<f64>,",
    "pub bus_voltage_volts: Observation<f64>,",
    "pub current_amps: Observation<f64>,",
  ].join("\n")],
  ["crates/bitaxe-api/src/legacy_units.rs", [
    "const MILLI_UNITS_PER_UNIT: f64 = 1_000.0;",
    "pub(crate) const fn millivolts_from_volts(volts: f64) -> f64",
    "pub(crate) const fn milliamps_from_amps(amps: f64) -> f64",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    "power: safe_telemetry.power_watts,",
    "voltage_millivolts: millivolts_from_volts(safe_telemetry.voltage_volts),",
    "current_milliamps: milliamps_from_amps(safe_telemetry.current_amps),",
  ].join("\n")],
  ["crates/bitaxe-api/src/statistics.rs", [
    "voltage_millivolts: millivolts_from_volts(safe_telemetry.voltage_volts),",
    "current_milliamps: milliamps_from_amps(safe_telemetry.current_amps),",
  ].join("\n")],
  ["tools/flash/src/campaign/network/validation.rs", [
    "(4_500.0..=5_500.0).contains(&sample.voltage_millivolts)",
    "sample.current_milliamps >= 0.0",
  ].join("\n")],
]);

const historicalDocuments = new Map<string, string>([
  ["crates/bitaxe-api/src/wire.rs", [
    "voltage: safe_telemetry.voltage_volts,",
    "current: safe_telemetry.current_amps,",
  ].join("\n")],
  ["crates/bitaxe-api/src/statistics.rs", [
    "voltage: safe_telemetry.voltage_volts,",
    "current: safe_telemetry.current_amps,",
  ].join("\n")],
]);

const referenceDocuments = new Map<string, string>([
  ["reference/esp-miner/main/power/INA260.h", [
    "INA260_REG_CURRENT 0x01     ///< Current measurement register (signed) in mA",
    "INA260_REG_BUSVOLTAGE 0x02  ///< Bus voltage measurement register in mV",
    "INA260_REG_POWER 0x03       ///< Power calculation register in mW",
  ].join("\n")],
  ["reference/esp-miner/main/power/INA260.c", [
    "last_current = (uint16_t)(data[1] | (data[0] << 8)) * 1.25;",
    "last_voltage = (uint16_t)(data[1] | (data[0] << 8)) * 1.25;",
    "last_power = (data[1] | (data[0] << 8)) * 10;",
  ].join("\n")],
  ["reference/esp-miner/main/power/power.c", [
    "pow_val = INA260_read_power() / 1000.0f;",
    "return INA260_read_voltage();",
  ].join("\n")],
  ["reference/esp-miner/main/http_server/system_api_json.c", [
    "cJSON_AddFloatToObject(root, \"voltage\", g->POWER_MANAGEMENT_MODULE.voltage);",
    "cJSON_AddFloatToObject(root, \"current\", g->POWER_MANAGEMENT_MODULE.current);",
    "cJSON_AddFloatToObject(root, \"coreVoltageActual\", g->POWER_MANAGEMENT_MODULE.core_voltage);",
  ].join("\n")],
  ["reference/esp-miner/main/tasks/statistics_task.c", [
    "statsData.voltage = power_management->voltage;",
    "statsData.current = power_management->current;",
    "statsData.coreVoltageActual = power_management->core_voltage;",
  ].join("\n")],
  ["reference/esp-miner/main/http_server/axe-os/src/app/components/home/home.component.ts", [
    "processed.voltage = processed.voltage / 1000;",
    "processed.current = processed.current / 1000;",
    "processed.coreVoltageActual = processed.coreVoltageActual / 1000;",
    "element[idxChartY1Data] = element[idxChartY1Data] / 1000;",
  ].join("\n")],
]);

const compatiblePaths = [
  "crates/bitaxe-safety/src/power.rs",
  "crates/bitaxe-safety/src/sensor_acquisition.rs",
  "firmware/bitaxe/src/safety_adapter/ina260.rs",
  "firmware/bitaxe/src/safety_adapter/i2c_bus.rs",
  "firmware/bitaxe/src/operator_sensor_runtime.rs",
  "crates/bitaxe-api/src/observation.rs",
  "crates/bitaxe-api/src/snapshot.rs",
  "crates/bitaxe-api/src/legacy_units.rs",
  "crates/bitaxe-api/src/wire.rs",
  "crates/bitaxe-api/src/statistics.rs",
  "tools/flash/src/campaign/network/validation.rs",
] as const;

function digest(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function sourceEvidence() {
  return {
    schema_version: "bitaxe-system-info-evidence-v1",
    board: 205,
    source_commit: attemptCommit,
    reference_commit: referenceCommit,
    package_manifest_sha256: "d".repeat(64),
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-system-info-evidence",
      request_sha256: "e".repeat(64),
    },
    detector_admitted: true,
    boot_observed: true,
    same_origin_observed: true,
    system_info: {
      boot_session_sha256: "f".repeat(64),
      http_revision: 7,
      websocket_revision: 8,
      same_boot_session: true,
      websocket_revision_not_earlier: true,
      field_contract_schema: "bitaxe-system-info-field-contract-v1",
      field_contract_sha256: "1".repeat(64),
      required_field_count: 94,
      unconditional_field_count: 87,
      conditional_field_count: 7,
      http_unconditional_fields_complete: true,
      websocket_unconditional_fields_complete: true,
      http_field_types_match: true,
      websocket_field_types_match: true,
      inactive_block_fields_absent: true,
      confirmed_setting_fields_present: true,
      retained_http_tuple_matches: true,
      retained_websocket_tuple_matches: true,
    },
    mining_state: "disabled",
    hardware_control_state: "disabled",
    cleanup_complete: true,
    redaction_status: "passed",
  };
}

function snapshot(revision: number, power = 10) {
  const stamp = { bootSession: 9_007_199_254_740_992, sequence: 2, acquiredAtMs: 3 };
  return {
    sourceCommit: attemptCommit,
    referenceCommit,
    appElfSha256: "2".repeat(64),
    bootSession: "3".repeat(32),
    operatorSnapshotRevision: revision,
    power,
    voltage: 5,
    current: 2,
    powerStatus: { state: "fresh", stamp },
    voltageStatus: { state: "fresh", stamp },
    currentStatus: { state: "fresh", stamp },
  };
}

async function privateFile(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

async function fixture(name: string, mutate?: (api: Record<string, unknown>, websocket: Record<string, unknown>) => void) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-ina260-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  for (const sourcePath of compatiblePaths) {
    const candidate = path.join(root, sourcePath);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${sourceDocuments.get(sourcePath) ?? "compatible"}\n`);
  }
  for (const [sourcePath, document] of referenceDocuments) {
    const candidate = path.join(root, sourcePath);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${document}\n`);
  }
  const hardwarePlan = path.join(root, "docs/parity/work-plans/20260812T222308Z-PWR-006/PLAN.md");
  await mkdir(path.dirname(hardwarePlan), { recursive: true });
  const hardwarePlanDocument = [
    "# Plan",
    "- Parity row: `PWR-006`",
    "- Active task: `task-parity-pwr006-ina260-live-projection`",
  ].join("\n") + "\n";
  await writeFile(hardwarePlan, hardwarePlanDocument);
  const correctionPlan = path.join(root, "docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md");
  await mkdir(path.dirname(correctionPlan), { recursive: true });
  const correctionPlanDocument = [
    "# Plan",
    "- Parity row: `PWR-006`",
    "- Active task: `task-parity-pwr006-legacy-wire-units`",
  ].join("\n") + "\n";
  await writeFile(correctionPlan, correctionPlanDocument);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-pwr006-legacy-wire-units | 2026-08-16 | Project",
    "Plan docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md",
    "Correct millivolts and milliamps through read-only reuse of prior evidence.",
    "### next-task | later | Other",
  ].join("\n"));

  const sourceProjection = path.join(root,
    "docs/parity/evidence/api002-system-info/system-info-projection.json");
  await mkdir(path.dirname(sourceProjection), { recursive: true });
  const sourceDocument = `${JSON.stringify(sourceEvidence(), null, 2)}\n`;
  await writeFile(sourceProjection, sourceDocument);
  const attemptRoot = path.join(root, "scratch/api002-system-info/attempt-002");
  await mkdir(attemptRoot, { recursive: true, mode: 0o700 });
  await chmod(attemptRoot, 0o700);
  const api = snapshot(7) as Record<string, unknown>;
  const websocket = snapshot(8) as Record<string, unknown>;
  for (const field of ["power", "voltage", "current"] as const) {
    const status = websocket[`${field}Status`] as Record<string, unknown>;
    const stamp = status["stamp"] as Record<string, unknown>;
    status["stamp"] = {
      acquiredAtMs: stamp["acquiredAtMs"],
      sequence: stamp["sequence"],
      bootSession: stamp["bootSession"],
    };
  }
  mutate?.(api, websocket);
  const apiDocument = `${JSON.stringify(api, null, 2)}\n`;
  const websocketDocument = `${JSON.stringify({ event: "update", data: websocket }, null, 2)}\n`;
  await privateFile(path.join(attemptRoot, "api.private.json"), apiDocument);
  await privateFile(path.join(attemptRoot, "websocket.private.json"), websocketDocument);
  await privateFile(path.join(attemptRoot, "final-evidence.private.json"), sourceDocument);
  for (const entry of [
    "flash-command-evidence.private.json",
    "flash-monitor.classifier-input.log",
    "retained-log.private.txt",
  ]) await privateFile(path.join(attemptRoot, entry), "protected\n");
  const projection = path.join(root, "docs/parity/evidence/pwr006-ina260/ina260-projection.json");
  const admitted: Ina260AdmittedDigests = {
    sourceProjection: digest(sourceDocument),
    apiSnapshot: digest(apiDocument),
    websocketSnapshot: digest(websocketDocument),
    finalEvidence: digest(sourceDocument),
    hardwarePlan: digest(hardwarePlanDocument),
    correctionPlan: digest(correctionPlanDocument),
  };
  return {
    root,
    projection,
    admitted,
    options: { attemptRoot, sourceProjection, attemptSourceCommit: attemptCommit, projection },
  };
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
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
    if (spec.args[0] === "-C" && spec.args[2] === "rev-parse") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "-C" && spec.args[2] === "status") return ok();
    if (spec.args[0] === "show") {
      if (options.sourceDrift) return ok("historical drift\n");
      const sourcePath = spec.args[1]?.split(":", 2)[1];
      return ok(`${historicalDocuments.get(sourcePath ?? "") ?? ""}\n`);
    }
    if (spec.args[0] === "status") return ok(options.dirty ? " M ina260.rs\n" : "");
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<Ina260EvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof Ina260EvidenceError);
    return error;
  }
}

async function projectFixture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
) {
  return projectIna260Evidence(
    value.root, value.options, processPort, "git", "source-validator", "validator", value.admitted,
  );
}

test("complete correlated INA260 snapshots emit only closed evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.telemetry.historical_http_complete_fresh_sample, true);
  assert.equal(evidence.telemetry.same_acquisition_stamps, true);
  assert.equal(evidence.telemetry.legacy_voltage_unit, "millivolts");
  assert.equal(evidence.telemetry.legacy_current_unit, "milliamps");
  assert.equal(evidence.telemetry.volts_to_millivolts_factor, 1_000);
  assert.equal(evidence.telemetry.campaign_min_input_millivolts, 4_500);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /hostname|origin|usbmodem|ssid|password|bootSession|acquiredAtMs|scratch\//iu);
});

for (const [name, mutate, options, category] of [
  ["stale-http", (api: Record<string, unknown>) => {
    (api["powerStatus"] as Record<string, unknown>)["state"] = "stale";
  }, {}, "evidence_invalid"],
  ["uncorrelated", (_api: Record<string, unknown>, websocket: Record<string, unknown>) => {
    websocket["power"] = 11;
  }, {}, "evidence_invalid"],
  ["source-drift", undefined, { sourceDrift: true }, "evidence_invalid"],
  ["dirty-source", undefined, { dirty: true }, "evidence_invalid"],
  ["validator-rejected", undefined, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", undefined, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final INA260 evidence`, async () => {
    // Arrange
    const value = await fixture(name, mutate);

    // Act
    const error = await captureError(projectFixture(value, fakePort(options)));

    // Assert
    assert.equal(error.category, category);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    await assert.rejects(readFile(`${value.projection}.candidate`, "utf8"), { code: "ENOENT" });
  });
}

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
      : localPort.run({
        ...spec,
        program: "/bin/sh",
        args: [spec.program, ...spec.args],
      }, maybeTimeoutMs),
  };

  // Act
  const evidence = await projectIna260Evidence(
    value.root, value.options, processPort, "git-fixture", validator, validator, value.admitted,
  );

  // Assert
  assert.equal(evidence.source.system_info_projection_valid, true);
  assert.equal(evidence.source.historical_source_semantics_admitted, true);
  assert.equal(evidence.source.current_source_semantics_admitted, true);
  assert.equal(evidence.source.reference_unit_semantics_admitted, true);
});

import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type Ina260Evidence,
  type SystemInfoEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type Ina260EvidenceOptions = {
  readonly attemptRoot: string;
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

export type Ina260AdmittedDigests = {
  readonly sourceProjection: string;
  readonly apiSnapshot: string;
  readonly websocketSnapshot: string;
  readonly finalEvidence: string;
  readonly hardwarePlan: string;
  readonly correctionPlan: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;
type JsonObject = Readonly<Record<string, unknown>>;
type AcquisitionStamp = readonly [number, number, number];
type TelemetrySample = {
  readonly values: readonly [number, number, number];
  readonly stamps: readonly [AcquisitionStamp, AcquisitionStamp, AcquisitionStamp];
};

const expectedSourceProjection =
  "docs/parity/evidence/api002-system-info/system-info-projection.json";
const expectedHardwarePlan = "docs/parity/work-plans/20260812T222308Z-PWR-006/PLAN.md";
const expectedCorrectionPlan = "docs/parity/work-plans/20260816T082924Z-PWR-006/PLAN.md";
const activeTask = "task-parity-pwr006-legacy-wire-units";
const expectedAttemptFiles = [
  "api.private.json",
  "final-evidence.private.json",
  "flash-command-evidence.private.json",
  "flash-monitor.classifier-input.log",
  "retained-log.private.txt",
  "websocket.private.json",
] as const;
const sourcePaths = [
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
const referencePaths = [
  "reference/esp-miner/main/power/INA260.h",
  "reference/esp-miner/main/power/INA260.c",
  "reference/esp-miner/main/power/power.c",
  "reference/esp-miner/main/http_server/system_api_json.c",
  "reference/esp-miner/main/tasks/statistics_task.c",
  "reference/esp-miner/main/http_server/axe-os/src/app/components/home/home.component.ts",
] as const;
const sourceFragments = new Map<string, readonly string[]>([
  [sourcePaths[0], [
    "pub bus_voltage_volts: f64,",
    "pub current_amps: f64,",
    "pub power_watts: f64,",
  ]],
  [sourcePaths[1], [
    "let current_ma = f64::from(i16::from_be_bytes(current)) * INA260_CURRENT_MILLIAMPS_PER_BIT;",
    "let bus_voltage_mv = f64::from(u16::from_be_bytes(bus_voltage)) * INA260_BUS_MILLIVOLTS_PER_BIT;",
    "let power_mw = f64::from(u16::from_be_bytes(power)) * INA260_POWER_MILLIWATTS_PER_BIT;",
    "bus_voltage_volts: bus_voltage_mv / 1000.0,",
    "current_amps: current_ma / 1000.0,",
    "power_watts: power_mw / 1000.0,",
  ]],
  [sourcePaths[2], [
    "Reads one complete INA260 triple through the closed read-only capability.",
    "bus.read_ina260(Ina260ReadRegister::Current, &mut current)",
    "bus.read_ina260(Ina260ReadRegister::BusVoltage, &mut bus_voltage)",
    "bus.read_ina260(Ina260ReadRegister::Power, &mut power)",
    "AcquisitionOutcome::Success(decode_ina260(current, bus_voltage, power))",
  ]],
  [sourcePaths[3], [
    "const INA260_I2C_ADDRESS: u8 = 0x40;",
    "Self::Current => 0x01,",
    "Self::BusVoltage => 0x02,",
    "Self::Power => 0x03,",
    "pub(crate) struct ReadOnlySensorBus",
  ]],
  [sourcePaths[4], [
    "power_watts: project_observation(",
    "bus_voltage_volts: project_observation(",
    "current_amps: project_observation(",
  ]],
  [sourcePaths[5], [
    "pub power_watts: Observation<f64>,",
    "pub bus_voltage_volts: Observation<f64>,",
    "pub current_amps: Observation<f64>,",
  ]],
  [sourcePaths[6], [
    "power_status: (&observations.power_watts).into(),",
    "voltage_status: (&observations.bus_voltage_volts).into(),",
    "current_status: (&observations.current_amps).into(),",
  ]],
  [sourcePaths[7], [
    "const MILLI_UNITS_PER_UNIT: f64 = 1_000.0;",
    "pub(crate) const fn millivolts_from_volts(volts: f64) -> f64",
    "pub(crate) const fn milliamps_from_amps(amps: f64) -> f64",
  ]],
  [sourcePaths[8], [
    "power: safe_telemetry.power_watts,",
    "voltage_millivolts: millivolts_from_volts(safe_telemetry.voltage_volts),",
    "current_milliamps: milliamps_from_amps(safe_telemetry.current_amps),",
  ]],
  [sourcePaths[9], [
    "voltage_millivolts: millivolts_from_volts(safe_telemetry.voltage_volts),",
    "current_milliamps: milliamps_from_amps(safe_telemetry.current_amps),",
  ]],
  [sourcePaths[10], [
    "(4_500.0..=5_500.0).contains(&sample.voltage_millivolts)",
    "sample.current_milliamps >= 0.0",
  ]],
]);
const historicalSourceFragments = new Map<string, readonly string[]>([
  [sourcePaths[8], [
    "voltage: safe_telemetry.voltage_volts,",
    "current: safe_telemetry.current_amps,",
  ]],
  [sourcePaths[9], [
    "voltage: safe_telemetry.voltage_volts,",
    "current: safe_telemetry.current_amps,",
  ]],
]);
const referenceFragments = new Map<string, readonly string[]>([
  [referencePaths[0], [
    "INA260_REG_CURRENT 0x01     ///< Current measurement register (signed) in mA",
    "INA260_REG_BUSVOLTAGE 0x02  ///< Bus voltage measurement register in mV",
    "INA260_REG_POWER 0x03       ///< Power calculation register in mW",
  ]],
  [referencePaths[1], [
    "last_current = (uint16_t)(data[1] | (data[0] << 8)) * 1.25;",
    "last_voltage = (uint16_t)(data[1] | (data[0] << 8)) * 1.25;",
    "last_power = (data[1] | (data[0] << 8)) * 10;",
  ]],
  [referencePaths[2], [
    "pow_val = INA260_read_power() / 1000.0f;",
    "return INA260_read_voltage();",
  ]],
  [referencePaths[3], [
    "cJSON_AddFloatToObject(root, \"voltage\", g->POWER_MANAGEMENT_MODULE.voltage);",
    "cJSON_AddFloatToObject(root, \"current\", g->POWER_MANAGEMENT_MODULE.current);",
    "cJSON_AddFloatToObject(root, \"coreVoltageActual\", g->POWER_MANAGEMENT_MODULE.core_voltage);",
  ]],
  [referencePaths[4], [
    "statsData.voltage = power_management->voltage;",
    "statsData.current = power_management->current;",
    "statsData.coreVoltageActual = power_management->core_voltage;",
  ]],
  [referencePaths[5], [
    "processed.voltage = processed.voltage / 1000;",
    "processed.current = processed.current / 1000;",
    "processed.coreVoltageActual = processed.coreVoltageActual / 1000;",
    "element[idxChartY1Data] = element[idxChartY1Data] / 1000;",
  ]],
]);
const expectedDigests: Ina260AdmittedDigests = {
  sourceProjection: "6ec58fdaeb7cbad3cf103832cd3e59fe470fcb05f6f6a4d41e218ffd6378991a",
  apiSnapshot: "9f0b2809d5e1fea364dadd8029319862b973cf802490f8be7c004dc20f82c2de",
  websocketSnapshot: "f2590f16dda7095f20e77f163ee84f7cea85468000e8ecc0b0cddf18cdc1a859",
  finalEvidence: "6ec58fdaeb7cbad3cf103832cd3e59fe470fcb05f6f6a4d41e218ffd6378991a",
  hardwarePlan: "e58742236746a59fb68afd92a5fe92b181a71e967e43d323789b9f22a58db818",
  correctionPlan: "9cac99ee0fe28580b1c729a9d9681721e07b1ac55b22624fb8073ffe786849f6",
};

export class Ina260EvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "Ina260EvidenceError";
  }
}

function failure(category: FailureCategory, message: string): Ina260EvidenceError {
  return new Ina260EvidenceError(category, message, {
    stage: "sealed_ina260_projection",
    hardware_rerun_used: false,
    projection_published: false,
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function string(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

function finiteNumber(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isFinite(candidate)) {
    throw failure("evidence_invalid", `${context} numeric field is invalid`);
  }
  return candidate;
}

function integer(value: JsonObject, field: string, context: string): number {
  const candidate = finiteNumber(value, field, context);
  if (!Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

function nonnegativeInteger(value: JsonObject, field: string, context: string): number {
  const candidate = finiteNumber(value, field, context);
  if (!Number.isInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

async function jsonFile(candidate: string, context: string): Promise<{
  readonly document: string;
  readonly value: JsonObject;
}> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof Ina260EvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  try {
    const outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
    if (outcome.timedOut || outcome.exitCode !== 0) {
      throw failure("evidence_invalid", `${context} did not pass`);
    }
    return outcome.stdout.trim();
  } catch (error) {
    if (error instanceof Ina260EvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof Ina260EvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function verifyProtectedModes(root: string): Promise<void> {
  const rootMetadata = await stat(root);
  if (!rootMetadata.isDirectory() || (rootMetadata.mode & 0o777) !== 0o700) {
    throw failure("evidence_invalid", "protected attempt root mode is invalid");
  }
  const entries = (await readdir(root)).sort();
  if (entries.length !== expectedAttemptFiles.length
    || entries.some((entry, index) => entry !== expectedAttemptFiles[index])) {
    throw failure("evidence_invalid", "protected attempt file set is invalid");
  }
  for (const entry of entries) {
    const metadata = await stat(path.join(root, entry));
    if (!metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
      throw failure("evidence_invalid", "protected attempt file mode is invalid");
    }
  }
}

function parseSource(document: string): SystemInfoEvidence {
  try {
    return object(JSON.parse(document), "system info projection") as unknown as SystemInfoEvidence;
  } catch (error) {
    if (error instanceof Ina260EvidenceError) throw error;
    throw failure("evidence_invalid", "system info projection is malformed");
  }
}

function validateSource(source: SystemInfoEvidence, attemptSourceCommit: string): void {
  const info = source.system_info;
  if (source.schema_version !== "bitaxe-system-info-evidence-v1"
    || source.board !== 205
    || source.source_commit !== attemptSourceCommit
    || !source.detector_admitted
    || !source.boot_observed
    || !source.same_origin_observed
    || !info["same_boot_session"]
    || !info["websocket_revision_not_earlier"]
    || !info["http_unconditional_fields_complete"]
    || !info["websocket_unconditional_fields_complete"]
    || !info["http_field_types_match"]
    || !info["websocket_field_types_match"]
    || source.mining_state !== "disabled"
    || source.hardware_control_state !== "disabled"
    || !source.cleanup_complete
    || source.redaction_status !== "passed") {
    throw failure("evidence_invalid", "system info source quorum is incomplete");
  }
}

function telemetrySample(snapshot: JsonObject, context: string): TelemetrySample {
  const fields = ["power", "voltage", "current"] as const;
  const values = fields.map((field) => finiteNumber(snapshot, field, context));
  const stamps = fields.map((field) => {
    const status = object(snapshot[`${field}Status`], `${context} ${field} status`);
    if (status["state"] !== "fresh") {
      throw failure("evidence_invalid", `${context} INA260 status is not fresh`);
    }
    const stamp = object(status["stamp"], `${context} ${field} stamp`);
    return [
      nonnegativeInteger(stamp, "bootSession", `${context} ${field} stamp`),
      nonnegativeInteger(stamp, "sequence", `${context} ${field} stamp`),
      nonnegativeInteger(stamp, "acquiredAtMs", `${context} ${field} stamp`),
    ] as const;
  });
  const [power, voltage, current] = values;
  if (power === undefined || voltage === undefined || current === undefined
    || power < 0 || power > 15 || voltage < 4.5 || voltage > 5.5 || current < 0) {
    throw failure("evidence_invalid", `${context} INA260 sample is outside the safe envelope`);
  }
  return {
    values: [power, voltage, current],
    stamps: stamps as [AcquisitionStamp, AcquisitionStamp, AcquisitionStamp],
  };
}

function validateSnapshotIdentity(
  api: JsonObject,
  websocket: JsonObject,
  source: SystemInfoEvidence,
): void {
  for (const field of ["sourceCommit", "referenceCommit", "appElfSha256"] as const) {
    if (string(api, field, "HTTP snapshot") !== string(websocket, field, "WebSocket snapshot")) {
      throw failure("evidence_invalid", "HTTP and WebSocket package identity differs");
    }
  }
  if (string(api, "sourceCommit", "HTTP snapshot") !== source.source_commit
    || string(api, "referenceCommit", "HTTP snapshot") !== source.reference_commit
    || !/^[0-9a-f]{64}$/u.test(string(api, "appElfSha256", "HTTP snapshot"))
    || string(api, "bootSession", "HTTP snapshot") !== string(websocket, "bootSession", "WebSocket snapshot")
    || integer(api, "operatorSnapshotRevision", "HTTP snapshot") !== source.system_info["http_revision"]
    || integer(websocket, "operatorSnapshotRevision", "WebSocket snapshot") !== source.system_info["websocket_revision"]) {
    throw failure("evidence_invalid", "snapshot identity does not match the source projection");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "INA260 source semantic fragment is not unique");
  }
}

async function validateCurrentSourceFragments(root: string): Promise<void> {
  for (const [sourcePath, fragments] of sourceFragments) {
    const document = await readFile(path.join(root, sourcePath), "utf8");
    for (const fragment of fragments) requireUniqueFragment(document, fragment);
  }
}

async function validateHistoricalSourceFragments(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
): Promise<void> {
  for (const [sourcePath, fragments] of historicalSourceFragments) {
    const document = await childText(
      processPort,
      gitProgram,
      ["show", `${attemptSourceCommit}:${sourcePath}`],
      "historical INA260 source",
    );
    for (const fragment of fragments) requireUniqueFragment(document, fragment);
  }
}

async function validateReferenceFragments(root: string): Promise<void> {
  for (const [sourcePath, fragments] of referenceFragments) {
    const document = await readFile(path.join(root, sourcePath), "utf8");
    for (const fragment of fragments) requireUniqueFragment(document, fragment);
  }
}

function validateTaskAndPlans(
  taskDocument: string,
  hardwarePlanDocument: string,
  correctionPlanDocument: string,
  hardwarePlanDigest: string,
  correctionPlanDigest: string,
): void {
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "PWR-006 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const block = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [expectedCorrectionPlan, "millivolts and milliamps", "read-only reuse"]) {
    if (!block.includes(required)) throw failure("evidence_invalid", "PWR-006 task contract is incomplete");
  }
  if (sha256(hardwarePlanDocument) !== hardwarePlanDigest
    || !hardwarePlanDocument.includes("- Parity row: `PWR-006`")
    || sha256(correctionPlanDocument) !== correctionPlanDigest
    || !correctionPlanDocument.includes("- Parity row: `PWR-006`")
    || !correctionPlanDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "PWR-006 immutable plan binding is invalid");
  }
}

export async function projectIna260Evidence(
  workspaceRoot: string,
  options: Ina260EvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admitted: Ina260AdmittedDigests = expectedDigests,
): Promise<Ina260Evidence> {
  const attemptRoot = assertWithinWorkspace(workspaceRoot, options.attemptRoot);
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "system info source projection path is invalid");
  }
  await requireAbsent(projection, "final INA260 projection");
  await requireAbsent(candidate, "INA260 projection candidate");

  try {
    await verifyProtectedModes(attemptRoot);
    const sourceFile = await jsonFile(sourceProjection, "system info source projection");
    const finalFile = await jsonFile(path.join(attemptRoot, "final-evidence.private.json"), "private final evidence");
    const apiFile = await jsonFile(path.join(attemptRoot, "api.private.json"), "private HTTP snapshot");
    const websocketFile = await jsonFile(path.join(attemptRoot, "websocket.private.json"), "private WebSocket snapshot");
    const hardwarePlanDocument = await readFile(path.join(workspaceRoot, expectedHardwarePlan), "utf8");
    const correctionPlanDocument = await readFile(path.join(workspaceRoot, expectedCorrectionPlan), "utf8");
    const taskDocument = await readFile(path.join(workspaceRoot, "TASKS.md"), "utf8");
    const observedDigests = {
      sourceProjection: sha256(sourceFile.document),
      apiSnapshot: sha256(apiFile.document),
      websocketSnapshot: sha256(websocketFile.document),
      finalEvidence: sha256(finalFile.document),
      hardwarePlan: sha256(hardwarePlanDocument),
      correctionPlan: sha256(correctionPlanDocument),
    };
    for (const key of Object.keys(admitted) as Array<keyof Ina260AdmittedDigests>) {
      if (observedDigests[key] !== admitted[key]) {
        throw failure("evidence_invalid", "INA260 admitted digest is invalid");
      }
    }
    if (sourceFile.document !== finalFile.document) {
      throw failure("evidence_invalid", "public and private system info evidence differ");
    }
    validateTaskAndPlans(
      taskDocument,
      hardwarePlanDocument,
      correctionPlanDocument,
      admitted.hardwarePlan,
      admitted.correctionPlan,
    );
    const source = parseSource(sourceFile.document);
    validateSource(source, options.attemptSourceCommit);
    await childText(processPort, sourceValidatorProgram, [sourceProjection], "system info validator");

    const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity");
    const referenceCommit = await childText(
      processPort, gitProgram, ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference identity",
    );
    if (!/^[0-9a-f]{40}$/u.test(currentSourceCommit)
      || referenceCommit !== source.reference_commit
      || source.source_commit !== options.attemptSourceCommit) {
      throw failure("evidence_invalid", "INA260 source or reference identity is invalid");
    }
    await childText(processPort, gitProgram,
      ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit], "source ancestry");
    const dirty = await childText(processPort, gitProgram, ["status", "--porcelain", "--", ...sourcePaths],
      "INA260 source cleanliness");
    if (dirty !== "") throw failure("evidence_invalid", "INA260 source paths are dirty");
    const referenceDirty = await childText(
      processPort,
      gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "status", "--porcelain"],
      "reference source cleanliness",
    );
    if (referenceDirty !== "") throw failure("evidence_invalid", "INA260 reference source is dirty");
    await validateHistoricalSourceFragments(processPort, gitProgram, options.attemptSourceCommit);
    await validateCurrentSourceFragments(workspaceRoot);
    await validateReferenceFragments(workspaceRoot);

    const envelope = websocketFile.value;
    if (envelope["event"] !== "update") throw failure("evidence_invalid", "WebSocket event is invalid");
    const websocket = object(envelope["data"], "WebSocket snapshot");
    validateSnapshotIdentity(apiFile.value, websocket, source);
    const httpSample = telemetrySample(apiFile.value, "HTTP snapshot");
    const websocketSample = telemetrySample(websocket, "WebSocket snapshot");
    if (JSON.stringify(httpSample.values) !== JSON.stringify(websocketSample.values)
      || JSON.stringify(httpSample.stamps) !== JSON.stringify(websocketSample.stamps)) {
      throw failure("evidence_invalid", "HTTP and WebSocket INA260 samples are not correlated");
    }

    const evidence: Ina260Evidence = {
      schema_version: "bitaxe-ina260-evidence-v2",
      board: 205,
      attempt_source_commit: options.attemptSourceCommit,
      current_source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: source.package_manifest_sha256,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "project-ina260-evidence",
        request_sha256: sha256(JSON.stringify({
          source: observedDigests.sourceProjection,
          api: observedDigests.apiSnapshot,
          websocket: observedDigests.websocketSnapshot,
          hardware_plan: observedDigests.hardwarePlan,
          correction_plan: observedDigests.correctionPlan,
          attempt_source_commit: options.attemptSourceCommit,
        })),
      },
      source: {
        system_info_projection_sha256: observedDigests.sourceProjection,
        api_snapshot_sha256: observedDigests.apiSnapshot,
        websocket_snapshot_sha256: observedDigests.websocketSnapshot,
        final_evidence_sha256: observedDigests.finalEvidence,
        system_info_projection_valid: true,
        protected_modes_valid: true,
        hardware_plan_sha256: observedDigests.hardwarePlan,
        correction_plan_sha256: observedDigests.correctionPlan,
        historical_source_semantics_admitted: true,
        current_source_semantics_admitted: true,
        reference_unit_semantics_admitted: true,
        current_source_path_count: sourcePaths.length,
        reference_path_count: referencePaths.length,
      },
      telemetry: {
        i2c_address: 0x40,
        current_register: 0x01,
        bus_voltage_register: 0x02,
        power_register: 0x03,
        complete_register_set: true,
        read_only_acquisition: true,
        historical_http_complete_fresh_sample: true,
        historical_websocket_complete_fresh_sample: true,
        historical_si_safe_ranges: true,
        same_historical_values: true,
        same_states: true,
        same_acquisition_stamps: true,
        same_boot_session: true,
        exact_package_identity: true,
        legacy_voltage_unit: "millivolts",
        legacy_current_unit: "milliamps",
        core_voltage_unit: "millivolts",
        power_unit: "watts",
        nominal_voltage_unit: "volts",
        volts_to_millivolts_factor: 1_000,
        amps_to_milliamps_factor: 1_000,
        system_info_conversion_proved: true,
        statistics_conversion_proved: true,
        campaign_min_input_millivolts: 4_500,
        campaign_max_input_millivolts: 5_500,
        campaign_safety_range_preserved: true,
      },
      detector_admitted: true,
      boot_observed: true,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      hardware_rerun_used: false,
      redaction_status: "passed",
    };
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8", flag: "wx", mode: 0o600,
    });
    await chmod(candidate, 0o600);
    await childText(processPort, validatorProgram, [candidate], "INA260 evidence validator");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof Ina260EvidenceError) throw error;
    throw failure("evidence_invalid", "INA260 evidence processing failed");
  }
}

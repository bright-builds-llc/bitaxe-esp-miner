import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type Emc2101ThermalEvidence,
  type SystemInfoEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import {
  captureSystemInfoEvidence,
  SystemInfoEvidenceError,
} from "./system-info-evidence.js";
import type { WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type Emc2101ThermalEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly detectorOutput: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<
  AutomationCategory,
  "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed"
>;
type AcquisitionStamp = readonly [number, number, number];
type ThermalSample = {
  readonly temperature: number;
  readonly state: string;
  readonly stamp: AcquisitionStamp;
};

const expectedPrivateRoot = "scratch/thr001-emc2101/attempt-002";
const expectedWrapperRoot = "scratch/thr001-emc2101/wrapper-002";
const expectedProjection =
  "docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json";
const expectedPlan = "docs/parity/work-plans/20260813T011207Z-THR-001/PLAN.md";
const expectedPlanSha256 = "02515b8d8d8c691a1a036026fa47c3f9d1caef0d504bcf4d3541aef9fb87e909";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-thr001-emc2101-live-thermal";
const minimumPlausibleTemperatureCelsius = -40;
const maximumPlausibleTemperatureCelsius = 150;
const asicThrottleTemperatureCelsius = 75;
const expectedAttemptFiles = [
  "api.private.json",
  "final-evidence.private.json",
  "flash-command-evidence.private.json",
  "flash-monitor.classifier-input.log",
  "retained-log.private.txt",
  "websocket.private.json",
] as const;
const sourcePaths = [
  "crates/bitaxe-safety/src/sensor_acquisition/emc2101.rs",
  "crates/bitaxe-safety/src/thermal.rs",
  "firmware/bitaxe/src/safety_adapter/emc2101.rs",
  "firmware/bitaxe/src/safety_adapter/i2c_bus.rs",
  "firmware/bitaxe/src/operator_sensor_runtime.rs",
  "crates/bitaxe-api/src/observation.rs",
  "crates/bitaxe-api/src/wire.rs",
] as const;
const sourceFragments = new Map<string, readonly string[]>([
  [sourcePaths[0], [
    "pub const ULTRA205_EMC2101_TEMP_OFFSET_C: f64 = 5.0;",
    "pub fn apply_ultra205_emc2101_temperature_offset(",
    "validate_temperature(temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C)",
  ]],
  [sourcePaths[1], [
    "pub const ASIC_THROTTLE_TEMP_C: f64 = 75.0;",
    "pub const MIN_PLAUSIBLE_TEMP_C: f64 = -40.0;",
    "pub const MAX_PLAUSIBLE_TEMP_C: f64 = 150.0;",
  ]],
  [sourcePaths[2], [
    "Self::InternalTemperature => 0x00,",
    "read_internal_temperature_acquisition(bus)",
    "apply_ultra205_emc2101_temperature_offset(temperature)",
  ]],
  [sourcePaths[3], [
    "const EMC2101_I2C_ADDRESS: u8 = 0x4c;",
    "pub(crate) struct ReadOnlySensorBus",
    "self.read_register(EMC2101_I2C_ADDRESS, register.address(), output)",
  ]],
  [sourcePaths[4], [
    "chip_temp_celsius: project_observation(",
  ]],
  [sourcePaths[5], [
    "pub chip_temp_celsius: Observation<f64>,",
    "(MIN_PLAUSIBLE_TEMP_C..ASIC_THROTTLE_TEMP_C).contains(&chip_temp_celsius)",
  ]],
  [sourcePaths[6], [
    "pub temp: f64,",
    "pub chip_temp_status: ObservationTruthWire,",
    "temp: safe_telemetry.chip_temp_celsius,",
  ]],
]);
const referenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/device_config.h", [
    '.board_version = "205",  .family = FAMILY_ULTRA,       .EMC2101 = true, .emc_internal_temp = true,                                  .temp_offset = 5,',
  ]],
  ["reference/esp-miner/main/thermal/EMC2101.c", [
    "float EMC2101_get_internal_temp(void)",
    "EMC2101_INTERNAL_TEMP",
    "return (float) temp + temp_offset;",
  ]],
]);

export class Emc2101ThermalEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "Emc2101ThermalEvidenceError";
  }
}

function failure(
  category: FailureCategory,
  message: string,
  recovery: Readonly<Record<string, unknown>> = {},
): Emc2101ThermalEvidenceError {
  return new Emc2101ThermalEvidenceError(category, message, {
    stage: "emc2101_thermal_capture",
    projection_published: false,
    ...recovery,
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

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

function requiredNumber(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isFinite(candidate)) {
    throw failure("evidence_invalid", `${context} numeric field is invalid`);
  }
  return candidate;
}

function requiredNonnegativeInteger(value: JsonObject, field: string, context: string): number {
  const candidate = requiredNumber(value, field, context);
  if (!Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof Emc2101ThermalEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
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
    if (outcome.timedOut) throw failure("timeout", `${context} timed out`);
    if (outcome.exitCode !== 0) throw failure("evidence_invalid", `${context} did not pass`);
    return outcome.stdout.trim();
  } catch (error) {
    if (error instanceof Emc2101ThermalEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "protected evidence mode is invalid");
  }
}

async function verifyProtectedLayout(
  workspaceRoot: string,
  privateRoot: string,
  wrapperRoot: string,
): Promise<void> {
  await requireMode(wrapperRoot, 0o700, true);
  for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await requireMode(path.join(wrapperRoot, name), 0o600, false);
  }
  await requireMode(privateRoot, 0o700, true);
  const entries = (await readdir(privateRoot)).sort();
  if (entries.length !== expectedAttemptFiles.length
    || entries.some((entry, index) => entry !== expectedAttemptFiles[index])) {
    throw failure("evidence_invalid", "protected attempt file set is invalid");
  }
  for (const entry of entries) await requireMode(path.join(privateRoot, entry), 0o600, false);
  const relativeWrapper = path.relative(workspaceRoot, wrapperRoot);
  if (relativeWrapper !== expectedWrapperRoot) {
    throw failure("evidence_invalid", "protected wrapper root is invalid");
  }
}

async function readJson(candidate: string, context: string): Promise<{
  readonly document: string;
  readonly value: JsonObject;
}> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof Emc2101ThermalEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

function thermalSample(snapshot: JsonObject, context: string): ThermalSample {
  const temperature = requiredNumber(snapshot, "temp", context);
  if (temperature < minimumPlausibleTemperatureCelsius
    || temperature > maximumPlausibleTemperatureCelsius
    || temperature >= asicThrottleTemperatureCelsius) {
    throw failure("hardware_blocked", `${context} thermal sample is outside the admitted envelope`);
  }
  const status = object(snapshot["chipTempStatus"], `${context} chip temperature status`);
  if (status["state"] !== "fresh") {
    throw failure("hardware_blocked", `${context} chip temperature is not fresh`);
  }
  const stamp = object(status["stamp"], `${context} chip temperature stamp`);
  return {
    temperature,
    state: "fresh",
    stamp: [
      requiredNonnegativeInteger(stamp, "bootSession", `${context} chip temperature stamp`),
      requiredNonnegativeInteger(stamp, "sequence", `${context} chip temperature stamp`),
      requiredNonnegativeInteger(stamp, "acquiredAtMs", `${context} chip temperature stamp`),
    ],
  };
}

function validateSnapshotIdentity(
  api: JsonObject,
  websocket: JsonObject,
  source: SystemInfoEvidence,
): void {
  for (const field of ["sourceCommit", "referenceCommit", "appElfSha256"] as const) {
    if (requiredString(api, field, "HTTP snapshot")
      !== requiredString(websocket, field, "WebSocket snapshot")) {
      throw failure("evidence_invalid", "HTTP and WebSocket package identity differs");
    }
  }
  if (requiredString(api, "sourceCommit", "HTTP snapshot") !== source.source_commit
    || requiredString(api, "referenceCommit", "HTTP snapshot") !== source.reference_commit
    || !/^[0-9a-f]{64}$/u.test(requiredString(api, "appElfSha256", "HTTP snapshot"))
    || requiredString(api, "bootSession", "HTTP snapshot")
      !== requiredString(websocket, "bootSession", "WebSocket snapshot")) {
    throw failure("evidence_invalid", "snapshot identity does not match exact package evidence");
  }
}

function validateTaskAndPlan(
  taskDocument: string,
  planDocument: string,
  admittedPlanSha256: string,
): void {
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "THR-001 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const block = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [expectedPlan, "bitaxe-emc2101-thermal-evidence-v1", "attempt-002"]) {
    if (!block.includes(required)) throw failure("evidence_invalid", "THR-001 task contract is incomplete");
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `THR-001`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "THR-001 immutable plan binding is invalid");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const normalizedDocument = document.replaceAll(/\s+/gu, "");
  const normalizedFragment = fragment.replaceAll(/\s+/gu, "");
  const first = normalizedDocument.indexOf(normalizedFragment);
  if (first === -1
    || normalizedDocument.indexOf(normalizedFragment, first + normalizedFragment.length) !== -1) {
    throw failure("evidence_invalid", "thermal source semantic fragment is not unique");
  }
}

export async function validateEmc2101SourceSemantics(workspaceRoot: string): Promise<void> {
  for (const [sourcePath, fragments] of [...sourceFragments, ...referenceFragments]) {
    const document = await readFile(path.join(workspaceRoot, sourcePath), "utf8");
    for (const fragment of fragments) requireUniqueFragment(document, fragment);
  }
}

function recoveryFacts(error: SystemInfoEvidenceError): Readonly<Record<string, unknown>> {
  const facts: Record<string, unknown> = {};
  for (const field of ["recovery_complete", "recovery_flash_used", "secondary_recovery_failure"]) {
    const value = error.publicValue[field];
    if (typeof value === "boolean") facts[field] = value;
  }
  return facts;
}

export async function captureEmc2101ThermalEvidence(
  workspaceRoot: string,
  options: Emc2101ThermalEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  systemInfoValidatorProgram: string,
  thermalValidatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<Emc2101ThermalEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const sourceProjection = path.join(wrapperRoot, "system-info-projection.private.json");
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || path.basename(detectorOutput) !== "detector.stdout") {
    throw failure("evidence_invalid", "THR-001 protected path contract is invalid");
  }
  await requireAbsent(sourceProjection, "protected system info projection");
  await requireAbsent(projection, "final thermal projection");
  await requireAbsent(candidate, "thermal projection candidate");

  let source: SystemInfoEvidence;
  try {
    source = await captureSystemInfoEvidence(workspaceRoot, {
      privateRoot: options.privateRoot,
      packageManifest: options.packageManifest,
      wifiCredentials: options.wifiCredentials,
      port: options.port,
      projection: path.relative(workspaceRoot, sourceProjection),
      captureTimeoutSeconds: options.captureTimeoutSeconds,
    }, processPort, flashProgram, systemInfoValidatorProgram, maybeWebSocketFactory);
  } catch (error) {
    if (error instanceof SystemInfoEvidenceError) {
      throw failure(error.category, error.message, recoveryFacts(error));
    }
    throw failure("evidence_invalid", "base system info capture failed");
  }

  try {
    await chmod(sourceProjection, 0o600);
    await verifyProtectedLayout(workspaceRoot, privateRoot, wrapperRoot);
    const sourceFile = await readJson(sourceProjection, "protected system info projection");
    const finalSourceFile = await readJson(
      path.join(privateRoot, "final-evidence.private.json"),
      "private system info evidence",
    );
    if (sourceFile.document !== finalSourceFile.document) {
      throw failure("evidence_invalid", "protected system info evidence differs");
    }
    const apiFile = await readJson(path.join(privateRoot, "api.private.json"), "private HTTP snapshot");
    const websocketFile = await readJson(
      path.join(privateRoot, "websocket.private.json"),
      "private WebSocket envelope",
    );
    const websocket = object(websocketFile.value["data"], "private WebSocket snapshot");
    if (websocketFile.value["event"] !== "update") {
      throw failure("evidence_invalid", "private WebSocket event is invalid");
    }
    const planDocument = await readFile(path.join(workspaceRoot, expectedPlan), "utf8");
    const taskDocument = await readFile(path.join(workspaceRoot, "TASKS.md"), "utf8");
    validateTaskAndPlan(taskDocument, planDocument, admittedPlanSha256);

    const currentSourceCommit = await childText(
      processPort,
      gitProgram,
      ["rev-parse", "HEAD"],
      "current source identity",
    );
    const pushedSourceCommit = await childText(
      processPort,
      gitProgram,
      ["rev-parse", "origin/main"],
      "pushed source identity",
    );
    const referenceCommit = await childText(
      processPort,
      gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference identity",
    );
    const dirty = await childText(
      processPort,
      gitProgram,
      ["status", "--porcelain", "--untracked-files=no"],
      "source cleanliness",
    );
    if (!/^[0-9a-f]{40}$/u.test(currentSourceCommit)
      || pushedSourceCommit !== currentSourceCommit
      || source.source_commit !== currentSourceCommit
      || source.reference_commit !== expectedReferenceCommit
      || referenceCommit !== expectedReferenceCommit
      || dirty !== "") {
      throw failure("evidence_invalid", "exact clean pushed source identity is invalid");
    }
    await validateEmc2101SourceSemantics(workspaceRoot);
    await childText(
      processPort,
      systemInfoValidatorProgram,
      [sourceProjection],
      "system info evidence validator",
    );

    validateSnapshotIdentity(apiFile.value, websocket, source);
    const httpSample = thermalSample(apiFile.value, "HTTP snapshot");
    const websocketSample = thermalSample(websocket, "WebSocket snapshot");
    if (httpSample.temperature !== websocketSample.temperature
      || httpSample.state !== websocketSample.state
      || JSON.stringify(httpSample.stamp) !== JSON.stringify(websocketSample.stamp)) {
      throw failure("evidence_invalid", "HTTP and WebSocket thermal samples are not correlated");
    }

    const manifestDocument = await readFile(
      assertWithinWorkspace(workspaceRoot, options.packageManifest),
      "utf8",
    );
    if (sha256(manifestDocument) !== source.package_manifest_sha256) {
      throw failure("evidence_invalid", "package manifest digest does not match source evidence");
    }
    const evidence: Emc2101ThermalEvidence = {
      schema_version: "bitaxe-emc2101-thermal-evidence-v1",
      board: 205,
      attempt_ordinal: 2,
      source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: source.package_manifest_sha256,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-emc2101-thermal-evidence",
        request_sha256: sha256(JSON.stringify({
          source: sha256(sourceFile.document),
          api: sha256(apiFile.document),
          websocket: sha256(websocketFile.document),
          plan: admittedPlanSha256,
          timeout: options.captureTimeoutSeconds,
        })),
      },
      source: {
        system_info_projection_sha256: sha256(sourceFile.document),
        api_snapshot_sha256: sha256(apiFile.document),
        websocket_snapshot_sha256: sha256(websocketFile.document),
        plan_sha256: admittedPlanSha256,
        system_info_projection_valid: true,
        protected_modes_valid: true,
        production_source_current: true,
        source_semantics_admitted: true,
        compatible_path_count: sourcePaths.length,
      },
      thermal: {
        i2c_address: 0x4c,
        internal_temperature_register: 0x00,
        temperature_offset_celsius: 5,
        read_only_acquisition: true,
        http_fresh_sample: true,
        websocket_fresh_sample: true,
        finite_plausible_range: true,
        below_throttle_threshold: true,
        same_temperature: true,
        same_state: true,
        same_acquisition_stamp: true,
        same_boot_session: true,
        exact_package_identity: true,
      },
      detector_admitted: true,
      boot_observed: true,
      mining_state: "disabled",
      hardware_control_state: "disabled",
      cleanup_complete: true,
      recovery_used: false,
      redaction_status: "passed",
    };
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await chmod(candidate, 0o600);
    await childText(processPort, thermalValidatorProgram, [candidate], "thermal evidence validator");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof Emc2101ThermalEvidenceError) throw error;
    throw failure("evidence_invalid", "EMC2101 thermal evidence processing failed");
  }
}

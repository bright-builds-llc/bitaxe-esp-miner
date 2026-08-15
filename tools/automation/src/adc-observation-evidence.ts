import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AdcObservationEvidence,
  type AutomationCategory,
  type SystemInfoEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import {
  captureSystemInfoEvidence,
  SystemInfoEvidenceError,
} from "./system-info-evidence.js";
import type { WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AdcObservationEvidenceOptions = {
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
const expectedPrivateRoot = "scratch/io002-adc/attempt-001";
const expectedWrapperRoot = "scratch/io002-adc/wrapper-001";
const expectedProjection =
  "docs/parity/evidence/io002-adc/adc-observation-projection.json";
const expectedPlan = "docs/parity/work-plans/20260815T210711Z-IO-002/PLAN.md";
const expectedPlanSha256 = "bb0db9d7338e79d86bd4a97105e85805db599593f82da06360505836b4506fb6";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-io002-adc-observation";
const expectedAttemptFiles = [
  "api.private.json",
  "final-evidence.private.json",
  "flash-command-evidence.private.json",
  "flash-monitor.classifier-input.log",
  "retained-log.private.txt",
  "websocket.private.json",
] as const;
const sourcePaths = [
  "crates/bitaxe-core/src/runtime_orchestration.rs",
  "crates/bitaxe-safety/src/core_voltage_acquisition.rs",
  "firmware/bitaxe/src/safety_adapter/adc.rs",
  "firmware/bitaxe/src/operator_sensor_runtime.rs",
  "crates/bitaxe-api/src/observation.rs",
  "crates/bitaxe-api/src/snapshot.rs",
  "crates/bitaxe-api/src/wire.rs",
] as const;
const sourceFragments = new Map<string, readonly string[]>([
  [sourcePaths[0], ["pub const OPERATOR_OBSERVATION_CADENCE_MS: u64 = 500;"]],
  [sourcePaths[1], [
    "AcquisitionOutcome::Success(millivolts)",
    "FaultReason::AdcReadFailed",
    "StaleReason::ProducerCadenceExpired",
  ]],
  [sourcePaths[2], [
    "ADC1<'static>",
    "Gpio2<'static>",
    "attenuation: attenuation::DB_12",
    "resolution: Resolution::new()",
    "calibration: Calibration::Curve",
  ]],
  [sourcePaths[3], [
    "pub const SENSOR_SWEEP_CADENCE_MS: u64 = OPERATOR_OBSERVATION_CADENCE_MS;",
    "safety_adapter::read_core_voltage_acquisition(adc)",
    "core_voltage_state.record(core_voltage_millivolts, boot_session, acquired_at)",
  ]],
  [sourcePaths[4], ["pub core_voltage_actual_mv: Observation<f64>"]],
  [sourcePaths[5], [
    "core_voltage_actual_mv: fresh_f64(&observations.core_voltage_actual_mv)",
    "core_voltage_status: (&observations.core_voltage_actual_mv).into()",
  ]],
  [sourcePaths[6], [
    '#[serde(rename = "coreVoltageActual")]',
    '#[serde(rename = "coreVoltageActualStatus")]',
    "core_voltage_actual: safe_telemetry.core_voltage_actual_mv",
  ]],
]);
const referenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/adc.c", [
    "#define ADC_ATTEN   ADC_ATTEN_DB_12",
    "#define ADC_CHANNEL ADC_CHANNEL_1",
    ".unit_id = ADC_UNIT_1",
    ".bitwidth = ADC_BITWIDTH_DEFAULT",
    "adc_cali_create_scheme_curve_fitting",
    "adc_cali_raw_to_voltage(adc1_cali_chan1_handle, adc_raw, &voltage)",
  ]],
]);

export class AdcObservationEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AdcObservationEvidenceError";
  }
}

function failure(
  category: FailureCategory,
  message: string,
  recovery: Readonly<Record<string, unknown>> = {},
): AdcObservationEvidenceError {
  return new AdcObservationEvidenceError(category, message, {
    stage: "adc_observation_capture",
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

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof AdcObservationEvidenceError) throw error;
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
    if (error instanceof AdcObservationEvidenceError) throw error;
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
  if (path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot) {
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
    if (error instanceof AdcObservationEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
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
    throw failure("evidence_invalid", "IO-002 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const block = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [expectedPlan, "bitaxe-adc-observation-evidence-v1", "attempt-001"]) {
    if (!block.includes(required)) throw failure("evidence_invalid", "IO-002 task contract is incomplete");
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `IO-002`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "IO-002 immutable plan binding is invalid");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const normalizedDocument = document.replaceAll(/\s+/gu, "");
  const normalizedFragment = fragment.replaceAll(/\s+/gu, "");
  const first = normalizedDocument.indexOf(normalizedFragment);
  if (first === -1
    || normalizedDocument.indexOf(normalizedFragment, first + normalizedFragment.length) !== -1) {
    throw failure("evidence_invalid", "ADC source semantic fragment is not unique");
  }
}

export async function validateAdcObservationSourceSemantics(workspaceRoot: string): Promise<void> {
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

export async function captureAdcObservationEvidence(
  workspaceRoot: string,
  options: AdcObservationEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  systemInfoValidatorProgram: string,
  adcInputValidatorProgram: string,
  adcValidatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<AdcObservationEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const sourceProjection = path.join(wrapperRoot, "system-info-projection.private.json");
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || path.basename(detectorOutput) !== "detector.stdout") {
    throw failure("evidence_invalid", "IO-002 protected path contract is invalid");
  }
  await requireAbsent(sourceProjection, "protected system info projection");
  await requireAbsent(projection, "final ADC projection");
  await requireAbsent(candidate, "ADC projection candidate");

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

    const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity");
    const pushedSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "origin/main"], "pushed source identity");
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
    await validateAdcObservationSourceSemantics(workspaceRoot);
    await childText(processPort, systemInfoValidatorProgram, [sourceProjection], "system info evidence validator");
    await childText(
      processPort,
      adcInputValidatorProgram,
      [path.join(privateRoot, "api.private.json"), path.join(privateRoot, "websocket.private.json")],
      "ADC input validator",
    );
    validateSnapshotIdentity(apiFile.value, websocket, source);

    const manifestDocument = await readFile(
      assertWithinWorkspace(workspaceRoot, options.packageManifest),
      "utf8",
    );
    if (sha256(manifestDocument) !== source.package_manifest_sha256) {
      throw failure("evidence_invalid", "package manifest digest does not match source evidence");
    }
    const evidence: AdcObservationEvidence = {
      schema_version: "bitaxe-adc-observation-evidence-v1",
      board: 205,
      attempt_ordinal: 1,
      source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: source.package_manifest_sha256,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-adc-observation-evidence",
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
      adc: {
        adc_unit: 1,
        adc_channel: 1,
        gpio: 2,
        attenuation_db: 12,
        default_resolution: true,
        curve_calibration: true,
        producer_cadence_ms: 500,
        read_only_acquisition: true,
        http_fresh_sample: true,
        websocket_fresh_sample: true,
        finite_positive_millivolts: true,
        plausible_millivolt_range: true,
        sequence_not_regressed: true,
        acquisition_time_not_regressed: true,
        same_boot_session: true,
        exact_public_correlation: true,
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
    await childText(processPort, adcValidatorProgram, [candidate], "ADC evidence validator");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof AdcObservationEvidenceError) throw error;
    throw failure("evidence_invalid", "ADC observation evidence processing failed");
  }
}

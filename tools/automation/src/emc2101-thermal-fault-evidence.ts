import { createHash, randomBytes } from "node:crypto";
import {
  chmod,
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import {
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type Emc2101ThermalFaultEvidence,
  type SystemInfoEvidence,
} from "./contracts.generated.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import type { ProcessPort } from "./process.js";
import {
  captureSystemInfoEvidence,
  SystemInfoEvidenceError,
} from "./system-info-evidence.js";
import type { WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type Emc2101ThermalFaultEvidenceOptions = {
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
type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};

const expectedPrivateRoot = "scratch/thr001-emc2101-fault/attempt-005";
const expectedWrapperRoot = "scratch/thr001-emc2101-fault/wrapper-005";
const expectedProjection =
  "docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-005.json";
const expectedPriorProjection =
  "docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json";
const expectedPlan = "docs/parity/work-plans/20260815T182438Z-THR-001/PLAN.md";
const expectedPlanSha256 = "8e8049fd6fbb19575f6abe593afcdd9ac2303eee0204b5f188d4b65aa7607d58";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-thr001-emc2101-live-thermal";
const stimulusKind = "emc2101_invalid_sample";
const markerLines = [
  "thermal_fault_stimulus state=baseline_ready redacted=true",
  "thermal_fault_stimulus state=fault_observed redacted=true",
  "thermal_fault_stimulus state=recovered redacted=true",
] as const;
const noRecovery: RecoveryFacts = {
  recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

export class Emc2101ThermalFaultEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "Emc2101ThermalFaultEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): Emc2101ThermalFaultEvidenceError {
    return new Emc2101ThermalFaultEvidenceError(
      this.category,
      this.message,
      { ...this.publicValue, ...recovery },
    );
  }
}

function failure(
  category: FailureCategory,
  message: string,
  recovery: RecoveryFacts = noRecovery,
): Emc2101ThermalFaultEvidenceError {
  return new Emc2101ThermalFaultEvidenceError(category, message, {
    stage: "emc2101_thermal_fault_capture",
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
    throw failure("evidence_invalid", `${context} identity is invalid`);
  }
  return candidate;
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof Emc2101ThermalFaultEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "protected evidence mode is invalid");
  }
}

async function createPrivateRoot(root: string): Promise<void> {
  await requireAbsent(root, "private attempt root");
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function privateJson(output: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(output, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
  return document;
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
    if (error instanceof Emc2101ThermalFaultEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

function validateTaskAndPlan(task: string, plan: string): void {
  const heading = `### ${activeTask} |`;
  const start = task.indexOf(heading);
  if (start === -1 || task.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "THR-001 active task binding is invalid");
  }
  const maybeEnd = task.indexOf("\n### ", start + heading.length);
  const block = task.slice(start, maybeEnd === -1 ? task.length : maybeEnd);
  for (const required of [
    expectedPlan,
    "bitaxe-emc2101-thermal-fault-evidence-v1",
    "attempt-005",
  ]) {
    if (!block.includes(required)) {
      throw failure("evidence_invalid", "THR-001 task contract is incomplete");
    }
  }
  if (sha256(plan) !== expectedPlanSha256
    || !plan.includes("- Parity row: `THR-001`")
    || !plan.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "THR-001 immutable plan binding is invalid");
  }
}

function bitaxeInfoPayload(line: string): string | undefined {
  return /^I \([0-9]+\) bitaxe_firmware: (.+)$/u.exec(line)?.[1];
}

function validateMarkerSequence(monitor: string): void {
  if (!hasPassiveSafeState(monitor)) {
    throw failure("evidence_invalid", "stimulus boot lacks passive safe-state evidence");
  }
  const observed = monitor
    .split(/\r?\n/u)
    .map(bitaxeInfoPayload)
    .filter((maybePayload): maybePayload is string =>
      maybePayload?.startsWith("thermal_fault_stimulus state=") === true);
  const hasCompleteWitness = observed.some((_, start) =>
    markerLines.every((expected, offset) => observed[start + offset] === expected));
  if (!hasCompleteWitness) {
    throw failure("evidence_invalid", "thermal fault marker sequence is incomplete");
  }
}

function validateRestoredThermalTruth(api: JsonObject, websocket: JsonObject): void {
  for (const snapshot of [api, websocket]) {
    const status = object(snapshot["chipTempStatus"], "restored temperature status");
    const temperature = snapshot["temp"];
    if (status["state"] !== "fresh"
      || Object.hasOwn(status, "reason")
      || typeof temperature !== "number"
      || !Number.isFinite(temperature)
      || temperature < -40
      || temperature >= 75) {
      throw failure("hardware_blocked", "restored thermal truth is not fresh and safe");
    }
  }
}

function recoveryFromSystemInfo(error: SystemInfoEvidenceError): RecoveryFacts {
  return {
    recovery_complete: error.publicValue["recovery_complete"] === true,
    recovery_flash_used: error.publicValue["recovery_flash_used"] === true,
    secondary_recovery_failure: error.publicValue["secondary_recovery_failure"] === true,
  };
}

async function verifyProtectedLayout(
  privateRoot: string,
  wrapperRoot: string,
  restoreProjection: string,
): Promise<void> {
  await requireMode(privateRoot, 0o700, true);
  await requireMode(wrapperRoot, 0o700, true);
  for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await requireMode(path.join(wrapperRoot, name), 0o600, false);
  }
  await requireMode(path.join(privateRoot, "thermal-fault-intent.private.json"), 0o600, false);
  await requireMode(restoreProjection, 0o600, false);
  for (const name of ["stimulus", "restore"]) {
    await requireMode(path.join(privateRoot, name), 0o700, true);
  }
  const stimulusEntries = (await readdir(path.join(privateRoot, "stimulus"))).sort();
  if (stimulusEntries.join(",")
    !== "flash-command-evidence.private.json,flash-monitor.classifier-input.log") {
    throw failure("evidence_invalid", "stimulus private file set is invalid");
  }
  const restoreEntries = (await readdir(path.join(privateRoot, "restore"))).sort();
  if (restoreEntries.join(",") !== [
    "api.private.json",
    "final-evidence.private.json",
    "flash-command-evidence.private.json",
    "flash-monitor.classifier-input.log",
    "retained-log.private.txt",
    "websocket.private.json",
  ].join(",")) {
    throw failure("evidence_invalid", "restoration private file set is invalid");
  }
  for (const directory of ["stimulus", "restore"]) {
    for (const entry of await readdir(path.join(privateRoot, directory))) {
      await requireMode(path.join(privateRoot, directory, entry), 0o600, false);
    }
  }
}

export async function captureEmc2101ThermalFaultEvidence(
  workspaceRoot: string,
  options: Emc2101ThermalFaultEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  systemInfoValidatorProgram: string,
  faultValidatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<Emc2101ThermalFaultEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  const priorProjection = path.join(workspaceRoot, expectedPriorProjection);
  const stimulusRoot = path.join(privateRoot, "stimulus");
  const restoreRoot = path.join(privateRoot, "restore");
  const restoreProjection = path.join(privateRoot, "restore-projection.private.json");
  const intentPath = path.join(privateRoot, "thermal-fault-intent.private.json");
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || path.basename(detectorOutput) !== "detector.stdout"
    || admittedPlanSha256 !== expectedPlanSha256) {
    throw failure("evidence_invalid", "THR-001 thermal fault path contract is invalid");
  }
  await requireAbsent(projection, "thermal fault projection");
  await requireAbsent(candidate, "thermal fault candidate");
  await createPrivateRoot(privateRoot);

  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = requiredString(manifest, "source_commit", "package manifest");
  const referenceCommit = requiredString(manifest, "reference_commit", "package manifest");
  const appElfSha256 = requiredString(manifest, "app_elf_sha256", "package manifest");
  if (!/^[0-9a-f]{40}$/u.test(sourceCommit)
    || referenceCommit !== expectedReferenceCommit
    || !/^[0-9a-f]{64}$/u.test(appElfSha256)) {
    throw failure("evidence_invalid", "package identity is invalid");
  }
  const planDocument = await readFile(path.join(workspaceRoot, expectedPlan), "utf8");
  const taskDocument = await readFile(path.join(workspaceRoot, "TASKS.md"), "utf8");
  validateTaskAndPlan(taskDocument, planDocument);
  const priorProjectionDocument = await readFile(priorProjection, "utf8");
  const currentSource = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "source identity");
  const pushedSource = await childText(processPort, gitProgram, ["rev-parse", "origin/main"], "pushed identity");
  const reference = await childText(
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
  if (currentSource !== sourceCommit || pushedSource !== currentSource
    || reference !== expectedReferenceCommit || dirty !== "") {
    throw failure("evidence_invalid", "source is not the exact clean pushed package identity");
  }

  const leaseBytes = randomBytes(8);
  if (leaseBytes.every((byte) => byte === 0)) leaseBytes[7] = 1;
  const intent = {
    schema_version: "esp-thermal-fault-stimulus-intent-v1",
    board: 205,
    attempt_ordinal: 5,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    plan_path: expectedPlan,
    plan_sha256: expectedPlanSha256,
    stimulus_kind: stimulusKind,
    sample_count: 5,
    lease_hex: leaseBytes.toString("hex"),
  };
  const intentDocument = await privateJson(intentPath, intent);

  let maybePrimary: Emc2101ThermalFaultEvidenceError | undefined;
  try {
    const outcome = await processPort.run(flashMonitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest: manifestPath,
      wifiCredentials: credentialsPath,
      thermalFaultStimulusIntent: path.relative(workspaceRoot, intentPath),
      captureTimeoutSeconds: options.captureTimeoutSeconds,
      evidenceMode: "dual",
      evidenceDir: stimulusRoot,
    }));
    if (outcome.timedOut) throw failure("timeout", "thermal fault flash-monitor timed out");
    if (outcome.exitCode !== 0) {
      throw failure("hardware_blocked", "thermal fault device session was not ready");
    }
    validateMarkerSequence(
      await readFile(path.join(stimulusRoot, "flash-monitor.classifier-input.log"), "utf8"),
    );
  } catch (error) {
    maybePrimary = error instanceof Emc2101ThermalFaultEvidenceError
      ? error
      : failure("process_failed", "thermal fault flash-monitor launch failed");
  }

  let restore: SystemInfoEvidence | undefined;
  try {
    restore = await captureSystemInfoEvidence(workspaceRoot, {
      privateRoot: path.relative(workspaceRoot, restoreRoot),
      packageManifest: options.packageManifest,
      wifiCredentials: options.wifiCredentials,
      port: options.port,
      projection: path.relative(workspaceRoot, restoreProjection),
      captureTimeoutSeconds: options.captureTimeoutSeconds,
    }, processPort, flashProgram, systemInfoValidatorProgram, maybeWebSocketFactory);
    await chmod(restoreProjection, 0o600);
  } catch (error) {
    if (maybePrimary !== undefined) {
      const secondary = error instanceof SystemInfoEvidenceError
        ? recoveryFromSystemInfo(error)
        : { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
      throw maybePrimary.withRecovery({
        recovery_complete: secondary.recovery_complete,
        recovery_flash_used: true,
        secondary_recovery_failure: true,
      });
    }
    if (error instanceof SystemInfoEvidenceError) {
      throw failure(error.category, error.message, recoveryFromSystemInfo(error));
    }
    throw failure("evidence_invalid", "ordinary restoration evidence is invalid", {
      recovery_complete: false,
      recovery_flash_used: true,
      secondary_recovery_failure: true,
    });
  }
  if (maybePrimary !== undefined) {
    throw maybePrimary.withRecovery({
      recovery_complete: true,
      recovery_flash_used: true,
      secondary_recovery_failure: false,
    });
  }

  const api = object(
    JSON.parse(await readFile(path.join(restoreRoot, "api.private.json"), "utf8")),
    "restored HTTP snapshot",
  );
  const envelope = object(
    JSON.parse(await readFile(path.join(restoreRoot, "websocket.private.json"), "utf8")),
    "restored WebSocket envelope",
  );
  const websocket = object(envelope["data"], "restored WebSocket snapshot");
  validateRestoredThermalTruth(api, websocket);
  const retained = await readFile(path.join(restoreRoot, "retained-log.private.txt"), "utf8");
  if (markerLines.some((marker) => retained.includes(marker))) {
    throw failure("evidence_invalid", "thermal fault stimulus replayed after restoration", {
      recovery_complete: true,
      recovery_flash_used: true,
      secondary_recovery_failure: false,
    });
  }
  await verifyProtectedLayout(privateRoot, wrapperRoot, restoreProjection);

  const restoreProjectionDocument = await readFile(restoreProjection, "utf8");
  const evidence: Emc2101ThermalFaultEvidence = {
    schema_version: "bitaxe-emc2101-thermal-fault-evidence-v1",
    board: 205,
    attempt_ordinal: 5,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    package_manifest_sha256: sha256(manifestDocument),
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-emc2101-thermal-fault-evidence",
      request_sha256: sha256(JSON.stringify({
        manifest: sha256(manifestDocument),
        plan: expectedPlanSha256,
        prior: sha256(priorProjectionDocument),
        timeout: options.captureTimeoutSeconds,
      })),
    },
    source: {
      plan_sha256: expectedPlanSha256,
      prior_thermal_projection_sha256: sha256(priorProjectionDocument),
      restore_projection_sha256: sha256(restoreProjectionDocument),
      intent_sha256: sha256(intentDocument),
      protected_modes_valid: true,
      production_source_current: true,
    },
    stimulus: {
      kind: stimulusKind,
      injected_sample_count: 5,
      real_healthy_baseline: true,
      real_reads_during_injection: true,
      typed_invalid_outcomes: true,
      thermal_reading_invalid_fault: true,
      baseline_marker_observed: true,
      fault_marker_observed: true,
      recovery_marker_observed: true,
      marker_order_exact: true,
      intent_consumed_before_use: true,
    },
    restoration: {
      ordinary_wifi_seed: true,
      exact_package_identity: true,
      http_fresh_sample: true,
      websocket_fresh_sample: true,
      below_throttle_threshold: true,
      fault_absent: true,
      stimulus_not_replayed: true,
    },
    detector_admitted: true,
    boot_observed: true,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    cleanup_complete: restore.cleanup_complete,
    recovery_used: true,
    redaction_status: "passed",
  };
  const privateFinal = path.join(privateRoot, "final-evidence.private.json");
  await privateJson(privateFinal, evidence);
  const validation = await processPort.run(
    internalCommandSpec(faultValidatorProgram, [privateFinal], (value) => value),
  );
  if (validation.timedOut) throw failure("timeout", "thermal fault evidence validation timed out");
  if (validation.exitCode !== 0) {
    throw failure("evidence_invalid", "thermal fault evidence validation failed");
  }
  await mkdir(path.dirname(projection), { recursive: true });
  await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
  });
  await rename(candidate, projection);
  return evidence;
}

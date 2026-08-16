import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type HashrateMonitorEvidence,
  type HashrateTransportQuorum,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type HashrateMonitorEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly poolCredentials: string;
  readonly detectorOutput: string;
  readonly port: string;
  readonly projection: string;
  readonly durationSeconds: number;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<
  AutomationCategory,
  "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed"
>;
type RuntimeAttestationParseFailure = typeof runtimeAttestationParseFailures[number];
type RuntimeAttestationParseFailureCounts = Readonly<{
  missing_marker: number;
  malformed_token: number;
  duplicate_field: number;
  unknown_field: number;
  missing_field: number;
  invalid_field: number;
  incomplete_readiness: number;
}>;
type RuntimeAttestationParseDiagnostic = Readonly<{
  runtime_attestation_parse_failure: RuntimeAttestationParseFailure;
}>;
type RuntimeAttestationParseEvidence = RuntimeAttestationParseDiagnostic & Readonly<{
  runtime_attestation_parse_failure_counts: RuntimeAttestationParseFailureCounts;
}>;
type WatchdogFailure = typeof watchdogFailures[number];
type WatchdogFailureDiagnostic = Readonly<{
  watchdog_failure: WatchdogFailure;
}>;
type CampaignFailureDiagnostic = RuntimeAttestationParseDiagnostic
  & Partial<WatchdogFailureDiagnostic>;

const expectedPrivateRoot = "scratch/stat001-hashrate-monitor/attempt-006";
const expectedWrapperRoot = "scratch/stat001-hashrate-monitor/wrapper-006";
const expectedProjection =
  "docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json";
const expectedPlan = "docs/parity/work-plans/20260816T173058Z-STAT-001/PLAN.md";
const expectedPlanSha256 = "ca1957e47576f18441bceb538e51b13c7bcddbec0809044b9d24d22e5a9baeb8";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-stat001-hashrate-monitor";
const runtimeAttestationParseFailures = [
  "none",
  "not_observed",
  "invalid_utf8",
  "missing_marker",
  "malformed_token",
  "duplicate_field",
  "unknown_field",
  "missing_field",
  "invalid_field",
  "incomplete_readiness",
] as const;
const watchdogFailures = [
  "none",
  "supervisor_unavailable",
  "checkpoint_unhealthy",
  "checkpoint_sequence_missing",
  "watchdog_reason_missing",
  "watchdog_unproved",
  "watchdog_invalid_observation",
  "watchdog_subscription_failed",
  "watchdog_feed_failed",
  "watchdog_unsubscription_failed",
  "watchdog_unsubscribed",
  "watchdog_reason_unknown",
  "watchdog_participation_inconsistent",
  "watchdog_feed_sequence_missing",
  "watchdog_feed_age_missing",
  "watchdog_feed_stale",
  "http_checkpoint_not_advanced",
  "http_feed_not_advanced",
  "websocket_checkpoint_not_advanced",
  "websocket_feed_not_advanced",
] as const;
const expectedAttemptFiles = [
  "campaign-diagnostics.private.json",
  "campaign-flash.private.json",
  "campaign-mining-diagnostics.private.json",
  "campaign-network.private.json",
  "campaign-observations.private.json",
  "campaign-result.json",
  "campaign-result.sha256",
] as const;
const sourceFragments = new Map<string, readonly string[]>([
  ["crates/bitaxe-core/src/hashrate.rs", [
    "const HASHRATE_REGISTER_UNIT_HASHES: f64 = 1_048_576.0;",
    "const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;",
    "const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;",
  ]],
  ["crates/bitaxe-stratum/src/v1/state.rs", ["pub hashrate_inputs: HashrateInputs"]],
  ["crates/bitaxe-stratum/src/v1/production_session/campaign.rs", [
    "Self::Conservative => (400, 1_100, 100)",
    "core_voltage_mv: i64,",
  ]],
  ["crates/bitaxe-api/src/mining.rs", [
    "hash_rate: hashrate.current_ghs,",
    "hashrate_monitor: HashrateMonitorWire {",
  ]],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub bus_voltage_volts: Observation<f64>,",
    "let min_input_voltage = INPUT_VOLTAGE_NOMINAL_VOLTS * (1.0 - INPUT_VOLTAGE_MARGIN_RATIO);",
    "(min_input_voltage..=max_input_voltage).contains(&bus_voltage_volts)",
  ]],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "hashRate")]',
    '#[serde(rename = "hashrateMonitor")]',
  ]],
  ["firmware/bitaxe/src/production_mining_session/hashrate.rs", [
    "const HASHRATE_CADENCE_MS: u64 = 1_000;",
    "const BM1366_HASH_DOMAIN_COUNT: usize = 4;",
  ]],
  ["firmware/bitaxe/src/production_mining_session/asic_worker.rs", [
    "request_hashrate_monitor_register_reads_tx()",
    "emit(AsicWorkerEvent::RegisterRead {",
  ]],
  ["firmware/bitaxe/src/runtime_snapshot.rs", ["publish_hashrate_snapshot"]],
  ["crates/bitaxe-safety/src/power.rs", [
    "pub const INPUT_VOLTAGE_NOMINAL_VOLTS: f64 = 5.0;",
    "pub const INPUT_VOLTAGE_MARGIN_RATIO: f64 = 0.10;",
  ]],
]);
const referenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/tasks/hashrate_monitor_task.c", [
    "#define HASHRATE_UNIT 0x100000uLL",
    "#define POLL_RATE 1000",
    "#define HASHRATE_1M_SIZE (60000 / POLL_RATE)",
    "void update_hash_counter(measurement_t * measurement, uint32_t value, uint64_t time_us)",
    "ASIC_read_registers(GLOBAL_STATE);",
  ]],
  ["reference/esp-miner/components/stratum/utils.c", [
    "#define HASH_CNT_LSB 0x100000000uLL",
    "float hashCounterToGhs(uint64_t duration_us, uint32_t counter)",
  ]],
  ["reference/esp-miner/main/device_config.h", [
    ".default_voltage_mv = 1200,",
    "FAMILY_ULTRA       = { .id = ULTRA,       .name = \"Ultra\",      .asic = ASIC_BM1366,   .asic_count = 1, .max_power =  25, .power_offset = 5,  .nominal_voltage = 5,",
  ]],
  ["reference/esp-miner/main/tasks/power_management_task.c", [
    "uint16_t voltage = nvs_config_get_u16(NVS_CONFIG_ASIC_VOLTAGE);",
    "VCORE_set_voltage(GLOBAL_STATE, (double) voltage / 1000.0);",
  ]],
]);

export class HashrateMonitorEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "HashrateMonitorEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): HashrateMonitorEvidenceError {
  return new HashrateMonitorEvidenceError(category, message, {
    stage: "hashrate_monitor_capture",
    projection_published: false,
  });
}

function hardwareBlocked(
  message: string,
  maybeDiagnostic?: CampaignFailureDiagnostic,
): HashrateMonitorEvidenceError {
  return new HashrateMonitorEvidenceError("hardware_blocked", message, {
    stage: "hashrate_monitor_capture",
    projection_published: false,
    ...(maybeDiagnostic ?? {}),
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

function requiredInteger(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

function requiredBoolean(value: JsonObject, field: string, context: string): boolean {
  const candidate = value[field];
  if (typeof candidate !== "boolean") {
    throw failure("evidence_invalid", `${context} boolean field is invalid`);
  }
  return candidate;
}

function runtimeAttestationParseDiagnostic(
  value: JsonObject,
): RuntimeAttestationParseEvidence {
  const failureValue = requiredString(
    value,
    "runtime_attestation_parse_failure",
    "campaign result",
  );
  if (!runtimeAttestationParseFailures.includes(
    failureValue as RuntimeAttestationParseFailure,
  )) {
    throw failure("evidence_invalid", "campaign runtime attestation diagnostic is invalid");
  }
  const counts = object(
    value["runtime_attestation_parse_failure_counts"],
    "campaign runtime attestation parse failure counts",
  );
  return {
    runtime_attestation_parse_failure: failureValue as RuntimeAttestationParseFailure,
    runtime_attestation_parse_failure_counts: {
      missing_marker: requiredInteger(counts, "missing_marker", "campaign parse failure counts"),
      malformed_token: requiredInteger(counts, "malformed_token", "campaign parse failure counts"),
      duplicate_field: requiredInteger(counts, "duplicate_field", "campaign parse failure counts"),
      unknown_field: requiredInteger(counts, "unknown_field", "campaign parse failure counts"),
      missing_field: requiredInteger(counts, "missing_field", "campaign parse failure counts"),
      invalid_field: requiredInteger(counts, "invalid_field", "campaign parse failure counts"),
      incomplete_readiness: requiredInteger(
        counts,
        "incomplete_readiness",
        "campaign parse failure counts",
      ),
    },
  };
}

function watchdogFailure(value: JsonObject): WatchdogFailure {
  const failureValue = requiredString(value, "watchdog_failure", "campaign result");
  if (!watchdogFailures.includes(failureValue as WatchdogFailure)) {
    throw failure("evidence_invalid", "campaign watchdog diagnostic is invalid");
  }
  return failureValue as WatchdogFailure;
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof HashrateMonitorEvidenceError) throw error;
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

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  let outcome;
  try {
    outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
  } catch {
    throw failure("process_failed", `${context} launch failed`);
  }
  if (outcome.timedOut) throw failure("timeout", `${context} timed out`);
  if (outcome.exitCode !== 0) throw failure("evidence_invalid", `${context} did not pass`);
  return outcome.stdout.trim();
}

async function readJson(candidate: string, context: string): Promise<{
  readonly document: string;
  readonly value: JsonObject;
}> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof HashrateMonitorEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function sealedCampaignFailureDiagnostic(
  privateRoot: string,
): Promise<CampaignFailureDiagnostic | undefined> {
  try {
    const resultPath = path.join(privateRoot, "campaign-result.json");
    const sealPath = path.join(privateRoot, "campaign-result.sha256");
    await requireMode(privateRoot, 0o700, true);
    await requireMode(resultPath, 0o600, false);
    await requireMode(sealPath, 0o600, false);
    const result = await readJson(resultPath, "campaign result");
    const seal = (await readFile(sealPath, "utf8")).trim();
    if (seal !== sha256(result.document)
      || requiredString(result.value, "schema", "campaign result") !== "mining-campaign-result-v12"
      || requiredString(result.value, "status", "campaign result") !== "failed") {
      return undefined;
    }
    const diagnostic = runtimeAttestationParseDiagnostic(result.value);
    const terminalCategory = requiredString(result.value, "terminal_category", "campaign result");
    const watchdogDiagnostic = watchdogFailure(result.value);
    if (terminalCategory === "watchdog_unresponsive") {
      if (watchdogDiagnostic === "none") return undefined;
      return {
        runtime_attestation_parse_failure: diagnostic.runtime_attestation_parse_failure,
        watchdog_failure: watchdogDiagnostic,
      };
    }
    if (watchdogDiagnostic !== "none") return undefined;
    return {
      runtime_attestation_parse_failure: diagnostic.runtime_attestation_parse_failure,
    };
  } catch {
    return undefined;
  }
}

export async function validateHashrateMonitorTaskAndSources(
  workspaceRoot: string,
  admittedPlanSha256: string,
): Promise<void> {
  const [taskDocument, planDocument] = await Promise.all([
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
    readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
  ]);
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const block = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1
    || !block.includes(expectedPlan) || !block.includes("attempt-006")
    || sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `STAT-001`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "STAT-001 task or immutable plan binding is invalid");
  }
  for (const [relative, fragments] of referenceFragments) {
    const document = await readFile(path.join(workspaceRoot, relative), "utf8");
    for (const fragment of fragments) {
      if (document.split(fragment).length !== 2) {
        throw failure("evidence_invalid", "pinned hashrate reference semantics are invalid");
      }
    }
  }
  for (const [relative, fragments] of sourceFragments) {
    const document = await readFile(path.join(workspaceRoot, relative), "utf8");
    for (const fragment of fragments) {
      if (document.split(fragment).length !== 2) {
        throw failure("evidence_invalid", "production hashrate source semantics are invalid");
      }
    }
  }
}

function transportQuorum(value: unknown, context: string): HashrateTransportQuorum {
  const transport = object(value, context);
  return {
    active_sample_count: requiredInteger(transport, "active_sample_count", context),
    positive_coherent_count: requiredInteger(transport, "positive_coherent_count", context),
    distinct_positive_count: requiredInteger(transport, "distinct_positive_count", context),
    warm_rolling_window_count: requiredInteger(transport, "warm_rolling_window_count", context),
    terminal_zero_confirmed: requiredBoolean(transport, "terminal_zero_confirmed", context),
  };
}

async function verifyProtectedLayout(privateRoot: string, wrapperRoot: string): Promise<void> {
  await requireMode(wrapperRoot, 0o700, true);
  for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await requireMode(path.join(wrapperRoot, name), 0o600, false);
  }
  await requireMode(privateRoot, 0o700, true);
  const entries = (await readdir(privateRoot)).sort();
  if (entries.length !== expectedAttemptFiles.length
    || entries.some((entry, index) => entry !== expectedAttemptFiles[index])) {
    throw failure("evidence_invalid", "protected campaign file set is invalid");
  }
  for (const entry of entries) await requireMode(path.join(privateRoot, entry), 0o600, false);
}

export async function captureHashrateMonitorEvidence(
  workspaceRoot: string,
  options: HashrateMonitorEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  validatorProgram: string,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<HashrateMonitorEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || options.durationSeconds !== 600) {
    throw failure("evidence_invalid", "STAT-001 protected path or duration contract is invalid");
  }
  await requireAbsent(privateRoot, "protected campaign root");
  await requireAbsent(projection, "hashrate projection");
  await requireAbsent(candidate, "hashrate projection candidate");
  await validateHashrateMonitorTaskAndSources(workspaceRoot, admittedPlanSha256);

  let campaign;
  try {
    campaign = await processPort.run(internalCommandSpec(flashProgram, [
      "mining-campaign", "--stage", "live-share", "--profile", "conservative",
      "--board", "205", "--port", options.port,
      "--manifest", options.packageManifest,
      "--wifi-credentials", options.wifiCredentials,
      "--pool-credentials", options.poolCredentials,
      "--evidence-dir", options.privateRoot,
      "--duration-seconds", String(options.durationSeconds), "--redact-evidence",
    ], (value) => value), options.captureTimeoutSeconds * 1_000);
  } catch {
    throw failure("process_failed", "hashrate campaign launch failed");
  }
  if (campaign.timedOut) throw failure("timeout", "hashrate campaign timed out");
  if (campaign.exitCode !== 0) {
    throw hardwareBlocked(
      "hashrate campaign did not complete",
      await sealedCampaignFailureDiagnostic(privateRoot),
    );
  }

  try {
    await verifyProtectedLayout(privateRoot, wrapperRoot);
    const resultFile = await readJson(path.join(privateRoot, "campaign-result.json"), "campaign result");
    const networkFile = await readJson(
      path.join(privateRoot, "campaign-network.private.json"),
      "campaign network evidence",
    );
    const seal = (await readFile(path.join(privateRoot, "campaign-result.sha256"), "utf8")).trim();
    if (seal !== sha256(resultFile.document)
      || requiredString(resultFile.value, "network_continuity_sha256", "campaign result")
        !== sha256(networkFile.document)) {
      throw failure("evidence_invalid", "campaign result seal is invalid");
    }
    const parseDiagnostic = runtimeAttestationParseDiagnostic(resultFile.value);
    const resultWatchdogFailure = watchdogFailure(resultFile.value);
    if (requiredString(resultFile.value, "schema", "campaign result") !== "mining-campaign-result-v12"
      || requiredString(resultFile.value, "status", "campaign result") !== "accepted"
      || requiredString(resultFile.value, "stage", "campaign result") !== "live-share"
      || requiredString(resultFile.value, "profile", "campaign result") !== "conservative"
      || requiredInteger(resultFile.value, "duration_seconds", "campaign result") !== 600
      || requiredString(resultFile.value, "runtime_identity", "campaign result") !== "trusted"
      || requiredString(resultFile.value, "safe_stop", "campaign result") !== "confirmed"
      || requiredString(resultFile.value, "usb_cleanup", "campaign result") !== "ready"
      || parseDiagnostic.runtime_attestation_parse_failure !== "none"
      || Object.values(parseDiagnostic.runtime_attestation_parse_failure_counts)
        .some((count) => count !== 0)
      || requiredString(networkFile.value, "schema", "campaign network evidence")
        !== "mining-campaign-network-continuity-v6"
      || requiredString(networkFile.value, "status", "campaign network evidence") !== "accepted") {
      throw failure("evidence_invalid", "campaign acceptance boundary is incomplete");
    }
    if (resultWatchdogFailure !== "none"
      || watchdogFailure(networkFile.value) !== resultWatchdogFailure) {
      throw failure("evidence_invalid", "campaign watchdog acceptance boundary is invalid");
    }
    const hashrate = object(networkFile.value["hashrate_monitor"], "hashrate monitor evidence");
    const http = transportQuorum(hashrate["http"], "HTTP hashrate evidence");
    const websocket = transportQuorum(hashrate["websocket"], "WebSocket hashrate evidence");
    const manifestFile = await readJson(
      assertWithinWorkspace(workspaceRoot, options.packageManifest),
      "package manifest",
    );
    const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "source identity");
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
    if (currentSourceCommit !== pushedSourceCommit || dirty !== ""
      || requiredString(manifestFile.value, "source_commit", "package manifest") !== currentSourceCommit
      || requiredString(manifestFile.value, "reference_commit", "package manifest") !== expectedReferenceCommit
      || referenceCommit !== expectedReferenceCommit) {
      throw failure("evidence_invalid", "exact clean pushed package identity is invalid");
    }
    const evidence: HashrateMonitorEvidence = {
      schema_version: "bitaxe-hashrate-monitor-evidence-v1",
      board: 205,
      attempt_ordinal: 6,
      source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: sha256(manifestFile.document),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-hashrate-monitor-evidence",
        request_sha256: sha256(JSON.stringify({
          result: sha256(resultFile.document),
          network: sha256(networkFile.document),
          plan: admittedPlanSha256,
          duration_seconds: options.durationSeconds,
        })),
      },
      source: {
        plan_sha256: admittedPlanSha256,
        campaign_result_sha256: sha256(resultFile.document),
        campaign_network_sha256: sha256(networkFile.document),
        source_semantics_current: true,
        reference_semantics_current: true,
        source_path_count: sourceFragments.size,
      },
      hashrate: {
        monitor_cadence_ms: requiredInteger(hashrate, "monitor_cadence_ms", "hashrate monitor evidence"),
        asic_count: requiredInteger(hashrate, "asic_count", "hashrate monitor evidence"),
        domain_count: requiredInteger(hashrate, "domain_count", "hashrate monitor evidence"),
        required_window_count: requiredInteger(networkFile.value, "required_window_count", "campaign network evidence"),
        covered_window_count: requiredInteger(networkFile.value, "covered_window_count", "campaign network evidence"),
        http,
        websocket,
      },
      detector_admitted: true,
      runtime_identity: "trusted",
      campaign_profile: "conservative",
      campaign_duration_seconds: 600,
      network_status: "accepted",
      mining_state: "active_then_paused",
      safe_stop_confirmed: true,
      cleanup_complete: true,
      hardware_rerun_used: false,
      redaction_status: "passed",
    };
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await childText(processPort, validatorProgram, [candidate], "hashrate evidence validator");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof HashrateMonitorEvidenceError) throw error;
    throw failure("evidence_invalid", "hashrate evidence processing failed");
  }
}

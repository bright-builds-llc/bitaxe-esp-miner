import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type Safe10Evidence,
} from "./contracts.generated.js";
import { portFromDetectorOutput } from "./detector.js";
import type { ProcessPort } from "./process.js";
import {
  safe10AttemptProductionDigest,
  safe10CurrentInventory,
  safe10ProductionFragments,
} from "./safe10-source-inventory.js";
import { assertWithinWorkspace } from "./workspace.js";

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed" | "timeout">;

const expectedPlan = "docs/parity/work-plans/20260818T122819Z-SAFE-10/PLAN.md";
const expectedPlanSha256 = "ca4230f4668843be0d1a433b061e6dddaf9fb25b3d318094e30945ca71648690";
const expectedTask = "task-parity-safe10-prerequisite-readiness";
const expectedAttemptRoot = "scratch/stat003-scoreboard/attempt-003";
const expectedDetectorOutput = "scratch/stat003-scoreboard/wrapper-003/detector.stdout";
const expectedAttemptPlan = "docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md";
const expectedAttemptPlanSha256 = "41ca445088dcf15c4c1c46e504a754c61260e7575eb16ccf68e0edb0fc742879";
const expectedAttemptClosure = "docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md";
const expectedAttemptSourceCommit = "60a56d4935ced15eeb5ec6950b1ad4ea35fdf223";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const expectedProjection = "docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json";

export type Safe10EvidenceOptions = Readonly<{
  attemptRoot: string;
  detectorOutput: string;
  attemptPlan: string;
  attemptClosure: string;
  projection: string;
}>;

export class Safe10EvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>> = {
      stage: "safe10_projection",
      projection_published: false,
    },
  ) {
    super(message);
    this.name = "Safe10EvidenceError";
  }
}

function failure(category: FailureCategory, message: string): Safe10EvidenceError {
  return new Safe10EvidenceError(category, message);
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

async function readJson(candidate: string, context: string): Promise<{
  readonly document: string;
  readonly value: JsonObject;
}> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof Safe10EvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "SAFE-10 protected evidence mode is invalid");
  }
}

async function requirePrivateTreeModes(root: string): Promise<void> {
  await requireMode(root, 0o700, true);
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) await requirePrivateTreeModes(candidate);
    else if (entry.isFile()) await requireMode(candidate, 0o600, false);
    else throw failure("evidence_invalid", "SAFE-10 protected evidence contains a special file");
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
  if (outcome.exitCode !== 0) throw failure("evidence_invalid", `${context} failed`);
  return outcome.stdout;
}

function exactFreshness(value: JsonObject, context: string): {
  readonly power: boolean;
  readonly voltage: boolean;
  readonly current: boolean;
  readonly chip: boolean;
  readonly vr: boolean;
  readonly fan: boolean;
} {
  return {
    power: requiredBoolean(value, "power_watts", context),
    voltage: requiredBoolean(value, "bus_voltage_volts", context),
    current: requiredBoolean(value, "current_amps", context),
    chip: requiredBoolean(value, "chip_temp_celsius", context),
    vr: requiredBoolean(value, "vr_temp_celsius", context),
    fan: requiredBoolean(value, "fan_rpm", context),
  };
}

export async function projectSafe10Evidence(
  workspaceRoot: string,
  options: Safe10EvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validatorProgram: string,
): Promise<Safe10Evidence> {
  const attemptRoot = assertWithinWorkspace(workspaceRoot, options.attemptRoot);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const attemptPlan = assertWithinWorkspace(workspaceRoot, options.attemptPlan);
  const attemptClosure = assertWithinWorkspace(workspaceRoot, options.attemptClosure);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, attemptRoot) !== expectedAttemptRoot
    || path.relative(workspaceRoot, detectorOutput) !== expectedDetectorOutput
    || path.relative(workspaceRoot, attemptPlan) !== expectedAttemptPlan
    || path.relative(workspaceRoot, attemptClosure) !== expectedAttemptClosure
    || path.relative(workspaceRoot, projection) !== expectedProjection) {
    throw failure("evidence_invalid", "SAFE-10 immutable path binding is invalid");
  }
  try {
    await stat(projection);
    throw failure("evidence_invalid", "SAFE-10 projection already exists");
  } catch (error) {
    if (error instanceof Safe10EvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await requirePrivateTreeModes(attemptRoot);
  await requireMode(detectorOutput, 0o600, false);
  await portFromDetectorOutput(workspaceRoot, options.detectorOutput);

  const [taskDocument, planDocument, attemptPlanDocument, attemptClosureDocument] =
    await Promise.all([
      readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
      readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
      readFile(attemptPlan, "utf8"),
      readFile(attemptClosure, "utf8"),
    ]);
  const heading = `### ${expectedTask} |`;
  const taskStart = taskDocument.indexOf(heading);
  const taskEnd = taskDocument.indexOf("\n### ", taskStart + heading.length);
  const taskBlock = taskDocument.slice(taskStart, taskEnd === -1 ? taskDocument.length : taskEnd);
  if (taskStart === -1 || taskDocument.indexOf(heading, taskStart + heading.length) !== -1
    || !taskBlock.includes(expectedPlan)
    || sha256(planDocument) !== expectedPlanSha256
    || sha256(attemptPlanDocument) !== expectedAttemptPlanSha256
    || !attemptClosureDocument.includes(`- Plan SHA-256: \`${expectedAttemptPlanSha256}\``)
    || !attemptClosureDocument.includes(expectedAttemptSourceCommit)) {
    throw failure("evidence_invalid", "SAFE-10 task or plan lineage is invalid");
  }

  const campaignRoot = path.join(attemptRoot, "campaign");
  const [result, network, observations, diagnostics] = await Promise.all([
    readJson(path.join(campaignRoot, "campaign-result.json"), "campaign result"),
    readJson(path.join(campaignRoot, "campaign-network.private.json"), "campaign network"),
    readJson(path.join(campaignRoot, "campaign-observations.private.json"), "campaign observations"),
    readJson(path.join(campaignRoot, "campaign-diagnostics.private.json"), "campaign diagnostics"),
  ]);
  const seal = (await readFile(path.join(campaignRoot, "campaign-result.sha256"), "utf8")).trim();
  if (seal !== sha256(result.document)
    || requiredString(result.value, "network_continuity_sha256", "campaign result") !== sha256(network.document)
    || requiredString(result.value, "observations_sha256", "campaign result") !== sha256(observations.document)
    || requiredString(result.value, "diagnostics_sha256", "campaign result") !== sha256(diagnostics.document)) {
    throw failure("evidence_invalid", "SAFE-10 campaign seal or digest chain is invalid");
  }

  const freshness = exactFreshness(
    object(result.value["observation_freshness"], "observation freshness"),
    "observation freshness",
  );
  const requirements = exactFreshness(
    object(result.value["observation_requirements"], "observation requirements"),
    "observation requirements",
  );
  const terminal = object(observations.value["terminal_marker"], "terminal marker");
  const readiness = object(terminal["readiness_transition"], "readiness transition");
  const qualifiedCount = requiredInteger(result.value, "qualified_candidate_count", "campaign result");
  const prerequisites = {
    power_watts_required: requirements.power,
    bus_voltage_required: requirements.voltage,
    current_required: requirements.current,
    chip_temperature_required: requirements.chip,
    vr_temperature_required: requirements.vr,
    fan_rpm_required: requirements.fan,
    power_watts_fresh: freshness.power,
    bus_voltage_fresh: freshness.voltage,
    current_fresh: freshness.current,
    chip_temperature_fresh: freshness.chip,
    vr_temperature_fresh: freshness.vr,
    fan_rpm_fresh: freshness.fan,
    fresh_observation_count: requiredInteger(result.value, "fresh_observation_count", "campaign result"),
    safety_fresh: requiredString(result.value, "safety", "campaign result") === "fresh",
    readiness_unblocked: requiredString(readiness, "current_blocker", "readiness transition") === "none",
    session_running_primary: requiredString(readiness, "session_phase", "readiness transition") === "running_primary",
    hardware_ready: requiredString(readiness, "hardware_state", "readiness transition") === "ready",
    readiness_safety_fresh: requiredString(readiness, "safety_sample", "readiness transition") === "fresh",
    observation_epoch_advanced: requiredString(readiness, "observation_epoch", "readiness transition") === "advanced",
    pending_observation_recovered: requiredBoolean(readiness, "pending_observation_recovered", "readiness transition"),
    active_ms: requiredInteger(result.value, "active_ms", "campaign result"),
    required_window_count: requiredInteger(network.value, "required_window_count", "campaign network"),
    covered_window_count: requiredInteger(network.value, "covered_window_count", "campaign network"),
    work_renewal_valid: requiredBoolean(network.value, "work_renewal_valid", "campaign network"),
    active_state_valid: requiredBoolean(network.value, "active_state_valid", "campaign network"),
    network_safety_valid: requiredBoolean(network.value, "safety_valid", "campaign network"),
    watchdog_valid: requiredBoolean(network.value, "watchdog_valid", "campaign network"),
    qualified_candidate_observed: qualifiedCount > 0,
    accepted_submit_observed: requiredString(result.value, "submit_outcome", "campaign result") === "accepted",
    terminal_http_valid: requiredBoolean(network.value, "terminal_http_valid", "campaign network"),
    terminal_websocket_valid: requiredBoolean(network.value, "terminal_websocket_valid", "campaign network"),
    terminal_pool_persisted: requiredBoolean(network.value, "terminal_pool_persisted", "campaign network"),
    final_terminal_consumed: requiredBoolean(network.value, "final_terminal_consumed", "campaign network"),
    serial_finished_observed: requiredBoolean(network.value, "serial_finished_observed", "campaign network"),
  };
  if (requiredString(result.value, "schema", "campaign result") !== "mining-campaign-result-v16"
    || requiredString(result.value, "status", "campaign result") !== "accepted"
    || requiredString(result.value, "stage", "campaign result") !== "live-share"
    || requiredString(result.value, "profile", "campaign result") !== "conservative"
    || requiredInteger(result.value, "duration_seconds", "campaign result") !== 600
    || requiredString(result.value, "runtime_identity", "campaign result") !== "trusted"
    || requiredString(result.value, "network_status", "campaign result") !== "accepted"
    || requiredString(result.value, "safe_stop", "campaign result") !== "confirmed"
    || requiredString(result.value, "usb_cleanup", "campaign result") !== "ready"
    || requiredString(network.value, "status", "campaign network") !== "accepted"
    || requiredString(network.value, "correlation_failure", "campaign network") !== "none"
    || requiredString(network.value, "watchdog_failure", "campaign network") !== "none"
    || requiredString(diagnostics.value, "runtime_attestation_mixed_reset_reason", "campaign diagnostics") !== "none"
    || requiredString(diagnostics.value, "panic_signature", "campaign diagnostics") !== "none"
    || requiredInteger(diagnostics.value, "panic_signature_count", "campaign diagnostics") !== 0) {
    throw failure("evidence_invalid", "SAFE-10 live campaign boundary is incomplete");
  }

  const currentCommit = (await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity")).trim();
  const pushedCommit = (await childText(processPort, gitProgram, ["rev-parse", "origin/main"], "pushed source identity")).trim();
  const referenceCommit = (await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference identity",
  )).trim();
  const dirty = (await childText(
    processPort,
    gitProgram,
    ["status", "--porcelain", "--untracked-files=no"],
    "source cleanliness",
  )).trim();
  if (currentCommit !== pushedCommit || referenceCommit !== expectedReferenceCommit || dirty !== "") {
    throw failure("evidence_invalid", "SAFE-10 clean pushed source identity is invalid");
  }
  const currentInventory = await safe10CurrentInventory(workspaceRoot);
  const attemptDocuments = new Map<string, Buffer>();
  for (const relative of safe10ProductionFragments.keys()) {
    const document = await childText(
      processPort,
      gitProgram,
      ["show", `${expectedAttemptSourceCommit}:${relative}`],
      "attempt source inventory",
    );
    attemptDocuments.set(relative, Buffer.from(document));
  }
  const attemptSourceDigest = safe10AttemptProductionDigest(attemptDocuments);
  const sourceCompatible = attemptSourceDigest === currentInventory.productionDigest;
  const evidence: Safe10Evidence = {
    schema_version: "bitaxe-safe10-evidence-v1",
    board: 205,
    attempt_ordinal: 3,
    attempt_source_commit: expectedAttemptSourceCommit,
    current_source_commit: currentCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-safe10-evidence",
      request_sha256: sha256(JSON.stringify({
        plan: expectedPlanSha256,
        attempt_plan: expectedAttemptPlanSha256,
        attempt_closure: sha256(attemptClosureDocument),
        campaign_result: sha256(result.document),
        current_source: currentInventory.digest,
        attempt_source: attemptSourceDigest,
      })),
    },
    source: {
      plan_sha256: expectedPlanSha256,
      attempt_plan_sha256: expectedAttemptPlanSha256,
      attempt_closure_sha256: sha256(attemptClosureDocument),
      campaign_result_sha256: sha256(result.document),
      campaign_network_sha256: sha256(network.document),
      campaign_observations_sha256: sha256(observations.document),
      campaign_diagnostics_sha256: sha256(diagnostics.document),
      current_source_inventory_sha256: currentInventory.digest,
      attempt_source_inventory_sha256: attemptSourceDigest,
      source_semantics_current: true,
      reference_semantics_current: true,
      attempt_source_compatible: sourceCompatible,
      source_path_count: currentInventory.pathCount,
      production_path_count: safe10ProductionFragments.size,
      reference_path_count: 2,
    },
    prerequisites,
    detector_admitted: true,
    runtime_identity: "trusted",
    campaign_stage: "live-share",
    campaign_profile: "conservative",
    campaign_status: "accepted",
    network_status: "accepted",
    safe_stop_confirmed: true,
    cleanup_complete: true,
    hardware_rerun_used: false,
    protected_modes_valid: true,
    redaction_status: "passed",
  };
  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      mode: 0o600,
      flag: "wx",
    });
    await childText(processPort, validatorProgram, [candidate], "SAFE-10 evidence validator");
    await chmod(candidate, 0o644);
    await rename(candidate, projection);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof Safe10EvidenceError) throw error;
    throw failure("evidence_invalid", "SAFE-10 evidence publication failed");
  }
}

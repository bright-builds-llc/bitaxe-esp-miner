import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type Cfg07Evidence,
} from "./contracts.generated.js";
import {
  cfg07AttemptProductionDigest,
  cfg07CurrentInventory,
  cfg07ProductionFragments,
} from "./cfg07-source-inventory.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed" | "timeout">;

const expectedPlan = "docs/parity/work-plans/20260818T150603Z-CFG-07/PLAN.md";
const expectedPlanSha256 = "be92a7b345f200028e2dec08fe5476f09d98dbb27fefe3c851f66ddeef9c91f1";
const expectedTask = "task-parity-cfg07-runtime-credentials";
const expectedSafe10Projection =
  "docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json";
const expectedSafe10Sha256 = "4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e";
const expectedAttemptPlan = "docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md";
const expectedAttemptPlanSha256 = "41ca445088dcf15c4c1c46e504a754c61260e7575eb16ccf68e0edb0fc742879";
const expectedAttemptClosure = "docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md";
const expectedAttemptClosureSha256 = "350a56d6eaab1ea066f71a24d5a964a27e37d5472aca733fe912218afa87a79d";
const expectedAttemptSourceCommit = "60a56d4935ced15eeb5ec6950b1ad4ea35fdf223";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const expectedProjection =
  "docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json";

export type Cfg07EvidenceOptions = Readonly<{
  safe10Projection: string;
  attemptPlan: string;
  attemptClosure: string;
  projection: string;
}>;

export class Cfg07EvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>> = {
      stage: "cfg07_projection",
      projection_published: false,
    },
  ) {
    super(message);
    this.name = "Cfg07EvidenceError";
  }
}

function failure(category: FailureCategory, message: string): Cfg07EvidenceError {
  return new Cfg07EvidenceError(category, message);
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

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", "CFG-07 projection already exists");
  } catch (error) {
    if (error instanceof Cfg07EvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function validateSafe10(value: JsonObject, document: string): void {
  const source = object(value["source"], "SAFE-10 source");
  const prerequisites = object(value["prerequisites"], "SAFE-10 prerequisites");
  if (sha256(document) !== expectedSafe10Sha256
    || requiredString(value, "schema_version", "SAFE-10 projection") !== "bitaxe-safe10-evidence-v1"
    || requiredInteger(value, "board", "SAFE-10 projection") !== 205
    || requiredInteger(value, "attempt_ordinal", "SAFE-10 projection") !== 3
    || requiredString(value, "attempt_source_commit", "SAFE-10 projection") !== expectedAttemptSourceCommit
    || requiredString(value, "reference_commit", "SAFE-10 projection") !== expectedReferenceCommit
    || requiredString(value, "runtime_identity", "SAFE-10 projection") !== "trusted"
    || requiredString(value, "campaign_stage", "SAFE-10 projection") !== "live-share"
    || requiredString(value, "campaign_profile", "SAFE-10 projection") !== "conservative"
    || requiredString(value, "campaign_status", "SAFE-10 projection") !== "accepted"
    || requiredString(value, "network_status", "SAFE-10 projection") !== "accepted"
    || !requiredBoolean(value, "detector_admitted", "SAFE-10 projection")
    || !requiredBoolean(value, "safe_stop_confirmed", "SAFE-10 projection")
    || !requiredBoolean(value, "cleanup_complete", "SAFE-10 projection")
    || !requiredBoolean(value, "protected_modes_valid", "SAFE-10 projection")
    || requiredString(value, "redaction_status", "SAFE-10 projection") !== "passed"
    || !requiredBoolean(source, "source_semantics_current", "SAFE-10 source")
    || !requiredBoolean(source, "reference_semantics_current", "SAFE-10 source")
    || !requiredBoolean(source, "attempt_source_compatible", "SAFE-10 source")
    || !requiredBoolean(prerequisites, "accepted_submit_observed", "SAFE-10 prerequisites")
    || !requiredBoolean(prerequisites, "qualified_candidate_observed", "SAFE-10 prerequisites")
    || requiredInteger(prerequisites, "active_ms", "SAFE-10 prerequisites") < 600_000) {
    throw failure("evidence_invalid", "CFG-07 live mining source evidence is incomplete");
  }
}

export async function projectCfg07Evidence(
  workspaceRoot: string,
  options: Cfg07EvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  safe10ValidatorProgram: string,
  cfg07ValidatorProgram: string,
): Promise<Cfg07Evidence> {
  const safe10Projection = assertWithinWorkspace(workspaceRoot, options.safe10Projection);
  const attemptPlan = assertWithinWorkspace(workspaceRoot, options.attemptPlan);
  const attemptClosure = assertWithinWorkspace(workspaceRoot, options.attemptClosure);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, safe10Projection) !== expectedSafe10Projection
    || path.relative(workspaceRoot, attemptPlan) !== expectedAttemptPlan
    || path.relative(workspaceRoot, attemptClosure) !== expectedAttemptClosure
    || path.relative(workspaceRoot, projection) !== expectedProjection) {
    throw failure("evidence_invalid", "CFG-07 immutable path binding is invalid");
  }
  await requireAbsent(projection);
  await requireAbsent(candidate);

  const [taskDocument, planDocument, attemptPlanDocument, attemptClosureDocument, safe10Document] =
    await Promise.all([
      readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
      readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
      readFile(attemptPlan, "utf8"),
      readFile(attemptClosure, "utf8"),
      readFile(safe10Projection, "utf8"),
    ]);
  const heading = `### ${expectedTask} |`;
  const taskStart = taskDocument.indexOf(heading);
  const taskEnd = taskDocument.indexOf("\n### ", taskStart + heading.length);
  const taskBlock = taskDocument.slice(taskStart, taskEnd === -1 ? taskDocument.length : taskEnd);
  if (taskStart === -1 || taskDocument.indexOf(heading, taskStart + heading.length) !== -1
    || !taskBlock.includes(expectedPlan)
    || sha256(planDocument) !== expectedPlanSha256
    || sha256(attemptPlanDocument) !== expectedAttemptPlanSha256
    || sha256(attemptClosureDocument) !== expectedAttemptClosureSha256
    || !attemptClosureDocument.includes(expectedAttemptSourceCommit)
    || !attemptPlanDocument.includes("--wifi-credentials wifi-credentials.json")
    || !attemptPlanDocument.includes("--pool-credentials pool-credentials.json")) {
    throw failure("evidence_invalid", "CFG-07 task or plan lineage is invalid");
  }

  await childText(
    processPort,
    safe10ValidatorProgram,
    [safe10Projection],
    "SAFE-10 independent validation",
  );
  let safe10: JsonObject;
  try {
    safe10 = object(JSON.parse(safe10Document), "SAFE-10 projection");
  } catch (error) {
    if (error instanceof Cfg07EvidenceError) throw error;
    throw failure("evidence_invalid", "SAFE-10 projection is malformed");
  }
  validateSafe10(safe10, safe10Document);

  const currentCommit = (await childText(
    processPort,
    gitProgram,
    ["rev-parse", "HEAD"],
    "current source identity",
  )).trim();
  const pushedCommit = (await childText(
    processPort,
    gitProgram,
    ["rev-parse", "origin/main"],
    "pushed source identity",
  )).trim();
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
    throw failure("evidence_invalid", "CFG-07 clean pushed source identity is invalid");
  }

  let currentInventory;
  let attemptSourceDigest;
  try {
    currentInventory = await cfg07CurrentInventory(workspaceRoot);
    const attemptDocuments = new Map<string, Buffer>();
    for (const relative of cfg07ProductionFragments.keys()) {
      const document = await childText(
        processPort,
        gitProgram,
        ["show", `${expectedAttemptSourceCommit}:${relative}`],
        "attempt credential source inventory",
      );
      attemptDocuments.set(relative, Buffer.from(document));
    }
    attemptSourceDigest = cfg07AttemptProductionDigest(attemptDocuments);
  } catch (error) {
    if (error instanceof Cfg07EvidenceError) throw error;
    throw failure("evidence_invalid", "CFG-07 source inventory is invalid");
  }
  const sourceCompatible = attemptSourceDigest === currentInventory.productionSemanticDigest;
  const evidence: Cfg07Evidence = {
    schema_version: "bitaxe-cfg07-evidence-v1",
    board: 205,
    attempt_ordinal: 3,
    attempt_source_commit: expectedAttemptSourceCommit,
    current_source_commit: currentCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-cfg07-evidence",
      request_sha256: sha256(JSON.stringify({
        plan: expectedPlanSha256,
        attempt_plan: expectedAttemptPlanSha256,
        attempt_closure: expectedAttemptClosureSha256,
        safe10_projection: expectedSafe10Sha256,
        current_source: currentInventory.digest,
        attempt_source: attemptSourceDigest,
      })),
    },
    source: {
      plan_sha256: expectedPlanSha256,
      attempt_plan_sha256: expectedAttemptPlanSha256,
      attempt_closure_sha256: expectedAttemptClosureSha256,
      safe10_projection_sha256: expectedSafe10Sha256,
      current_source_inventory_sha256: currentInventory.digest,
      attempt_source_inventory_sha256: attemptSourceDigest,
      source_semantics_current: true,
      reference_semantics_current: true,
      attempt_source_compatible: sourceCompatible,
      source_path_count: currentInventory.pathCount,
      production_path_count: cfg07ProductionFragments.size,
      reference_path_count: 2,
    },
    credentials: {
      runtime_credentials_input: "local-owner-supplied",
      wifi_input_required: true,
      pool_input_required: true,
      inputs_forwarded_to_campaign: true,
      live_mining_credentials_consumed: true,
      accepted_submit_observed: true,
      committed_credential_values: "none",
      raw_artifacts_committed: "no",
      credential_contents_read_by_projector: false,
    },
    detector_admitted: true,
    runtime_identity: "trusted",
    campaign_stage: "live-share",
    campaign_profile: "conservative",
    campaign_status: "accepted",
    network_status: "accepted",
    safe_stop_status: "complete",
    cleanup_complete: true,
    protected_modes_valid: true,
    redaction_status: "passed",
  };
  if (!sourceCompatible) {
    throw failure("evidence_invalid", "CFG-07 attempt/current credential semantics differ");
  }

  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, undefined, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await childText(
      processPort,
      cfg07ValidatorProgram,
      [candidate],
      "CFG-07 independent validation",
    );
    await chmod(candidate, 0o644);
    await rename(candidate, projection);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    if (error instanceof Cfg07EvidenceError) throw error;
    throw failure("process_failed", "CFG-07 projection publication failed");
  }
  return evidence;
}

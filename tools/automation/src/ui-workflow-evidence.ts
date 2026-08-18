import { createHash } from "node:crypto";
import {
  chmod,
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  unlink,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type OperatorSnapshotEvidence,
} from "./contracts.generated.js";
import type { UiWorkflowEvidence } from "./ui-workflow-contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type UiWorkflowEvidenceOptions = {
  readonly privateRoot: string;
  readonly attemptSourceCommit: string;
  readonly operatorSnapshotProjection: string;
  readonly browserAttestation: string;
  readonly projection: string;
};

export type UiWorkflowValidators = {
  readonly operatorSnapshot: string;
  readonly settings: string;
  readonly log: string;
  readonly partition: string;
  readonly rollback: string;
  readonly evidence: string;
};

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;
type CapturedPackageIdentity = {
  readonly sourceCommit: string;
  readonly referenceCommit: string;
  readonly packageManifestSha256: string;
  readonly appElfSha256: string;
  readonly wwwSpiffsSha256: string;
};

const expectedAttemptSourceCommit = "bf5b74f98cdb117ca5682b0118a61743db85856f";
const priorPlan = "docs/parity/work-plans/20260813T045300Z-UI-004/PLAN.md";
const priorPlanSha256 = "ce9b94f1a3336500bbbb6adc0ab51d5c4a26f5ea44eba72928dd07b6dca42dd7";
const priorClosure = "docs/parity/work-plans/20260813T045300Z-UI-004/CLOSURE.md";
const priorClosureSha256 = "fac39b921e588e76c8f37922eb38ccde01a66b8160d0b284a059c3eb27e36b27";
const currentPlan = "docs/parity/work-plans/20260816T000806Z-UI-004/PLAN.md";
const currentPlanSha256 = "07a8b8c487ab9dfcd312f3824a902c50a766b79caee8131e1c4fd3180222f305";
const activeTask = "task-parity-ui004-projection-continuation";
const compatibilityPaths = [
  "firmware/bitaxe/static/www/index.html",
  "firmware/bitaxe/static/www/assets/app.css",
  "firmware/bitaxe/static/www/assets/ui-core.js",
  "firmware/bitaxe/static/www/assets/api-client.js",
  "firmware/bitaxe/static/www/assets/app.js",
  "firmware/bitaxe/src/static_files.rs",
  "firmware/bitaxe/src/filesystem.rs",
  "crates/bitaxe-api/src/static_plan.rs",
  "tools/automation/src/static-ui.test.ts",
  "tools/automation/src/static-provenance.test.ts",
] as const;
const expectedRoutes = ["dashboard", "network", "pool", "settings", "logs", "update", "theme"] as const;
const expectedBrowserArtifactKinds = [
  ...expectedRoutes.map((route) => `desktop-${route}`),
  ...expectedRoutes.map((route) => `mobile-${route}`),
  "mobile-navigation-open",
  "mobile-navigation-closed",
  "write-only-secrets",
  "update-guard",
  "console",
  "network",
] as const;

const joinedSources = [
  {
    label: "theme",
    path: "docs/parity/evidence/api010-theme-durability/theme-durability-projection.json",
    digest: "fbf93cd115e1c99cd1c727b2e4536c49f571b69172fc94228d4942181d005288",
    schema: "bitaxe-theme-durability-evidence-v1",
  },
  {
    label: "settings",
    path: "docs/parity/evidence/api003-settings-patch/settings-patch-projection.json",
    digest: "6cad3810a4f0f5573c055141ff1ede6c6c629092816bd5d2147c6130a5f8c2d8",
    schema: "bitaxe-settings-patch-evidence-v1",
  },
  {
    label: "log",
    path: "docs/parity/evidence/log001-retained-stream/log-buffer-projection.json",
    digest: "a72f2a89acdfeb71e9e172b553da5875d080877f16e5879faa9da9f2dbcbc62f",
    schema: "bitaxe-log-buffer-evidence-v1",
  },
  {
    label: "partition",
    path: "docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json",
    digest: "a9c79eecfc8ad75859d676d7e4b6ea0a6047be6710a808f0bab98ab752ccb10a",
    schema: "bitaxe-partition-layout-evidence-v1",
  },
  {
    label: "rollback",
    path: "docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json",
    digest: "2c4387346d91ae4f265c149ab32b66ffa032cfa641f82ae6772f9b8ce0533c0d",
    schema: "bitaxe-sdkconfig-rollback-evidence-v1",
  },
] as const;

const implementationResult = "docs/parity/work-plans/20260804T190000Z-UI-004/RESULT.md";
const implementationResultSha256 = "efefe5870656dba5b6d6eecf507826ff7bff7bb6ede5c3fdc21b926148fa668e";
const staticUiContract = "tools/automation/src/static-ui.test.ts";
const staticUiContractSha256 = "14e7afb073a39ef4e4cf2b4ea242e0b9e2d87b89d5218c9996ba2155faa520b9";

export class UiWorkflowEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "UiWorkflowEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): UiWorkflowEvidenceError {
  return new UiWorkflowEvidenceError(category, message, {
    stage: "ui_workflow_projection",
    projection_published: false,
    hardware_rerun_used: false,
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
    throw failure("evidence_invalid", `${context} ${field} is invalid`);
  }
  return candidate;
}

function trueField(value: JsonObject, field: string, context: string): void {
  if (value[field] !== true) throw failure("evidence_invalid", `${context} ${field} is incomplete`);
}

function parseJson(document: string, context: string): JsonObject {
  try {
    return object(JSON.parse(document), context);
  } catch (error) {
    if (error instanceof UiWorkflowEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function child(
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
    if (error instanceof UiWorkflowEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof UiWorkflowEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function assertProtectedTree(root: string): Promise<void> {
  const rootStat = await stat(root);
  if (!rootStat.isDirectory() || (rootStat.mode & 0o777) !== 0o700) {
    throw failure("evidence_invalid", "private root mode is invalid");
  }
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const childPath = path.join(root, entry.name);
    const childStat = await stat(childPath);
    if (entry.isDirectory()) {
      if ((childStat.mode & 0o777) !== 0o700) {
        throw failure("evidence_invalid", "private directory mode is invalid");
      }
      await assertProtectedTree(childPath);
    } else if (!entry.isFile() || (childStat.mode & 0o777) !== 0o600) {
      throw failure("evidence_invalid", "private file mode is invalid");
    }
  }
}

function validateOperatorSnapshot(
  value: JsonObject,
  packageIdentity: CapturedPackageIdentity,
): OperatorSnapshotEvidence {
  if (value["schema_version"] !== "bitaxe-operator-snapshot-evidence-v1"
    || value["board"] !== 205
    || value["source_commit"] !== packageIdentity.sourceCommit
    || value["reference_commit"] !== packageIdentity.referenceCommit
    || value["package_manifest_sha256"] !== packageIdentity.packageManifestSha256
    || value["mining_state"] !== "disabled"
    || value["hardware_control_state"] !== "disabled"
    || value["cleanup_complete"] !== true
    || value["redaction_status"] !== "passed") {
    throw failure("evidence_invalid", "operator snapshot evidence does not match the exact package");
  }
  const restart = object(value["restart_session"], "operator snapshot restart session");
  if (restart["terminal_category"] !== "ready"
    || restart["request_attempt_count"] !== 1
    || restart["software_reset_observed"] !== true
    || restart["cleanup_complete"] !== true) {
    throw failure("evidence_invalid", "operator snapshot restart evidence is incomplete");
  }
  return value as OperatorSnapshotEvidence;
}

function capturedPackageIdentity(
  operator: JsonObject,
  browser: JsonObject,
  attemptSourceCommit: string,
): CapturedPackageIdentity {
  if (attemptSourceCommit !== expectedAttemptSourceCommit
    || string(operator, "source_commit", "operator snapshot") !== attemptSourceCommit
    || string(browser, "source_commit", "browser attestation") !== attemptSourceCommit) {
    throw failure("evidence_invalid", "captured source identity is invalid");
  }
  const operatorReference = string(operator, "reference_commit", "operator snapshot");
  if (string(browser, "reference_commit", "browser attestation") !== operatorReference) {
    throw failure("evidence_invalid", "captured reference identity is invalid");
  }
  const identity = {
    sourceCommit: attemptSourceCommit,
    referenceCommit: operatorReference,
    packageManifestSha256: string(operator, "package_manifest_sha256", "operator snapshot"),
    appElfSha256: string(browser, "app_elf_sha256", "browser attestation"),
    wwwSpiffsSha256: string(browser, "www_spiffs_sha256", "browser attestation"),
  };
  for (const digest of [
    identity.packageManifestSha256,
    identity.appElfSha256,
    identity.wwwSpiffsSha256,
  ]) {
    if (!/^[0-9a-f]{64}$/u.test(digest)) {
      throw failure("evidence_invalid", "captured package digest is invalid");
    }
  }
  return identity;
}

function validateBrowserAttestation(
  value: JsonObject,
  packageIdentity: CapturedPackageIdentity,
): UiWorkflowEvidence["browser"] {
  if (value["schema_version"] !== "bitaxe-ui-browser-attestation-v1"
    || value["source_commit"] !== packageIdentity.sourceCommit
    || value["reference_commit"] !== packageIdentity.referenceCommit
    || value["app_elf_sha256"] !== packageIdentity.appElfSha256
    || value["www_spiffs_sha256"] !== packageIdentity.wwwSpiffsSha256) {
    throw failure("evidence_invalid", "browser attestation identity is invalid");
  }
  const routes = value["routes"];
  if (!Array.isArray(routes) || routes.length !== expectedRoutes.length
    || expectedRoutes.some((route, index) => routes[index] !== route)) {
    throw failure("evidence_invalid", "browser route attestation is invalid");
  }
  const browser = object(value["browser"], "browser attestation");
  const counts = ["desktop_route_count", "mobile_route_count", "console_error_count", "unexpected_request_failure_count"];
  if (counts.some((field) => typeof browser[field] !== "number" || !Number.isSafeInteger(browser[field]))) {
    throw failure("evidence_invalid", "browser attestation counts are invalid");
  }
  for (const field of [
    "same_origin_requests_observed", "log_websocket_observed", "mobile_navigation_opened",
    "mobile_navigation_closed", "write_only_secrets_blank", "no_file_update_disabled",
    "otawww_unavailable", "desktop_viewport_observed", "mobile_viewport_observed",
    "browser_cleanup_complete",
  ]) trueField(browser, field, "browser attestation");
  const projection: UiWorkflowEvidence["browser"] = {
    expected_route_count: expectedRoutes.length,
    desktop_route_count: Number(browser["desktop_route_count"]),
    mobile_route_count: Number(browser["mobile_route_count"]),
    same_origin_requests_observed: true,
    log_websocket_observed: true,
    mobile_navigation_opened: true,
    mobile_navigation_closed: true,
    write_only_secrets_blank: true,
    no_file_update_disabled: true,
    otawww_unavailable: true,
    console_error_count: Number(browser["console_error_count"]),
    unexpected_request_failure_count: Number(browser["unexpected_request_failure_count"]),
    desktop_viewport_observed: true,
    mobile_viewport_observed: true,
    browser_cleanup_complete: true,
  };
  if (projection.desktop_route_count !== expectedRoutes.length
    || projection.mobile_route_count !== expectedRoutes.length
    || projection.console_error_count !== 0
    || projection.unexpected_request_failure_count !== 0) {
    throw failure("evidence_invalid", "browser attestation quorum is incomplete");
  }
  return projection;
}

async function validateBrowserArtifacts(value: JsonObject, browserRoot: string): Promise<void> {
  const artifacts = value["artifacts"];
  if (!Array.isArray(artifacts) || artifacts.length !== expectedBrowserArtifactKinds.length) {
    throw failure("evidence_invalid", "browser artifact manifest is incomplete");
  }
  const observed = new Set<string>();
  for (const artifact of artifacts) {
    const entry = object(artifact, "browser artifact");
    const kind = string(entry, "kind", "browser artifact");
    const relativePath = string(entry, "relative_path", "browser artifact");
    const expectedDigest = string(entry, "sha256", "browser artifact");
    if (!expectedBrowserArtifactKinds.includes(kind as typeof expectedBrowserArtifactKinds[number])
      || observed.has(kind)
      || path.isAbsolute(relativePath)
      || path.normalize(relativePath) !== relativePath
      || relativePath.startsWith("..")) {
      throw failure("evidence_invalid", "browser artifact identity is invalid");
    }
    const artifactPath = assertWithinWorkspace(browserRoot, relativePath);
    let artifactValid = false;
    try {
      const artifactStat = await stat(artifactPath);
      artifactValid = artifactStat.isFile()
        && (artifactStat.mode & 0o777) === 0o600
        && sha256(await readFile(artifactPath)) === expectedDigest;
    } catch {
      artifactValid = false;
    }
    if (!artifactValid) {
      throw failure("evidence_invalid", "browser artifact is invalid");
    }
    observed.add(kind);
  }
  if (expectedBrowserArtifactKinds.some((kind) => !observed.has(kind))) {
    throw failure("evidence_invalid", "browser artifact quorum is incomplete");
  }
}

function validateTheme(value: JsonObject): void {
  if (value["schema_version"] !== "bitaxe-theme-durability-evidence-v1"
    || value["board"] !== 205
    || value["theme_get_observed"] !== true
    || value["theme_post_readback"] !== true
    || value["normal_restart_observed"] !== true
    || value["post_restart_persistence"] !== true
    || value["restoration_complete"] !== true
    || value["cleanup_complete"] !== true
    || value["redaction_status"] !== "passed") {
    throw failure("evidence_invalid", "theme source evidence is incomplete");
  }
}

function validatePlanAndTask(
  priorPlanDocument: string,
  priorClosureDocument: string,
  currentPlanDocument: string,
  task: string,
): void {
  if (sha256(priorPlanDocument) !== priorPlanSha256
    || sha256(priorClosureDocument) !== priorClosureSha256
    || sha256(currentPlanDocument) !== currentPlanSha256
    || !currentPlanDocument.includes("- Parity row: `UI-004`")
    || !currentPlanDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "immutable UI workflow plan lineage is invalid");
  }
  const heading = `### ${activeTask} |`;
  const start = task.indexOf(heading);
  if (start === -1 || task.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "active UI workflow task is invalid");
  }
  const maybeEnd = task.indexOf("\n### ", start + heading.length);
  const block = task.slice(start, maybeEnd === -1 ? task.length : maybeEnd);
  if (!block.includes(`Plan: \`${currentPlan}\``)
    || !block.includes("bitaxe-ui-workflow-evidence-v1")
    || !block.includes("umask 077")
    || !block.includes("Starting the projector consumes the transaction")
    || !block.includes("without another transaction")) {
    throw failure("evidence_invalid", "active UI workflow task is incomplete");
  }
}

async function compatibilitySourceSet(workspaceRoot: string): Promise<string> {
  const entries = await Promise.all(compatibilityPaths.map(async (relativePath) => ({
    path: relativePath,
    sha256: sha256(await readFile(path.join(workspaceRoot, relativePath))),
  })));
  return sha256(JSON.stringify(entries));
}

export async function projectUiWorkflowEvidence(
  workspaceRoot: string,
  options: UiWorkflowEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validators: UiWorkflowValidators,
): Promise<UiWorkflowEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const operatorSnapshotProjection = assertWithinWorkspace(workspaceRoot, options.operatorSnapshotProjection);
  const browserAttestation = assertWithinWorkspace(workspaceRoot, options.browserAttestation);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  await requireAbsent(projection, "UI workflow projection");
  await requireAbsent(candidate, "UI workflow candidate");
  await assertProtectedTree(privateRoot);
  await assertProtectedTree(path.dirname(operatorSnapshotProjection));
  await assertProtectedTree(path.dirname(browserAttestation));

  const [
    operatorDocument,
    browserDocument,
    priorPlanDocument,
    priorClosureDocument,
    currentPlanDocument,
    task,
  ] = await Promise.all([
    readFile(operatorSnapshotProjection, "utf8"),
    readFile(browserAttestation, "utf8"),
    readFile(path.join(workspaceRoot, priorPlan), "utf8"),
    readFile(path.join(workspaceRoot, priorClosure), "utf8"),
    readFile(path.join(workspaceRoot, currentPlan), "utf8"),
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
  ]);
  validatePlanAndTask(
    priorPlanDocument,
    priorClosureDocument,
    currentPlanDocument,
    task,
  );
  const operatorValue = parseJson(operatorDocument, "operator snapshot evidence");
  const browserValue = parseJson(browserDocument, "browser attestation");
  const packageIdentity = capturedPackageIdentity(
    operatorValue,
    browserValue,
    options.attemptSourceCommit,
  );
  await child(processPort, validators.operatorSnapshot, [operatorSnapshotProjection], "operator snapshot validation");
  validateOperatorSnapshot(operatorValue, packageIdentity);
  const browser = validateBrowserAttestation(browserValue, packageIdentity);
  await validateBrowserArtifacts(browserValue, path.dirname(browserAttestation));

  const joinedDocuments = new Map<string, string>();
  for (const source of joinedSources) {
    const sourcePath = path.join(workspaceRoot, source.path);
    const document = await readFile(sourcePath, "utf8");
    if (sha256(document) !== source.digest) {
      throw failure("evidence_invalid", `${source.label} source digest is invalid`);
    }
    const value = parseJson(document, `${source.label} source evidence`);
    if (value["schema_version"] !== source.schema || value["board"] !== 205
      || value["reference_commit"] !== packageIdentity.referenceCommit
      || value["redaction_status"] !== "passed") {
      throw failure("evidence_invalid", `${source.label} source evidence is invalid`);
    }
    joinedDocuments.set(source.label, document);
    if (source.label === "theme") validateTheme(value);
    const maybeValidator = validators[source.label as keyof UiWorkflowValidators];
    if (source.label !== "theme" && maybeValidator !== undefined) {
      await child(processPort, maybeValidator, [sourcePath], `${source.label} source validation`);
    }
    const sourceCommit = string(value, "source_commit", `${source.label} source evidence`);
    await child(processPort, gitProgram,
      ["merge-base", "--is-ancestor", sourceCommit, options.attemptSourceCommit],
      `${source.label} source ancestry`);
  }

  for (const [sourcePath, expectedDigest] of [
    [implementationResult, implementationResultSha256],
    [staticUiContract, staticUiContractSha256],
  ] as const) {
    if (sha256(await readFile(path.join(workspaceRoot, sourcePath), "utf8")) !== expectedDigest) {
      throw failure("evidence_invalid", "UI implementation source evidence drifted");
    }
  }

  const [projectorSourceCommit, reference, dirty] = await Promise.all([
    child(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    child(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
    child(processPort, gitProgram, ["status", "--porcelain"], "workspace cleanliness"),
  ]);
  if (!/^[0-9a-f]{40}$/u.test(projectorSourceCommit)
    || reference !== packageIdentity.referenceCommit
    || dirty !== "") {
    throw failure("evidence_invalid", "current projector source is not clean and exact");
  }
  await child(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, projectorSourceCommit],
    "captured source ancestry");
  await child(processPort, gitProgram,
    ["diff", "--quiet", options.attemptSourceCommit, projectorSourceCommit, "--", ...compatibilityPaths],
    "UI compatibility source paths");
  const compatibilityDirty = await child(processPort, gitProgram,
    ["status", "--porcelain", "--", ...compatibilityPaths],
    "UI compatibility source cleanliness");
  if (compatibilityDirty !== "") {
    throw failure("evidence_invalid", "UI compatibility source paths are dirty");
  }
  const compatibilitySourceSetSha256 = await compatibilitySourceSet(workspaceRoot);

  const requestSha256 = sha256(JSON.stringify({
    command: "project-ui-workflow-evidence",
    attempt_source_commit: options.attemptSourceCommit,
    projector_source_commit: projectorSourceCommit,
    package_manifest_sha256: packageIdentity.packageManifestSha256,
    operator_snapshot_evidence_sha256: sha256(operatorDocument),
    browser_attestation_sha256: sha256(browserDocument),
    prior_plan_sha256: priorPlanSha256,
    prior_closure_sha256: priorClosureSha256,
    current_plan_sha256: currentPlanSha256,
    compatibility_source_set_sha256: compatibilitySourceSetSha256,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: UiWorkflowEvidence = {
    schema_version: "bitaxe-ui-workflow-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    projector_source_commit: projectorSourceCommit,
    reference_commit: reference,
    package_manifest_sha256: packageIdentity.packageManifestSha256,
    app_elf_sha256: packageIdentity.appElfSha256,
    www_spiffs_sha256: packageIdentity.wwwSpiffsSha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-ui-workflow-evidence",
      request_sha256: requestSha256,
    },
    sources: {
      operator_snapshot_evidence_sha256: sha256(operatorDocument),
      browser_attestation_sha256: sha256(browserDocument),
      theme_evidence_sha256: sha256(joinedDocuments.get("theme") ?? ""),
      settings_evidence_sha256: sha256(joinedDocuments.get("settings") ?? ""),
      log_evidence_sha256: sha256(joinedDocuments.get("log") ?? ""),
      partition_evidence_sha256: sha256(joinedDocuments.get("partition") ?? ""),
      rollback_evidence_sha256: sha256(joinedDocuments.get("rollback") ?? ""),
      implementation_result_sha256: implementationResultSha256,
      static_ui_contract_sha256: staticUiContractSha256,
      prior_plan_sha256: priorPlanSha256,
      prior_closure_sha256: priorClosureSha256,
      current_plan_sha256: currentPlanSha256,
      compatibility_source_set_sha256: compatibilitySourceSetSha256,
      compatibility_path_count: compatibilityPaths.length,
      all_source_evidence_valid: true,
      joined_source_commits_ancestral: true,
      attempt_source_ancestral: true,
      compatibility_paths_unchanged: true,
      compatibility_paths_clean: true,
    },
    browser,
    exact_package_observed: true,
    normal_restart_observed: true,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    device_cleanup_complete: true,
    private_modes_valid: true,
    hardware_rerun_used: false,
    redaction_status: "passed",
  };

  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await chmod(candidate, 0o600);
    await child(processPort, validators.evidence, [candidate], "independent UI workflow validation");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    throw error;
  }
  return evidence;
}

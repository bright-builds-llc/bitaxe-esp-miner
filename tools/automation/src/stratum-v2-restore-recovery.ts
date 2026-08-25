import { chmod, mkdir, readFile, rename, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  normalizePackageCandidate,
  searchExactPackage,
  type PackageCandidate,
  type PackageSearchResult,
} from "./stratum-v2-restore-artifacts.js";
import {
  parseInstalledIdentity,
  projectRestoreReadiness,
  restoreBundleSchema,
  sha256,
  type RestoreBundle,
  type RestoreReadinessProjection,
} from "./stratum-v2-restore-model.js";
import {
  rebuildInstalledPackage,
  type RebuildResult,
} from "./stratum-v2-restore-rebuild.js";
import { captureRestoreSnapshot } from "./stratum-v2-restore-snapshot.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import {
  runValidatorChild,
  validateValidatorChildReceipt,
} from "./stratum-v2-validator-child.js";
import {
  fetchRuntimeObject,
  monitorRuntimeOrigin,
  type RuntimeMonitorDiagnostics,
} from "./stratum-v2-runtime-admission.js";

export type RestoreRecoveryArgs = {
  readonly board: "205";
  readonly port: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly redactEvidence: true;
};

const taskId = "task-parity-str005-autonomous-continuation";
const planRelative =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const expectedPrivateRoot = "scratch/str005-installed-package-recovery/recovery-004";
const expectedProjection =
  "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-004.json";

export class RestoreRecoveryError extends Error {
  public constructor(
    public readonly category: "invalid_invocation" | "evidence_invalid" | "hardware_blocked" | "process_failed",
    public readonly checkpoint: string,
  ) {
    super("restore recovery failed");
    this.name = "RestoreRecoveryError";
  }
}

function fail(category: RestoreRecoveryError["category"], checkpoint: string): never {
  throw new RestoreRecoveryError(category, checkpoint);
}

export function parseRestoreRecoveryArgs(values: readonly string[]): RestoreRecoveryArgs {
  const options = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (options.has(key)) fail("invalid_invocation", "invocation");
      options.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || value === undefined || !key.startsWith("--") || value.startsWith("--")
      || options.has(key)) {
      fail("invalid_invocation", "invocation");
    }
    options.set(key, value);
    index += 1;
  }
  const allowed = new Set([
    "--board", "--port", "--private-root", "--projection", "--redact-evidence",
  ]);
  if ([...options.keys()].some(key => !allowed.has(key))) fail("invalid_invocation", "invocation");
  const required = (key: string): string => {
    const maybeValue = options.get(key);
    if (typeof maybeValue !== "string" || maybeValue.length === 0) {
      fail("invalid_invocation", "invocation");
    }
    return maybeValue;
  };
  const board = required("--board");
  if (board !== "205" || options.get("--redact-evidence") !== true) {
    fail("invalid_invocation", "invocation");
  }
  return {
    board,
    port: required("--port"),
    privateRoot: required("--private-root"),
    projection: required("--projection"),
    redactEvidence: true,
  };
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await stat(candidate);
    fail("evidence_invalid", "outputs_absent");
  } catch (error) {
    if (error instanceof RestoreRecoveryError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") fail("evidence_invalid", "outputs_absent");
  }
}

async function privateJson(candidate: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(candidate, document, { mode: 0o600, flag: "wx" });
  await chmod(candidate, 0o600);
  return document;
}

async function gitText(workspace: string, args: readonly string[]): Promise<string> {
  const outcome = await runCampaignProcess(workspace, "git", args, 10_000);
  if (outcome.exitCode !== 0) fail("evidence_invalid", "source_identity");
  return outcome.stdout.trim();
}

async function admitSource(workspace: string, args: RestoreRecoveryArgs): Promise<{
  readonly sourceCommit: string;
  readonly planSha256: string;
}> {
  const status = await gitText(workspace, ["status", "--porcelain"]);
  const sync = await gitText(workspace, ["rev-list", "--left-right", "--count", "HEAD...@{u}"]);
  const sourceCommit = await gitText(workspace, ["rev-parse", "HEAD"]);
  if (status !== "" || sync !== "0\t0" || !/^[0-9a-f]{40}$/u.test(sourceCommit)) {
    fail("evidence_invalid", "source_identity");
  }
  const plan = await readFile(path.join(workspace, planRelative), "utf8");
  const tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  if (!plan.includes(`- Active task: \`${taskId}\``)
    || !tasks.includes(`### ${taskId}`)
    || !tasks.includes(planRelative)) {
    fail("evidence_invalid", "plan_binding");
  }
  const ignored = await runCampaignProcess(
    workspace,
    "git",
    ["check-ignore", "-q", args.privateRoot],
    5_000,
  );
  if (ignored.exitCode !== 0) fail("evidence_invalid", "private_root_ignored");
  return { sourceCommit, planSha256: sha256(plan) };
}

function packageBundle(
  privateRoot: string,
  candidate: PackageCandidate,
  identity: ReturnType<typeof parseInstalledIdentity>,
  sourceCommit: string,
  planSha256: string,
): RestoreBundle {
  const manifestRelative = path.relative(privateRoot, candidate.manifestPath);
  if (manifestRelative.startsWith("..") || path.isAbsolute(manifestRelative)) {
    fail("evidence_invalid", "package_containment");
  }
  return {
    schema_version: restoreBundleSchema,
    kind: "package_v3",
    board: 205,
    installed_identity: identity,
    package_manifest: manifestRelative,
    package_manifest_sha256: sha256(candidate.manifestDocument),
    factory_sha256: candidate.factorySha256,
    capture_source_commit: sourceCommit,
    plan_sha256: planSha256,
  };
}

async function runtimeIdentity(
  workspace: string,
  port: string,
  diagnostics: RuntimeMonitorDiagnostics,
): Promise<ReturnType<typeof parseInstalledIdentity>> {
  const runtimeFail = (
    category: string,
    _message: string,
    checkpoint: string,
  ): never => {
    fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", checkpoint);
  };
  const origin = await monitorRuntimeOrigin(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    port,
    runCampaignProcess,
    runtimeFail,
    diagnostics,
  );
  try {
    return parseInstalledIdentity(
      await fetchRuntimeObject(origin, "/api/system/info", runtimeFail),
    );
  } catch (error) {
    if (error instanceof RestoreRecoveryError) throw error;
    fail("evidence_invalid", "installed_identity");
  }
}

export async function recoverInstalledFirmware(
  workspace: string,
  args: RestoreRecoveryArgs,
  validatorProgram: string,
): Promise<RestoreReadinessProjection> {
  if (args.board !== "205"
    || args.privateRoot !== expectedPrivateRoot
    || args.projection !== expectedProjection
    || !args.redactEvidence
    || args.port.length === 0) {
    fail("invalid_invocation", "invocation");
  }
  const privateRoot = path.resolve(workspace, args.privateRoot);
  const projection = path.resolve(workspace, args.projection);
  await requireAbsent(privateRoot);
  await requireAbsent(projection);
  const source = await admitSource(workspace, args);
  await mkdir(path.dirname(privateRoot), { recursive: true, mode: 0o700 });
  await chmod(path.dirname(privateRoot), 0o700);
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const identity = await runtimeIdentity(workspace, args.port, {
    receiptPath: path.join(privateRoot, "runtime-monitor-initial-receipt.private.json"),
    sourceCommit: source.sourceCommit,
    planSha256: source.planSha256,
  });
  await privateJson(path.join(privateRoot, "installed-identity.private.json"), identity);
  let search: PackageSearchResult;
  try {
    search = await searchExactPackage(
      [
        path.join(workspace, "scratch"),
        path.join(workspace, "bazel-bin"),
        path.join(workspace, "bazel-out"),
      ],
      identity,
    );
  } catch {
    fail("evidence_invalid", "artifact_search");
  }
  let maybeCandidate: PackageCandidate | undefined;
  if (search.maybeCandidate !== undefined) {
    try {
      maybeCandidate = await normalizePackageCandidate(
        search.maybeCandidate,
        path.join(privateRoot, "recovered-package-search"),
      );
    } catch {
      maybeCandidate = undefined;
    }
  }
  let rebuildAttempted = false;
  if (maybeCandidate === undefined) {
    let rebuilt: RebuildResult;
    try {
      rebuilt = await rebuildInstalledPackage(workspace, privateRoot, identity);
    } catch {
      fail("process_failed", "rebuild_cleanup");
    }
    rebuildAttempted = rebuilt.attempted;
    maybeCandidate = rebuilt.maybeCandidate;
  }
  let bundle: RestoreBundle;
  if (maybeCandidate === undefined) {
    try {
      bundle = await captureRestoreSnapshot(
        workspace,
        args.port,
        privateRoot,
        identity,
        source.sourceCommit,
        source.planSha256,
      );
    } catch {
      fail("hardware_blocked", "snapshot_capture");
    }
  } else {
    bundle = packageBundle(
      privateRoot,
      maybeCandidate,
      identity,
      source.sourceCommit,
      source.planSha256,
    );
  }
  const unchanged = await runtimeIdentity(workspace, args.port, {
    receiptPath: path.join(privateRoot, "runtime-monitor-final-receipt.private.json"),
    sourceCommit: source.sourceCommit,
    planSha256: source.planSha256,
  });
  if (JSON.stringify(unchanged) !== JSON.stringify(identity)) {
    fail("hardware_blocked", "runtime_changed");
  }
  const bundlePath = path.join(privateRoot, "restore-bundle.private.json");
  const bundleDocument = await privateJson(bundlePath, bundle);
  const projectionValue = projectRestoreReadiness(
    bundle,
    bundleDocument,
    search.inspectedCount,
    rebuildAttempted,
  );
  await mkdir(path.dirname(projection), { recursive: true });
  const candidateProjection = `${projection}.candidate`;
  await writeFile(candidateProjection, `${JSON.stringify(projectionValue, null, 2)}\n`, {
    mode: 0o600,
    flag: "wx",
  });
  await chmod(candidateProjection, 0o600);
  const receiptPath = path.join(privateRoot, "validator-child-receipt.private.json");
  const validation = await runValidatorChild({
    workspace,
    program: validatorProgram,
    args: [bundlePath, candidateProjection, source.sourceCommit, source.planSha256],
    receiptPath,
    sourceCommit: source.sourceCommit,
    planSha256: source.planSha256,
  });
  await validateValidatorChildReceipt(receiptPath, source.sourceCommit, source.planSha256);
  if (!validation.validation_accepted) fail("evidence_invalid", "independent_validation");
  await rename(candidateProjection, projection);
  return projectionValue;
}

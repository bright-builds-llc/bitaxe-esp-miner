import { readFile } from "node:fs/promises";
import path from "node:path";

import { sha256, type RestoreBundle } from "./stratum-v2-restore-model.js";
import { validateRestoreReadiness } from "./stratum-v2-restore-validator.js";
import { validateValidatorChildReceipt } from "./stratum-v2-validator-child.js";

const restoreProjection =
  "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json";
const restorePlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";

type RunProcess = (
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
) => Promise<{ readonly exitCode: number; readonly stdout: string }>;

const postRecoveryHostOnlyFiles = new Set([
  "tools/automation/src/stratum-v2-campaign-preflight.ts",
  "tools/automation/src/stratum-v2-campaign.test.ts",
]);

export async function validateRecoverySourceLineage(
  workspace: string,
  captureSource: string,
  currentSource: string,
  runProcess: RunProcess,
): Promise<void> {
  if (captureSource === currentSource) return;
  const ancestry = await runProcess(
    workspace,
    "git",
    ["merge-base", "--is-ancestor", captureSource, currentSource],
    5_000,
  );
  const changed = await runProcess(
    workspace,
    "git",
    ["diff", "--name-only", `${captureSource}..${currentSource}`],
    5_000,
  );
  const files = changed.stdout.split(/\r?\n/u).filter(value => value.length > 0);
  if (ancestry.exitCode !== 0
    || changed.exitCode !== 0
    || files.length !== postRecoveryHostOnlyFiles.size
    || files.some(value => !postRecoveryHostOnlyFiles.has(value))) {
    throw new Error("restore recovery source lineage is invalid");
  }
}

export async function admitStratumV2RestoreBundle(
  workspace: string,
  restoreBundle: string,
  runProcess: RunProcess,
): Promise<{ readonly bundle: RestoreBundle; readonly path: string }> {
  const bundlePath = path.resolve(workspace, restoreBundle);
  const projectionPath = path.resolve(workspace, restoreProjection);
  const source = (
    await runProcess(workspace, "git", ["rev-parse", "HEAD"], 5_000)
  ).stdout.trim();
  const planDocument = await readFile(path.join(workspace, restorePlan), "utf8");
  const planSha256 = sha256(planDocument);
  const bundle = JSON.parse(await readFile(bundlePath, "utf8")) as RestoreBundle;
  await validateRecoverySourceLineage(
    workspace,
    bundle.capture_source_commit,
    source,
    runProcess,
  );
  await validateRestoreReadiness(
    bundlePath,
    projectionPath,
    bundle.capture_source_commit,
    planSha256,
  );
  await validateValidatorChildReceipt(
    path.join(path.dirname(bundlePath), "validator-child-receipt.private.json"),
    bundle.capture_source_commit,
    planSha256,
  );
  return {
    bundle,
    path: bundlePath,
  };
}

export function restoreRuntimeMatches(
  bundle: RestoreBundle,
  runtime: Record<string, unknown>,
): boolean {
  const identity = bundle.installed_identity;
  return runtime["sourceCommit"] === identity.source_commit
    && runtime["appElfSha256"] === identity.app_elf_sha256
    && runtime["buildTimestampUtc"] === identity.build_timestamp_utc
    && runtime["version"] === identity.build_label
    && runtime["runningPartition"] === identity.running_partition
    && runtime["startMiningOnBoot"] === false;
}

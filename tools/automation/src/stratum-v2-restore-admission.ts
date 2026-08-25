import { readFile } from "node:fs/promises";
import path from "node:path";

import { sha256, type RestoreBundle } from "./stratum-v2-restore-model.js";
import { validateRestoreReadiness } from "./stratum-v2-restore-validator.js";
import { validateValidatorChildReceipt } from "./stratum-v2-validator-child.js";

const restoreProjection =
  "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-005.json";
const restorePlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";

type RunProcess = (
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
) => Promise<{ readonly stdout: string }>;

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
  await validateRestoreReadiness(bundlePath, projectionPath, source, planSha256);
  await validateValidatorChildReceipt(
    path.join(path.dirname(bundlePath), "validator-child-receipt.private.json"),
    source,
    planSha256,
  );
  return {
    bundle: JSON.parse(await readFile(bundlePath, "utf8")) as RestoreBundle,
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

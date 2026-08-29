import { lstat, readFile } from "node:fs/promises";
import path from "node:path";

import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import {
  noiseStagesFromMonitor,
  noiseTerminalFromMonitor,
  noiseTimingsFromMonitor,
} from "./stratum-v2-noise-diagnostic-markers.js";
import type { NoiseDiagnosticArgs } from "./stratum-v2-noise-diagnostic.js";
import { noiseAuthEvaluatorIdentity } from "./stratum-v2-noise-diagnostic-validator.js";
import { buildNoiseAuthProjection } from "./stratum-v2-noise-projection.js";
import { publishNoiseAuthProjection } from "./stratum-v2-noise-publish.js";

const diagnosticRoot = "scratch/str005-noise-auth/diagnostic-001";
const recoveryRoot = "scratch/str005-noise-auth/recovery-001";
const publicProjection =
  "docs/parity/evidence/str005-noise-auth/noise-auth-projection-001.json";
const plan = "docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md";
const planSha256 = "9a3e5a630a52de6b8819dcb33aac64f5324df030fab50fd248fc33437b6587ea";

async function privateObject(candidate: string): Promise<JsonObject> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
    throw new Error("protected_input");
  }
  const value: unknown = JSON.parse(await readFile(candidate, "utf8"));
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("protected_input");
  }
  return value as JsonObject;
}

export async function finalizeNoiseAuthDiagnostic(
  workspace: string,
  args: NoiseDiagnosticArgs,
): Promise<JsonObject> {
  if (args.action !== "finalize" || args.privateRoot !== diagnosticRoot
    || args.projection !== publicProjection || args.plan !== plan
    || args.diagnosticOrdinal !== 1) {
    throw new Error("finalize_invocation");
  }
  try {
    await lstat(path.join(workspace, publicProjection));
    throw new Error("projection_exists");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  const [intent, diagnosticChild, fixtureTerminal, recoveryResult, monitorOutput] =
    await Promise.all([
      privateObject(path.join(workspace, diagnosticRoot, "intent.private.json")),
      privateObject(path.join(workspace, diagnosticRoot, "diagnostic-child.private.json")),
      privateObject(path.join(workspace, diagnosticRoot, "fixture/terminal.json")),
      privateObject(path.join(workspace, recoveryRoot, "recovery-result.private.json")),
      readFile(path.join(workspace, diagnosticRoot, "diagnostic.stdout.private.log"), "utf8"),
    ]);
  const diagnosticSource = String(intent["source_commit"] ?? "");
  const recoverySource = String(recoveryResult["source_commit"] ?? "");
  if (intent["schema_version"] !== "bitaxe-stratum-v2-noise-auth-intent-v1"
    || intent["diagnostic_ordinal"] !== 1 || intent["plan_sha256"] !== planSha256
    || typeof intent["package_manifest_sha256"] !== "string"
    || typeof intent["reference_commit"] !== "string"
    || typeof intent["app_elf_sha256"] !== "string"
    || recoveryResult["restored_identity"] !== true
    || recoveryResult["settings_exact"] !== true
    || recoveryResult["mineonboot_disabled"] !== true
    || recoveryResult["mining_inactive"] !== true
    || recoveryResult["zero_work"] !== true
    || recoveryResult["cleanup_complete"] !== true) {
    throw new Error("finalize_identity");
  }
  for (const [ancestor, descendant] of [
    [diagnosticSource, recoverySource] as const,
    [recoverySource, "HEAD"] as const,
  ]) {
    const ancestry = await runCampaignProcess(
      workspace,
      "git",
      ["merge-base", "--is-ancestor", ancestor, descendant],
      5_000,
    );
    if (ancestry.exitCode !== 0) throw new Error("finalize_lineage");
  }
  const terminal = noiseTerminalFromMonitor(monitorOutput);
  const fixtureProgress = fixtureTerminal["progress"] as JsonObject;
  const projection = buildNoiseAuthProjection({
    sourceCommit: diagnosticSource,
    referenceCommit: intent["reference_commit"],
    appElfSha256: intent["app_elf_sha256"],
    planSha256,
    packageManifestSha256: String(intent["package_manifest_sha256"]),
    evaluatorSha256: await noiseAuthEvaluatorIdentity(workspace),
    earliestCategory: String(terminal["category"] ?? "terminal_missing"),
    stages: noiseStagesFromMonitor(monitorOutput),
    timings: noiseTimingsFromMonitor(monitorOutput),
    terminal,
    monitorOutput,
    fixtureTerminal,
    fixtureProgress,
    diagnosticExitCode: Number(diagnosticChild["exit_code"] ?? 1),
    restoration: {
      identity_exact: true,
      settings_exact: true,
      mineonboot_disabled: true,
      mining_inactive: true,
      zero_work: true,
      usb_cleanup_complete: true,
      owned_processes_remaining: 0,
    },
  });
  await publishNoiseAuthProjection(
    workspace,
    diagnosticRoot,
    publicProjection,
    projection,
    diagnosticSource,
    1,
    runCampaignProcess,
  );
  return {
    schema_version: "bitaxe-stratum-v2-noise-auth-finalize-result-v1",
    status: "accepted",
    category: "complete",
    projection_published: true,
    diagnostic_source_commit: diagnosticSource,
  };
}

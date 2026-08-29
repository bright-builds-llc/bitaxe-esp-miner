import { createHash } from "node:crypto";
import { lstat, readFile } from "node:fs/promises";
import path from "node:path";

import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import {
  tcpPayloadSocketErrorsFromMonitor,
  tcpPayloadStagesFromMonitor,
  tcpPayloadTerminalFromMonitor,
  tcpPayloadTimingsFromMonitor,
} from "./stratum-v2-tcp-payload-markers.js";
import { tcpPayloadDiagnosticAccepted } from "./stratum-v2-tcp-payload-process.js";
import type { TcpPayloadDiagnosticArgs } from "./stratum-v2-tcp-payload.js";
import { tcpPayloadEvaluatorIdentity } from "./stratum-v2-tcp-payload-validator.js";
import { buildTcpPayloadProjection } from "./stratum-v2-tcp-projection.js";
import { publishTcpPayloadProjection } from "./stratum-v2-tcp-publish.js";

const diagnosticRoot = "scratch/str005-tcp-payload/diagnostic-009";
const recoveryRoot = "scratch/str005-tcp-payload/recovery-003";
const publicProjection =
  "docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json";
const plan =
  "docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md";
const planSha256 = "544f57f8c940bc4e5cfeb69539928e153629b55dc12c5d04e404219ca48a5ba5";

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

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

export async function finalizeTcpPayloadDiagnostic(
  workspace: string,
  args: TcpPayloadDiagnosticArgs,
): Promise<JsonObject> {
  if (args.action !== "finalize"
    || args.privateRoot !== diagnosticRoot
    || args.projection !== publicProjection
    || args.plan !== plan
    || args.diagnosticOrdinal !== 9) {
    throw new Error("finalize_invocation");
  }
  try {
    await lstat(path.join(workspace, publicProjection));
    throw new Error("projection_exists");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  const intent = await privateObject(path.join(workspace, diagnosticRoot, "intent.private.json"));
  const diagnosticChild = await privateObject(path.join(
    workspace,
    diagnosticRoot,
    "diagnostic-child.private.json",
  ));
  const fixtureTerminal = await privateObject(path.join(
    workspace,
    diagnosticRoot,
    "fixture/terminal.json",
  ));
  const recoveryResult = await privateObject(path.join(
    workspace,
    recoveryRoot,
    "recovery-result.private.json",
  ));
  const monitorOutput = await readFile(path.join(
    workspace,
    diagnosticRoot,
    "diagnostic.stdout.private.log",
  ), "utf8");
  const manifestDocument = await readFile(path.join(workspace, args.packageManifest), "utf8");
  const manifest = JSON.parse(manifestDocument) as JsonObject;
  const diagnosticSource = String(intent["source_commit"] ?? "");
  if (intent["diagnostic_ordinal"] !== 9
    || intent["plan_sha256"] !== planSha256
    || manifest["source_commit"] !== diagnosticSource
    || recoveryResult["source_commit"] !== diagnosticSource
    || recoveryResult["package_manifest_sha256"] !== sha256(manifestDocument)
    || recoveryResult["restored_identity"] !== true
    || recoveryResult["settings_exact"] !== true
    || recoveryResult["mineonboot_disabled"] !== true
    || recoveryResult["mining_inactive"] !== true
    || recoveryResult["zero_work"] !== true
    || recoveryResult["cleanup_complete"] !== true) {
    throw new Error("finalize_identity");
  }
  const ancestry = await runCampaignProcess(
    workspace,
    "git",
    ["merge-base", "--is-ancestor", diagnosticSource, "HEAD"],
    5_000,
  );
  if (ancestry.exitCode !== 0) throw new Error("finalize_lineage");
  const terminal = tcpPayloadTerminalFromMonitor(monitorOutput);
  const fixtureProgress = fixtureTerminal["progress"] as JsonObject;
  const projection = buildTcpPayloadProjection({
    sourceCommit: diagnosticSource,
    referenceCommit: manifest["reference_commit"],
    appElfSha256: manifest["app_elf_sha256"],
    planSha256,
    packageManifestSha256: sha256(manifestDocument),
    evaluatorSha256: await tcpPayloadEvaluatorIdentity(workspace),
    earliestCategory: String(terminal["category"] ?? "terminal_missing"),
    stages: tcpPayloadStagesFromMonitor(monitorOutput),
    timings: tcpPayloadTimingsFromMonitor(monitorOutput),
    socketErrors: tcpPayloadSocketErrorsFromMonitor(monitorOutput),
    terminal,
    monitorOutput,
    fixtureTerminal,
    fixtureProgress,
    diagnosticAccepted: tcpPayloadDiagnosticAccepted(
      Number(diagnosticChild["exit_code"] ?? 1),
      diagnosticChild["timed_out"] === true,
      terminal,
      fixtureTerminal,
    ),
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
  await publishTcpPayloadProjection(
    workspace,
    diagnosticRoot,
    publicProjection,
    projection,
    diagnosticSource,
    9,
    runCampaignProcess,
  );
  return {
    schema_version: "bitaxe-stratum-v2-tcp-finalize-result-v1",
    status: "accepted",
    category: "complete",
    projection_published: true,
    diagnostic_source_commit: diagnosticSource,
  };
}

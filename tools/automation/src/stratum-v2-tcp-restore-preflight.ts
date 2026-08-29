import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { sha256, type RestoreBundle } from "./stratum-v2-restore-model.js";

const preflightRootRelative = "scratch/str005-tcp-payload/preflight-009";
const recoveryPlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";

type RunProcess = (
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
) => Promise<{ readonly exitCode: number; readonly stdout: string; readonly stderr: string }>;

export class TcpPayloadRestorePreflightError extends Error {
  public constructor() {
    super("restore_admission");
    this.name = "TcpPayloadRestorePreflightError";
  }
}

export type RestorePreflightInput = {
  readonly workspace: string;
  readonly flashProgram: string;
  readonly port: string;
  readonly restoreBundleRelative: string;
  readonly restoreBundlePath: string;
  readonly restoreBundle: RestoreBundle;
  readonly planRelative: string;
  readonly planSha256: string;
  readonly wifiCredentialsRelative: string;
  readonly sourceCommit: string;
  readonly referenceCommit: unknown;
  readonly runProcess: RunProcess;
};

export type RestorePreflightContract = {
  readonly rootRelative: string;
  readonly ordinal: number;
  readonly action: string;
};

async function isValidReceipt(
  candidate: string,
  sourceCommit: string,
  planSha256: string,
  bundleSha256: string,
): Promise<boolean> {
  try {
    const metadata = await lstat(candidate);
    if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
      return false;
    }
    const value = JSON.parse(await readFile(candidate, "utf8")) as Record<string, unknown>;
    return value["schema_version"] === "bitaxe-stratum-v2-restore-preflight-v1"
      && value["source_commit"] === sourceCommit
      && value["plan_sha256"] === planSha256
      && value["bundle_sha256"] === bundleSha256
      && value["status"] === "ready";
  } catch { return false; }
}

export async function admitDiagnosticRestorePreflight(
  input: RestorePreflightInput,
  contract: RestorePreflightContract,
): Promise<void> {
  const root = path.join(input.workspace, contract.rootRelative);
  const authorizationRelative = path.join(
    contract.rootRelative,
    "restore-authorization.private.json",
  );
  const authorization = path.join(input.workspace, authorizationRelative);
  const receipt = path.join(root, "admission-receipt.private.json");
  const bundleDocument = await readFile(input.restoreBundlePath, "utf8");
  const bundleSha256 = sha256(bundleDocument);
  if (await isValidReceipt(receipt, input.sourceCommit, input.planSha256, bundleSha256)) return;
  try {
    await mkdir(root, { recursive: true, mode: 0o700 });
    await chmod(root, 0o700);
    const recoveryPlanDocument = await readFile(path.join(input.workspace, recoveryPlan), "utf8");
    await writeFile(authorization, `${JSON.stringify({
      schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
      board: 205,
      ordinal: contract.ordinal,
      action: contract.action,
      current_source_commit: input.sourceCommit,
      reference_commit: input.referenceCommit,
      bundle_sha256: bundleSha256,
      bundle_capture_source_commit: input.restoreBundle.capture_source_commit,
      recovery_plan_sha256: sha256(recoveryPlanDocument),
      remediation_plan_sha256: input.planSha256,
    }, null, 2)}\n`, { flag: "wx", mode: 0o600 });
    await chmod(authorization, 0o600);
    const outcome = await input.runProcess(input.workspace, input.flashProgram, [
      "restore-installed", "--board", "205", "--port", input.port,
      "--restore-bundle", input.restoreBundleRelative,
      "--restore-authorization", authorizationRelative,
      "--remediation-plan", input.planRelative,
      "--private-root", contract.rootRelative,
      "--wifi-credentials", input.wifiCredentialsRelative,
      "--admission-only", "--redact-evidence",
    ], 120_000);
    if (outcome.exitCode !== 0) throw new TcpPayloadRestorePreflightError();
    await writeFile(receipt, `${JSON.stringify({
      schema_version: "bitaxe-stratum-v2-restore-preflight-v1",
      status: "ready",
      source_commit: input.sourceCommit,
      plan_sha256: input.planSha256,
      bundle_sha256: bundleSha256,
    }, null, 2)}\n`, { flag: "wx", mode: 0o600 });
    await chmod(receipt, 0o600);
  } catch (error) {
    if (error instanceof TcpPayloadRestorePreflightError) throw error;
    throw new TcpPayloadRestorePreflightError();
  }
}

export async function admitTcpPayloadRestorePreflight(input: RestorePreflightInput): Promise<void> {
  await admitDiagnosticRestorePreflight(input, {
    rootRelative: preflightRootRelative,
    ordinal: 9,
    action: "tcp_payload_restore_preflight",
  });
}

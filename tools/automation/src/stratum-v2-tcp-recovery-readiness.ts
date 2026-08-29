import path from "node:path";

import { admitStratumV2RestoreBundle } from "./stratum-v2-restore-admission.js";
import {
  TcpPayloadRecoveryToolingError,
  validateTcpPayloadRecoveryTooling,
} from "./stratum-v2-tcp-recovery-tooling.js";
import {
  admitDiagnosticRestorePreflight,
  TcpPayloadRestorePreflightError,
} from "./stratum-v2-tcp-restore-preflight.js";

type RunProcess = (
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
) => Promise<{ readonly exitCode: number; readonly stdout: string; readonly stderr: string }>;

type Fail = (category: string, message: string, checkpoint: string) => never;

type RecoveryReadinessInput = {
  readonly workspace: string;
  readonly port: string;
  readonly restoreBundleRelative: string;
  readonly planRelative: string;
  readonly planSha256: string;
  readonly wifiCredentialsRelative: string;
  readonly sourceCommit: string;
  readonly referenceCommit: unknown;
  readonly runProcess: RunProcess;
  readonly fail: Fail;
};

type DiagnosticRecoveryReadinessInput = RecoveryReadinessInput & {
  readonly preflightRootRelative: string;
  readonly restoreOrdinal: number;
  readonly restoreAction: string;
};

export async function admitDiagnosticRecoveryReadiness(input: DiagnosticRecoveryReadinessInput) {
  let restore: Awaited<ReturnType<typeof admitStratumV2RestoreBundle>>;
  try {
    restore = await admitStratumV2RestoreBundle(
      input.workspace,
      input.restoreBundleRelative,
      input.runProcess,
    );
  } catch (error) {
    if (error instanceof Error
      && error.message === "public recovery projection mode is invalid") {
      input.fail("evidence_invalid", "recovery projection mode", "recovery_projection_mode");
    }
    input.fail("evidence_invalid", "restore readiness invalid", "restoration_inputs");
  }
  try {
    await validateTcpPayloadRecoveryTooling(input.workspace, input.runProcess);
  } catch (error) {
    if (error instanceof TcpPayloadRecoveryToolingError) {
      input.fail("evidence_invalid", "restore tooling unavailable", error.checkpoint);
    }
    input.fail("evidence_invalid", "restore tooling failed", "restore_tooling");
  }
  try {
    await admitDiagnosticRestorePreflight({
      workspace: input.workspace,
      flashProgram: path.join(input.workspace, "bazel-bin/tools/flash/flash"),
      port: input.port,
      restoreBundleRelative: input.restoreBundleRelative,
      restoreBundlePath: restore.path,
      restoreBundle: restore.bundle,
      planRelative: input.planRelative,
      planSha256: input.planSha256,
      wifiCredentialsRelative: input.wifiCredentialsRelative,
      sourceCommit: input.sourceCommit,
      referenceCommit: input.referenceCommit,
      runProcess: input.runProcess,
    }, {
      rootRelative: input.preflightRootRelative,
      ordinal: input.restoreOrdinal,
      action: input.restoreAction,
    });
  } catch (error) {
    if (error instanceof TcpPayloadRestorePreflightError) {
      input.fail("evidence_invalid", "restore admission failed", "restore_admission");
    }
    input.fail("evidence_invalid", "restore admission failed", "restore_admission");
  }
  return restore;
}

export async function admitTcpPayloadRecoveryReadiness(input: RecoveryReadinessInput) {
  return admitDiagnosticRecoveryReadiness({
    ...input,
    preflightRootRelative: "scratch/str005-tcp-payload/preflight-009",
    restoreOrdinal: 9,
    restoreAction: "tcp_payload_restore_preflight",
  });
}

import { createHash } from "node:crypto";

import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { sourceWorkspaceRoot } from "./workspace.js";

export type NativeUsbRecoveryAction = "preflight" | "start" | "finalize";

export type NativeUsbRecoveryArgs = {
  readonly action: NativeUsbRecoveryAction;
  readonly board: "205";
  readonly port: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly restoreBundle: string;
  readonly privateRoot: string;
  readonly plan: string;
  readonly recoveryOrdinal: 2 | 3;
  readonly projection: string;
  readonly redactEvidence: true;
};

export const planRelative =
  "docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md";
export const planSha256 =
  "cbc11639a51e67d24a04b33c05dd3dd2e570914be79f3a3d80b7326894e74eca";
export const taskId = "task-native-usb-recovery-transition-205";
export const packageManifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
export const wifiCredentials = "wifi-credentials.json";
export const restoreBundle =
  "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
export const primaryRoot = "scratch/native-usb-transition/recovery-002";
export const contingencyRoot = "scratch/native-usb-transition/recovery-003";
export const transientPreflightRoot = "scratch/native-usb-transition/.preflight-002";
export const diagnosticRoot = "scratch/native-usb-transition/diagnostic-001";
export const projectionRelative =
  "docs/parity/evidence/native-usb-transition/transition-projection-001.json";
export const recoveryPlan =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
export const backupRelative =
  "scratch/str005-stratum-v2/attempt-004/settings-backup.private.json";
export const backupSha256 =
  "ac3d28d451c466f4fc6bfdc40b327c891dac9f3eba644ce62a7f2a2276790631";

export class NativeUsbRecoveryError extends Error {
  public constructor(
    public readonly category: string,
    public readonly checkpoint: string,
  ) {
    super(`${category}:${checkpoint}`);
    this.name = "NativeUsbRecoveryError";
  }
}

export function fail(category: string, checkpoint: string): never {
  throw new NativeUsbRecoveryError(category, checkpoint);
}

export function object(value: unknown, checkpoint: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", checkpoint);
  }
  return value as JsonObject;
}

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function nativeUsbRecoveryWorkspaceRoot(
  environment: NodeJS.ProcessEnv = process.env,
  currentDirectory = process.cwd(),
): string {
  const configured = environment["BUILD_WORKSPACE_DIRECTORY"];
  return sourceWorkspaceRoot(configured === undefined
    ? [currentDirectory]
    : [configured, currentDirectory]);
}

export function parseNativeUsbRecoveryArgs(
  action: string | undefined,
  values: readonly string[],
): NativeUsbRecoveryArgs {
  if (action !== "preflight" && action !== "start" && action !== "finalize") {
    fail("invalid_invocation", "action");
  }
  const parsed = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (parsed.has(key)) fail("invalid_invocation", "duplicate_option");
      parsed.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || !key.startsWith("--") || value === undefined
      || value.startsWith("--") || parsed.has(key)) {
      fail("invalid_invocation", "option_shape");
    }
    parsed.set(key, value);
    index += 1;
  }
  const allowed = new Set([
    "--board", "--port", "--package-manifest", "--wifi-credentials", "--restore-bundle",
    "--private-root", "--plan", "--recovery-ordinal", "--projection", "--redact-evidence",
  ]);
  if ([...parsed.keys()].some(key => !allowed.has(key))) {
    fail("invalid_invocation", "unsupported_option");
  }
  const value = (key: string): string => {
    const candidate = parsed.get(key);
    if (typeof candidate !== "string" || candidate.length === 0) {
      fail("invalid_invocation", "required_option");
    }
    return candidate;
  };
  const ordinalValue = value("--recovery-ordinal");
  const recoveryOrdinal = ordinalValue === "2" ? 2 : ordinalValue === "3" ? 3 : undefined;
  if (recoveryOrdinal === undefined) fail("invalid_invocation", "ordinal");
  const expectedRoot = recoveryOrdinal === 2 ? primaryRoot : contingencyRoot;
  if (value("--board") !== "205"
    || value("--package-manifest") !== packageManifest
    || value("--wifi-credentials") !== wifiCredentials
    || value("--restore-bundle") !== restoreBundle
    || value("--private-root") !== expectedRoot
    || value("--plan") !== planRelative
    || value("--projection") !== projectionRelative
    || parsed.get("--redact-evidence") !== true
    || (action === "preflight" && recoveryOrdinal !== 2)) {
    fail("invalid_invocation", "contract");
  }
  return {
    action,
    board: "205",
    port: value("--port"),
    packageManifest,
    wifiCredentials,
    restoreBundle,
    privateRoot: expectedRoot,
    plan: planRelative,
    recoveryOrdinal,
    projection: projectionRelative,
    redactEvidence: true,
  };
}

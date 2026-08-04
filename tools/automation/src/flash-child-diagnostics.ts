import { lstat, readFile } from "node:fs/promises";

import type { ProcessOutcome } from "./process.js";

const digestPattern = /^[a-f0-9]{64}$/u;
const effectStatuses: ReadonlySet<string> = new Set([
  "completed",
  "failed_after_completed_device_effect",
  "failed_confirmed_partial_device_effect",
  "failed_no_device_effect",
] as const);
const dualEvidenceMarkers: ReadonlySet<string> = new Set([
  "capture_failed",
  "capture_not_accepted",
  "evidence_record_failed",
  "flash_workflow_failed",
  "monitor_preparation_failed",
  "path_preflight_failed",
  "private_capture_failed",
  "private_capture_unreadable",
  "private_digest_failed",
  "root_admission_failed",
] as const);
const effectFailures: ReadonlySet<unknown> = new Set([
  "flash_failed",
  "invocation_construction_failed",
  "parser_failed",
]);

type EffectStatus = "completed"
  | "failed_after_completed_device_effect"
  | "failed_confirmed_partial_device_effect"
  | "failed_no_device_effect";

type EffectInspection = {
  readonly flash_effect_result_status: "valid" | "missing" | "invalid";
  readonly flash_effect_status: EffectStatus | "unavailable";
};

type ExpectedEffectIdentity = {
  readonly packageIdentityDigest: string;
  readonly factoryImageDigest: string;
};

export type FlashChildFailureFacts = EffectInspection & {
  readonly stage: "initial_flash_monitor";
  readonly flash_monitor_exit_code: number | "unavailable";
  readonly flash_monitor_timed_out: boolean;
  readonly flash_monitor_terminal_marker: string;
};

function object(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
}

function isDigest(value: unknown): value is string {
  return typeof value === "string" && digestPattern.test(value);
}

export function factoryImageDigest(manifest: Record<string, unknown>): string {
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts)) throw new Error("package manifest artifacts are invalid");
  const matches = artifacts.filter((candidate) => object(candidate)?.["kind"] === "factory_merged_image");
  if (matches.length !== 1) throw new Error("package manifest factory image is invalid");
  const digest = object(matches[0])?.["sha256"];
  if (!isDigest(digest)) throw new Error("package manifest factory image digest is invalid");
  return digest;
}

export function flashEffectEnvironment(
  resultPath: string,
  expected: ExpectedEffectIdentity,
): Readonly<Record<string, string>> {
  if (!isDigest(expected.packageIdentityDigest) || !isDigest(expected.factoryImageDigest)) {
    throw new Error("flash effect identity is invalid");
  }
  return {
    PHASE36_EFFECT_RESULT_PATH: resultPath,
    PHASE36_EFFECT_OPERATION: "exact_package_flash",
    PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST: expected.packageIdentityDigest,
    PHASE36_EFFECT_FACTORY_IMAGE_DIGEST: expected.factoryImageDigest,
  };
}

function validEffect(value: unknown, expected: ExpectedEffectIdentity): EffectStatus | undefined {
  const candidate = object(value);
  if (candidate === undefined) return undefined;
  const keys = Object.keys(candidate).sort();
  const expectedKeys = [
    "factory_image_digest",
    "failure",
    "operation",
    "package_identity_digest",
    "schema_version",
    "status",
  ];
  if (JSON.stringify(keys) !== JSON.stringify(expectedKeys)) return undefined;
  if (candidate["schema_version"] !== "phase36-effect-result-v1") return undefined;
  if (candidate["operation"] !== "exact_package_flash") return undefined;
  if (candidate["package_identity_digest"] !== expected.packageIdentityDigest) return undefined;
  if (candidate["factory_image_digest"] !== expected.factoryImageDigest) return undefined;
  const status = candidate["status"];
  if (typeof status !== "string" || !effectStatuses.has(status)) return undefined;
  const failure = candidate["failure"];
  if (status === "completed" ? failure !== null : !effectFailures.has(failure)) return undefined;
  return status as EffectStatus;
}

export async function inspectFlashEffect(
  resultPath: string,
  expected: ExpectedEffectIdentity,
): Promise<EffectInspection> {
  let metadata;
  try {
    metadata = await lstat(resultPath);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return { flash_effect_result_status: "missing", flash_effect_status: "unavailable" };
    }
    return { flash_effect_result_status: "invalid", flash_effect_status: "unavailable" };
  }
  if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o600) {
    return { flash_effect_result_status: "invalid", flash_effect_status: "unavailable" };
  }
  try {
    const status = validEffect(JSON.parse(await readFile(resultPath, "utf8")), expected);
    return status === undefined
      ? { flash_effect_result_status: "invalid", flash_effect_status: "unavailable" }
      : { flash_effect_result_status: "valid", flash_effect_status: status };
  } catch {
    return { flash_effect_result_status: "invalid", flash_effect_status: "unavailable" };
  }
}

export function flashMonitorTerminalMarker(stderr: string): string {
  const matches = [...stderr.matchAll(/dual_evidence=failed reason=([a-z_]+)/gu)]
    .map((match) => match[1])
    .filter((reason): reason is string => reason !== undefined && dualEvidenceMarkers.has(reason));
  const unique = [...new Set(matches)];
  if (unique.length === 0) return "unclassified";
  if (unique.length > 1) return "multiple_allowlisted_markers";
  return unique[0] ?? "unclassified";
}

export function flashChildFailureFacts(
  maybeOutcome: ProcessOutcome | undefined,
  effect: EffectInspection,
): FlashChildFailureFacts {
  const maybeExitCode = maybeOutcome?.exitCode;
  const exitCode = typeof maybeExitCode === "number"
    && Number.isSafeInteger(maybeExitCode)
    && maybeExitCode >= 0
    && maybeExitCode <= 255
    ? maybeExitCode
    : "unavailable";
  return {
    stage: "initial_flash_monitor",
    flash_monitor_exit_code: exitCode,
    flash_monitor_timed_out: maybeOutcome?.timedOut ?? false,
    flash_monitor_terminal_marker: maybeOutcome === undefined
      ? "launch_failed"
      : flashMonitorTerminalMarker(maybeOutcome.stderr),
    ...effect,
  };
}

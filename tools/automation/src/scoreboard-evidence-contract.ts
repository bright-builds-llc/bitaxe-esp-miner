import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

import type { AutomationCategory } from "./contracts.generated.js";
import { scoreboardSourceInventory } from "./scoreboard-source-inventory.js";

export type JsonObject = Readonly<Record<string, unknown>>;
export type ScoreboardView = Readonly<{
  count: number;
  digest: string;
  durableDigest: string;
}>;
type FailureCategory = Extract<
  AutomationCategory,
  "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed"
>;

export const expectedPrivateRoot = "scratch/stat003-scoreboard/attempt-005";
export const expectedWrapperRoot = "scratch/stat003-scoreboard/wrapper-005";
export const expectedProjection =
  "docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json";
export const expectedPlan = "docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md";
export const expectedPlanSha256 =
  "43d13ec599e9f46988f0ebb44607dc000eff95db78c37fdc340fe52e14365684";
export const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const activeTask = "task-parity-stat003-scoreboard";

export class ScoreboardEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ScoreboardEvidenceError";
  }
}

export function failure(
  category: FailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): ScoreboardEvidenceError {
  return new ScoreboardEvidenceError(category, message, {
    stage: "scoreboard_capture",
    projection_published: false,
    ...facts,
  });
}

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

export function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

export function requiredInteger(value: JsonObject, field: string, context: string): number {
  const candidate = value[field];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 0) {
    throw failure("evidence_invalid", `${context} integer field is invalid`);
  }
  return candidate;
}

export function requiredBoolean(value: JsonObject, field: string, context: string): boolean {
  const candidate = value[field];
  if (typeof candidate !== "boolean") {
    throw failure("evidence_invalid", `${context} boolean field is invalid`);
  }
  return candidate;
}

export function bootMiningDisabled(
  startMiningOnBoot: boolean,
  miningActivity: string,
): boolean {
  return !startMiningOnBoot
    && (miningActivity === "paused" || miningActivity === "safe_blocked");
}

const fractionMask = (1n << 52n) - 1n;
const implicitFractionBit = 1n << 52n;

/** Matches the pinned Rust `{:.1}` and upstream C `%.1f` durable codec. */
export function durableDifficulty(difficulty: number): number {
  if (!Number.isFinite(difficulty) || difficulty <= 0) {
    throw failure("evidence_invalid", "scoreboard durable difficulty is invalid");
  }

  const bytes = new ArrayBuffer(8);
  const view = new DataView(bytes);
  view.setFloat64(0, difficulty, false);
  const bits = view.getBigUint64(0, false);
  const exponentBits = Number((bits >> 52n) & 0x7ffn);
  const fraction = bits & fractionMask;
  const significand = exponentBits === 0 ? fraction : implicitFractionBit + fraction;
  const binaryExponent = exponentBits === 0 ? -1074 : exponentBits - 1023 - 52;
  const scaledSignificand = significand * 10n;

  let roundedTenths: bigint;
  if (binaryExponent >= 0) {
    roundedTenths = scaledSignificand << BigInt(binaryExponent);
  } else {
    const divisor = 1n << BigInt(-binaryExponent);
    const quotient = scaledSignificand / divisor;
    const remainder = scaledSignificand % divisor;
    const doubledRemainder = remainder * 2n;
    const roundsUp = doubledRemainder > divisor
      || (doubledRemainder === divisor && quotient % 2n !== 0n);
    roundedTenths = quotient + (roundsUp ? 1n : 0n);
  }

  const digits = roundedTenths.toString();
  const decimal = digits.length === 1
    ? `0.${digits}`
    : `${digits.slice(0, -1)}.${digits.slice(-1)}`;
  const projected = Number(decimal);
  if (!Number.isFinite(projected) || projected <= 0) {
    throw failure("evidence_invalid", "scoreboard durable difficulty projection is invalid");
  }
  return projected;
}

export function scoreboardView(value: readonly unknown[], context: string): ScoreboardView {
  if (value.length === 0 || value.length > 20) {
    throw failure("hardware_blocked", `${context} entry count is incomplete`);
  }
  let maybePreviousDifficulty: number | undefined;
  const normalized = value.map((candidate) => {
    const entry = object(candidate, context);
    const keys = Object.keys(entry).sort();
    const expectedKeys = [
      "difficulty", "extranonce2", "job_id", "nonce", "ntime", "version_bits",
    ];
    if (keys.length !== expectedKeys.length
      || keys.some((key, index) => key !== expectedKeys[index])) {
      throw failure("evidence_invalid", `${context} wire shape is invalid`);
    }
    const difficulty = entry["difficulty"];
    if (typeof difficulty !== "number" || !Number.isFinite(difficulty) || difficulty <= 0) {
      throw failure("evidence_invalid", `${context} difficulty is invalid`);
    }
    if (maybePreviousDifficulty !== undefined && difficulty > maybePreviousDifficulty) {
      throw failure("evidence_invalid", `${context} order is invalid`);
    }
    maybePreviousDifficulty = difficulty;
    const jobId = requiredString(entry, "job_id", context);
    const extranonce2 = requiredString(entry, "extranonce2", context);
    const ntime = requiredInteger(entry, "ntime", context);
    const nonce = requiredString(entry, "nonce", context);
    const versionBits = requiredString(entry, "version_bits", context);
    if (jobId.length > 31 || extranonce2.length > 31 || ntime > 0xffff_ffff
      || !/^[0-9A-F]{8}$/u.test(nonce) || !/^[0-9A-F]{8}$/u.test(versionBits)) {
      throw failure("evidence_invalid", `${context} bounded field is invalid`);
    }
    return { difficulty, job_id: jobId, extranonce2, ntime, nonce, version_bits: versionBits };
  });
  const durable = normalized.map((entry) => ({
    ...entry,
    difficulty: durableDifficulty(entry.difficulty),
  }));
  return {
    count: normalized.length,
    digest: sha256(JSON.stringify(normalized)),
    durableDigest: sha256(JSON.stringify(durable)),
  };
}

export function scoreboardRestartPersists(
  beforeRestart: ScoreboardView,
  afterRestart: ScoreboardView,
): boolean {
  return beforeRestart.count === afterRestart.count
    && beforeRestart.durableDigest === afterRestart.digest
    && afterRestart.durableDigest === afterRestart.digest;
}

export async function validateScoreboardTaskAndSources(
  workspaceRoot: string,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<{ readonly digest: string; readonly pathCount: number }> {
  const [taskDocument, archivedTaskDocument, planDocument] = await Promise.all([
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
    readFile(path.join(workspaceRoot, "TASKS.archive.md"), "utf8").catch(() => ""),
    readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
  ]);
  const heading = `### ${activeTask} |`;
  const activeCount = taskDocument.split(heading).length - 1;
  const archivedCount = archivedTaskDocument.split(heading).length - 1;
  const sourceDocument = activeCount === 1 ? taskDocument : archivedTaskDocument;
  const start = sourceDocument.indexOf(heading);
  const maybeEnd = sourceDocument.indexOf("\n### ", start + heading.length);
  const block = sourceDocument.slice(start, maybeEnd === -1 ? sourceDocument.length : maybeEnd);
  if (activeCount + archivedCount !== 1
    || !block.includes(expectedPlan) || !block.includes("attempt-005")
    || sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `STAT-003`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "STAT-003 task or immutable plan binding is invalid");
  }
  try {
    const inventory = await scoreboardSourceInventory(workspaceRoot);
    if (inventory.pathCount !== 32) {
      throw new Error("path count drifted");
    }
    return inventory;
  } catch {
    throw failure("evidence_invalid", "scoreboard source semantics are invalid");
  }
}

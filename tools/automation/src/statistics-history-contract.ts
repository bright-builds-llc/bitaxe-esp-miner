import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

import type { AutomationCategory } from "./contracts.generated.js";

export type JsonObject = Readonly<Record<string, unknown>>;
export type HistoryView = {
  readonly labels: readonly string[];
  readonly rows: readonly (readonly number[])[];
  readonly timestamps: readonly number[];
};
export type RecoveryFacts = {
  readonly restoration_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly recovery_origin_readmitted: boolean;
  readonly secondary_recovery_failure: boolean;
};
type FailureCategory = Extract<
  AutomationCategory,
  "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed"
>;

export const expectedPrivateRoot = "scratch/stat002-statistics-history/attempt-003";
export const expectedWrapperRoot = "scratch/stat002-statistics-history/wrapper-003";
export const expectedProjection =
  "docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json";
export const expectedPlanSha256 =
  "7eeeededa3d7a6f671fe00eb6e2b0cbd2fd86d5516fc865d041191182029c631";
export const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
export const expectedLabels = [
  "hashrate", "hashrate_1m", "hashrate_10m", "hashrate_1h", "errorPercentage",
  "asicTemp", "asicTemp2", "vrTemp", "asicVoltage", "voltage", "power", "current",
  "fanSpeed", "fanRpm", "fan2Rpm", "wifiRssi", "freeHeap", "responseTime", "timestamp",
] as const;
export const noRecovery: RecoveryFacts = {
  restoration_complete: false,
  recovery_flash_used: false,
  recovery_origin_readmitted: false,
  secondary_recovery_failure: false,
};

const expectedPlan = "docs/parity/work-plans/20260816T221106Z-STAT-002/PLAN.md";
const activeTask = "task-parity-stat002-statistics-history";
const sourceFragments = new Map<string, readonly string[]>([
  ["tools/automation/src/cli.ts", [
    "createLocalProcessPort({ cwd: root, timeoutMs: 900_000 })",
  ]],
  ["tools/automation/src/statistics-history-evidence.ts", [
    "outcome = await processPort.run(spec);",
  ]],
  ["firmware/bitaxe/src/statistics_runtime.rs", [
    "pub const STATISTICS_CADENCE_MS: u64 = 1_000;",
    "record_statistics_sample(now_ms, frequency_seconds)",
  ]],
  ["firmware/bitaxe/src/runtime_snapshot.rs", [
    "pub fn record_statistics_sample(timestamp_ms: u64, frequency_seconds: u16)",
    "statistics_response(timestamp_ms, None, &statistics_samples())",
  ]],
  ["crates/bitaxe-api/src/statistics/history.rs", [
    "pub const MAX_STATISTICS_SAMPLES: usize = 720;",
    "if frequency_seconds == 0 {\n            return Ok(self.disable());",
  ]],
  ["crates/bitaxe-api/src/statistics.rs", [
    "const ALL_COLUMNS: [StatisticsColumn; 18]",
    "labels.push(TIMESTAMP_LABEL.to_owned())",
    "voltage_millivolts: millivolts_from_volts",
    "current_milliamps: milliamps_from_amps",
  ]],
]);
const referenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/tasks/statistics_task.c", [
    "#define DEFAULT_POLL_RATE 1000",
    "if (0 != configStatsFrequency)",
    "if (currentTime >= statsData.timestamp + 1000)",
    "addStatisticData(&statsData, configStatsFrequency);",
    "removeStatisticsBuffer();",
  ]],
]);

export class StatisticsHistoryEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "StatisticsHistoryEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): StatisticsHistoryEvidenceError {
    return new StatisticsHistoryEvidenceError(this.category, this.message, {
      ...this.publicValue,
      ...recovery,
    });
  }
}

export function failure(
  category: FailureCategory,
  message: string,
  facts: Readonly<Record<string, unknown>> = {},
): StatisticsHistoryEvidenceError {
  return new StatisticsHistoryEvidenceError(category, message, {
    stage: "statistics_history_capture",
    projection_published: false,
    ...facts,
    ...noRecovery,
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
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} string field is invalid`);
  }
  return candidate;
}

export function requiredFrequency(value: JsonObject, context: string): number {
  const candidate = value["statsFrequency"];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate)
    || candidate < 0 || candidate > 65_535) {
    throw failure("evidence_invalid", `${context} statsFrequency is invalid`);
  }
  return candidate;
}

export function validateIdentity(value: JsonObject, manifest: JsonObject): void {
  for (const [wire, source] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (requiredString(value, wire, "system info")
      !== requiredString(manifest, source, "package manifest")) {
      throw failure("evidence_invalid", "system info does not match the exact package");
    }
  }
}

export function historyView(value: unknown, context: string): HistoryView {
  const document = object(value, context);
  const labels = document["labels"];
  const statistics = document["statistics"];
  if (!Array.isArray(labels) || labels.length !== expectedLabels.length
    || labels.some((label, index) => label !== expectedLabels[index])) {
    throw failure("evidence_invalid", `${context} labels are invalid`);
  }
  if (!Array.isArray(statistics)) {
    throw failure("evidence_invalid", `${context} rows are invalid`);
  }
  const rows = statistics.map((maybeRow) => {
    if (!Array.isArray(maybeRow) || maybeRow.length !== expectedLabels.length
      || maybeRow.some((cell) => typeof cell !== "number" || !Number.isFinite(cell))) {
      throw failure("evidence_invalid", `${context} row is invalid`);
    }
    return maybeRow as number[];
  });
  const timestamps = rows.map((row) => row[expectedLabels.length - 1] as number);
  if (timestamps.some((timestamp) => !Number.isSafeInteger(timestamp) || timestamp < 0)
    || timestamps.some((timestamp, index) => index > 0 && timestamp <= (timestamps[index - 1] ?? -1))) {
    throw failure("hardware_blocked", `${context} timestamps are not strictly increasing`);
  }
  return { labels: labels as string[], rows, timestamps };
}

export function historyDigest(view: HistoryView): string {
  return sha256(JSON.stringify({ labels: view.labels, rows: view.rows }));
}

export async function validateStatisticsHistoryTaskAndSources(
  workspaceRoot: string,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<void> {
  const [taskDocument, planDocument] = await Promise.all([
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
    readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
  ]);
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const block = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1
    || !block.includes(expectedPlan) || !block.includes("attempt-003")
    || sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `STAT-002`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)) {
    throw failure("evidence_invalid", "STAT-002 task or immutable plan binding is invalid");
  }
  for (const [relative, fragments] of [...sourceFragments, ...referenceFragments]) {
    const document = await readFile(path.join(workspaceRoot, relative), "utf8");
    for (const fragment of fragments) {
      if (document.split(fragment).length !== 2) {
        throw failure("evidence_invalid", "statistics source semantics are invalid");
      }
    }
  }
}

// The SELF-001 campaign's typed failure and its protected-file and JSON helpers.
import { createHash } from "node:crypto";
import { chmod, rename, stat, writeFile } from "node:fs/promises";

export type JsonObject = Record<string, unknown>;

export class SelfTestCampaignError extends Error {
  public constructor(
    public readonly category: "evidence_invalid" | "hardware_blocked" | "process_failed" | "timeout",
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "SelfTestCampaignError";
  }
}

export function failure(
  category: SelfTestCampaignError["category"],
  message: string,
  stage: string,
): SelfTestCampaignError {
  return new SelfTestCampaignError(category, message, {
    stage,
    projection_published: false,
    checkpoint: "unavailable",
  });
}

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`, "contract");
  }
  return value as JsonObject;
}

export function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate.length === 0) {
    throw failure("evidence_invalid", `${context} identity is invalid`, "contract");
  }
  return candidate;
}

export async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent`, "preflight");
  } catch (error) {
    if (error instanceof SelfTestCampaignError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

export async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "protected evidence mode is invalid", "privacy");
  }
}

export async function privateJson(output: string, value: unknown): Promise<string> {
  const document = `${JSON.stringify(value, null, 2)}\n`;
  await writeFile(output, document, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
  return document;
}

export async function replacePrivateJson(output: string, value: unknown): Promise<string> {
  const temporary = `${output}.tmp`;
  await requireAbsent(temporary, "private replacement");
  const document = await privateJson(temporary, value);
  await rename(temporary, output);
  await requireMode(output, 0o600, false);
  return document;
}

import { chmod, lstat, mkdir, readdir, stat, writeFile } from "node:fs/promises";
import http from "node:http";
import path from "node:path";

import type { AutomationCategory } from "./contracts.generated.js";

export type FailureCategory = Extract<
  AutomationCategory,
  | "package_invalid" | "process_failed" | "timeout" | "hardware_blocked" | "evidence_invalid"
  | "origin_unavailable" | "update_not_observed" | "asset_identity_mismatch" | "interruption_not_observed"
  | "recovery_not_observed" | "nvs_not_preserved"
>;
export type JsonObject = Readonly<Record<string, unknown>>;
export type RecoveryFacts = {
  readonly recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};

export const noRecovery: RecoveryFacts = { recovery_complete: false, recovery_flash_used: false, secondary_recovery_failure: false };

export class OtawwwEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "OtawwwEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): OtawwwEvidenceError {
    return new OtawwwEvidenceError(this.category, this.message, { ...this.publicValue, ...recovery });
  }
}

export function failure(category: FailureCategory, message: string, facts: Readonly<Record<string, unknown>> = {}): OtawwwEvidenceError {
  return new OtawwwEvidenceError(category, message, { stage: "otawww_capture", ...noRecovery, ...facts });
}

export function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

export function text(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw failure("evidence_invalid", `${context} ${field} is invalid`);
  return candidate;
}

export function ordinal(value: JsonObject, context: string): number {
  const candidate = value["bootOrdinal"];
  if (typeof candidate !== "number" || !Number.isSafeInteger(candidate) || candidate < 1) {
    throw failure("evidence_invalid", `${context} boot ordinal is invalid`);
  }
  return candidate;
}

export async function absent(target: string, context: string): Promise<void> {
  try {
    await stat(target);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
  throw failure("evidence_invalid", `${context} must be absent before launch`);
}

export async function createPrivateRoot(root: string): Promise<void> {
  await absent(root, "private attempt root");
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

export async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

export async function privateDir(dir: string): Promise<void> {
  await mkdir(dir, { mode: 0o700 });
  await chmod(dir, 0o700);
}

export async function privateModesValid(root: string): Promise<boolean> {
  const metadata = await lstat(root);
  if (!metadata.isDirectory() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o700) return false;
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!await privateModesValid(candidate)) return false;
      continue;
    }
    const childMetadata = await lstat(candidate);
    if (!childMetadata.isFile() || childMetadata.isSymbolicLink() || (childMetadata.mode & 0o777) !== 0o600) return false;
  }
  return true;
}

export const delay = async (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

/** One same-origin binary POST with an explicit Origin header and a bounded reply. */
export async function postBinaryOnce(
  origin: URL,
  route: string,
  body: Buffer,
  timeoutMs: number,
): Promise<{ readonly status: number; readonly body: string }> {
  const target = new URL(route, origin);
  if (target.origin !== origin.origin || origin.pathname !== "/" || origin.search !== "" || origin.hash !== "") {
    throw new Error("binary upload target must stay on the admitted origin");
  }
  return new Promise((resolve, reject) => {
    const request = http.request(target, {
      method: "POST",
      headers: {
        Origin: origin.origin,
        "Content-Type": "application/octet-stream",
        "Content-Length": String(body.length),
        Connection: "close",
      },
    }, (response) => {
      const chunks: Buffer[] = [];
      let size = 0;
      response.on("data", (chunk: Buffer) => {
        size += chunk.length;
        if (size > 4_096) {
          request.destroy(new Error("upload response is too large"));
          return;
        }
        chunks.push(chunk);
      });
      response.on("end", () => resolve({ status: response.statusCode ?? 0, body: Buffer.concat(chunks).toString("utf8") }));
      response.on("error", reject);
    });
    request.setTimeout(timeoutMs, () => request.destroy(new Error("upload timed out")));
    request.on("error", reject);
    request.end(body);
  });
}

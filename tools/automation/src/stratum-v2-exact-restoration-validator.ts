import { readFile } from "node:fs/promises";

const allowedKeys = new Set([
  "schema_version", "status", "board", "remediation_ordinal",
  "original_runtime_restored", "settings_restored", "theme_restored",
  "mineonboot_false", "mining_safe_blocked", "zero_hashrate",
  "usb_cleanup_ready", "redaction_status", "source_commit",
]);

export async function validateExactRestorationProjection(
  candidate: string,
  sourceCommit: string,
): Promise<void> {
  const value = JSON.parse(await readFile(candidate, "utf8")) as Record<string, unknown>;
  if (Object.keys(value).length !== allowedKeys.size
    || Object.keys(value).some(key => !allowedKeys.has(key))
    || value["schema_version"] !== "bitaxe-stratum-v2-exact-restoration-v1"
    || value["status"] !== "accepted" || value["board"] !== 205
    || value["remediation_ordinal"] !== 1 || value["source_commit"] !== sourceCommit
    || value["redaction_status"] !== "passed"
    || ["original_runtime_restored", "settings_restored", "theme_restored",
      "mineonboot_false", "mining_safe_blocked", "zero_hashrate",
      "usb_cleanup_ready"].some(key => value[key] !== true)) {
    throw new Error("exact restoration projection rejected");
  }
}

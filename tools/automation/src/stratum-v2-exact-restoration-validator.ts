import { readFile } from "node:fs/promises";

const allowedKeys = new Set([
  "schema_version", "status", "board", "remediation_ordinal",
  "original_runtime_restored", "settings_restored", "theme_restored",
  "mineonboot_false", "mining_inactive", "mining_activity_category",
  "zero_hashrate", "zero_shares", "read_only_finalization",
  "usb_cleanup_ready", "redaction_status", "source_commit",
]);

export async function validateExactRestorationProjection(
  candidate: string,
  sourceCommit: string,
): Promise<void> {
  const value = JSON.parse(await readFile(candidate, "utf8")) as Record<string, unknown>;
  const ordinal = value["remediation_ordinal"];
  const category = value["mining_activity_category"];
  const modeValid = (ordinal === 2 && category === "paused" && value["read_only_finalization"] === true)
    || ([3, 4].includes(Number(ordinal)) && ["paused", "safe_blocked"].includes(String(category ?? ""))
      && value["read_only_finalization"] === false);
  if (Object.keys(value).length !== allowedKeys.size
    || Object.keys(value).some(key => !allowedKeys.has(key))
    || value["schema_version"] !== "bitaxe-stratum-v2-exact-restoration-v2"
    || value["status"] !== "accepted" || value["board"] !== 205
    || !modeValid || value["source_commit"] !== sourceCommit
    || value["redaction_status"] !== "passed"
    || ["original_runtime_restored", "settings_restored", "theme_restored",
      "mineonboot_false", "mining_inactive", "zero_hashrate", "zero_shares",
      "usb_cleanup_ready"].some(key => value[key] !== true)) {
    throw new Error("exact restoration projection rejected");
  }
}

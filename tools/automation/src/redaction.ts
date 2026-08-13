import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

const semanticSchemas = new Set([
  "bitaxe-hardware-attempt-v1",
  "bitaxe-correlated-runtime-evidence-v1",
  "bitaxe-substantive-evidence-v1",
  "bitaxe-version-evidence-v1",
  "bitaxe-automation-migration-v1",
  "bitaxe-settings-durability-evidence-v2",
  "bitaxe-theme-durability-evidence-v1",
  "bitaxe-system-info-evidence-v1",
  "bitaxe-ultra205-defaults-evidence-v1",
  "bitaxe-settings-patch-evidence-v1",
  "bitaxe-log-buffer-evidence-v1",
  "bitaxe-partition-layout-evidence-v1",
  "bitaxe-network-scan-evidence-v1",
  "bitaxe-asic-initialization-evidence-v1",
  "bitaxe-asic-power-initialization-evidence-v1",
  "bitaxe-core-voltage-control-evidence-v1",
  "bitaxe-ina260-evidence-v1",
  "bitaxe-emc2101-thermal-evidence-v1",
  "bitaxe-asic-reset-evidence-v1",
  "bitaxe-asic-work-send-evidence-v1",
  "bitaxe-asic-result-parsing-evidence-v1",
]);

const safeSemanticKeys = new Set([
  "exactly_one_chip_detected",
  "exactly_one_chip_detected_after_reset",
  "same_origin_api_observed",
  "same_origin_observed",
  "trusted_origin_preserved",
]);

const prohibitedKeys = new RegExp([
  "password",
  "secret",
  "token",
  "api[_-]?key",
  "ssid",
  "device[_-]?url",
  "origin",
  "mac",
  "(?:^|[_-])ip(?:v[46])?(?:$|[_-])",
  "pool(?:url|port|user|worker)",
  "owner(?:address)?",
  "btc(?:address)?",
  "usb[_-]?(?:port|path)",
  "serial[_-]?port",
].join("|"), "iu");
const localPath = /(?:\/Users\/[^\s"']+|\/home\/[^\s"']+|[A-Za-z]:\\[^\s"']+)/u;
const networkAddress = /(?:\b(?:\d{1,3}\.){3}\d{1,3}\b|\b[0-9a-f]{2}(?::[0-9a-f]{2}){5}\b|https?:\/\/)/iu;

function inspectValue(value: unknown, keyPath: string): string[] {
  if (Array.isArray(value)) return value.flatMap((item, index) => inspectValue(item, `${keyPath}[${String(index)}]`));
  if (typeof value === "object" && value !== null) {
    const violations: string[] = [];
    for (const [key, child] of Object.entries(value)) {
      if (prohibitedKeys.test(key) && !safeSemanticKeys.has(key)) {
        violations.push(`${keyPath}.${key}: prohibited operational field`);
      }
      violations.push(...inspectValue(child, `${keyPath}.${key}`));
    }
    return violations;
  }
  if (typeof value !== "string") return [];
  const violations: string[] = [];
  if (localPath.test(value)) violations.push(`${keyPath}: local path`);
  if (networkAddress.test(value)) violations.push(`${keyPath}: network address or origin`);
  return violations;
}

async function jsonFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const child = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...await jsonFiles(child));
    else if (entry.isFile() && entry.name.endsWith(".json")) files.push(child);
  }
  return files;
}

export async function verifySemanticEvidenceRedaction(root: string): Promise<{ readonly checked: number }> {
  let checked = 0;
  const violations: string[] = [];
  for (const file of await jsonFiles(root)) {
    let value: unknown;
    try {
      value = JSON.parse(await readFile(file, "utf8"));
    } catch {
      continue;
    }
    if (typeof value !== "object" || value === null) continue;
    const schema = (value as Record<string, unknown>)["schema_version"];
    if (typeof schema !== "string" || !semanticSchemas.has(schema)) continue;
    checked += 1;
    for (const violation of inspectValue(value, "$")) {
      violations.push(`${path.relative(root, file)} ${violation}`);
    }
  }
  if (violations.length > 0) throw new Error(`semantic evidence redaction failed (${String(violations.length)} violation(s))`);
  return { checked };
}

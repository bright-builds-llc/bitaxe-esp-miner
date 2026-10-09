import { validateNoiseSerialProjection } from "./noise-serial-redaction.js";
import { parseProjection as validateV2SerialProjection } from "../../../scripts/str005-v2-serial/projection.mjs";
import {
  PROJECTION_PROFILES as bwgRestorationProfiles,
  validateProjection as validateBwgRestorationProjection,
} from "../../../scripts/bwg-restoration/projection.mjs";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

const semanticSchemas = new Set([
  "str005-v2-serial-projection-v1",
  "bitaxe-stratum-v2-noise-serial-projection-v2",
  "fixed-usb-cycle-report-v1",
  "fixed-usb-window-report-v1",
  "bitaxe-hardware-attempt-v1",
  "bitaxe-correlated-runtime-evidence-v1",
  "bitaxe-substantive-evidence-v1",
  "bitaxe-version-evidence-v1",
  "bitaxe-automation-migration-v1",
  "bitaxe-settings-durability-evidence-v2",
  "bitaxe-theme-durability-evidence-v1",
  "bitaxe-system-info-evidence-v1",
  "bitaxe-adc-observation-evidence-v1",
  "bitaxe-ultra205-defaults-evidence-v1",
  "bitaxe-settings-patch-evidence-v1",
  "bitaxe-log-buffer-evidence-v1",
  "bitaxe-scoreboard-evidence-v2",
  "bitaxe-partition-layout-evidence-v1",
  "bitaxe-network-scan-evidence-v1",
  "bitaxe-asic-initialization-evidence-v1",
  "bitaxe-asic-power-initialization-evidence-v1",
  "bitaxe-core-voltage-control-evidence-v1",
  "bitaxe-ina260-evidence-v1",
  "bitaxe-ina260-evidence-v2",
  "bitaxe-emc2101-thermal-evidence-v1",
  "bitaxe-asic-reset-evidence-v1",
  "bitaxe-asic-work-send-evidence-v1",
  "bitaxe-asic-result-parsing-evidence-v1",
  "bitaxe-ui-workflow-evidence-v1",
  "bitaxe-otawww-evidence-v1",
]);

/**
 * Evidence identified by a top-level `profile` rather than a schema: the BWG restoration projections
 * (bwg-worker-restoration-result/0.2, /0.3 and /0.4). Each is also checked by its own closed validator.
 */
const semanticProfiles = new Set<string>(bwgRestorationProfiles);

const safeSemanticKeys = new Set([
  "exactly_one_chip_detected",
  "exactly_one_chip_detected_after_reset",
  // A BWG restoration fact: the restore-watcher token was observed before the restore instruction; it carries no token.
  "restoreTokenBeforeRestoreInstruction",
  "same_origin_api_observed",
  "same_origin_observed",
  "same_origin_requests_observed",
  "trusted_origin_preserved",
  "write_only_secrets_blank",
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

function inspectValue(value: unknown, keyPath: string, workerPrivacy = false): string[] {
  if (Array.isArray(value)) return value.flatMap((item, index) => inspectValue(item, `${keyPath}[${String(index)}]`, workerPrivacy));
  if (typeof value === "object" && value !== null) {
    const violations: string[] = [];
    for (const [key, child] of Object.entries(value)) {
      if (prohibitedKeys.test(key) && !safeSemanticKeys.has(key)) {
        violations.push(`${keyPath}.${key}: prohibited operational field`);
      }
      if (workerPrivacy && /(?:device_identity|settings|authorization_high_water).*sha256/iu.test(key)) {
        violations.push(`${keyPath}.${key}: private Worker fingerprint`);
      }
      violations.push(...inspectValue(child, `${keyPath}.${key}`, workerPrivacy));
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

/** The registered schema or profile that identifies a semantic evidence file, or `undefined` when unregistered. */
function registeredIdentity(fields: Record<string, unknown>): string | undefined {
  const schema = fields["schema_version"] ?? fields["schema"];
  if (typeof schema === "string" && semanticSchemas.has(schema)) return schema;
  const profile = fields["profile"];
  if (typeof profile === "string" && semanticProfiles.has(profile)) return profile;
  return undefined;
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
    const fields = value as Record<string, unknown>;
    const schema = registeredIdentity(fields);
    if (schema === undefined) continue;
    if (schema === "bitaxe-stratum-v2-noise-serial-projection-v2") validateNoiseSerialProjection(fields);
    if (schema === "str005-v2-serial-projection-v1") validateV2SerialProjection(fields);
    if (semanticProfiles.has(schema)) validateBwgRestorationProjection(fields);
    checked += 1;
    for (const violation of inspectValue(value, "$", schema.startsWith("fixed-usb-"))) {
      violations.push(`${path.relative(root, file)} ${violation}`);
    }
  }
  if (violations.length > 0) throw new Error(`semantic evidence redaction failed (${String(violations.length)} violation(s))`);
  return { checked };
}

// Seal-time credential scan and the sealed result (ADR-0036). Finish scans every file in the attempt root for the
// exact pool values and for credential shapes before it writes `result.json`; only counts leave this module.
import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { readPoolForSigning } from "../fixed-usb-qualification/authority.mjs";
import { digest, readJson, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { CAMPAIGN_RESULT, RESULT_SCHEMA } from "./contract.mjs";

/** Shorter values (a pool password such as `x`) occur everywhere; matching them proves nothing. */
export const MINIMUM_SECRET_LENGTH = 8;

/** Random base64url material mixes all three classes; closed tokens and lower-case hex digests do not. */
const mixed = (text) => /[A-Z]/u.test(text) && /[a-z]/u.test(text) && /[0-9]/u.test(text);
const HEX_DIGEST = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/u;

/** Credential shapes the Gate and authority produce: compact JWS, long base64url runs, challenge and lease ids. */
const SHAPES = Object.freeze([
  { pattern: /[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}/gu, admit: mixed },
  { pattern: /[A-Za-z0-9_-]{40,}/gu, admit: (run) => !HEX_DIGEST.test(run) && mixed(run) },
  // A random base64url suffix has an upper-case letter or a hyphen; closed lower-case tokens such as lease_start_failed do not.
  { pattern: /(?<![A-Za-z0-9_-])(?:challenge|lease)_[A-Za-z0-9_-]{16,}/gu, admit: (match) => /[A-Z-]/u.test(match.slice(match.indexOf("_") + 1)) },
]);

/** Count credential-shaped matches in one file's text. */
export function shapeHits(text) {
  return SHAPES.reduce((total, { pattern, admit }) => total + [...text.matchAll(pattern)].filter(([match]) => admit(match)).length, 0);
}

/**
 * The exact pool values to look for, raw and JSON-escaped. They stay in memory for the scan and are never
 * written or printed.
 */
export async function poolScanValues(firmwareRoot, path) {
  const pool = await readPoolForSigning(firmwareRoot, path);
  const endpoint = new URL(pool.endpoint);
  const values = [pool.endpoint, endpoint.host, endpoint.hostname, pool.username, pool.password];
  return [...new Set(values.flatMap((value) => [value, JSON.stringify(value).slice(1, -1)]))].filter((value) => value.length >= MINIMUM_SECRET_LENGTH);
}

/** Scan every regular file below `root`; anything other than a file or directory is refused. */
export async function scanAttemptRoot(root, values) {
  const needles = values.map((value) => Buffer.from(value, "utf8"));
  const scan = { files: 0, hits: 0 };
  async function visit(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) { await visit(path); continue; }
      requireCondition(entry.isFile(), "credential_scan_entry");
      const bytes = await readFile(path);
      scan.files += 1;
      scan.hits += needles.filter((needle) => bytes.includes(needle)).length + shapeHits(bytes.toString("latin1"));
    }
  }
  await visit(root);
  return scan;
}

/** Serve's campaign verdict, or `completion_missing` when the campaign never finished. */
export async function campaignResult(root, context) {
  const maybeRecord = await readJson(resolve(root, CAMPAIGN_RESULT)).catch((error) => {
    if (error.code === "ENOENT") return null;
    throw error;
  });
  if (maybeRecord === null) {
    return { schema: RESULT_SCHEMA, attempt: context.attempt, result: "unverified", failure: { scenario: null, category: "completion_missing" },
      scenarios: [], parity_promotion: false };
  }
  requireCondition(maybeRecord.result?.schema === RESULT_SCHEMA && maybeRecord.sha256 === digest(JSON.stringify(maybeRecord.result)),
    "restoration_campaign_result_invalid");
  return maybeRecord.result;
}

/**
 * The sealed result: the campaign verdict plus the scan counts. A hit makes it unverified; the campaign's own
 * earlier failure keeps precedence, otherwise the failure is `credential_in_evidence`.
 */
export function sealedResult(campaign, scan) {
  const leaked = scan.hits > 0;
  return { ...campaign, result: leaked ? "unverified" : campaign.result,
    failure: campaign.failure ?? (leaked ? { scenario: null, category: "credential_in_evidence" } : null), credential_scan: scan };
}

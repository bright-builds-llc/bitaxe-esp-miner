// Closed public projections for BWG-007 (ADR-0019 batch-publication rules, ADR-0036): publish writes
// `bwg-worker-restoration-result/0.3`, whose booleans are derived from measured judge facts and the seal-time
// credential scan; the 0.2 validator stays so the published attempt-008 files still validate.
import { randomBytes } from "node:crypto";
import { link, lstat, open, unlink } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { QualificationError, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { SCENARIO_PLANS, SCENARIOS } from "./contract.mjs";

export const PROJECTION_PROFILE = "bwg-worker-restoration-result/0.3";
export const LEGACY_PROJECTION_PROFILE = "bwg-worker-restoration-result/0.2";
export const PROJECTION_DIRECTORY = "docs/parity/evidence/bwg-worker-restoration";
const COMMIT_FIELDS = ["firmwareCommit", "gateCommit", "referenceCommit"];
const DIGEST_FIELDS = ["appElfSha256", "packageManifestSha256", "gateBundleSha256", "gatePageSha256", "trustSha256", "recordsSha256", "scenarioResultSha256"];
const LEGACY_BOOLEAN_FIELDS = ["baselineConfirmed", "cleanupConfirmed", "campaignEventCredentialsAbsent", "sameDeviceAcrossScenarios"];
const BOOLEAN_FIELDS = [...LEGACY_BOOLEAN_FIELDS, "poolConfigurationNeverPersisted"];
const PHYSICAL_FACTS = ["watcherBeforeRemovalInstruction", "removalObservedByWatcher", "absenceBoundMet", "restoreTokenBeforeRestoreInstruction",
  "enumerationChanged", "samePhysicalIdentity", "stableBeforeReconnect", "rearmsWithinCap", "pageOrderConfirmed", "deviceEndedLease"];

/** The 0.2 facts each scenario published (attempt-008); kept only to validate those files. */
export const LEGACY_FACT_ALLOWLIST = Object.freeze({
  completion: ["operatorEndedLease", "renewalAccepted", "stimulusCounterConsistent"],
  pause: ["operatorEndedLease", "stimulusCounterConsistent"],
  cancel: ["operatorEndedLease", "stimulusCounterConsistent"],
  expiry: ["noRenewal", "deviceEndedLease", "expiredAfterWindow", "noOperatorStop", "stimulusCounterConsistent"],
  monotonic_uncertainty: ["stimulusIdleBefore", "stimulusAcknowledged", "deviceEndedLease", "noRelabel", "endedWithinBound", "sameSession",
    "monotonicDetectionCounted", "stimulusCounterConsistent"],
  disconnect: [...PHYSICAL_FACTS, "stimulusCounterConsistent"],
  reboot: [...PHYSICAL_FACTS, "rebootClearedStimulus", "highWaterNotAdvancedAfterReboot", "preRebootStatusObserved", "stimulusCounterConsistent"],
  authorization_negatives: ["rebootReportedBeforeN1", "durableReplayAttributed", "expiredContextAttributed", "crossContextAttributed", "renewalReplayAttributed",
    "highWaterUnchangedAcrossReboot", "rejectedStartsNeverStarted", "stimulusCounterConsistent"],
});

/** The only facts each scenario may publish in 0.3; all must be `true` in a published projection. */
export const FACT_ALLOWLIST = Object.freeze(Object.fromEntries(SCENARIOS.map((scenario) => [scenario, Object.freeze([
  ...LEGACY_FACT_ALLOWLIST[scenario], "deviceIdentityStable", "poolConfigurationUnchanged",
  ...(["disconnect", "reboot"].includes(scenario) ? ["sameKeyReacquired"] : []),
])])));

const PROFILES = Object.freeze({
  [LEGACY_PROJECTION_PROFILE]: { booleans: LEGACY_BOOLEAN_FIELDS, facts: LEGACY_FACT_ALLOWLIST },
  [PROJECTION_PROFILE]: { booleans: BOOLEAN_FIELDS, facts: FACT_ALLOWLIST },
});
const keysOf = (booleans) => ["profile", "attemptId", "scenario", "outcome", "terminalReason", ...COMMIT_FIELDS, ...DIGEST_FIELDS, ...booleans, "facts"].sort();
const SECRET_LOOKING = /password|username|user=|endpoint|credential|challenge|lease_|authorization|jwk|stratum|pool|fingerprint|serial|\/dev\/|:\/\/|@|bearer|token|\.bitaxe|bc1|nonce/iu;

export function canonicalJson(value) {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
}

const CLOSED_STRINGS = new Set([...Object.keys(PROFILES), "complete", ...SCENARIOS, ...SCENARIOS.map((scenario) => SCENARIO_PLANS[scenario].terminal)]);
const HEX = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/u;

/** Every free string, at any depth, must not look like a credential, endpoint, identifier or device path. */
function requireNothingSecretLooking(value) {
  if (typeof value === "string") {
    requireCondition(CLOSED_STRINGS.has(value) || HEX.test(value) || !SECRET_LOOKING.test(value), "projection_private_value");
    return;
  }
  if (value === null || typeof value !== "object") return;
  // Keys are closed by the exact field and fact allowlists below; values are scanned here.
  for (const item of Object.values(value)) requireNothingSecretLooking(item);
}

/** Closed validator for either profile: exact keys, closed values, allowlisted digests, booleans only, nothing secret-looking. */
export function validateProjection(value) {
  requireNothingSecretLooking(value);
  const profile = PROFILES[value?.profile];
  requireCondition(profile !== undefined, "projection_identity");
  requireCondition(value !== null && typeof value === "object" && !Array.isArray(value) &&
    canonicalJson(Object.keys(value).sort()) === canonicalJson(keysOf(profile.booleans)), "projection_fields");
  requireCondition(value.outcome === "complete" && /^bwg007-attempt-[0-9]{3}$/u.test(value.attemptId) &&
    SCENARIOS.includes(value.scenario) && value.terminalReason === SCENARIO_PLANS[value.scenario].terminal, "projection_identity");
  requireCondition(COMMIT_FIELDS.every((field) => /^[0-9a-f]{40}$/u.test(value[field])) && DIGEST_FIELDS.every((field) => /^[0-9a-f]{64}$/u.test(value[field])),
    "projection_digests");
  requireCondition(profile.booleans.every((field) => value[field] === true), "projection_booleans");
  const facts = value.facts, allowed = profile.facts[value.scenario];
  requireCondition(facts !== null && typeof facts === "object" && !Array.isArray(facts) &&
    canonicalJson(Object.keys(facts).sort()) === canonicalJson([...allowed].sort()) && Object.values(facts).every((fact) => fact === true), "projection_facts");
  return value;
}

/**
 * Attempt-wide booleans, measured rather than asserted: the same device identity throughout (every scenario's
 * `deviceIdentityStable` and epoch 1 in the last final state) and a seal-time scan that found no credential.
 */
export function attemptFacts(scenarioResults, sealedResult) {
  const scan = sealedResult.credential_scan;
  return {
    sameDeviceAcrossScenarios: scenarioResults.length === SCENARIOS.length && scenarioResults.every((result) => result.facts?.deviceIdentityStable === true) &&
      scenarioResults.at(-1)?.final_state?.deviceIdentity?.epoch === 1,
    campaignEventCredentialsAbsent: Number.isSafeInteger(scan?.files) && scan.files > 0 && scan.hits === 0,
  };
}

/** Build one scenario's 0.3 projection from its sealed private result, the attempt facts and the frozen context. */
export function buildProjection({ attemptId, context, scenarioResult, recordsSha256, scenarioResultSha256, attempt }) {
  const facts = Object.fromEntries(FACT_ALLOWLIST[scenarioResult.scenario].map((name) => [name, scenarioResult.facts[name]]));
  return validateProjection({
    profile: PROJECTION_PROFILE, attemptId, scenario: scenarioResult.scenario, outcome: "complete", terminalReason: scenarioResult.terminal_reason,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, referenceCommit: context.reference_commit,
    appElfSha256: context.app_elf_sha256, packageManifestSha256: context.manifest_sha256, gateBundleSha256: context.gate_bundle_sha256,
    gatePageSha256: context.gate_page_sha256, trustSha256: context.trust_sha256, recordsSha256, scenarioResultSha256,
    baselineConfirmed: scenarioResult.checks?.baselineConfirmed === true, cleanupConfirmed: scenarioResult.checks?.cleanupConfirmed === true,
    campaignEventCredentialsAbsent: attempt.campaignEventCredentialsAbsent === true, sameDeviceAcrossScenarios: attempt.sameDeviceAcrossScenarios === true,
    poolConfigurationNeverPersisted: scenarioResult.facts.poolConfigurationUnchanged === true, facts,
  });
}

/** Exclusive write through a random temporary name and a hard link, so a target never appears half-written. */
export async function writeAtomicNew(path, text, mode, operations = {}) {
  const temporary = resolve(dirname(path), `.${basename(path)}.${randomBytes(16).toString("hex")}`);
  const file = await (operations.open ?? open)(temporary, "wx", mode);
  try { await file.chmod(mode); await file.writeFile(text); await file.sync(); } finally { await file.close(); }
  try { await (operations.link ?? link)(temporary, path); } finally { await unlink(temporary).catch(() => undefined); }
}

/**
 * Publish the complete set or nothing: every target must be absent, every projection valid, and any failure
 * removes the targets this call already wrote.
 */
export async function publishProjectionSet(directory, projections, operations = {}) {
  requireCondition(projections.length === SCENARIOS.length && SCENARIOS.every((scenario, index) => projections[index]?.scenario === scenario),
    "projection_set_incomplete");
  for (const projection of projections) validateProjection(projection);
  requireCondition(new Set(projections.map((projection) => canonicalJson({ ...projection, scenario: null, terminalReason: null, facts: null,
    recordsSha256: null, scenarioResultSha256: null }))).size === 1, "projection_identity_drift");
  const targets = projections.map((projection) => resolve(directory, `${projection.attemptId}-${projection.scenario}.json`));
  for (const target of targets) {
    try { await lstat(target); } catch (error) { if (error.code === "ENOENT") continue; throw error; }
    throw new QualificationError("projection_exists");
  }
  const written = [];
  try {
    for (const [index, target] of targets.entries()) {
      await writeAtomicNew(target, `${canonicalJson(projections[index])}\n`, 0o644, operations);
      written.push(target);
    }
  } catch (error) {
    const cleanup = await Promise.allSettled(written.map((target) => unlink(target)));
    if (cleanup.some((outcome) => outcome.status === "rejected")) throw new QualificationError("projection_rollback_failed");
    throw error instanceof QualificationError ? error : new QualificationError("projection_publication_failed");
  }
  return targets;
}

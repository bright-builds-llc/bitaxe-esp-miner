// Mirrors of the Gate restoration page's parsers for the host routes it calls (bitaxe-turnstile-system
// web/worker-acceptance-activation.ts, web/worker-restoration-operations.ts, web/worker-restoration-qualification.ts,
// web/worker-controller-semantics.ts). The Gate pin does not carry these files yet, so the host tests check
// the same exact shapes; the lead re-runs them against the real parsers once the pin moves.
const SCENARIOS = ["completion", "pause", "cancel", "expiry", "monotonic_uncertainty", "disconnect", "reboot", "authorization_negatives"];

function exact(value, required, optional = []) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("record");
  if (!required.every((key) => Object.hasOwn(value, key)) || Object.keys(value).some((key) => !required.includes(key) && !optional.includes(key))) throw new Error("fields");
  return value;
}

export function parseActivation(value) {
  exact(value, ["challengeId", "retentionExpiryUnixSeconds"]);
  if (typeof value.challengeId !== "string" || !Number.isSafeInteger(value.retentionExpiryUnixSeconds)) throw new Error("activation_invalid");
  return value;
}

export function parseCheckpoint(value) {
  exact(value, ["checkpoint"]);
  if (!/^[a-z][a-z0-9_]{0,63}$/u.test(value.checkpoint)) throw new Error("physical_window_state");
  return value;
}

export function parseCompletionNonce(value) {
  exact(value, ["nonce"]);
  if (typeof value.nonce !== "string") throw new Error("completion_context");
  return value.nonce;
}

export function parseCompletion(value) {
  exact(value, ["result", "scenario", "cleanup_confirmed"]);
  if (!["passed", "unverified"].includes(value.result) || !SCENARIOS.includes(value.scenario) || value.cleanup_confirmed !== true) throw new Error("completion_receipt");
  return value;
}

const maximumRenewals = (value) => value.durationMilliseconds === 60000 && value.renewAfterMilliseconds === 20000 ? 1
  : value.durationMilliseconds === 30000 && value.renewAfterMilliseconds === 10000 ? 0 : undefined;

export function parseGrant(value) {
  exact(value, ["protocolVersion", "leaseId", "challengeId", "authorization", "durationMilliseconds", "renewAfterMilliseconds", "stratum"],
    ["acceptanceCampaign", "qualificationAttempt", "hardwareProfile", "soakAllowance"]);
  exact(value.stratum, ["endpoint", "username", "password"], ["suggestedDifficulty"]);
  if (value.protocolVersion !== "bwg-worker-controller/0.4") throw new Error("protocol");
  if (value.acceptanceCampaign || value.qualificationAttempt || value.soakAllowance) throw new Error("restoration_grant_budgeted");
  if ((value.hardwareProfile ?? "conservative") !== "conservative") throw new Error("restoration_grant_profile");
  if (maximumRenewals(value) === undefined) throw new Error("restoration_window");
  return value;
}

export function parseRenewal(value) {
  exact(value, ["protocolVersion", "leaseId", "authorization", "durationMilliseconds", "renewAfterMilliseconds"]);
  if (maximumRenewals(value) !== 1) throw new Error("restoration_renewal_window");
  return value;
}

export function parseWindow(value) {
  exact(value, ["grant", "renewals"]);
  const grant = parseGrant(value.grant);
  if (!Array.isArray(value.renewals) || value.renewals.length > maximumRenewals(grant)) throw new Error("restoration_renewal_bound");
  if (value.renewals.map(parseRenewal).some((renewal) => renewal.leaseId !== grant.leaseId)) throw new Error("renewal_lease_mismatch");
  return value;
}

export function parseReplay(value) {
  if (value?.operation === "start") return parseGrant(exact(value, ["operation", "grant"]).grant) && value;
  if (value?.operation === "renew") return parseRenewal(exact(value, ["operation", "renewal"]).renewal) && value;
  throw new Error("replay_artifact_invalid");
}

/** The page configures only from exactly the four ordinary keys plus `restorationQualification: true`. */
export function parseRestorationConfiguration(value) {
  exact(value, ["expectedGateCommit", "expectedFirmwareSourceCommit", "expectedAppElfSha256", "trust", "restorationQualification"]);
  if (value.restorationQualification !== true || !/^[0-9a-f]{40}$/u.test(value.expectedFirmwareSourceCommit) || !/^[0-9a-f]{64}$/u.test(value.expectedAppElfSha256)) {
    throw new Error("configuration_invalid");
  }
  return value;
}

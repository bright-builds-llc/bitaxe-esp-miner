const DIGEST_FIELDS = [
  "appElfSha256", "eventsSha256", "gateBundleSha256", "packageManifestSha256",
  "restoreBundleSha256", "runtimeAttestationSha256",
];
const GATE_COMMIT = "0b07d36942aa8ca3473771d2f72a373e66cedf58";
const TERMINAL_REASONS = {
  completion: ["challenge_satisfied"],
  pause: ["paused"],
  cancel: ["cancelled"],
  expiry: ["lease_expired"],
  disconnect: ["connectivity_lost"],
  reboot: ["reboot"],
  monotonic_uncertainty: ["monotonic_reset"],
  authorization_negatives: ["control_failed"],
};

export function validatePublicProjection(value) {
  const keys = [
    "appElfSha256", "attemptId", "baselineConfirmed", "campaignEventCredentialsAbsent",
    "cleanupConfirmed", "eventsSha256", "firmwareCommit", "gateBundleSha256", "gateCommit",
    "gateProfileCommit", "outcome", "packageManifestSha256", "profile", "restoreBundleSha256",
    "runtimeAttestationSha256", "sameDeviceAcrossScenarios", "scenario", "terminalReason",
  ];
  if (
    typeof value !== "object" || value === null || Array.isArray(value) ||
    Object.keys(value).sort().join(",") !== keys.sort().join(",") ||
    value.profile !== "bwg-worker-restoration-result/0.1" || value.outcome !== "complete" ||
    !/^bwg007-attempt-[0-9]{3}$/.test(value.attemptId) ||
    !/^[0-9a-f]{40}$/.test(value.firmwareCommit) ||
    value.gateCommit !== GATE_COMMIT || value.gateProfileCommit !== GATE_COMMIT ||
    DIGEST_FIELDS.some((field) => !/^[0-9a-f]{64}$/.test(value[field])) ||
    value.baselineConfirmed !== true || value.cleanupConfirmed !== true ||
    value.campaignEventCredentialsAbsent !== true || value.sameDeviceAcrossScenarios !== true ||
    !Object.hasOwn(TERMINAL_REASONS, value.scenario) ||
    !TERMINAL_REASONS[value.scenario].includes(value.terminalReason)
  ) throw new Error("public_projection_invalid");
  if (Object.values(value).filter((item) => typeof item === "string").some((item) =>
    /password|username|endpoint|credential|challengeId|leaseId|jwk|fingerprint|serial|port/i
      .test(item))) throw new Error("public_projection_private_field");
  return value;
}

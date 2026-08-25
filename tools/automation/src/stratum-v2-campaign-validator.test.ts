import assert from "node:assert/strict";
import test from "node:test";

import { validateStratumV2CampaignProjection } from "./stratum-v2-campaign-validator.js";

const source = "a".repeat(40);
const reference = "b".repeat(40);
const manifest = "c".repeat(64);

function validProjection(): Record<string, unknown> {
  return {
    schema_version: "bitaxe-stratum-v2-campaign-projection-v1",
    status: "accepted",
    board: 205,
    source_commit: source,
    reference_commit: reference,
    package_manifest_sha256: manifest,
    fixture_accepted: true,
    share_target_valid: true,
    safe_stop_complete: true,
    settings_restored: true,
    package_restored: true,
    mineonboot_false: true,
    usb_cleanup_ready: true,
    redaction_status: "passed",
    exact_non_claims: [
      "external_production_pool", "mixed_protocol_live_fallback", "other_boards",
      "unbounded_mining", "ota", "release_readiness",
    ],
  };
}

test("validator accepts the exact closed projection", () => {
  // Arrange
  const projection = validProjection();

  // Act / Assert
  assert.doesNotThrow(() =>
    validateStratumV2CampaignProjection(projection, source, reference, manifest));
});

test("validator rejects secret fields, missing proofs, and identity drift", () => {
  // Arrange
  const secret = { ...validProjection(), poolPassword: "canary" };
  const incomplete = { ...validProjection(), safe_stop_complete: false };
  const drifted = validProjection();

  // Act / Assert
  assert.throws(() =>
    validateStratumV2CampaignProjection(secret, source, reference, manifest));
  assert.throws(() =>
    validateStratumV2CampaignProjection(incomplete, source, reference, manifest));
  assert.throws(() =>
    validateStratumV2CampaignProjection(drifted, "e".repeat(40), reference, manifest));
});

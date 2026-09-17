import assert from "node:assert/strict";
import test from "node:test";
import { signV2Attempt } from "./signing.mjs";
import { PROFILE } from "./values.mjs";

const attempt = { schema: "worker-qualification-attempt-v1", id: Buffer.alloc(16, 1).toString("base64url"),
  ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 };
const stratum = { profile: PROFILE, endpoint: "stratum+tcp://192.168.20.3:33000/",
  authorityPublicKey: Buffer.alloc(32, 2).toString("base64url"), userIdentity: "synthetic-user" };
const options = { attempt, stratum, challengeId: "challenge_synthetic", binding: Buffer.alloc(32, 3).toString("base64url"), failed: () => false };

test("V2 initial grant and renewals retain the frozen60s/20s windows", async () => {
  // Arrange
  const operations = [];
  const sign = async (operation, input) => { operations.push(input); return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "synthetic-signature" }; };
  // Act
  const result = await signV2Attempt({ ...options, sign });
  // Assert
  assert.equal(operations.length, 10);
  assert.equal(result.grant.qualificationAttempt.ordinal, 18);
  assert.equal(result.grant.renewAfterMilliseconds, 20000);
  assert.deepEqual(result.grant.stratum, stratum);
  assert.equal(result.renewals.length, 9);
  for (const renewal of result.renewals) {
    assert.equal(renewal.durationMilliseconds, 60000); assert.equal(renewal.renewAfterMilliseconds, 20000);
    assert.equal(Object.hasOwn(renewal, "stratum"), false);
    assert.equal(renewal.leaseId, result.grant.leaseId);
  }
});

test("failure during an awaited signature prevents every subsequent signing effect", async () => {
  // Arrange
  let failed = false, calls = 0;
  const sign = async (operation) => { calls++; failed = true;
    return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "synthetic-signature" }; };
  // Act / Assert
  await assert.rejects(signV2Attempt({ ...options, sign, failed: () => failed }), { code: "v2_terminal_failure" });
  assert.equal(calls, 1);
});

test("diagnostic or altered reservation types cannot use the V2 signer", async () => {
  let calls = 0;
  await assert.rejects(signV2Attempt({ ...options, attempt: { ...attempt, purpose: "diagnostic", maximumActiveMilliseconds: 30000 },
    sign: async () => { calls++; } }), { code: "v2_signing_context" });
  assert.equal(calls, 0);
});

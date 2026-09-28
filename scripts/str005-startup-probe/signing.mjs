import { nonce } from '../fixed-usb-qualification/contract.mjs';
import { validateAttempt, validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { bytes, check, parseStratum } from '../str005-v2-serial/values.mjs';
export function freshAttempt(ledger) {
  validateLedger(ledger);
  check(!ledger.pending && ledger.last_completed_ordinal + 1 === ledger.next_ordinal, 'startup_ledger_pending');
  return validateAttempt({ schema: 'worker-qualification-attempt-v1', id: nonce(), ordinal: ledger.next_ordinal,
    purpose: 'normal', maximumActiveMilliseconds: 180000 });
}
/** A durable issuance claim belongs to the caller; no renewal can be produced here. */
export async function signStart({ attempt, challengeId, binding, stratum, sign, verify }) {
  validateAttempt(attempt); bytes(binding, 32); parseStratum(stratum);
  check(attempt.purpose === 'normal' && /^challenge_[A-Za-z0-9_-]{1,118}$/u.test(challengeId), 'startup_signing_context');
  await verify();
  const request = { protocolVersion: 'bwg-worker-controller/0.4', leaseId: `lease_${nonce()}`, challengeId,
    durationMilliseconds: 60000, renewAfterMilliseconds: 20000, stratum, qualificationAttempt: attempt };
  const artifact = await sign('start', { operation: 'start', activeChallengeId: challengeId, controlSessionBindingSha256: binding, request });
  await verify();
  check(artifact.profile === 'bwg-worker-lease-authorization-artifact/0.1' && artifact.operation === 'start' &&
    typeof artifact.authorization === 'string' && artifact.authorization.length > 0 && artifact.authorization.length <= 8192, 'startup_signature_shape');
  return { grant: { ...request, authorization: artifact.authorization }, renewals: [] };
}

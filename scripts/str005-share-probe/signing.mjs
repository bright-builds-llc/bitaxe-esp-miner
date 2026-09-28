import { signStart } from '../str005-startup-probe/signing.mjs';
import { check } from '../str005-v2-serial/values.mjs';
/** One durable issuance claim; at most two same-lease renewals, never a new allowance. */
export async function signWindow(input) {
  const window = await signStart(input);
  const { protocolVersion, leaseId, durationMilliseconds, renewAfterMilliseconds } = window.grant;
  const request = { protocolVersion, leaseId, durationMilliseconds, renewAfterMilliseconds };
  for (let index = 0; index < 2; index++) {
    await input.verify();
    const artifact = await input.sign('renew', { operation: 'renew', activeChallengeId: input.challengeId,
      controlSessionBindingSha256: input.binding, request });
    await input.verify();
    check(artifact.profile === 'bwg-worker-lease-authorization-artifact/0.1' && artifact.operation === 'renew' &&
      typeof artifact.authorization === 'string' && artifact.authorization.length > 0 && artifact.authorization.length <= 8192, 'share_signature_shape');
    window.renewals.push({ ...request, authorization: artifact.authorization });
  }
  return window;
}

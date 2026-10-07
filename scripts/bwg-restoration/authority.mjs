// Sign one unbudgeted Conservative Stratum V1 restoration window through the Gate development authority
// (scripts/worker-development-authority.ts sign-start / sign-renew). Inputs and outputs travel over the
// child's stdin and stdout, so no signing intermediate ever reaches disk.
import { canonicalBase64, nonce, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { ARTIFACT_PROFILE, PROTOCOL_VERSION, WINDOWS } from "./contract.mjs";

const CHALLENGE = /^challenge_[A-Za-z0-9_-]{1,118}$/u;

/**
 * `window` is `standard` (60,000/20,000 ms) or `expiry` (30,000/10,000 ms); only `standard` may carry its
 * single renewal. The grant names no campaign, attempt, soak allowance or hardware profile.
 */
export async function signRestorationWindow({ window, renewals, challengeId, binding, stratum, sign }) {
  const shape = WINDOWS[window];
  requireCondition(shape !== undefined && Number.isInteger(renewals) && renewals >= 0 && renewals <= (window === "standard" ? 1 : 0), "restoration_window");
  requireCondition(CHALLENGE.test(challengeId) && canonicalBase64(binding, 32), "signing_context");
  const common = { protocolVersion: PROTOCOL_VERSION, leaseId: `lease_${nonce()}`, ...shape };
  const attach = async (operation, request) => {
    const artifact = await sign(operation, { operation, activeChallengeId: challengeId, controlSessionBindingSha256: binding, request });
    requireCondition(artifact?.profile === ARTIFACT_PROFILE && artifact.operation === operation && typeof artifact.authorization === "string" &&
      artifact.authorization.length > 0 && artifact.authorization.length <= 8192, "authorization_shape");
    return { ...request, authorization: artifact.authorization };
  };
  const grant = await attach("start", { ...common, challengeId, stratum });
  const signedRenewals = [];
  for (let index = 0; index < renewals; index += 1) signedRenewals.push(await attach("renew", { ...common }));
  return { grant, renewals: signedRenewals };
}

// Sign one soak grant and its renewals in a single batch (firmware ADR-0033, decision 6).
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { nonce } from "../fixed-usb-qualification/contract.mjs";
import { SOAK_RENEWALS } from "./contract.mjs";

const ARTIFACT_PROFILE = "bwg-worker-lease-authorization-artifact/0.1";

export async function signSoak({ allowance, challengeId, binding, stratum, sign }) {
  const common = { protocolVersion: "bwg-worker-controller/0.4", leaseId: `lease_${nonce()}`, durationMilliseconds: 60000, renewAfterMilliseconds: 20000 };
  const attach = async (operation, request) => {
    const artifact = await sign(operation, { operation, activeChallengeId: challengeId, controlSessionBindingSha256: binding, request });
    requireCondition(artifact.profile === ARTIFACT_PROFILE && artifact.operation === operation && typeof artifact.authorization === "string" &&
      artifact.authorization.length > 0 && artifact.authorization.length <= 8192, "authorization_shape");
    return { ...request, authorization: artifact.authorization };
  };
  const grant = await attach("start", { ...common, challengeId, stratum, hardwareProfile: "upstream-default", soakAllowance: allowance });
  const renewals = [];
  for (let index = 0; index < SOAK_RENEWALS; index += 1) renewals.push(await attach("renew", { ...common }));
  return { grant, renewals };
}

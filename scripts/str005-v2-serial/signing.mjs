import { resolve } from "node:path";
import { authorityCall } from "../fixed-usb-qualification/authority.mjs";
import { nonce, protectedPath } from "../fixed-usb-qualification/contract.mjs";
import { validateAttempt } from "../fixed-usb-qualification/iterative-contract.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { bytes, check, parseStratum, sha256 } from "./values.mjs";

/** Private pipes only; exit receipts contain no grant, user, endpoint or secret-derived digest. */
export async function createSigner(root, context, directory, failed, operations = {}) {
  check(context.scope === "share", "v2_signing_scope");
  await protectedPath(directory, true);
  const contextSha256 = sha256(JSON.stringify(context)); let calls = 0;
  return async (operation, input) => {
    check(!failed() && ["public-trust", "start", "renew"].includes(operation), "v2_signing_admission");
    const index = ++calls;
    check(index <= 11, "v2_signing_bound");
    const command = operation === "public-trust" ? operation : `sign-${operation}`;
    let output;
    try { output = await (operations.authorityCall ?? authorityCall)(context.gate_root, directory, command, input, "bun", {
      maybeEnvironment: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C" },
      maybeObserveExit: (observation) => writeNew(resolve(root, `signer-${String(index).padStart(2, "0")}.exit.json`), {
        schema: "str005-v2-signer-exit-v1", contextSha256, index, operation, observation,
      }),
    }); } catch (error) {
      const code = ["authority_unavailable", "authority_failed", "authority_output", "authority_exit_evidence"].includes(error.code)
        ? `v2_${error.code}` : "v2_signer_failed";
      check(false, code);
    }
    check(!failed(), "v2_terminal_failure"); return output;
  };
}

/** Exactly one normal allowance; the caller consumes the durable issuance claim first. */
export async function signV2Attempt({ attempt, challengeId, binding, stratum, sign, failed }) {
  validateAttempt(attempt); bytes(binding, 32); parseStratum(stratum);
  check(attempt.purpose === "normal" && attempt.maximumActiveMilliseconds === 180000 &&
    /^challenge_[A-Za-z0-9_-]{1,118}$/u.test(challengeId), "v2_signing_context");
  const common = { protocolVersion: "bwg-worker-controller/0.4", leaseId: `lease_${nonce()}`,
    durationMilliseconds: 60000, renewAfterMilliseconds: 20000 };
  const attach = async (operation, request) => {
    check(!failed(), "v2_terminal_failure");
    const artifact = await sign(operation, { operation, activeChallengeId: challengeId,
      controlSessionBindingSha256: binding, request });
    check(!failed(), "v2_terminal_failure");
    check(artifact.profile === "bwg-worker-lease-authorization-artifact/0.1" && artifact.operation === operation &&
      typeof artifact.authorization === "string" && artifact.authorization.length > 0 && artifact.authorization.length <= 8192,
    "v2_authorization_shape");
    return { ...request, authorization: artifact.authorization };
  };
  const grant = await attach("start", { ...common, challengeId, stratum, qualificationAttempt: attempt });
  const renewals = [];
  for (let index = 0; index < 9; index++) renewals.push(await attach("renew", common));
  return { grant, renewals };
}

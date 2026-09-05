import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { canonicalBase64, exactObject, ignored, nonce, protectedPath, QualificationError, requireCondition, WINDOW_MS } from "./contract.mjs";

export function authorityCall(gateRoot, directory, operation, input, program = "bun") {
  return new Promise((resolveResult, reject) => {
    const args = [resolve(gateRoot, "scripts/worker-development-authority.ts"), operation, "--directory", directory];
    if (operation !== "public-trust") args.push("--input", "-");
    args.push("--output", "-");
    const child = spawn(program, args, { cwd: gateRoot, stdio: ["pipe", "pipe", "pipe"] });
    let bytes = Buffer.alloc(0), overflow = false, stderrBytes = 0, inputFailed = false;
    const timer = setTimeout(() => child.kill("SIGKILL"), 10000);
    child.stdout.on("data", (chunk) => {
      if (bytes.length + chunk.length > 65536) { overflow = true; child.kill("SIGKILL"); return; }
      bytes = Buffer.concat([bytes, chunk]);
    });
    // Never copy signer diagnostics, which could contain sensitive input, into evidence.
    child.stderr.on("data", (chunk) => {
      stderrBytes += chunk.length;
      if (stderrBytes > 65536) { overflow = true; child.kill("SIGKILL"); }
    });
    child.once("error", () => { clearTimeout(timer); reject(new QualificationError("authority_unavailable")); });
    child.once("close", (code) => {
      clearTimeout(timer);
      if (code !== 0 || overflow || inputFailed) { reject(new QualificationError("authority_failed")); return; }
      try { resolveResult(JSON.parse(bytes.toString("utf8"))); }
      catch { reject(new QualificationError("authority_output")); }
    });
    child.stdin.on("error", () => { inputFailed = true; child.kill("SIGKILL"); });
    child.stdin.end(input === undefined ? undefined : JSON.stringify(input));
  });
}

export async function readPoolForSigning(firmwareRoot, path) {
  await protectedPath(path);
  ignored(firmwareRoot, path);
  const input = JSON.parse(await readFile(path, "utf8"));
  exactObject(input, ["poolURL", "poolPort", "poolUser", "poolPassword"]);
  requireCondition(typeof input.poolURL === "string" && input.poolURL.length <= 1024 &&
    Number.isInteger(input.poolPort) && input.poolPort > 0 && input.poolPort <= 65535 &&
    typeof input.poolUser === "string" && input.poolUser.length > 0 && input.poolUser.length <= 1024 &&
    typeof input.poolPassword === "string" && input.poolPassword.length <= 1024, "pool_input_shape");
  let endpoint;
  try { endpoint = new URL(input.poolURL.includes("://") ? input.poolURL : `stratum+tcp://${input.poolURL}/`); }
  catch { throw new QualificationError("pool_input_shape"); }
  requireCondition(endpoint.protocol === "stratum+tcp:" && endpoint.username === "" && endpoint.password === "" &&
    endpoint.pathname === "/" && endpoint.search === "" && endpoint.hash === "", "pool_input_shape");
  endpoint.port = String(input.poolPort);
  return { endpoint: endpoint.href, username: input.poolUser, password: input.poolPassword };
}

export async function signWindow({ campaignId, index, challengeId, binding, stratum, sign }) {
  requireCondition(canonicalBase64(campaignId, 16) && canonicalBase64(binding, 32) &&
    /^challenge_[A-Za-z0-9_-]{1,118}$/u.test(challengeId) && Number.isInteger(index) && index >= 0 && index < 3, "signing_context");
  const leaseId = `lease_${nonce()}`;
  const common = { protocolVersion: "bwg-worker-controller/0.4", leaseId,
    durationMilliseconds: 60000, renewAfterMilliseconds: 20000 };
  const grant = { ...common, challengeId, stratum,
    acceptanceCampaign: { id: campaignId, window: index, maximumActiveMilliseconds: WINDOW_MS[index] } };
  const input = (operation, request) => ({ operation, activeChallengeId: challengeId,
    controlSessionBindingSha256: binding, request });
  const attach = async (operation, request) => {
    const artifact = await sign(operation, input(operation, request));
    requireCondition(artifact.profile === "bwg-worker-lease-authorization-artifact/0.1" && artifact.operation === operation &&
      typeof artifact.authorization === "string" && artifact.authorization.length > 0 && artifact.authorization.length <= 8192, "authorization_shape");
    return { ...request, authorization: artifact.authorization };
  };
  const signedGrant = await attach("start", grant);
  const renewals = [];
  for (let index = 0; index < Math.ceil(WINDOW_MS[grant.acceptanceCampaign.window] / 20000); index += 1) {
    renewals.push(await attach("renew", common));
  }
  return { grant: signedGrant, renewals };
}

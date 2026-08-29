#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { createServer } from "node:http";
import { appendFile, lstat, open, readFile, readdir, unlink } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  RESULT_PROFILE,
  SCENARIOS,
  authoritySnapshot,
  browserPoolCredentials,
  canonicalJson,
  detectorPort,
  digestFile,
  exactOptions,
  randomIdentifier,
  readJson,
  requireCleanRepository,
  requireFreshDetector,
  requireProtectedDirectory,
  requireProtectedFile,
  sha256,
  safeEvent,
  expectedTerminalReasons,
  physicalInstruction,
  redactedProjection,
  validatePackage,
  validatePreflightScope,
  validatePoolCredentials,
  validateCompletion,
  validatePoolReadiness,
  validatePreflight,
  validateAuthorityDirectory,
} from "./contract.mjs";
import { validateRuntimeAttestationCapture } from "./runtime-attestation.mjs";
import { resolvePrivatePoolEndpoint } from "./private-pool.mjs";
import { publishProjectionSet } from "./publish-projections.mjs";

const SOURCE_ROOT = dirname(fileURLToPath(import.meta.url));
const OPTIONS = ["--preflight"];
const MAXIMUM_BODY_BYTES = 4_096;

async function writeExclusive(path, value, mode = 0o600) {
  const handle = await open(path, "wx", mode);
  try {
    await handle.writeFile(typeof value === "string" ? value : `${canonicalJson(value)}\n`);
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function boundedJson(request) {
  const chunks = [];
  let length = 0;
  for await (const chunk of request) {
    length += chunk.length;
    if (length > MAXIMUM_BODY_BYTES) throw new Error("body_oversized");
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}

async function maybePublishEvidenceSet(privateParent, identity) {
  const completed = new Map();
  for (const entry of await readdir(privateParent, { withFileTypes: true })) {
    if (!entry.isDirectory() || !/^bwg007-attempt-[0-9]{3}$/.test(entry.name)) continue;
    const root = resolve(privateParent, entry.name);
    const resultPath = resolve(root, "result.private.json");
    const pendingPath = resolve(root, "projection.pending.private.json");
    let result;
    try {
      await requireProtectedDirectory(root);
      await requireProtectedFile(resultPath);
      result = await readJson(resultPath);
    } catch (error) {
      if (error?.code === "ENOENT") continue;
      throw error;
    }
    if (result.outcome !== "complete") continue;
    await requireProtectedFile(pendingPath);
    const admittedPreflightPath = resolve(root, "preflight.private.json");
    await requireProtectedFile(admittedPreflightPath);
    const admitted = validatePreflight(await readJson(admittedPreflightPath));
    validatePreflightScope(admitted, admittedPreflightPath);
    if (
      admitted.firmwareCommit !== identity.firmwareCommit ||
      admitted.gateCommit !== identity.gateCommit ||
      admitted.packageManifestSha256 !== identity.packageManifestSha256 ||
      admitted.restoreBundleSha256 !== identity.restoreBundleSha256 ||
      admitted.poolCredentialsSha256 !== identity.poolCredentialsSha256 ||
      admitted.poolReadinessSha256 !== identity.poolReadinessSha256 ||
      admitted.poolResolvedEndpointsSha256 !== identity.poolResolvedEndpointsSha256 ||
      admitted.authorityStaticSha256 !== identity.authorityStaticSha256 ||
      admitted.gateBundleSha256 !== identity.gateBundleSha256 ||
      admitted.gateTrustSha256 !== identity.gateTrustSha256 ||
      result.profile !== RESULT_PROFILE || result.attemptId !== admitted.attemptId ||
      result.scenario !== admitted.scenario || !SCENARIOS.includes(result.scenario) ||
      Object.keys(result).sort().join(",") !== [
        "attemptId", "detectorPortSha256", "eventsSha256", "outcome", "profile",
        "deviceIdentityFingerprintSha256", "runtimeAttestationSamples",
        "runtimeAttestationSha256", "scenario", "terminalReason",
      ].sort().join(",") ||
      !expectedTerminalReasons(result.scenario).includes(result.terminalReason) ||
      !/^[0-9a-f]{64}$/.test(result.detectorPortSha256) ||
      !/^[0-9a-f]{64}$/.test(result.deviceIdentityFingerprintSha256) ||
      !/^[0-9a-f]{64}$/.test(result.eventsSha256) ||
      !/^[0-9a-f]{64}$/.test(result.runtimeAttestationSha256) ||
      !Number.isInteger(result.runtimeAttestationSamples) || result.runtimeAttestationSamples < 2
    ) {
      throw new Error("evidence_set_invalid");
    }
    const expected = redactedProjection({
      attemptId: admitted.attemptId,
      scenario: admitted.scenario,
      terminalReason: result.terminalReason,
      firmwareCommit: admitted.firmwareCommit,
      gateCommit: admitted.gateCommit,
      gateProfileCommit: admitted.gateProfileCommit,
      packageManifestSha256: admitted.packageManifestSha256,
      appElfSha256: admitted.appElfSha256,
      gateBundleSha256: admitted.gateBundleSha256,
      restoreBundleSha256: admitted.restoreBundleSha256,
      runtimeAttestationSha256: result.runtimeAttestationSha256,
      eventsSha256: result.eventsSha256,
    });
    if (canonicalJson(await readJson(pendingPath)) !== canonicalJson(expected)) {
      throw new Error("pending_projection_invalid");
    }
    if (completed.has(admitted.scenario)) throw new Error("scenario_duplicate");
    completed.set(admitted.scenario, {
      target: admitted.projection,
      projection: expected,
      deviceIdentityFingerprintSha256: result.deviceIdentityFingerprintSha256,
    });
  }
  return publishProjectionSet(completed);
}

async function main(args) {
  const options = exactOptions(args, OPTIONS);
  const preflightPath = resolve(options["--preflight"]);
  if (basename(preflightPath) !== "preflight.private.json") throw new Error("preflight_path_invalid");
  await requireProtectedFile(preflightPath);
  const preflight = validatePreflight(await readJson(preflightPath));
  if (preflight.scenario === "monotonic_uncertainty") {
    throw new Error("monotonic_stimulus_not_authorized");
  }
  if (preflight.scenario === "authorization_negatives") {
    throw new Error("durable_replay_attribution_not_available");
  }
  const attemptRoot = validatePreflightScope(preflight, preflightPath);
  requireCleanRepository(preflight.gateRepository, preflight.gateCommit);
  requireCleanRepository(preflight.firmwareRepository, preflight.firmwareCommit);
  if ((await validatePackage(
    preflight.packageManifest,
    preflight.firmwareCommit,
    preflight.firmwareRepository,
  )).digest !==
      preflight.packageManifestSha256) {
    throw new Error("package_drift");
  }
  if (await requireFreshDetector(preflight.detectorOutput) !== preflight.detectorSha256) {
    throw new Error("detector_drift");
  }
  if (await digestFile(preflight.restoreBundle) !== preflight.restoreBundleSha256) {
    throw new Error("restore_bundle_drift");
  }
  for (const path of [preflight.restoreAuthorization, preflight.wifiCredentials]) {
    await requireProtectedFile(path);
  }
  for (const [path, digest] of [
    [preflight.restoreAuthorization, preflight.restoreAuthorizationSha256],
    [preflight.remediationPlan, preflight.remediationPlanSha256],
    [preflight.wifiCredentials, preflight.wifiCredentialsSha256],
  ]) {
    if (await digestFile(path) !== digest) throw new Error("recovery_input_drift");
  }
  await validateAuthorityDirectory(preflight.authorityDirectory);
  await validatePoolCredentials(preflight.poolCredentials);
  if (await digestFile(preflight.poolCredentials) !== preflight.poolCredentialsSha256) {
    throw new Error("pool_credentials_drift");
  }
  if ((await validatePoolReadiness(
    preflight.poolReadiness,
    preflight.firmwareCommit,
    preflight.referenceCommit,
    preflight.poolCredentialsSha256,
  )).digest !==
      preflight.poolReadinessSha256) {
    throw new Error("pool_readiness_drift");
  }
  const authority = await authoritySnapshot(preflight.authorityDirectory);
  if (
    authority.staticSha256 !== preflight.authorityStaticSha256 ||
    authority.sequenceSha256 !== preflight.authoritySequenceSha256
  ) {
    throw new Error("authority_drift");
  }
  await requireProtectedDirectory(attemptRoot);
  if (preflight.recoveryRoot !== resolve(attemptRoot, "recovery")) {
    throw new Error("recovery_root_invalid");
  }
  await requireProtectedDirectory(preflight.recoveryRoot);
  const eventsPath = resolve(attemptRoot, "events.private.ndjson");
  const resultPath = resolve(attemptRoot, "result.private.json");
  const pendingProjectionPath = resolve(attemptRoot, "projection.pending.private.json");
  const deviceIdentityPath = resolve(attemptRoot, "device-identity.private.json");
  const urlPath = resolve(attemptRoot, "browser-url.private.txt");
  await writeExclusive(eventsPath, "");
  const monitor = spawnSync(
    "bazel",
    [
      "run", "//tools/flash:flash", "--", "monitor", "--board", "205", "--port",
      detectorPort(await readFile(preflight.detectorOutput, "utf8")),
      "--capture-timeout-seconds", "25",
    ],
    {
      cwd: preflight.firmwareRepository,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 45_000,
    },
  );
  if (monitor.status !== 0 || monitor.error) throw new Error("runtime_attestation_capture_failed");
  const runtimeCapture = `${monitor.stdout}${monitor.stderr}`;
  const runtimeIdentity = validateRuntimeAttestationCapture(runtimeCapture, {
    firmwareCommit: preflight.firmwareCommit,
    referenceCommit: preflight.referenceCommit,
    appElfSha256: preflight.appElfSha256,
  });
  const runtimeCapturePath = resolve(attemptRoot, "runtime-attestation.private.log");
  await writeExclusive(runtimeCapturePath, runtimeCapture);
  await appendFile(
    eventsPath,
    `${canonicalJson(safeEvent({ event: "runtime_identity_admitted" }))}\n`,
    { mode: 0o600 },
  );
  const poolDocument = await readJson(preflight.poolCredentials);
  const resolvedPoolAddress = await resolvePrivatePoolEndpoint(
    poolDocument,
    preflight.poolResolvedEndpointsSha256,
  );
  const pool = browserPoolCredentials(poolDocument, resolvedPoolAddress);
  const trust = await readJson(resolve(
    preflight.gateRepository,
    "conformance/bwg-worker-deployment-trust-0.1/trust.json",
  ));
  if (await digestFile(resolve(
    preflight.gateRepository,
    "conformance/bwg-worker-deployment-trust-0.1/trust.json",
  )) !== preflight.gateTrustSha256) throw new Error("gate_trust_drift");
  if (canonicalJson(authority.trust) !== canonicalJson(trust)) {
    throw new Error("authority_trust_mismatch");
  }
  const gateBundle = await readFile(resolve(
    preflight.gateRepository,
    "dist/worker-controller-v03/worker-controller-v03-entry.js",
  ));
  if (sha256(gateBundle) !== preflight.gateBundleSha256) throw new Error("gate_bundle_drift");
  const browserSource = await readFile(resolve(SOURCE_ROOT, "browser.mjs"));
  const browserContract = await readFile(resolve(SOURCE_ROOT, "browser-contract.mjs"));
  const browserHtml = await readFile(resolve(SOURCE_ROOT, "browser.html"));
  const token = randomBytes(32).toString("hex");
  const challengeId = randomIdentifier("challenge_");
  const leaseId = randomIdentifier("lease_");
  const expiry = preflight.scenario === "expiry";
  const startRequest = {
    protocolVersion: "bwg-worker-controller/0.3",
    leaseId,
    challengeId,
    durationMilliseconds: expiry ? 3_000 : 60_000,
    renewAfterMilliseconds: expiry ? 1_000 : 20_000,
    stratum: pool,
  };
  const renewalRequest = {
    protocolVersion: "bwg-worker-controller/0.3",
    leaseId,
    durationMilliseconds: 60_000,
    renewAfterMilliseconds: 20_000,
  };
  let terminalOutcome = "open";
  let authorizationOrdinal = 0;

  async function record(value) {
    await appendFile(eventsPath, `${canonicalJson(safeEvent(value))}\n`, { mode: 0o600 });
  }

  async function sign(operation, binding) {
    if (!/^[A-Za-z0-9_-]{43}$/.test(binding)) throw new Error("binding_invalid");
    if ((await authoritySnapshot(preflight.authorityDirectory)).staticSha256 !==
        preflight.authorityStaticSha256) {
      throw new Error("authority_drift");
    }
    authorizationOrdinal += 1;
    const stem = `authorization-${String(authorizationOrdinal).padStart(3, "0")}`;
    const inputPath = resolve(attemptRoot, `${stem}.input.private.json`);
    const outputPath = resolve(attemptRoot, `${stem}.output.private.json`);
    const request = operation === "start" ? startRequest : renewalRequest;
    await writeExclusive(inputPath, {
      operation,
      activeChallengeId: challengeId,
      controlSessionBindingSha256: binding,
      request,
    });
    let maybePrimaryError;
    let maybeResponse;
    try {
      const command = operation === "start" ? "sign-start" : "sign-renew";
      const result = spawnSync(
        "bun",
        [
          resolve(preflight.gateRepository, "scripts/worker-development-authority.ts"),
          command,
          "--directory",
          preflight.authorityDirectory,
          "--input",
          inputPath,
          "--output",
          outputPath,
        ],
        { stdio: "ignore", timeout: 30_000 },
      );
      if (result.status !== 0) throw new Error("authorization_failed");
      const artifact = await readJson(outputPath);
      if (artifact.operation !== operation || typeof artifact.authorization !== "string") {
        throw new Error("authorization_invalid");
      }
      maybeResponse = { request: { ...request, authorization: artifact.authorization } };
    } catch (error) {
      maybePrimaryError = error;
    }
    const cleanupErrors = [];
    for (const path of [inputPath, outputPath]) {
      try {
        await unlink(path);
      } catch (error) {
        if (error?.code !== "ENOENT") cleanupErrors.push(error);
      }
      try {
        await lstat(path);
        cleanupErrors.push(new Error("authorization_file_retained"));
      } catch (error) {
        if (error?.code !== "ENOENT") cleanupErrors.push(error);
      }
    }
    if (cleanupErrors.length > 0) {
      throw new AggregateError(
        maybePrimaryError ? [maybePrimaryError, ...cleanupErrors] : cleanupErrors,
        "authorization_cleanup_failed",
      );
    }
    if (maybePrimaryError) throw maybePrimaryError;
    return maybeResponse;
  }

  function workspaceRelative(path) {
    const value = relative(preflight.firmwareRepository, path);
    if (value.startsWith("..") || value === "") throw new Error("recovery_path_invalid");
    return value;
  }

  async function recoverInstalledPackage() {
    const result = spawnSync(
      "bazel",
      [
        "run",
        "//tools/flash:flash",
        "--",
        "restore-installed",
        "--board",
        "205",
        "--port",
        detectorPort(await readFile(preflight.detectorOutput, "utf8")),
        "--restore-bundle",
        workspaceRelative(preflight.restoreBundle),
        "--restore-authorization",
        workspaceRelative(preflight.restoreAuthorization),
        "--remediation-plan",
        workspaceRelative(preflight.remediationPlan),
        "--private-root",
        workspaceRelative(preflight.recoveryRoot),
        "--wifi-credentials",
        workspaceRelative(preflight.wifiCredentials),
        "--redact-evidence",
      ],
      {
        cwd: preflight.firmwareRepository,
        stdio: "ignore",
        timeout: 15 * 60 * 1_000,
      },
    );
    return result.status === 0 ? "recovered" : "recovery_failed";
  }

  const server = createServer(async (request, response) => {
    let terminalRoute = false;
    try {
      const url = new URL(request.url ?? "/", "http://127.0.0.1");
      const [requestToken, route] = url.pathname.split("/").filter(Boolean);
      if (requestToken !== token || request.headers.host?.startsWith("127.0.0.1:") !== true) {
        throw new Error("request_denied");
      }
      if (
        request.method === "POST" &&
        request.headers.origin !== `http://${request.headers.host}`
      ) {
        throw new Error("origin_denied");
      }
      response.setHeader("cache-control", "no-store");
      response.setHeader(
        "content-security-policy",
        "default-src 'self'; connect-src 'self'; script-src 'self'; style-src 'unsafe-inline'",
      );
      if (request.method === "GET" && route === undefined) {
        response.setHeader("content-type", "text/html; charset=utf-8");
        response.end(browserHtml);
        return;
      }
      if (request.method === "GET" && route === "browser.mjs") {
        response.setHeader("content-type", "text/javascript; charset=utf-8");
        response.end(browserSource);
        return;
      }
      if (request.method === "GET" && route === "browser-contract.mjs") {
        response.setHeader("content-type", "text/javascript; charset=utf-8");
        response.end(browserContract);
        return;
      }
      if (request.method === "GET" && route === "gate.js") {
        response.setHeader("content-type", "text/javascript; charset=utf-8");
        response.end(gateBundle);
        return;
      }
      if (request.method === "GET" && route === "config") {
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({
          scenario: preflight.scenario,
          minimumActiveMilliseconds: 5_000,
          contextExpiryMilliseconds: 60_050,
          physicalInstruction: physicalInstruction(preflight.scenario),
          controller: {
            deviceFilter: { vendorId: 0x1209, productId: 0xb17a },
            trustedUpdateKeys: trust.updateAuthority.keys,
            expectedFirmwareSourceCommit: preflight.firmwareCommit,
            continuityScope: {
              challengeId,
              retentionExpiryUnixSeconds: Math.floor(Date.now() / 1_000) + 900,
            },
          },
        }));
        return;
      }
      if (request.method !== "POST") throw new Error("method_invalid");
      const rawBody = await boundedJson(request);
      response.setHeader("content-type", "application/json");
      if (route === "authorize") {
        if (
          typeof rawBody !== "object" || rawBody === null || Array.isArray(rawBody) ||
          Object.keys(rawBody).sort().join(",") !==
            "controlSessionBindingSha256,operation" ||
          !["start", "renew"].includes(rawBody.operation)
        ) {
          throw new Error("operation_invalid");
        }
        response.end(JSON.stringify(await sign(
          rawBody.operation,
          rawBody.controlSessionBindingSha256,
        )));
        return;
      }
      if (route === "identity") {
        if (
          typeof rawBody !== "object" || rawBody === null || Array.isArray(rawBody) ||
          Object.keys(rawBody).join(",") !== "deviceIdentityFingerprint" ||
          !/^[A-Za-z0-9_-]{43}$/.test(rawBody.deviceIdentityFingerprint)
        ) throw new Error("device_identity_invalid");
        await writeExclusive(deviceIdentityPath, rawBody);
        response.end("{}");
        return;
      }
      const body = safeEvent(rawBody);
      if (route === "event") {
        if (typeof body.event !== "string") throw new Error("event_invalid");
        await record(body);
        response.end("{}");
        return;
      }
      if (route === "failed") {
        terminalRoute = true;
        if (
          typeof body.category !== "string" ||
          Object.keys(body).some((key) => !["category", "cleanupCategory"].includes(key))
        ) {
          throw new Error("failure_invalid");
        }
        if (terminalOutcome === "open") {
          const recoveryOutcome = body.cleanupCategory === "cleanup_failed" ||
            body.category === "browser_closed_active"
            ? await recoverInstalledPackage()
            : "not_required";
          await record({
            event: "failed",
            category: body.category,
            ...(body.cleanupCategory ? { cleanupCategory: body.cleanupCategory } : {}),
          });
          await writeExclusive(resultPath, {
            profile: RESULT_PROFILE,
            attemptId: preflight.attemptId,
            scenario: preflight.scenario,
            outcome: "failed",
            category: body.category,
            cleanupCategory: body.cleanupCategory ?? "none",
            recoveryOutcome,
          });
          terminalOutcome = "failed";
          setImmediate(() => server.close());
        }
        response.end("{}");
        return;
      }
      if (route === "complete") {
        terminalRoute = true;
        if (terminalOutcome !== "open") throw new Error("completion_invalid");
        const eventText = await readFile(eventsPath, "utf8");
        await requireProtectedFile(deviceIdentityPath);
        const deviceIdentity = await readJson(deviceIdentityPath);
        const events = eventText.split(/\r?\n/).filter(Boolean).map((line) => JSON.parse(line));
        const terminalReason = validateCompletion(preflight.scenario, body, events);
        for (const prohibited of [
          pool.endpoint,
          pool.username,
          pool.password,
          challengeId,
          leaseId,
        ]) {
          if (eventText.includes(prohibited)) throw new Error("private_material_exposed");
        }
        await record({ event: "complete", outcome: "complete" });
        const eventsSha256 = await digestFile(eventsPath);
        const projection = redactedProjection({
          attemptId: preflight.attemptId,
          scenario: preflight.scenario,
          terminalReason,
          firmwareCommit: preflight.firmwareCommit,
          gateCommit: preflight.gateCommit,
          gateProfileCommit: preflight.gateProfileCommit,
          packageManifestSha256: preflight.packageManifestSha256,
          appElfSha256: preflight.appElfSha256,
          gateBundleSha256: preflight.gateBundleSha256,
          restoreBundleSha256: preflight.restoreBundleSha256,
          runtimeAttestationSha256: runtimeIdentity.captureSha256,
          eventsSha256,
        });
        await writeExclusive(pendingProjectionPath, projection);
        await writeExclusive(resultPath, {
          profile: RESULT_PROFILE,
          attemptId: preflight.attemptId,
          scenario: preflight.scenario,
          outcome: "complete",
          terminalReason,
          eventsSha256,
          runtimeAttestationSha256: runtimeIdentity.captureSha256,
          runtimeAttestationSamples: runtimeIdentity.sampleCount,
          deviceIdentityFingerprintSha256: sha256(deviceIdentity.deviceIdentityFingerprint),
          detectorPortSha256: sha256(detectorPort(await readFile(preflight.detectorOutput, "utf8"))),
        });
        terminalOutcome = "complete";
        response.end("{}");
        setImmediate(() => server.close());
        return;
      }
      throw new Error("route_invalid");
    } catch {
      if (terminalRoute && terminalOutcome === "open") {
        terminalOutcome = "failed";
        try {
          await unlink(preflight.projection);
        } catch (error) {
          if (error?.code !== "ENOENT") {
            response.statusCode = 500;
          }
        }
        setImmediate(() => server.close());
      }
      response.statusCode = 400;
      response.end("{}");
    }
  });

  await new Promise((resolveReady, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolveReady);
  });
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("server_unavailable");
  const browserUrl = `http://127.0.0.1:${address.port}/${token}/`;
  await writeExclusive(urlPath, `${browserUrl}\n`);
  process.stdout.write(
    `bwg_worker_restoration_campaign=waiting scenario=${preflight.scenario} browser_url_file=${urlPath}\n`,
  );
  await new Promise((resolveClosed) => server.once("close", resolveClosed));
  if (terminalOutcome !== "complete") throw new Error("campaign_incomplete");
  const evidencePublished = await maybePublishEvidenceSet(dirname(attemptRoot), {
    firmwareCommit: preflight.firmwareCommit,
    gateCommit: preflight.gateCommit,
    packageManifestSha256: preflight.packageManifestSha256,
    restoreBundleSha256: preflight.restoreBundleSha256,
    poolCredentialsSha256: preflight.poolCredentialsSha256,
    poolReadinessSha256: preflight.poolReadinessSha256,
    poolResolvedEndpointsSha256: preflight.poolResolvedEndpointsSha256,
    authorityStaticSha256: preflight.authorityStaticSha256,
    gateBundleSha256: preflight.gateBundleSha256,
    gateTrustSha256: preflight.gateTrustSha256,
  });
  process.stdout.write(
    `bwg_worker_restoration_campaign=complete scenario=${preflight.scenario} ` +
    `evidence=${evidencePublished ? "published" : "private_pending"}\n`,
  );
}

await main(process.argv.slice(2)).catch(() => {
  process.stderr.write("bwg_worker_restoration_campaign=blocked category=campaign_failed\n");
  process.exitCode = 1;
});

#!/usr/bin/env node

import { lstat, mkdir, open, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

import {
  GATE_PROFILE_COMMIT,
  GATE_BROWSER_COMMIT,
  PREFLIGHT_PROFILE,
  absolutePath,
  authoritySnapshot,
  canonicalJson,
  digestFile,
  exactOptions,
  gitHead,
  parseScenario,
  preflightDigest,
  requireCleanRepository,
  requireAncestor,
  requireFreshDetector,
  requireGateBrowserBuild,
  requirePackageAdmission,
  requirePathWithin,
  requireProtectedDirectory,
  requireProtectedFile,
  validatePackage,
  validatePoolCredentials,
  validatePoolReadiness,
  readJson,
} from "./contract.mjs";

const OPTIONS = [
  "--private-parent",
  "--attempt-id",
  "--package-manifest",
  "--firmware-repository",
  "--gate-repository",
  "--gate-commit",
  "--authority-directory",
  "--pool-credentials",
  "--pool-readiness",
  "--restore-bundle",
  "--wifi-credentials",
  "--detector-output",
  "--scenario",
  "--projection",
];

async function writeExclusive(path, value) {
  const handle = await open(path, "wx", 0o600);
  try {
    await handle.writeFile(`${canonicalJson(value)}\n`, "utf8");
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function main(args) {
  const options = exactOptions(args, OPTIONS);
  const privateParent = absolutePath(options["--private-parent"]);
  const attemptId = options["--attempt-id"];
  if (!/^bwg007-attempt-[0-9]{3}$/.test(attemptId)) throw new Error("attempt_id_invalid");
  await requireProtectedDirectory(privateParent);
  const attemptRoot = requirePathWithin(resolve(privateParent, attemptId), privateParent);
  const packageManifest = absolutePath(options["--package-manifest"]);
  const gateRepository = absolutePath(options["--gate-repository"]);
  const authorityDirectory = absolutePath(options["--authority-directory"]);
  const poolCredentials = absolutePath(options["--pool-credentials"]);
  const poolReadiness = absolutePath(options["--pool-readiness"]);
  const restoreBundle = absolutePath(options["--restore-bundle"]);
  const wifiCredentials = absolutePath(options["--wifi-credentials"]);
  const detectorOutput = absolutePath(options["--detector-output"]);
  const scenario = parseScenario(options["--scenario"]);
  const firmwareRepository = absolutePath(options["--firmware-repository"]);
  if (privateParent !== resolve(firmwareRepository, "scratch/bwg-worker-restoration")) {
    throw new Error("private_parent_invalid");
  }
  if (packageManifest !== resolve(
    firmwareRepository,
    "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  )) {
    throw new Error("package_manifest_path_invalid");
  }
  const projection = requirePathWithin(
    absolutePath(options["--projection"]),
    resolve(firmwareRepository, "docs/parity/evidence/bwg-worker-restoration"),
  );
  if (projection !== resolve(
    firmwareRepository,
    `docs/parity/evidence/bwg-worker-restoration/${attemptId}-${scenario}.json`,
  )) {
    throw new Error("projection_path_invalid");
  }
  const projectionParent = await lstat(dirname(projection));
  if (!projectionParent.isDirectory() || projectionParent.isSymbolicLink()) {
    throw new Error("projection_parent_invalid");
  }
  try {
    await lstat(projection);
    throw new Error("projection_exists");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const firmwareHead = gitHead(firmwareRepository);
  requireCleanRepository(firmwareRepository, firmwareHead);
  const gateCommit = options["--gate-commit"];
  if (gateCommit !== GATE_BROWSER_COMMIT) throw new Error("gate_commit_invalid");
  requireCleanRepository(gateRepository, gateCommit);
  requireAncestor(gateRepository, GATE_PROFILE_COMMIT, gateCommit);
  requireGateBrowserBuild(gateRepository);
  requireCleanRepository(gateRepository, gateCommit);
  const packageEvidence = await validatePackage(
    packageManifest,
    firmwareHead,
    firmwareRepository,
  );
  requirePackageAdmission(packageManifest, firmwareRepository);
  const authority = await authoritySnapshot(authorityDirectory);
  const poolShapeSha256 = await validatePoolCredentials(poolCredentials);
  const poolCredentialsSha256 = await digestFile(poolCredentials);
  const poolReadinessEvidence = await validatePoolReadiness(
    poolReadiness,
    firmwareHead,
    packageEvidence.manifest.reference_commit,
    poolCredentialsSha256,
  );
  await requireProtectedFile(restoreBundle);
  const restoreBundleSha256 = await digestFile(restoreBundle);
  await requireProtectedFile(wifiCredentials);
  const expectedRestoreBundle = resolve(
    firmwareRepository,
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  );
  if (restoreBundle !== expectedRestoreBundle) throw new Error("restore_bundle_invalid");
  const remediationPlan = resolve(
    firmwareRepository,
    "docs/adr/0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md",
  );
  const remediationPlanSha256 = await digestFile(remediationPlan);
  const restoreDocument = await readJson(restoreBundle);
  if (
    restoreDocument.schema_version !== "bitaxe-stratum-v2-restore-bundle-v1" ||
    restoreDocument.board !== 205 ||
    typeof restoreDocument.capture_source_commit !== "string" ||
    !/^[0-9a-f]{40}$/.test(restoreDocument.capture_source_commit) ||
    typeof restoreDocument.plan_sha256 !== "string" ||
    !/^[0-9a-f]{64}$/.test(restoreDocument.plan_sha256)
  ) {
    throw new Error("restore_bundle_invalid");
  }
  const detectorSha256 = await requireFreshDetector(detectorOutput);
  const gateBundle = resolve(
    gateRepository,
    "dist/worker-controller-v03/worker-controller-v03-entry.js",
  );
  const gateTrust = resolve(
    gateRepository,
    "conformance/bwg-worker-deployment-trust-0.1/trust.json",
  );
  await readFile(gateBundle);
  const gateTrustDocument = await readJson(gateTrust);
  const firmwareTrustDocument = await readJson(resolve(
    firmwareRepository,
    "firmware/bitaxe/bwg/deployment-trust.json",
  ));
  if (
    canonicalJson(authority.trust) !== canonicalJson(gateTrustDocument) ||
    canonicalJson(authority.trust) !== canonicalJson(firmwareTrustDocument)
  ) {
    throw new Error("authority_trust_mismatch");
  }
  const document = {
    profile: PREFLIGHT_PROFILE,
    attemptId,
    scenario,
    firmwareRepository,
    firmwareCommit: firmwareHead,
    referenceCommit: packageEvidence.manifest.reference_commit,
    appElfSha256: packageEvidence.manifest.app_elf_sha256,
    gateCommit,
    gateProfileCommit: GATE_PROFILE_COMMIT,
    packageManifest,
    packageManifestSha256: packageEvidence.digest,
    gateRepository,
    gateBundleSha256: await digestFile(gateBundle),
    gateTrustSha256: await digestFile(gateTrust),
    authorityDirectory,
    authorityStaticSha256: authority.staticSha256,
    authoritySequenceSha256: authority.sequenceSha256,
    poolCredentials,
    poolShapeSha256,
    poolCredentialsSha256,
    poolReadiness,
    poolReadinessSha256: poolReadinessEvidence.digest,
    poolResolvedEndpointsSha256: poolReadinessEvidence.resolvedEndpointsSha256,
    restoreBundle,
    restoreBundleSha256,
    remediationPlan,
    remediationPlanSha256,
    wifiCredentials,
    wifiCredentialsSha256: await digestFile(wifiCredentials),
    detectorOutput,
    detectorSha256,
    projection,
    allowedInterfaces: ["usb", "barrel_power"],
    forbiddenInterfaces: ["uart", "pins", "probes", "erasure", "ad_hoc_writes"],
    preflightDigestSha256: "",
  };
  await mkdir(attemptRoot, { mode: 0o700 });
  const recoveryRoot = resolve(attemptRoot, "recovery");
  await mkdir(recoveryRoot, { mode: 0o700 });
  const restoreAuthorization = resolve(recoveryRoot, "restore-authorization.private.json");
  await writeExclusive(restoreAuthorization, {
    schema_version: "bitaxe-stratum-v2-restore-authorization-v1",
    board: 205,
    ordinal: Number(attemptId.slice(-3)),
    action: "bwg_worker_restoration",
    current_source_commit: firmwareHead,
    reference_commit: gitHead(resolve(firmwareRepository, "reference/esp-miner")),
    bundle_sha256: restoreBundleSha256,
    bundle_capture_source_commit: restoreDocument.capture_source_commit,
    recovery_plan_sha256: restoreDocument.plan_sha256,
    remediation_plan_sha256: remediationPlanSha256,
  });
  document.recoveryRoot = recoveryRoot;
  document.restoreAuthorization = restoreAuthorization;
  document.restoreAuthorizationSha256 = await digestFile(restoreAuthorization);
  document.remediationPlan = remediationPlan;
  document.remediationPlanSha256 = remediationPlanSha256;
  document.preflightDigestSha256 = preflightDigest(document);
  await writeExclusive(resolve(attemptRoot, "preflight.private.json"), document);
  process.stdout.write(
    `bwg_worker_restoration_preflight=ready scenario=${scenario} digest=${document.preflightDigestSha256}\n`,
  );
}

await main(process.argv.slice(2)).catch(() => {
  process.stderr.write("bwg_worker_restoration_preflight=blocked category=preflight_failed\n");
  process.exitCode = 1;
});

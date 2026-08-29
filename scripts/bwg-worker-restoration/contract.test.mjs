import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, mkdtemp, open, readdir, symlink, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import test from "node:test";

import {
  GATE_PROFILE_COMMIT,
  GATE_BROWSER_COMMIT,
  PREFLIGHT_PROFILE,
  SCENARIOS,
  exactOptions,
  detectorPort,
  parseScenario,
  preflightDigest,
  requireFreshDetector,
  requireProtectedFile,
  safeEvent,
  physicalInstruction,
  redactedProjection,
  requiresPhysicalReacquisition,
  validatePoolCredentials,
  validatePoolReadiness,
  validateCompletion,
  validatePackage,
  validatePreflight,
  validatePreflightScope,
} from "./contract.mjs";
import { validateRuntimeAttestationCapture } from "./runtime-attestation.mjs";
import { privateIpv4 } from "./private-pool.mjs";
import { writeAtomicNew } from "./atomic-publication.mjs";
import { validatePublicProjection } from "./projection-validator.mjs";

test("options and scenario vocabularies are exact", () => {
  // Arrange / Act
  const parsed = exactOptions(["--one", "value"], ["--one"]);

  // Assert
  assert.deepEqual(parsed, { "--one": "value" });
  assert.throws(() => exactOptions(["--one", "a", "--one", "b"], ["--one"]));
  assert.throws(() => exactOptions(["--unknown", "a"], ["--one"]));
  for (const scenario of SCENARIOS) assert.equal(parseScenario(scenario), scenario);
  assert.throws(() => parseScenario("arbitrary"));
});

test("public event and physical instruction vocabularies stay closed", () => {
  // Arrange / Act / Assert
  assert.deepEqual(safeEvent({ event: "complete", outcome: "complete" }), {
    event: "complete",
    outcome: "complete",
  });
  assert.throws(() => safeEvent({ event: "complete", password: "forbidden" }));
  assert.equal(detectorPort("port: /dev/cu.usbmodem1\n"), "/dev/cu.usbmodem1");
  assert.throws(() => detectorPort("port: /dev/one\nport: /dev/two\n"));
  assert.match(physicalInstruction("disconnect"), /barrel power remains connected/);
  assert.match(physicalInstruction("reboot"), /Restore barrel power first/);
  assert.equal(requiresPhysicalReacquisition("reboot"), true);
  assert.equal(requiresPhysicalReacquisition("pause"), false);
  const projection = redactedProjection({
    attemptId: "bwg007-attempt-001",
    scenario: "reboot",
    terminalReason: "reboot",
    firmwareCommit: "a".repeat(40),
    gateCommit: GATE_BROWSER_COMMIT,
    gateProfileCommit: GATE_PROFILE_COMMIT,
    packageManifestSha256: "c".repeat(64),
    gateBundleSha256: "d".repeat(64),
    eventsSha256: "e".repeat(64),
    appElfSha256: "f".repeat(64),
    restoreBundleSha256: "1".repeat(64),
    runtimeAttestationSha256: "2".repeat(64),
    password: "forbidden",
  });
  assert.equal(Object.hasOwn(projection, "password"), false);
  assert.equal(projection.campaignEventCredentialsAbsent, true);
  assert.equal(validatePublicProjection({
    ...projection,
    sameDeviceAcrossScenarios: true,
  }).sameDeviceAcrossScenarios, true);
  assert.throws(() => validatePublicProjection({
    ...projection,
    sameDeviceAcrossScenarios: true,
    poolPassword: "forbidden",
  }));
  assert.equal(
    validateCompletion(
      "reboot",
      {
        outcome: "complete",
        state: "baseline",
        restorationStatus: "confirmed",
        restorationReason: "reboot",
        cleanup: "confirmed",
      },
      [
        { event: "runtime_identity_admitted" },
        { event: "worker_admitted", mode: "initial", count: "1" },
        { event: "capability_admitted" },
        { event: "lease_started", state: "mining", restorationStatus: "pending" },
        { event: "transport_disconnected", reason: "connectivity_lost" },
        { event: "disconnect_handled", reason: "connectivity_lost" },
        {
          event: "baseline_confirmed",
          state: "baseline",
          restorationStatus: "confirmed",
          restorationReason: "reboot",
        },
      ],
    ),
    "reboot",
  );
  assert.throws(() => validateCompletion(
    "reboot",
    {
      outcome: "complete",
      state: "baseline",
      restorationStatus: "confirmed",
      restorationReason: "connectivity_lost",
      cleanup: "confirmed",
    },
    [],
  ));
});

test("preflight digest binds every admitted field", () => {
  // Arrange
  const document = {
    profile: PREFLIGHT_PROFILE,
    attemptId: "bwg007-attempt-001",
    scenario: "pause",
    firmwareRepository: "/tmp/firmware",
    packageManifest: "/tmp/package.json",
    gateRepository: "/tmp/gate",
    authorityDirectory: "/tmp/authority",
    poolCredentials: "/tmp/pool.json",
    poolReadiness: "/tmp/readiness.json",
    restoreBundle: "/tmp/restore.json",
    restoreAuthorization: "/tmp/restore-authorization.json",
    recoveryRoot: "/tmp/recovery",
    remediationPlan: "/tmp/remediation.json",
    wifiCredentials: "/tmp/wifi.json",
    detectorOutput: "/tmp/detector.txt",
    firmwareCommit: "a".repeat(40),
    referenceCommit: "b".repeat(40),
    appElfSha256: "a".repeat(64),
    gateCommit: GATE_BROWSER_COMMIT,
    gateProfileCommit: GATE_PROFILE_COMMIT,
    packageManifestSha256: "c".repeat(64),
    detectorSha256: "d".repeat(64),
    gateBundleSha256: "e".repeat(64),
    gateTrustSha256: "f".repeat(64),
    poolShapeSha256: "1".repeat(64),
    poolCredentialsSha256: "3".repeat(64),
    poolReadinessSha256: "4".repeat(64),
    poolResolvedEndpointsSha256: "0".repeat(64),
    authorityStaticSha256: "5".repeat(64),
    authoritySequenceSha256: "6".repeat(64),
    restoreBundleSha256: "2".repeat(64),
    restoreAuthorizationSha256: "7".repeat(64),
    remediationPlanSha256: "8".repeat(64),
    wifiCredentialsSha256: "9".repeat(64),
    allowedInterfaces: ["usb", "barrel_power"],
    forbiddenInterfaces: ["uart", "pins", "probes", "erasure", "ad_hoc_writes"],
    projection: "/tmp/projection.json",
    preflightDigestSha256: "",
  };
  document.preflightDigestSha256 = preflightDigest(document);

  // Act / Assert
  assert.equal(validatePreflight(document), document);
  assert.throws(() => validatePreflight({ ...document, scenario: "cancel" }));
});

test("protected inputs and detector admission fail closed", async () => {
  // Arrange
  const root = await mkdtemp(resolve(tmpdir(), "bwg-restoration-"));
  const detector = resolve(root, "detector.private.txt");
  const credentials = resolve(root, "pool.private.json");
  const readiness = resolve(root, "readiness.private.json");
  await writeFile(
    detector,
    "configuration_candidate: Bitaxe_A1B2\nport: /dev/cu.usbmodem1\nusb_session: ready\n",
    { mode: 0o600 },
  );
  await writeFile(credentials, JSON.stringify({
    poolURL: "pool.fixture.local",
    poolPort: 3333,
    poolUser: "fixture-user",
    poolPassword: "fixture-password",
  }), { mode: 0o600 });
  const credentialDigest = createHash("sha256").update(JSON.stringify({
    poolURL: "pool.fixture.local",
    poolPort: 3333,
    poolUser: "fixture-user",
    poolPassword: "fixture-password",
  })).digest("hex");
  await writeFile(readiness, JSON.stringify({
    schema_version: "bitaxe-pool-readiness-evidence-v1",
    attempt_ordinal: 5,
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    pool_config: "local-owner-supplied",
    pool_credentials_sha256: credentialDigest,
    private_lan_only: true,
    resolved_endpoints_sha256: "c".repeat(64),
    protocol: "stratum_v1_configure_subscribe_authorize",
    samples_required: 3,
    samples_completed: 3,
    ready_samples: 3,
    consecutive_ready: true,
    configure_succeeded: true,
    subscribe_succeeded: true,
    authorize_succeeded: true,
    shares_submitted: false,
    sample_timeout_seconds: 15,
    sample_delay_seconds: 2,
    max_server_bytes: 65_536,
    max_server_messages: 256,
    endpoint_redacted: true,
    credentials_redacted: true,
    bounded: true,
    terminal_category: "ready",
  }), { mode: 0o600 });

  // Act / Assert
  assert.match(await requireFreshDetector(detector), /^[0-9a-f]{64}$/);
  assert.match(await validatePoolCredentials(credentials), /^[0-9a-f]{64}$/);
  assert.match((await validatePoolReadiness(
    readiness,
    "a".repeat(40),
    "b".repeat(40),
    credentialDigest,
  )).digest, /^[0-9a-f]{64}$/);
  assert.equal(privateIpv4("192.168.1.2"), true);
  assert.equal(privateIpv4("127.0.0.1"), false);
  assert.equal(privateIpv4("8.8.8.8"), false);
  await chmod(detector, 0o644);
  await assert.rejects(requireProtectedFile(detector));
  const link = resolve(root, "link");
  await symlink(credentials, link);
  await assert.rejects(requireProtectedFile(link));
});

test("package and campaign paths remain confined to their exact repository seams", async () => {
  // Arrange
  const firmware = await mkdtemp(resolve(tmpdir(), "bwg-package-"));
  const manifestRoot = resolve(firmware, "bazel-bin/firmware/bitaxe");
  await mkdir(manifestRoot, { recursive: true });
  await mkdir(resolve(firmware, "firmware/bitaxe"), { recursive: true });
  const artifactPaths = {
    firmware_elf: "bitaxe-ultra205.elf",
    firmware_ota_image: "esp-miner.bin",
    www_spiffs_image: "www.bin",
    factory_merged_image: "bitaxe-ultra205-factory.bin",
    partition_table: "firmware/bitaxe/partitions-ultra205.csv",
    otadata_initial: "otadata-initial.bin",
  };
  const artifacts = [];
  for (const [kind, path] of Object.entries(artifactPaths)) {
    const contents = Buffer.from(`fixture-${kind}`);
    const target = kind === "partition_table"
      ? resolve(firmware, path)
      : resolve(manifestRoot, path);
    await writeFile(target, contents);
    artifacts.push({ kind, path, sha256: createHash("sha256").update(contents).digest("hex") });
  }
  const manifestPath = resolve(manifestRoot, "bitaxe-ultra205-package.json");
  const manifest = {
    schema_version: 3,
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    app_elf_sha256: artifacts[0].sha256,
    build_identity: { source_dirty: false },
    default_flash_image: "bitaxe-ultra205.elf",
    image_metadata: { board: "205", asic: "BM1366" },
    artifacts,
  };
  await writeFile(manifestPath, JSON.stringify(manifest));

  // Act / Assert
  assert.equal(
    (await validatePackage(manifestPath, "a".repeat(40), firmware)).manifest.source_commit,
    "a".repeat(40),
  );
  const attemptId = "bwg007-attempt-001";
  const scenario = "pause";
  const attemptRoot = resolve(firmware, `scratch/bwg-worker-restoration/${attemptId}`);
  assert.equal(validatePreflightScope({
    firmwareRepository: firmware,
    attemptId,
    scenario,
    recoveryRoot: resolve(attemptRoot, "recovery"),
    restoreAuthorization: resolve(attemptRoot, "recovery/restore-authorization.private.json"),
    packageManifest: manifestPath,
    remediationPlan: resolve(
      firmware,
      "docs/adr/0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md",
    ),
    projection: resolve(
      firmware,
      `docs/parity/evidence/bwg-worker-restoration/${attemptId}-${scenario}.json`,
    ),
  }, resolve(attemptRoot, "preflight.private.json")), attemptRoot);
  const escaped = structuredClone(manifest);
  escaped.artifacts[0].path = "../bitaxe-ultra205.elf";
  await writeFile(manifestPath, JSON.stringify(escaped));
  await assert.rejects(validatePackage(manifestPath, "a".repeat(40), firmware));
});

test("runtime identity capture binds two monotonic samples to the exact package", () => {
  // Arrange
  const source = "a".repeat(40);
  const reference = "b".repeat(40);
  const app = "c".repeat(64);
  const marker = (uptime) =>
    `runtime_boot_attestation schema_version=1 session=${"d".repeat(32)} boot_ordinal=7 ` +
    `reset_reason=other uptime_ms=${uptime} board=205 asic=BM1366 mining=disabled ` +
    `work_submission=disabled hardware_control=disabled firmware_commit=${source} ` +
    `reference_commit=${reference} app_elf_sha256=${app} esp_idf_version=v5.5.4 ` +
    "ota_boot_validation=complete spiffs_mount=available api_route_shell=started redacted=true";
  const capture = `${marker(10_000)}\n${marker(20_000)}\n`;

  // Act
  const admitted = validateRuntimeAttestationCapture(capture, {
    firmwareCommit: source,
    referenceCommit: reference,
    appElfSha256: app,
  });

  // Assert
  assert.equal(admitted.sampleCount, 2);
  assert.match(admitted.captureSha256, /^[0-9a-f]{64}$/);
  assert.throws(() => validateRuntimeAttestationCapture(marker(10_000), {
    firmwareCommit: source,
    referenceCommit: reference,
    appElfSha256: app,
  }));
});

test("atomic publication removes a linked target when temporary cleanup fails", async () => {
  // Arrange
  const root = await mkdtemp(resolve(tmpdir(), "bwg-publication-"));
  const target = resolve(root, "projection.json");
  let failedOnce = false;

  // Act
  const publication = writeAtomicNew(target, "{}\n", 0o644, {
    async unlinkFile(path) {
      if (!failedOnce && path !== target) {
        failedOnce = true;
        throw new Error("injected temporary cleanup failure");
      }
      return unlink(path);
    },
  });

  // Assert
  await assert.rejects(publication);
  await assert.rejects(lstat(target), { code: "ENOENT" });
});

test("atomic publication removes its temporary file when writing fails", async () => {
  // Arrange
  const root = await mkdtemp(resolve(tmpdir(), "bwg-publication-write-"));
  const target = resolve(root, "projection.json");

  // Act
  const publication = writeAtomicNew(target, "{}\n", 0o644, {
    async openFile(path, flags, mode) {
      const handle = await open(path, flags, mode);
      return {
        writeFile: async () => { throw new Error("injected write failure"); },
        sync: () => handle.sync(),
        close: () => handle.close(),
      };
    },
  });

  // Assert
  await assert.rejects(publication);
  assert.deepEqual(await readdir(root), []);
});

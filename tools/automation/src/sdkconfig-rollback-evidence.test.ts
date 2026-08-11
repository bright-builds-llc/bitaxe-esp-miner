import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureSdkconfigRollbackEvidence,
  SdkconfigRollbackEvidenceError,
  type SdkconfigRollbackEvidenceOptions,
} from "./sdkconfig-rollback-evidence.js";
import { createFakeProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const normalApp = "c".repeat(64);
const probeApp = "d".repeat(64);
const safeState = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";
const normalSession = "1".repeat(32);
const probeSession = "2".repeat(32);
const finalSession = "3".repeat(32);
const probeImage = Buffer.alloc(8_192, 0x5a);
const sha256 = (value: string | Buffer): string => createHash("sha256").update(value).digest("hex");
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly probeImage: string;
  readonly probeMetadata: string;
  readonly credentials: string;
  readonly projection: string;
};

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-rollback-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  const manifest = path.join(inputs, "package.json");
  const probeImagePath = path.join(inputs, "probe.bin");
  const probeMetadata = path.join(inputs, "probe.json");
  const credentials = path.join(inputs, "wifi.json");
  await writeFile(path.join(inputs, "bitaxe-firmware.sdkconfig"), [
    "CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y",
    "CONFIG_APP_ROLLBACK_ENABLE=y",
    "CONFIG_APP_PROJECT_VER=\"fixture\"",
    "CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64",
    "# CONFIG_BOOTLOADER_APP_ANTI_ROLLBACK is not set",
    "# CONFIG_APP_ANTI_ROLLBACK is not set",
    "",
  ].join("\n"));
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: normalApp,
    build_identity: { label: "fixture", source_dirty: false },
    artifacts: [{ kind: "factory_merged_image", path: "factory.bin", sha256: "e".repeat(64) }],
  }));
  await writeFile(probeImagePath, probeImage);
  await writeFile(probeMetadata, JSON.stringify({
    schema_version: "bitaxe-rollback-probe-v1",
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    source_dirty: false,
    build_label: "fixture",
    app_elf_sha256: probeApp,
    ota_image_sha256: sha256(probeImage),
    ota_image_bytes: probeImage.length,
    rollback_probe: true,
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    manifest,
    probeImage: probeImagePath,
    probeMetadata,
    credentials,
    projection: path.join(root, "docs", "projection.json"),
  };
}

function options(value: Fixture): SdkconfigRollbackEvidenceOptions {
  return {
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    rollbackProbeImage: value.probeImage,
    rollbackProbeMetadata: value.probeMetadata,
    wifiCredentials: value.credentials,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 360,
  };
}

function readyProjection(terminal = "ready"): Readonly<Record<string, unknown>> {
  return {
    schema_version: "esp-device-session-v1", terminal_category: terminal,
    platform_category: "macos", board_category: "205", same_physical_device: true,
    stable_enumeration: true, reenumerated: true, reader_armed: true,
    pre_restart_serial_delivery: true, post_restart_serial_delivery: true,
    serial_delivery: "correlated", request_outcome: "response_received",
    request_attempt_count: 1, service_loss_observed: true, trusted_origin_preserved: true,
    application_recovered: true, build_identity_matches: true, boot_session_changed: true,
    boot_ordinal_advanced_by_one: true, software_reset_observed: true,
    postcondition_matches: true, cleanup_complete: true, usb_disappearance_count: 1,
    enumeration_change_count: 1, serial_byte_count: 500, http_observation_count: 2,
    duration_millis: 1_000,
  };
}

async function writeCompletedEffect(spec: Parameters<ProcessPort["run"]>[0]): Promise<void> {
  const effectPath = String(spec.environment?.["PHASE36_EFFECT_RESULT_PATH"]);
  await writeFile(effectPath, `${JSON.stringify({
    schema_version: "phase36-effect-result-v1",
    operation: "exact_package_flash",
    status: "completed",
    failure: null,
    package_identity_digest: spec.environment?.["PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"],
    factory_image_digest: spec.environment?.["PHASE36_EFFECT_FACTORY_IMAGE_DIGEST"],
  })}\n`, { mode: 0o600 });
  await chmod(effectPath, 0o600);
}

function fakePort(origin: string, terminal = "ready", recoveryFails = false): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    const command = spec.args[0];
    if (command === "flash-monitor") {
      assert.equal(spec.args[spec.args.indexOf("--capture-timeout-seconds") + 1], "90");
      await writeCompletedEffect(spec);
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), [
        `runtime_boot_identity session=${normalSession} ordinal=7`,
        safeState,
        `runtime_origin session=${normalSession} device_url=${origin} redacted=true`,
        "",
      ].join("\n"), { mode: 0o600 });
      return ok();
    }
    if (command === "ota-live" || command === "reboot-live") {
      assert.equal(spec.args[spec.args.indexOf("--timeout-seconds") + 1], "360");
      const sessionRoot = String(spec.args[spec.args.indexOf("--private-root") + 1]);
      const projection = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
      const sessionTerminal = command === "ota-live" ? terminal : "ready";
      if (sessionTerminal !== "missing_projection") {
        const value = sessionTerminal === "malformed_projection" ? {} : readyProjection(sessionTerminal);
        await writeFile(projection, `${JSON.stringify(value)}\n`, { mode: 0o600 });
        await chmod(projection, 0o600);
      }
      await writeFile(path.join(sessionRoot, "serial.private.bin"), [
        safeState,
        ...(command === "ota-live" ? ["ota_boot_validation=rollback_probe_pending"] : []),
        "",
      ].join("\n"), { mode: 0o600 });
      return command === "ota-live" && terminal !== "ready" ? { ...ok(), exitCode: 1 } : ok();
    }
    if (command === "flash") return recoveryFails ? { ...ok(), exitCode: 1 } : ok();
    if (spec.program === "validator") return ok();
    throw new Error("unexpected child process");
  });
}

async function interruptedUploadServer(): Promise<{
  origin: string;
  close: (expectedResetCount?: number) => Promise<void>;
}> {
  let maybeUnexpectedError: Error | undefined;
  let resetCount = 0;
  const server = net.createServer((socket) => {
    socket.on("error", (error) => {
      if ((error as NodeJS.ErrnoException).code === "ECONNRESET") resetCount += 1;
      else maybeUnexpectedError = error;
    });
    socket.resume();
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (address === null || typeof address !== "object") throw new Error("fixture server address is unavailable");
  const port = address.port;
  return {
    origin: `http://127.0.0.1:${String(port)}`,
    close: async (expectedResetCount = 1) => {
      await new Promise<void>((resolve, reject) => {
        server.close((error) => error === undefined ? resolve() : reject(error));
      });
      if (maybeUnexpectedError !== undefined) throw maybeUnexpectedError;
      assert.equal(resetCount, expectedResetCount);
    },
  };
}

function installDeviceApi(
  baselineFailures = 0,
  maybeOnBaselineAttempt?: () => void,
) {
  let infoCall = 0;
  let remainingBaselineFailures = baselineFailures;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname === "/api/system/logs") {
      return new Response("firmware_ota_update=protocol_error code=fixture\n", { status: 200 });
    }
    assert.equal(target.pathname, "/api/system/info");
    if (infoCall === 0) {
      maybeOnBaselineAttempt?.();
      if (remainingBaselineFailures > 0) {
        remainingBaselineFailures -= 1;
        throw new Error("fixture baseline is temporarily unavailable");
      }
    }
    infoCall += 1;
    const values = [
      { app: normalApp, session: normalSession, ordinal: 7, partition: "factory" },
      { app: normalApp, session: normalSession, ordinal: 7, partition: "factory" },
      { app: probeApp, session: probeSession, ordinal: 8, partition: "ota_0" },
      { app: normalApp, session: finalSession, ordinal: 9, partition: "factory" },
    ];
    const current = values[infoCall - 1];
    assert.notEqual(current, undefined);
    return new Response(JSON.stringify({
      sourceCommit,
      referenceCommit,
      appElfSha256: current?.app,
      hostname: "private-hostname",
      bootSession: current?.session,
      bootOrdinal: current?.ordinal,
      runningPartition: current?.partition,
    }), { status: 200, headers: { "content-type": "application/json" } });
  };
  return () => {
    globalThis.fetch = originalFetch;
  };
}

async function captureError(promise: Promise<unknown>): Promise<SdkconfigRollbackEvidenceError> {
  try {
    await promise;
    assert.fail("expected SDK config rollback capture failure");
  } catch (error) {
    assert.ok(error instanceof SdkconfigRollbackEvidenceError);
    return error;
  }
}

test("ready interrupted-update transaction publishes aggregate-only rollback evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const server = await interruptedUploadServer();
  let baselineAttempts = 0;
  const restoreFetch = installDeviceApi(2, () => {
    baselineAttempts += 1;
  });
  try {
    // Act
    const evidence = await captureSdkconfigRollbackEvidence(
      value.root,
      options(value),
      fakePort(server.origin),
      "flash",
      "device-session",
      "validator",
    );
    const projection = await readFile(value.projection, "utf8");
    const baselineDocument = JSON.parse(await readFile(
      path.join(value.root, "scratch", "attempt", "baseline-system-info.private.json"),
      "utf8",
    )) as Readonly<Record<string, unknown>>;

    // Assert
    assert.equal(evidence.rollback.interruption_protocol_abort_observed, true);
    assert.equal(evidence.rollback.probe_pending_validation_observed, true);
    assert.equal(evidence.rollback.final_normal_build_restored, true);
    assert.equal(baselineAttempts, 3);
    assert.equal(baselineDocument["runningPartition"], "factory");
    assert.doesNotMatch(projection, /private-hostname|private-sensitive-port|127\.0\.0\.1|\/dev\//u);
  } finally {
    restoreFetch();
    await server.close();
  }
});

test("baseline HTTP readiness exhaustion is typed and triggers recovery", async () => {
  // Arrange
  const value = await fixture("baseline-exhausted");
  const server = await interruptedUploadServer();
  let baselineAttempts = 0;
  const restoreFetch = installDeviceApi(Number.MAX_SAFE_INTEGER, () => {
    baselineAttempts += 1;
  });
  try {
    // Act
    const error = await captureError(captureSdkconfigRollbackEvidence(
      value.root,
      options(value),
      fakePort(server.origin),
      "flash",
      "device-session",
      "validator",
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["recovery_complete"], true);
    assert.equal(error.publicValue["recovery_flash_used"], true);
    assert.equal(error.publicValue["secondary_recovery_failure"], false);
    assert.equal(baselineAttempts, 6);
    await assert.rejects(
      readFile(path.join(value.root, "scratch", "attempt", "baseline-system-info.private.json"), "utf8"),
      { code: "ENOENT" },
    );
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    restoreFetch();
    await server.close(0);
  }
});

test("non-ready probe preserves its primary category when exact-package recovery also fails", async () => {
  // Arrange
  const value = await fixture("blocked");
  const server = await interruptedUploadServer();
  const restoreFetch = installDeviceApi();
  try {
    // Act
    const error = await captureError(captureSdkconfigRollbackEvidence(
      value.root,
      options(value),
      fakePort(server.origin, "service_recovery_timeout", true),
      "flash",
      "device-session",
      "validator",
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["terminal_category"], "service_recovery_timeout");
    assert.equal(error.publicValue["recovery_flash_used"], true);
    assert.equal(error.publicValue["secondary_recovery_failure"], true);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    restoreFetch();
    await server.close();
  }
});

test("every non-ready device-session category withholds final evidence", async () => {
  // Arrange
  const categories = [
    "incomplete", "observer_unqualified", "restart_request_not_sent", "restart_attribution_ambiguous",
    "usb_identity_unavailable", "usb_identity_drift", "service_recovery_timeout", "boot_identity_invalid",
    "build_identity_mismatch", "session_not_advanced", "reset_reason_wrong", "ordinal_not_next",
    "postcondition_mismatch",
  ];

  for (const category of categories) {
    const value = await fixture(category);
    const server = await interruptedUploadServer();
    const restoreFetch = installDeviceApi();
    try {
      // Act
      const error = await captureError(captureSdkconfigRollbackEvidence(
        value.root,
        options(value),
        fakePort(server.origin, category),
        "flash",
        "device-session",
        "validator",
      ));

      // Assert
      assert.equal(error.category, "hardware_blocked");
      assert.equal(error.publicValue["terminal_category"], category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restoreFetch();
      await server.close();
    }
  }
});

for (const projectionFailure of ["missing_projection", "malformed_projection"]) {
  test(`${projectionFailure} is evidence-invalid and triggers bounded recovery`, async () => {
    // Arrange
    const value = await fixture(projectionFailure);
    const server = await interruptedUploadServer();
    const restoreFetch = installDeviceApi();
    try {
      // Act
      const error = await captureError(captureSdkconfigRollbackEvidence(
        value.root,
        options(value),
        fakePort(server.origin, projectionFailure),
        "flash",
        "device-session",
        "validator",
      ));

      // Assert
      assert.equal(error.category, "evidence_invalid");
      assert.equal(error.publicValue["recovery_complete"], true);
      assert.equal(error.publicValue["recovery_flash_used"], true);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restoreFetch();
      await server.close();
    }
  });
}

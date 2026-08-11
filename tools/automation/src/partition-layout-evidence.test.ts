import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  capturePartitionLayoutEvidence,
  PartitionLayoutEvidenceError,
  type PartitionLayoutEvidenceOptions,
} from "./partition-layout-evidence.js";
import { createFakeProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const privateOrigin = "http://private-device.test";
const safeState = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";
const partitionTable = [
  "# Name, Type, SubType, Offset, Size",
  "nvs, data, nvs, 0x9000, 0x6000",
  "phy_init, data, phy, 0xf000, 0x1000",
  "factory, app, factory, 0x10000, 4M",
  "www, data, spiffs, 0x410000, 3M",
  "ota_0, app, ota_0, 0x710000, 4M",
  "ota_1, app, ota_1, 0xb10000, 4M",
  "otadata, data, ota, 0xf10000, 8K",
  "coredump, data, coredump, , 64K",
  "",
].join("\n");
const otaImage = Buffer.from("exact-ota-image", "utf8");
const sha256 = (value: string | Buffer): string => createHash("sha256").update(value).digest("hex");
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly credentials: string;
  readonly projection: string;
};

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-partition-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  const manifest = path.join(inputs, "package.json");
  const credentials = path.join(inputs, "wifi.json");
  await writeFile(path.join(inputs, "ota.bin"), otaImage);
  await writeFile(path.join(inputs, "partitions.csv"), partitionTable);
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    artifacts: [
      { kind: "factory_merged_image", path: "factory.bin", sha256: "d".repeat(64) },
      { kind: "firmware_ota_image", path: "ota.bin", sha256: sha256(otaImage) },
      { kind: "partition_table", path: "partitions.csv", sha256: sha256(partitionTable) },
    ],
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return { root, manifest, credentials, projection: path.join(root, "docs", "projection.json") };
}

function options(value: Fixture): PartitionLayoutEvidenceOptions {
  return {
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    wifiCredentials: value.credentials,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 360,
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
    duration_millis: 1000,
  };
}

function fakePort(terminal = "ready"): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      await writeCompletedEffect(spec);
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      const monitor = [
        "runtime_boot_identity session=11111111111111111111111111111111 ordinal=7",
        safeState,
        `runtime_origin session=11111111111111111111111111111111 device_url=${privateOrigin} redacted=true`,
        "",
      ].join("\n");
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), monitor, { mode: 0o600 });
      return ok();
    }
    if (spec.args[0] === "ota-live") {
      assert.ok(spec.args.indexOf("--ota-image") > spec.args.indexOf("--intent-input"));
      const sessionRoot = String(spec.args[spec.args.indexOf("--private-root") + 1]);
      const projection = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
      await writeFile(projection, `${JSON.stringify(readyProjection(terminal))}\n`, { mode: 0o600 });
      await chmod(projection, 0o600);
      await writeFile(path.join(sessionRoot, "serial.private.bin"), [
        safeState,
        "runtime_boot_attestation schema_version=1 ota_boot_validation=complete redacted=true",
        "",
      ].join("\n"), { mode: 0o600 });
      return terminal === "ready" ? ok() : { ...ok(), exitCode: 1 };
    }
    if (spec.program === "validator") return ok();
    throw new Error("unexpected child process");
  });
}

function installDeviceApi() {
  let call = 0;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    assert.equal(target.pathname, "/api/system/info");
    call += 1;
    return new Response(JSON.stringify({
      sourceCommit,
      referenceCommit,
      appElfSha256,
      hostname: "private-hostname",
      bootSession: call === 1 ? "1".repeat(32) : "2".repeat(32),
      bootOrdinal: call === 1 ? 7 : 8,
      runningPartition: call === 1 ? "factory" : "ota_0",
    }), { status: 200, headers: { "content-type": "application/json" } });
  };
  return () => {
    globalThis.fetch = originalFetch;
  };
}

async function captureError(promise: Promise<unknown>): Promise<PartitionLayoutEvidenceError> {
  try {
    await promise;
    assert.fail("expected partition capture failure");
  } catch (error) {
    assert.ok(error instanceof PartitionLayoutEvidenceError);
    return error;
  }
}

test("ready OTA session publishes aggregate-only partition evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const restore = installDeviceApi();

  try {
    // Act
    const evidence = await capturePartitionLayoutEvidence(
      value.root,
      options(value),
      fakePort(),
      "flash",
      "device-session",
      "validator",
    );
    const projection = await readFile(value.projection, "utf8");

    // Assert
    assert.equal(evidence.partition_layout["factory_baseline_observed"], true);
    assert.equal(evidence.partition_layout["ota_0_recovered"], true);
    assert.equal(evidence.ota_session["request_attempt_count"], 1);
    assert.doesNotMatch(projection, /private-device|private-hostname|private-sensitive-port|\/dev\//u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
  } finally {
    restore();
  }
});

test("non-ready OTA session withholds evidence with its closed category", async () => {
  // Arrange
  const value = await fixture("blocked");
  const restore = installDeviceApi();

  try {
    // Act
    const error = await captureError(capturePartitionLayoutEvidence(
      value.root,
      options(value),
      fakePort("service_recovery_timeout"),
      "flash",
      "device-session",
      "validator",
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["terminal_category"], "service_recovery_timeout");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    restore();
  }
});

import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { verifySemanticEvidenceRedaction } from "./redaction.js";

test("semantic evidence scanner accepts digests and rejects operational fields", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-"));
  await mkdir(path.join(root, "safe"));
  await writeFile(path.join(root, "safe", "evidence.json"), JSON.stringify({
    schema_version: "bitaxe-version-evidence-v1",
    package_manifest_sha256: "0".repeat(64),
    same_origin_api_observed: true,
  }));

  // Act / Assert
  assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
  await writeFile(path.join(root, "unsafe.json"), JSON.stringify({
    schema_version: "bitaxe-version-evidence-v1",
    device_url: "http://192.0.2.1",
  }));
  await assert.rejects(verifySemanticEvidenceRedaction(root));
  await rm(root, { recursive: true });
});

test("settings PATCH baseline digest keys do not collide with the origin denylist", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-settings-patch-"));
  await writeFile(path.join(root, "evidence.json"), JSON.stringify({
    schema_version: "bitaxe-settings-patch-evidence-v1",
    settings_patch: {
      hostname_baseline_sha256: "0".repeat(64),
      hostname_candidate_sha256: "1".repeat(64),
      rotation_baseline_sha256: "2".repeat(64),
      rotation_candidate_sha256: "3".repeat(64),
    },
  }));

  try {
    // Act
    const result = await verifySemanticEvidenceRedaction(root);

    // Assert
    assert.equal(result.checked, 1);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("log buffer evidence is admitted and operational device fields are rejected", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-log-buffer-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-log-buffer-evidence-v1",
    same_origin_observed: true,
    log_buffer: { raw_frame_sha256: "0".repeat(64) },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-log-buffer-evidence-v1",
      usb_port: "/dev/private-device",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("network scan evidence admits aggregates and rejects raw radio identity", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-network-scan-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-network-scan-evidence-v1",
    same_origin_observed: true,
    scan: {
      record_count: 2,
      address_family: "v6",
      address_kind: "link_local",
    },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-network-scan-evidence-v1",
      ssid: "private-nearby-network",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("ASIC initialization admits the exact single-chip boolean only", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-asic-init-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-asic-initialization-evidence-v1",
    initialization: { exactly_one_chip_detected: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-asic-initialization-evidence-v1",
      ip: "192.0.2.1",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("ASIC reset evidence is included in the operational-field scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-asic-reset-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-asic-reset-evidence-v1",
    reset: { exactly_one_chip_detected_after_reset: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-asic-reset-evidence-v1",
      serial_port: "/dev/private-device",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("ASIC power initialization evidence is included in the operational-field scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-asic-power-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-asic-power-initialization-evidence-v1",
    power_initialization: { exactly_one_chip_detected_after_reset: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-asic-power-initialization-evidence-v1",
      usb_port: "/dev/private-device",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("core-voltage-control evidence is included in the operational-field scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-core-voltage-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-core-voltage-control-evidence-v1",
    voltage_control: { target_millivolts: 1_100, compatible_path_count: 5 },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-core-voltage-control-evidence-v1",
      ip_address: "redacted",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

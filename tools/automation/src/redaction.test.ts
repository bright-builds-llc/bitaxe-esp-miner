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

test("scoreboard v2 retained identity is included in the redaction scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-scoreboard-v2-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-scoreboard-evidence-v2",
    capture_package_identity_sha256: "0".repeat(64),
    scoreboard: { post_restart_persistence: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-scoreboard-evidence-v2",
      poolUser: "private-worker",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("UI workflow evidence admits only closed origin and write-only-secret facts", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-ui-workflow-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-ui-workflow-evidence-v1",
    browser: {
      same_origin_requests_observed: true,
      write_only_secrets_blank: true,
    },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-ui-workflow-evidence-v1",
      device_url: "redacted",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
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

test("INA260 evidence is included in the operational-field scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-ina260-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-ina260-evidence-v1",
    telemetry: { i2c_address: 64, same_values: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-ina260-evidence-v1",
      serial_port: "/dev/private-device",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

test("EMC2101 thermal evidence is included in the operational-field scan", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-emc2101-"));
  const evidence = path.join(root, "evidence.json");
  await writeFile(evidence, JSON.stringify({
    schema_version: "bitaxe-emc2101-thermal-evidence-v1",
    thermal: { i2c_address: 76, same_temperature: true },
  }));

  try {
    // Act / Assert
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    await writeFile(evidence, JSON.stringify({
      schema_version: "bitaxe-emc2101-thermal-evidence-v1",
      device_url: "redacted",
    }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally {
    await rm(root, { recursive: true });
  }
});

for (const schema of ["fixed-usb-cycle-report-v1", "fixed-usb-window-report-v1"]) {
  test(`${schema} scans closed reports and rejects private Worker fingerprints`, async () => {
    // Arrange
    const root = await mkdtemp(path.join(tmpdir(), "bitaxe-redaction-fixed-usb-"));
    const evidence = path.join(root, "report.json");
    try {
      await writeFile(evidence, JSON.stringify({ schema, settings_match: true, active_ms: 1000 }));
      // Act / Assert
      assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
      for (const key of ["device_identity_sha256", "settings_before_sha256", "authorization_high_water_sha256", "poolUser"]) {
        await writeFile(evidence, JSON.stringify({ schema, [key]: "0".repeat(64) }));
        await assert.rejects(verifySemanticEvidenceRedaction(root));
      }
    } finally {
      await rm(root, { recursive: true });
    }
  });
}

test("Noise v2 public schema admits only closed safe fields and amended cleanup timing", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "noise-redaction-"));
  const value = {
    schema_version: "bitaxe-stratum-v2-noise-serial-projection-v2", status: "accepted", board: 205, diagnostic_ordinal: 1,
    source_commit: "a".repeat(40), gate_commit: "b".repeat(40), reference_commit: "c".repeat(40),
    provenance: Object.fromEntries(["app_elf", "package_manifest", "contract", "fixture", "evaluator", "sealed_inventory", "private_result"].map(key => [key, "d".repeat(64)])),
    criteria: Object.fromEntries(["identity", "continuity", "authority", "tcp_delivery", "noise_authentication", "encrypted_proof", "no_new_work", "preservation", "accounting", "restoration", "cleanup", "privacy"].map(key => [key, true])),
    timings_ms: { preparation: 100, connect: 10, act_one_write: 1, act_two_read: 20, proof_write: 1, diagnostic: 10000, device_cleanup: 6000, fixture_lifetime: 20000, host_cleanup: 100 },
    counts: { exact_peer_connections: 1, unexpected_peer_connections: 0, act_one_written: 64, act_one_received: 64, proof_written: 22, proof_received: 22, new_work: 0, new_shares: 0 }, redaction_status: "passed",
  };
  const target = path.join(root, "projection.json");
  try {
    // Act / Assert
    await writeFile(target, JSON.stringify(value));
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    for (const extra of [{ endpoint: "sensitive" }, { observation: "unclassified" }, { private_fingerprint: "f".repeat(64) }]) {
      await writeFile(target, JSON.stringify({ ...value, ...extra }));
      await assert.rejects(verifySemanticEvidenceRedaction(root));
    }
    await writeFile(target, JSON.stringify({ ...value, timings_ms: { ...value.timings_ms, device_cleanup: 120001 } }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally { await rm(root, { recursive: true, force: true }); }
});


test("V2 serial projection uses its versioned closed parser before privacy scanning", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "v2-serial-redaction-")), target = path.join(root, "projection.json");
  const value = {
    schema: "str005-v2-serial-projection-v1", scope: "channel", status: "accepted", board: 205, hostOrdinal: 1,
    sourceCommit: "a".repeat(40), gateCommit: "b".repeat(40),
    provenance: { ...Object.fromEntries(["contract", "contracts", "appElf", "packageManifest", "evaluator", "fixture", "observer", "privateResult", "sealedInventory"].map(key => [key, "c".repeat(64)])), channelResult: null, channelSeal: null },
    criteria: Object.fromEntries(["identity", "continuity", "authority", "standardChannel", "targetAndJob", "accounting", "preservation", "restoration", "cleanup", "privacy", "noAsicWork", "noReservation"].map(key => [key, true])),
    counts: { installations: 5, continuityCycles: 4, connections: 1, deviceRecords: 3, submitted: 0, deviceAcknowledged: 0, workDispatched: 0 },
    timings: { preparationUs: 10, maximumReadUs: 10, maximumWriteUs: 10, connectUs: 10, resourceReleaseUs: 10, hostCleanupMs: 10 },
    nonClaims: ["external-pool-acceptance", "private-socket-tuple-reconstruction", "complete-native-callgraph-bound", "asic-mining", "accepted-share", "funded-work"], redactionStatus: "passed",
  };
  try {
    // Act / Assert
    await writeFile(target, JSON.stringify(value));
    assert.equal((await verifySemanticEvidenceRedaction(root)).checked, 1);
    for (const extra of [{ endpoint: "private" }, { attemptId: "private" }, { privateFingerprint: "d".repeat(64) }]) {
      await writeFile(target, JSON.stringify({ ...value, ...extra }));
      await assert.rejects(verifySemanticEvidenceRedaction(root));
    }
    await writeFile(target, JSON.stringify({ ...value, counts: { ...value.counts, submitted: 1 } }));
    await assert.rejects(verifySemanticEvidenceRedaction(root));
  } finally { await rm(root, { recursive: true, force: true }); }
});

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

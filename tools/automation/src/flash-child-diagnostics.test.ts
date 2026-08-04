import assert from "node:assert/strict";
import { chmod, mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  flashMonitorTerminalMarker,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";

const packageDigest = "a".repeat(64);
const factoryDigest = "b".repeat(64);
const expected = { packageIdentityDigest: packageDigest, factoryImageDigest: factoryDigest };

function effect(status: "completed" | "failed_no_device_effect" = "completed") {
  return {
    schema_version: "phase36-effect-result-v1",
    operation: "exact_package_flash",
    status,
    failure: status === "completed" ? null : "flash_failed",
    package_identity_digest: packageDigest,
    factory_image_digest: factoryDigest,
  };
}

test("factory image and effect environment parse exact package identities", () => {
  // Arrange
  const manifest = { artifacts: [{ kind: "factory_merged_image", sha256: factoryDigest }] };

  // Act
  const digest = factoryImageDigest(manifest);
  const environment = flashEffectEnvironment("/private/effect.json", expected);

  // Assert
  assert.equal(digest, factoryDigest);
  assert.deepEqual(environment, {
    PHASE36_EFFECT_RESULT_PATH: "/private/effect.json",
    PHASE36_EFFECT_OPERATION: "exact_package_flash",
    PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST: packageDigest,
    PHASE36_EFFECT_FACTORY_IMAGE_DIGEST: factoryDigest,
  });
});

test("effect inspection distinguishes valid missing and malformed private artifacts", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-flash-effect-"));
  const validPath = path.join(root, "valid.json");
  const malformedPath = path.join(root, "malformed.json");
  await writeFile(validPath, `${JSON.stringify(effect("failed_no_device_effect"))}\n`, { mode: 0o600 });
  await chmod(validPath, 0o600);
  await writeFile(malformedPath, "{}\n", { mode: 0o600 });
  await chmod(malformedPath, 0o600);

  // Act
  const valid = await inspectFlashEffect(validPath, expected);
  const missing = await inspectFlashEffect(path.join(root, "missing.json"), expected);
  const malformed = await inspectFlashEffect(malformedPath, expected);

  // Assert
  assert.deepEqual(valid, {
    flash_effect_result_status: "valid",
    flash_effect_status: "failed_no_device_effect",
  });
  assert.deepEqual(missing, {
    flash_effect_result_status: "missing",
    flash_effect_status: "unavailable",
  });
  assert.deepEqual(malformed, {
    flash_effect_result_status: "invalid",
    flash_effect_status: "unavailable",
  });
});

test("pre-effect invocation failure remains a valid no-device-effect result", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-flash-pre-effect-"));
  const resultPath = path.join(root, "effect.json");
  const result = {
    ...effect("failed_no_device_effect"),
    failure: "invocation_construction_failed",
  };
  await writeFile(resultPath, `${JSON.stringify(result)}\n`, { mode: 0o600 });
  await chmod(resultPath, 0o600);

  // Act
  const inspected = await inspectFlashEffect(resultPath, expected);

  // Assert
  assert.deepEqual(inspected, {
    flash_effect_result_status: "valid",
    flash_effect_status: "failed_no_device_effect",
  });
});

test("terminal classification emits only allowlisted markers and bounded process facts", () => {
  // Arrange
  const stderr = "private /dev/value dual_evidence=failed reason=flash_workflow_failed token=secret";

  // Act
  const marker = flashMonitorTerminalMarker(stderr);
  const facts = flashChildFailureFacts({ exitCode: 17, stdout: "private", stderr, timedOut: false }, {
    flash_effect_result_status: "valid",
    flash_effect_status: "failed_no_device_effect",
  });

  // Assert
  assert.equal(marker, "flash_workflow_failed");
  assert.deepEqual(facts, {
    stage: "initial_flash_monitor",
    flash_monitor_exit_code: 17,
    flash_monitor_timed_out: false,
    flash_monitor_terminal_marker: "flash_workflow_failed",
    flash_effect_result_status: "valid",
    flash_effect_status: "failed_no_device_effect",
  });
  assert.doesNotMatch(JSON.stringify(facts), /private|secret|\/dev/u);
});

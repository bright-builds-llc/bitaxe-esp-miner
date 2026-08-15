import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  DetectorHandoffError,
  portFromDetectorOutput,
  provisioningDetectorHandoffFromOutput,
} from "./detector.js";

test("detector handoff accepts one protected typed port line", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-detector-output-"));
  const output = path.join(workspace, "detector.stdout");
  await writeFile(output, "espflash_version: 4.3.0\nport: /dev/cu.usbmodem-test\nusb_session: ready\n", { mode: 0o600 });

  try {
    // Act
    const port = await portFromDetectorOutput(workspace, "detector.stdout");

    // Assert
    assert.equal(port, "/dev/cu.usbmodem-test");
  } finally {
    await rm(workspace, { recursive: true });
  }
});

test("provisioning detector handoff accepts one private exact-device candidate", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-detector-output-"));
  const output = path.join(workspace, "detector.stdout");
  await writeFile(
    output,
    "configuration_candidate: Bitaxe_A1B2\nport: /dev/cu.usbmodem-test\nusb_session: ready\n",
    { mode: 0o600 },
  );

  try {
    // Act
    const handoff = await provisioningDetectorHandoffFromOutput(workspace, "detector.stdout");

    // Assert
    assert.deepEqual(handoff, {
      port: "/dev/cu.usbmodem-test",
      configurationCandidate: "Bitaxe_A1B2",
    });
  } finally {
    await rm(workspace, { recursive: true });
  }
});

test("provisioning detector handoff rejects missing duplicate and malformed candidates", async () => {
  for (const candidates of [
    "",
    "configuration_candidate: private\n",
    "configuration_candidate: Bitaxe_A1B2\nconfiguration_candidate: Bitaxe_C3D4\n",
  ]) {
    // Arrange
    const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-detector-output-"));
    const output = path.join(workspace, "detector.stdout");
    await writeFile(output, `${candidates}port: /dev/cu.usbmodem-test\n`, { mode: 0o600 });

    try {
      // Act / Assert
      await assert.rejects(provisioningDetectorHandoffFromOutput(workspace, "detector.stdout"));
    } finally {
      await rm(workspace, { recursive: true });
    }
  }
});

test("detector handoff rejects the obsolete equals-delimited line", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-detector-output-"));
  const output = path.join(workspace, "detector.stdout");
  await writeFile(output, "port=/dev/cu.usbmodem-test\n", { mode: 0o600 });

  try {
    // Act / Assert
    await assert.rejects(portFromDetectorOutput(workspace, "detector.stdout"), {
      message: "detector output must contain exactly one admitted port",
    });
  } finally {
    await rm(workspace, { recursive: true });
  }
});

test("missing detector handoff is typed evidence failure before workflow launch", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(tmpdir(), "bitaxe-detector-output-"));

  try {
    // Act / Assert
    await assert.rejects(portFromDetectorOutput(workspace, "missing.stdout"), (error: unknown) => {
      assert.ok(error instanceof DetectorHandoffError);
      assert.equal(error.category, "evidence_invalid");
      assert.deepEqual(error.publicValue, { detector_admitted: false });
      return true;
    });
  } finally {
    await rm(workspace, { recursive: true });
  }
});

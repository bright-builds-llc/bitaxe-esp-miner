import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { portFromDetectorOutput } from "./detector.js";

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

import test from "node:test";
import assert from "node:assert/strict";
import { chmod, stat, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { workspaceFixture } from "./fixtures.test-helper.mjs";
import { heapCapture, windowNames } from "./heap-capture.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);
const thresholds = { minFreeBytes: 16384, minLargestBlockBytes: 8192, minSamples: 2 };

test("chained windows are captured privately and judged together", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  // Act
  const result = await heapCapture({ parent: fixture.parent, name: "idle", windows: 2, seconds: 1200, maybeThresholds: thresholds }, fixture.operations);
  // Assert
  assert.deepEqual(result.windows.map((window) => [window.window, window.samples]), [["idle-001", 2], ["idle-002", 2]]);
  assert.equal(result.judgement.passed, true);
  assert.equal((await stat(resolve(fixture.parent, "idle-002.capture.log"))).mode & 0o777, 0o600);
});

test("each window's monitor is receive-only with the detected port and identity", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  // Act
  await heapCapture({ parent: fixture.parent, name: "idle", windows: 1, seconds: 360 }, fixture.operations);
  // Assert
  assert.deepEqual((await fixture.calls())[1], ["monitor", "--board", "205", "--port", "/dev/cu.usbmodemFAKE1",
    "--expected-physical-sha256", "a".repeat(64), "--capture-timeout-seconds", "360"]);
});

test("an exhausted series fails its judgement", async () => {
  // Arrange
  const fixture = await workspaceFixture({ freeBytes: 2631 });
  // Act
  const result = await heapCapture({ parent: fixture.parent, name: "idle", windows: 1, seconds: 360, maybeThresholds: thresholds }, fixture.operations);
  // Assert
  assert.deepEqual(result.judgement.failures, ["free_below_minimum"]);
});

test("a device change between windows stops the chain", async () => {
  // Arrange
  const fixture = await workspaceFixture({ physicals: ["a".repeat(64), "e".repeat(64)] });
  // Act / Assert
  await rejectsWith(heapCapture({ parent: fixture.parent, name: "idle", windows: 2, seconds: 360 }, fixture.operations), "physical_identity_changed");
});

test("an existing capture file for any planned window is refused before the first detector", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  await writeFile(resolve(fixture.parent, "idle-003.capture.log"), "", { mode: 0o600 });
  // Act / Assert
  await rejectsWith(heapCapture({ parent: fixture.parent, name: "idle", windows: 3, seconds: 360 }, fixture.operations), "heap_capture_output_exists");
  assert.deepEqual(await fixture.calls(), []);
});

test("a parent that is not owner-only is refused", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  await chmod(fixture.parent, 0o750);
  // Act / Assert
  await rejectsWith(heapCapture({ parent: fixture.parent, name: "idle", windows: 1, seconds: 360 }, fixture.operations), "private_directory_mode");
});

test("windows shorter than the hardware monitor minimum are refused", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  // Act / Assert
  await rejectsWith(heapCapture({ parent: fixture.parent, name: "idle", windows: 1, seconds: 359 }, fixture.operations), "heap_capture_seconds");
});

test("window names are numbered and bounded", () => {
  // Arrange / Act
  const names = windowNames("phase-d", 3);
  // Assert
  assert.deepEqual(names, ["phase-d-001", "phase-d-002", "phase-d-003"]);
  assert.throws(() => windowNames("../x", 1), (error) => error.code === "heap_capture_name");
});

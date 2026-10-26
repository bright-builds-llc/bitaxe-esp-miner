import assert from "node:assert/strict";
import test from "node:test";

import { selfTestFlashMonitorSpec } from "./self-test-campaign-flash.js";
import {
  selfTestCampaignProfile,
  testFirmwareArgs,
  ULTRA205_SELF_TEST_PROFILE,
} from "./self-test-campaign-profile.js";

test("the 205's campaign keeps its attempt-005 contract with or without --board", () => {
  // Act
  const profiles = [undefined, "205"].map(board =>
    selfTestCampaignProfile(board, "scratch/self001-full-lifecycle/attempt-005"));
  // Assert
  assert.deepEqual(profiles, [ULTRA205_SELF_TEST_PROFILE, ULTRA205_SELF_TEST_PROFILE]);
});

test("a 601 campaign takes its attempt from its own root", () => {
  // Act
  const profile = selfTestCampaignProfile("601", "scratch/self001-gamma601/attempt-002");
  // Assert
  assert.deepEqual(
    [profile?.board, profile?.attemptOrdinal, profile?.frequencyMhz, profile?.coreVoltageMv],
    [601, 2, 525, 1_150],
  );
  assert.equal(
    profile?.projection,
    "docs/parity/evidence/self001-gamma601/attempt-002/self-test-projection.json",
  );
  assert.equal(profile?.activeTask, "task-gamma-601-bring-up");
});

test("a root outside the board's contract is refused", () => {
  // Act
  const profiles = [
    selfTestCampaignProfile("601", "scratch/self001-full-lifecycle/attempt-005"),
    selfTestCampaignProfile("205", "scratch/self001-gamma601/attempt-001"),
    selfTestCampaignProfile("601", "scratch/self001-gamma601/attempt-000"),
    selfTestCampaignProfile("700", "scratch/self001-gamma601/attempt-001"),
  ];
  // Assert
  assert.deepEqual(profiles, [null, null, null, null]);
});

test("only the 601's mining build is admitted as test firmware", () => {
  // Act
  const args = [
    testFirmwareArgs(601, { firmware_variant: "gamma601-mining" }),
    testFirmwareArgs(601, { firmware_variant: "production" }),
    testFirmwareArgs(205, {}),
    testFirmwareArgs(601, { firmware_variant: "gamma601-overheat-trip-test" }),
    testFirmwareArgs(205, { firmware_variant: "overheat-trip-test" }),
    testFirmwareArgs(205, { firmware_variant: "gamma601-mining" }),
  ];
  // Assert
  assert.deepEqual(args, [["--allow-test-firmware", "gamma601-mining"], [], [], null, null, null]);
});

test("the 601's self-test install names its board and its test variant", () => {
  // Act
  const spec = selfTestFlashMonitorSpec(
    { board: 601, testFirmwareArgs: ["--allow-test-firmware", "gamma601-mining"] },
    "flash", "/dev/private", "package.json", "wifi.json", "intent.json", "evidence",
  );
  // Assert
  assert.deepEqual(spec.args.slice(0, 10), [
    "flash-monitor", "--board", "601", "--port", "/dev/private", "--manifest", "package.json",
    "--allow-test-firmware", "gamma601-mining", "--factory-reset",
  ]);
});

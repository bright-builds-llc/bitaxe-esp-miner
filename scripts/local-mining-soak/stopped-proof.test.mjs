import assert from "node:assert/strict";
import test from "node:test";
import { GAMMA601_LIMITS } from "./judge.mjs";
import { limitsForBoard, STOPPED_PROOF_SCHEMA, stoppedProof } from "./stopped-proof.mjs";

const INSTALLED = { firmwareCommit: "c".repeat(40), appElfSha256: "e".repeat(64) };
const DETECTOR = { port: "/dev/cu.usbmodem-test", physical: "6".repeat(64) };

/** Paused, powered-down device info; `overrides(index)` changes the nth reading. */
function client(overrides = () => ({})) {
  let reads = 0;
  return { async info() {
    const index = reads++;
    return { sourceCommit: INSTALLED.firmwareCommit, appElfSha256: INSTALLED.appElfSha256, bootSession: "a".repeat(32), bootOrdinal: 8,
      miningPaused: true, miningActivity: "paused", startMiningOnBoot: false, power: 0.4, powerStatus: { state: "fresh" },
      voltageStatus: { state: "fresh" }, chipTempStatus: { state: "fresh" }, fanRpmStatus: { state: "fresh" }, ...overrides(index) };
  } };
}

const build = (device, requireReleased = () => {}) => stoppedProof({ client: device, installed: INSTALLED, bootOrdinal: 8, detector: DETECTOR,
  sourceCommit: "d".repeat(40), requireReleased, sleep: async () => {}, now: () => 1_000 });

test("a paused, powered-down device yields the stopped proof", async () => {
  // Arrange / Act
  const proof = await build(client());
  // Assert
  assert.equal(proof.schema, STOPPED_PROOF_SCHEMA);
  assert.equal(proof.physical_identity_sha256, DETECTOR.physical);
  assert.equal(proof.stopped_samples, 2);
});

test("an ASIC not powered down on either sample refuses", async () => {
  // Arrange
  const device = client((index) => (index === 1 ? { power: 11.8 } : {}));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_not_stopped" });
});

test("an ASIC not powered down on the first sample refuses", async () => {
  // Arrange
  const device = client((index) => (index === 0 ? { power: 11.8 } : {}));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_not_stopped" });
});

test("a stale power reading is not a powered-down ASIC", async () => {
  // Arrange
  const device = client(() => ({ powerStatus: { state: "stale" } }));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_not_stopped" });
});

test("a boot other than the handed-off one refuses", async () => {
  // Arrange: the device rebooted after the handoff, or another device answers.
  const device = client(() => ({ bootOrdinal: 9 }));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_boot_ordinal" });
});

test("the proof records the injected observation time", async () => {
  // Arrange / Act
  const proof = await build(client());
  // Assert
  assert.equal(proof.observed_at_unix_ms, 1_000);
});

test("a reset between the samples refuses", async () => {
  // Arrange
  const device = client((index) => (index === 1 ? { bootSession: "b".repeat(32) } : {}));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_boot" });
});

test("other installed firmware refuses", async () => {
  // Arrange
  const device = client(() => ({ appElfSha256: "f".repeat(64) }));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_identity" });
});

test("mining on boot refuses", async () => {
  // Arrange
  const device = client(() => ({ startMiningOnBoot: true }));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_mine_on_boot" });
});

test("a serial holder refuses", async () => {
  // Arrange
  const held = () => { throw Object.assign(new Error("held"), { code: "noise_resource_present" }); };
  // Act / Assert
  await assert.rejects(build(client(), held), { code: "noise_resource_present" });
});

test("a powered-down 601 at 5.6 W, inside its regulator offset, yields the proof under the 601's limits", async () => {
  // Arrange
  const device = client(() => ({ power: 5.6 }));
  // Act
  const proof = await stoppedProof({ client: device, installed: INSTALLED, bootOrdinal: 8, detector: DETECTOR,
    sourceCommit: "d".repeat(40), requireReleased: () => {}, sleep: async () => {}, now: () => 1_000, limits: GAMMA601_LIMITS });
  // Assert
  assert.equal(proof.asic_stopped, true);
});

test("a 601 at 5.6 W refuses under the 205's limits", async () => {
  // Arrange
  const device = client(() => ({ power: 5.6 }));
  // Act / Assert
  await assert.rejects(build(device), { code: "stopped_proof_not_stopped" });
});

test("the installed manifest's board chooses the limits", () => {
  // Act
  const chosen = ["601", "205", undefined].map(limitsForBoard);
  // Assert
  assert.deepEqual(chosen.map((limits) => limits.idlePowerW), [GAMMA601_LIMITS.idlePowerW, 5, 5]);
});

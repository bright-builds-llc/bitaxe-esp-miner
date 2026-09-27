import test from "node:test";
import assert from "node:assert/strict";
import { correctionJudgment } from "./correction-judge.mjs";
import { syntheticTimingV2, syntheticCapture } from "./measurement.fixture.mjs";
import { parseDeviceObservations } from "./device-observations.mjs";
function facts() {
  const identity = { firmware_commit: "a".repeat(40), app_elf_sha256: "b".repeat(64) };
  const device = parseDeviceObservations(syntheticCapture({ package: identity }).log, { firmwareCommit: identity.firmware_commit, appElfSha256: identity.app_elf_sha256 });
  return [{ status: "measurement_complete", capture: { qualified: true }, restoration: { confirmed: true }, cleanup: { complete: true } }, syntheticTimingV2(), device,
    { deviceRestorationConfirmed: true, deviceLeaseInactive: true, preservation: { device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false } }];
}
test("qualified correction joins actual parsed native marker and host evidence", () => { assert.equal(correctionJudgment(...facts()).accepted, true); });
for (const [name, change] of Object.entries({
  retained_host_failure: a => { a[1].earliestFailure = { stage: "read", category: "monitor_failed" }; },
  capture_unqualified: a => { a[0].capture.qualified = false; },
  late_reader: a => { a[1].events.find(v => v.stage === "reader_opened").elapsedUs = 1500001; },
  incomplete_capture: a => { a[1].captureComplete = false; },
  native_failure: a => { a[2].counters.actualFailures = 1; },
  missing_restoration: a => { a[3].deviceRestorationConfirmed = false; },
  missing_cleanup: a => { a[0].cleanup.complete = false; },
  incomplete_measurement: a => { a[0].status = "unverified"; },
})) test(`correction remains rejected for ${name}`, () => { const values = facts(); change(values); assert.equal(correctionJudgment(...values).accepted, false); });
test("reader headroom includes the exact 1500 ms boundary", () => { const values = facts(); values[1].events.find(v => v.stage === "reader_opened").elapsedUs = 1500000; assert.equal(correctionJudgment(...values).checks.readerHeadroom, true); });

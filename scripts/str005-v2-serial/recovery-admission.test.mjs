import assert from "node:assert/strict";
import test from "node:test";
import { validateRecoveryDetector } from "./recovery-admission.mjs";

const previous = { port: "/dev/cu.test", physical: "a".repeat(64) };
const detector = `espflash_version: 4.5.0\nport: ${previous.port}\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${previous.physical}\nexecution_owner: unknown\nrom_admitted: false\n`;
test("production colon-delimited detector output admits only the same runtime device", () => {
  assert.doesNotThrow(() => validateRecoveryDetector(detector, previous, 500));
  assert.throws(() => validateRecoveryDetector(detector.replaceAll(": ", "="), previous, 500));
});
test("stale, duplicate, wrong-profile and changed-device detector facts fail closed", () => {
  assert.throws(() => validateRecoveryDetector(detector, previous, 60001));
  assert.throws(() => validateRecoveryDetector(detector, previous, -1));
  assert.throws(() => validateRecoveryDetector(`${detector}port: ${previous.port}\n`, previous, 0));
  assert.throws(() => validateRecoveryDetector(detector.replace("serial_jtag_runtime", "rom"), previous, 0));
  assert.throws(() => validateRecoveryDetector(detector, { ...previous, physical: "b".repeat(64) }, 0));
});

import { check } from "./values.mjs";

/** Consume the production detector's colon-delimited facts, never infer authority from a node. */
export function validateRecoveryDetector(text, previous, ageMs) {
  check(typeof text === "string" && Buffer.byteLength(text) <= 1048576 &&
    Number.isFinite(ageMs) && ageMs >= 0 && ageMs <= 60000, "recovery_detector_stale");
  const rows = [...text.matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  for (const [key, expected] of [["port", previous.port], ["physical_identity_sha256", previous.physical], ["usb_profile", "serial_jtag_runtime"]]) {
    const matches = rows.filter(row => row[1] === key);
    check(matches.length === 1 && matches[0][2] === expected, "recovery_detector_identity");
  }
}

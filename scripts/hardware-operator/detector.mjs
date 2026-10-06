// Pure reading of `just detect-ultra205` output (`label: value` lines, tools/flash/src/output.rs).
import { refuse } from "./errors.mjs";

/** The admitted runtime device: one native-USB Serial/JTAG port and its stable physical identity. */
export function parseDetector(text) {
  const fields = new Map();
  for (const [, key, value] of text.matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)) {
    refuse(!fields.has(key), "detector_duplicate_field");
    fields.set(key, value);
  }
  const port = fields.get("port"), physical = fields.get("physical_identity_sha256");
  refuse(typeof port === "string" && /^\/dev\/cu\.[A-Za-z0-9._-]+$/u.test(port), "detector_port");
  refuse(typeof physical === "string" && /^[0-9a-f]{64}$/u.test(physical), "detector_physical_identity");
  refuse(fields.get("usb_profile") === "serial_jtag_runtime", "detector_profile");
  return { port, physical };
}

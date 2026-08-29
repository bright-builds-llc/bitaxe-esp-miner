import { createHash } from "node:crypto";

const REQUIRED_KEYS = [
  "api_route_shell", "app_elf_sha256", "asic", "board", "boot_ordinal",
  "esp_idf_version", "firmware_commit", "hardware_control", "mining",
  "ota_boot_validation", "redacted", "reference_commit", "reset_reason", "schema_version",
  "session", "spiffs_mount", "uptime_ms", "work_submission",
];

export function validateRuntimeAttestationCapture(text, expected) {
  const samples = text.split(/\r?\n/).flatMap((line) => {
    const marker = line.indexOf("runtime_boot_attestation ");
    if (marker === -1) return [];
    const fields = {};
    for (const token of line.slice(marker + "runtime_boot_attestation ".length).trim().split(/\s+/)) {
      const separator = token.indexOf("=");
      if (separator <= 0) throw new Error("runtime_attestation_invalid");
      const key = token.slice(0, separator);
      if (Object.hasOwn(fields, key)) throw new Error("runtime_attestation_invalid");
      fields[key] = token.slice(separator + 1);
    }
    if (Object.keys(fields).sort().join(",") !== REQUIRED_KEYS.join(",")) {
      throw new Error("runtime_attestation_invalid");
    }
    return [fields];
  });
  if (samples.length < 2) throw new Error("runtime_attestation_insufficient");
  for (let index = 0; index < samples.length; index += 1) {
    const sample = samples[index];
    if (
      sample.schema_version !== "1" || sample.board !== "205" || sample.asic !== "BM1366" ||
      sample.mining !== "disabled" || sample.work_submission !== "disabled" ||
      sample.hardware_control !== "disabled" || sample.ota_boot_validation !== "complete" ||
      sample.spiffs_mount !== "available" || sample.api_route_shell !== "started" ||
      sample.redacted !== "true" || sample.firmware_commit !== expected.firmwareCommit ||
      sample.reference_commit !== expected.referenceCommit ||
      sample.app_elf_sha256 !== expected.appElfSha256 ||
      !["power_on", "software_cpu", "watchdog", "panic", "brownout", "other"]
        .includes(sample.reset_reason) ||
      !/^[0-9a-f]{32}$/.test(sample.session) || !/^[1-9][0-9]*$/.test(sample.boot_ordinal) ||
      !/^[0-9]+$/.test(sample.uptime_ms) || !/^[A-Za-z0-9._-]+$/.test(sample.esp_idf_version)
    ) {
      throw new Error("runtime_attestation_invalid");
    }
    if (index > 0) {
      const previous = samples[index - 1];
      if (
        sample.session !== previous.session || sample.boot_ordinal !== previous.boot_ordinal ||
        BigInt(sample.uptime_ms) <= BigInt(previous.uptime_ms)
      ) {
        throw new Error("runtime_attestation_not_stable");
      }
    }
  }
  return {
    sampleCount: samples.length,
    captureSha256: createHash("sha256").update(text).digest("hex"),
  };
}

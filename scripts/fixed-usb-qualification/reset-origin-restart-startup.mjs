import { parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";
import { exactObject, requireCondition as check } from "./contract.mjs";

/** Reduce complete public boot markers; never infer a second boot from its initial reset category. */
export function inspectRestartStartup(bytes, context) {
  check(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= 1048576, "restart_install_capture_bound");
  const lines = bytes.toString("utf8").split("\n");
  lines.pop();
  const boots = [],
    complete = [];
  let identities = 0,
    profiles = 0,
    lastStartup = -1,
    ready = false,
    profileOrdinal;
  for (const raw of lines) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    check(
      !/^(rust_panic_receipt|allocation_failure|allocation_failure_context|wifi_startup_failure|bwg_worker_start_failure|storage_http_failure)/u.test(
        line,
      ) && !/Guru Meditation Error|abort\(\) was called|stack overflow/iu.test(line),
      "restart_install_failure_observed",
    );
    if (line.startsWith("usb_reboot_discriminator")) {
      const m =
        /^usb_reboot_discriminator schema=v1 boot_ordinal=([0-9]+) reset_reason=(power_on|software_cpu|panic|other) uptime_ms=([0-9]+) redacted=true$/u.exec(
          line,
        );
      check(m, "restart_install_boot_marker");
      const boot = { ordinal: Number(m[1]), reason: m[2], uptime: Number(m[3]) };
      check(Number.isSafeInteger(boot.ordinal) && boot.ordinal > 0 && Number.isSafeInteger(boot.uptime), "restart_install_boot_marker");
      const prior = boots.at(-1);
      check(
        !prior || (prior.ordinal === boot.ordinal && prior.reason === boot.reason && boot.uptime >= prior.uptime),
        "restart_install_boot_transition",
      );
      check(profileOrdinal === undefined || profileOrdinal === boot.ordinal, "restart_install_profile_boot");
      boots.push(boot);
    } else if (line.startsWith("usb_startup")) {
      const m =
        /^usb_startup schema=v1 stage=([a-z_]+) state=(entered|complete|failed) first_failure=([a-z_]+) uptime_ms=([0-9]+) redacted=true$/u.exec(
          line,
        );
      check(m && m[2] !== "failed" && m[3] === "none", "restart_install_startup_failure");
      parseResetOriginDiagnostic({
        category: "startup",
        authoritative: false,
        stage: m[1],
        state: m[2],
        first_failure: m[3],
        uptime_ms: Number(m[4]),
      });
      const uptime = Number(m[4]);
      check(Number.isSafeInteger(uptime) && uptime >= lastStartup, "restart_install_startup_regression");
      lastStartup = uptime;
      const currentReady = m[1] === "runtime_ready" && m[2] === "complete";
      check(!ready || currentReady, "restart_install_startup_regression");
      ready = currentReady;
      if (currentReady) complete.push(uptime);
    } else if (line.startsWith("usb_runtime_identity")) {
      const m = /^usb_runtime_identity schema=v1 firmware_commit=([0-9a-f]{40}) app_elf_sha256=([0-9a-f]{64}) redacted=true$/u.exec(line);
      check(m && m[1] === context.firmware_commit && m[2] === context.app_elf_sha256, "restart_install_identity");
      identities++;
    } else if (line.startsWith("usb_boot_profile")) {
      check(line.startsWith("usb_boot_profile="), "restart_install_profile");
      let value;
      try {
        value = JSON.parse(line.slice("usb_boot_profile=".length));
      } catch {
        check(false, "restart_install_profile");
      }
      exactObject(value, ["schema_version", "transport", "reason", "baseline", "firmware_commit", "app_elf_sha256", "boot_ordinal"]);
      check(
        value.schema_version === 1 &&
          value.transport === "serial_jtag_runtime" &&
          value.reason === "worker_started" &&
          value.baseline === "confirmed" &&
          value.firmware_commit === context.firmware_commit &&
          value.app_elf_sha256 === context.app_elf_sha256 &&
          Number.isSafeInteger(value.boot_ordinal) &&
          value.boot_ordinal > 0,
        "restart_install_profile",
      );
      check(profileOrdinal === undefined || profileOrdinal === value.boot_ordinal, "restart_install_profile_boot");
      profileOrdinal = value.boot_ordinal;
      profiles++;
      check(
        boots.every((b) => b.ordinal === value.boot_ordinal),
        "restart_install_profile_boot",
      );
    } else if (line.startsWith("storage_http_status")) {
      check(/^storage_http_status schema=v1 spiffs_available=true http_ready=true redacted=true$/u.test(line), "restart_install_storage");
    }
  }
  const bootSpan = boots.length ? boots.at(-1).uptime - boots[0].uptime : 0;
  const startupSpan = complete.length ? complete.at(-1) - complete[0] : 0;
  check(
    identities > 0 && profiles > 0 && ready && boots.length >= 2 && complete.length >= 2 && bootSpan >= 1000 && startupSpan >= 1000,
    "restart_install_advancing_startup",
  );
  return {
    schema: "worker-restart-install-startup-v1",
    boot_ordinal: boots[0].ordinal,
    initial_reset_category: boots[0].reason,
    observed_transitions: 0,
    boot_records: boots.length,
    complete_records: complete.length,
    boot_span_ms: bootSpan,
    startup_span_ms: startupSpan,
    prior_reset_attribution: "unknown",
    legacy_grade_replaced: false,
  };
}

#!/usr/bin/env node

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const workspaceRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(workspaceRoot, path), "utf8");
const requireText = (path, needles) => {
  const source = read(path);
  for (const needle of needles) {
    if (!source.includes(needle)) {
      throw new Error(`${path}: missing native USB contract ${JSON.stringify(needle)}`);
    }
  }
  return source;
};

const count = (source, pattern) => [...source.matchAll(pattern)].length;

const nativeOwner = requireText("firmware/bitaxe/bwg/native/bwg_usb.c", [
  "tinyusb_driver_install",
  "tinyusb_driver_uninstall",
  "RTC_CNTL_FORCE_DOWNLOAD_BOOT",
  "usb_runtime_line_coding",
  "usb_runtime_line_state",
]);
if (count(nativeOwner, /tinyusb_driver_install\s*\(/g) !== 1) {
  throw new Error("firmware must retain exactly one TinyUSB installation owner");
}

requireText("crates/bitaxe-core/src/usb_maintenance.rs", [
  "UsbMaintenanceState",
  "RequestSafeStop",
  "EmitReady",
  "RestartBootloader",
  "HANDOFF_WINDOW_MS",
]);
requireText("firmware/bitaxe/src/usb_runtime.rs", [
  "bitaxe_core::usb_maintenance",
  "bwg_usb_install",
  "bwg_usb_restart_bootloader",
]);
requireText("firmware/bitaxe/src/bwg_worker_usb.rs", [
  "usb_maintenance=",
  "status",
  "ready",
  "usb_runtime_line_coding",
  "usb_runtime_line_state",
  "restart_into_rom_downloader",
  "worker.has_active_lease()",
  "maintenance_ingress_open",
]);
requireText("firmware/bitaxe/src/startup.rs", [
  "maybe_tcp_payload_admission.is_some()",
  "maybe_noise_diagnostic_admission.is_some()",
  "maybe_self_test_admission.is_some()",
  "usb_runtime=serial_jtag",
]);

requireText("tools/device-session/src/usb_ownership.rs", [
  "WorkerRuntime",
  "SerialJtagRuntime",
  "RomDownloader",
  "handoff_worker_to_rom",
  "plan_usb_operation",
]);
requireText("tools/device-session/src/macos.rs", [
  "physical_identity_digest",
  "enumeration_token",
  "product_name",
]);
requireText("tools/flash/src/environment/usb_ownership.rs", [
  "ensure_bootloader",
  "handoff_worker_to_rom(session)",
  "no-reset-no-sync",
  "ensure_observable_runtime",
  "UsbOperation::Recover",
]);
const flashExecution = read("tools/flash/src/environment.rs");
if (count(flashExecution, /self\.ensure_bootloader\(\)\?/g) < 2) {
  throw new Error("espflash and managed recovery writes must both route through UsbOwnership");
}
requireText("tools/flash/src/commands.rs", [
  "UsbProfile::WorkerRuntime",
  "handoff_required",
  "runtime_profile_unknown",
]);

requireText("AGENTS.md", [
  "docs/hardware/native-usb-ownership.md",
  "ADR-0020",
  "just verify-native-usb-ownership",
]);
requireText("docs/adr/0015-separate-bootloader-runtime-and-control-transports.md", ["ADR-0020"]);
requireText("docs/adr/0018-possession-bound-worker-usb-control.md", ["ADR-0020"]);
requireText("docs/adr/0020-time-multiplex-native-usb-ownership.md", [
  "Worker runtime",
  "ROM downloader",
]);
requireText("docs/hardware/native-usb-ownership.md", [
  "Monitoring never arms handoff",
  "just verify-native-usb-ownership",
]);
requireText("docs/hardware/esp-device-session.md", ["native-usb-ownership.md"]);
requireText(".codex/tasks/lessons.md", ["lesson-visible-cdc-is-not-flash-admission"]);

process.stdout.write("native_usb_ownership=verified\n");

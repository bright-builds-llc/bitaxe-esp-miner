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

const rustOwner = [
  read("firmware/bitaxe/src/usb_runtime.rs"),
  read("firmware/bitaxe/src/usb_runtime/tinyusb.rs"),
  read("firmware/bitaxe/src/usb_runtime/callbacks.rs"),
].join("\n");
for (const required of [
  "tinyusb_driver_install",
  "WORKER_DEVICE_DESCRIPTOR",
  "WORKER_CONFIGURATION_DESCRIPTOR",
  "tud_mount_cb",
  "tud_umount_cb",
  "tud_vendor_rx_cb",
  "tud_cdc_rx_cb",
  "tud_cdc_line_coding_cb",
  "tud_cdc_line_state_cb",
  "tud_vendor_n_write",
  "tud_cdc_n_write",
  "bytes.is_null()",
  "coding.is_null()",
  "read_unaligned()",
]) {
  if (!rustOwner.includes(required)) {
    throw new Error(`Rust USB owner is missing ${JSON.stringify(required)}`);
  }
}
if (count(rustOwner, /tinyusb_driver_install\s*\(/g) !== 1) {
  throw new Error("firmware must retain exactly one TinyUSB installation owner");
}

const phyAdapter = requireText("firmware/bitaxe/bwg/native/usb_phy_handoff.c", [
  "bitaxe_usb_restart_bootloader",
  "tinyusb_driver_uninstall",
  "RTC_CNTL_FORCE_DOWNLOAD_BOOT",
  "USB_SERIAL_JTAG",
]);
for (const forbidden of [
  "tinyusb_driver_install",
  "tud_mount_cb",
  "tud_umount_cb",
  "tud_vendor_rx_cb",
  "tud_cdc_rx_cb",
  "TUD_CDC_DESCRIPTOR",
  "BWG_DEVICE_DESCRIPTOR",
]) {
  if (phyAdapter.includes(forbidden)) {
    throw new Error(`C PHY Adapter retained Rust-owned behavior ${JSON.stringify(forbidden)}`);
  }
}

requireText("crates/bitaxe-core/src/usb_maintenance.rs", [
  "UsbMaintenanceState",
  "RequestSafeStop",
  "EmitReady",
  "RestartBootloader",
  "HANDOFF_WINDOW_MS",
]);
requireText("crates/bitaxe-core/src/usb_worker.rs", [
  "WORKER_DEVICE_DESCRIPTOR",
  "WORKER_CONFIGURATION_DESCRIPTOR",
  "VendorWriteProgress",
  "MAX_VENDOR_WRITE_WAITS",
]);
requireText("firmware/bitaxe/src/usb_runtime.rs", [
  "bitaxe_core::usb_maintenance",
  "install_worker_runtime",
  "send_worker_frame",
  "emit_evidence",
  "bitaxe_usb_restart_bootloader",
]);
requireText("firmware/bitaxe/src/bwg_worker_usb.rs", [
  "usb_maintenance=",
  "status",
  "ready",
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
  "maintenance_control_steps",
  "MaintenanceControlStep::ClearDtr",
  "MaintenanceControlStep::SetBitRate(115_200)",
  "MaintenanceControlStep::AssertDtr",
  "MaintenanceControlStep::SetBitRate(1_200)",
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
  "ESPFLASH_ADMITTED_ROM_BEFORE",
  'ESPFLASH_ADMITTED_ROM_BEFORE: &str = "no-reset"',
  'ESPTOOL_ADMITTED_ROM_BEFORE: &str = "no_reset"',
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

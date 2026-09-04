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
  "tud_mounted",
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
const tinyUsbAdapter = read("firmware/bitaxe/src/usb_runtime/tinyusb.rs");
const evidenceStart = tinyUsbAdapter.indexOf("fn try_emit_evidence(");
const observerStart = tinyUsbAdapter.indexOf("pub(super) fn worker_observer_state(");
if (evidenceStart < 0 || observerStart <= evidenceStart) {
  throw new Error("Worker CDC emission and observer sampling must remain separate");
}
const evidenceWriter = tinyUsbAdapter.slice(evidenceStart, observerStart);
if (evidenceWriter.includes("tud_cdc_n_connected")) {
  throw new Error("Worker CDC evidence must not require host DTR");
}
if (!evidenceWriter.includes("tud_mounted()") || rustOwner.includes("tud_cdc_n_write_clear")) {
  throw new Error("Worker CDC emission must preserve mounted evidence and queued receipts");
}

const phyAdapter = requireText("firmware/bitaxe/bwg/native/usb_phy_handoff.c", [
  "bitaxe_usb_restart_bootloader",
  "tinyusb_driver_uninstall",
  "RTC_CNTL_FORCE_DOWNLOAD_BOOT",
  "USB_SERIAL_JTAG",
  "gpio_set_level(USBPHY_DM_NUM, 0)",
  "gpio_set_level(USBPHY_DP_NUM, 0)",
  "USB_SERIAL_JTAG_INTR_BUS_RESET",
  "BUS_RESET_TIMEOUT_US",
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
  "CommitRestart",
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
  "committed",
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
requireText("firmware/bitaxe/sdkconfig.defaults", [
  "CONFIG_TINYUSB_TASK_STACK_SIZE=3072",
  "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304",
]);
if (read("firmware/bitaxe/sdkconfig.defaults").includes("TINYUSB_VENDOR_RX_BUFSIZE")) {
  throw new Error("firmware sdkconfig retained an unknown TinyUSB vendor RX symbol");
}

requireText("tools/device-session/src/usb_ownership.rs", [
  "WorkerRuntime",
  "SerialJtagRuntime",
  "RomDownloader",
  "plan_usb_operation",
]);
requireText("tools/device-session/src/usb_ownership/execution.rs", [
  "UsbExecutionOwner",
  "admit_rom_execution",
  "admit_application_execution",
]);
requireText("tools/device-session/src/usb_ownership/maintenance.rs", [
  "handoff_worker_to_rom",
  "maintenance_control_steps",
  "maintenance_commit_steps",
  "MaintenanceCommitStep::AwaitCommitted",
  "HandoffCommitTimeout",
  "MaintenanceControlStep::ClearDtr",
  "MaintenanceControlStep::SetBitRate(115_200)",
  "MaintenanceControlStep::AssertDtr",
  "MaintenanceControlStep::SetBitRate(1_200)",
  "maintenance_step={}",
]);
requireText("tools/device-session/src/usb_ownership/profile_trace.rs", [
  "ProfileObservationTrace",
  "ProfileObservationCategory::SameWorker",
]);
requireText("tools/device-session/src/macos.rs", [
  "physical_identity_digest",
  "enumeration_token",
  "product_name",
  "ReceiveOnlyReader::open(port).is_ok()",
]);
requireText("tools/device-session/src/reboot_loop.rs", [
  "observe_usb_reboot_loop",
  "WorkerUsbBootMarker::parse",
  "UsbRebootLoopCategory::ChipReset",
  "UsbRebootLoopCategory::UsbStackReset",
]);
const receiveOnlyAdapter = read("tools/device-session/src/macos/receive_only.rs")
  .split("#[cfg(test)]")[0];
for (const required of [
  ".read(true)",
  "cfmakeraw",
  "B115200",
  "CLOCAL",
  "CREAD",
  "!libc::HUPCL",
]) {
  if (!receiveOnlyAdapter.includes(required)) {
    throw new Error(`macOS receive-only Adapter is missing ${JSON.stringify(required)}`);
  }
}
for (const forbidden of [".write(", "write_all", "TIOCM", "ioctl(", "B1200"]) {
  if (receiveOnlyAdapter.includes(forbidden)) {
    throw new Error(
      `macOS receive-only Adapter contains forbidden operation ${JSON.stringify(forbidden)}`,
    );
  }
}
const workerEvidenceAdapter = read("tools/device-session/src/macos/worker_evidence.rs")
  .split("#[cfg(test)]")[0];
for (const required of ["configure_serial", ".write(true)", "TIOCMBIS", "TIOCMBIC"]) {
  if (!workerEvidenceAdapter.includes(required)) {
    throw new Error(`Worker evidence Adapter is missing ${JSON.stringify(required)}`);
  }
}
for (const forbidden of ["B1200", "write_all", "std::io::Write", "SetBitRate(1_200)"]) {
  if (workerEvidenceAdapter.includes(forbidden)) {
    throw new Error(
      `Worker evidence Adapter contains forbidden operation ${JSON.stringify(forbidden)}`,
    );
  }
}
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
const transitionVerifier = requireText("tools/flash/src/native_usb_transition.rs", [
  "run_verify_native_usb_transition",
  "UsbOperation::VerifyTransition",
  "transition-result.private.json",
  "device_write_observed",
  "restoration_complete",
]);
for (const forbidden of [
  "write-bin",
  "write_flash",
  "erase_flash",
  "generate_nvs_partition",
  "wifi_credentials",
  "pool_credentials",
]) {
  if (transitionVerifier.includes(forbidden)) {
    throw new Error(`no-write transition verifier contains ${JSON.stringify(forbidden)}`);
  }
}
requireText("tools/device-session/src/usb_ownership/verification.rs", [
  "verify_native_usb_transition",
  "run_installed_application",
  "board-info",
  '"no-reset"',
  '"hard-reset"',
  '"hard_reset"',
  "reacquire_profile(UsbProfile::WorkerRuntime)",
]);
requireText("crates/bitaxe-api/src/usb_boot_profile.rs", [
  "UsbBootProfileMarker",
  "UsbBootProfileReplay",
  "usb_boot_profile=",
]);
requireText("firmware/bitaxe/src/boot_evidence.rs", [
  "publish_usb_boot_profile",
  "usb_profile::emit_due",
  "worker_rust_panic_marker",
  "worker_allocation_failure_marker",
]);
requireText("firmware/bitaxe/src/panic_evidence.rs", [
  "RTC_PANIC_RECEIPT",
  "RTC_ALLOCATION_FAILURE_RECEIPT",
  "std::panic::set_hook",
  "heap_caps_register_failed_alloc_callback",
  "write_volatile",
]);
requireText("firmware/bitaxe/src/boot_evidence/usb_profile.rs", [
  "UsbBootProfileReplay::new",
  "maybe_take_due",
]);
requireText("tools/automation/src/native-usb-transition-recovery.ts", [
  "preflightNativeUsbRecovery",
  "startNativeUsbRecovery",
  "finalizeNativeUsbRecovery",
  "native_usb_recovery",
  "restore_admission",
  "restoration_complete",
  "completedRestoreCommandReceipt",
  "repeated_write_allowed: false",
]);
requireText("tools/automation/src/native-usb-display-recovery.ts", [
  "native-usb-display-recovery",
  "completedRestoreCommandReceipt",
  "display-origin-capture",
]);
requireText("tools/automation/src/native-usb-config-ap-recovery.ts", [
  "native-usb-config-ap-recovery",
  "nvs_checkpoint_required",
  "completedRestoreCommandReceipt",
  "host_network_effect: false",
]);
requireText("tools/automation/src/native-usb-rom-exit.ts", [
  "native-usb-rom-exit",
  "managed_esptool_hard_reset",
  "execution_owner",
]);
requireText("tools/automation/src/native-usb-owner-recovery.ts", [
  "native-usb-owner-recovery",
  "passive_marker_status",
  "manual_boot_reset",
  "managed_esptool_hard_reset",
  "physical_identity_match",
]);
requireText("tools/flash/src/owner_recovery.rs", [
  "classify_runtime_boot_attestations",
  "RuntimeAttestationStatus::InsufficientSamples",
  "board-info",
  '"no-reset"',
  "device_write_observed: false",
  "host_network_effect: false",
]);
requireText("tools/flash/src/environment/owner_recovery.rs", [
  "force_download_read_args",
  "run_installed_application",
  "observe_receive_only",
]);
requireText("tools/flash/src/flash_transfer.rs", [
  '"--no-stub"',
  '"--flash_size"',
  '"16MB"',
  '"read_flash"',
]);
for (const candidate of [
  "tools/flash/src/boot_chain.rs",
  "tools/flash/src/nvs_readback.rs",
  "tools/flash/src/usb_stability.rs",
]) {
  if (read(candidate).includes('"read_flash"')) {
    throw new Error(`${candidate}: raw read_flash bypasses UsbFlashTransfer`);
  }
}
requireText("tools/flash/src/usb_stability.rs", [
  "rom_no_stub",
  "digest_match_count",
  "device_write_observed: false",
  "host_network_effect: false",
]);
const nvsReadback = requireText("tools/flash/src/nvs_readback.rs", [
  "read_flash",
  "0x9000",
  "0x6000",
  "compare_expected_nvs",
  "device_write_observed",
  "run_nvs_runtime_restore",
  "nvs_read_repeated",
]);
for (const forbidden of ["write_flash", "erase_flash", "erase_region"]) {
  if (nvsReadback.includes(forbidden)) {
    throw new Error(`NVS discriminator contains forbidden effect ${JSON.stringify(forbidden)}`);
  }
}
const displayRecovery = requireText("tools/flash/src/display_recovery.rs", [
  "run_display_recovery_start",
  "display_mac_sha256",
  "usb_mac_bound",
  "patch_system_settings_once",
]);
for (const forbidden of ["write-bin", "write_flash", "erase-flash", "generate_nvs_partition"]) {
  if (displayRecovery.includes(forbidden)) {
    throw new Error(`display recovery contains forbidden effect ${JSON.stringify(forbidden)}`);
  }
}
requireText("Justfile", [
  "verify-native-usb-transition",
  "native-usb-transition-recovery",
  "native-usb-display-recovery",
  "native-usb-config-ap-recovery",
  "native-usb-rom-exit",
  "native-usb-owner-recovery",
  "usb-stability-read",
  "diagnose-usb-reboot-loop",
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
  "just native-usb-display-recovery",
  "just native-usb-config-ap-recovery",
  "just usb-stability-read",
  "usb-stability-baseline-v1.json",
  "execution owner",
]);
requireText("docs/parity/evidence/native-usb-stability/usb-stability-baseline-v1.json", [
  '"adapter": "rom_no_stub"',
  '"repeated_passed": 20',
  '"sequential_passed": 16',
  '"full_factory_digest_match": true',
  '"device_write_observed": false',
  '"redaction_status": "passed"',
]);
requireText("docs/hardware/esp-device-session.md", ["native-usb-ownership.md"]);
requireText(".codex/tasks/lessons.md", ["lesson-visible-cdc-is-not-flash-admission"]);

process.stdout.write("native_usb_ownership=verified\n");

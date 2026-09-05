#!/usr/bin/env node
import { readFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (file) => readFileSync(resolve(root, file), "utf8");
function requireText(file, values) {
  const text = read(file);
  for (const value of values) if (!text.includes(value)) throw new Error(`${file}: missing ${value}`);
  return text;
}
const driver = requireText("firmware/bitaxe/src/usb_runtime.rs", [
  "usb_serial_jtag_driver_install", "usb_serial_jtag_read_bytes", "usb_serial_jtag_write_bytes",
]);
for (const forbidden of ["tinyusb_driver_install", "bitaxe_usb_restart_bootloader", "tud_", "TIOCM"]) {
  if (driver.includes(forbidden)) throw new Error(`fixed driver retained ${forbidden}`);
}
const cargo = read("firmware/bitaxe/Cargo.toml");
if (cargo.includes("esp_tinyusb") || cargo.includes("usb_phy_handoff")) throw new Error("firmware retained TinyUSB dependency");
const config = read("firmware/bitaxe/sdkconfig.defaults");
if (/^CONFIG_TINYUSB[^\n]*=y$/mu.test(config)) throw new Error("TinyUSB is enabled");
requireText("firmware/bitaxe/src/bwg_worker_usb/link.rs", ["check_deadline", "SerialEnvelope", "heartbeat"]);
requireText("firmware/bitaxe/src/bwg_worker_usb/writer.rs", ["usb_runtime::write"]);
requireText("AGENTS.md", ["ADR-0021", "docs/hardware/native-usb-ownership.md"]);
requireText("docs/adr/0021-fixed-serial-jtag-worker-transport.md", ["Web Serial", "240", "three seconds"]);
requireText("docs/hardware/native-usb-ownership.md", ["NVS", "logical", "cleanup"]);
for (const file of ["firmware/bitaxe/bwg/native/usb_phy_handoff.c", "firmware/bitaxe/src/usb_runtime/tinyusb.rs"]) {
  if (existsSync(resolve(root,file))) throw new Error(`obsolete active PHY source remains: ${file}`);
}
process.stdout.write("native_usb_ownership=verified fixed_serial_jtag=true\n");

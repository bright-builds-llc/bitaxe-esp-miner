#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { accessSync, constants, readdirSync } from "node:fs";
import path from "node:path";

const elf = process.argv[2];
if (elf === undefined || elf.length === 0) {
  throw new Error("native USB symbol verification requires one firmware ELF");
}

const user = process.env["USER"];
if (user === undefined || user.length === 0) {
  throw new Error("native USB symbol verification requires the local user identity");
}
const toolRoot = path.join("/Users", user, ".rustup/toolchains/esp/xtensa-esp-elf");
const candidates = readdirSync(toolRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => path.join(
    toolRoot,
    entry.name,
    "xtensa-esp-elf/bin/xtensa-esp32s3-elf-nm",
  ))
  .filter((candidate) => {
    try {
      accessSync(candidate, constants.X_OK);
      return true;
    } catch {
      return false;
    }
  });
if (candidates.length !== 1) {
  throw new Error("native USB symbol verification requires one managed Xtensa nm");
}
const outcome = spawnSync(candidates[0], ["-g", elf], { encoding: "utf8" });
if (outcome.status !== 0) {
  throw new Error("native USB symbol inspection failed");
}

const symbols = outcome.stdout
  .split(/\r?\n/u)
  .map((line) => line.trim().split(/\s+/u).at(-1))
  .filter((symbol) => symbol !== undefined && symbol.length > 0);
const required = [
  "bitaxe_usb_restart_bootloader",
  "tud_mount_cb",
  "tud_umount_cb",
  "tud_vendor_rx_cb",
  "tud_cdc_rx_cb",
  "tud_cdc_line_coding_cb",
  "tud_cdc_line_state_cb",
];
for (const symbol of required) {
  const matches = symbols.filter((candidate) => candidate === symbol).length;
  if (matches !== 1) {
    throw new Error(`native USB symbol ${symbol} resolved ${String(matches)} times`);
  }
}
for (const removed of [
  "bwg_usb_install",
  "bwg_usb_vendor_write",
  "bwg_usb_evidence_write",
  "bwg_usb_restart_bootloader",
  "bwg_worker_usb_attached",
  "bwg_worker_usb_detached",
  "bwg_worker_usb_vendor_received",
  "usb_runtime_line_coding",
  "usb_runtime_line_state",
]) {
  if (symbols.includes(removed)) {
    throw new Error(`removed native USB symbol remains linked: ${removed}`);
  }
}

process.stdout.write("native_usb_symbols=verified\n");

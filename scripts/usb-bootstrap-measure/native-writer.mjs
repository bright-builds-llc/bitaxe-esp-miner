import { dirname, resolve } from "node:path";
import { readFile } from "node:fs/promises";
import { inspectUsbWriterStack, USB_STACK_AUDIT_SOURCES, selectedUsbWriter } from "./native-stack.mjs";
import { check, object, uint } from "./values.mjs";
export async function inspectWriter(context, operations = {}) {
  const manifest = JSON.parse(await readFile(context.package.manifest));
  const entry = manifest.artifacts.find(item => item.kind === "firmware_elf");
  return (operations.inspectUsbWriterStack ?? inspectUsbWriterStack)({ firmwareRoot: context.firmwareRoot,
    elfPath: resolve(dirname(context.package.manifest), entry.path), expectedElfSha256: context.package.app_elf_sha256,
    expectedObjdumpSha256: context.nativeReadiness.objdumpSha256 });
}
export function validateWriter(value, context) {
  object(value, ["schema", "stackBytes", "requiredMarginBytes", "selectedFrameSumBytes", "remainingBytes", "frames", "roles", "completeCallgraphBound", "hardwareFitVerified",
    "elfSha256", "objdumpSha256", "supplementalDecodeRanges", "unresolvedJumps", "decodeComplete", "sources"]);
  uint(value.selectedFrameSumBytes); uint(value.remainingBytes);
  check(Array.isArray(value.frames) && value.frames.length > 0 && value.frames.length <= 128 && new Set(value.frames.map(row => row.symbol)).size === value.frames.length, "bootstrap_writer_audit");
  for (const frame of value.frames) { object(frame, ["symbol", "entryBytes"]); check(typeof frame.symbol === "string" && selectedUsbWriter(frame.symbol), "bootstrap_writer_audit"); uint(frame.entryBytes); }
  check(value.frames.reduce((sum, frame) => sum + frame.entryBytes, 0) === value.selectedFrameSumBytes && value.frames.filter(row => row.symbol === "bitaxe_firmware::bwg_worker_usb::writer::run").length === 1, "bootstrap_writer_audit");
  object(value.roles, ["run", "emit", "write", "marker", "record"]);
  const symbols = { run: "bitaxe_firmware::bwg_worker_usb::writer::run", emit: "bitaxe_firmware::bwg_worker_usb::writer::emit",
    write: "bitaxe_firmware::usb_runtime::write_measured_if", marker: "bitaxe_firmware::usb_runtime::measurement::Retained::marker", record: "bitaxe_firmware::usb_runtime::measurement::Retained::record" };
  for (const [role, symbol] of Object.entries(symbols)) check(value.roles[role] === (value.frames.some(frame => frame.symbol === symbol) ? "standalone_measured" : "absent_or_inlined"), "bootstrap_writer_audit");
  check(Object.values(value.roles).every(role => ["standalone_measured", "absent_or_inlined"].includes(role)) && value.roles.run === "standalone_measured", "bootstrap_writer_audit");
  check(Array.isArray(value.sources) && value.sources.length === USB_STACK_AUDIT_SOURCES.length && new Set(value.sources.map(row => row.path)).size === value.sources.length &&
    USB_STACK_AUDIT_SOURCES.every(path => value.sources.some(row => row.path === path)), "bootstrap_writer_audit");
  for (const row of value.sources) { object(row, ["path", "sha256", "length"]); uint(row.length); }
  check(value.schema === "usb-bootstrap-writer-stack-v1" && value.stackBytes === 8192 && value.requiredMarginBytes === 512 &&
    value.selectedFrameSumBytes + value.requiredMarginBytes <= value.stackBytes && value.remainingBytes === value.stackBytes - value.selectedFrameSumBytes &&
    value.completeCallgraphBound === false && value.hardwareFitVerified === false && value.elfSha256 === context.package.app_elf_sha256 &&
    value.objdumpSha256 === context.nativeReadiness.objdumpSha256 && value.decodeComplete === true && value.unresolvedJumps.length === 0 &&
    value.sources.every(row => context.sourceInventory.some(file => file.path === row.path && file.sha256 === row.sha256 && file.length === row.length)), "bootstrap_writer_audit");
  return value;
}

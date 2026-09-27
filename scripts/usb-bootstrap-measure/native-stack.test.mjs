import test from "node:test";
import assert from "node:assert/strict";
import { auditUsbWriterFrames } from "./native-stack.mjs";
import { parseNativeFunctions } from "../telemetry-stack-audit.mjs";
const source = '.name("bwg-serial-writer".into())\n.stack_size(8192)';
const body = (address, symbol, bytes = 128, extra = "") => `${address} <${symbol}>:\n ${address}: 004136 entry a1, ${bytes}\n${extra}\n ${parseInt(address, 16) + 3}: f01d retw.n\n`;
const run = "bitaxe_firmware::bwg_worker_usb::writer::run";
test("measures native frames and reports missing standalone bodies without claiming inlining proof", () => {
  // Arrange
  const functions = parseNativeFunctions(body("1000", run) + body("2000", "bitaxe_firmware::usb_runtime::write_measured_if", 256));
  // Act
  const result = auditUsbWriterFrames(functions, source);
  // Assert
  assert.equal(result.selectedFrameSumBytes, 384); assert.equal(result.roles.write, "standalone_measured");
  assert.equal(result.roles.marker, "absent_or_inlined"); assert.equal(result.completeCallgraphBound, false);
});
test("rejects stack growth and missing writer root", () => {
  assert.throws(() => auditUsbWriterFrames(parseNativeFunctions(body("1000", run, 8000)), source), /bootstrap_writer_stack_budget/u);
  assert.throws(() => auditUsbWriterFrames(new Map(), source), /bootstrap_writer_stack_root/u);
  assert.throws(() => auditUsbWriterFrames(parseNativeFunctions(body("1000", run)), source.replace("8192", "16384")), /bootstrap_writer_stack_declaration/u);
});
test("rejects dynamic frame adjustment", () => {
  const functions = parseNativeFunctions(body("1000", run, 128, " 1003: 112233 addi a1, a1, -16"));
  assert.throws(() => auditUsbWriterFrames(functions, source));
});
test("rejects selected recursive calls rather than counting one activation", () => {
  const functions = parseNativeFunctions(body("1000", run, 128, " 1003: 112233 call8 1000 <bitaxe_firmware::bwg_worker_usb::writer::run>"));
  assert.throws(() => auditUsbWriterFrames(functions, source), /bootstrap_writer_stack_recursion/u);
});

import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import { createHash } from "node:crypto";
import { inspectUsbWriterStack, USB_STACK_AUDIT_SOURCES } from "./native-stack.mjs";
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
async function inspectionFixture(t) {
  const root = await mkdtemp(resolve(tmpdir(), "usb-writer-native-test-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const files = [...USB_STACK_AUDIT_SOURCES, ".embuild/espressif/tools/xtensa-esp-elf/esp-14.2.0_20260121/xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump", "test.elf"];
  for (const path of files) { await mkdir(dirname(resolve(root, path)), { recursive: true });
    await writeFile(resolve(root, path), path.endsWith("bwg_worker_usb.rs") ? source : "fixture"); }
  return { firmwareRoot: root, elfPath: resolve(root, "test.elf"), expectedElfSha256: hash("fixture"), expectedObjdumpSha256: hash("fixture") };
}
const inspectedText = (terminal = "retw.n") => `1000 <${run}>:\n 1000: 004136 entry a1, 128\n 1003: f01d ${terminal}\n2000 <fixture_end>:\n 2000: f01d retw.n\n`;
test("inspector composes the real decoder result and retains pre-audit source identity", async t => {
  // Arrange
  const options = await inspectionFixture(t);
  // Act
  const result = await inspectUsbWriterStack(options, { run: async () => ({ stdout: inspectedText() }) });
  // Assert
  assert.equal(result.selectedFrameSumBytes, 128); assert.equal(result.decodeComplete, true);
  assert.deepEqual(result.unresolvedJumps, []); assert.equal(result.sources.length, USB_STACK_AUDIT_SOURCES.length);
  assert.equal(result.sources.find(item => item.path.endsWith("bwg_worker_usb.rs")).sha256, hash(source));
});
test("inspector rejects source changes during disassembly", async t => {
  // Arrange
  const options = await inspectionFixture(t);
  // Act / Assert
  await assert.rejects(inspectUsbWriterStack(options, { run: async () => {
    await writeFile(resolve(options.firmwareRoot, "firmware/bitaxe/src/usb_runtime.rs"), "changed");
    return { stdout: inspectedText() };
  } }), /bootstrap_writer_stack_source_changed/u);
});
test("inspector exposes unresolved native jumps as a decoding limitation", async t => {
  // Arrange
  const options = await inspectionFixture(t);
  // Act
  const result = await inspectUsbWriterStack(options, { run: async () => ({ stdout: inspectedText("jx a2") }) });
  // Assert
  assert.equal(result.decodeComplete, false); assert.equal(result.unresolvedJumps.length, 1);
  assert.equal(result.completeCallgraphBound, false);
});
test("distinct monomorphized addresses preserve duplicate demangled names", () => {
  // Arrange
  const generic = "bitaxe_firmware::usb_runtime::write_measured_if";
  const functions = parseNativeFunctions(body("1000", run) + body("2000", generic, 256) + body("3000", generic, 384));
  // Act
  const value = auditUsbWriterFrames(functions, source);
  // Assert
  assert.equal(value.selectedFrameSumBytes, 768); assert.equal(value.frames.length, 3);
  assert.equal(new Set(value.frames.map(frame => frame.address)).size, 3);
  assert.equal(value.frames.filter(frame => frame.symbol === generic).length, 2);
});

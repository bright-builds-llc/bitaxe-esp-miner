import { execFile } from "node:child_process";
import { readFile, realpath } from "node:fs/promises";
import { resolve } from "node:path";
import { promisify } from "node:util";
import { nativeFrame, parseNativeFunctions } from "../telemetry-stack-audit.mjs";
import { noiseNativeCalls, noiseNativeInstructions } from "../noise-native-calls.mjs";
import { resolveNoiseInstructions } from "../noise-native-disassembly.mjs";
import { check, sha256 } from "./values.mjs";

const run = promisify(execFile);
const WRITER = "bitaxe_firmware::bwg_worker_usb::writer::";
const RUNTIME = "bitaxe_firmware::usb_runtime::";
const ROLES = { run: `${WRITER}run`, emit: `${WRITER}emit`, write: `${RUNTIME}write_measured_if`,
  marker: `${RUNTIME}measurement::Retained::marker`, record: `${RUNTIME}measurement::Retained::record` };
export const USB_STACK_AUDIT_SOURCES = Object.freeze(["scripts/usb-bootstrap-measure/native-stack.mjs",
  "scripts/telemetry-stack-audit.mjs", "scripts/noise-native-calls.mjs", "scripts/noise-native-disassembly.mjs",
  "firmware/bitaxe/src/bwg_worker_usb.rs", "firmware/bitaxe/src/bwg_worker_usb/writer.rs",
  "firmware/bitaxe/src/usb_runtime.rs", "firmware/bitaxe/src/usb_tx_measurement.rs"]);
export const selectedUsbWriter = symbol => symbol.startsWith(WRITER) || symbol.startsWith(RUNTIME);

/** Sum all selected standalone frames, conservatively including mutually exclusive branches.
 * Missing standalone symbols are reported as absent-or-inlined, never fabricated as zero-cost bodies.
 * Allocators, formatting, ESP-IDF, indirect calls and platform ancestors are not a whole-callgraph proof.
 */
export function auditUsbWriterFrames(functions, ownerSource) {
  check(/\.name\("bwg-serial-writer"\.into\(\)\)\s*\.stack_size\(8192\)/u.test(ownerSource), "bootstrap_writer_stack_declaration");
  const selected = [...functions.values()].filter(fn => selectedUsbWriter(fn.symbol));
  check(selected.length > 0 && selected.length <= 128 && selected.filter(fn => fn.symbol === ROLES.run).length === 1, "bootstrap_writer_stack_root");
  const active = new Set(), visited = new Set();
  const visit = fn => {
    check(!active.has(fn.address), "bootstrap_writer_stack_recursion"); if (visited.has(fn.address)) return;
    active.add(fn.address);
    for (const edge of noiseNativeCalls(fn, { compilerPrivateSpills: true })) {
      const target = functions.get(edge.target); if (target && selectedUsbWriter(target.symbol)) visit(target);
    }
    active.delete(fn.address); visited.add(fn.address);
  };
  for (const fn of selected) visit(fn);
  const frames = selected.map(fn => ({ symbol: fn.symbol,
    entryBytes: nativeFrame({ ...fn, instructions: noiseNativeInstructions(fn) }) }));
  const selectedFrameSumBytes = frames.reduce((sum, frame) => sum + frame.entryBytes, 0);
  check(selectedFrameSumBytes + 512 <= 8192, "bootstrap_writer_stack_budget");
  const roles = Object.fromEntries(Object.entries(ROLES).map(([role, symbol]) => [role,
    frames.some(frame => frame.symbol === symbol) ? "standalone_measured" : "absent_or_inlined"]));
  return { schema: "usb-bootstrap-writer-stack-v1", stackBytes: 8192, requiredMarginBytes: 512,
    selectedFrameSumBytes, remainingBytes: 8192 - selectedFrameSumBytes, frames, roles,
    completeCallgraphBound: false, hardwareFitVerified: false };
}

/** Inspect the actual bound ELF with the existing pinned native disassembler; no execution or hardware. */
export async function inspectUsbWriterStack({ firmwareRoot, elfPath, expectedElfSha256, expectedObjdumpSha256 }, operations = {}) {
  const execute = operations.run ?? run;
  const root = await realpath(firmwareRoot), elf = await realpath(elfPath);
  const capturedSources = new Map(await Promise.all(USB_STACK_AUDIT_SOURCES.map(async path =>
    [path, await readFile(resolve(root, path))])));
  const tool = resolve(root, ".embuild/espressif/tools/xtensa-esp-elf/esp-14.2.0_20260121/xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump");
  check(/^[a-f0-9]{64}$/u.test(expectedElfSha256) && /^[a-f0-9]{64}$/u.test(expectedObjdumpSha256), "bootstrap_writer_stack_identity");
  check(sha256(await readFile(elf)) === expectedElfSha256 && sha256(await readFile(tool)) === expectedObjdumpSha256, "bootstrap_writer_stack_identity");
  const env = { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C" };
  const { stdout: disassembly } = await execute(tool, ["-Cd", elf], { timeout: 30000, maxBuffer: 128 * 1024 * 1024, env });
  const functions = parseNativeFunctions(disassembly);
  const deadline = performance.now() + 30000;
  const decoded = await resolveNoiseInstructions(functions, disassembly, async (start, end) => {
    check(performance.now() < deadline, "bootstrap_writer_stack_decode_timeout");
    const { stdout } = await execute(tool, ["-Cd", `--start-address=0x${start.toString(16)}`, `--stop-address=0x${end.toString(16)}`, elf],
      { timeout: 5000, maxBuffer: 1024 * 1024, env }); return stdout;
  }, { rootSymbols: [ROLES.run], selectSymbol: selectedUsbWriter, compilerPrivateSpills: true });
  const owner = capturedSources.get("firmware/bitaxe/src/bwg_worker_usb.rs").toString("utf8");
  const result = auditUsbWriterFrames(decoded.functions, owner);
  for (const [path, bytes] of capturedSources) check(bytes.equals(await readFile(resolve(root, path))), "bootstrap_writer_stack_source_changed");
  check(sha256(await readFile(elf)) === expectedElfSha256 && sha256(await readFile(tool)) === expectedObjdumpSha256, "bootstrap_writer_stack_identity");
  return { ...result, elfSha256: expectedElfSha256, objdumpSha256: expectedObjdumpSha256,
    supplementalDecodeRanges: decoded.supplementalRanges, unresolvedJumps: decoded.unresolvedJumps,
    decodeComplete: decoded.unresolvedJumps.length === 0,
    sources: [...capturedSources].map(([path, bytes]) => ({ path, sha256: sha256(bytes), length: bytes.length })) };
}

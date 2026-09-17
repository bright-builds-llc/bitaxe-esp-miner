import { execFile } from "node:child_process";
import { readFile, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { promisify } from "node:util";
import { inspectNoiseNativeReadiness, NATIVE_AUDITOR_SOURCES } from "./noise-native-readiness.mjs";
import { parseNativeFunctions } from "./telemetry-stack-audit.mjs";
import { resolveNoiseInstructions } from "./noise-native-disassembly.mjs";
import { auditV2Paths, selectedV2Symbol, V2_ROOTS } from "./v2-native-paths.mjs";
import { check, sha256 } from "./str005-v2-serial/values.mjs";

const run = promisify(execFile);
export const V2_NATIVE_AUDITOR_SOURCES = Object.freeze([
  "scripts/v2-native-readiness.mjs", "scripts/v2-native-paths.mjs", "scripts/str005-v2-serial/values.mjs", ...NATIVE_AUDITOR_SOURCES,
]);
export const V2_NATIVE_SOURCE_FILES = Object.freeze([
  "firmware/bitaxe/src/noise_serial_runtime.rs",
  "firmware/bitaxe/src/noise_serial_runtime/channel.rs",
  "firmware/bitaxe/src/production_mining_session.rs",
  "firmware/bitaxe/src/production_mining_session/transport.rs",
  "firmware/bitaxe/src/production_mining_session/transport/borrow.rs",
  "firmware/bitaxe/src/production_mining_session/transport/v2.rs",
  "firmware/bitaxe/src/production_mining_session/revocation.rs",
  "firmware/bitaxe/src/production_mining_session/revocation/global.rs",
  "firmware/bitaxe/src/production_mining_session/revocation/budget_observation.rs",
  "firmware/bitaxe/src/production_mining_session/asic_worker.rs",
  "firmware/bitaxe/src/v2_serial_runtime.rs",
  "firmware/bitaxe/src/v2_serial_runtime/observer.rs",
  "firmware/bitaxe/src/v2_serial_runtime/facts.rs",
]);

/** Actual current clean package only. This is selected static readiness, never a hardware verdict. */
export async function inspectV2NativeReadiness(options) {
  const root = await realpath(options.firmwareRoot), captured = new Map();
  for (const path of [...V2_NATIVE_SOURCE_FILES, ...V2_NATIVE_AUDITOR_SOURCES])
    captured.set(path, await readFile(resolve(root, path)));
  const common = await inspectNoiseNativeReadiness({ ...options, firmwareRoot: root }, { compilerPrivateSpills: true });
  const manifestPath = await realpath(options.manifestPath), manifestBytes = await readFile(manifestPath);
  const manifest = JSON.parse(manifestBytes), elfPath = resolve(dirname(manifestPath), "bitaxe-ultra205.elf");
  check(sha256(manifestBytes) === common.packageManifestSha256 && manifest.app_elf_sha256 === options.expectedElfSha256,
    "v2_native_manifest_changed");
  const elf = await readFile(elfPath); check(sha256(elf) === options.expectedElfSha256, "v2_native_elf_changed");
  const tool = resolve(root, ".embuild/espressif/tools/xtensa-esp-elf/esp-14.2.0_20260121/xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump");
  const toolBytes = await readFile(tool); check(sha256(toolBytes) === common.objdumpSha256, "v2_native_tool_changed");
  const environment = { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C" };
  const { stdout: disassembly } = await run(tool, ["-Cd", elfPath], { timeout: 30000, maxBuffer: 128 * 1024 * 1024, env: environment });
  const deadline = performance.now() + 30000;
  const decoded = await resolveNoiseInstructions(parseNativeFunctions(disassembly), disassembly, async (start, end) => {
    check(performance.now() < deadline, "v2_native_decode_timeout");
    const value = await run(tool, ["-Cd", `--start-address=0x${start.toString(16)}`, `--stop-address=0x${end.toString(16)}`, elfPath], {
      timeout: Math.max(1, Math.floor(deadline - performance.now())), maxBuffer: 1024 * 1024, env: environment,
    }); return value.stdout;
  }, { rootSymbols: V2_ROOTS, selectSymbol: selectedV2Symbol, compilerPrivateSpills: true });
  const text = (path) => captured.get(`firmware/bitaxe/src/${path}`).toString("utf8");
  const paths = auditV2Paths(decoded.functions, { transportSource: text("production_mining_session/transport.rs"),
    borrowSource: text("production_mining_session/transport/borrow.rs"), channelSource: text("noise_serial_runtime/channel.rs"),
    runtimeSource: text("v2_serial_runtime.rs"), noiseOwnerSource: text("noise_serial_runtime.rs") });
  check(Object.keys(paths).sort().join(",") === "channel,share", "v2_native_scope_missing");
  for (const [path, bytes] of captured) check(sha256(await readFile(resolve(root, path))) === sha256(bytes), "v2_native_source_changed");
  check(sha256(await readFile(elfPath)) === sha256(elf) && sha256(await readFile(manifestPath)) === sha256(manifestBytes) &&
    sha256(await readFile(tool)) === sha256(toolBytes), "v2_native_input_changed");
  const hashes = (paths) => paths.map((path) => ({ path, sha256: sha256(captured.get(path)) }));
  return { schema: "str005-v2-native-readiness-v1", result: "selected_native_checks_passed",
    firmwareCommit: options.expectedSourceCommit, elfSha256: options.expectedElfSha256,
    packageManifestSha256: common.packageManifestSha256, appImageSha256: common.appImageSha256,
    appImageBytes: common.appImageBytes, imageSlotBytes: common.imageSlotBytes, sdkconfigSha256: common.sdkconfigSha256,
    sourceFiles: hashes(V2_NATIVE_SOURCE_FILES), auditorSources: hashes(V2_NATIVE_AUDITOR_SOURCES),
    objdumpSha256: common.objdumpSha256, noise: common.noise, production: common.production, telemetry: common.telemetry,
    v2: { ...paths, supplementalRanges: decoded.supplementalRanges, unresolvedJumps: decoded.unresolvedJumps },
    hardwareQualified: false, startupHeapQualified: false, completeCallgraphBound: false };
}

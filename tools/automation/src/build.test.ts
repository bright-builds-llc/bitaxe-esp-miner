import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  buildFirmware, FIRMWARE_BUILD_TIMEOUT_MS, FirmwareBuildTimeoutError, rejectUnknownKconfigWarnings, requireResolvedUsbMemoryContract,
  requireResolvedCoreDumpContract, requireCoreDumpPartition, requireDebugArtifacts,
} from "./build.js";
import { createFakeProcessPort, type ProcessLifetime, type ProcessOutcome } from "./process.js";
import { maybeTypedFailureCategory, maybeTypedFailurePublicValue } from "./typed-failure.js";

const resolved = [
  "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304",
  "CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0",
  "CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384",
  "CONFIG_ESP_MAIN_TASK_AFFINITY=0x0",
  "CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5",
  "CONFIG_SPIRAM_TRY_ALLOCATE_WIFI_LWIP=y",
  "CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM=6",
  "CONFIG_ESP_WIFI_STATIC_TX_BUFFER_NUM=6",
  "CONFIG_ESP_WIFI_TX_BUFFER_TYPE=0",
  "CONFIG_ESP_WIFI_DYNAMIC_RX_BUFFER_NUM=32",
  "CONFIG_ESP_WIFI_AMPDU_RX_ENABLED=y",
  "CONFIG_ESP_WIFI_RX_BA_WIN=12",
  "",
].join("\n");

test("firmware build rejects unknown Kconfig symbols", () => {
  // Arrange / Act / Assert
  assert.throws(
    () => rejectUnknownKconfigWarnings("warning: unknown kconfig symbol 'BOGUS' assigned to '1'"),
    /unknown Kconfig/u,
  );
  assert.doesNotThrow(() => rejectUnknownKconfigWarnings("warning: ordinary compiler warning"));
});

test("resolved coexistence profile preserves static DMA TX and the internal reserve", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireResolvedUsbMemoryContract(resolved));
  for (const [before, after] of [["98304", "65536"], ["TX_BUFFER_TYPE=0", "TX_BUFFER_TYPE=1"]] as const) {
    assert.throws(() => requireResolvedUsbMemoryContract(resolved.replace(before, after)), /USB memory contract/u);
  }
});

test("packaging rejects stale large Wi-Fi pools instead of trusting requested defaults", () => {
  // Arrange
  const stale = resolved.replace("STATIC_RX_BUFFER_NUM=6", "STATIC_RX_BUFFER_NUM=16")
    .replace("STATIC_TX_BUFFER_NUM=6", "STATIC_TX_BUFFER_NUM=16");
  // Act / Assert
  assert.throws(() => requireResolvedUsbMemoryContract(stale), /USB memory contract/u);
});

test("missing or duplicate resolved buffer fields fail closed", () => {
  // Arrange
  const field = "CONFIG_ESP_WIFI_RX_BA_WIN=12\n";
  // Act / Assert
  for (const candidate of [resolved.replace(field, ""), resolved + field]) {
    assert.throws(() => requireResolvedUsbMemoryContract(candidate), /USB memory contract/u);
  }
});


test("main telemetry handoff rejects incompatible stack or scheduling configuration", () => {
  // Arrange / Act / Assert
  for (const [before, after] of [
    ["MAIN_TASK_STACK_SIZE=16384", "MAIN_TASK_STACK_SIZE=8192"],
    ["MAIN_TASK_AFFINITY=0x0", "MAIN_TASK_AFFINITY=0x1"],
    ["PTHREAD_TASK_PRIO_DEFAULT=5", "PTHREAD_TASK_PRIO_DEFAULT=1"],
  ] as const) {
    assert.throws(() => requireResolvedUsbMemoryContract(resolved.replace(before, after)), /USB memory contract/u);
  }
});

const captureConfig = [
  "CONFIG_ESP_COREDUMP_ENABLE_TO_FLASH=y", "CONFIG_ESP_COREDUMP_DATA_FORMAT_ELF=y",
  "CONFIG_ESP_COREDUMP_CHECKSUM_SHA256=y", "# CONFIG_ESP_COREDUMP_CAPTURE_DRAM is not set",
  "CONFIG_ESP_COREDUMP_MAX_TASKS_NUM=64", "CONFIG_ESP_COREDUMP_STACK_SIZE=4096",
  "CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE=y", "CONFIG_ESP_COREDUMP_CHECK_BOOT=y",
  "CONFIG_ESP_CONSOLE_UART_DEFAULT=y", "CONFIG_ESP_CONSOLE_SECONDARY_NONE=y",
  "CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y", "CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64",
  "CONFIG_COMPILER_OPTIMIZATION_PERF=y", "# CONFIG_COMPILER_OPTIMIZATION_DEBUG is not set",
  "# CONFIG_ESP_COREDUMP_ENABLE_TO_UART is not set", "# CONFIG_ESP_COREDUMP_ENABLE_TO_NONE is not set",
  "# CONFIG_ESP_COREDUMP_LOGS is not set",
].join("\n");

test("resolved crash capture rejects missing, duplicate and weakened settings", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireResolvedCoreDumpContract(captureConfig));
  for (const line of captureConfig.split("\n").filter(value => !value.startsWith("#"))) {
    assert.throws(() => requireResolvedCoreDumpContract(captureConfig.replace(line, "")), /core dump contract/u);
    assert.throws(() => requireResolvedCoreDumpContract(`${captureConfig}\n${line}`), /core dump contract/u);
  }
  assert.throws(() => requireResolvedCoreDumpContract(captureConfig.replace("FLASH_NO_OVERWRITE=y", "FLASH_NO_OVERWRITE=n")), /core dump contract/u);
  assert.throws(() => requireResolvedCoreDumpContract(`${captureConfig}\nCONFIG_ESP_COREDUMP_ENABLE_TO_UART=y`), /core dump contract/u);
});

test("resolved crash capture requires explicit heap capture disablement while retaining task capacity", () => {
  // Arrange
  const disabled = "# CONFIG_ESP_COREDUMP_CAPTURE_DRAM is not set";

  // Act / Assert
  assert.doesNotThrow(() => requireResolvedCoreDumpContract(captureConfig));
  for (const replacement of ["", "CONFIG_ESP_COREDUMP_CAPTURE_DRAM=y", "CONFIG_ESP_COREDUMP_CAPTURE_DRAM=n"]) {
    assert.throws(() => requireResolvedCoreDumpContract(captureConfig.replace(disabled, replacement)), /requires disabled CONFIG_ESP_COREDUMP_CAPTURE_DRAM/u);
  }
  assert.throws(() => requireResolvedCoreDumpContract(`${captureConfig}\nCONFIG_ESP_COREDUMP_CAPTURE_DRAM=y`), /requires disabled CONFIG_ESP_COREDUMP_CAPTURE_DRAM/u);
  assert.throws(() => requireResolvedCoreDumpContract(captureConfig.replace("MAX_TASKS_NUM=64", "MAX_TASKS_NUM=32")), /core dump contract/u);
  assert.throws(() => requireResolvedCoreDumpContract(captureConfig.replace("STACK_SIZE=4096", "STACK_SIZE=2048")), /core dump contract/u);
});

function corePartition(): Buffer {
  const table = Buffer.alloc(32);
  table.writeUInt16LE(0x50aa, 0);
  table[2] = 1; table[3] = 3;
  table.writeUInt32LE(0xf12000, 4); table.writeUInt32LE(0xee000, 8);
  table.write("coredump", 12, "ascii");
  return table;
}

test("generated dump partition consumes only the existing protected trailing capacity", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireCoreDumpPartition(corePartition()));
  for (const [offset, value] of [[4, 0xf10000], [8, 0xef000], [8, 0x10000], [28, 1]] as const) {
    const table = corePartition(); table.writeUInt32LE(value, offset);
    assert.throws(() => requireCoreDumpPartition(table), /reserved tail contract/u);
  }
  assert.throws(() => requireCoreDumpPartition(Buffer.alloc(32)), /one core dump/u);
  assert.throws(() => requireCoreDumpPartition(Buffer.concat([corePartition(), corePartition()])), /one core dump/u);
});

test("debug artifact gate rejects stripped ELF and empty linker map", () => {
  // Arrange
  const sections = "  12 .debug_info 00000040 00000000\n  13 .debug_line 00000020 00000000";
  const map = "Linker script and memory map\n.text 0x42000000";
  // Act / Assert
  assert.doesNotThrow(() => requireDebugArtifacts(sections, map));
  assert.throws(() => requireDebugArtifacts(sections.replace(".debug_info", ".comment"), map), /debug_info/u);
  assert.throws(() => requireDebugArtifacts(sections.replace("00000020", "00000000"), map), /debug_line/u);
  assert.throws(() => requireDebugArtifacts(sections, ""), /linker map/u);
});

test("Rust release DWARF cannot silently switch ESP-IDF to debug optimization", () => {
  // Arrange: esp-idf-sys maps Cargo DEBUG=true to its generated -Og defaults.
  const drifted = captureConfig.replace("CONFIG_COMPILER_OPTIMIZATION_PERF=y", "# CONFIG_COMPILER_OPTIMIZATION_PERF is not set")
    .replace("# CONFIG_COMPILER_OPTIMIZATION_DEBUG is not set", "CONFIG_COMPILER_OPTIMIZATION_DEBUG=y");
  // Act / Assert
  assert.throws(() => requireResolvedCoreDumpContract(drifted), /COMPILER_OPTIMIZATION_PERF/u);
  assert.throws(() => requireResolvedCoreDumpContract(captureConfig + "\nCONFIG_COMPILER_OPTIMIZATION_DEBUG=y"), /COMPILER_OPTIMIZATION_DEBUG/u);
  assert.doesNotThrow(() => requireResolvedCoreDumpContract(captureConfig));
});

test("resolved default-allocation policy prefers PSRAM at every size and keeps the internal reserve", () => {
  // Arrange: only default malloc placement changes, not required internal task stacks.
  const stale = resolved.replace("MALLOC_ALWAYSINTERNAL=0", "MALLOC_ALWAYSINTERNAL=2048");
  const reducedReserve = resolved.replace("MALLOC_RESERVE_INTERNAL=98304", "MALLOC_RESERVE_INTERNAL=32768");
  // Act / Assert
  assert.throws(() => requireResolvedUsbMemoryContract(stale), /MALLOC_ALWAYSINTERNAL/u);
  assert.throws(() => requireResolvedUsbMemoryContract(reducedReserve), /MALLOC_RESERVE_INTERNAL/u);
  assert.doesNotThrow(() => requireResolvedUsbMemoryContract(resolved));
});

/** A workspace holding only the inputs buildFirmware reads before it starts Cargo. */
async function buildWorkspace(): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-build-timeout-"));
  await mkdir(path.join(root, "firmware/bitaxe"), { recursive: true });
  await writeFile(path.join(root, "firmware/bitaxe/sdkconfig.defaults"), "CONFIG_EXAMPLE=y\n");
  for (const name of ["stamp.txt", "identity.defaults", "timestamp.txt"]) await writeFile(path.join(root, name), "x\n");
  return root;
}

/** Run buildFirmware against a fake port whose Cargo run ends with `outcome`; returns the error and the requested lifetime. */
async function buildWith(root: string, outcome: ProcessOutcome): Promise<{ error: unknown; lifetime: ProcessLifetime | undefined }> {
  let lifetime: ProcessLifetime | undefined;
  const port = createFakeProcessPort(async (_spec, maybeLifetime) => { lifetime = maybeLifetime; return outcome; });
  const request = { outputDir: "out", buildProvenanceStamp: "stamp.txt", identitySdkconfigDefaults: "identity.defaults",
    buildTimestampUtc: "timestamp.txt", buildMode: "normal" } as const;
  const error = await buildFirmware(root, request, port).then(() => undefined, (failure: unknown) => failure);
  return { error, lifetime };
}

test("a firmware build killed at its bound is a typed firmware_build_timed_out failure with its diagnostic", async () => {
  // Arrange
  const root = await buildWorkspace();

  try {
    // Act
    const { error, lifetime } = await buildWith(root, { exitCode: 1, stdout: "", stderr: "Compiling esp-idf-sys\n", timedOut: true });

    // Assert
    assert.ok(error instanceof FirmwareBuildTimeoutError);
    assert.equal(lifetime, FIRMWARE_BUILD_TIMEOUT_MS);
    assert.equal(error.message, "firmware_build_timed_out (bound 900000 ms)");
    assert.equal(maybeTypedFailureCategory(error), "timeout");
    const publicValue = maybeTypedFailurePublicValue(error);
    assert.deepEqual([publicValue?.["failure"], publicValue?.["timeout_ms"]], ["firmware_build_timed_out", 900_000]);
    const diagnostic = path.join(root, error.diagnostic, "cargo.stderr");
    assert.equal(await readFile(diagnostic, "utf8"), "Compiling esp-idf-sys\n");
    assert.equal((await stat(diagnostic)).mode & 0o777, 0o600);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("an ordinary firmware build failure stays an untyped process failure", async () => {
  // Arrange
  const root = await buildWorkspace();

  try {
    // Act
    const { error } = await buildWith(root, { exitCode: 101, stdout: "", stderr: "error[E0308]\n", timedOut: false });

    // Assert
    assert.ok(error instanceof Error && !(error instanceof FirmwareBuildTimeoutError));
    assert.match(error.message, /^firmware Cargo build failed; protected diagnostic scratch\/firmware-build-/u);
    assert.equal(maybeTypedFailureCategory(error), undefined);
  } finally {
    await rm(root, { recursive: true });
  }
});

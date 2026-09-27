import assert from "node:assert/strict";
import test from "node:test";

import { rejectUnknownKconfigWarnings, requireResolvedUsbMemoryContract, requireResolvedCoreDumpContract, requireCoreDumpPartition, requireDebugArtifacts } from "./build.js";

const resolved = [
  "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304",
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
  "CONFIG_ESP_COREDUMP_CHECKSUM_SHA256=y", "CONFIG_ESP_COREDUMP_CAPTURE_DRAM=y",
  "CONFIG_ESP_COREDUMP_MAX_TASKS_NUM=64", "CONFIG_ESP_COREDUMP_STACK_SIZE=4096",
  "CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE=y", "CONFIG_ESP_COREDUMP_CHECK_BOOT=y",
  "CONFIG_ESP_CONSOLE_UART_DEFAULT=y", "CONFIG_ESP_CONSOLE_SECONDARY_NONE=y",
  "CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y", "CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64",
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

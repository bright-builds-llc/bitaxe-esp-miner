import { createHash } from "node:crypto";
import { copyFile, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";

import { internalCommandSpec } from "./contracts.generated.js";
import { verifyFirmwareStackBudget } from "./firmware-stack-budget.js";
import type { ProcessPort } from "./process.js";

export type BuildFirmwareRequest = {
  readonly outputDir: string;
  readonly buildProvenanceStamp: string;
  readonly identitySdkconfigDefaults: string;
  readonly buildTimestampUtc: string;
  readonly buildMode: "normal" | "rollback-probe";
};

const target = "xtensa-esp32s3-espidf";
const packageName = "bitaxe-firmware";

export async function buildFirmware(
  workspaceRoot: string,
  request: BuildFirmwareRequest,
  processPort: ProcessPort,
): Promise<void> {
  const outputDir = path.resolve(workspaceRoot, request.outputDir);
  const rollbackProbe = request.buildMode === "rollback-probe";
  const artifactPrefix = rollbackProbe ? "bitaxe-firmware-rollback-probe" : "bitaxe-firmware";
  const cargoTargetDir = path.join(
    workspaceRoot,
    rollbackProbe ? ".bazel-firmware-rollback-probe-target" : ".bazel-firmware-target",
  );
  const sourceElf = path.join(cargoTargetDir, target, "release", packageName);
  const sourceMap = `${sourceElf}.map`;
  const provenanceStamp = path.resolve(workspaceRoot, request.buildProvenanceStamp);
  const identityDefaults = path.resolve(workspaceRoot, request.identitySdkconfigDefaults);
  const buildTimestamp = path.resolve(workspaceRoot, request.buildTimestampUtc);
  await Promise.all([readFile(provenanceStamp), readFile(identityDefaults), readFile(buildTimestamp)]);
  await mkdir(outputDir, { recursive: true });

  const outputSdkconfig = path.join(outputDir, "sdkconfig");
  const outputDefaults = path.join(outputDir, "sdkconfig.defaults");
  await Promise.all([rm(outputSdkconfig, { force: true }), rm(outputDefaults, { force: true })]);
  const [baseDefaults, identityText] = await Promise.all([
    readFile(path.join(workspaceRoot, "firmware/bitaxe/sdkconfig.defaults"), "utf8"),
    readFile(identityDefaults, "utf8"),
  ]);
  await writeFile(outputDefaults, `${baseDefaults.trimEnd()}\n\n${identityText.trimEnd()}\n`);

  const espEnvironment = await processPort.loadEspEnvironment();
  const environment = {
    ...espEnvironment,
    AR_xtensa_esp32s3_espidf: "xtensa-esp32s3-elf-ar",
    BITAXE_BUILD_PROVENANCE_STAMP: provenanceStamp,
    BITAXE_BUILD_TIMESTAMP_UTC_FILE: buildTimestamp,
    CARGO_TARGET_DIR: cargoTargetDir,
    // sdkconfig pins IDF -O2 independently of Cargo DEBUG=true for Rust DWARF.
    CARGO_PROFILE_RELEASE_DEBUG: "2",
    CARGO_PROFILE_RELEASE_STRIP: "none",
    BITAXE_LINKER_MAP: sourceMap,
    CC_xtensa_esp32s3_espidf: "xtensa-esp32s3-elf-gcc",
    CFLAGS_xtensa_esp32s3_espidf: "-mlongcalls",
    ESP_IDF_SDKCONFIG: outputSdkconfig,
    ESP_IDF_SDKCONFIG_DEFAULTS: outputDefaults,
    ESP_IDF_SYS_ROOT_CRATE: packageName,
    ESP_IDF_TOOLS_INSTALL_DIR: "workspace",
    ESP_IDF_VERSION: "tag:v5.5.4",
    BITAXE_OTA_ROLLBACK_PROBE: rollbackProbe ? "1" : "0",
  };
  const cargo = await processPort.run(
    internalCommandSpec(
      "cargo",
      ["build", "-p", packageName, "--release", "--target", target],
      (value) => value,
      environment,
    ),
  );
  if (cargo.exitCode !== 0) {
    await mkdir(path.join(workspaceRoot, "scratch"), { recursive: true });
    const diagnostic = await mkdtemp(path.join(workspaceRoot, "scratch/firmware-build-"));
    await writeFile(path.join(diagnostic, "cargo.stderr"), cargo.stderr, { mode: 0o600 });
    throw new Error(`firmware Cargo build failed; protected diagnostic ${path.relative(workspaceRoot, diagnostic)}`);
  }
  rejectUnknownKconfigWarnings(`${cargo.stdout}\n${cargo.stderr}`);

  const sections = await processPort.run(internalCommandSpec(
    "xtensa-esp32s3-elf-objdump", ["-h", sourceElf], (value) => value, espEnvironment,
  ));
  if (sections.timedOut || sections.exitCode !== 0) throw new Error("firmware debug section inspection failed");
  requireDebugArtifacts(sections.stdout, await readFile(sourceMap, "utf8"));
  const disassembly = await processPort.run(internalCommandSpec(
    "xtensa-esp32s3-elf-objdump",
    ["-d", "-C", sourceElf],
    (value) => value,
    espEnvironment,
  ));
  if (disassembly.timedOut || disassembly.exitCode !== 0) {
    throw new Error("firmware stack disassembly failed");
  }
  verifyFirmwareStackBudget(disassembly.stdout);
  const buildLabel = await requiredStampField(provenanceStamp, "build_label");
  const generated = await findGeneratedIdfBuild(cargoTargetDir, buildLabel);
  const sdkconfig = await readFile(path.join(generated, "sdkconfig"), "utf8");
  requireResolvedUsbMemoryContract(sdkconfig);
  requireResolvedCoreDumpContract(sdkconfig);
  requireCoreDumpPartition(await readFile(path.join(generated, "build/partition_table/partition-table.bin")));
  await copyFile(sourceElf, path.join(outputDir, `${artifactPrefix}.elf`));
  await copyFile(sourceMap, path.join(outputDir, `${artifactPrefix}.map`));
  const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
  await writeFile(path.join(outputDir, `${artifactPrefix}.debug.json`), `${JSON.stringify({
    schema: "bitaxe-firmware-debug-artifacts-v1",
    elf: { path: `${artifactPrefix}.elf`, sha256: digest(await readFile(sourceElf)) },
    map: { path: `${artifactPrefix}.map`, sha256: digest(await readFile(sourceMap)) },
    sdkconfig: { path: `${artifactPrefix}.sdkconfig`, sha256: digest(sdkconfig) },
    sourceProvenanceSha256: digest(await readFile(provenanceStamp)),
  }, null, 2)}\n`);
  await Promise.all([
    copyFile(path.join(generated, "sdkconfig"), path.join(outputDir, `${artifactPrefix}.sdkconfig`)),
    copyFile(
      path.join(generated, "build/bootloader/bootloader.bin"),
      path.join(outputDir, `${artifactPrefix}-bootloader.bin`),
    ),
    copyFile(
      path.join(generated, "build/partition_table/partition-table.bin"),
      path.join(outputDir, `${artifactPrefix}-partition-table.bin`),
    ),
    copyFile(
      path.join(generated, "build/ota_data_initial.bin"),
      path.join(outputDir, `${artifactPrefix}-otadata-initial.bin`),
    ),
  ]);
}

export function rejectUnknownKconfigWarnings(output: string): void {
  if (/warning: unknown kconfig symbol /u.test(output)) {
    throw new Error("firmware sdkconfig contains an unknown Kconfig symbol");
  }
}

export function requireResolvedUsbMemoryContract(sdkconfig: string): void {
  const lines = sdkconfig.split(/\r?\n/u);
  for (const required of [
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
  ]) {
    const key = required.slice(0, required.indexOf("=") + 1);
    const values = lines.filter(line => line.startsWith(key));
    if (values.length !== 1 || values[0] !== required) {
      throw new Error(`resolved USB memory contract does not contain ${required}`);
    }
  }
}

async function requiredStampField(file: string, key: string): Promise<string> {
  const values = (await readFile(file, "utf8"))
    .split(/\r?\n/u)
    .filter((line) => line.startsWith(`${key}=`))
    .map((line) => line.slice(key.length + 1))
    .filter((value) => value.length > 0);
  if (values.length !== 1) throw new Error(`provenance stamp requires exactly one ${key}`);
  const value = values[0];
  if (value === undefined) throw new Error(`provenance stamp is missing ${key}`);
  return value;
}

async function findGeneratedIdfBuild(cargoTargetDir: string, buildLabel: string): Promise<string> {
  const buildRoot = path.join(cargoTargetDir, target, "release", "build");
  const candidates = (await readdir(buildRoot, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("esp-idf-sys-"))
    .map((entry) => path.join(buildRoot, entry.name, "out"));
  const matches: string[] = [];
  for (const candidate of candidates) {
    try {
      const sdkconfig = await readFile(path.join(candidate, "sdkconfig"), "utf8");
      if (
        sdkconfig.split(/\r?\n/u).includes(`CONFIG_APP_PROJECT_VER="${buildLabel}"`) &&
        sdkconfig.split(/\r?\n/u).includes("CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64")
      ) {
        await Promise.all([
          readFile(path.join(candidate, "build/bootloader/bootloader.bin")),
          readFile(path.join(candidate, "build/partition_table/partition-table.bin")),
          readFile(path.join(candidate, "build/ota_data_initial.bin")),
        ]);
        matches.push(candidate);
      }
    } catch {
      // A stale Cargo build directory is not a matching generated build.
    }
  }
  if (matches.length !== 1) {
    throw new Error(`expected exactly one generated ESP-IDF build for ${buildLabel}, found ${String(matches.length)}`);
  }
  const match = matches[0];
  if (match === undefined) throw new Error("generated ESP-IDF build disappeared");
  return match;
}

/** Fail closed on resolved capture settings; requested defaults are not evidence. */
export function requireResolvedCoreDumpContract(sdkconfig: string): void {
  const lines = sdkconfig.split(/\r?\n/u);
  for (const required of [
    "CONFIG_ESP_COREDUMP_ENABLE_TO_FLASH=y", "CONFIG_ESP_COREDUMP_DATA_FORMAT_ELF=y",
    "CONFIG_ESP_COREDUMP_CHECKSUM_SHA256=y",
    "CONFIG_ESP_COREDUMP_MAX_TASKS_NUM=64", "CONFIG_ESP_COREDUMP_STACK_SIZE=4096",
    "CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE=y", "CONFIG_ESP_COREDUMP_CHECK_BOOT=y",
    "CONFIG_ESP_CONSOLE_UART_DEFAULT=y", "CONFIG_ESP_CONSOLE_SECONDARY_NONE=y",
    "CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y", "CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64",
    "CONFIG_COMPILER_OPTIMIZATION_PERF=y",
  ]) {
    const prefix = required.slice(0, required.indexOf("=") + 1);
    const matches = lines.filter(line => line.startsWith(prefix));
    if (matches.length !== 1 || matches[0] !== required) throw new Error(`resolved core dump contract missing ${required}`);
  }
  for (const key of ["CONFIG_ESP_COREDUMP_ENABLE_TO_UART", "CONFIG_ESP_COREDUMP_ENABLE_TO_NONE", "CONFIG_ESP_COREDUMP_LOGS", "CONFIG_COMPILER_OPTIMIZATION_DEBUG", "CONFIG_ESP_COREDUMP_CAPTURE_DRAM"]) {
    if (!lines.includes(`# ${key} is not set`) || lines.some(line => line.startsWith(`${key}=`))) {
      throw new Error(`resolved core dump contract requires disabled ${key}`);
    }
  }
}

/** Inspect the generated binary table so auto-offset changes cannot evade the gate. */
export function requireCoreDumpPartition(table: Buffer): void {
  let matches = 0;
  for (let offset = 0; offset + 32 <= table.length; offset += 32) {
    if (table.readUInt16LE(offset) !== 0x50aa) break;
    const label = table.subarray(offset + 12, offset + 28).toString("ascii").replace(/\0.*$/u, "");
    if (label !== "coredump") continue;
    matches += 1;
    if (table[offset + 2] !== 1 || table[offset + 3] !== 3 || table.readUInt32LE(offset + 4) !== 0xf12000 ||
        table.readUInt32LE(offset + 8) !== 0xee000 || table.readUInt32LE(offset + 28) !== 0) {
      throw new Error("generated core dump partition violates reserved tail contract");
    }
  }
  if (matches !== 1) throw new Error("generated table requires one core dump partition");
}

/** Debug information is retained in the exact optimized firmware ELF, never a rebuilt surrogate. */
export function requireDebugArtifacts(sections: string, map: string): void {
  for (const section of [".debug_info", ".debug_line"]) {
    const line = sections.split(/\r?\n/u).find(value => value.trim().split(/\s+/u)[1] === section);
    const size = line?.trim().split(/\s+/u)[2];
    if (size === undefined || !/^[0-9a-f]+$/iu.test(size) || Number.parseInt(size, 16) === 0) {
      throw new Error(`firmware ELF lacks nonempty ${section}`);
    }
  }
  if (!map.includes("Linker script and memory map") || !map.includes(".text")) throw new Error("firmware linker map missing");
}

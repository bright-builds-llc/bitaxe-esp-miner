import { copyFile, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
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
  if (cargo.exitCode !== 0) throw new Error("firmware Cargo build failed");
  rejectUnknownKconfigWarnings(`${cargo.stdout}\n${cargo.stderr}`);

  const sourceElf = path.join(cargoTargetDir, target, "release", packageName);
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
  await copyFile(sourceElf, path.join(outputDir, `${artifactPrefix}.elf`));
  const buildLabel = await requiredStampField(provenanceStamp, "build_label");
  const generated = await findGeneratedIdfBuild(cargoTargetDir, buildLabel);
  requireResolvedUsbMemoryContract(await readFile(path.join(generated, "sdkconfig"), "utf8"));
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
    "CONFIG_TINYUSB_TASK_STACK_SIZE=3072",
    "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=65536",
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

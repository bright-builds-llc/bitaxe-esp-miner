import { copyFile, cp, mkdtemp, mkdir, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

import { internalCommandSpec } from "./contracts.generated.js";
import type { ProcessPort, ProcessOutcome } from "./process.js";

export type PackageFirmwareRequest = {
  readonly firmwareElf: string;
  readonly buildProvenanceStamp: string;
  readonly espIdfSdkconfig: string;
  readonly bootloaderBin: string;
  readonly partitionTableBin: string;
  readonly otadataInitialBin: string;
  readonly outDir: string;
  readonly manifest: string;
};

const otaPartitionBytes = 0x400000;

export async function packageFirmware(
  workspaceRoot: string,
  request: PackageFirmwareRequest,
  processPort: ProcessPort,
  xtaskProgram: string,
): Promise<void> {
  const input = resolveRequest(workspaceRoot, request);
  await Promise.all(Object.values(input).map((file) => file === input.outDir || file === input.manifest ? undefined : readFile(file)));
  await requireSuccess(
    processPort.run(internalCommandSpec(xtaskProgram, ["verify-reference"], (value) => value)),
    "reference verification",
  );
  await mkdir(input.outDir, { recursive: true });

  const packageElf = path.join(input.outDir, "bitaxe-ultra205.elf");
  const firmwareOta = path.join(input.outDir, "esp-miner.bin");
  const wwwImage = path.join(input.outDir, "www.bin");
  const otadata = path.join(input.outDir, "otadata-initial.bin");
  const factoryImage = path.join(input.outDir, "bitaxe-ultra205-factory.bin");
  await copyFile(input.firmwareElf, packageElf);

  const buildLabel = await requiredStampField(input.buildProvenanceStamp, "build_label");
  const staging = await mkdtemp(path.join(tmpdir(), "bitaxe-www-"));
  try {
    await cp(path.join(workspaceRoot, "firmware/bitaxe/static/www"), staging, { recursive: true });
    await writeFile(path.join(staging, "version.txt"), `${buildLabel}\n`, { mode: 0o600 });
    const sdkconfig = await readFile(input.espIdfSdkconfig, "utf8");
    if (
      !sdkconfig.split(/\r?\n/u).includes(`CONFIG_APP_PROJECT_VER="${buildLabel}"`) ||
      !sdkconfig.split(/\r?\n/u).includes("CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64")
    ) {
      throw new Error("explicit ESP-IDF sdkconfig does not match the build identity contract");
    }
    const spiffsgen = await findManagedTool(workspaceRoot, [
      ".embuild/espressif/esp-idf/v5.5.4/components/spiffs/spiffsgen.py",
    ]);
    const esptool = await findEsptool(workspaceRoot);
    await requireSuccess(
      processPort.run(
        internalCommandSpec(
          "python3",
          [spiffsgen, "--obj-name-len", "64", "0x300000", staging, wwwImage],
          (value) => value,
        ),
      ),
      "SPIFFS image generation",
    );
    await copyFile(input.otadataInitialBin, otadata);
    await requireSuccess(
      processPort.run(
        internalCommandSpec(
          esptool,
          [
            "--chip", "esp32s3", "elf2image", "--version", "2", "--flash_size", "16MB",
            "--flash_mode", "dio", "--flash_freq", "80m", "--elf-sha256-offset", "0xb0",
            "--min-rev-full", "0", "--max-rev-full", "99", "-o", firmwareOta, packageElf,
          ],
          (value) => value,
        ),
      ),
      "firmware image generation",
    );
    if ((await stat(firmwareOta)).size > otaPartitionBytes) {
      throw new Error("firmware OTA image exceeds the ota_0 partition");
    }
    const imageInfo = await requireSuccess(
      processPort.run(internalCommandSpec(esptool, ["image_info", "--version", "2", firmwareOta], (value) => value)),
      "firmware image inspection",
    );
    const appVersion = requiredPrefixedLine(imageInfo.stdout, "App version: ");
    const elfSha = requiredPrefixedLine(imageInfo.stdout, "ELF file SHA256: ");
    if (!/^[0-9a-f]{64}$/u.test(elfSha) || /^0+$/u.test(elfSha)) {
      throw new Error("application descriptor ELF SHA-256 is invalid");
    }
    await requireSuccess(
      processPort.run(
        internalCommandSpec(
          esptool,
          [
            "--chip", "esp32s3", "merge_bin", "--flash_mode", "dio", "--flash_size", "16MB",
            "--flash_freq", "80m", "0x0", input.bootloaderBin, "0x8000", input.partitionTableBin,
            "0x10000", firmwareOta, "0x410000", wwwImage, "0xf10000", otadata, "-o", factoryImage,
          ],
          (value) => value,
        ),
      ),
      "factory image merge",
    );
    await requireSuccess(
      processPort.run(
        internalCommandSpec(
          xtaskProgram,
          [
            "package-firmware", "--board", "205",
            "--firmware-elf", packageElf, "--build-provenance-stamp", input.buildProvenanceStamp,
            "--app-descriptor-version", appVersion, "--app-elf-sha256", elfSha,
            "--firmware-ota-image", firmwareOta, "--www-bin", wwwImage,
            "--partition-table", path.join(workspaceRoot, "firmware/bitaxe/partitions-ultra205.csv"),
            "--otadata-initial", otadata, "--default-flash-image", packageElf, "--out-dir", input.outDir,
            "--manifest", input.manifest, "--factory-image", factoryImage,
            "--release-name", "bitaxe-ultra205", "--install-notes", "docs/release/ultra-205.md",
            "--license-inventory", "docs/release/license-inventory.md",
            "--provenance-manifest", "docs/release/provenance-manifest.md",
            "--otadata-source", input.otadataInitialBin,
          ],
          (value) => value,
        ),
      ),
      "package manifest generation",
    );
  } finally {
    await rm(staging, { recursive: true, force: true });
  }
}

function resolveRequest(workspaceRoot: string, request: PackageFirmwareRequest): PackageFirmwareRequest {
  return {
    firmwareElf: path.resolve(workspaceRoot, request.firmwareElf),
    buildProvenanceStamp: path.resolve(workspaceRoot, request.buildProvenanceStamp),
    espIdfSdkconfig: path.resolve(workspaceRoot, request.espIdfSdkconfig),
    bootloaderBin: path.resolve(workspaceRoot, request.bootloaderBin),
    partitionTableBin: path.resolve(workspaceRoot, request.partitionTableBin),
    otadataInitialBin: path.resolve(workspaceRoot, request.otadataInitialBin),
    outDir: path.resolve(workspaceRoot, request.outDir),
    manifest: path.resolve(workspaceRoot, request.manifest),
  };
}

export async function requiredStampField(file: string, key: string): Promise<string> {
  const values = (await readFile(file, "utf8"))
    .split(/\r?\n/u)
    .filter((line) => line.startsWith(`${key}=`))
    .map((line) => line.slice(key.length + 1))
    .filter(Boolean);
  if (values.length !== 1 || values[0] === undefined) throw new Error(`missing unique stamp field ${key}`);
  return values[0];
}

async function findManagedTool(workspaceRoot: string, candidates: readonly string[]): Promise<string> {
  for (const candidate of candidates) {
    const resolved = path.join(workspaceRoot, candidate);
    try {
      if ((await stat(resolved)).isFile()) return resolved;
    } catch {
      // Continue to the next pinned managed location.
    }
  }
  throw new Error("required managed ESP-IDF tool is unavailable");
}

export async function findEsptool(workspaceRoot: string): Promise<string> {
  const pythonRoot = path.join(workspaceRoot, ".embuild/espressif/python_env");
  try {
    const environments = await readdir(pythonRoot);
    const candidates = environments
      .filter((name) => name.startsWith("idf5.5") && name.endsWith("_env"))
      .map((name) => path.join(pythonRoot, name, "bin/esptool.py"));
    for (const candidate of candidates) {
      try {
        if ((await stat(candidate)).isFile()) return candidate;
      } catch {
        // Continue to the managed source-tree fallback.
      }
    }
  } catch {
    // Continue to the managed source-tree fallback.
  }
  return findManagedTool(workspaceRoot, [
    ".embuild/espressif/esp-idf/v5.5.4/components/esptool_py/esptool/esptool.py",
  ]);
}

export async function requireSuccess(promise: Promise<ProcessOutcome>, label: string): Promise<ProcessOutcome> {
  const outcome = await promise;
  if (outcome.exitCode !== 0) throw new Error(`${label} failed`);
  return outcome;
}

function requiredPrefixedLine(text: string, prefix: string): string {
  const values = text.split(/\r?\n/u).filter((line) => line.startsWith(prefix));
  if (values.length !== 1 || values[0] === undefined) throw new Error(`image metadata is missing ${prefix.trim()}`);
  return values[0].slice(prefix.length).trim();
}

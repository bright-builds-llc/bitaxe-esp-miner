import { createHash } from "node:crypto";
import { readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { internalCommandSpec } from "./contracts.generated.js";
import { findEsptool, requiredStampField, requireSuccess } from "./package.js";
import type { ProcessPort } from "./process.js";

export const rollbackProbeSchema = "bitaxe-rollback-probe-v1";
const otaPartitionBytes = 0x400000;

export type PackageRollbackProbeRequest = {
  readonly firmwareElf: string;
  readonly buildProvenanceStamp: string;
  readonly outputImage: string;
  readonly metadata: string;
};

export type RollbackProbeMetadata = {
  readonly schema_version: typeof rollbackProbeSchema;
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly source_dirty: boolean;
  readonly build_label: string;
  readonly app_elf_sha256: string;
  readonly ota_image_sha256: string;
  readonly ota_image_bytes: number;
  readonly rollback_probe: true;
};

export async function packageRollbackProbe(
  workspaceRoot: string,
  request: PackageRollbackProbeRequest,
  processPort: ProcessPort,
): Promise<RollbackProbeMetadata> {
  const firmwareElf = path.resolve(workspaceRoot, request.firmwareElf);
  const stampPath = path.resolve(workspaceRoot, request.buildProvenanceStamp);
  const outputImage = path.resolve(workspaceRoot, request.outputImage);
  const metadataPath = path.resolve(workspaceRoot, request.metadata);
  await Promise.all([readFile(firmwareElf), readFile(stampPath)]);
  const sourceDirty = await requiredStampField(stampPath, "source_dirty");
  if (sourceDirty !== "true" && sourceDirty !== "false") {
    throw new Error("rollback probe source provenance is invalid");
  }
  const esptool = await findEsptool(workspaceRoot);
  await requireSuccess(
    processPort.run(internalCommandSpec(
      esptool,
      [
        "--chip", "esp32s3", "elf2image", "--version", "2", "--flash_size", "16MB",
        "--flash_mode", "dio", "--flash_freq", "80m", "--elf-sha256-offset", "0xb0",
        "--min-rev-full", "0", "--max-rev-full", "99", "-o", outputImage, firmwareElf,
      ],
      (value) => value,
    )),
    "rollback probe image generation",
  );
  const image = await readFile(outputImage);
  if (image.length === 0 || image.length > otaPartitionBytes) {
    throw new Error("rollback probe image exceeds the admitted OTA partition");
  }
  const imageInfo = await requireSuccess(
    processPort.run(internalCommandSpec(
      esptool,
      ["image_info", "--version", "2", outputImage],
      (value) => value,
    )),
    "rollback probe image inspection",
  );
  const appElfSha256 = requiredPrefixedLine(imageInfo.stdout, "ELF file SHA256: ");
  if (!/^[0-9a-f]{64}$/u.test(appElfSha256) || /^0+$/u.test(appElfSha256)) {
    throw new Error("rollback probe ELF SHA-256 is invalid");
  }
  const metadata: RollbackProbeMetadata = {
    schema_version: rollbackProbeSchema,
    source_commit: await requiredStampField(stampPath, "source_commit"),
    reference_commit: await requiredStampField(stampPath, "reference_commit"),
    source_dirty: sourceDirty === "true",
    build_label: await requiredStampField(stampPath, "build_label"),
    app_elf_sha256: appElfSha256,
    ota_image_sha256: createHash("sha256").update(image).digest("hex"),
    ota_image_bytes: image.length,
    rollback_probe: true,
  };
  await writeFile(metadataPath, `${JSON.stringify(metadata, null, 2)}\n`, {
    encoding: "utf8",
    mode: 0o600,
  });
  if ((await stat(metadataPath)).size === 0) throw new Error("rollback probe metadata is empty");
  return metadata;
}

function requiredPrefixedLine(text: string, prefix: string): string {
  const values = text.split(/\r?\n/u).filter((line) => line.startsWith(prefix));
  if (values.length !== 1 || values[0] === undefined) {
    throw new Error(`image metadata is missing ${prefix.trim()}`);
  }
  return values[0].slice(prefix.length).trim();
}

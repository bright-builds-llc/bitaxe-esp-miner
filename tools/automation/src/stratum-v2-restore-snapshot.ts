import { chmod, mkdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

import {
  restoreBundleSchema,
  sha256,
  snapshotRangeTemplates,
  type InstalledIdentity,
  type SnapshotRestoreBundle,
  validateSnapshotRanges,
} from "./stratum-v2-restore-model.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import { findEsptool } from "./package.js";

function hex(value: number): string { return `0x${value.toString(16)}`; }

async function requireSuccess(
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
): Promise<string> {
  const outcome = await runCampaignProcess(workspace, program, args, timeoutMillis);
  if (outcome.exitCode !== 0) throw new Error("snapshot child failed");
  return outcome.stdout;
}

function prefixedLine(text: string, prefix: string): string {
  const values = text.split(/\r?\n/u).filter(line => line.startsWith(prefix));
  if (values.length !== 1 || values[0] === undefined) throw new Error("snapshot image metadata missing");
  return values[0].slice(prefix.length).trim();
}

async function validatePartitionTable(workspace: string, snapshot: string): Promise<void> {
  const actual = await readFile(snapshot);
  const expected = await readFile(
    path.join(workspace, "bazel-bin/firmware/bitaxe/bitaxe-firmware-partition-table.bin"),
  );
  if (actual.length !== 0x1000 || expected.length > actual.length) {
    throw new Error("snapshot partition table size mismatch");
  }
  if (!actual.subarray(0, expected.length).equals(expected)
    || actual.subarray(expected.length).some(byte => byte !== 0xff)) {
    throw new Error("snapshot partition table mismatch");
  }
}

export async function captureRestoreSnapshot(
  workspace: string,
  port: string,
  privateRoot: string,
  identity: InstalledIdentity,
  captureSourceCommit: string,
  planSha256: string,
): Promise<SnapshotRestoreBundle> {
  const directory = path.join(privateRoot, "snapshot");
  await mkdir(directory, { mode: 0o700 });
  await chmod(directory, 0o700);
  const templates = snapshotRangeTemplates("snapshot");
  const ranges = [];
  for (const template of templates) {
    const candidate = path.join(privateRoot, template.path);
    await requireSuccess(workspace, "espflash", [
      "read-flash",
      "--chip", "esp32s3",
      "--port", port,
      "--non-interactive",
      "--before", "usb-reset",
      "--after", "hard-reset",
      "--skip-update-check",
      hex(template.address),
      hex(template.size),
      candidate,
    ], 300_000);
    await chmod(candidate, 0o600);
    const metadata = await stat(candidate);
    if (!metadata.isFile() || metadata.size !== template.size) {
      throw new Error("snapshot range size mismatch");
    }
    ranges.push({ ...template, sha256: sha256(await readFile(candidate)) });
  }
  validateSnapshotRanges(ranges);
  const partition = ranges.find(range => range.name === "partition_table");
  if (partition === undefined) throw new Error("snapshot partition table missing");
  await validatePartitionTable(workspace, path.join(privateRoot, partition.path));
  const running = ranges.find(range => range.name === identity.running_partition);
  if (running === undefined) throw new Error("snapshot running image missing");
  const esptool = await findEsptool(workspace);
  const info = await requireSuccess(
    workspace,
    esptool,
    ["image_info", "--version", "2", path.join(privateRoot, running.path)],
    30_000,
  );
  if (prefixedLine(info, "App version: ") !== identity.build_label
    || prefixedLine(info, "ELF file SHA256: ") !== identity.app_elf_sha256) {
    throw new Error("snapshot running image identity mismatch");
  }
  return {
    schema_version: restoreBundleSchema,
    kind: "flash_snapshot_v1",
    board: 205,
    installed_identity: identity,
    ranges,
    capture_source_commit: captureSourceCommit,
    plan_sha256: planSha256,
  };
}

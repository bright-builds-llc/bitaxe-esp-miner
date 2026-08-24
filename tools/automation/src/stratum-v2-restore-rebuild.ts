import { chmod, mkdir, readFile, symlink, writeFile } from "node:fs/promises";
import path from "node:path";

import { buildFirmware } from "./build.js";
import { packageFirmware } from "./package.js";
import { createLocalProcessPort } from "./process.js";
import {
  normalizePackageCandidate,
  type PackageCandidate,
} from "./stratum-v2-restore-artifacts.js";
import {
  formattedDateStatus,
  sha256,
  stableStatus,
  type InstalledIdentity,
} from "./stratum-v2-restore-model.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";

export type RebuildResult = {
  readonly attempted: boolean;
  readonly maybeCandidate: PackageCandidate | undefined;
};

async function requireSuccess(
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
): Promise<string> {
  const outcome = await runCampaignProcess(workspace, program, args, timeoutMillis);
  if (outcome.exitCode !== 0) throw new Error("recovery child failed");
  return outcome.stdout.trim();
}

async function writePrivate(candidate: string, contents: string): Promise<void> {
  await writeFile(candidate, contents, { mode: 0o600, flag: "wx" });
  await chmod(candidate, 0o600);
}

export async function rebuildInstalledPackage(
  workspace: string,
  privateRoot: string,
  identity: InstalledIdentity,
): Promise<RebuildResult> {
  if (identity.source_dirty) return { attempted: false, maybeCandidate: undefined };
  const commitAvailable = await runCampaignProcess(
    workspace,
    "git",
    ["cat-file", "-e", `${identity.source_commit}^{commit}`],
    5_000,
  );
  if (commitAvailable.exitCode !== 0) return { attempted: false, maybeCandidate: undefined };
  const worktree = path.join(privateRoot, "rebuild-worktree");
  const inputs = path.join(privateRoot, "rebuild-inputs");
  let worktreeCreated = false;
  try {
    await requireSuccess(
      workspace,
      "git",
      ["worktree", "add", "--detach", worktree, identity.source_commit],
      30_000,
    );
    worktreeCreated = true;
    await requireSuccess(
      worktree,
      "git",
      ["submodule", "update", "--init", "--recursive", "reference/esp-miner"],
      120_000,
    );
    try {
      await symlink(path.join(workspace, ".embuild"), path.join(worktree, ".embuild"));
    } catch {
      // Historical worktrees may already have generated ESP tooling state.
    }
    await mkdir(inputs, { mode: 0o700 });
    await chmod(inputs, 0o700);
    const stable = path.join(inputs, "stable-status.private.txt");
    const volatile = path.join(inputs, "volatile-status.private.txt");
    const stamp = path.join(inputs, "build-provenance.private.stamp");
    const defaults = path.join(inputs, "build-identity.private.defaults");
    const timestamp = path.join(inputs, "build-timestamp.private.txt");
    await writePrivate(stable, stableStatus(identity));
    await writePrivate(volatile, formattedDateStatus(identity.build_timestamp_utc));
    const xtask = path.join(workspace, "bazel-bin/tools/xtask/xtask");
    await requireSuccess(
      worktree,
      xtask,
      [
        "materialize-build-provenance",
        "--status-file", stable,
        "--volatile-status-file", volatile,
        "--stamp-out", stamp,
        "--sdkconfig-defaults-out", defaults,
        "--build-timestamp-out", timestamp,
      ],
      30_000,
    );
    for (const candidate of [stamp, defaults, timestamp]) await chmod(candidate, 0o600);
    const output = path.join(worktree, "recovery-build");
    const processPort = createLocalProcessPort({ cwd: worktree, timeoutMs: 900_000 });
    await buildFirmware(worktree, {
      outputDir: output,
      buildProvenanceStamp: stamp,
      identitySdkconfigDefaults: defaults,
      buildTimestampUtc: timestamp,
      buildMode: "normal",
    }, processPort);
    const packageDirectory = path.join(worktree, "recovery-package");
    const manifestPath = path.join(packageDirectory, "bitaxe-ultra205-package.json");
    await packageFirmware(worktree, {
      firmwareElf: path.join(output, "bitaxe-firmware.elf"),
      buildProvenanceStamp: stamp,
      espIdfSdkconfig: path.join(output, "bitaxe-firmware.sdkconfig"),
      bootloaderBin: path.join(output, "bitaxe-firmware-bootloader.bin"),
      partitionTableBin: path.join(output, "bitaxe-firmware-partition-table.bin"),
      otadataInitialBin: path.join(output, "bitaxe-firmware-otadata-initial.bin"),
      outDir: packageDirectory,
      manifest: manifestPath,
    }, processPort, xtask);
    const manifestDocument = await readFile(manifestPath, "utf8");
    const manifest = JSON.parse(manifestDocument) as Record<string, unknown>;
    if (manifest["source_commit"] !== identity.source_commit
      || manifest["reference_commit"] !== identity.reference_commit
      || manifest["app_elf_sha256"] !== identity.app_elf_sha256) {
      return { attempted: true, maybeCandidate: undefined };
    }
    const artifacts = manifest["artifacts"];
    if (!Array.isArray(artifacts)) return { attempted: true, maybeCandidate: undefined };
    const factory = artifacts.find(value => typeof value === "object"
      && value !== null
      && !Array.isArray(value)
      && (value as Record<string, unknown>)["kind"] === "factory_merged_image") as
      | Record<string, unknown>
      | undefined;
    if (typeof factory?.["sha256"] !== "string") return { attempted: true, maybeCandidate: undefined };
    const normalized = await normalizePackageCandidate(
      { manifestPath, manifestDocument, factorySha256: factory["sha256"] },
      path.join(privateRoot, "recovered-package"),
      worktree,
    );
    if (sha256(await readFile(path.join(privateRoot, "recovered-package/bitaxe-ultra205.elf")))
      !== identity.app_elf_sha256) {
      return { attempted: true, maybeCandidate: undefined };
    }
    return { attempted: true, maybeCandidate: normalized };
  } catch {
    return { attempted: true, maybeCandidate: undefined };
  } finally {
    if (worktreeCreated) {
      const removed = await runCampaignProcess(
        workspace,
        "git",
        ["worktree", "remove", "--force", worktree],
        30_000,
      );
      const pruned = await runCampaignProcess(workspace, "git", ["worktree", "prune"], 5_000);
      if (removed.exitCode !== 0 || pruned.exitCode !== 0) {
        throw new Error("owned recovery worktree cleanup failed");
      }
    }
  }
}

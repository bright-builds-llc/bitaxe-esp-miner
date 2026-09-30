import { readFile, lstat } from 'node:fs/promises';
import { resolve, dirname, join } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
export const sha = value => createHash('sha256').update(value).digest('hex');

export async function treeHash(repo, roots) {
  const paths = execFileSync('git', ['ls-files', '-c', '-o', '--exclude-standard', '-z', '--', ...roots], { cwd: repo, encoding: 'utf8' }).split('\0').filter(Boolean);
  const entries = [];
  for (const path of [...new Set(paths)].sort()) {
    const full = join(repo, path);
    try {
      const info = await lstat(full);
      if (info.isSymbolicLink() || !info.isFile()) throw Error('binding_source_kind');
      entries.push([path, sha(await readFile(full))]);
    } catch (error) { if (error.code !== 'ENOENT') throw error; entries.push([path, 'deleted']); }
  }
  return sha(JSON.stringify(entries.sort((a,b) => a[0].localeCompare(b[0], 'en'))));
}

export async function sourceBindings(repo) {
  const git = args => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
  return { source_commit: git(['rev-parse', 'HEAD']), source_dirty: !!git(['status', '--porcelain', '--untracked-files=normal']),
    source_sha256: await treeHash(repo, ['Cargo.toml', 'Cargo.lock', 'crates', 'firmware/bitaxe', 'firmware/virtual-ultra205', 'tools/virtual-board', 'tools/stratum-v2-fixture', 'scripts/virtual-board', 'scripts/virtual-emulator']),
    sdk: 'v5.5.4', model_sha256: await treeHash(repo, ['crates/bitaxe-virtual-board']),
    scenario_sha256: await treeHash(repo, ['crates/bitaxe-simulation']),
    fixture_sha256: await treeHash(repo, ['tools/stratum-v2-fixture']),
    validator_sha256: await treeHash(repo, ['scripts/virtual-board', 'tools/virtual-board']),
  };
}

export async function packageBinding(path, repo) {
  const bytes = await readFile(path); const manifest = JSON.parse(bytes);
  if (![3,4].includes(manifest.schema_version) || manifest.execution_profile && manifest.execution_profile !== 'physical-ultra205'
      || manifest.build_identity?.channel === 'virtual-ultra205') throw Error('production_manifest_profile');
  const base = dirname(resolve(path));
  const artifacts = [];
  for (const item of manifest.artifacts ?? []) {
    if (typeof item.path !== 'string' || resolve(base, item.path) !== join(base, item.path) || item.path.includes('..') || item.path.startsWith('/')) throw Error('production_artifact_path');
    const sourcePath = item.kind === 'partition_table' && item.path === 'firmware/bitaxe/partitions-ultra205.csv' ? join(repo, item.path) : join(base, item.path);
    const content = await readFile(sourcePath);
    if (sha(content) !== item.sha256) throw Error('production_artifact_digest');
    if (item.kind === 'firmware_elf') {
      if (content.length < 20 || !content.subarray(0, 7).equals(Buffer.from([127,69,76,70,1,1,1])) || content.readUInt16LE(18) !== 94) throw Error('production_elf_architecture');
      if (content.includes(Buffer.from('BITAXE_EXECUTION_PROFILE=virtual-ultra205'))) throw Error('production_virtual_marker');
    }
    artifacts.push({ ...item, bytes: content.length, source_path: sourcePath });
  }
  const elf = artifacts.filter(item => item.kind === 'firmware_elf');
  if (elf.length !== 1 || elf[0].sha256 !== manifest.app_elf_sha256) throw Error('production_elf_binding');
  const sdk = await readFile(join(base, 'bitaxe-firmware.sdkconfig'));
  for (const setting of ['CONFIG_IDF_TARGET="esp32s3"', 'CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y']) {
    if (!sdk.toString('utf8').split('\n').includes(setting)) throw Error('production_sdk_target');
  }
  if (manifest.image_metadata?.esp_idf_version !== 'v5.5.4' || manifest.image_metadata?.board !== '205') throw Error('production_board_sdk_identity');
  return { manifest_sha256: sha(bytes), app_elf_sha256: manifest.app_elf_sha256, source_commit: manifest.source_commit,
    source_dirty: manifest.build_identity?.source_dirty, sdkconfig_sha256: sha(sdk), artifacts };
}

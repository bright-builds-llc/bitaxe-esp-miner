import { readFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const ROOTS = ['crates/bitaxe-api','crates/bitaxe-asic','crates/bitaxe-config','crates/bitaxe-core','crates/bitaxe-safety',
  'crates/bitaxe-stratum','crates/bitaxe-runtime','crates/bitaxe-virtual-board','crates/bitaxe-worker-control',
  'crates/bitaxe-simulation','tools/stratum-v2-fixture','tools/virtual-board'];
const FILES = ['Cargo.toml','Cargo.lock','BUILD.bazel','MODULE.bazel','MODULE.bazel.lock','.bazelrc','.bazelversion','rust-toolchain.toml','.cargo/config.toml','firmware/bitaxe/sdkconfig.defaults'];
async function sourceFiles(repo, folder) {
  const files = [];
  for (const item of await readdir(join(repo, folder), { withFileTypes: true })) {
    const path = `${folder}/${item.name}`;
    if (item.isSymbolicLink()) throw Error('compiler_source_symlink');
    if (item.isDirectory()) files.push(...await sourceFiles(repo, path));
    else if ((/\.(rs|toml|bzl|mjs)$/.test(path) || item.name === 'BUILD.bazel')) files.push(path);
  }
  return files;
}
async function folderHash(repo, folder) {
  const hash = createHash('sha256');
  for (const path of (await sourceFiles(repo, folder)).filter(path => /\.(rs|toml)$/.test(path)).sort()) {
    const name = Buffer.from(path), bytes = await readFile(join(repo, path));
    for (const item of [name, bytes]) { const length = Buffer.alloc(8); length.writeBigUInt64LE(BigInt(item.length)); hash.update(length); hash.update(item); }
  }
  return hash.digest('hex');
}
/** Independently rejudge immutable embedded compiler inputs and live evidence fields. */
export async function compilerBindings(repo) {
  const files = [...FILES];
  for (const root of ROOTS) files.push(...await sourceFiles(repo, root));
  const rows = [];
  for (const path of [...new Set(files)].sort()) rows.push([path, sha(await readFile(join(repo, path)))]);
  return { compiled_inputs_sha256: sha(JSON.stringify(rows)),
    worker_control_sha256: await folderHash(repo, 'crates/bitaxe-worker-control'),
    fixture_sha256: await folderHash(repo, 'tools/stratum-v2-fixture'),
    cargo_lock_sha256: sha(await readFile(join(repo, 'Cargo.lock'))) };
}

export function classifyHost(envelope, life, expected) {
  const bound = envelope?.schema === 'bitaxe_virtual_board_run_v3' && envelope.scenario === expected.scenario && envelope.seed === expected.seed
    && envelope.source_commit === expected.source.source_commit && envelope.source_dirty === expected.source.source_dirty
    && envelope.executable_sha256 === expected.binarySha
    && Object.entries(expected.compiler).every(([field,value]) => envelope[field] === value)
    && (!expected.production ? envelope.package_binding === null : envelope.package_binding?.manifest_sha256 === expected.production.manifest_sha256
      && envelope.package_binding?.app_elf_sha256 === expected.production.app_elf_sha256 && envelope.package_binding?.source_commit === expected.production.source_commit);
  if (!bound || !life.current_resources_released || envelope.result?.checks.some(row => row.status === 'failed')) return 'failed';
  if (envelope.result?.checks.some(row => row.status === 'unsupported')) {
    const completion = expected.completion;
    return completion?.code === 2 && !completion.timedOut && !completion.interrupted && completion.released === true ? 'unsupported' : 'failed';
  }
  return life.qualification_success && !life.earliest_failure && !life.cleanup_failures.length && envelope.passed ? 'passed' : 'failed';
}

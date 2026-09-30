import { readFile, mkdir, stat } from 'node:fs/promises';
import { join, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import { EMULATOR, selectRelease } from './lock.mjs';
import { runPrivate } from './process.mjs';

export function managedPaths(repo) {
  const idf = join(repo, '.embuild/espressif/esp-idf/v5.5.4');
  return { idf, python: join(repo, '.embuild/espressif/python_env/idf5.5_py3.9_env/bin/python'),
    binary: join(repo, '.embuild/espressif/tools/qemu-xtensa', EMULATOR.version, 'qemu/bin/qemu-system-xtensa') };
}

export async function bootstrap(repo, evidence) {
  const tools = managedPaths(repo);
  selectRelease(JSON.parse(await readFile(join(tools.idf, 'tools/tools.json'), 'utf8')));
  await runPrivate(tools.python, [join(tools.idf, 'tools/idf_tools.py'), 'install', `qemu-xtensa@${EMULATOR.version}`], evidence, 'bootstrap',
    { cwd: repo, env: { PATH: process.env.PATH, HOME: process.env.HOME, IDF_TOOLS_PATH: join(repo, '.embuild/espressif'), IDF_PATH: tools.idf }, timeoutMs: 300000 });
  return doctor(repo, evidence);
}

export async function doctor(repo, evidence) {
  const tools = managedPaths(repo);
  const selected = selectRelease(JSON.parse(await readFile(join(tools.idf, 'tools/tools.json'), 'utf8')));
  await stat(tools.binary);
  await runPrivate(tools.python, [fileURLToPath(new URL('./verify_install.py', import.meta.url)),
    join(repo, '.embuild/espressif/dist', basename(new URL(selected.url).pathname)),
    join(repo, '.embuild/espressif/tools/qemu-xtensa', EMULATOR.version), selected.sha256, String(selected.size)],
    evidence, 'verify-install', { timeoutMs: 30000 });
  await runPrivate(tools.binary, ['--version'], evidence, 'doctor', { timeoutMs: 10000 });
  const version = await readFile(join(evidence, 'doctor.stdout.log'), 'utf8');
  if (!version.includes(`QEMU emulator version 9.2.2 (${EMULATOR.version})`)) throw Error('emulator_binary_version');
  return { schema: 'bitaxe-virtual-emulator-doctor-v1', installed: true, version: EMULATOR.version,
    archiveSha256: selected.sha256, deviceEffects: false };
}

/** Evidence roots are new and private, never an overwrite of an earlier execution. */
export async function newEvidenceRoot(path) {
  await mkdir(path, { recursive: false, mode: 0o700 });
  const info = await stat(path);
  if ((info.mode & 0o077) !== 0) throw Error('emulator_evidence_permissions');
}

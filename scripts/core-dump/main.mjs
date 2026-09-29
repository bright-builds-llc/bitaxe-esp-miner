import { execFileSync } from 'node:child_process';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, privateDirectory, sha256, snapshot } from './files.mjs';
import { analysisArguments } from './analysis.mjs';
import { privateProcess } from './process.mjs';

const sourceDirectory = dirname(fileURLToPath(import.meta.url));
const expectedVersion = '1.17.2';

async function children(root) {
  try { return await readdir(root); } catch (error) { if (error.code === 'ENOENT') return []; throw error; }
}

/** Discover only the repo-managed IDF 5.5 environment and pinned decoder. */
export async function managedTools(repo) {
  const idf = join(repo, '.embuild/espressif/esp-idf/v5.5.4');
  const version = await readFile(join(idf, 'tools/cmake/version.cmake'), 'utf8');
  check(['MAJOR 5', 'MINOR 5', 'PATCH 4'].every(part => version.includes(`set(IDF_VERSION_${part})`)), 'decoder_version');
  const environments = join(repo, '.embuild/espressif/python_env');
  for (const environment of (await children(environments)).sort().reverse()) {
    if (!/^idf5\.5_py\d+\.\d+_env$/.test(environment)) continue;
    const root = join(environments, environment);
    for (const python of await children(join(root, 'lib'))) {
      const metadata = join(root, 'lib', python, 'site-packages', `esp_coredump-${expectedVersion}.dist-info`, 'METADATA');
      try {
        check((await readFile(metadata, 'utf8')).includes(`\nVersion: ${expectedVersion}\n`), 'decoder_version');
        return { python: join(root, 'bin/python'), idf };
      } catch (error) { if (error.code !== 'ENOENT') throw error; }
    }
  }
  throw new Error('managed_decoder_missing');
}

/** Select exactly the debugger recommended by the pinned IDF tool manifest. */
export async function managedGdb(repo) {
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  const tools = manifest.tools.filter(tool => tool.name === 'xtensa-esp-elf-gdb');
  check(tools.length === 1, 'managed_gdb_version');
  const recommended = tools[0].versions.filter(version => version.status === 'recommended');
  check(recommended.length === 1 && /^[a-zA-Z0-9_.-]+$/.test(recommended[0].name), 'managed_gdb_version');
  const version = recommended[0].name;
  const path = join(repo, '.embuild/espressif/tools/xtensa-esp-elf-gdb', version, 'xtensa-esp-elf-gdb/bin/xtensa-esp32s3-elf-gdb');
  try { return { path, version, sha256: sha256(await readFile(path)) }; }
  catch (error) { if (error.code === 'ENOENT') throw new Error('managed_gdb_missing'); throw error; }
}

function argumentsFor(argv) {
  const [action, ...rest] = argv;
  check(['inspect', 'analyze', 'verify-cutoff', 'verify-provenance'].includes(action), 'action_required');
  const options = {};
  for (let index = 0; index < rest.length; index += 2) {
    const key = rest[index];
    check(['--dump', '--elf', '--elf-sha256', '--private-root'].includes(key) && !Object.hasOwn(options, key), 'arguments_invalid');
    check(typeof rest[index + 1] === 'string', 'arguments_invalid');
    options[key] = rest[index + 1];
  }
  check(Object.keys(options).length === 4 && /^[a-f0-9]{64}$/.test(options['--elf-sha256']), 'arguments_invalid');
  return { action, options };
}

/** Offline-only raw core inspection. No device acquisition or automatic serial fallback. */
export async function main(argv) {
  process.umask(0o077);
  const { action, options } = argumentsFor(argv);
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  const root = options['--private-root'];
  check(isAbsolute(root) && resolve(root) === root && root.startsWith(`${repo}/`), 'private_root_invalid');
  await privateDirectory(dirname(root));
  await privateDirectory(dirname(options['--dump']));
  try { execFileSync('git', ['check-ignore', '--quiet', '--', root], { cwd: repo, stdio: 'ignore' }); }
  catch { throw new Error('private_root_not_ignored'); }
  check(options['--dump'].startsWith(`${repo}/`), 'input_not_ignored');
  try { execFileSync('git', ['check-ignore', '--quiet', '--', options['--dump']], { cwd: repo, stdio: 'ignore' }); }
  catch { throw new Error('input_not_ignored'); }
  await mkdir(root, { mode: 0o700 }); // Existing roots always fail before sensitive reads.
  const dump = join(root, 'dump.raw'), elf = join(root, 'firmware.elf');
  const dumpSha = await snapshot(options['--dump'], dump, true);
  const elfSha = await snapshot(options['--elf'], elf, false);
  check(elfSha === options['--elf-sha256'], 'elf_identity');
  await writeFile(join(root, 'inputs.json'), JSON.stringify({ schema: 'bitaxe-private-core-inputs/1', dump_sha256: dumpSha,
    elf_sha256: elfSha, inspector_sha256: sha256(await readFile(join(sourceDirectory, 'decode_core.py'))),
    cutoff_verifier_sha256: sha256(await readFile(join(sourceDirectory, 'cutoff.py'))),
    provenance_verifier_sha256: sha256(await readFile(join(sourceDirectory, 'provenance.py'))) }), { flag: 'wx', mode: 0o600 });
  const tools = await managedTools(repo);
  const tmp = join(root, 'tmp'); await mkdir(tmp, { mode: 0o700 });
  const env = { PATH: process.env.PATH, HOME: process.env.HOME, GDBHISTFILE: join(root, 'gdb-history'), TMPDIR: tmp, TMP: tmp, TEMP: tmp, PYTHONNOUSERSITE: '1', PYTHONDONTWRITEBYTECODE: '1', IDF_PATH: tools.idf };
  await privateProcess(tools.python, [join(sourceDirectory, 'decode_core.py'), dump, elf, elfSha, join(root, 'inspection.json'), action], root, 'inspect', env);
  if (action === 'analyze') {
    const gdb = await managedGdb(repo);
    await privateProcess(gdb.path, ['--nx', '--batch', '--version'], root, 'gdb-version', env);
    const versionOutput = await readFile(join(root, 'gdb-version.stdout'), 'utf8');
    check(versionOutput.split('\n')[0] === `GNU gdb (esp-gdb) ${gdb.version}`, 'managed_gdb_version');
    await writeFile(join(root, 'analysis-inputs.json'), JSON.stringify({ schema: 'bitaxe-private-core-analysis/1',
      elf_sha256: elfSha, dump_sha256: dumpSha, gdb_version: gdb.version, gdb_sha256: gdb.sha256, decoder_version: expectedVersion,
      core_format: 'verified-elf', explicit_core: true, core_elf_sha256: sha256(await readFile(join(root, 'core.elf'))), frame_arguments: 'none', bounded_ms: 60000 }), { flag: 'wx', mode: 0o600 });
    await privateProcess(gdb.path, analysisArguments(elf, join(root, 'core.elf')), root, 'analysis', env, 60000);
  }
  check(sha256(await readFile(dump)) === dumpSha && sha256(await readFile(elf)) === elfSha, 'input_changed');
  const summary = JSON.parse(await readFile(join(root, 'inspection.json'), 'utf8'));
  check(summary.elf_sha256 === elfSha && summary.dump_sha256 === dumpSha && summary.full_elf_identity_verified === true, 'inspection_invalid');
  if (['verify-cutoff', 'verify-provenance'].includes(action)) check(summary.native_cutoff_verified === true && summary.captured_memory_verified === true && summary.generation_revoked === true && summary.asic_outputs_disabled === true, 'inspection_invalid');
  if (action === 'verify-provenance') check(summary.fault_provenance_verified === true, 'inspection_invalid');
  return { status: 'passed', action, ...summary };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => console.log(JSON.stringify(result))).catch(error => {
    const allowed = new Set(['action_required', 'arguments_invalid', 'private_root_invalid', 'private_root_not_ignored', 'absolute_path_required',
      'symlink_rejected', 'input_file', 'input_not_private', 'input_changed', 'elf_identity', 'managed_decoder_missing', 'managed_gdb_missing',
      'managed_gdb_version', 'private_parent_required', 'input_not_ignored', 'decoder_version', 'decoder_spawn', 'decoder_timeout', 'decoder_interrupted', 'decoder_failed', 'inspection_invalid']);
    console.error(JSON.stringify({ status: 'blocked', category: allowed.has(error.message) ? error.message : 'private_operation_failed' }));
    process.exitCode = 1;
  });
}

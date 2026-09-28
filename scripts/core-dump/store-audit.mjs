import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { noiseNativeCalls } from '../noise-native-calls.mjs';
import { check, sha256, privateDirectory } from './files.mjs';
import { STORE_SEAMS, instructions, elfView, symbols, auditStore, auditBootInitializer } from './store-audit-model.mjs';

/** Prove the selected SDK routes and the wrappers' bounded diagnostic-only memory accesses. */
export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'store_audit_arguments');
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  const tool = manifest.tools.find(item => item.name === 'xtensa-esp-elf');
  const versions = tool.versions.filter(item => item.status === 'recommended'); check(versions.length === 1, 'store_audit_tool');
  const bin = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', versions[0].name, 'xtensa-esp-elf/bin');
  const elfPath = resolve(argv[1]), output = resolve(argv[3]), bytes = await readFile(elfPath), elf = elfView(bytes);
  const toolHashes = { objdump: sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-objdump'))), nm: sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-nm'))) };
  const run = (name, args) => execFileSync(join(bin, `xtensa-esp32s3-elf-${name}`), args, { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, timeout: 30000 });
  const entries = symbols(run('nm', ['-S', elfPath]));
  const names = [...STORE_SEAMS.map(name => `__wrap_esp_core_dump_${name}`), 'bitaxe_core_dump_receipt_update',
    'esp_core_dump_write_elf_and_check', 'esp_core_dump_store', '__esp_system_init_fn_init_coredump'];
  const functions = new Map();
  for (const name of names) {
    const found = entries.filter(row => row.name === name); check(found.length === 1, 'store_audit_symbol');
    const symbol = found[0], text = run('objdump', ['-d', `--start-address=${symbol.address}`, `--stop-address=${symbol.address + symbol.size}`, elfPath]);
    const code = instructions(text);
    for (const row of code.filter(row => row.op === 'l32r')) {
      const literal = /,\s*([0-9a-f]+)/u.exec(row.args), annotation = /\(([0-9a-f]+) <.+>\)\s*$/u.exec(row.args);
      if (annotation) check(literal && elf.word(parseInt(literal[1], 16)) === parseInt(annotation[1], 16), 'store_audit_literal_annotation');
    }
    functions.set(name, { ...symbol, code });
  }
  const sdkCalls = new Map(['esp_core_dump_write_elf_and_check', 'esp_core_dump_store'].map(name => {
    const fn = functions.get(name); return [name, noiseNativeCalls({ address: fn.address, symbol: fn.name, instructions: fn.code })];
  }));
  const boot = functions.get('__esp_system_init_fn_init_coredump');
  const result = { ...auditStore({ elf, entries, functions, sdkCalls }),
    ...auditBootInitializer(elf, entries, noiseNativeCalls({ address: boot.address, symbol: boot.name, instructions: boot.code })), elf_sha256: sha256(bytes),
    objdump_sha256: toolHashes.objdump, nm_sha256: toolHashes.nm };
  check(sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-objdump'))) === toolHashes.objdump &&
    sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-nm'))) === toolHashes.nm, 'store_audit_tool_changed');
  check(sha256(await readFile(elfPath)) === result.elf_sha256, 'store_audit_elf_changed');
  await privateDirectory(resolve(output, '..'));
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => console.log(JSON.stringify(result))).catch(error => {
    const category = /^store_audit_[a-z_]+$/u.test(error.message) ? error.message : 'store_audit_blocked';
    console.error(JSON.stringify({ native_core_store_audit: 'blocked', category })); process.exitCode = 1;
  });
}

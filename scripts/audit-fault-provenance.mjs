import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseNativeFunctions } from './telemetry-stack-audit.mjs';
import { elfView, symbols } from './core-dump/store-audit-model.mjs';
import { verifyIrqRomLeaf } from './fault-provenance-rom.mjs';
import { resolveNoiseInstructions } from './noise-native-disassembly.mjs';
import { auditFaultProvenance, ROOTS } from './fault-provenance-audit.mjs';
import { check, sha256, privateDirectory } from './core-dump/files.mjs';
/** Audit the exact native image without any device or debugger access. */
export async function inspectFaultProvenance(elfPath, repo) {
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json')));
  const versions = manifest.tools.find(tool => tool.name === 'xtensa-esp-elf').versions.filter(v => v.status === 'recommended');
  check(versions.length === 1, 'fault_audit_tool');
  const bin = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', versions[0].name, 'xtensa-esp-elf/bin');
  const toolHashes = Object.fromEntries(await Promise.all(['objdump', 'nm'].map(async tool => [tool, sha256(await readFile(join(bin, `xtensa-esp32s3-elf-${tool}`)))])));
  const run = (tool, args) => execFileSync(join(bin, `xtensa-esp32s3-elf-${tool}`), args, { encoding: 'utf8', maxBuffer: 128 * 1024 * 1024, timeout: 60000 });
  const bytes = await readFile(elfPath), elf = elfView(bytes);
  const entries = symbols(run('nm', ['-S', elfPath]));
  const romVersions = manifest.tools.find(tool => tool.name === 'esp-rom-elfs').versions.filter(v => v.status === 'recommended');
  check(romVersions.length === 1, 'fault_audit_rom_version');
  const romPath = join(repo, '.embuild/espressif/tools/esp-rom-elfs', romVersions[0].name, 'esp32s3_rev0_rom.elf');
  const romBytes = await readFile(romPath), selectorBytes = await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/idf_py_actions/roms.json'));
  const romLeaf = verifyIrqRomLeaf(entries, symbols(run('nm', ['-S', romPath])), parseNativeFunctions(run('objdump', ['-d', '--disassemble=_xtos_set_intlevel', romPath]) + run('objdump', ['-d', '--disassemble=__call__xtos_set_intlevel', romPath])), romBytes, JSON.parse(selectorBytes));
  const disassembly = run('objdump', ['-d', elfPath]), functions = parseNativeFunctions(disassembly);
  const deadline = performance.now() + 60000;
  const decoded = await resolveNoiseInstructions(functions, disassembly, async (start, end) => {
    check(performance.now() < deadline, 'fault_audit_decode_deadline');
    return run('objdump', ['-d', `--start-address=${start}`, `--stop-address=${end}`, elfPath]);
  }, { rootSymbols: [...ROOTS, 'esp_core_dump_get_task_snapshot'],
    selectSymbol: name => !['esp_core_dump_check_task', 'esp_core_dump_port_set_crashed_tcb'].includes(name) && (ROOTS.includes(name) || !/assert|abort|malloc|calloc|realloc|printf|log_write|panic/iu.test(name)) && [...functions.values()].some(fn => fn.symbol === name && fn.address >= 0x40370000 && fn.address < 0x403e0000), compilerPrivateSpills: true });
  const result = { ...auditFaultProvenance({ entries, functions: decoded.functions, word: elf.word, romLeaf }), supplemental_decode_ranges: decoded.supplementalRanges, irq_rom_exception: { ...romLeaf, address: undefined, tool_version: romVersions[0].name, selector_sha256: sha256(selectorBytes) } };
  check(sha256(await readFile(romPath)) === sha256(romBytes), 'fault_audit_rom_changed');
  for (const tool of ['objdump', 'nm']) check(sha256(await readFile(join(bin, `xtensa-esp32s3-elf-${tool}`))) === toolHashes[tool], 'fault_audit_tool_changed');
  check(sha256(await readFile(elfPath)) === sha256(bytes), 'fault_audit_input_changed');
  return { ...result, elf_sha256: sha256(bytes), objdump_sha256: sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-objdump'))),
    nm_sha256: sha256(await readFile(join(bin, 'xtensa-esp32s3-elf-nm'))) };
}
export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'fault_audit_arguments');
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const result = await inspectFaultProvenance(resolve(argv[1]), repo); await privateDirectory(resolve(argv[3], '..'));
  await writeFile(argv[3], JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 }); return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => console.log(JSON.stringify(value))).catch(error => {
  console.error(JSON.stringify({ native_fault_audit: 'blocked', category: /^fault_audit_[a-z_]+$/u.test(error.message) ? error.message : 'fault_audit_rejected' })); process.exitCode = 1;
});

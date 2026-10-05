import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { auditStackRealignment } from './stack-realignment-audit.mjs';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const check = (ok, code) => { if (!ok) throw Error(code); };

/** The repository root and the objdump pinned by the managed ESP-IDF tool manifest. */
export async function pinnedObjdump() {
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  const versions = manifest.tools.find(item => item.name === 'xtensa-esp-elf').versions.filter(item => item.status === 'recommended');
  check(versions.length === 1, 'stack_realignment_tool_pin');
  return { repo, objdump: join(repo, '.embuild/espressif/tools/xtensa-esp-elf', versions[0].name, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump') };
}
/** The exact Xtensa device ELF bytes; anything else is refused. */
export async function deviceElf(path) {
  const elf = await readFile(path);
  check(elf.subarray(0, 4).equals(Buffer.from([127, 69, 76, 70])) && elf.readUInt16LE(18) === 94, 'stack_realignment_elf_arch');
  return elf;
}

/** Reads the exact device ELF with the pinned objdump; this command never opens a device. */
export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'stack_realignment_arguments');
  const { repo, objdump } = await pinnedObjdump();
  const elfPath = resolve(argv[1]), output = resolve(argv[3]);
  const elf = await deviceElf(elfPath);
  const allowlist = JSON.parse(await readFile(join(repo, 'scripts/stack-realignment-allowlist.json'), 'utf8'));
  check(allowlist.schema === 'stack-realignment-allowlist-v1', 'stack_realignment_allowlist');
  const disassembly = execFileSync(objdump, ['-d', '-C', elfPath], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, timeout: 120000 });
  const result = { ...auditStackRealignment(disassembly, allowlist.callers), elf_sha256: sha256(elf),
    objdump_sha256: sha256(await readFile(objdump)) };
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => {
    console.log(JSON.stringify({ result: result.result, unexpected_callers: result.unexpected_callers, elf_sha256: result.elf_sha256 }));
    if (result.result !== 'no_runtime_realignment_callers') process.exitCode = 1;
  }).catch(error => { console.error(JSON.stringify({ stack_realignment_audit: 'blocked', code: error.message })); process.exitCode = 1; });
}

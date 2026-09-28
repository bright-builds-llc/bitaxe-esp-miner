import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { auditSignedStart } from './signed-start-stack-audit.mjs';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const check = (ok, code) => { if (!ok) throw Error(code); };

/** Read the exact optimized native ELF; this command never opens a device. */
export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'signed_start_arguments');
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  const tool = manifest.tools.find(item => item.name === 'xtensa-esp-elf');
  const versions = tool.versions.filter(item => item.status === 'recommended'); check(versions.length === 1, 'signed_start_tool_pin');
  const objdump = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', versions[0].name, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
  const ownerSource = await readFile(join(repo, 'firmware/bitaxe/src/bwg_worker_usb.rs'), 'utf8');
  check(ownerSource.includes('const OWNER_STACK_BYTES: usize = 16 * 1024;') && ownerSource.includes('.stack_size(OWNER_STACK_BYTES)'), 'signed_start_stack_contract');
  const elfPath = resolve(argv[1]), output = resolve(argv[3]);
  const elf = await readFile(elfPath), binary = await readFile(objdump);
  check(elf.subarray(0, 7).equals(Buffer.from([127,69,76,70,1,1,1])) && elf.readUInt16LE(18) === 94, 'signed_start_elf_arch');
  const disassembly = execFileSync(objdump, ['-d', '-C', elfPath], { encoding: 'utf8', maxBuffer: 128 * 1024 * 1024, timeout: 60000 });
  const result = { ...auditSignedStart(disassembly), elf_sha256: sha256(elf), objdump_sha256: sha256(binary) };
  check(sha256(await readFile(elfPath)) === result.elf_sha256 && sha256(await readFile(objdump)) === result.objdump_sha256, 'signed_start_input_changed');
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => {
    console.log(JSON.stringify(result)); if (result.result !== 'selected_path_with_headroom') process.exitCode = 1;
  }).catch(() => { console.error('{"signed_start_stack_audit":"blocked"}'); process.exitCode = 1; });
}

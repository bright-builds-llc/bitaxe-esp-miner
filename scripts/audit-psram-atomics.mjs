// `just audit-psram-atomics <elf> <output>`: refuses a device ELF with a hardware compare-exchange
// outside the allowed native functions (ADR-0039), such as a preserved older image. It reads the ELF
// with the pinned objdump and never opens a device.
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deviceElf, pinnedObjdump } from './audit-stack-realignment.mjs';
import { auditPsramAtomics } from './psram-atomics-audit.mjs';
import { boundedDisassembly } from './verify-psram-atomics.mjs';

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const check = (ok, code) => { if (!ok) throw Error(code); };

export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'psram_atomics_arguments');
  const { objdump } = await pinnedObjdump();
  const elfPath = resolve(argv[1]), output = resolve(argv[3]);
  const elf = await deviceElf(elfPath);
  const result = { ...auditPsramAtomics(boundedDisassembly(objdump, elfPath)), elf_sha256: sha256(elf),
    objdump_sha256: sha256(await readFile(objdump)) };
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((result) => {
    console.log(JSON.stringify({ result: result.result, unexpected_functions: result.unexpected_functions.length, elf_sha256: result.elf_sha256 }));
    if (result.result !== 'no_unexpected_hardware_atomics') process.exitCode = 1;
  }).catch((error) => { console.error(JSON.stringify({ psram_atomics_audit: 'blocked', code: error.message })); process.exitCode = 1; });
}

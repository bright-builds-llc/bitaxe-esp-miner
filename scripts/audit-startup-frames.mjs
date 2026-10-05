import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deviceElf, pinnedObjdump } from './audit-stack-realignment.mjs';
import { auditStartupFrames } from './startup-frame-audit.mjs';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const check = (ok, code) => { if (!ok) throw Error(code); };

/** Reads the exact device ELF with the pinned objdump; this command never opens a device. */
export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'startup_frame_arguments');
  const { repo, objdump } = await pinnedObjdump();
  const elfPath = resolve(argv[1]), output = resolve(argv[3]);
  const elf = await deviceElf(elfPath);
  const budget = JSON.parse(await readFile(join(repo, 'scripts/startup-frame-budget.json'), 'utf8'));
  check(budget.schema === 'startup-frame-budget-v1', 'startup_frame_budget');
  const disassembly = execFileSync(objdump, ['-d', '-C', elfPath], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, timeout: 120000 });
  const result = { ...auditStartupFrames(disassembly, budget), elf_sha256: sha256(elf), objdump_sha256: sha256(await readFile(objdump)) };
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => {
    console.log(JSON.stringify({ result: result.result, violations: result.violations, elf_sha256: result.elf_sha256 }));
    if (result.result !== 'startup_frames_within_budget') process.exitCode = 1;
  }).catch(error => { console.error(JSON.stringify({ startup_frame_audit: 'blocked', code: error.message })); process.exitCode = 1; });
}

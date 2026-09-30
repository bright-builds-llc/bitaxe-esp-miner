import { readFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { runPrivate } from '../virtual-emulator/process.mjs';
import { writeJson } from './runner.mjs';

const NATIVE = [
  ['signed-start', 'scripts/audit-signed-start-stack.mjs'],
  ['signed-renew', 'scripts/audit-signed-renew-stack.mjs'],
  ['panic-cutoff', 'scripts/core-dump/native-audit.mjs'],
  ['core-store', 'scripts/core-dump/store-audit.mjs'],
  ['fault-provenance', 'scripts/audit-fault-provenance.mjs'],
];
/** Every audit sees the exact full production ELF already bound by the manifest. */
export async function nativeAudits(repo, manifestPath, binding, root) {
  const elf = binding.artifacts.find(item => item.kind === 'firmware_elf');
  const rows = [];
  for (const [id, script] of NATIVE) {
    const output = join(root, `${id}.audit.json`);
    try {
      await runPrivate(process.execPath, [join(repo, script), '--elf', join(dirname(manifestPath), elf.path), '--output', output], root, id, { cwd: repo, timeoutMs: 120000 });
      const result = JSON.parse(await readFile(output, 'utf8'));
      rows.push({ id, status: result.elf_sha256 === binding.app_elf_sha256 ? 'passed' : 'failed', result });
    } catch { rows.push({ id, status: 'failed', category: 'native_audit_rejected' }); }
  }
  await writeJson(join(root, 'native-audits.json'), rows);
  return rows;
}

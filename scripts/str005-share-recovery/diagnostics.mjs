import { proof } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';
export const DIAGNOSTIC_FILE = 'diagnostic-export-recovery.json';
/** Historical receipts remain readable; two competing diagnostic artifacts reject. */
export async function readRecoveryPart(root, stage) {
  if (stage !== 'diagnostics') return (await proof(root, `${stage}.json`)).value;
  const found = [];
  for (const name of [DIAGNOSTIC_FILE, 'diagnostics.json']) {
    try { found.push((await proof(root, name)).value); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  check(found.length <= 1, 'share_recovery_ambiguous_diagnostics');
  if (!found.length) throw Object.assign(Error('missing_diagnostics'), { code: 'ENOENT' });
  return found[0];
}

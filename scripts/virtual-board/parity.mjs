import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

const PROGRESS_SCHEMA = 'parity-progress-v1';

/** Reports the ledger's latest verified/active totals; virtual runs never promote a row. */
export function parityFromLedger(text) {
  const lines = text.split('\n').filter(line => line.trim() !== '');
  if (lines.length === 0) throw Error('parity_ledger_empty');
  const latest = JSON.parse(lines[lines.length - 1]);
  if (latest.schema_version !== PROGRESS_SCHEMA
      || !Number.isInteger(latest.verified_total) || !Number.isInteger(latest.active_total)) throw Error('parity_ledger_invalid');
  return `${latest.verified_total}/${latest.active_total}`;
}

export async function ledgerParity(repo) {
  return parityFromLedger(await readFile(join(repo, 'docs/parity/progress.jsonl'), 'utf8'));
}

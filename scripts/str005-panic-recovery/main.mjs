import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { main as collect } from '../str005-share-recovery/main.mjs';
import { PANIC_RECOVERY } from './profile.mjs';

export const main = argv => collect(argv, PANIC_RECOVERY);
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: /^share_recovery_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'share_recovery_failed' })}\n`); process.exitCode = 1; });

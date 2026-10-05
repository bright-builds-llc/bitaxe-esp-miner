import { writeFile } from 'node:fs/promises';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { git } from '../fixed-usb-qualification/contract.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { HEAD, HEAD_PATH, advanceInstall, recordStart, verifyHead } from './head.mjs';

/** `show` verifies the committed head; `advance-install` and `record-start` rewrite it from a sealed root. */
export async function main(argv) {
  const [action, flag, root] = argv;
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  if (action === 'show') { check(argv.length === 1, 'lineage_arguments'); return verifyHead(firmwareRoot); }
  check(['advance-install', 'record-start'].includes(action) && flag === '--root' && argv.length === 3 &&
    typeof root === 'string' && resolve(root) === root, 'lineage_arguments');
  const path = relative(firmwareRoot, root);
  await verifyHead(firmwareRoot);
  const next = action === 'advance-install' ? await advanceInstall(firmwareRoot, path) : await recordStart(firmwareRoot, HEAD, path);
  await writeFile(resolve(firmwareRoot, HEAD_PATH), `${JSON.stringify(next, null, 2)}\n`);
  return next;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: /^lineage_[a-z_]+$/u.test(error.code ?? error.message ?? '') ? (error.code ?? error.message) : 'lineage_failed' })}\n`); process.exitCode = 1; });

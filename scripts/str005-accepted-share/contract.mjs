import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-v2-accepted-share-probe';
export const ENABLED_LINE = 'Accepted share probe hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-accepted-share-amendment.md';
export const SCHEMA = 'str005-accepted-share-context-v1';
export const ADMISSION = 'accepted-share-v1';
/** The audited install, the last sealed Start and the restart that cleared its retained record. */
export const PINS = Object.freeze({ installationProfile: 'step5-diagnostic-reinstall',
  installationResult: '04f2f8d1689f8059eb59a8522c7b582b87dbb22375436f6fd4294d4b654afa46',
  installationSeal: '0751d602e4874a1224923d8e5ef2b86e492fbfc9f7b382140965cb7519a81092',
  previousStartResult: '4ea90e4f71ad877e9c480f1edc30b00248f0d54910d178c85f755675030208ea',
  previousStartSeal: 'd37808f87818e418449fee720c8618d31ea79a2a39ebc90d8002d4699238623c',
  // Restart002: one no-mining restart after start004 (boot 12 -> 13, ledger unchanged).
  restartResult: '98bdfdf2e4cc10b718d9bed093626bdd8e7cc54ebbfedafe136afdc6e010a899',
  restartSeal: '7f602f4f01ce607f99bd00c9a46155234716851b11cbfb6b7ee374bb65e59af2',
  gate: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' });
/** The exact active task line and the compiled flag both admit effects; a restart must be pinned. */
export function taskEnabled(tasks, compiled = ENABLED, pins = PINS) {
  const active = tasks.split(/^## Active$/mu)[1]?.split(/^## /mu)[0] ?? '';
  const blocks = active.split(/^### /mu).filter(block => block.startsWith(`${TASK} |`));
  check(compiled && pins.restartResult !== null && blocks.length === 1 &&
    blocks[0].split(/\r?\n/u).some(line => line.trim() === ENABLED_LINE), 'share_disabled');
}
export async function source(repo) {
  const commit = git(repo, ['rev-parse', 'HEAD']); cleanPushed(repo, commit);
  taskEnabled(await readFile(resolve(repo, 'TASKS.md'), 'utf8'));
  return { commit, contractSha256: sha256(await readFile(resolve(repo, CONTRACT))) };
}
const OPTIONS = ['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--previous-start-root', '--restart-root',
  '--authority-directory'];
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'share_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(OPTIONS.includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'share_arguments');
    options[key] = value;
  }
  const required = action === 'preflight'
    ? ['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--previous-start-root', '--restart-root']
    : action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key)), 'share_arguments');
  return { action, options };
}

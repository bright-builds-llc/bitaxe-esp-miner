import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HEAD } from '../str005-lineage/head.mjs';
export const TASK = 'task-str005-renewal-current-image';
export const ENABLED_LINE = 'Renewal current image hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-accepted-share-amendment.md';
export const SCHEMA = 'str005-accepted-share-context-v1';
export const ADMISSION = 'accepted-share-v1';
/** Renewals the Gate must confirm before the probe may Stop; 0 is the plain accepted-share probe. */
export const MINIMUM_RENEWALS = 1;
/** The verified lineage head's install, its latest sealed Start and the restart that cleared that Start's retained record. */
export const PINS = Object.freeze({ installationProfile: HEAD.install.profile,
  installationResult: HEAD.install.result,
  installationSeal: HEAD.install.seal,
  // Share-current-001: the lineage head's latest Start on 2bff1004 (boot 319, ledger 31/30/3,900,000 ms).
  previousStartResult: '0227a65e35eb7e11778d39ec44dc2520d9e4e23f5772a45db77d498eae29fe00',
  previousStartSeal: '2d0d31b9976acdd7d1374d4ad3b389fca140d3189d1b66eefa13144836869116',
  // Restart008: one no-mining restart after share-current-001 (boot 319 -> 320, ledger unchanged).
  restartResult: '003683ef76032c87ba2ca632e9988dec4dd6d6f824c03341e364d70f4b5d9195',
  restartSeal: '764cbaa7026f253cfad5453c58586e5125098b953c22a51ac21ff1b47ed99f70',
  gate: HEAD.install.identity.gate_commit });
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

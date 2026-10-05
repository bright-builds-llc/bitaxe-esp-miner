import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HEAD } from '../str005-lineage/head.mjs';
export const TASK = 'task-str005-share-current-image';
export const ENABLED_LINE = 'Share current image hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-accepted-share-amendment.md';
export const SCHEMA = 'str005-accepted-share-context-v1';
export const ADMISSION = 'accepted-share-v1';
/** The verified lineage head's install, its latest sealed Start and the restart that cleared that Start's retained record. */
export const PINS = Object.freeze({ installationProfile: HEAD.install.profile,
  installationResult: HEAD.install.result,
  installationSeal: HEAD.install.seal,
  // Heartbeat008: the lineage head's latest Start on 60e344e2 (boot 301, ledger 28/27/3,360,000 ms).
  previousStartResult: 'fcbb3ce7767778183584a36dc23a1efcd31874b11ded3e518d562b015a0cbdae',
  previousStartSeal: '68621dc4406ec506e4b067b0c1cdd5447439e206db331b50c7ce4cc215005b92',
  // Pinned once its sealed no-mining restart after heartbeat008 passes.
  restartResult: null,
  restartSeal: null,
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

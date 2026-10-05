import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HEAD } from '../str005-lineage/head.mjs';
export const TASK = 'task-str005-heartbeat-current-image';
export const ENABLED_LINE = 'Heartbeat current image hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-heartbeat-shutdown-amendment.md';
export const SCHEMA = 'str005-heartbeat-shutdown-context-v1';
export const ADMISSION = 'heartbeat-shutdown-v1';
/** The verified lineage head's install, plus a current recovery that re-bases the boot after the install's
 * later reboots. A previous Start and its restart are pinned only when a Start ran on this install. */
export const PINS = Object.freeze({ installationProfile: HEAD.install.profile,
  installationResult: HEAD.install.result,
  installationSeal: HEAD.install.seal,
  previousStartResult: null,
  previousStartSeal: null,
  restartResult: null,
  restartSeal: null,
  // Recovery017: current safe recovery on 60e344e2 after the endurance loops (boot 301, ledger 27/26/3,180,000 ms).
  currentRecoveryResult: '473546a8714067aa3c2af3c1772ce48050cf1c2aca8dc584886b6393a9b8f11b',
  currentRecoverySeal: '7aacb820d2ccfcb939e32f25cbafe8a6bb473ac8723debe3bc62af1e23680efc',
  gate: HEAD.install.identity.gate_commit });
/** The exact active task line and the compiled flag both admit effects; a pinned Start needs a pinned restart. */
export function taskEnabled(tasks, compiled = ENABLED, pins = PINS) {
  const active = tasks.split(/^## Active$/mu)[1]?.split(/^## /mu)[0] ?? '';
  const blocks = active.split(/^### /mu).filter(block => block.startsWith(`${TASK} |`));
  check(compiled && (pins.previousStartResult === null || pins.restartResult !== null) && blocks.length === 1 &&
    blocks[0].split(/\r?\n/u).some(line => line.trim() === ENABLED_LINE), 'heartbeat_disabled');
}
export async function source(repo) {
  const commit = git(repo, ['rev-parse', 'HEAD']); cleanPushed(repo, commit);
  taskEnabled(await readFile(resolve(repo, 'TASKS.md'), 'utf8'));
  return { commit, contractSha256: sha256(await readFile(resolve(repo, CONTRACT))) };
}
const OPTIONS = ['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--previous-start-root', '--restart-root',
  '--current-recovery-root', '--authority-directory'];
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'heartbeat_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(OPTIONS.includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'heartbeat_arguments');
    options[key] = value;
  }
  // The shared preflight requires the Start, restart and recovery roots exactly when they are pinned.
  const [required, optional] = action === 'preflight'
    ? [['--private-root', '--gate-root', '--fixture-binary', '--installation-root'],
      ['--previous-start-root', '--restart-root', '--current-recovery-root']]
    : [action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'], []];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key) || optional.includes(key)),
    'heartbeat_arguments');
  return { action, options };
}

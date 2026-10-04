import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-heartbeat-shutdown-probe';
export const ENABLED_LINE = 'Heartbeat shutdown probe hardware: enabled.';
export const ENABLED = true;
export const CONTRACT = 'docs/hardware/str005-heartbeat-shutdown-amendment.md';
export const SCHEMA = 'str005-heartbeat-shutdown-context-v1';
export const ADMISSION = 'heartbeat-shutdown-v1';
/** The audited install and, when a Start ran on it, that Start and the restart clearing its retained record. */
export const PINS = Object.freeze({ installationProfile: 'realignment-fix-install',
  // Realignment-fix attempt-001: the corrected image 7ca3e29c (ledger 26/25/3,000,000 ms, boot 5).
  installationResult: '13296cfb30ec0721b290141deb658d94fa7eb724c8d7a3a39f3acff478e79293',
  installationSeal: 'd3ec8ccb73cde4b655120781c45ba5a8ba6b463cdd151d3ff14600006d9551f0',
  // No Start has run on this image, so no retained record needs a restart first.
  previousStartResult: null,
  previousStartSeal: null,
  restartResult: null,
  restartSeal: null,
  gate: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' });
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
  '--authority-directory'];
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'heartbeat_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(OPTIONS.includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'heartbeat_arguments');
    options[key] = value;
  }
  // The shared preflight requires the Start and restart roots exactly when they are pinned.
  const [required, optional] = action === 'preflight'
    ? [['--private-root', '--gate-root', '--fixture-binary', '--installation-root'], ['--previous-start-root', '--restart-root']]
    : [action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'], []];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key) || optional.includes(key)),
    'heartbeat_arguments');
  return { action, options };
}

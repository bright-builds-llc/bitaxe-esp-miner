import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HEAD } from '../str005-lineage/head.mjs';
export const TASK = 'task-str005-renewal-window-current-image';
export const ENABLED_LINE = 'Renewal window hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-accepted-share-amendment.md';
export const SCHEMA = 'str005-accepted-share-context-v1';
export const ADMISSION = 'accepted-share-v1';
/** Renewals the Gate must confirm before the probe may Stop; 0 is the plain accepted-share probe. */
export const MINIMUM_RENEWALS = 1;
/** Longest Stop request after the Start reply. 80 s stays inside the lease that two renewals extend to about
 * 100 s after the reply, and makes a run without a qualifying share about a 1% outcome (45 s left about 6%). */
export const OBSERVE_WINDOW_MS = 80000;
/** The verified lineage head's install, its latest sealed Start and the restart that cleared that Start's retained record. */
export const PINS = Object.freeze({ installationProfile: HEAD.install.profile,
  installationResult: HEAD.install.result,
  installationSeal: HEAD.install.seal,
  // Renew-current-001: the lineage head's latest Start on 2bff1004 (boot 320, ledger 32/31/4,080,000 ms).
  previousStartResult: '81ae7513e93a85c42c753a725356c0635176958203cdc63761a0fc26fc0fec89',
  previousStartSeal: '998da7e265215bc5004b2d46f4824ca69936e70a780dbedd91c1986c96f8f91d',
  // Pinned once its sealed no-mining restart after renew-current-001 passes.
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

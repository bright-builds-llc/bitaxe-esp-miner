import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-heartbeat-shutdown-probe';
export const ENABLED_LINE = 'Heartbeat shutdown probe hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-heartbeat-shutdown-amendment.md';
export const SCHEMA = 'str005-heartbeat-shutdown-context-v1';
export const ADMISSION = 'heartbeat-shutdown-v1';
/** The audited install, the last sealed Start and the restart that cleared its retained record. */
export const PINS = Object.freeze({ installationProfile: 'step5-diagnostic-reinstall',
  installationResult: '04f2f8d1689f8059eb59a8522c7b582b87dbb22375436f6fd4294d4b654afa46',
  installationSeal: '0751d602e4874a1224923d8e5ef2b86e492fbfc9f7b382140965cb7519a81092',
  previousStartResult: '9e17a8b28e0146f41c9d8d1079b42b86bb1b26c4236b70f157df85cb85d5a402',
  previousStartSeal: 'abde26a6e92e3a8e0d1edfa18558c68bc24e880a027644cb51f98bfef39aedff',
  // Restart003: one no-mining restart after share001 (boot 13 -> 14, ledger unchanged).
  restartResult: 'c8893b66f34c25aeb0d678209df4c864d5014608cdbd4ebfa74d14426202e9ec',
  restartSeal: '67d50e3a57b56df061b321aedde3312ca4cb9bdda8a3c973f31df03b8169965a',
  gate: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' });
/** The exact active task line and the compiled flag both admit effects; a restart must be pinned. */
export function taskEnabled(tasks, compiled = ENABLED, pins = PINS) {
  const active = tasks.split(/^## Active$/mu)[1]?.split(/^## /mu)[0] ?? '';
  const blocks = active.split(/^### /mu).filter(block => block.startsWith(`${TASK} |`));
  check(compiled && pins.restartResult !== null && blocks.length === 1 &&
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
  const required = action === 'preflight'
    ? ['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--previous-start-root', '--restart-root']
    : action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key)), 'heartbeat_arguments');
  return { action, options };
}

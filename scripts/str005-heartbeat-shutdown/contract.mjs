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
/** The audited install, the last sealed Start and the restart that cleared its retained record. */
export const PINS = Object.freeze({ installationProfile: 'step5-diagnostic-reinstall',
  installationResult: '04f2f8d1689f8059eb59a8522c7b582b87dbb22375436f6fd4294d4b654afa46',
  installationSeal: '0751d602e4874a1224923d8e5ef2b86e492fbfc9f7b382140965cb7519a81092',
  // Heartbeat002: the last sealed heartbeat-loss Start (ledger 26/25/3,000,000 ms, boot 14).
  previousStartResult: '6ffa84ea1113c5bd57c711f1c3fe4192feaf79bc7ad4a71ec87a82ca90647936',
  previousStartSeal: 'aad964919c38207cd5890011fad2599c9063a5b8f0c551f6523e53c9e361fd83',
  // Restart004: one no-mining restart after heartbeat002 (boot 14 -> 15, ledger unchanged).
  restartResult: '9b393edb80167eff732bdbc6c9158c319e9afc3beb99d0acc2f038f683ddf010',
  restartSeal: '05905a53b31841cadecfd2c1217576c49648cba7b53db65d729f77c1de594f3e',
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

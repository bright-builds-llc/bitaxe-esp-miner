import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-step5-revocation-detail';
export const ENABLED_LINE = 'Step-5 diagnostic Start hardware: enabled.';
export const ENABLED = false;
export const CONTRACT = 'docs/hardware/str005-step5-diagnostic-amendment.md';
/** Phase-1 install seal; set only after that attempt passes, finalizes and is reviewed. */
// Phase-1 attempt-003's firmware failed the native panic cutoff audit; the corrected
// firmware is pinned here only after its phase-1b reinstall passes.
export const PINS = Object.freeze({ installationProfile: 'step5-diagnostic-reinstall',
  installationResult: '04f2f8d1689f8059eb59a8522c7b582b87dbb22375436f6fd4294d4b654afa46',
  installationSeal: '0751d602e4874a1224923d8e5ef2b86e492fbfc9f7b382140965cb7519a81092',
  gate: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' });
/** The exact task line under `## Active` and the compiled flag both admit effects. */
export function taskEnabled(tasks, compiled = ENABLED) {
  const active = tasks.split(/^## Active$/mu)[1]?.split(/^## /mu)[0] ?? '';
  const blocks = active.split(/^### /mu).filter(block => block.startsWith(`${TASK} |`));
  check(compiled && blocks.length === 1 && blocks[0].split(/\r?\n/u).some(line => line.trim() === ENABLED_LINE), 'step5_disabled');
}
export async function source(repo) {
  const commit = git(repo, ['rev-parse', 'HEAD']); cleanPushed(repo, commit);
  taskEnabled(await readFile(resolve(repo, 'TASKS.md'), 'utf8'));
  return { commit, contractSha256: sha256(await readFile(resolve(repo, CONTRACT))) };
}
const OPTIONS = ['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--authority-directory'];
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'step5_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(OPTIONS.includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'step5_arguments');
    options[key] = value;
  }
  const required = action === 'preflight' ? ['--private-root', '--gate-root', '--fixture-binary', '--installation-root'] :
    action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key)), 'step5_arguments');
  return { action, options };
}

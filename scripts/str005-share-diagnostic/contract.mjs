import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const FLAGS = Object.freeze({ recovery: true, clear: false, installation: false, capture: false, 'archive-clear': true });
export const TASK = 'task-str005-v2-accepted-share-probe';
export const CONTRACT = 'scripts/str005-share-diagnostic/CONTRACT.md';
export function sourceScope(tasks, stage, flags = FLAGS) {
  const blocks = tasks.split(`### ${TASK} |`);
  check(Object.hasOwn(flags, stage) && flags[stage] && blocks.length === 2 &&
    (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${TASK} |`), 'diagnostic_stage_disabled');
  const lines = blocks[1].split(/^### /mu)[0].split(/\r?\n/u).map(line => line.trim());
  check(lines.includes(`Share diagnostic ${stage} hardware: enabled.`), 'diagnostic_stage_disabled');
  return { stage, installEnabled: stage === 'installation', selfTestEnabled: stage === 'capture',
    recoveryOnly: stage === 'recovery', captureExisting: stage === 'capture' };
}
export async function currentSource(repo, stage) {
  const commit = git(repo, ['rev-parse', 'HEAD']); cleanPushed(repo, commit);
  const tasks = await readFile(resolve(repo, 'TASKS.md'), 'utf8');
  return { commit, ...sourceScope(tasks, stage), contractSha256: sha256(await readFile(resolve(repo, CONTRACT))) };
}
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'install', 'clear', 'finish'].includes(action) && rest.length % 2 === 0, 'diagnostic_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(['--private-root', '--stage', '--gate-root', '--manifest', '--clear-root', '--recovery-root', '--installation-root', '--retained-manifest', '--capture-root'].includes(key) && !options[key] &&
      typeof value === 'string' && (key === '--stage' ? Object.hasOwn(FLAGS, value) : resolve(value) === value), 'diagnostic_arguments');
    options[key] = value;
  }
  check(options['--private-root'] && (action === 'preflight' ? options['--stage'] : Object.keys(options).length === 1), 'diagnostic_arguments');
  if (action === 'preflight') {
    const stage = options['--stage'];
    const required = ['--private-root', '--stage', '--gate-root', ...(stage === 'clear' ? ['--recovery-root'] : stage === 'archive-clear' ? ['--recovery-root', '--capture-root'] : stage === 'installation' ? ['--manifest', '--clear-root'] : stage === 'capture' ? ['--installation-root', '--recovery-root', '--retained-manifest'] : [])];
    const allowed = [...required, ...(stage === 'recovery' ? ['--installation-root'] : [])];
    check(required.every(key => options[key]) && Object.keys(options).every(key => allowed.includes(key)), 'diagnostic_stage_arguments');
  }
  return { action, options };
}

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-v2-accepted-share-probe';
export const ENABLED = true;
export const CONTRACT = 'scripts/str005-status-repro/CONTRACT.md';
export const PINS = Object.freeze({
  installation: '81a48efde23fa55b52cfabfb5f0a521fbb54ee00a7c4fd967b03087b0856f3d7',
  capture: '8806886244f104ad611f0717932b1f4cbf6dfc56840cb0edd464924fca804a16',
  clear: '30ed731be9a6e698ae8d56d13aaf5e0c47354d60e3f766261d22467ef9948d82',
  recovery: 'ab3dc48ef94d0fc9c8c0137728950d21880131ac769afffe867e36a6d0ec2d0c',
  firmware: 'ce8f015811b93385c5aab0bac7bcdf6307452b31',
  elf: '453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31',
  gate: '9643e87664397a321c715a3a1b1bb6c1183b83ea',
});
export function taskEnabled(tasks, compiled = ENABLED) {
  const blocks = tasks.split(`### ${TASK} |`);
  check(compiled && blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${TASK} |`) &&
    blocks[1].split(/^### /mu)[0].split(/\r?\n/u).some(line => line.trim() === 'Status reproduction hardware: enabled.'),
  'status_repro_disabled');
}
export async function source(repo) {
  const commit = git(repo, ['rev-parse', 'HEAD']); cleanPushed(repo, commit);
  taskEnabled(await readFile(resolve(repo, 'TASKS.md'), 'utf8'));
  return { commit, contractSha256: sha256(await readFile(resolve(repo, CONTRACT))) };
}
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'status_repro_arguments');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(['--private-root', '--gate-root', '--fixture-binary', '--installation-root', '--capture-root', '--clear-root', '--recovery-root', '--authority-directory'].includes(key) &&
      !options[key] && typeof value === 'string' && resolve(value) === value, 'status_repro_arguments');
    options[key] = value;
  }
  const required = action === 'preflight' ? ['--private-root','--gate-root','--fixture-binary','--installation-root','--capture-root','--clear-root','--recovery-root'] :
    action === 'serve' ? ['--private-root','--authority-directory'] : ['--private-root'];
  check(required.every(key => options[key]) && Object.keys(options).every(key => required.includes(key)), 'status_repro_arguments');
  return { action, options };
}

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { cleanPushed, git } from '../fixed-usb-qualification/contract.mjs';
import { proof, privateRoot, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateLedger, validateAttempt } from '../fixed-usb-qualification/iterative-contract.mjs';
export const HARDWARE_ENABLED = false;
export const TASK = 'task-str005-v2-accepted-share-probe';
export const CONTRACT = 'scripts/str005-share-recovery/CONTRACT.md';
export const SHARE_SEAL = '1125f5d0dea1fa310ad3001cfa3c2aaa775924aacd1240a68555021ce559261e';
/** The sealed predecessor, task gate and listener a current-recovery collection is bound to. */
export const SHARE001 = Object.freeze({
  enabled: HARDWARE_ENABLED, task: TASK, contract: CONTRACT, seal: SHARE_SEAL, rootOption: '--share-root', port: 0, failedBootOrdinal: 15,
  lines: ['Share failure recovery hardware: enabled.', `Share failure recovery seal: ${SHARE_SEAL}.`],
  contextSchema: 'str005-share-context-v1', admitResult: result => result.complete === false && result.accepted_share_verified === false,
  identity: { firmware_commit: 'f000872f2e436aa7cdaa8cbfa41eee965a27731e',
    app_elf_sha256: 'a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2', gate_commit: '9643e87664397a321c715a3a1b1bb6c1183b83ea' },
});
export function requireEnabled(tasks, enabled = HARDWARE_ENABLED, profile = SHARE001) {
  const blocks = tasks.split(`### ${profile.task} |`), lines = blocks[1]?.split(/^### /mu)[0].split(/\r?\n/u) ?? [];
  check(enabled && blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${profile.task} |`) &&
    profile.lines.every(line => lines.includes(line)), 'share_recovery_disabled');
}
export function argumentsFor(argv, enabled = HARDWARE_ENABLED, profile = SHARE001) {
  const [action, ...args] = argv;
  check(['preflight', 'serve', 'finish'].includes(action) && args.length % 2 === 0, 'share_recovery_arguments');
  if (action !== 'finish') check(enabled, 'share_recovery_disabled');
  const options = {}, allowed = action === 'preflight' ? ['--private-root', profile.rootOption, '--gate-root'] : ['--private-root'];
  for (let i = 0; i < args.length; i += 2) {
    check(allowed.includes(args[i]) && !options[args[i]] && typeof args[i + 1] === 'string' && resolve(args[i + 1]) === args[i + 1], 'share_recovery_arguments');
    options[args[i]] = args[i + 1];
  }
  check(allowed.every(key => options[key]), 'share_recovery_arguments'); return { action, options };
}
export async function currentSource(root, effect = true, profile = SHARE001) {
  const source_commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source_commit);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8');
  if (effect) requireEnabled(tasks, profile.enabled, profile);
  return { source_commit, contractSha256: sha256(await readFile(resolve(root, profile.contract))) };
}
export async function predecessor(root, oldRoot, profile = SHARE001) { await privateRoot(oldRoot);
  const seal = await proof(oldRoot, 'sealed-inventory.json'); check(seal.sha256 === profile.seal, 'share_recovery_predecessor_seal');
  await verifyInventory(oldRoot, seal.value.files, new Set(['sealed-inventory.json']));
  const context = (await proof(oldRoot, 'context.json')).value, before = (await proof(oldRoot, 'before.json')).value;
  const result = (await proof(oldRoot, 'result.json')).value;
  check(context.schema === profile.contextSchema && profile.admitResult(result) &&
    Object.entries(profile.identity).every(([key, value]) => context[key] === value), 'share_recovery_predecessor_identity');
  validateLedger(before.ledger); validateAttempt(before.attempt);
  return { context, before, seal: seal.sha256 };
}

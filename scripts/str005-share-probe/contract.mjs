import { resolve } from 'node:path';
import { check } from '../str005-v2-serial/values.mjs';
export const HARDWARE_ENABLED = true;
export const TASK = 'task-str005-v2-accepted-share-probe';
export const CONTRACT = 'scripts/str005-share-probe/CONTRACT.md';
export const STARTUP_SEAL = 'cc5bab7bd34da6f3b50388c31f5e1fc4aefb6a7faf12bd14678a6dc08593e7a1';
export function argumentsFor(argv, enabled = HARDWARE_ENABLED) {
  const [action, ...args] = argv;
  check(['preflight', 'serve', 'finish'].includes(action) && args.length % 2 === 0, 'share_arguments');
  if (action !== 'finish') check(enabled, 'share_hardware_disabled');
  const allowed = action === 'preflight' ? ['--private-root', '--admission', '--gate-root', '--fixture-binary'] :
    action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'];
  const options = {};
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index], value = args[index + 1];
    check(allowed.includes(key) && !Object.hasOwn(options, key) && value && resolve(value) === value, 'share_arguments'); options[key] = value;
  }
  check(allowed.every(key => options[key]), 'share_arguments'); return { action, options };
}
export function requireEnabled(tasks, enabled = HARDWARE_ENABLED) {
  check(enabled, 'share_hardware_disabled');
  const blocks = tasks.split(`### ${TASK} |`); check(blocks.length === 2, 'share_task_missing');
  const block = blocks[1].split(/^### /mu)[0];
  const active = tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '';
  check(active.includes(`### ${TASK} |`) && block.split(/\r?\n/u).some(line => line.trim() === 'Share probe hardware: enabled.'), 'share_task_disabled');
  const pins = [...block.matchAll(/^Share admission sha256: ([a-f0-9]{64})\.$/gmu)];
  check(pins.length === 1, 'share_admission_pin'); return pins[0][1];
}

export function requireGatePin(moduleText, gateCommit) {
  const pins = [...moduleText.matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === gateCommit, 'share_gate_pin');
}

export function requireFrozenSource(current, context) {
  check(['source_commit', 'admissionSha256', 'contractSha256'].every(key => current[key] === context[key]), 'share_source_changed');
}

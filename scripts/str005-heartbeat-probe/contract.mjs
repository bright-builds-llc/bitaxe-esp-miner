import { check } from '../str005-v2-serial/values.mjs';
export const HARDWARE_ENABLED = false;
export const TASK = 'task-str005-heartbeat-shutdown-probe';
export const STARTUP_SEAL = 'cc5bab7bd34da6f3b50388c31f5e1fc4aefb6a7faf12bd14678a6dc08593e7a1';
export { LIMITS } from './limits.mjs';
/** No argument, environment override or historical command can activate hardware. */
export function argumentsFor(argv) {
  const [action, ...args] = argv;
  check(action === 'inspect' && args.length === 2 && args[0] === '--private-root' && typeof args[1] === 'string' && args[1].startsWith('/'),
    'heartbeat_hardware_disabled');
  return { action, root: args[1] };
}
export const CONTRACT = 'scripts/str005-heartbeat-probe/README.md';
export function effectArguments(argv, enabled = HARDWARE_ENABLED) {
  const [action, ...args] = argv;
  check(['preflight', 'serve', 'finish'].includes(action) && args.length % 2 === 0, 'heartbeat_arguments');
  if (action !== 'finish') check(enabled, 'heartbeat_hardware_disabled');
  const allowed = action === 'preflight' ? ['--private-root', '--preparation-root', '--gate-root', '--fixture-binary'] :
    action === 'serve' ? ['--private-root', '--authority-directory'] : ['--private-root'];
  const options = {};
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index], value = args[index + 1];
    check(allowed.includes(key) && !options[key] && typeof value === 'string' && value.startsWith('/'), 'heartbeat_arguments'); options[key] = value;
  }
  check(allowed.every(key => options[key]), 'heartbeat_arguments'); return { action, options };
}
export function requireEnabled(tasks, enabled = HARDWARE_ENABLED) {
  check(enabled, 'heartbeat_hardware_disabled');
  const blocks = tasks.split(`### ${TASK} |`), active = tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '';
  check(blocks.length === 2 && active.includes(`### ${TASK} |`), 'heartbeat_task');
  const block = blocks[1].split(/^### /mu)[0];
  check(block.split(/\r?\n/u).includes('Heartbeat probe hardware: enabled.'), 'heartbeat_task_disabled');
  const pins = [...block.matchAll(/^Heartbeat preparation sha256: ([a-f0-9]{64})\.$/gmu)];
  check(pins.length === 1, 'heartbeat_preparation_pin'); return pins[0][1];
}

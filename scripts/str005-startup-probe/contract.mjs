import { resolve } from 'node:path';
import { check, object } from '../str005-v2-serial/values.mjs';
export const TASK = 'task-str005-mining-startup-probe';
export const CONTRACT = 'docs/hardware/str005-startup-probe.md';
// Activation requires reviewed, published source and actual capture/clear evidence.
export const HARDWARE_ENABLED = false;
export const LIMITS = Object.freeze({ replyMs: 30000, observeMs: 5000, stopRequestMs: 35000,
  readMs: 30000, stopMs: 150000, closeMs: 150000, fixtureCleanupMs: 5000, clearMs: 1200000 });
export function requireEnabled(tasks, hardwareEnabled = HARDWARE_ENABLED) {
  const blocks = tasks.split(`### ${TASK} |`);
  const active = tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '';
  check(hardwareEnabled && blocks.length === 2 && active.includes(`### ${TASK} |`) &&
    blocks[1].split(/^### /mu)[0].split(/\r?\n/u).includes('Startup probe hardware: enabled.'), 'startup_hardware_disabled');
}
export function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'clear', 'serve', 'finish'].includes(action) && rest.length % 2 === 0, 'startup_arguments');
  for (let index = 0; index < rest.length; index += 2) {
    const key = rest[index], value = rest[index + 1];
    check(['--private-root', '--bindings', '--gate-root', '--fixture-binary', '--authority-directory', '--flash-binary'].includes(key) &&
      !Object.hasOwn(options, key) && typeof value === 'string' && resolve(value) === value, 'startup_arguments');
    options[key] = value;
  }
  check(options['--private-root'], 'startup_private_root');
  if (action === 'preflight') check(options['--bindings'] && options['--gate-root'] && options['--fixture-binary'] && options['--flash-binary'], 'startup_preflight_arguments');
  else if (action === 'serve') check(options['--authority-directory'] && Object.keys(options).length === 2, 'startup_serve_arguments');
  else check(Object.keys(options).length === 1, 'startup_phase_arguments');
  return { action, options };
}
export function validateBindings(value) {
  object(value, ['schema', 'captureRoot', 'archiveRoot', 'archiveRelative', 'decoderRoot', 'recoveryRoot', 'recoveryRelative']);
  check(value.schema === 'str005-startup-proof-inputs-v1', 'startup_bindings_schema');
  for (const key of ['captureRoot', 'archiveRoot', 'decoderRoot', 'recoveryRoot']) check(typeof value[key] === 'string' && resolve(value[key]) === value[key], 'startup_binding_path');
  for (const key of ['archiveRelative', 'recoveryRelative']) check(typeof value[key] === 'string' && value[key].length > 0 &&
    !value[key].startsWith('/') && !value[key].split('/').some(part => ['', '.', '..'].includes(part)), 'startup_binding_relative');
  check(value.archiveRoot === value.captureRoot && value.recoveryRoot === value.captureRoot &&
    value.decoderRoot === resolve(value.captureRoot, 'cutoff-review') && value.archiveRelative === 'self-test-core/core-dump.private.bin' &&
    /^candidate-recovery-[0-9]{3}\/current-recovery\.json$/u.test(value.recoveryRelative), 'startup_capture_lineage');
  return value;
}

/** Pure early admission lets disabled behavior remain tested after activation. */
export function admitArguments(argv, hardwareEnabled = HARDWARE_ENABLED) {
  const parsed = argumentsFor(argv);
  if (parsed.action !== 'finish') check(hardwareEnabled, 'startup_hardware_disabled');
  return parsed;
}

import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, ENABLED, ENABLED_LINE, PINS, TASK, taskEnabled } from './contract.mjs';

const tasks = line => `## Active\n### ${TASK} | 2026-09-27 | synthetic\n\n${line}\n\n## Future\n`;
const installOnly = { ...PINS, previousStartResult: null, previousStartSeal: null, restartResult: null, restartSeal: null };
const startWithoutRestart = { ...installOnly, previousStartResult: 'a'.repeat(64), previousStartSeal: 'b'.repeat(64) };

test('heartbeat effects need the compiled flag and the exact active line', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => taskEnabled(tasks(ENABLED_LINE), true, installOnly));
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), false, installOnly), /heartbeat_disabled/u);
  assert.throws(() => taskEnabled(tasks('Heartbeat shutdown probe hardware: disabled.'), true, installOnly), /heartbeat_disabled/u);
});

test('a pinned previous Start needs a pinned restart', () => {
  // Arrange / Act / Assert
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), true, startWithoutRestart), /heartbeat_disabled/u);
});

test('the compiled pins hold a restart whenever they hold a previous Start', () => {
  // Arrange / Act / Assert
  assert.ok(PINS.previousStartResult === null || PINS.restartResult !== null);
});

test('preflight binds the install and accepts optional Start and restart roots; serve takes no pool input', () => {
  // Arrange
  const base = ['--private-root', '/p', '--gate-root', '/g', '--fixture-binary', '/f', '--installation-root', '/i'];
  // Act / Assert
  assert.equal(Object.keys(argumentsFor(['preflight', ...base]).options).length, 4);
  assert.equal(Object.keys(argumentsFor(['preflight', ...base, '--previous-start-root', '/s', '--restart-root', '/r']).options).length, 6);
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/p']), /heartbeat_arguments/u);
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p', '--authority-directory', '/a', '--pool-credentials', '/c']), /heartbeat_arguments/u);
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p', '--authority-directory', '/a', '--restart-root', '/r']), /heartbeat_arguments/u);
});

test('the heartbeat owner loads on the granted origin without a device', async () => {
  // Arrange / Act
  const loaded = await import('./main.mjs');
  // Assert
  assert.equal(loaded.PORT, 48765);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, ENABLED, ENABLED_LINE, PINS, TASK, taskEnabled } from './contract.mjs';

const tasks = line => `## Active\n### ${TASK} | 2026-09-27 | synthetic\n\n${line}\n\n## Future\n`;
const pinned = { ...PINS, restartResult: 'a'.repeat(64), restartSeal: 'b'.repeat(64) };
const unpinned = { ...PINS, restartResult: null, restartSeal: null };

test('heartbeat effects need the compiled flag, the exact active line and a pinned restart', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => taskEnabled(tasks(ENABLED_LINE), true, pinned));
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), true, unpinned), /heartbeat_disabled/u);
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), false, pinned), /heartbeat_disabled/u);
});

test('the compiled flag requires a pinned restart', () => {
  // Arrange / Act / Assert
  assert.ok(!ENABLED || PINS.restartResult !== null);
});

test('preflight binds the install, share001 and the restart; serve takes no pool input', () => {
  // Arrange
  const args = ['--private-root', '/p', '--gate-root', '/g', '--fixture-binary', '/f', '--installation-root', '/i',
    '--previous-start-root', '/s', '--restart-root', '/r'];
  // Act / Assert
  assert.equal(Object.keys(argumentsFor(['preflight', ...args]).options).length, 6);
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p', '--authority-directory', '/a', '--pool-credentials', '/c']), /heartbeat_arguments/u);
});

test('the heartbeat owner loads on the granted origin without a device', async () => {
  // Arrange / Act
  const loaded = await import('./main.mjs');
  // Assert
  assert.equal(loaded.PORT, 48765);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, ENABLED_LINE, PINS, TASK, taskEnabled } from './contract.mjs';

const tasks = line => `## Active\n### ${TASK} | 2026-09-27 | synthetic\n\n${line}\n\n## Future\n`;
const pinned = { ...PINS, restartResult: 'a'.repeat(64), restartSeal: 'b'.repeat(64) };

test('share effects need the compiled flag, the exact active line and a pinned restart', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => taskEnabled(tasks(ENABLED_LINE), true, pinned));
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), true, PINS), /share_disabled/u);
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), false, pinned), /share_disabled/u);
  assert.throws(() => taskEnabled(tasks('Accepted share probe hardware: disabled.'), true, pinned), /share_disabled/u);
});

test('preflight binds the install, the previous Start and the restart', () => {
  // Arrange
  const args = ['--private-root', '/p', '--gate-root', '/g', '--fixture-binary', '/f', '--installation-root', '/i', '--previous-start-root', '/s'];
  // Act / Assert
  assert.throws(() => argumentsFor(['preflight', ...args]), /share_arguments/u);
  assert.equal(Object.keys(argumentsFor(['preflight', ...args, '--restart-root', '/r']).options).length, 6);
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p', '--authority-directory', '/a', '--pool-credentials', '/c']), /share_arguments/u);
});

test('the share owner and its routes load without a device', async () => {
  // Arrange / Act
  const loaded = await import('./main.mjs');
  // Assert
  assert.equal(loaded.PORT, 48765);
});

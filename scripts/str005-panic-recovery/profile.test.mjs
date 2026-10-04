import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, requireEnabled } from '../str005-share-recovery/contract.mjs';
import { ENABLED, PANIC_RECOVERY } from './profile.mjs';

const tasks = line => `## Active\n### ${PANIC_RECOVERY.task} | fixture\n${line}\n`;

test('the panic recovery gate needs the compiled flag and its exact active task line', () => {
  // Arrange
  const enabled = tasks('Idle panic recovery hardware: enabled.');
  // Act / Assert
  assert.doesNotThrow(() => requireEnabled(enabled, true, PANIC_RECOVERY));
  assert.throws(() => requireEnabled(enabled, false, PANIC_RECOVERY), /share_recovery_disabled/u);
  assert.throws(() => requireEnabled(tasks('Idle panic recovery hardware: disabled.'), true, PANIC_RECOVERY), /share_recovery_disabled/u);
});

test('preflight names the predecessor by its generic option, never the Share001 one', () => {
  // Arrange
  const argv = ['preflight', '--private-root', '/p/a', '--predecessor-root', '/p/h', '--gate-root', '/g'];
  // Act
  const { options } = argumentsFor(argv, true, PANIC_RECOVERY);
  // Assert
  assert.equal(options['--predecessor-root'], '/p/h');
  assert.throws(() => argumentsFor(argv.map(value => value === '--predecessor-root' ? '--share-root' : value), true, PANIC_RECOVERY));
});

test('the profile serves on the granted origin and binds the installed image', () => {
  // Arrange / Act / Assert
  assert.equal(PANIC_RECOVERY.port, 48765);
  assert.equal(PANIC_RECOVERY.enabled, ENABLED);
  assert.equal(PANIC_RECOVERY.identity.firmware_commit, '654338d0101521490d90330c5a4a10e5ec32e5c2');
});

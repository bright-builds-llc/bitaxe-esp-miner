import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, requireEnabled } from '../str005-share-recovery/contract.mjs';
import { ENABLED, PANIC_RECOVERY } from './profile.mjs';
import { CONTROL_DIAGNOSTIC_RECOVERY, retainedAttempt } from './control-diagnostic.mjs';

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

test('after a new image, any boot is post-failure because the RTC ordinal restarts', () => {
  // Arrange / Act / Assert
  assert.equal(CONTROL_DIAGNOSTIC_RECOVERY.failedBootOrdinal, 0);
  assert.equal(PANIC_RECOVERY.failedBootOrdinal, 15);
});

const install = { firmware_commit: 'c'.repeat(40), app_elf_sha256: 'e'.repeat(64), physical: 'p'.repeat(64) };
const ledger = next => ({ schema: 'worker-qualification-ledger-v1', next_ordinal: next, last_completed_ordinal: next - 1,
  total_charged_ms: 3000000, pending: false });

test('without a later Start, status names the install lineage attempt', () => {
  // Arrange
  const fallback = { attemptId: 'install-attempt', ledger: ledger(26) };
  // Act
  const before = retainedAttempt(install, fallback, null);
  // Assert
  assert.equal(before.attempt.id, 'install-attempt');
});

test('a later sealed Start on the same install and board supplies the retained attempt', () => {
  // Arrange
  const latest = { context: { ...install, attemptId: 'heartbeat-attempt' }, ledger: ledger(27) };
  // Act
  const before = retainedAttempt(install, { attemptId: 'install-attempt', ledger: ledger(26) }, latest);
  // Assert
  assert.deepEqual([before.attempt.id, before.ledger.next_ordinal], ['heartbeat-attempt', 27]);
});

test('a later Start on another image or board is refused', () => {
  // Arrange
  const latest = { context: { ...install, physical: 'q'.repeat(64), attemptId: 'heartbeat-attempt' }, ledger: ledger(27) };
  // Act / Assert
  assert.throws(() => retainedAttempt(install, { attemptId: 'install-attempt', ledger: ledger(26) }, latest),
    /share_recovery_predecessor_identity/u);
});

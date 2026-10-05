import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, requireEnabled } from '../str005-share-recovery/contract.mjs';
import { ENABLED, PANIC_RECOVERY } from './profile.mjs';
import { CONTROL_DIAGNOSTIC_RECOVERY, retainedAttempt } from './control-diagnostic.mjs';
import { statusModeFor } from '../str005-share-recovery/main.mjs';

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
const fallback = { attemptId: 'install-attempt', ledger: ledger(26) };

test('without a later Start, nothing is retained and the install lineage attempt is named', () => {
  // Arrange / Act
  const before = retainedAttempt(install, fallback, null);
  // Assert
  assert.deepEqual(before.attempt, { id: 'install-attempt', recordRetained: false });
});

test('a later sealed Start on the same install and board names its device record attempt', () => {
  // Arrange
  const latest = { context: { ...install, attemptId: 'owner-nonce' }, ledger: ledger(27), deviceRecordAttemptId: 'device-attempt' };
  // Act
  const before = retainedAttempt(install, fallback, latest);
  // Assert
  assert.deepEqual([before.attempt, before.ledger.next_ordinal], [{ id: 'device-attempt', recordRetained: true }, 27]);
});

test('a later Start on another board is refused', () => {
  // Arrange
  const latest = { context: { ...install, physical: 'q'.repeat(64) }, ledger: ledger(27), deviceRecordAttemptId: 'device-attempt' };
  // Act / Assert
  assert.throws(() => retainedAttempt(install, fallback, latest), /share_recovery_predecessor_identity/u);
});

test('a retained record is read by its attempt; otherwise the current state is discovered', () => {
  // Arrange / Act / Assert
  assert.equal(statusModeFor({ id: 'device-attempt', recordRetained: true }), 'confirmed');
  assert.equal(statusModeFor({ id: 'install-attempt', recordRetained: false }), 'discover_current');
  assert.equal(statusModeFor({ id: 'share-attempt' }), 'discover_current');
});

test('the recovery owner reads a sealed Start\'s retained record through the production Gate decoder', async () => {
  // Arrange
  const { execFile } = await import('node:child_process'), { promisify } = await import('node:util');
  const { dirname, resolve } = await import('node:path'), { fileURLToPath } = await import('node:url');
  const here = dirname(fileURLToPath(import.meta.url));
  const gateRoot = process.argv[2] ? dirname(resolve(process.argv[2])) : resolve(here, '../../../bitaxe-turnstile-system');
  // Act
  const result = await promisify(execFile)('bun', [resolve(here, 'retained-start.fixture.mjs'), gateRoot], { timeout: 30000 });
  // Assert
  assert.equal(result.stdout.trim(), 'retained_start_recovery_passed'); assert.equal(result.stderr, '');
});

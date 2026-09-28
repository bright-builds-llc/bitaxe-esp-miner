import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor } from './preparation.mjs';
test('heartbeat restart preparation cannot execute without its own published activation', () => {
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/new', '--share-root', '/sealed', '--gate-root', '/gate']), /preparation_disabled/);
});
test('heartbeat restart preparation admits only a share predecessor, never flash or authority input', () => {
  const args = ['preflight', '--private-root', '/new', '--share-root', '/sealed', '--gate-root', '/gate'];
  assert.equal(argumentsFor(args, true).options['--share-root'], '/sealed');
  for (const flag of ['--authority-directory', '--flash-binary', '--startup-root', '--recovery-root'])
    assert.throws(() => argumentsFor([...args, flag, '/forbidden'], true));
});


import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { recoveryCollectionStarted } from './prepared.mjs';
import { validatePreparationTimeline } from '../str005-startup-probe/prepared-evidence.mjs';
async function collectionFixture(t, collectionTime, resultTime = collectionTime) {
  const root = await mkdtemp(resolve(tmpdir(), 'heartbeat-collection-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  for (const [name, value] of Object.entries({
    'serve-claim.json': { startedAtUnixMs: 1 },
    'collection-begin.json': { schema: 'str005-recovery-collection-v1', startedAtUnixMs: collectionTime },
    'result.json': { current_recovery_complete: true, startedAtUnixMs: resultTime },
  })) await writeFile(resolve(root, name), JSON.stringify(value), { mode: 0o600 });
  return root;
}
test('heartbeat preparation accepts fresh collection after an arbitrarily long server handoff', async t => {
  // Arrange
  const root = await collectionFixture(t, 86400000);
  // Act
  const started = await recoveryCollectionStarted(root);
  // Assert
  assert.equal(started, 86400000);
  assert.doesNotThrow(() => validatePreparationTimeline(started, 86410000, 86420000));
});
test('heartbeat preparation rejects stale actual collection regardless of server lifetime', async t => {
  const root = await collectionFixture(t, 86400000);
  const started = await recoveryCollectionStarted(root);
  assert.throws(() => validatePreparationTimeline(started, 86500000, 86520001), /startup_preparation_stale/);
});
test('heartbeat preparation rejects a result timestamp substituted for its collection trigger', async t => {
  const root = await collectionFixture(t, 86400000, 86410000);
  await assert.rejects(recoveryCollectionStarted(root), /heartbeat_preparation_collection/);
});

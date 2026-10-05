import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, ENABLED, PROFILE } from './main.mjs';
import { createBatches, gateConfiguration, validateRow } from './server.mjs';
import { LIMITS, PROOFS_PER_ROUND, SESSION_PROOFS } from './loop.mjs';

test('effects need the compiled flag; finish stays available', () => {
  // Arrange
  const serve = ['serve', '--private-root', '/p/a'];
  // Act / Assert
  assert.throws(() => argumentsFor(serve, false), /review_loop_disabled/u);
  assert.doesNotThrow(() => argumentsFor(serve, true));
  assert.doesNotThrow(() => argumentsFor(['finish', '--private-root', '/p/a'], false));
  assert.equal(PROFILE.enabled, ENABLED);
});

test('the loop owner has its own task line and never accepts authority or pool inputs', () => {
  // Arrange / Act / Assert
  assert.deepEqual(PROFILE.lines, ['Control review loop hardware: enabled.']);
  assert.throws(() => argumentsFor(['serve', '--private-root', '/p/a', '--authority-directory', '/secret'], true));
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/p/a', '--predecessor-root', '/p/i', '--gate-root', '/g', '--pool-credentials', '/x'], true));
});

test('rows carry only the batch, counts, closed categories and closed rejections', () => {
  // Arrange
  const failure = { batch: 2, kind: 'failure', completed: 4, iteration: 5, operation: 'status', category: 'command_rejected',
    rejection: 'invalid_transition' };
  // Act / Assert
  assert.deepEqual(validateRow(failure), failure);
  assert.deepEqual(validateRow({ batch: 1, kind: 'complete', completed: 100 }), { batch: 1, kind: 'complete', completed: 100 });
  assert.throws(() => validateRow({ ...failure, category: 'raw device text' }), /review_loop_row/u);
  assert.throws(() => validateRow({ ...failure, rejection: 'raw device text' }), /review_loop_row/u);
  assert.throws(() => validateRow({ ...failure, iteration: 9 }), /review_loop_row/u);
  assert.throws(() => validateRow({ ...failure, batch: 6 }), /review_loop_row/u);
  assert.throws(() => validateRow({ batch: 1, kind: 'complete', completed: 12 }), /review_loop_row/u);
  assert.throws(() => validateRow({ batch: 1, kind: 'progress', completed: 25, payload: 'x' }));
});

test('a session stays below the device nonce cap: admission plus two proofs per round', () => {
  // Arrange / Act
  const proofs = 1 + LIMITS.iterations * PROOFS_PER_ROUND;
  // Assert
  assert.ok(proofs < SESSION_PROOFS, `${proofs} proofs would reach the ${SESSION_PROOFS}-nonce cap`);
});

test('batches begin in order, one at a time, and stop after a failure or the last batch', () => {
  // Arrange
  const batches = createBatches(2);
  // Act / Assert
  assert.throws(() => batches.begin(2), /review_loop_batch/u);
  batches.begin(1);
  assert.throws(() => batches.begin(2), /review_loop_batch/u);
  batches.row({ batch: 1, kind: 'complete', completed: 100 });
  batches.begin(2);
  batches.row({ batch: 2, kind: 'complete', completed: 100 });
  assert.equal(batches.finished, true);
  assert.throws(() => batches.begin(3), /review_loop_batch/u);
  const failed = createBatches(3); failed.begin(1);
  failed.row({ batch: 1, kind: 'failure', completed: 3, iteration: 4, operation: 'ledger', category: 'timeout' });
  assert.equal(failed.finished, true);
  assert.throws(() => failed.begin(2), /review_loop_batch/u);
});

test('the Gate configuration starts before and moves to the share-scoped candidate phase', () => {
  // Arrange
  const context = { scope: 'share', gate_commit: 'g', firmware_commit: 'f', app_elf_sha256: 'e' };
  // Act
  const [before, candidate] = ['before', 'candidate'].map(phase => gateConfiguration(context, { keys: [] }, phase));
  // Assert
  assert.deepEqual([before.stratumV2Qualification, candidate.stratumV2Qualification], ['before', 'candidate']);
  assert.equal(candidate.stratumV2Scope, 'share');
  assert.throws(() => gateConfiguration({ ...context, scope: undefined }, {}, 'before'), /review_loop_scope/u);
  assert.throws(() => gateConfiguration(context, {}, 'restart'), /review_loop_scope/u);
});

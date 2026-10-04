import test from 'node:test';
import assert from 'node:assert/strict';
import { argumentsFor, ENABLED, PROFILE } from './main.mjs';
import { gateConfiguration, validateRow } from './server.mjs';

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

test('rows carry only counts and closed categories', () => {
  // Arrange
  const failure = { kind: 'failure', completed: 4, iteration: 5, operation: 'status', category: 'timeout' };
  // Act / Assert
  assert.deepEqual(validateRow(failure), failure);
  assert.deepEqual(validateRow({ kind: 'complete', completed: 300 }), { kind: 'complete', completed: 300 });
  assert.throws(() => validateRow({ ...failure, category: 'raw device text' }), /review_loop_row/u);
  assert.throws(() => validateRow({ ...failure, iteration: 9 }), /review_loop_row/u);
  assert.throws(() => validateRow({ kind: 'complete', completed: 12 }), /review_loop_row/u);
  assert.throws(() => validateRow({ kind: 'progress', completed: 25, payload: 'x' }));
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

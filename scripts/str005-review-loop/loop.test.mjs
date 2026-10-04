import test from 'node:test';
import assert from 'node:assert/strict';
import { failureCategory, runLoop } from './loop.mjs';

function gate(overrides = {}) {
  const calls = [];
  return { calls, value: {
    state: () => ({ serialFailureCategory: overrides.category ?? null }),
    reviewQualificationAttempts: async () => { calls.push('ledger'); },
    reviewBudget: async campaign => { calls.push(`budget:${campaign}`); if (overrides.failBudgetAt === calls.length) throw Error('serial'); },
    stratumV2Possession: async () => { calls.push('possession'); return 'binding'; },
    stratumV2Status: async (scope, attempt, binding) => { calls.push(`status:${scope}:${attempt}:${binding}`); },
  } };
}

test('a clean loop issues every review in order and reports completion', async () => {
  // Arrange
  const fake = gate(), rows = [];
  // Act
  const result = await runLoop({ gate: fake.value, campaignId: 'c', record: async row => rows.push(row), iterations: 2, progressEvery: 1 });
  // Assert
  assert.deepEqual(result, { completed: 2, failure: null });
  assert.deepEqual(fake.calls.slice(0, 4), ['ledger', 'budget:c', 'possession', 'status:share:null:binding']);
  assert.deepEqual(rows.map(row => row.kind), ['progress', 'progress', 'complete']);
});

test('the first failure stops the loop with its iteration, operation and Gate category', async () => {
  // Arrange: the budget review of the second iteration fails with a serial timeout.
  const fake = gate({ failBudgetAt: 6, category: 'timeout' }), rows = [];
  // Act
  const result = await runLoop({ gate: fake.value, campaignId: 'c', record: async row => rows.push(row), iterations: 5 });
  // Assert
  assert.deepEqual(result.failure, { iteration: 2, operation: 'original_budget', category: 'timeout' });
  assert.equal(result.completed, 1);
  assert.deepEqual(rows, [{ kind: 'failure', completed: 1, iteration: 2, operation: 'original_budget', category: 'timeout' }]);
});

test('an operation that never settles fails as a timeout', async () => {
  // Arrange
  const fake = gate(); fake.value.reviewQualificationAttempts = () => new Promise(() => {});
  // Act
  const result = await runLoop({ gate: fake.value, campaignId: 'c', record: async () => {}, iterations: 1, operationMs: 10 });
  // Assert
  assert.deepEqual(result.failure, { iteration: 1, operation: 'ledger', category: 'timeout' });
});

test('unknown Gate categories fold into operation_failed', () => {
  // Arrange / Act / Assert
  assert.equal(failureCategory(Error('x'), { serialFailureCategory: 'device payload' }), 'operation_failed');
  assert.equal(failureCategory(Error('x'), { serialFailureCategory: 'closed' }), 'closed');
});

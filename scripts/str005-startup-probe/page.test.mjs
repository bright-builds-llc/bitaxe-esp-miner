import test from 'node:test';
import assert from 'node:assert/strict';
import { createPage } from './page.mjs';

test('a probe before the baseline is refused without consuming the one-shot coordinator', async () => {
  // Arrange
  let invocations = 0;
  const posts = [];
  const page = createPage({ state: () => ({ connected: true }) }, async path => { posts.push(path); return {}; }, () => {},
    { coordinator: () => async () => { invocations++; return { complete: true }; } });
  // Act
  await assert.rejects(page.run(), /startup_candidate_required/u);
  // Assert
  assert.equal(invocations, 0);
  assert.deepEqual(posts, []);
});

test('a failed baseline operation is recorded by name and Gate serial category', async () => {
  // Arrange
  const posts = [];
  const gate = { state: () => ({ connected: true, serialFailureCategory: 'timeout' }), reviewQualificationAttempts: async () => ({}),
    reviewBudget: async () => { throw Error('serial timeout'); } };
  const page = createPage(gate, async (path, value) => { posts.push([path, value]); return { originalCampaignId: 'c', nonce: 'n' }; }, () => {});
  // Act
  await assert.rejects(page.baseline(), /serial timeout/u);
  // Assert
  assert.deepEqual(posts.at(-1), ['/startup/baseline-failure', { schema: 'str005-baseline-failure-v1', operation: 'original_budget', category: 'timeout' }]);
});

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

import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createReleaseOwner, fixtureReleaseComplete, fixtureCompletionComplete } from './server-release.mjs';
import { processSnapshot, requireGone, requirePoolListenerAbsent } from '../str005-v2-serial/host-resources.mjs';
async function listener(t, port = 0) {
  const child = spawn(process.execPath, [new URL('./server-release-child.mjs', import.meta.url).pathname, String(port)],
    { detached: true, stdio: ['pipe', 'pipe', 'pipe'] });
  const exited = once(child, 'exit'); let finished = false; exited.then(() => { finished = true; });
  t.after(async () => { if (!finished) child.kill('SIGTERM'); await exited; });
  const [chunk] = await once(child.stdout, 'data'); const selected = Number(chunk.toString().trim());
  assert.ok(Number.isInteger(selected) && selected > 0);
  const owner = (await processSnapshot()).find(row => row.pid === child.pid);
  assert.ok(owner);
  return { child, exited, owner, port: selected, async stop() { if (!finished) child.kill('SIGTERM'); await exited; } };
}
for (const [name, completionFails, releaseFails] of [
  ['success', false, false], ['natural completion fails but close and reap succeed', true, false],
  ['actual listener release failure', false, true], ['both completion and actual release fail', true, true],
]) test(`real process fixture release: ${name}`, async t => {
  // Arrange
  const process = await listener(t), receipts = new Map(); let finishes = 0, closes = 0;
  const completionError = Error('natural completion deadline');
  const fixture = { async close() {
    closes++; await process.stop();
    if (releaseFails) await listener(t, process.port);
  } };
  const release = createReleaseOwner({ getFixture: () => fixture,
    async finishFixture() {
      finishes++;
      if (completionFails) { assert.equal(process.child.exitCode, null); throw completionError; }
      process.child.stdin.write('finish\n'); const [code] = await process.exited; assert.equal(code, 0);
    }, async assertFixtureReleased() { await requireGone([process.owner]); requirePoolListenerAbsent(process.port); },
    persist: async (name, value) => { receipts.set(name, value); } });
  // Act
  const result = await release().then(() => null, error => error);
  const repeated = await release().then(() => null, error => error);
  // Assert
  assert.equal(repeated, result); assert.equal(finishes, 1); assert.equal(closes, 1);
  assert.equal(fixtureCompletionComplete(receipts.get('fixture-completion.json')), !completionFails);
  assert.equal(fixtureReleaseComplete(receipts.get('fixture-release.json')), !releaseFails);
  if (completionFails && releaseFails) { assert.ok(result instanceof AggregateError); assert.equal(result.errors[0], completionError); }
  else if (completionFails) assert.equal(result, completionError);
  else if (releaseFails) assert.match(result.message, /v2_pool_listener_present/);
  else assert.equal(result, null);
});
test('auxiliary cleanup and both receipts survive completion and persistence errors', async () => {
  const calls = [], failure = Error('completion');
  const release = createReleaseOwner({ getFixture: () => ({ close: async () => calls.push('close') }),
    finishFixture: async () => { throw failure; }, assertFixtureReleased: async () => calls.push('released'),
    releaseAuxiliary: async () => calls.push('auxiliary'), persist: async name => { calls.push(name); if (name === 'fixture-completion.json') throw Error('disk'); } });
  const error = await release().catch(value => value);
  assert.equal(error.errors[0], failure);
  assert.deepEqual(calls, ['close', 'released', 'auxiliary', 'fixture-completion.json', 'fixture-release.json']);
});
test('v1 false remains false and v2 cannot conceal incomplete release', () => {
  assert.equal(fixtureReleaseComplete({ schema: 'str005-startup-fixture-release-v1', complete: false }), false);
  assert.throws(() => fixtureReleaseComplete({ schema: 'str005-startup-fixture-release-v2', fixtureStarted: true,
    fixtureReleased: false, auxiliaryReleased: true, complete: true }), /startup_fixture_release_shape/);
  assert.throws(() => fixtureCompletionComplete({ schema: 'str005-startup-fixture-completion-v1', required: false, complete: true }), /startup_fixture_completion_shape/);
});

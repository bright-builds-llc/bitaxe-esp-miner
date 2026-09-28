import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHeartbeatOwner } from './owner.mjs';
import { recoveryFixture } from './recovery.fixture.mjs';
test('heartbeat extra routes bind actual passive owner before accepting suppression', async t => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'heartbeat-owner-')); t.after(() => rm(root, { recursive: true, force: true }));
  const f = recoveryFixture(); f.context.cadence_observer = { sha256: 'd'.repeat(64) };
  let connected = false, alive = false, time = 100000, finished = false;
  const owner = createHeartbeatOwner(root, f.context, { verify: async () => {}, now: () => time,
    operations: { createObserver: () => ({ status: () => ({ connected, alive, failed: false }),
      start: async () => { connected = alive = true; return { connected: true }; },
      finish: async () => { alive = false; finished = true; return { cleanupComplete: true, closed: true, reason: 'requested', exitCode: 0 }; } }),
    setInterval: () => ({ unref() {} }), clearInterval() {} } });
  const call = (path, input) => owner.extraRoute({ path, input, before: f.parts.before, ready: async () => {}, observeReady: async () => {} });
  // Act / Assert: an unarmed observer never permits a Start claim.
  await assert.rejects(call('/heartbeat/start-claim', {}), /v2_observer_not_live/);
  const challenge = await call('/observer/context', {});
  await call('/observer/start', { nonce: challenge.value.nonce, state: f.parts.before.state,
    endpoint: { schema: 'worker-telemetry-endpoint-v1', ipv4: '192.168.1.2', httpPort: 80, observedAtUs: 1000,
      bootOrdinal: 1, generation: 2, controlSessionBindingSha256: Buffer.alloc(32, 1).toString('base64url') } });
  await call('/heartbeat/start-claim', {});
  await assert.rejects(call('/heartbeat/start-claim', {}), /heartbeat_start_claim/);
  time++;
  await call('/heartbeat/suppressed', { clientConfirmedAtMs: 250, headroom: f.parts.run.headroom, state: f.parts.run.suppressedState });
  await assert.rejects(call('/heartbeat/suppressed', { clientConfirmedAtMs: 250, headroom: f.parts.run.headroom, state: f.parts.run.suppressedState }), /heartbeat_suppression_once/);
  time += 8000; await call('/heartbeat/tail', {}); await owner.release(); assert.equal(finished, true);
});
test('heartbeat observer cleanup remains reachable after a failed acquisition', async t => {
  const root = await mkdtemp(join(tmpdir(), 'heartbeat-owner-')); t.after(() => rm(root, { recursive: true, force: true }));
  let stopped = false;
  const owner = createHeartbeatOwner(root, { scope: 'share' }, { verify: async () => {}, operations: {
    createObserver: () => ({ status: () => ({ connected: false, alive: false, failed: true }), finish: async () => { stopped = true; return {}; } }),
  } });
  await owner.release(); assert.equal(stopped, true);
});

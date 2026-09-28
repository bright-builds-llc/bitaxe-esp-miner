import test from 'node:test';
import assert from 'node:assert/strict';
import { createRoutes } from './routes.mjs';
import { state as fixtureState, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
function fixture(maybeEffectError) {
  const context = { source_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), gate_commit: 'd'.repeat(40),
    expectedBootOrdinal: 5, expectedLedger: structuredClone(ledger), detector: { physical: 'e'.repeat(64) }, scope: 'share', attemptId: Buffer.alloc(16, 1).toString('base64url') };
  const state = fixtureState(context), closed = fixtureState(context, 'candidate', true), saved = new Map(), calls = [];
  let time = 1000;
  const status = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', record: null, connection: null,
    observation: { bootOrdinal: 5, workerGeneration: 7, serialTransportEpoch: 8, observedAtUs: 1000000,
      clockValid: true, stationIpv4: '192.168.1.10', wifiConnected: true, socket: null } };
  const routes = createRoutes(context, { verify: async () => {}, verifyEffect: async () => { if (maybeEffectError) throw maybeEffectError; }, now: () => time,
    persist: async (name, value) => { assert.ok(!saved.has(name)); saved.set(name, value); }, release: async () => { calls.push('release'); },
    prewarm: async () => { calls.push('prewarm'); }, sign: async operation => { calls.push(operation);
      return { profile: 'bwg-worker-lease-authorization-artifact/0.1', operation, authorization: 'synthetic' }; },
    launch: async () => { calls.push('fixture'); return { alive() {}, requireStartWindow() {}, validateStation() {},
      stratum: { profile: 'bwg-worker-stratum-v2-standard/0.1', endpoint: 'stratum+tcp://192.168.1.2:3333/',
        authorityPublicKey: Buffer.alloc(32, 1).toString('base64url'), userIdentity: 'synthetic' } }; } });
  const call = (path, value = {}) => routes.handle(path, value);
  async function before() {
    await call('/activate'); const challenge = await call('/startup/baseline-begin');
    await routes.diagnostics({ schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 5, reset_reason: 'power_on', uptime_ms: 1000 }] });
    await call('/startup/baseline', { nonce: challenge.nonce, state, ledger, original_budget: original, status });
  }
  async function candidate() { await before(); await call('/startup/candidate', { state: closed }); await call('/activate'); }
  return { routes, context, state, closed, status, saved, calls, call, before, candidate, advance: ms => { time += ms; } };
}
test('baseline proof uses collection origin and cannot be refreshed by a delayed close', async () => {
  const f = fixture(); await f.before(); f.advance(120001);
  await assert.rejects(f.call('/startup/candidate', { state: f.closed }), { code: 'startup_baseline_stale' });
  assert.equal(f.saved.has('current-recovery.json'), false);
});
test('recovery sequence getter is read-only and zero-round evidence is separate from fresh rounds', async () => {
  const f = fixture(); await f.candidate();
  assert.equal(f.routes.recoverySequence, 0);
  await f.call('/startup/recovery', { sequence: 0, stage: 'ledger', value: ledger });
  await f.call('/startup/recovery-finished', { sequence: 0, failures: [] });
  const first = await f.call('/startup/recovery-begin', { state: f.state, status: f.status });
  assert.equal(first.sequence, 1); assert.equal(f.routes.recoverySequence, 1); assert.equal(f.routes.recoverySequence, 1);
  await f.call('/startup/recovery', { sequence: 1, stage: 'ledger', value: ledger });
  assert.ok(f.saved.has('recovery-0-ledger.json')); assert.ok(f.saved.has('recovery-1-ledger.json'));
  await assert.rejects(f.call('/startup/recovery-begin', { state: f.state, status: f.status }), { code: 'startup_recovery_fresh_session' });
});
test('fresh measured allowance signs once, after signer prewarm and fixture readiness, without renewals', async () => {
  const f = fixture(); await f.candidate();
  const cooling = await f.call('/cooling-review-context');
  await f.call('/cooling-review', { nonce: cooling.nonce, state: f.state, budget_before: ledger, budget_after: ledger,
    proof: { schema: 'worker-cooling-proof-v1', fan_duty_percent: 100, fan_rpm: 3000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false },
    restoration: { schema: 'worker-cooling-baseline-v1', fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false } });
  const binding = Buffer.alloc(32, 1).toString('base64url'), budget = await f.call('/budget-review-context');
  await f.call('/budget-review', { nonce: budget.nonce, report: ledger, state: f.state, controlSessionBindingSha256: binding });
  await f.call('/startup/fixture', { status: f.status, controlSessionBindingSha256: binding });
  await f.call('/authorization-context', { controlSessionBindingSha256: binding });
  const artifacts = await f.routes.handle('/window-artifacts', undefined, 'GET');
  assert.equal(artifacts.grant.qualificationAttempt.ordinal, ledger.next_ordinal);
  assert.equal(artifacts.grant.qualificationAttempt.maximumActiveMilliseconds, 180000);
  assert.deepEqual(artifacts.renewals, []); assert.deepEqual(f.calls, ['prewarm', 'fixture', 'start']);
  await assert.rejects(f.routes.handle('/window-artifacts', undefined, 'GET'));
  await f.call('/startup/recovery', { sequence: 0, stage: 'ledger', value: ledger }); await f.call('/startup/release');
  assert.equal(f.calls.at(-1), 'release');
});

for (const [label, change] of [
  ['unexpected reboot', f => { f.status.observation.bootOrdinal += 1; }],
  ['accounting changed after preparation', f => { f.context.expectedLedger.total_charged_ms += 180000; }],
]) test(`prepared startup rejects ${label} before candidate admission`, async () => {
  // Arrange
  const f = fixture(); change(f);
  // Act / Assert
  await assert.rejects(f.before(), { code: 'startup_prepared_baseline' });
  assert.equal(f.saved.has('before.json'), false);
});

test('the first physical-admission failure remains precise through recovery and release', async () => {
  // Arrange
  const f = fixture(Object.assign(Error('stale'), { code: 'panic_detector_stale', detectorAgeMs: 64000 }));
  await f.candidate();
  // Act
  await assert.rejects(f.call('/cooling-review-context'), { code: 'panic_detector_stale' });
  await assert.rejects(f.call('/budget-review-context'));
  await f.call('/startup/recovery', { sequence: 0, stage: 'ledger', value: ledger });
  await f.call('/startup/release');
  // Assert
  assert.deepEqual(f.saved.get('first-failure.json'), { schema: 'str005-startup-failure-v1', phase: 'cooling', category: 'panic_detector_stale', detector_age_ms: 64000 });
  assert.ok(f.saved.has('recovery-0-ledger.json')); assert.ok(f.calls.includes('release'));
});

test('fresh recovery HTTP challenge admits accounting without retained status and closes against late writes', async () => {
  // Arrange: real HTTP boundary over production routes; synthetic already authenticated page state.
  const { createServer } = await import('node:http'), { once } = await import('node:events');
  const f = fixture(); await f.candidate();
  const server = createServer(async (request, response) => {
    const chunks = []; for await (const chunk of request) chunks.push(chunk);
    try { const result = await f.call(request.url, JSON.parse(Buffer.concat(chunks))); response.writeHead(200); response.end(JSON.stringify(result)); }
    catch { response.writeHead(400); response.end('{}'); }
  });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const post = async (path, value = {}) => { const response = await fetch(`http://127.0.0.1:${server.address().port}${path}`, {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(value) }); return { status: response.status, value: await response.json() }; };
  try {
    // Act: no status query or status proof precedes collection admission.
    const { value: challenge } = await post('/startup/recovery-challenge');
    const bound = { sequence: challenge.sequence, nonce: challenge.nonce, binding: Buffer.alloc(32, 9).toString('base64url') };
    assert.equal((await post('/startup/recovery-open', { ...bound, state: f.state })).status, 200);
    const ticket = (await post('/startup/recovery-stage-open', { ...bound, stage: 'ledger', limitMs: 30000 })).value;
    assert.equal((await post('/startup/recovery-part-v2', { ...bound, ticket, stage: 'ledger', value: ledger })).status, 200);
    assert.equal((await post('/startup/recovery-stage-close', { ...bound, stage: 'ledger', ticket })).status, 200);
    assert.equal((await post('/startup/recovery-part-v2', { ...bound, stage: 'finished', value: { failures: ['status'] } })).status, 200);
    // Assert: finished admission cannot be replayed or receive late data.
    assert.ok(f.saved.has('recovery-1-session.json')); assert.ok(f.saved.has('recovery-1-ledger.json'));
    assert.equal((await post('/startup/recovery-part-v2', { ...bound, stage: 'original_budget', value: original })).status, 400);
    assert.equal((await post('/startup/recovery-open', { ...bound, state: f.state })).status, 400);
  } finally { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
});

test('expired recovery collection rejects late observations but permits failure cleanup evidence', async () => {
  const f = fixture(); await f.candidate();
  const challenge = await f.call('/startup/recovery-challenge');
  const bound = { sequence: challenge.sequence, nonce: challenge.nonce, binding: Buffer.alloc(32, 8).toString('base64url') };
  await f.call('/startup/recovery-open', { ...bound, state: f.state });
  const ticket = await f.call('/startup/recovery-stage-open', { ...bound, stage: 'ledger', limitMs: 30000 }); f.advance(30001);
  await assert.rejects(f.call('/startup/recovery-part-v2', { ...bound, ticket, stage: 'ledger', value: ledger }), { code: 'startup_recovery_stale' });
  await f.call('/startup/recovery-part-v2', { ...bound, stage: 'closed', value: f.closed });
  await f.call('/startup/recovery-part-v2', { ...bound, stage: 'finished', value: { failures: ['ledger'] } });
  assert.equal(f.saved.has('recovery-1-ledger.json'), false); assert.ok(f.saved.has('recovery-1-closed.json'));
});

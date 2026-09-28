import test from 'node:test';
import assert from 'node:assert/strict';
import { createRoutes } from '../str005-startup-probe/routes.mjs';
import { state as fixtureState, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
function fixture() {
  const context = { source_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), gate_commit: 'd'.repeat(40),
    expectedBootOrdinal: 5, expectedLedger: structuredClone(ledger), detector: { physical: 'e'.repeat(64) }, scope: 'share', attemptId: Buffer.alloc(16, 1).toString('base64url') };
  const state = fixtureState(context), closed = fixtureState(context, 'candidate', true), saved = new Map(), calls = [];
  let time = 1000;
  const status = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', record: null, connection: null,
    observation: { bootOrdinal: 5, workerGeneration: 7, serialTransportEpoch: 8, observedAtUs: 1000000,
      clockValid: true, stationIpv4: '192.168.1.10', wifiConnected: true, socket: null } };
  const routes = createRoutes(context, { policy: { extraRoute: async ({ path, observeReady }) => {
    if (path !== '/share/observe-test') return; await observeReady(); return { handled: true, value: { recorded: true } }; } }, verify: async () => {}, verifyEffect: async () => { if (time > 61000) throw Object.assign(Error('stale'), { code: 'panic_detector_stale' }); }, now: () => time,
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
async function admitted(f) {
  await f.candidate();
  const cooling = await f.call('/cooling-review-context');
  await f.call('/cooling-review', { nonce: cooling.nonce, state: f.state, budget_before: ledger, budget_after: ledger,
    proof: { schema: 'worker-cooling-proof-v1', fan_duty_percent: 100, fan_rpm: 3000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false },
    restoration: { schema: 'worker-cooling-baseline-v1', fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false } });
  const binding = Buffer.alloc(32, 1).toString('base64url'), budget = await f.call('/budget-review-context');
  await f.call('/budget-review', { nonce: budget.nonce, report: ledger, state: f.state, controlSessionBindingSha256: binding });
  await f.call('/startup/fixture', { status: f.status, controlSessionBindingSha256: binding });
  await f.call('/authorization-context', { controlSessionBindingSha256: binding });
  await f.routes.handle('/window-artifacts', undefined, 'GET');
  await f.call('/startup/start-admit', { state: f.state, status: f.status, controlSessionBindingSha256: binding });
}
test('ongoing observations use admitted logical lineage after initial detector expires', async () => {
  // Arrange
  const f = fixture(); await admitted(f); f.advance(60001);
  // Act
  const result = await f.call('/share/observe-test');
  // Assert
  assert.equal(result.recorded, true);
  await assert.rejects(f.call('/authorization-context', {}), { code: 'panic_detector_stale' });
});
test('read-only hook cannot run before fresh Start admission', async () => {
  const f = fixture(); await f.candidate();
  await assert.rejects(f.call('/share/observe-test'), { code: 'startup_observation_admission' });
});

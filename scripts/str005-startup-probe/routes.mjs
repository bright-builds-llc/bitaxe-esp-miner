import { nonce } from '../fixed-usb-qualification/contract.mjs';
import { validateLedger, requireExhaustedOriginal, validateCooling } from '../fixed-usb-qualification/iterative-contract.mjs';
import { parseStatus } from '../str005-v2-serial/device.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { check, object, bytes } from '../str005-v2-serial/values.mjs';
import { freshAttempt, signStart } from './signing.mjs';
import { currentProof } from '../str005-panic-probe/model.mjs';
import { baseline, validateRun } from './evidence.mjs';
import { isDeepStrictEqual as same } from 'node:util';
/** One fresh lifetime, with recovery routes outside the effect failure latch. */
export function createRoutes(context, { verify, verifyEffect = verify, persist, sign, launch, release, prewarm = async () => {}, now = Date.now }) {
  let scope, before, review, cooling, budgetChallenge, coolingChallenge, fixture, network, artifacts, beforeDiagnostics, baselineChallenge;
  let failed = false, issued = false, delivered = false, candidate = false, recoveryRound = 0, activeRecovery = 0, startState = 'not_invoked';
  const stored = new Set(), sessions = new Set();
  const save = async (name, value) => { check(!stored.has(name), 'startup_record_consumed'); await persist(name, value); stored.add(name); };
  const fresh = value => check(value && scope && value.scope === scope.challengeId && now() >= value.at && now() - value.at <= 45000, 'startup_review_stale');
  const unchanged = value => { validateLedger(value); check(before && !value.pending && same(value, before.ledger), 'startup_accounting_changed'); };
  async function ready() { check(!failed && candidate && scope && before, 'startup_effect_admission'); await verifyEffect(); }
  async function networkReady() { await ready(); fresh(review); fresh(network); check(fixture && now() - network.at <= 10000, 'startup_fixture_stale'); fixture.alive(); fixture.requireStartWindow(); }
  async function route(path, input, method) {
    if (path === '/activate') {
      object(input, []); await verify(); scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
      review = cooling = network = undefined; return scope;
    }
    if (path === '/startup/context') { object(input, []); return { originalCampaignId: context.original_campaign_id, attemptId: before?.attempt.id ?? null, startState }; }
    if (path === '/startup/release') { object(input, []); await release(); return { released: true }; }
    if (path === '/startup/result') { const value = validateRun(input, context); if (value.firstFailure) failed = true; await save('run.json', value); startState = value.observedStart ? 'confirmed' : value.startInvokedAt === null ? 'not_invoked' : 'unknown'; return { recorded: true }; }
    if (path === '/startup/recovery-begin') {
      object(input, ['state', 'status']); check(before && recoveryRound < 4, 'startup_recovery_bound');
      const status = parseStatus(input.status), key = `${status.observation.bootOrdinal}:${status.observation.serialTransportEpoch}`;
      check(input.state.connected && !input.state.running && !sessions.has(key), 'startup_recovery_fresh_session');
      const { validateState } = await import('../fixed-usb-qualification/judge.mjs'); validateState(input.state, context);
      check(input.state.preservation?.baseline_id === before.state.preservation.baseline_id, 'startup_recovery_preservation');
      sessions.add(key); activeRecovery = ++recoveryRound;
      await save(`recovery-${activeRecovery}-session.json`, { key, boot: status.observation.bootOrdinal, transport: status.observation.serialTransportEpoch });
      return { sequence: activeRecovery };
    }
    if (path === '/startup/recovery') {
      object(input, ['sequence', 'stage', 'value']); check(input.sequence === activeRecovery && activeRecovery >= 0, 'startup_recovery_session');
      const value = projectRecoveryPart(input.stage, input.value, { ...context, attemptId: before.attempt.id });
      await save(`recovery-${activeRecovery}-${input.stage}.json`, value); return { recorded: true };
    }
    if (path === '/startup/recovery-finished') {
      object(input, ['sequence', 'failures']); check(input.sequence === activeRecovery && Array.isArray(input.failures) &&
        input.failures.every(stage => ['state', 'ledger', 'original_budget', 'status', 'diagnostics', 'closed'].includes(stage)), 'startup_recovery_finished');
      await save(`recovery-${activeRecovery}-finished.json`, { failures: input.failures }); return { recorded: true };
    }
    if (path === '/startup/baseline-begin') {
      object(input, []); check(!before && !baselineChallenge && scope, 'startup_baseline_consumed'); await verify();
      baselineChallenge = { nonce: nonce(), at: now() }; return { nonce: baselineChallenge.nonce };
    }
    if (path === '/startup/baseline') {
      object(input, ['nonce', 'state', 'ledger', 'original_budget', 'status']);
      check(baselineChallenge && input.nonce === baselineChallenge.nonce && now() >= baselineChallenge.at && now() - baselineChallenge.at <= 120000, 'startup_baseline_stale'); check(!before && scope, 'startup_baseline_consumed'); await verify();
      baseline(input.state, context); validateLedger(input.ledger); requireExhaustedOriginal(input.original_budget);
      const status = parseStatus(input.status); check(status.scope === 'share' && status.state === 'idle', 'startup_before_idle');
      check(status.observation.bootOrdinal === context.expectedBootOrdinal && same(input.ledger, context.expectedLedger), 'startup_prepared_baseline');
      before = { observedAtUnixMs: baselineChallenge.at, state: input.state, ledger: input.ledger, original_budget: input.original_budget, status: projectRecoveryPart('status', status, context), attempt: freshAttempt(input.ledger) };
      await save('before.json', before); return { attemptId: before.attempt.id };
    }
    if (path === '/startup/candidate') {
      object(input, ['state']); check(before && !candidate && !failed, 'startup_candidate');
      const state = input.state; check(state.status === 'closed' && !state.connected && !state.running && state.serialOwnershipReleased &&
        state.preservation?.baseline_id === before.state.preservation.baseline_id, 'startup_before_close');
      await save('before-closed.json', state);
      check(now() >= before.observedAtUnixMs && now() - before.observedAtUnixMs <= 120000, 'startup_baseline_stale');
      const parts = { state: before.state, ledger: before.ledger, original_budget: before.original_budget, status: before.status,
        diagnostics: beforeDiagnostics, closed: state, finished: { failures: [] } };
      const recovery = currentProof({ ...context, commit: context.source_commit, before_source: context }, parts, before.observedAtUnixMs);
      await save('current-recovery.json', recovery); candidate = true; return { candidate: true };
    }
    await ready();
    if (path === '/cooling-review-context' || path === '/budget-review-context') {
      object(input, []); check(!issued, 'startup_issuance_consumed');
      const challenge = { nonce: nonce(), at: now(), scope: scope.challengeId };
      if (path === '/cooling-review-context') coolingChallenge = challenge;
      else { fresh(cooling); budgetChallenge = challenge; }
      return { nonce: challenge.nonce, mode: 'iterative' };
    }
    if (path === '/cooling-review') {
      object(input, ['nonce', 'proof', 'restoration', 'budget_before', 'budget_after', 'state']);
      const challenge = coolingChallenge; coolingChallenge = null; fresh(challenge); check(input.nonce === challenge.nonce, 'startup_cooling_nonce');
      validateCooling(input.proof, input.restoration); unchanged(input.budget_before); unchanged(input.budget_after); baseline(input.state, context);
      await save('cooling.json', input); cooling = { at: now(), scope: scope.challengeId };
      return { cooling_review_saved: true, review_file: 'cooling.json' };
    }
    if (path === '/budget-review') {
      object(input, ['nonce', 'report', 'controlSessionBindingSha256', 'state']);
      const challenge = budgetChallenge; budgetChallenge = null; fresh(challenge); fresh(cooling);
      check(input.nonce === challenge.nonce, 'startup_budget_nonce'); unchanged(input.report); baseline(input.state, context); bytes(input.controlSessionBindingSha256, 32);
      await prewarm(); fresh(challenge);
      review = { at: now(), scope: scope.challengeId, binding: input.controlSessionBindingSha256 };
      await save('budget-review.json', { ledger: input.report, state: input.state, at: review.at }); return { budget_review_saved: true };
    }
    if (path === '/startup/fixture') {
      object(input, ['status', 'controlSessionBindingSha256']); fresh(review);
      check(!fixture && !issued && input.controlSessionBindingSha256 === review.binding, 'startup_fixture_admission');
      const status = parseStatus(input.status), o = status.observation;
      check(status.scope === 'share' && status.state === 'idle' && o.bootOrdinal === context.expectedBootOrdinal && o.wifiConnected && o.stationIpv4, 'startup_network');
      fixture = await launch({ ...context, attemptId: before.attempt.id, qualificationAttempt: before.attempt }, o.stationIpv4);
      network = { scope: scope.challengeId, at: now(), boot: o.bootOrdinal, generation: o.workerGeneration, transport: o.serialTransportEpoch };
      return { fixture_ready: true };
    }
    if (path === '/authorization-context') {
      object(input, ['controlSessionBindingSha256']); await networkReady();
      check(!issued && input.controlSessionBindingSha256 === review.binding, 'startup_issuance_consumed'); issued = true;
      await save('issuance-claim.json', { attempt: before.attempt, at: now(), deviceReservationObserved: false });
      artifacts = await signStart({ attempt: before.attempt, challengeId: scope.challengeId, binding: review.binding, stratum: fixture.stratum, sign, verify: networkReady });
      await save('issued.json', { attemptId: before.attempt.id, authorizationCount: 1, renewalCount: 0 }); return { ready: true };
    }
    if (path === '/window-artifacts') {
      check(method === 'GET' && input === undefined && issued && !delivered && artifacts, 'startup_delivery_consumed'); await networkReady();
      delivered = true; const result = artifacts; artifacts = null;
      await save('delivered.json', { attemptId: before.attempt.id, at: now(), deviceReservationObserved: false }); return result;
    }
    if (path === '/startup/start-admit') {
      object(input, ['state', 'status', 'controlSessionBindingSha256']); await networkReady();
      check(delivered && input.controlSessionBindingSha256 === review.binding, 'startup_start_binding'); baseline(input.state, context);
      const status = parseStatus(input.status), o = status.observation;
      check(status.state === 'idle' && o.bootOrdinal === network.boot && o.workerGeneration === network.generation && o.serialTransportEpoch === network.transport, 'startup_start_session');
      fixture.validateStation(o.stationIpv4); sessions.add(`${o.bootOrdinal}:${o.serialTransportEpoch}`);
      await save('start-admitted.json', { state: input.state, status: projectRecoveryPart('status', status, context) });
      return { generation: o.workerGeneration, attemptId: before.attempt.id };
    }
    check(false, 'startup_route_unavailable');
  }
  return { async handle(path, input, method = 'POST') {
    try { return await route(path, input, method); }
    catch (error) { if (!path.startsWith('/startup/recovery') && path !== '/startup/release') failed = true; throw error; }
  }, async diagnostics(value) {
    if (!candidate) { beforeDiagnostics = value; await save('before-diagnostics.json', value); return; }
    await save(`recovery-${activeRecovery}-diagnostics.json`, projectRecoveryPart('diagnostics', value, context));
  }, fail() { failed = true; }, failed: () => failed, get attemptId() { return before?.attempt.id; }, get recoverySequence() { return activeRecovery; } };
}

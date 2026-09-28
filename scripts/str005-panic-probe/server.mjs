import { validateCandidateFailure } from './candidate-failure.mjs';
import { requireStoreReady, recoveryStoreObservation } from './store-diagnostics.mjs';
import { verifyNativeAudit } from './audit.mjs';
import { validateSelfTest } from './self-test-evidence.mjs';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { BUNDLE, nonce } from '../fixed-usb-qualification/contract.mjs';
import { body, send } from '../fixed-usb-qualification/http.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { writeNew, canonical } from '../str005-noise-serial/files.mjs';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { validateCaptureDiagnosticPair } from './capture-diagnostics.mjs';
import { reviewExistingCapture, requireCurrentCaptureReview } from './capture-existing.mjs';
import { inspectInstall } from './install.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { parseStatus } from '../str005-v2-serial/device.mjs';
import { validatePart, validateFinished, baselineConclusion, currentProof, validateCandidateState, baselineIdentity } from './model.mjs';

/** Stage A exposes read/close operations only; later stages require separate admission. */
export function createProbeServer({ root, context, page, bundle, client, trust }, operations = {}) {
  const now = operations.now ?? Date.now;
  let queue = Promise.resolve(), finished = false, maybeFirstAt, candidateConfigured = false, selfTestClaimed = false, maybeSelfTestRequest, maybeCaptureReview;
  const saved = new Set(), parts = {}, candidateSessions = new Set();
  let baselineBegan = false, candidateRecoveryFailed = false;
  let candidateRound = 0, maybeCandidateRound, maybeCandidateDiagnostics;
  const scope = { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(now() / 1000) + 86400 };
  const persist = operations.persist ?? ((stage, value) => writeNew(resolve(root, `baseline-${stage}.json`), value));
  const server = createServer((request, response) => {
    const pending = queue.then(() => handle(request, response));
    queue = pending.catch(() => {
      if (!response.headersSent && !response.destroyed) send(response, 400, { error: 'panic_request_rejected' });
      else response.destroy();
    });
  });
  server.requestTimeout = 10000; server.headersTimeout = 10000;
  async function save(stage, value) {
    check(!context.renewSuccessor || baselineBegan || ['closed', 'finished'].includes(stage), 'renew_baseline_not_begun');
    check(!finished && !saved.has(stage), 'panic_part_consumed');
    await persist(stage, value); saved.add(stage); parts[stage] = value; maybeFirstAt ??= now();
  }
  async function handle(request, response) {
    const host = `127.0.0.1:${server.address().port}`, origin = `http://${host}`;
    check(request.headers.host === host, 'panic_host');
    const path = new URL(request.url, origin).pathname;
    if (request.method === 'GET') {
      if (path === '/context') return send(response, 200, { ...configuration(context, 'before', trust), coreDumpSelfTestQualification: true });
      if (path === '/') return send(response, 200, Buffer.from(`${page}\n<script type="module" src="/panic-probe-client.mjs"></script>`), 'text/html');
      if (path === `/${BUNDLE}`) return send(response, 200, bundle, 'text/javascript');
      if (path === '/panic-probe-client.mjs') return send(response, 200, client, 'text/javascript');
      return send(response, 404, { error: 'panic_route_unavailable' });
    }
    check(request.method === 'POST' && (request.headers.origin === origin ||
      (!request.headers.origin && request.headers['sec-fetch-site'] === 'same-origin')), 'panic_origin');
    const input = await body(request);
    if (path === '/activate') { object(input, []); return send(response, 200, scope); }
    if (path === '/baseline-begin') {
      object(input, []); check(context.renewSuccessor && !baselineBegan && saved.size === 0, 'renew_baseline_consumed');
      baselineBegan = true; maybeFirstAt = now();
      await (operations.persistBegin ?? (value => writeNew(resolve(root, 'baseline-begin.json'), value)))({ schema: 'str005-renew-baseline-begin-v1', startedAtUnixMs: maybeFirstAt });
      return send(response, 200, { begun: true });
    }
    if (path === '/probe-context') { object(input, []); return send(response, 200, { originalCampaignId: context.original_campaign_id, renewSuccessor: context.renewSuccessor === true, retainedAttemptId: context.retainedBaseline?.attemptId ?? null, recoveryOnly: context.recoveryOnly === true, captureExisting: context.captureExisting === true, selfTestEnabled: context.selfTestEnabled === true }); }
    if (context.recoveryOnly) check(!['/candidate', '/self-test-claim', '/self-test-result', '/candidate-recovery-begin', '/candidate-part', '/install'].includes(path), 'panic_recovery_only');
    if (context.renewSuccessor && ['/candidate', '/self-test-claim'].includes(path)) {
      check(typeof operations.verifyEffect === 'function', 'renew_effect_verifier_missing');
      await operations.verifyEffect();
    }
    if (path === '/candidate') {
      object(input, []); check((context.installEnabled || context.captureExisting) && finished && baselineConclusion(parts, context).complete && !candidateConfigured, 'panic_candidate_admission');
      if (context.captureExisting) {
        check(now() >= maybeFirstAt && now() - maybeFirstAt <= 120000, 'panic_capture_baseline_stale');
        await (operations.verifyNativeAudit ?? verifyNativeAudit)(root, context);
        maybeCaptureReview = reviewExistingCapture(parts, context, maybeFirstAt);
        await (operations.persistCapture ?? (value => writeNew(resolve(root, 'capture-review.json'), value)))(maybeCaptureReview);
      } else {
        const reviewed = await (operations.inspectInstall ?? inspectInstall)(root, context);
        await (operations.persistCandidate ?? (value => writeNew(resolve(root, 'candidate-install-review.json'), value)))(reviewed);
      }
      candidateConfigured = true;
      return send(response, 200, { ...configuration(context, 'candidate', trust), coreDumpSelfTestQualification: true });
    }
    if (path === '/self-test-claim') {
      object(input, ['state', 'status', 'ledger']);
      check(context.selfTestEnabled && candidateConfigured && !candidateRecoveryFailed && !selfTestClaimed, 'panic_self_test_admission');
      await (operations.verifyNativeAudit ?? verifyNativeAudit)(root, context);
      validateState(input.state, context); validateLedger(input.ledger); const status = parseStatus(input.status);
      if (context.captureExisting) requireCurrentCaptureReview(maybeCaptureReview, status, now());
      check(input.state.status === 'ready' && input.state.connected && !input.state.running && input.state.deviceLeaseInactive === true &&
        input.state.deviceBaselineConfirmed === true && input.state.preservation?.baseline_id === parts.closed.preservation.baseline_id &&
        input.state.preservation.settings_match && input.state.preservation.device_identity_match && input.state.preservation.authorization_high_water_match &&
        input.state.preservation.mine_on_boot === false && status.state === 'idle' && !input.ledger.pending &&
        canonical(input.ledger) === canonical(parts.ledger), 'panic_candidate_preservation');
      if (context.storeAuditRequired) {
        check(maybeCandidateDiagnostics && now() >= maybeCandidateDiagnostics.at && now() - maybeCandidateDiagnostics.at <= 30000, 'panic_store_diagnostics_stale');
        requireStoreReady(maybeCandidateDiagnostics.value, context, status.observation.bootOrdinal);
      }
      const request = { requestNonce: nonce(), expectedBootOrdinal: status.observation.bootOrdinal };
      await (operations.persistClaim ?? (value => writeNew(resolve(root, 'self-test-claim.json'), value)))({ request,
        state: input.state, status: validatePart('status', input.status, context), ledger: input.ledger, claimedAtUnixMs: now() });
      selfTestClaimed = true; maybeSelfTestRequest = request; return send(response, 200, request);
    }
    if (path === '/candidate-recovery-failure') {
      check(candidateConfigured && !candidateRecoveryFailed, 'panic_candidate_failure_consumed');
      validateCandidateFailure(input); candidateRecoveryFailed = true;
      let valid = false;
      try { validateCandidateState(input.closed, context, maybeSelfTestRequest); valid = true; }
      catch { /* Keep the original cause without persisting unvalidated page fields. */ }
      const closed = valid && input.close_complete && input.closed.status === 'closed' && input.closed.connected === false &&
        input.closed.serialOwnershipReleased === true && input.closed.deviceLeaseInactive === true;
      await (operations.persistCandidateFailure ?? (value => writeNew(resolve(root, 'candidate-recovery-failure.json'), value)))({
        schema: input.schema, first_failure: input.first_failure,
        stop_complete: Boolean(closed && input.stop_complete && input.closed.deviceRestorationConfirmed && input.closed.deviceBaselineConfirmed),
        close_complete: Boolean(closed), closed_state_valid: valid });
      return send(response, 200, { recorded: true, complete: false });
    }
    if (path === '/candidate-recovery-begin') {
      object(input, ['state', 'status']); check(candidateConfigured && !candidateRecoveryFailed && candidateRound < 8 && (!maybeCandidateRound || maybeCandidateRound.finished), 'panic_recovery_round');
      validateCandidateState(input.state, context, maybeSelfTestRequest); const status = parseStatus(input.status);
      check(input.state.status === 'ready' && input.state.connected && !input.state.running && input.state.deviceLeaseInactive &&
        input.state.preservation?.baseline_id === parts.closed.preservation.baseline_id && status.state === 'idle', 'panic_recovery_connection');
      const session = `${status.observation.bootOrdinal}:${status.observation.serialTransportEpoch}`;
      check(!candidateSessions.has(session), 'panic_recovery_fresh_session');
      const sequence = candidateRound + 1, relative = `candidate-recovery-${String(sequence).padStart(3, '0')}`;
      await (operations.createRecoveryRound ?? (path => mkdir(resolve(root, path), { mode: 0o700 })))(relative);
      maybeCandidateRound = { sequence, relative, session, parts: {}, saved: new Set(), started: now(), finished: false };
      if (context.renewSuccessor) await (operations.persistCandidatePart ?? ((path, value) => writeNew(resolve(root, path), value)))(`${relative}/collection-begin.json`, { schema: 'str005-renew-candidate-begin-v1', startedAtUnixMs: maybeCandidateRound.started, session });
      candidateRound = sequence; candidateSessions.add(session);
      return send(response, 200, { sequence, proofRelativePath: `${relative}/current-recovery.json` });
    }
    if (path === '/candidate-part') {
      object(input, ['sequence', 'stage', 'value']); const round = maybeCandidateRound;
      check(!candidateRecoveryFailed && round && input.sequence === round.sequence && !round.finished && !round.saved.has(input.stage), 'panic_candidate_part');
      const candidateContext = { ...context, retainedBaseline: undefined, before_source: context };
      const value = input.stage === 'finished' ? validateFinished(input.value, context) : ['state', 'closed'].includes(input.stage) ?
        validateCandidateState(input.value, context, maybeSelfTestRequest) : validatePart(input.stage, input.value, candidateContext);
      check(input.stage !== 'diagnostics', 'panic_diagnostic_route');
      await (operations.persistCandidatePart ?? ((path, value) => writeNew(resolve(root, path), value)))(`${round.relative}/${input.stage}.json`, value);
      round.parts[input.stage] = value; round.saved.add(input.stage);
      if (input.stage === 'finished') {
        round.finished = true;
        if (baselineConclusion(round.parts, candidateContext).complete) {
          if (context.renewSuccessor) check(canonical(round.parts.ledger) === canonical(parts.ledger) &&
            canonical(round.parts.original_budget) === canonical(parts.original_budget), 'renew_candidate_accounting_changed');
          check(now() - round.started <= 120000 &&
            `${round.parts.status.observation.bootOrdinal}:${round.parts.status.observation.serialTransportEpoch}` === round.session &&
            round.parts.state.preservation.baseline_id === parts.closed.preservation.baseline_id, 'panic_candidate_recovery_bound');
          await (operations.persistCandidateProof ?? ((path, value) => writeNew(resolve(root, path), value)))(`${round.relative}/current-recovery.json`, currentProof(candidateContext, round.parts, context.renewSuccessor ? round.started : now()));
          const maybeStore = context.storeAuditRequired ? recoveryStoreObservation(round.parts.diagnostics, round.parts.status, context, maybeSelfTestRequest) : undefined;
          if (maybeStore) await (operations.persistStoreObservation ?? ((path, value) => writeNew(resolve(root, path), value)))(`${round.relative}/store-observation.json`, maybeStore);
        }
      }
      return send(response, 200, { recorded: true, proofRelativePath: `${round.relative}/current-recovery.json`, complete: round.finished && baselineConclusion(round.parts, candidateContext).complete });
    }
    if (path === '/self-test-result') {
      object(input, ['evidence']); check(selfTestClaimed, 'panic_self_test_unclaimed');
      // The Gate-owned parser validates the complete evidence in the client; the server
      // retains it only through the same source-pinned parser boundary below.
      const value = await (operations.validateSelfTest ?? validateSelfTest)(input.evidence, context.gate_root, maybeSelfTestRequest, context);
      await (operations.persistSelfTest ?? (value => writeNew(resolve(root, 'self-test-result.json'), value)))(value);
      return send(response, 200, { recorded: true });
    }
    if (path === '/diagnostic-export') {
      const value = await (operations.validateDiagnostics ?? validateDiagnosticExport)(input, context.gate_root);
      if (candidateConfigured) {
        check(!candidateRecoveryFailed, 'panic_candidate_recovery_consumed');
        maybeCandidateDiagnostics = { value, at: now() };
        if (maybeCandidateRound && !maybeCandidateRound.finished) {
          check(!maybeCandidateRound.saved.has('diagnostics'), 'panic_part_consumed');
          await (operations.persistCandidatePart ?? ((path, value) => writeNew(resolve(root, path), value)))(`${maybeCandidateRound.relative}/diagnostics.json`, value);
          maybeCandidateRound.parts.diagnostics = value; maybeCandidateRound.saved.add('diagnostics');
        }
        await (operations.persistCandidateDiagnostics ?? (value => writeNew(resolve(root, `diagnostic-export-${nonce()}.json`), value)))(value);
      } else {
        const stage = context.captureExisting && saved.has('diagnostics') ? 'diagnostics_confirmation' : 'diagnostics';
        await save(stage, value);
      }
      return send(response, 200, { diagnostic_export_saved: true, review_file: 'diagnostic-export-baseline.json' });
    }
    check(path === '/part', 'panic_route_unavailable'); object(input, ['stage', 'value']);
    if (input.stage === 'diagnostics_confirmation_status') {
      check(context.captureExisting === true && !candidateConfigured, 'panic_capture_confirmation_mode');
      object(input.value, ['state', 'status']); validateState(input.value.state, baselineIdentity(context));
      await save(input.stage, { state: structuredClone(input.value.state), status: validatePart('status', input.value.status, context) });
      validateCaptureDiagnosticPair(parts, context);
      return send(response, 200, { recorded: true });
    }
    if (input.stage === 'finished') {
      await save('finished', validateFinished(input.value, context)); finished = true;
      if (baselineConclusion(parts, context).complete) {
        if (context.captureExisting) validateCaptureDiagnosticPair(parts, context);
        check(now() - maybeFirstAt <= 120000, 'panic_observations_stale');
        await (operations.persistProof ?? (value => writeNew(resolve(root, 'current-recovery.json'), value)))(currentProof(context, parts, context.renewSuccessor ? maybeFirstAt : now()));
      }
    }
    else await save(input.stage, validatePart(input.stage, input.value, context));
    return send(response, 200, { recorded: true });
  }
  server.release = async () => {
    const closed = server.listening ? once(server, 'close') : Promise.resolve();
    server.close(); server.closeAllConnections(); await queue; await closed;
  };
  return server;
}

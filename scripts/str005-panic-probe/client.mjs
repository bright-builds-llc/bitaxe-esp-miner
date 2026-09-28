/** Independent read/close steps retain partial evidence without a test-success latch. */
export function createBaselineCollector({ gate, published, save, campaignId, timeoutMs = 30000, captureExisting = false, attemptId = null, begin, wait = waitForDiagnosticAdvance }) {
  if (begin) return createRenewBaselineCollector({ gate, published, save, campaignId, timeoutMs, captureExisting, attemptId, begin, wait });
  let consumed = false;
  return async () => {
    if (consumed) throw Error('panic_baseline_consumed');
    consumed = true;
    const failures = [];
    async function collect(stage, operation, limit = timeoutMs) {
      let timer; const controller = new AbortController();
      try {
        await Promise.race([Promise.resolve().then(() => operation(controller.signal)).then(value => { controller.signal.throwIfAborted(); return save(stage, value); }),
          new Promise((_, reject) => { timer = setTimeout(() => { controller.abort(); reject(Error('timeout')); }, limit); })]);
      } catch { if (!failures.includes(stage)) failures.push(stage); }
      finally { clearTimeout(timer); controller.abort(); }
    }
    try {
      await collect('state', async () => { await gate.refresh(); return published(); });
      await collect('ledger', () => gate.reviewQualificationAttempts());
      await collect('original_budget', () => gate.reviewBudget(campaignId));
      await collect('status', async () => gate.stratumV2Status('share', attemptId, await gate.stratumV2Possession()));
      await collect('diagnostics', signal => captureExisting ? collectCaptureDiagnostics({ gate, published, save, signal, wait }) : gate.exportDiagnostics());
    } finally {
      await collect('closed', async () => {
        let maybeStopError;
        try { await gate.stop(); } catch (error) { maybeStopError = error; }
        await gate.close();
        if (maybeStopError && !failures.includes('closed')) failures.push('closed');
        return published();
      }, 150000);
    }
    await save('finished', { failures });
    return { complete: failures.length === 0, failures };
  };
}
/** Successor failure collection bounds begin, Stop and Close independently. */
export function createRenewBaselineCollector({ gate, published, save, campaignId, timeoutMs = 30000,
  cleanupMs = 150000, captureExisting = false, attemptId = null, begin, wait = waitForDiagnosticAdvance }) {
  let consumed = false;
  return async () => {
    if (consumed) throw Error('panic_baseline_consumed'); consumed = true;
    const failures = []; let firstFailure = null;
    async function attempt(phase, operation, stage = phase, limit = timeoutMs) {
      const controller = new AbortController(); let timer;
      try {
        await Promise.race([Promise.resolve().then(() => operation(controller.signal)).then(async value => {
          controller.signal.throwIfAborted(); if (stage) await save(stage, value);
        }), new Promise((_, reject) => { timer = setTimeout(() => { controller.abort(); reject(Error('timeout')); }, limit); })]);
        return true;
      } catch (error) {
        firstFailure ??= { phase, category: controller.signal.aborted ? 'timeout' : 'operation_failed' };
        const failedStage = ['begin', 'state'].includes(phase) ? 'state' : ['stop', 'close'].includes(phase) ? 'closed' : phase;
        if (!failures.includes(failedStage)) failures.push(failedStage);
        return false;
      } finally { clearTimeout(timer); controller.abort(); }
    }
    try {
      if (await attempt('begin', begin, null)) {
        await attempt('state', async () => { await gate.refresh(); return published(); });
        await attempt('ledger', () => gate.reviewQualificationAttempts());
        await attempt('original_budget', () => gate.reviewBudget(campaignId));
        await attempt('status', async () => gate.stratumV2Status('share', attemptId, await gate.stratumV2Possession()));
        await attempt('diagnostics', signal => captureExisting ? collectCaptureDiagnostics({ gate, published, save, signal, wait }) : gate.exportDiagnostics());
      }
    } finally {
      await attempt('stop', () => gate.stop(), null, cleanupMs);
      await attempt('close', async () => { await gate.close(); return published(); }, 'closed', cleanupMs);
    }
    const finished = { failures, first_failure: firstFailure };
    await save('finished', finished);
    return { complete: failures.length === 0, ...finished };
  };
}
/** A candidate prelude is part of the owned attempt, including unconditional release. */
export function createCandidateRecovery({ gate, published, begin, collect, failure, timeoutMs = 30000, cleanupMs = 150000, collectionMs = 600000 }) {
  let failed = false, running = false;
  const execute = async () => {
    if (failed || running) throw Error('panic_candidate_recovery_consumed'); running = true;
    let firstFailure = null, result, stopComplete = false, closeComplete = false;
    async function bounded(phase, operation, limit = timeoutMs) {
      let timer;
      try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
        timer = setTimeout(() => reject(Object.assign(Error('timeout'), { boundedTimeout: true })), limit);
      })]); }
      catch (error) {
        firstFailure ??= { phase, category: error?.boundedTimeout ? 'timeout' : error?.code === 'v2_retained_evidence' || error?.message === 'v2_retained_evidence' ? 'v2_retained_evidence' : 'operation_failed' };
        throw error;
      } finally { clearTimeout(timer); }
    }
    try {
      const possession = await bounded('possession', () => gate.stratumV2Possession());
      const status = await bounded('status', () => gate.stratumV2Status('share', null, possession));
      const round = await bounded('begin', () => begin({ state: published(), status }));
      result = await bounded('collection', () => collect(round), collectionMs);
      if (result.complete !== true) firstFailure ??= result.first_failure ?? { phase: 'collection', category: 'operation_failed' };
    } catch { firstFailure ??= { phase: 'collection', category: 'operation_failed' }; }
    finally {
      const alreadyClosed = () => { try { const value = published(); return value.status === 'closed' && value.connected === false && value.serialOwnershipReleased === true; } catch { return false; } };
      if (alreadyClosed()) {
        const closed = published();
        stopComplete = closed.deviceRestorationConfirmed === true && closed.deviceLeaseInactive === true && closed.running === false;
        closeComplete = true;
        if (!stopComplete) firstFailure ??= { phase: 'stop', category: 'operation_failed' };
      }
      else {
        try { await bounded('stop', () => gate.stop(), cleanupMs); stopComplete = true; } catch { /* Preserve first cause and still close. */ }
        try { await bounded('close', () => gate.close(), cleanupMs); closeComplete = alreadyClosed(); } catch { /* Record unproved closure. */ }
        if (!closeComplete) firstFailure ??= { phase: 'close', category: 'operation_failed' };
      }
      running = false;
    }
    if (firstFailure) {
      failed = true;
      let maybeClosed = null; try { maybeClosed = published(); } catch { /* No closure claim from unreadable page state. */ }
      const value = { schema: 'str005-candidate-recovery-failure-v1', first_failure: firstFailure,
        stop_complete: stopComplete, close_complete: closeComplete, closed: maybeClosed };
      await bounded('failure_record', () => failure(value));
      return { complete: false, first_failure: firstFailure };
    }
    return result;
  };
  execute.blocked = () => failed;
  return execute;
}
/** Both snapshots are produced by the Gate while the same serial owner remains open. */
export async function collectCaptureDiagnostics({ gate, published, save, signal, wait = waitForDiagnosticAdvance }) {
  await gate.exportDiagnostics(); signal.throwIfAborted();
  await wait(2000, signal); signal.throwIfAborted();
  await gate.exportDiagnostics(); signal.throwIfAborted();
  const status = await gate.stratumV2Status('share', null, await gate.stratumV2Possession()); signal.throwIfAborted();
  await save('diagnostics_confirmation_status', { state: published(), status });
  return { diagnostic_exports: 2 };
}
function waitForDiagnosticAdvance(milliseconds, signal) {
  return new Promise((resolve, reject) => {
    const stop = () => { clearTimeout(timer); reject(Error('diagnostic_wait_cancelled')); };
    const timer = setTimeout(() => { signal.removeEventListener('abort', stop); resolve(); }, milliseconds);
    signal.addEventListener('abort', stop, { once: true });
    if (signal.aborted) stop();
  });
}
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const post = async (path, value) => {
    const response = await fetch(path, { method: 'POST', headers: { 'Content-Type': 'application/json' },
      cache: 'no-store', redirect: 'error', body: JSON.stringify(value) });
    if (!response.ok) throw Error('panic_record_rejected');
    return response.json();
  };
  const gate = window.workerAcceptance;
  const context = await post('/probe-context', {});
  const selfTestEnabled = context.selfTestEnabled === true && !context.recoveryOnly;
  const coreDumpSelfTest = selfTestEnabled ? gate.coreDumpSelfTest.bind(gate) : undefined;
  const exportSelfTest = selfTestEnabled ? gate.exportCoreDumpSelfTestEvidence.bind(gate) : undefined;
  const allowed = Object.fromEntries(['refresh', 'reviewQualificationAttempts', 'reviewBudget', 'exportDiagnostics', 'stratumV2Possession', 'stratumV2Status', 'stop', 'close']
    .map(key => [key, gate[key].bind(gate)]));
  // This stage cannot issue a grant, reserve an ordinal, trigger a panic or start mining.
  for (const key of ['prepareStartAuthorization', 'loadWindow', 'loadSignedWindow', 'startWindow', 'stratumV2ChannelStart',
    'suppressHeartbeats', 'armForegroundLoss', 'submitCoolingReview', 'proveCoolingForQualification', 'restoreCoolingBaseline', 'coreDumpSelfTest']) delete gate[key];
  for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
  const button = document.createElement('button'), output = document.createElement('pre');
  button.id = 'capture-panic-baseline'; button.textContent = 'Capture baseline and accounting, then close';
  output.id = 'panic-probe-result'; output.setAttribute('role', 'status');
  let consumed = false;
  const update = () => { const state = gate.state(); button.disabled = consumed || state.status !== 'ready' || !state.connected || state.running; };
  const collect = createBaselineCollector({ gate: allowed, published: () => gate.state(), campaignId: context.originalCampaignId, captureExisting: context.captureExisting === true, attemptId: context.retainedAttemptId ?? null,
    begin: context.renewSuccessor ? () => post('/baseline-begin', {}) : undefined,
    save: (stage, value) => stage === 'diagnostics' ? Promise.resolve() : post('/part', { stage, value }) });
  button.addEventListener('click', async () => {
    if (button.disabled) return;
    consumed = true; update(); output.textContent = 'Capturing authenticated baseline';
    try { output.textContent = JSON.stringify(await collect()); }
    catch { output.textContent = 'Partial baseline retained. Stop/Close remain available. Do not flash or Start.'; }
  });
  const candidate = document.createElement('button'), selfTest = document.createElement('button'), recover = document.createElement('button');
  candidate.id = 'configure-panic-candidate'; candidate.textContent = context.captureExisting ? 'Qualify recovered image for capture' : 'Verify installation and configure candidate';
  selfTest.id = 'capture-core-self-test'; selfTest.textContent = 'Run one off-only core capture self-test';
  let candidateConfigured = false, selfTestUsed = false;
  recover.id = 'recover-panic-candidate'; recover.textContent = 'Collect candidate recovery and close';
  let recovering = false, recoveryCount = 0;
  const recoverCandidate = createCandidateRecovery({ gate: allowed, published: () => gate.state(),
    begin: input => post('/candidate-recovery-begin', input), failure: value => post('/candidate-recovery-failure', value),
    collect: async round => {
      recoveryCount += 1; let maybeProofReceipt;
      const collect = createBaselineCollector({ gate: allowed, published: () => gate.state(), campaignId: context.originalCampaignId,
        begin: context.renewSuccessor ? async () => {} : undefined,
        save: async (stage, value) => {
          if (stage === 'diagnostics') return;
          const receipt = await post('/candidate-part', { sequence: round.sequence, stage, value });
          if (stage === 'finished') maybeProofReceipt = receipt;
        } });
      const collected = await collect();
      return { ...collected, complete: collected.complete === true && maybeProofReceipt?.complete === true, proofRelativePath: maybeProofReceipt?.complete ? round.proofRelativePath : null };
    } });
  recover.addEventListener('click', async () => {
    if (recover.disabled) return; recovering = true; recover.disabled = true;
    try { output.textContent = JSON.stringify(await recoverCandidate()); }
    catch { output.textContent = 'Candidate recovery partial; preserve evidence. Stop/Close were attempted independently.'; }
    finally { recovering = false; updateCandidate(); }
  });
  candidate.addEventListener('click', async () => {
    if (context.recoveryOnly) return;
    candidate.disabled = true;
    try { await gate.configure(await post('/candidate', {})); candidateConfigured = true; output.textContent = 'Candidate configured. Use native Connect Worker.'; }
    catch { candidate.disabled = false; output.textContent = context.captureExisting ? 'Capture blocked: a fresh healthy baseline and preserved empty core are required.' : 'Candidate blocked: complete baseline and repo-owned installation first.'; }
  });
  selfTest.addEventListener('click', async () => {
    if (!selfTestEnabled || selfTestUsed || !candidateConfigured) return;
    selfTestUsed = true; selfTest.disabled = true;
    let maybeFailure;
    try {
      await allowed.exportDiagnostics();
      const request = await post('/self-test-claim', { state: gate.state(), ledger: await allowed.reviewQualificationAttempts(),
        status: await allowed.stratumV2Status('share', null, await allowed.stratumV2Possession()) });
      await coreDumpSelfTest(request);
    } catch (error) { maybeFailure = error; }
    finally {
      try { const evidence = exportSelfTest(); if (evidence) await post('/self-test-result', { evidence }); } catch { maybeFailure ??= Error('evidence'); }
      try { await allowed.exportDiagnostics(); } catch { maybeFailure ??= Error('diagnostics'); }
      try { await allowed.stop(); } catch { maybeFailure ??= Error('stop'); }
      try { await allowed.close(); } catch { maybeFailure ??= Error('close'); }
    }
    output.textContent = maybeFailure ? 'Self-test incomplete; evidence retained. Collect the dump and diagnose before any Start.' :
      'Self-test returned and closed. Core capture is unverified until offline dump analysis succeeds. Start remains unavailable.';
  });
  const updateCandidate = () => { const state = gate.state(); selfTest.disabled = recoverCandidate.blocked() || !selfTestEnabled || selfTestUsed || recovering || !candidateConfigured || state.status !== 'ready' || !state.connected || state.running;
    recover.disabled = recoverCandidate.blocked() || recovering || recoveryCount >= 8 || !candidateConfigured || state.status !== 'ready' || !state.connected || state.running; };
  if (context.recoveryOnly) document.body.append(button, output);
  else document.body.append(button, candidate, ...(selfTestEnabled ? [selfTest] : []), recover, output); update(); updateCandidate();
  const originalUpdate = update;

  const stateOutput = document.getElementById('state');
  if (stateOutput) new MutationObserver(() => { originalUpdate(); updateCandidate(); }).observe(stateOutput, { childList: true, subtree: true, characterData: true });
}

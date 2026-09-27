/** Independent read/close steps retain partial evidence without a test-success latch. */
export function createBaselineCollector({ gate, published, save, campaignId, timeoutMs = 30000 }) {
  let consumed = false;
  return async () => {
    if (consumed) throw Error('panic_baseline_consumed');
    consumed = true;
    const failures = [];
    async function collect(stage, operation, limit = timeoutMs) {
      let timer;
      try {
        await Promise.race([Promise.resolve().then(operation).then(value => save(stage, value)),
          new Promise((_, reject) => { timer = setTimeout(() => reject(Error('timeout')), limit); })]);
      } catch { if (!failures.includes(stage)) failures.push(stage); }
      finally { clearTimeout(timer); }
    }
    try {
      await collect('state', async () => { await gate.refresh(); return published(); });
      await collect('ledger', () => gate.reviewQualificationAttempts());
      await collect('original_budget', () => gate.reviewBudget(campaignId));
      await collect('status', async () => gate.stratumV2Status('share', null, await gate.stratumV2Possession()));
      await collect('diagnostics', () => gate.exportDiagnostics());
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
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const post = async (path, value) => {
    const response = await fetch(path, { method: 'POST', headers: { 'Content-Type': 'application/json' },
      cache: 'no-store', redirect: 'error', body: JSON.stringify(value) });
    if (!response.ok) throw Error('panic_record_rejected');
    return response.json();
  };
  const gate = window.workerAcceptance;
  const context = await post('/probe-context', {});
  const coreDumpSelfTest = context.recoveryOnly ? undefined : gate.coreDumpSelfTest.bind(gate);
  const exportSelfTest = context.recoveryOnly ? undefined : gate.exportCoreDumpSelfTestEvidence.bind(gate);
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
  const collect = createBaselineCollector({ gate: allowed, published: () => gate.state(), campaignId: context.originalCampaignId,
    save: (stage, value) => stage === 'diagnostics' ? Promise.resolve() : post('/part', { stage, value }) });
  button.addEventListener('click', async () => {
    if (button.disabled) return;
    consumed = true; update(); output.textContent = 'Capturing authenticated baseline';
    try { output.textContent = JSON.stringify(await collect()); }
    catch { output.textContent = 'Partial baseline retained. Stop/Close remain available. Do not flash or Start.'; }
  });
  const candidate = document.createElement('button'), selfTest = document.createElement('button'), recover = document.createElement('button');
  candidate.id = 'configure-panic-candidate'; candidate.textContent = 'Verify installation and configure candidate';
  selfTest.id = 'capture-core-self-test'; selfTest.textContent = 'Run one off-only core capture self-test';
  let candidateConfigured = false, selfTestUsed = false;
  recover.id = 'recover-panic-candidate'; recover.textContent = 'Collect candidate recovery and close';
  let recovering = false, recoveryCount = 0;
  recover.addEventListener('click', async () => {
    if (recover.disabled) return; recovering = true; recover.disabled = true;
    try {
      const round = await post('/candidate-recovery-begin', { state: gate.state(),
        status: await allowed.stratumV2Status('share', null, await allowed.stratumV2Possession()) });
      recoveryCount += 1;
      let maybeProofReceipt;
      const collect = createBaselineCollector({ gate: allowed, published: () => gate.state(), campaignId: context.originalCampaignId,
        save: async (stage, value) => {
          if (stage === 'diagnostics') return;
          const receipt = await post('/candidate-part', { sequence: round.sequence, stage, value });
          if (stage === 'finished') maybeProofReceipt = receipt;
        } });
      const collected = await collect();
      output.textContent = JSON.stringify({ ...collected, proofRelativePath: maybeProofReceipt?.complete ? round.proofRelativePath : null });
    } catch { output.textContent = 'Candidate recovery partial; retain this attempt and close. No Start is available.'; }
    finally { recovering = false; updateCandidate(); }
  });
  candidate.addEventListener('click', async () => {
    if (context.recoveryOnly) return;
    candidate.disabled = true;
    try { await gate.configure(await post('/candidate', {})); candidateConfigured = true; output.textContent = 'Candidate configured. Use native Connect Worker.'; }
    catch { candidate.disabled = false; output.textContent = 'Candidate blocked: complete baseline and repo-owned installation first.'; }
  });
  selfTest.addEventListener('click', async () => {
    if (context.recoveryOnly || selfTestUsed || !candidateConfigured) return;
    selfTestUsed = true; selfTest.disabled = true;
    let maybeFailure;
    try {
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
  const updateCandidate = () => { const state = gate.state(); selfTest.disabled = context.recoveryOnly || selfTestUsed || recovering || !candidateConfigured || state.status !== 'ready' || !state.connected || state.running;
    recover.disabled = recovering || recoveryCount >= 8 || !candidateConfigured || state.status !== 'ready' || !state.connected || state.running; };
  if (context.recoveryOnly) document.body.append(button, output);
  else document.body.append(button, candidate, selfTest, recover, output); update(); updateCandidate();
  const originalUpdate = update;

  const stateOutput = document.getElementById('state');
  if (stateOutput) new MutationObserver(() => { originalUpdate(); updateCandidate(); }).observe(stateOutput, { childList: true, subtree: true, characterData: true });
}

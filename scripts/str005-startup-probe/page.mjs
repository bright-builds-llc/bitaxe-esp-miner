import { createCoordinator, collectRecovery, readRecoveryStatus } from './client.mjs';
export function createPage(gate, post, notice) {
  let candidate = false, attemptId, binding, recoverySequence = 0, startState = 'not_invoked';
  const state = () => gate.state();
  const recovery = async sequence => {
    const context = await post('/startup/context', {});
    await collectRecovery({ gate, campaignId: context.originalCampaignId, attemptId, statusMode: startState,
      save: (stage, value) => post('/startup/recovery', { sequence, stage, value }) });
  };
  const run = createCoordinator({ gate, prepare: async () => {
    if (!candidate || !state().connected) throw Error('startup_candidate_required');
    await gate.submitCoolingReview(); await gate.submitBudgetReview();
    binding = (await gate.stratumV2TelemetryEndpoint()).controlSessionBindingSha256;
    await post('/startup/fixture', { status: await gate.stratumV2Status('share', null, binding), controlSessionBindingSha256: binding });
    const signed = await gate.prepareStartAuthorization(); if (signed.controlSessionBindingSha256 !== binding) throw Error('startup_binding_changed');
    await gate.loadSignedWindow();
    const admitted = await post('/startup/start-admit', { state: state(), status: await gate.stratumV2Status('share', null, binding), controlSessionBindingSha256: binding });
    attemptId = admitted.attemptId; return admitted;
  }, record: async value => {
    if (Object.hasOwn(value, 'closed')) return post('/startup/recovery', { sequence: 0, stage: 'closed', value: value.closed });
    startState = value.observedStart ? 'confirmed' : value.startInvokedAt === null ? 'not_invoked' : 'unknown';
    return post('/startup/result', value);
  }, recover: async () => {
    let failures = [];
    try { await recovery(0); } catch (error) { failures = error.failures ?? ['status']; throw error; }
    finally { await post('/startup/recovery-finished', { sequence: 0, failures }); }
  }, release: () => post('/startup/release', {}) });
  return {
    async baseline() {
      if (candidate) throw Error('startup_candidate_consumed');
      const challenge = await post('/startup/baseline-begin', {});
      const context = await post('/startup/context', {});
      const ledger = await gate.reviewQualificationAttempts(), original_budget = await gate.reviewBudget(context.originalCampaignId);
      const status = await gate.stratumV2Status('share', null, await gate.stratumV2Possession());
      await gate.exportDiagnostics();
      const receipt = await post('/startup/baseline', { nonce: challenge.nonce, state: state(), ledger, original_budget, status }); attemptId = receipt.attemptId;
      await gate.close(); await gate.configure(await post('/startup/candidate', { state: state() })); candidate = true;
      notice('Use native Connect Worker for a fresh candidate session.');
    },
    async run() { const result = await run(); notice(JSON.stringify(result)); return result; },
    async recoverFresh() {
      const failures = [];
      try {
        const context = await post('/startup/context', {});
        attemptId ??= context.attemptId;
        if (context.startState === 'confirmed') startState = 'confirmed';
        const status = await readRecoveryStatus(gate, attemptId, startState);
        const round = await post('/startup/recovery-begin', { state: state(), status }); recoverySequence = round.sequence;
        await recovery(recoverySequence);
      } catch (error) { failures.push(...(error.failures ?? ['status'])); }
      finally {
        try { await gate.stop(); } catch { failures.push('state'); }
        try { await gate.close(); await post('/startup/recovery', { sequence: recoverySequence, stage: 'closed', value: state() }); }
        catch { failures.push('closed'); }
      }
      await post('/startup/recovery-finished', { sequence: recoverySequence, failures });
      notice(failures.length ? 'Recovery incomplete; preserve evidence.' : 'Fresh recovery recorded. Host finalization remains required.');
    },
  };
}
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const publicGate = window.workerAcceptance;
  const names = ['state', 'configure', 'refresh', 'close', 'stop', 'reviewQualificationAttempts', 'reviewBudget', 'stratumV2Possession', 'stratumV2Status',
    'stratumV2TelemetryEndpoint', 'submitCoolingReview', 'submitBudgetReview', 'prepareStartAuthorization', 'loadSignedWindow', 'startWindow', 'exportDiagnostics'];
  const gate = Object.fromEntries(names.map(name => [name, publicGate[name].bind(publicGate)]));
  for (const name of ['startWindow', 'loadWindow', 'loadSignedWindow', 'prepareStartAuthorization', 'suppressHeartbeats', 'armForegroundLoss',
    'coreDumpSelfTest', 'stratumV2ChannelStart', 'submitCoolingReview', 'proveCoolingForQualification', 'restoreCoolingBaseline']) delete publicGate[name];
  for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
  const output = document.createElement('pre'); output.id = 'startup-result'; document.body.append(output);
  const post = async (path, value) => { const result = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!result.ok) throw Error('startup_request_rejected'); return result.json(); };
  const page = createPage(gate, post, value => { output.textContent = value; });
  for (const [label, name] of [['Record baseline and configure candidate', 'baseline'], ['Run one startup probe', 'run'], ['Collect fresh recovery and close', 'recoverFresh']]) {
    const button = document.createElement('button'); button.textContent = label;
    button.addEventListener('click', async () => { button.disabled = true; try { await page[name](); }
      catch { output.textContent = 'Operation incomplete; retain evidence. Stop and Close remain available.'; } }); document.body.append(button);
  }
}

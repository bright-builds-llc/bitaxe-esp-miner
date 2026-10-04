import { createRecoveryCollection } from './recovery-collection.mjs';
import { createCoordinator, collectRecovery } from './client.mjs';
export function createPage(gate, post, notice, options = {}) {
  let candidate = false, attemptId, binding, startState = 'not_invoked';
  const state = () => gate.state();
  const recovery = async sequence => {
    const context = await post('/startup/context', {});
    await collectRecovery({ gate, campaignId: context.originalCampaignId, attemptId, statusMode: startState,
      save: (stage, value) => post('/startup/recovery', { sequence, stage, value }) });
  };
  const run = (options.coordinator ?? createCoordinator)({ ...options.coordinatorOptions, gate, prepare: async () => {
    if (!candidate || !state().connected) throw Error('startup_candidate_required');
    await gate.submitCoolingReview(); await gate.submitBudgetReview();
    const endpoint = await gate.stratumV2TelemetryEndpoint(); binding = endpoint.controlSessionBindingSha256;
    if (options.beforeFixture) await options.beforeFixture({ gate, post, binding, endpoint });
    await post('/startup/fixture', { status: await gate.stratumV2Status('share', null, binding), controlSessionBindingSha256: binding });
    const signed = await gate.prepareStartAuthorization(); if (signed.controlSessionBindingSha256 !== binding) throw Error('startup_binding_changed');
    await gate.loadSignedWindow();
    const admitted = await post('/startup/start-admit', { state: state(), status: await gate.stratumV2Status('share', null, binding), controlSessionBindingSha256: binding });
    attemptId = admitted.attemptId; return options.prepare ? options.prepare({ ...admitted, binding }) : { ...admitted, binding };
  }, recordFailure: value => post('/startup/client-failure', value), record: async value => {
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
    async run() {
      // The coordinator is one-shot; invoking it before the baseline would seal an attempt that never reached Start.
      if (!candidate) throw Error('startup_candidate_required');
      const result = await run(); notice(JSON.stringify(result)); return result;
    },
    async recoverFresh() {
      let challenge, recoveryBinding;
      const collect = createRecoveryCollection({ gate,
        begin: async () => {
          challenge = await post('/startup/recovery-challenge', {});
          attemptId ??= challenge.attemptId;
          recoveryBinding = await gate.stratumV2Possession();
          await post('/startup/recovery-open', { sequence: challenge.sequence, nonce: challenge.nonce, binding: recoveryBinding, state: state() });
          return { binding: recoveryBinding, attemptId, campaignId: challenge.campaignId, statusMode: challenge.startState };
        },
        beforeStage: (stage, limitMs) => post('/startup/recovery-stage-open', { sequence: challenge.sequence, nonce: challenge.nonce, binding: recoveryBinding, stage, limitMs }),
        afterStage: (stage, ticket) => post('/startup/recovery-stage-close', { sequence: challenge.sequence, nonce: challenge.nonce, binding: recoveryBinding, stage, ticket }),
        save: (stage, value, ticket) => {
          if (!challenge) throw Error('startup_recovery_challenge_missing');
          return post('/startup/recovery-part-v2', { sequence: challenge.sequence, nonce: challenge.nonce, binding: recoveryBinding ?? null, stage, value, ...(ticket === undefined ? {} : { ticket }) });
        },
      });
      const result = await collect();
      notice(result.complete ? 'Fresh recovery recorded. Host finalization remains required.' : 'Recovery incomplete; preserve evidence.');
      return result;
    },
  };
}
export function installPage(options = {}) {
  const publicGate = window.workerAcceptance;
  const names = ['state', 'configure', 'refresh', 'close', 'stop', 'reviewQualificationAttempts', 'reviewBudget', 'stratumV2Possession', 'stratumV2Status',
    'stratumV2TelemetryEndpoint', 'submitCoolingReview', 'submitBudgetReview', 'prepareStartAuthorization', 'loadSignedWindow', 'startWindow', 'exportDiagnostics', ...(options.capabilities ?? [])];
  const gate = Object.fromEntries(names.map(name => [name, publicGate[name].bind(publicGate)]));
  for (const name of ['startWindow', 'loadWindow', 'loadSignedWindow', 'prepareStartAuthorization', 'suppressHeartbeats', 'armForegroundLoss',
    'coreDumpSelfTest', 'stratumV2ChannelStart', 'submitCoolingReview', 'proveCoolingForQualification', 'restoreCoolingBaseline']) delete publicGate[name];
  for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
  const output = document.createElement('pre'); output.id = 'startup-result'; document.body.append(output);
  const post = async (path, value) => { const result = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!result.ok) throw Error('startup_request_rejected'); return result.json(); };
  const page = createPage(gate, post, value => { output.textContent = value; }, { ...options, coordinatorOptions: { ...options.coordinatorOptions, post } });
  for (const [label, name] of [['Record baseline and configure candidate', 'baseline'], [options.runLabel ?? 'Run one startup probe', 'run'], ['Collect fresh recovery and close', 'recoverFresh']]) {
    const button = document.createElement('button'); button.textContent = label;
    button.addEventListener('click', async () => { button.disabled = true; try { await page[name](); }
      catch (error) {
        if (error.message === 'startup_candidate_required') { button.disabled = false; output.textContent = 'Record baseline and configure candidate first.'; return; }
        output.textContent = 'Operation incomplete; retain evidence. Stop and Close remain available.';
      } }); document.body.append(button);
  }
}

if (typeof window !== 'undefined' && typeof document !== 'undefined' && !globalThis.str005CustomPage) installPage();

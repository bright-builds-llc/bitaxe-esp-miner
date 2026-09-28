import { prepareRecovery } from './bootstrap.mjs';
import { createRecoveryCollection } from './recovery-collection.mjs';
const original = window.workerAcceptance;
const gate = Object.fromEntries(['state', 'configure', 'refresh', 'stop', 'close', 'reviewQualificationAttempts', 'reviewBudget',
  'stratumV2Possession', 'stratumV2Status', 'exportDiagnostics'].map(key => [key, original[key].bind(original)]));
for (const key of Object.keys(original)) if (!['state', 'connect', 'stop', 'close'].includes(key)) delete original[key];
for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
const post = async (path, value) => { const response = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
  headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!response.ok) throw Error('share_recovery_rejected'); return response.json(); };
const setup = await post('/recovery-context', {});
let collectionId = null, prepared = false, used = false;
const prepare = document.createElement('button'), collect = document.createElement('button'), output = document.createElement('pre');
prepare.textContent = 'Prepare fresh recovery session'; collect.textContent = 'Collect current recovery and release'; collect.disabled = true; output.id = 'share-recovery-result';
prepare.addEventListener('click', async () => {
  prepare.disabled = true;
  const result = await prepareRecovery({ gate, configuration: setup.candidateConfiguration,
    record: value => post('/recovery-prepare', value),
    saveFailure: async value => {
      if (value.closed) { try { await post('/recovery-part', { collectionId: null, stage: 'closed', ticket: null, value: value.closed }); } catch { /* Errors still have an independent persistence path. */ } }
      await post('/recovery-part', { collectionId: null, stage: 'errors', ticket: null, value: value.errors });
      await post('/recovery-part', { collectionId: null, stage: 'finished', ticket: null, value: value.finished });
    } });
  prepared = result.prepared; collect.disabled = !prepared;
  output.textContent = prepared ? 'Connect Worker for the fresh candidate session, then collect.' : 'Preparation failed. Stop/Close were attempted; preserve this partial attempt.';
});
const run = createRecoveryCollection({ gate,
  begin: async () => { const challenge = await post('/recovery-challenge', {}); const binding = await gate.stratumV2Possession(); const reply = await post('/recovery-begin', { state: gate.state(), nonce: challenge.nonce, binding }); collectionId = reply.collectionId; return { ...reply, binding }; },
  beforeStage: (phase) => post('/stage-begin', { collectionId, phase }),
  afterStage: (phase, ticket) => ticket ? post('/stage-end', { collectionId, phase, token: ticket.token }) : Promise.resolve(),
  save: (stage, value, ticket) => post('/recovery-part', { collectionId, stage, ticket: ticket?.token ?? null, value }) });
collect.addEventListener('click', async () => {
  if (!prepared || used) return; used = true; collect.disabled = true;
  try { output.textContent = JSON.stringify(await run()); }
  catch { output.textContent = 'Partial collection retained. Stop/Close were attempted; no retry or device effect is admitted.'; }
});
document.body.append(prepare, collect, output);

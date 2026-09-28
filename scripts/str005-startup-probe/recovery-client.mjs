import { readRecoveryStatus } from './retained-status.mjs';
const categories = new Set(['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed']);
/** Independent current-state observations; no operation can issue new work authority. */
export function createCurrentRecovery({ gate, attemptId, campaignId, save, begin = async () => {}, readMs = 30000, cleanupMs = 150000 }) {
  let consumed = false;
  return async () => {
    if (consumed) throw Error('recovery_consumed'); consumed = true;
    const failures = [];
    async function collect(stage, operation, limit = readMs, persist = true) {
      let timer, active = true;
      try { await Promise.race([Promise.resolve().then(operation).then(value => active && persist ? save(stage, value) : undefined),
        new Promise((_, reject) => { timer = setTimeout(() => reject(Object.assign(Error('timeout'), { category: 'timeout' })), limit); })]); }
      catch (error) { failures.push({ stage, category: categories.has(error?.category) ? error.category : 'operation_failed' }); }
      finally { active = false; clearTimeout(timer); }
    }
    try {
      await collect('begin', begin, readMs, false);
      await collect('ledger', () => gate.reviewQualificationAttempts());
      await collect('original_budget', () => gate.reviewBudget(campaignId));
      await collect('diagnostics', async () => { await gate.exportDiagnostics(); return { exported: true }; });
      await collect('status', () => readRecoveryStatus(gate, attemptId, 'confirmed'));
    } finally {
      // Stop errors do not prevent an independent state read or physical serial release.
      await collect('stop', async () => { await gate.stop(); return { requested: true }; }, cleanupMs);
      await collect('state', async () => { await gate.refresh(); return gate.state(); });
      await collect('closed', async () => { await gate.close(); return gate.state(); }, cleanupMs);
    }
    await save('finished', { failures });
    return { current_recovery_complete: failures.length === 0, failures, qualification_complete: false };
  };
}
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const publicGate = window.workerAcceptance;
  const gate = Object.fromEntries(['state', 'refresh', 'stop', 'close', 'reviewQualificationAttempts', 'reviewBudget',
    'stratumV2Possession', 'stratumV2Status', 'exportDiagnostics'].map(name => [name, publicGate[name].bind(publicGate)]));
  for (const name of Object.keys(publicGate)) if (!['state', 'connect', 'stop', 'close'].includes(name)) delete publicGate[name];
  for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
  const post = async (path, value) => { const result = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!result.ok) throw Error('recovery_request_rejected'); return result.json(); };
  const context = await post('/recovery-context', {});
  const collect = createCurrentRecovery({ gate, attemptId: context.attemptId, campaignId: context.campaignId,
    begin: () => post('/recovery-begin', {}),
    save: (stage, value) => stage === 'diagnostics' ? Promise.resolve() : post('/recovery-part', { stage, value }) });
  const button = document.createElement('button'), output = document.createElement('pre');
  button.textContent = 'Collect current recovery and release'; output.id = 'current-recovery-result';
  button.addEventListener('click', async () => { button.disabled = true;
    try { output.textContent = JSON.stringify(await collect()); }
    catch { output.textContent = 'Partial current recovery retained. Stop and Close remain available.'; } });
  document.body.append(button, output);
}

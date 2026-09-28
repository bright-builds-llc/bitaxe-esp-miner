/** A separate restart-only page keeps one private baseline across exactly one software reset. */
export function createPreparationClient({ gate, post, readMs = 30000, cleanupMs = 150000 }) {
  let consumed = false;
  return async () => {
    if (consumed) throw Error('preparation_consumed'); consumed = true;
    const failures = [];
    async function bounded(operation, limit) { let timer; try { return await Promise.race([Promise.resolve().then(operation),
      new Promise((_, reject) => { timer = setTimeout(() => reject(Error('preparation_timeout')), limit); })]); } finally { clearTimeout(timer); } }
    async function collect(stage, operation, limit = readMs) {
      let active = true;
      try { await bounded(async () => { const value = await operation(); if (active) await post('/part', { stage, value }); }, limit); }
      catch { failures.push(stage); } finally { active = false; }
    }
    try {
      const context = await post('/preparation-context', {});
      await post('/begin', {});
      await collect('before_state', async () => { await gate.refresh(); return gate.state(); });
      await collect('before_ledger', () => gate.reviewQualificationAttempts());
      await collect('before_budget', () => gate.reviewBudget(context.campaignId));
      await bounded(() => gate.exportDiagnostics(), readMs);
      if (failures.length) throw Error('preparation_before_incomplete');
      const request = await post('/restart-claim', {});
      try { await bounded(() => gate.qualificationRestart(request), 30000); }
      catch { failures.push('restart'); }
      const evidence = gate.exportRestartEvidence();
      if (evidence) { try { if (!(await post('/restart-evidence', { evidence })).verified) failures.push('evidence'); } catch { failures.push('evidence'); } }
      else failures.push('evidence');
      await collect('after_state', async () => { await gate.refresh(); return gate.state(); });
      await collect('after_ledger', () => gate.reviewQualificationAttempts());
      await collect('after_budget', () => gate.reviewBudget(context.campaignId));
    } catch { failures.push('preparation'); }
    finally {
      // Even a rejected/ambiguous restart or missing observation cannot skip release.
      try { await bounded(() => gate.stop(), cleanupMs); } catch { failures.push('stop'); }
      await collect('closed', async () => { await gate.close(); return gate.state(); }, cleanupMs);
    }
    await post('/finished', { failures: [...new Set(failures)] });
    return { complete: failures.length === 0, failures, mining_started: false };
  };
}
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  const publicGate = window.workerAcceptance;
  const gate = Object.fromEntries(['state', 'refresh', 'stop', 'close', 'reviewQualificationAttempts', 'reviewBudget', 'exportDiagnostics',
    'qualificationRestart', 'exportRestartEvidence'].map(name => [name, publicGate[name].bind(publicGate)]));
  for (const name of Object.keys(publicGate)) if (!['state', 'connect', 'stop', 'close'].includes(name)) delete publicGate[name];
  for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
  const post = async (path, value) => { const response = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!response.ok) throw Error('preparation_rejected'); return response.json(); };
  const run = createPreparationClient({ gate, post }), output = document.createElement('pre'), button = document.createElement('button');
  button.textContent = 'Prepare idle startup with one software restart';
  button.addEventListener('click', async () => { button.disabled = true; try { output.textContent = JSON.stringify(await run()); }
    catch { output.textContent = 'Preparation incomplete. Preserve evidence; Stop and Close remain available.'; } });
  document.body.append(button, output);
}

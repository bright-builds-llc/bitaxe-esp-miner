import { runLoop } from './loop.mjs';
const original = window.workerAcceptance;
const gate = Object.fromEntries(['state', 'stop', 'close', 'configure', 'reviewQualificationAttempts', 'reviewBudget', 'stratumV2Possession', 'stratumV2Status']
  .map(key => [key, original[key].bind(original)]));
// Only native Connect, Stop and Close stay reachable; no effect capability is exposed.
for (const key of Object.keys(original)) if (!['state', 'connect', 'stop', 'close'].includes(key)) delete original[key];
for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
const post = async (path, value) => { const response = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
  headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!response.ok) throw Error('review_loop_rejected'); return response.json(); };
const prepare = document.createElement('button'), button = document.createElement('button'), output = document.createElement('pre');
prepare.textContent = 'Prepare candidate session'; button.textContent = 'Run bounded read-only review loop'; output.id = 'review-loop-result';
button.disabled = true;
// A before-phase baseline in this page admits the candidate phase that V2 reads require.
prepare.addEventListener('click', async () => {
  prepare.disabled = true;
  try {
    const before = gate.state();
    if (before.status !== 'ready' || !before.connected || before.running || !before.deviceLeaseInactive || !before.deviceBaselineConfirmed) throw Error('review_loop_not_idle');
    await gate.stop(); await gate.close(); await gate.configure(await post('/loop/candidate', {}));
    button.disabled = false; output.textContent = 'Connect Worker for the candidate session, then run the loop.';
  } catch { output.textContent = JSON.stringify({ error: 'review_loop_prepare_failed' }); }
});
button.addEventListener('click', async () => {
  button.disabled = true;
  try {
    const setup = await post('/loop/begin', { state: gate.state() });
    const result = await runLoop({ gate, campaignId: setup.campaignId, iterations: setup.iterations, record: row => post('/loop/row', row) });
    output.textContent = JSON.stringify(result);
  } catch { output.textContent = JSON.stringify({ error: 'review_loop_incomplete' }); }
  finally { try { await gate.close(); } catch { /* Close remains available as a native control. */ } }
});
document.body.append(prepare, button, output);

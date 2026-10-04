import { runLoop } from './loop.mjs';
const original = window.workerAcceptance;
const gate = Object.fromEntries(['state', 'close', 'reviewQualificationAttempts', 'reviewBudget', 'stratumV2Possession', 'stratumV2Status']
  .map(key => [key, original[key].bind(original)]));
// Only native Connect, Stop and Close stay reachable; no effect capability is exposed.
for (const key of Object.keys(original)) if (!['state', 'connect', 'stop', 'close'].includes(key)) delete original[key];
for (const id of ['prepare', 'load', 'start', 'arm-foreground', 'suppress', 'authorization-context', 'probe', 'configuration']) document.getElementById(id)?.remove();
const post = async (path, value) => { const response = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
  headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); if (!response.ok) throw Error('review_loop_rejected'); return response.json(); };
const button = document.createElement('button'), output = document.createElement('pre');
button.textContent = 'Run bounded read-only review loop'; output.id = 'review-loop-result';
button.addEventListener('click', async () => {
  button.disabled = true;
  try {
    const setup = await post('/loop/begin', { state: gate.state() });
    const result = await runLoop({ gate, campaignId: setup.campaignId, iterations: setup.iterations, record: row => post('/loop/row', row) });
    output.textContent = JSON.stringify(result);
  } catch { output.textContent = JSON.stringify({ error: 'review_loop_incomplete' }); }
  finally { try { await gate.close(); } catch { /* Close remains available as a native control. */ } }
});
document.body.append(button, output);

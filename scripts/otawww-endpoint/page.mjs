// Runs in the Gate tab. After Connect it reads the station endpoint once, hands it to the local supervisor,
// closes the Worker and leaves the page, which releases the serial port for the CLI run.
const gate = window.workerAcceptance;
const output = document.createElement('pre'); output.id = 'endpoint-handoff-result';
output.textContent = 'Connect the Worker to hand off its station endpoint.';
document.body.append(output);
const post = async (path, value) => {
  const response = await fetch(path, { method: 'POST', cache: 'no-store', redirect: 'error',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  if (!response.ok) throw Error('endpoint_handoff_rejected');
  return response.json();
};
let used = false;
async function handoff() {
  const { nonce } = await post('/endpoint-context', {});
  const requestedAtUnixMs = Date.now();
  const endpoint = await gate.observeStationEndpoint();
  const receivedAtUnixMs = Date.now();
  await post('/endpoint', { nonce, endpoint, requestedAtUnixMs, receivedAtUnixMs });
}
async function release() {
  try { await gate.stop(); } catch { /* An idle Worker has nothing to stop. */ }
  await gate.close();
  await post('/closed', { state: gate.state() });
}
const timer = setInterval(async () => {
  const state = gate.state();
  if (used || !state.connected || state.running || state.status !== 'ready') return;
  used = true; clearInterval(timer);
  let message;
  try { await handoff(); message = 'Endpoint handed off.'; } catch { message = 'Endpoint handoff failed.'; }
  try { await release(); output.textContent = `${message} Worker closed; leaving the page.`; location.replace('about:blank'); }
  catch { output.textContent = `${message} Worker close failed; preserve this attempt.`; }
}, 500);

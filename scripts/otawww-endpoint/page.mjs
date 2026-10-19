// Runs in the Gate tab. After Connect it reads the station endpoint once, hands it to the local supervisor,
// closes the Worker and leaves the page, which releases the serial port for the CLI run. A failed Connect or
// endpoint read is recorded and the page reloads into a fresh Gate document for the next Connect.
const WORKER_FILTER = { usbVendorId: 0x303a, usbProductId: 0x1001 };
const CATEGORY = /^[a-z][a-z0-9_]{0,63}$/u;
const GATE_FIELDS = ['status', 'failure', 'serialFailureCategory', 'admissionFailureStage', 'connected', 'serialOwnershipReleased'];
const TICK_MS = 500;
const RELOAD_SETTLE_MS = 1000;

/** Granted Worker ports and those Chrome reports attached. The Gate reuses a grant only when exactly one is attached. */
export function portCounts(ports) {
  const granted = ports.filter(port => {
    const info = port.getInfo();
    return info.usbVendorId === WORKER_FILTER.usbVendorId && info.usbProductId === WORKER_FILTER.usbProductId;
  });
  return { granted: granted.length, attached: granted.filter(port => port.connected !== false).length };
}

/**
 * Connect proceeds on a configured Gate when the Gate will reuse the single attached grant, or when there is no grant
 * and the chooser is the only way to grant one. Otherwise it would open Chrome's chooser, which a click cannot answer.
 */
export function connectHold(state, ports) {
  if (state.status === 'unconfigured') return 'gate_loading';
  if (ports.granted > 0 && ports.attached !== 1) return 'worker_port_not_attached';
  return undefined;
}

/** The Gate's public failure categories and flags, never its configuration or device values. */
export function gateFields(state) {
  return Object.fromEntries(GATE_FIELDS.filter(key => state[key] !== undefined).map(key => [key, state[key]]));
}

/** Only the closed categories a Gate or firmware error carries; arbitrary text is dropped. */
export function errorFields(error) {
  if (!error || typeof error !== 'object') return {};
  return Object.fromEntries(['category', 'rejection', 'message'].filter(key => typeof error[key] === 'string' && CATEGORY.test(error[key]))
    .map(key => [key, error[key]]));
}

/**
 * Drives one page load. `env` supplies the Gate, Web Serial, the page's published Gate state, the local server and
 * navigation, so the flow runs unchanged under test.
 */
export function startHandoffPage(env) {
  const { gate } = env;
  let ports = { granted: 0, attached: 0 }, maybeClick, ticking = false, busy = false;
  const foreground = () => ({ visible: env.visible(), focused: env.focused() });
  const refreshPorts = async () => {
    try { ports = portCounts(await env.serial.getPorts()); } catch { ports = { granted: 0, attached: 0 }; }
  };
  const release = async () => {
    try { await gate.stop(); } catch { /* An idle Worker has nothing to stop. */ }
    await gate.close();
  };
  async function leave(message) {
    try { await release(); } catch { env.show(`${message} Worker close failed; preserve this attempt.`); return; }
    try { await env.post('/closed', { state: gate.state() }); }
    catch { env.show(`${message} The closed Worker was not recorded; leaving the page.`); env.leave(); return; }
    env.show(`${message} Worker closed; leaving the page.`); env.leave();
  }
  async function failed(step, state, error) {
    const attempt = { step, gate: gateFields(state), error: errorFields(error), ports: maybeClick?.ports ?? ports, foreground: maybeClick?.foreground ?? foreground() };
    let maybeRetry;
    try { maybeRetry = await env.post('/attempt', attempt); } catch { maybeRetry = undefined; }
    if (!maybeRetry?.retry) return leave(`Endpoint handoff failed at ${step}; no attempts remain.`);
    try { await release(); } catch { env.show(`Attempt ${maybeRetry.failed} failed at ${step}. Worker close failed; preserve this attempt.`); return; }
    env.show(`Attempt ${maybeRetry.failed} of ${maybeRetry.max} failed at ${step}; reloading for the next Connect.`);
    env.after(RELOAD_SETTLE_MS, env.reload);
  }
  async function handoff() {
    let step = 'endpoint_context';
    try {
      const { nonce } = await env.post('/endpoint-context', {});
      step = 'observe';
      const requestedAtUnixMs = env.now();
      const endpoint = await gate.observeStationEndpoint();
      const receivedAtUnixMs = env.now();
      step = 'submit';
      await env.post('/endpoint', { nonce, endpoint, requestedAtUnixMs, receivedAtUnixMs });
    } catch (error) { return failed(step, gate.state(), error); }
    return leave('Endpoint handed off.');
  }
  async function tick() {
    if (ticking || busy) return;
    ticking = true;
    try {
      await refreshPorts();
      const state = gate.state();
      // Connect marks the Gate ready before its own status read settles and publishes only afterwards; an endpoint
      // read during that read finds the one control slot taken and loses the session.
      const published = env.publishedState();
      if (state.connected && !state.running && state.status === 'ready' && published?.status === 'ready') { busy = true; await handoff(); return; }
      if (maybeClick && !state.connected && ['failed', 'disconnected'].includes(state.status)) { busy = true; await failed('connect', state); }
    } finally { ticking = false; }
  }
  env.onConnectClick(event => {
    const maybeHold = busy ? 'handoff_in_progress' : connectHold(gate.state(), ports);
    if (maybeHold) {
      event.preventDefault(); event.stopImmediatePropagation();
      env.show(`Connect held (${maybeHold}); wait a moment and click Connect again.`);
      return;
    }
    maybeClick = { ports, foreground: foreground() };
  });
  env.onPortsChanged(() => { void refreshPorts(); });
  env.every(TICK_MS, () => { void tick(); });
  void refreshPorts();
  void env.get('/attempts').then(({ failed, max }) => {
    env.show(failed === 0 ? 'Connect the Worker to hand off its station endpoint.' : `Attempt ${failed} of ${max} failed; click Connect for the next attempt.`);
  }, () => env.show('Connect the Worker to hand off its station endpoint.'));
  return { tick };
}

function browserEnvironment() {
  const output = document.createElement('pre'); output.id = 'endpoint-handoff-result';
  document.body.append(output);
  const request = async (path, init) => {
    const response = await fetch(path, { cache: 'no-store', redirect: 'error', ...init });
    if (!response.ok) throw Error('endpoint_handoff_rejected');
    return response.json();
  };
  return {
    gate: window.workerAcceptance,
    serial: navigator.serial,
    show: text => { output.textContent = text; },
    visible: () => document.visibilityState === 'visible',
    focused: () => document.hasFocus(),
    publishedState: () => { try { return JSON.parse(document.querySelector('#state')?.textContent ?? ''); } catch { return undefined; } },
    // Capture runs before the Gate's own Connect listener, so a held click never reaches it.
    onConnectClick: listener => window.addEventListener('click', event => {
      if (event.target instanceof Element && event.target.closest('#connect')) listener(event);
    }, true),
    onPortsChanged: listener => { for (const type of ['connect', 'disconnect']) navigator.serial.addEventListener(type, listener); },
    every: (milliseconds, listener) => setInterval(listener, milliseconds),
    after: (milliseconds, listener) => setTimeout(listener, milliseconds),
    now: () => Date.now(),
    get: path => request(path, { method: 'GET' }),
    post: (path, value) => request(path, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value) }),
    reload: () => location.reload(),
    leave: () => location.replace('about:blank'),
  };
}

if (typeof window !== 'undefined' && window.workerAcceptance) startHandoffPage(browserEnvironment());

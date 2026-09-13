// This module never persists the private endpoint or raw telemetry frames.
export function createCadenceSupervisor(operations) {
  const { worker, flush, fetch: request, now = Date.now, monotonic = () => performance.now(),
    wait = ms => new Promise(resolve => setTimeout(resolve, ms)), maybeOnState = () => {},
    every = callback => { const timer = setInterval(callback, 1000); return () => clearInterval(timer); } } = operations;
  let stage = "ready", started = false, observerUsed = false, observerStarted = false, observerClosed = false, observerStopping = false;
  let maybeCancelWatcher, checkingObserver = false, maybeFailure, maybeCleanup, maybeRun, maybeResume;
  let maybeFaultAtUnixMs, maybeFaultObservedAtMs, maybeCompletion;
  const state = () => ({ schema: "worker-cadence-run-v1", stage, started, observerClosed,
    ...(maybeFaultAtUnixMs === undefined ? {} : { faultObservedAtUnixMs: maybeFaultAtUnixMs }),
    ...(maybeFailure ? { failure: "cadence_qualification_failed" } : {}),
    ...(maybeCompletion ? { result: maybeCompletion.result } : {}) });
  function publish(value) { stage = value; maybeOnState(state()); }
  function check() { if (maybeFailure) throw maybeFailure; }
  async function bounded(action, milliseconds = 30000) {
    let timer;
    try {
      return await Promise.race([Promise.resolve().then(action), new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error("cadence_operation_timeout")), milliseconds);
      })]);
    } finally { clearTimeout(timer); }
  }
  async function perform(action, milliseconds) {
    check(); const result = await bounded(action, milliseconds); check(); return result;
  }
  async function http(route, maybeInput) {
    const controller = new AbortController(); const timer = setTimeout(() => controller.abort(), 15000);
    try {
      const response = await request(route, { method: maybeInput === undefined ? "GET" : "POST", cache: "no-store", signal: controller.signal,
        ...(maybeInput === undefined ? {} : { headers: { "Content-Type": "application/json" }, body: JSON.stringify(maybeInput) }) });
      if (!response.ok) throw new Error("cadence_supervisor_rejected");
      return await response.json();
    } finally { clearTimeout(timer); }
  }
  async function readStatus() {
    const value = await http("/cadence/status");
    if (!value || !value.observer || typeof value.observer.alive !== "boolean" || typeof value.observer.connected !== "boolean") throw new Error("cadence_observer_unavailable");
    return value;
  }
  async function requireObserver() {
    check(); const value = await readStatus();
    if (!value.observer.connected || !value.observer.alive || value.observer.failed) throw new Error("cadence_observer_failed");
    return value;
  }
  function cleanup() {
    return maybeCleanup ??= (async () => {
      maybeCancelWatcher?.(); maybeCancelWatcher = undefined;
      const settled = await Promise.allSettled([
        (async () => {
          let maybeStopFailure;
          try { if (worker().state().connected) await bounded(() => worker().stop(), 150000); }
          catch (error) { maybeStopFailure = error; }
          try { await bounded(() => worker().close(), 150000); }
          catch (error) { throw maybeStopFailure ? new AggregateError([maybeStopFailure, error], "Worker cleanup failed") : error; }
          if (maybeStopFailure) throw maybeStopFailure;
        })(),
        stopObserver(),
      ]);
      return settled.filter(value => value.status === "rejected").map(value => value.reason);
    })();
  }
  function abort(error) {
    maybeFailure ??= error; publish("failed");
    if (maybeResume) { maybeResume.reject(maybeFailure); maybeResume = undefined; }
    void cleanup();
  }
  async function startObserver() {
    if (observerUsed || observerStopping) throw new Error("cadence_observer_already_used");
    observerUsed = true;
    await bounded(() => worker().submitBudgetReview()); await flush();
    const { nonce } = await http("/cadence/observer-context", {});
    const requestedAtUnixMs = now(); const endpoint = await bounded(() => worker().cadenceEndpoint(), 5000);
    const receivedAtUnixMs = now(); await flush();
    const result = await http("/cadence/observer/start", { nonce, endpoint, requestedAtUnixMs, receivedAtUnixMs, state: worker().state() });
    if (result?.observer_connected !== true) throw new Error("cadence_observer_start_receipt");
    observerStarted = true;
    maybeCancelWatcher = every(() => {
      if (checkingObserver || observerStopping || maybeFailure) return;
      checkingObserver = true;
      requireObserver().catch(error => { if (!observerStopping) abort(error); }).finally(() => { checkingObserver = false; });
    });
    return result;
  }
  async function stopObserver() {
    observerStopping = true; maybeCancelWatcher?.(); maybeCancelWatcher = undefined;
    const result = await http("/cadence/observer/stop", {});
    if (result?.cleanupComplete !== true) throw new Error("cadence_observer_cleanup");
    if (observerStarted && (result?.reason !== "requested" || result.closed !== true || result.cleanupComplete !== true || result.exitCode !== 0)) throw new Error("cadence_observer_cleanup");
    observerClosed = true; return result;
  }
  async function arm(phase) {
    await requireObserver(); const receipt = await bounded(() => worker().cadenceArm(phase)); await flush();
    await http("/cadence/phase/start", { arm: receipt }); return receipt;
  }
  async function collect(phase) {
    check();
    if (phase !== "mining" || !observerClosed) await requireObserver();
    const review = await bounded(() => worker().cadenceReview()); await flush();
    return http("/cadence/phase/finish", { phase, review });
  }
  async function waitFor(deadline, live) {
    let previous = monotonic();
    while (previous < deadline) {
      check(); if (live) await requireObserver();
      await wait(Math.min(1000, deadline - previous));
      const next = monotonic(); if (!Number.isFinite(next) || next < previous) throw new Error("cadence_clock");
      previous = next;
    }
    check();
  }
  async function runPhase(phase) {
    await arm(phase);
    const began = monotonic();
    if (phase === "usb") await bounded(() => worker().cadenceUsbPhase(), 90000);
    // The wait starts after the arm acknowledgement; the device review proves completion.
    await waitFor(began + 62500, true); return collect(phase);
  }
  async function waitForCut(startedAtUnixMs) {
    const deadline = monotonic() + 180000;
    while (monotonic() < deadline) {
      check(); await flush(); const observed = await requireObserver(); const local = worker().state();
      const fault = observed.faultObservedAtUnixMs;
      if (fault !== undefined && fault !== null) {
        if (!Number.isSafeInteger(fault) || fault < startedAtUnixMs || fault > now() ||
          !local.heartbeatSuppressed || local.cadence?.suppressionRequested !== true ||
          (maybeFaultAtUnixMs !== undefined && maybeFaultAtUnixMs !== fault)) throw new Error("cadence_fault_evidence");
        maybeFaultAtUnixMs ??= fault; maybeFaultObservedAtMs ??= monotonic();
        const stopped = local.running === false && local.connected === false && local.serialOwnershipReleased === true;
        if (stopped && now() >= fault + 5000 && monotonic() >= maybeFaultObservedAtMs + 5000) return;
      }
      if (local.failure && !(fault && local.failure === "window_control_failed" && local.serialFailureCategory === "liveness_lost")) throw new Error("cadence_worker_failed");
      await waitFor(Math.min(deadline, monotonic() + 1000), true);
    }
    throw new Error("cadence_fault_timeout");
  }
  async function recover() {
    publish("cooling_wait");
    if (maybeFaultAtUnixMs === undefined || maybeFaultObservedAtMs === undefined) throw new Error("cadence_fault_missing");
    await waitFor(maybeFaultObservedAtMs + 145000, false);
    if (now() < maybeFaultAtUnixMs + 145000) throw new Error("cadence_clock");
    await bounded(() => worker().close(), 150000); await flush();
    const resumed = new Promise((resolve, reject) => { maybeResume = { resolve, reject }; });
    publish("awaiting_reconnect");
    await resumed;
    publish("reviewing"); await collect("mining");
    const completion = await bounded(() => worker().submitAttemptCompletion(), 150000);
    if (completion?.result !== "passed" || completion.cleanup_confirmed !== true) throw new Error("cadence_result_unverified");
    maybeCompletion = { result: completion.result }; publish("complete"); return completion;
  }
  function resumeQualification() {
    if (stage !== "awaiting_reconnect" || !maybeResume) return Promise.reject(new Error("cadence_reconnect_not_armed"));
    const resumed = maybeResume; maybeResume = undefined; publish("reconnecting");
    // Invoke connect in the original trusted click task: no await or promise callback before it.
    try { Promise.resolve(worker().connect()).then(resumed.resolve, resumed.reject); }
    catch (error) { resumed.reject(error); }
    return maybeRun;
  }
  function runQualification() {
    if (started || observerUsed) return Promise.reject(new Error("cadence_run_consumed"));
    started = true;
    maybeRun = (async () => {
      try {
        const initial = worker().state();
        if (!initial.connected || initial.running || !initial.deviceLeaseInactive || initial.cadence?.enabled !== true) throw new Error("cadence_run_admission");
        publish("cooling_review"); await perform(() => worker().submitCoolingReview(), 150000); await flush(); check();
        publish("observer_start"); await startObserver(); check();
        publish("idle"); await runPhase("idle"); check();
        publish("usb"); await runPhase("usb"); check();
        publish("mining_arm"); await arm("mining");
        await perform(() => worker().submitBudgetReview()); await perform(() => worker().prepareStartAuthorization());
        await perform(() => worker().loadSignedWindow()); check();
        const startedAtUnixMs = now(); publish("mining"); await perform(() => worker().startWindow());
        await waitForCut(startedAtUnixMs); publish("observer_stop"); await stopObserver();
        return await recover();
      } catch (error) {
        maybeFailure ??= error; publish("failed"); const errors = await cleanup();
        if (errors.length) throw new AggregateError([maybeFailure, ...errors], "Cadence qualification failed; cleanup requires review");
        throw maybeFailure;
      }
    })();
    return maybeRun;
  }
  return { startObserver, runIdle: () => runPhase("idle"), runUsb: () => runPhase("usb"), armMining: () => arm("mining"),
    collectMining: () => collect("mining"), stopObserver, runQualification, resumeQualification, state };
}

if (typeof window !== "undefined") {
  const reconnect = document.createElement("button"); reconnect.textContent = "Reconnect to verify restoration"; reconnect.hidden = true;
  reconnect.id = "cadence-reconnect"; document.body.append(reconnect);
  const supervisor = createCadenceSupervisor({ worker: () => window.workerAcceptance, flush: () => window.recoverySupervisor.flush(),
    fetch: (...args) => fetch(...args), maybeOnState(value) {
      reconnect.hidden = value.stage !== "awaiting_reconnect";
      const note = document.querySelector("#supervisor-status");
      if (note) note.textContent = value.stage === "awaiting_reconnect"
        ? "Observer closed; recovery wait complete. Reconnect to verify restoration."
        : `Cadence qualification: ${value.stage}.`;
    } });
  reconnect.addEventListener("click", () => {
    supervisor.resumeQualification().catch(() => {
      document.querySelector("#supervisor-status").textContent = "Cadence restoration verification failed; inspect the retained result.";
    });
  });
  window.cadenceSupervisor = supervisor;
}

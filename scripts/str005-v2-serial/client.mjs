const fail = code => { throw new Error(code); };
const token = value => typeof value === "string" && /^[A-Za-z0-9_-]{21}[AQgw]$/u.test(value);
/** A timeout stops this coordinator; it never retries a possibly consumed command. */
async function bounded(promise, milliseconds, code = "v2_client_timeout") {
  if (!(milliseconds > 0)) fail(code);
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(code)), milliseconds); })]); }
  finally { clearTimeout(timer); }
}
function baseline(state, scope, after = false, completed = false) {
  const preserved = state?.preservation, recovered = state?.authorizationRecovery;
  const authorizationMatches = preserved?.authorization_high_water_match ||
    (after && scope === "share" && recovered?.matched === true && recovered.generation === state.qualification?.generation);
  if (!state || state.status !== "ready" || !state.connected || state.running || state.failure ||
    !state.deviceLeaseInactive || !state.deviceBaselineConfirmed || !preserved?.device_identity_match ||
    !preserved.settings_match || !authorizationMatches || preserved.mine_on_boot ||
    (scope === "channel" && state.renewalsConfirmed !== 0) ||
    (after && scope === "share" && completed && (recovered?.matched !== true || recovered.generation !== state.qualification?.generation))) fail("v2_client_baseline");
}
function classification(error) {
  if (error?.rejection === "restoration_pending") return "restoration_pending";
  const categories = { timeout: "timeout", closed: "closed", restoration_pending: "restoration_pending", shape: "shape", session: "shape" };
  const messages = { v2_client_timeout: "timeout", v2_client_horizon: "observation_horizon", v2_client_terminal: "unexpected_terminal" };
  return categories[error?.category] ?? messages[error?.message] ?? "operation_failed";
}
/** Browser-only coordinator. Injected operations exercise the same order in composed software tests. */
export function createV2Coordinator(operations) {
  const { gate, request, published, now, sleep } = operations;
  const notice = operations.notice ?? (() => {});
  let queue = Promise.resolve(), recordFailed = false, running = false, started = false, completed = false;
  let phase = "configuration", maybeScope, maybeBinding, maybeAttemptId, maybeFailure = null;
  let channelDelivered = false, connectionRecorded = false, maybeRestoreAfter = null, maybeFixtureRequestAt = null;
  const call = (path, input, method = "POST") => bounded(request(path, input, method), 30000);
  function observe(state) {
    if (!state.expectedFirmwareSourceCommit) return;
    queue = queue.then(() => call("/record", { state })).catch(() => {
      recordFailed = true; notice("Evidence recording failed. Retain this page and complete cleanup.");
    });
  }
  async function flush() { await Promise.resolve(); await queue; if (recordFailed) fail("v2_client_journal"); }
  async function scopeState(allowFailure = false) {
    const value = await call("/supervisor-state", undefined, "GET");
    if (!value || Object.keys(value).length !== 3 || !["channel", "share"].includes(value.scope) || !["before", "candidate"].includes(value.phase) || typeof value.failed !== "boolean" ||
      (maybeScope && value.scope !== maybeScope) || (!allowFailure && value.failed)) fail("v2_client_scope");
    maybeScope = value.scope; return value;
  }
  async function refresh(after = false) {
    await bounded(gate.refresh(), 30000); await flush(); baseline(published(), maybeScope, after, completed);
  }
  async function accounting(stage) {
    phase = "accounting";
    if (!["before-install", "before", "after"].includes(stage)) fail("v2_client_stage");
    await scopeState(stage === "after");
    const { campaignId } = await call("/accounting-context", {});
    if (!token(campaignId)) fail("v2_client_shape");
    const ledger = await bounded(gate.reviewQualificationAttempts(), 30000);
    const original_budget = await bounded(gate.reviewBudget(campaignId), 30000);
    await refresh(stage === "after");
    const result = await call("/accounting", { stage, ledger, original_budget, state: published() });
    if (result?.accounting_saved !== true || result.stage !== stage) fail("v2_client_shape");
    return { accounting_saved: true, stage };
  }
  async function recordStatus(status) {
    await call("/device/record", { status });
    if (status.connection !== null && !connectionRecorded) {
      await call("/protocol/connection", { status }); connectionRecorded = true;
    }
    if (status.record?.firstFailure || status.record?.secondaryFailures?.length) fail("v2_client_terminal");
  }
  async function idle(binding) { return bounded(gate.stratumV2Status(maybeScope, null, binding), 30000); }
  async function fixture(status) {
    phase = "fixture";
    maybeFixtureRequestAt = now();
    const result = await call("/fixture/start", { status });
    if (result?.fixture_ready !== true || !token(result.attemptId)) fail("v2_client_shape");
    maybeAttemptId = result.attemptId;
  }
  async function channel() {
    await accounting("before"); phase = "admission";
    maybeBinding = await bounded(gate.stratumV2Possession(), 30000);
    await fixture(await idle(maybeBinding));
    phase = "admission"; const fresh = await idle(maybeBinding);
    const input = await call("/start/claim", { status: fresh });
    if (input?.attemptId !== maybeAttemptId) fail("v2_client_shape");
    phase = "start"; const deadline = now() + 125000;
    let status = await bounded(gate.stratumV2ChannelStart(input, maybeBinding), Math.min(30000, deadline - now()), "v2_client_horizon");
    channelDelivered = true;
    for (;;) {
      if (now() > deadline) fail("v2_client_horizon");
      await bounded(recordStatus(status), deadline - now(), "v2_client_horizon");
      if (status.state === "terminal") break;
      phase = "poll"; await sleep(250);
      status = await bounded(gate.stratumV2Status("channel", maybeAttemptId, maybeBinding), Math.min(30000, deadline - now()), "v2_client_horizon");
    }
    if (status.record?.outcome !== "accepted") fail("v2_client_terminal");
    phase = "restoration";
    await call("/protocol/complete", {});
    await bounded(gate.stop(), 150000); await flush(); await bounded(gate.close(), 150000); await flush();
  }
  async function share() {
    await accounting("before"); phase = "admission";
    await bounded(gate.submitCoolingReview(), 150000); await flush();
    await bounded(gate.submitBudgetReview(), 30000); await flush();
    phase = "observer"; const { nonce } = await call("/observer/context", {});
    if (!token(nonce)) fail("v2_client_shape");
    const endpoint = await bounded(gate.stratumV2TelemetryEndpoint(), 30000);
    maybeBinding = endpoint.controlSessionBindingSha256;
    if (typeof maybeBinding !== "string") fail("v2_client_shape");
    const observer = await call("/observer/start", { nonce, endpoint, state: published() });
    if (observer?.observer_connected !== true) fail("v2_client_shape");
    await fixture(await idle(maybeBinding));
    phase = "admission"; const fresh = await idle(maybeBinding);
    await call("/start/network", { status: fresh, controlSessionBindingSha256: maybeBinding });
    const signed = await bounded(gate.prepareStartAuthorization(), 30000);
    if (signed?.controlSessionBindingSha256 !== maybeBinding) fail("v2_client_shape");
    await bounded(gate.loadSignedWindow(), 30000); await flush();
    const finalFreshIdle = await idle(maybeBinding); // Refresh admission without another possession proof.
    phase = "start"; const deadline = now() + 240000;
    const startInvokedAtPageMs = now();
    if (maybeFixtureRequestAt === null || startInvokedAtPageMs < maybeFixtureRequestAt || startInvokedAtPageMs - maybeFixtureRequestAt > 10000)
      fail("v2_client_fixture_deadline");
    await bounded(gate.startWindow(), 30000);
    const startRepliedAtPageMs = now();
    if (startRepliedAtPageMs < startInvokedAtPageMs || startRepliedAtPageMs - startInvokedAtPageMs > 30000) fail("v2_client_timeout");
    await flush();
    const observed = await call("/share/start-observed", { status: finalFreshIdle, timing: {
      fixtureRequestAtPageMs: maybeFixtureRequestAt, startInvokedAtPageMs, startRepliedAtPageMs } });
    if (observed?.start_observed !== true) fail("v2_client_shape");
    for (;;) {
      phase = "poll";
      if (now() >= deadline) fail("v2_client_horizon");
      const status = await bounded(gate.stratumV2Status("share", maybeAttemptId, maybeBinding), Math.min(30000, deadline - now()), "v2_client_horizon");
      await bounded(recordStatus(status), deadline - now(), "v2_client_horizon");
      const selection = await call("/share/select", { status });
      if (selection?.eligible === true) {
        if (!Number.isSafeInteger(selection.submissionSequence) || typeof selection.selectedDeviceAckSha256 !== "string" || !/^[0-9a-f]{64}$/u.test(selection.selectedDeviceAckSha256)) fail("v2_client_shape");
        await cutHeartbeats(selection.selectedDeviceAckSha256); break;
      }
      if (selection?.eligible !== false || status.state === "terminal" || !published().running) fail("v2_client_terminal");
      await sleep(500);
    }
    await call("/protocol/complete", {});
    phase = "restoration"; await bounded(gate.close(), 150000); await flush();
  }
  async function cutHeartbeats(selectedDeviceAckSha256) {
    phase = "fault";
    const { nonce } = await call("/fault/claim", { selectedDeviceAckSha256 });
    if (!token(nonce)) fail("v2_client_shape");
    const headroom = await bounded(gate.suppressHeartbeats(), 30000); await flush();
    await call("/fault/confirm", { nonce, headroom, state: published() });
    // This clock belongs to this same retained page; server clocks are never subtracted from it.
    maybeRestoreAfter = now() + 145000;
    await sleep(5100); await call("/observer/finish", {});
  }
  async function cleanup() {
    if (maybeScope === "channel" && channelDelivered && maybeBinding && maybeAttemptId) {
      try { const status = await bounded(gate.stratumV2ChannelCancel(maybeAttemptId, maybeBinding), 30000); await call("/device/record", { status }); }
      catch { notice("Cancellation evidence is incomplete; retain the original failure."); }
    }
    try { await bounded(gate.stop(), 150000); }
    catch { notice("Restoration is unconfirmed; retain the original failure."); }
    try { await bounded(gate.close(), 150000); }
    catch { notice("Host cleanup is incomplete; retain the original failure."); }
    try { await flush(); }
    catch { notice("Final evidence recording is incomplete; retain the original failure."); }
  }
  async function report(error) {
    if (maybeFailure === null) {
      maybeFailure = { phase: recordFailed ? "journal" : phase, code: classification(error) };
      try { await call("/client-failure", maybeFailure); }
      catch { notice("Failure recording is incomplete. Preserve the supervisor artifacts."); }
    }
    return new Error(`v2_client_${maybeFailure.code}`);
  }
  async function guarded(operation) {
    try { return await operation(); } catch (error) { throw await report(error); }
  }
  const supervisor = {
    flush,
    recordAccounting: stage => guarded(() => accounting(stage)),
    configureCandidate: () => guarded(async () => {
      phase = "configuration"; await flush();
      const state = published();
      if (state.status !== "closed" || !state.serialOwnershipReleased) fail("v2_client_release");
      await gate.configure(await call("/candidate-context", {})); await flush(); return { configured: true };
    }),
    recordCycle: index => guarded(async () => {
      phase = "probe"; if (running || started || !Number.isInteger(index) || index < 1 || index > 4) fail("v2_client_cycle");
      await scopeState(); await refresh();
      const binding = await bounded(gate.stratumV2Possession(), 30000), status = await idle(binding);
      const { probe_nonce: nonce } = await call("/probe/claim", { index, status });
      if (!token(nonce)) fail("v2_client_shape");
      const probe = await bounded(gate.probe(), 30000); await flush();
      const after = await idle(binding);
      await call("/probe/complete", { index, nonce, probe, status: after, state: published() });
      const result = await call("/cycle", { index });
      if (result?.cycle_verified !== true || result.index !== index) fail("v2_client_shape");
      return { cycle_verified: true, index };
    }),
    async run() {
      if (running || started) fail("v2_client_start_consumed");
      running = true; started = true;
      try {
        const info = await scopeState(); if (info.phase !== "candidate") fail("v2_client_candidate_required");
        if (maybeScope === "channel") await channel(); else await share();
        completed = true;
        const remaining = maybeRestoreAfter === null ? 0 : Math.max(0, Math.ceil(maybeRestoreAfter - now()));
        notice("Phase recorded. Keep this page; reconnect through a native gesture after the required wait.");
        return { phase_complete: true, scope: maybeScope, hardware_qualified: false, requires_native_reconnect: true, restoration_wait_remaining_ms: remaining };
      } catch (error) {
        const failure = await report(error); await cleanup(); throw failure;
      } finally { running = false; }
    },
    restoreAndRecord: () => guarded(async () => {
      phase = "restoration"; await scopeState(true);
      if (maybeRestoreAfter !== null && now() < maybeRestoreAfter) fail("v2_client_restoration_wait");
      if (maybeScope === "share" && completed && maybeRestoreAfter === null) fail("v2_client_restoration_origin");
      await refresh(true);
      const { attemptId } = await call("/restoration/context", {});
      if (!token(attemptId) || (maybeAttemptId && attemptId !== maybeAttemptId)) fail("v2_client_shape");
      const binding = await bounded(gate.stratumV2Possession(), 30000);
      const status = await bounded(gate.stratumV2Status(maybeScope, attemptId, binding), 30000);
      await call("/device/record", { status }); await refresh(true);
      await call("/restoration", { status, state: published() });
      await accounting("after"); return { restoration_recorded: true, accounting_recorded: true, hardware_qualified: false };
    }),
  };
  return { supervisor, observe };
}

/** Only the qualified Gate page owns Serial permission and device control. */
export function installV2Coordinator(page, document, fetchImpl = fetch) {
  for (const id of ["prepare", "load", "start", "arm-foreground", "suppress", "authorization-context"]) document.getElementById(id)?.remove();
  const notice = document.createElement("p"); notice.id = "v2-supervisor-status";
  notice.textContent = "V2 qualification. Retain this page and its private baseline throughout this scope.";
  document.body.append(notice);
  const published = () => {
    const text = document.querySelector("#state")?.textContent;
    if (!text) fail("v2_client_state_missing"); return JSON.parse(text);
  };
  const request = async (path, input, method) => {
    const response = await fetchImpl(path, { method, cache: "no-store", redirect: "error", headers: { "Content-Type": "application/json" },
      ...(method === "POST" ? { body: JSON.stringify(input) } : {}) });
    const value = await response.json(); if (!response.ok) fail("v2_client_supervisor_rejected"); return value;
  };
  const coordinator = createV2Coordinator({ gate: page.workerAcceptance, request, published, now: () => performance.now(),
    sleep: ms => new Promise(resolveSleep => setTimeout(resolveSleep, ms)), notice: text => { notice.textContent = text; } });
  const output = document.querySelector("#state");
  if (output) new MutationObserver(() => coordinator.observe(published())).observe(output, { childList: true, subtree: true, characterData: true });
  page.v2Supervisor = coordinator.supervisor;
  return coordinator;
}
if (typeof window !== "undefined" && typeof document !== "undefined") installV2Coordinator(window, document);

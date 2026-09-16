// The qualified Gate page owns USB; this coordinator has no work/signing routes.
for (const id of ["prepare", "load", "start", "arm-foreground", "suppress"]) document.getElementById(id)?.remove();
const notice = document.createElement("p"); notice.id = "noise-supervisor-status";
notice.textContent = "No-mining Noise qualification. Preserve this page throughout all updates.";
document.body.append(notice);
let queue = Promise.resolve(), recordFailed = false, running = false, started = false;
let operationPhase = "configuration";
async function post(path, input) {
  const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" },
    cache: "no-store", body: JSON.stringify(input) });
  const value = await response.json();
  if (!response.ok) throw new Error(value.error ?? "noise_supervisor_rejected");
  return value;
}
function published() {
  const text = document.querySelector("#state")?.textContent;
  if (!text) throw new Error("noise_published_state_missing");
  return JSON.parse(text);
}
function enqueue(state) {
  if (!state.expectedFirmwareSourceCommit) return;
  queue = queue.then(() => post("/record", { state })).catch(() => {
    recordFailed = true; notice.textContent = "Evidence recording failed. Close the Worker and retain this page.";
  });
}
const output = document.querySelector("#state");
if (output) new MutationObserver(() => enqueue(published())).observe(output, { childList: true, subtree: true, characterData: true });
async function flush() { await Promise.resolve(); await queue; if (recordFailed) throw new Error("noise_journal_failed"); }
function baseline(state) {
  if (state.status !== "ready" || !state.connected || state.running || state.failure || state.renewalsConfirmed !== 0 ||
    !state.deviceLeaseInactive || !state.deviceBaselineConfirmed || !state.preservation?.device_identity_match ||
    !state.preservation.settings_match || !state.preservation.authorization_high_water_match || state.preservation.mine_on_boot)
    throw new Error("noise_baseline_required");
}
async function refresh() { await window.workerAcceptance.refresh(); await flush(); baseline(published()); }
async function recordAccounting(stage) {
  operationPhase = stage === "after" ? "accounting_after" : "accounting_before";
  const { campaignId } = await post("/accounting-context", {});
  const ledger = await window.workerAcceptance.reviewQualificationAttempts();
  const original_budget = await window.workerAcceptance.reviewBudget(campaignId);
  await refresh();
  return post("/accounting", { stage, ledger, original_budget, state: published() });
}
async function configureCandidate() {
  operationPhase = "configuration";
  await flush();
  const state = published();
  if (state.status !== "closed" || !state.serialOwnershipReleased) throw new Error("noise_release_required");
  const config = await post("/candidate-context", {});
  await window.workerAcceptance.configure(config); await flush();
  return { configured: true };
}
async function recordCycle(index) {
  operationPhase = "probe";
  await refresh();
  const binding = await window.workerAcceptance.noiseDiagnosticPossession();
  const status = await window.workerAcceptance.noiseDiagnosticStatus(null, binding);
  const { probe_nonce: nonce } = await post("/probe/claim", { index, status });
  const probe = await window.workerAcceptance.probe(); await flush();
  const after = await window.workerAcceptance.noiseDiagnosticStatus(null, binding);
  await post("/probe/complete", { index, nonce, probe, status: after, state: published() });
  return post("/cycle", { index });
}
const sleep = (ms) => new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
async function run() {
  if (running || started) throw new Error("noise_start_consumed");
  running = true; started = true;
  let maybeBinding, maybeId;
  const beginning = performance.now();
  try {
    await recordAccounting("before");
    maybeBinding = await window.workerAcceptance.noiseDiagnosticPossession();
    const initial = await window.workerAcceptance.noiseDiagnosticStatus(null, maybeBinding);
    operationPhase = "fixture_ready";
    await post("/fixture/start", { status: initial });
    maybeBinding = await window.workerAcceptance.noiseDiagnosticPossession();
    const fresh = await window.workerAcceptance.noiseDiagnosticStatus(null, maybeBinding);
    operationPhase = "start_admission";
    const input = await post("/start/claim", { status: fresh });
    maybeId = input.attemptId;
    operationPhase = "start_request";
    const sentAt = performance.now();
    let status = await window.workerAcceptance.noiseDiagnosticStart(input, maybeBinding);
    await post("/noise/record", { status });
    const deadline = sentAt + 125000;
    while (status.state !== "terminal") {
      if (performance.now() >= deadline) throw new Error("noise_observation_horizon");
      await sleep(250);
      operationPhase = "status_poll";
      status = await window.workerAcceptance.noiseDiagnosticStatus(maybeId, maybeBinding);
      await post("/noise/record", { status });
    }
    if (status.job?.terminal?.outcome !== "accepted") throw new Error("noise_device_failed");
    await window.workerAcceptance.refresh(); await flush();
    operationPhase = "completion";
    await post("/diagnostic/complete", {});
    await window.workerAcceptance.stop(); await flush();
    await window.workerAcceptance.close(); await flush();
    notice.textContent = "Exchange recorded and Worker closed. Reconnect this same page, then record restoration and accounting.";
    return { exchange_recorded: true, hardware_qualified: false };
  } catch (error) {
    const known = { timeout: "serial_timeout", closed: "serial_closed", restoration_pending: "restoration_pending", shape: "shape", session: "binding_mismatch" };
    const messages = { noise_observation_horizon: "observation_horizon", noise_journal_failed: "record_rejected", noise_device_failed: "unexpected_terminal" };
    const code = known[error?.category] ?? messages[error?.message] ?? "operation_failed";
    try { await post("/client-failure", { phase: recordFailed ? "journal" : operationPhase, code }); }
    finally {
      if (maybeBinding && maybeId && performance.now() - beginning < 150000) {
        try { const status = await window.workerAcceptance.noiseDiagnosticCancel(maybeId, maybeBinding); await post("/noise/record", { status }); }
        catch { notice.textContent = "Cancellation unconfirmed. Close the host owner; retain the failure for review."; }
      }
    }
    throw error;
  } finally { running = false; }
}
async function restoreAndRecord() {
  operationPhase = "restoration";
  // The operator has closed and natively reconnected this exact page.
  await refresh();
  const { attemptId } = await post("/restoration/context", {});
  const binding = await window.workerAcceptance.noiseDiagnosticPossession();
  const retained = await window.workerAcceptance.noiseDiagnosticStatus(attemptId, binding);
  await post("/noise/record", { status: retained });
  await refresh();
  await post("/restoration", { status: retained, state: published() });
  return recordAccounting("after");
}
Object.assign(window, { noiseSupervisor: { flush, recordAccounting, configureCandidate, recordCycle, run, restoreAndRecord } });

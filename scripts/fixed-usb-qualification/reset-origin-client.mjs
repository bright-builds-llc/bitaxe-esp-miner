const note = document.createElement("p");
note.id = "reset-origin-status";
note.textContent = "Connect the Worker to collect a read-only reset-origin observation.";
document.body.append(note);
let stage = "idle", started = false, cleanupFailed = false;
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function post(path, value, timeoutMs = 5000) {
  const controller = new AbortController(), timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" }, cache: "no-store", body: JSON.stringify(value), signal: controller.signal });
    if (!response.ok) throw Error("reset_origin_request_rejected");
    return await response.json();
  } finally { clearTimeout(timer); }
}
async function bounded(operation, code) {
  let timer;
  try {
    return await Promise.race([operation, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error(code)), 60000);
    })]);
  } finally { clearTimeout(timer); }
}
async function exportDiagnostics() {
  const text = document.querySelector("#diagnostics")?.textContent;
  if (!text) throw Error("reset_origin_diagnostics_missing");
  const receipt = await post("/diagnostic-export", { schema: "worker-diagnostic-export-v1", observations: JSON.parse(text) });
  if (receipt.diagnostic_export_saved !== true || !/^diagnostic-export-[A-Za-z0-9_-]+\.json$/u.test(receipt.review_file))
    throw Error("reset_origin_export_receipt");
}
function baseline() {
  const s = window.workerAcceptance.state();
  if (s.status !== "ready" || !s.connected || s.running || s.failure || !s.deviceLeaseInactive || s.deviceBaselineConfirmed !== true ||
    !s.preservation?.device_identity_match || !s.preservation.settings_match || !s.preservation.authorization_high_water_match || s.preservation.mine_on_boot !== false)
    throw Error("reset_origin_baseline");
}
async function waitForDiagnostics() {
  const began = performance.now();
  while (performance.now() - began < 15000) {
    baseline();
    const text = document.querySelector("#diagnostics")?.textContent;
    if (text) {
      const values = JSON.parse(text);
      if (Array.isArray(values) && ["boot", "runtime_identity", "storage_http_status"].every(category => values.some(v => v.category === category)) &&
        values.some(v => v.category === "startup" && v.stage === "runtime_ready" && v.state === "complete" && v.first_failure === "none")) return;
    }
    await delay(250);
  }
  throw Error("reset_origin_diagnostics_missing");
}
async function run() {
  if (started) throw Error("reset_origin_already_started");
  baseline(); started = true; stage = "preparing";
  const failures = [];
  try {
    await waitForDiagnostics();
    await bounded(window.noMiningSupervisor.recordAccounting("before"), "reset_origin_accounting_timeout");
    await exportDiagnostics();
    await bounded(window.noMiningSupervisor.flush(), "reset_origin_record_timeout");
    await post("/reset-origin/start", {}, 30000);
    stage = "observing"; note.textContent = "Recording read-only startup diagnostics.";
    const began = performance.now(); let refreshed = began;
    while (performance.now() - began < 130000) {
      baseline();
      if (performance.now() - refreshed >= 1000) { await window.workerAcceptance.refresh(); refreshed = performance.now(); baseline(); }
      await exportDiagnostics();
      await delay(250);
    }
    await bounded(window.noMiningSupervisor.flush(), "reset_origin_record_timeout"); await post("/reset-origin/end", {}, 30000);
    await bounded(window.noMiningSupervisor.recordAccounting("after"), "reset_origin_accounting_timeout");
    await window.workerAcceptance.close(); await bounded(window.noMiningSupervisor.flush(), "reset_origin_record_timeout");
    stage = "closed"; note.textContent = "Observation collected and Worker connection closed. Independent review is required.";
    return { observation_collected: true, recovery_authorized: false };
  } catch (error) {
    failures.push(error);
    stage = "failed"; note.textContent = "Observation interrupted. Its evidence is retained.";
    try { await post("/reset-origin/failure", { code: "observer_client_failed" }); }
    catch (recordError) { failures.push(recordError); cleanupFailed = true; }
  } finally {
    if (stage !== "closed") {
      try { await window.workerAcceptance.close(); await bounded(window.noMiningSupervisor.flush(), "reset_origin_record_timeout"); }
      catch (cleanupError) { failures.push(cleanupError); cleanupFailed = true; }
    }
  }
  if (failures.length) throw new AggregateError(failures, "reset_origin_observation_failed");
}
Object.assign(window, { resetOriginObserver: { run, state: () => ({ stage, started, cleanupFailed }) } });

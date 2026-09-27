import { accountingBaselineAllowed } from "./accounting-baseline.mjs";
export function installMeasurementPage(document, window, fetch, MutationObserver) {
// No device authority, signing material or arbitrary command input is accepted here.
for (const id of ["close", "prepare", "load", "start", "arm-foreground", "suppress", "probe", "restart"]) document.getElementById(id)?.remove();
for (const id of ["configuration", "authorization-context"]) {
  const element = document.getElementById(id); if (element) (element.closest("label") ?? element).remove();
}
for (const paragraph of document.querySelectorAll("p")) {
  if (/Signed private windows/u.test(paragraph.textContent ?? "")) paragraph.remove();
  else if (/Signed leases stay in memory/u.test(paragraph.textContent ?? "")) paragraph.textContent = "Task-gated bootstrap measurement. Keep this tab visible; each connection requires fresh permission and possession. No signing or mining.";
}
const notice = document.createElement("p"); notice.id = "bootstrap-status"; notice.textContent = "Bootstrap timing measurement. No mining."; document.body.append(notice);
let queue = Promise.resolve(), maybeFailure, busy = false, accounted = false, configured = false, baselineId, afterAccounted = false, maybeAfterButton;
function updateAvailability() {
  if (maybeAfterButton) maybeAfterButton.disabled = busy || !configured || afterAccounted || !accountingBaselineAllowed(window.workerAcceptance?.state(), "after", baselineId);
}
async function call(path, input) {
  const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(input), signal: AbortSignal.timeout(30000), cache: "no-store" });
  if (!response.ok) throw Error("bootstrap_client_failed"); return response.json();
}
async function flush() {
  while (true) {
    await Promise.resolve(); const pending = queue; await pending; await Promise.resolve();
    if (pending !== queue) continue;
    if (maybeFailure) throw maybeFailure; return;
  }
}
const output = document.querySelector("#state");
if (output) new MutationObserver(() => {
  updateAvailability();
  const state = window.workerAcceptance?.state(); if (!state?.expectedFirmwareSourceCommit) return;
  queue = queue.then(() => call("/record", { state })).catch(error => { maybeFailure ??= error; notice.textContent = "Evidence failed; use Stop/Restore and close."; });
}).observe(output, { childList: true, subtree: true, characterData: true });
function baseline(stage) {
  const s = window.workerAcceptance.state();
  if (!accountingBaselineAllowed(s, stage, baselineId)) throw Error("bootstrap_client_failed");
  return s;
}
async function accounting(stage) {
  await window.workerAcceptance.refresh(); const before = baseline(stage); baselineId ??= before.preservation.baseline_id;
  const { campaignId } = await call("/accounting-context", {});
  const ledger = await window.workerAcceptance.reviewQualificationAttempts(), original = await window.workerAcceptance.reviewBudget(campaignId);
  await flush(); const state = baseline(stage); return call("/accounting", { stage, ledger, original, state });
}
for (const [label, action] of [["Record initial accounting", "before"], ["Configure measured candidate", "configure"], ["Record restored baseline/accounting", "after"], ["Close and flush serial", "close"]]) {
  const button = document.createElement("button"); button.textContent = label; button.id = `bootstrap-${action}`; document.body.append(button);
  if (action === "after") maybeAfterButton = button;
  button.onclick = async () => {
    if (busy || button.disabled) return; busy = true; updateAvailability();
    try {
      if (action === "before") { if (accounted) throw Error(); await accounting("before"); accounted = true; }
      if (action === "configure") {
        const state = window.workerAcceptance.state(); if (!accounted || configured || state.status !== "closed" || !state.serialOwnershipReleased) throw Error();
        await flush(); window.workerAcceptance.configure(await call("/candidate-context", {})); configured = true; await flush();
      }
      if (action === "after") { if (!configured) throw Error(); await accounting("after"); afterAccounted = true; }
      if (action === "close") { await window.workerAcceptance.close(); await flush(); }
      notice.textContent = `${label}: complete`;
    } catch {
      notice.textContent = `${label}: failed; use Stop/Restore and close.`;
      try { await call("/client-failure", { code: "bootstrap_client_failed" }); } catch { notice.textContent += " Failure recording unavailable."; }
    } finally { busy = false; updateAvailability(); }
  };
}

updateAvailability();
return { flush };
}
if (typeof document !== "undefined" && typeof window !== "undefined") installMeasurementPage(document, window, fetch, MutationObserver);

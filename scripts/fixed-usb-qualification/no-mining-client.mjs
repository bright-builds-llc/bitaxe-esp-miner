// The pinned Gate owns USB. This observer has no signing or work-window routes.
for (const id of ["prepare", "load", "start", "arm-foreground", "suppress"]) {
  const maybeButton = document.getElementById(id);
  if (maybeButton) maybeButton.remove();
}
const note = document.createElement("p");
note.id = "supervisor-status";
note.textContent = "No-mining qualification. Close the Worker connection before flashing.";
document.body.append(note);
let queue = Promise.resolve();
let recordFailed = false;
let readOnlyInterruptionAttempted = false;
const maybeOutput = document.querySelector("#state");
if (maybeOutput) new MutationObserver(() => {
  const maybeState = window.workerAcceptance?.state();
  if (!maybeState?.expectedFirmwareSourceCommit) return;
  queue = queue.then(async () => {
    const response = await fetch("/record", { method: "POST", headers: { "Content-Type": "application/json" },
      cache: "no-store", body: JSON.stringify({ state: maybeState }), keepalive: true });
    if (!response.ok) throw new Error("no_mining_record_rejected");
  }).catch(() => { recordFailed = true; note.textContent = "No-mining evidence rejected. Stop and inspect the preserved failure before continuing."; });
}).observe(maybeOutput, { childList: true, subtree: true, characterData: true });

async function post(path, value) {
  const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" },
    cache: "no-store", body: JSON.stringify(value) });
  if (!response.ok) throw new Error("no_mining_request_rejected");
  return response.json();
}
async function flush() {
  await queue;
  if (recordFailed) throw new Error("no_mining_record_rejected");
  return { records_flushed: true };
}
async function recordAccounting(stage) {
  if (!["before", "after"].includes(stage)) throw new Error("no_mining_accounting_stage");
  const { campaignId } = await post("/original-budget-context", {});
  const ledger = await window.workerAcceptance.reviewQualificationAttempts();
  const original_budget = await window.workerAcceptance.reviewBudget(campaignId);
  await flush();
  const state = window.workerAcceptance.state();
  const receipt = await post("/accounting", { stage, ledger, original_budget, state });
  await flush();
  // Never expose the original campaign identifier through this observer API.
  return receipt;
}
function publishedState() {
  const maybeState = document.querySelector("#state");
  if (!maybeState?.textContent) throw new Error("read_only_interruption_state_missing");
  return JSON.parse(maybeState.textContent);
}
async function interruptReadOnlyStatus() {
  if (readOnlyInterruptionAttempted) throw new Error("read_only_interruption_already_attempted");
  await flush();
  if (readOnlyInterruptionAttempted) throw new Error("read_only_interruption_already_attempted");
  const before = publishedState();
  if (before.status !== "ready" || !before.connected || before.running || !before.deviceLeaseInactive ||
    before.deviceBaselineConfirmed !== true || before.failure || before.serialOwnershipReleased || before.renewalsConfirmed !== 0 ||
    !before.preservation?.device_identity_match || !before.preservation.settings_match ||
    !before.preservation.authorization_high_water_match || before.preservation.mine_on_boot !== false)
    throw new Error("read_only_interruption_baseline");
  readOnlyInterruptionAttempted = true;
  const receipt = await window.workerAcceptance.interruptPendingStatusForQualification();
  await flush();
  const after = publishedState();
  return post("/read-only-interruption", { receipt, before, after });
}
Object.assign(window, { noMiningSupervisor: { flush, recordAccounting, interruptReadOnlyStatus } });

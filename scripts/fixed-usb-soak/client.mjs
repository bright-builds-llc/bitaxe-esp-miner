// Page agent for the local soak supervisor: records every published Gate state in order and exposes the
// flush hook the Gate awaits before soak completion. The Gate SDK owns all USB.
const note = document.createElement("p");
note.id = "supervisor-status";
note.textContent = "Local soak supervisor connected.";
document.body.append(note);
let queue = Promise.resolve();
let recordFailed = false;
async function post(route, body) {
  const response = await fetch(route, { method: "POST", headers: { "Content-Type": "application/json" }, cache: "no-store", body: JSON.stringify(body), keepalive: true });
  if (!response.ok) throw new Error("supervisor_request_rejected");
  return response.json();
}
function enqueue(operation) {
  queue = queue.then(operation).catch(() => { recordFailed = true; note.textContent = "Soak record rejected; the soak cannot pass. Inspect the local result."; });
}
const output = document.querySelector("#state");
if (output) new MutationObserver(() => {
  const state = window.workerAcceptance?.state();
  if (!state?.expectedFirmwareSourceCommit) return;
  enqueue(() => post("/record", { state }));
}).observe(output, { childList: true, subtree: true, characterData: true });
/** One station endpoint while idle, handed to the supervisor's observer without entering page state. */
async function startObserver() {
  const endpoint = await window.workerAcceptance.observeStationEndpoint();
  return post("/observer/start", { endpoint, state: window.workerAcceptance.state() });
}
async function flush() {
  await Promise.resolve();
  await queue;
  if (recordFailed) throw new Error("supervisor_record_rejected");
  return { records_flushed: true };
}
Object.assign(window, { recoverySupervisor: { flush }, soakSupervisor: { startObserver } });

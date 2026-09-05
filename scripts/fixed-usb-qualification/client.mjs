// Records only the Gate's already-closed diagnostics. The Gate SDK owns all USB.
const note = document.createElement("p");
note.id = "supervisor-status";
note.textContent = "Local supervisor connected. Close the Worker connection before flashing.";
document.body.append(note);
const finish = document.createElement("button");
finish.textContent = "Validate window and select next";
finish.id = "finish-window";
document.body.append(finish);
let queue = Promise.resolve();
async function post(route, body) {
  const response = await fetch(route, { method: "POST", headers: { "Content-Type": "application/json" },
    cache: "no-store", body: JSON.stringify(body), keepalive: true });
  if (!response.ok) throw new Error("supervisor_request_rejected");
  return response.json();
}
function current() {
  return window.workerAcceptance?.state();
}
function enqueue(operation) {
  queue = queue.then(operation).catch(() => { note.textContent = "Supervisor record rejected; inspect the bounded local result before continuing."; });
}
const output = document.querySelector("#state");
if (output) new MutationObserver(() => {
  const state = current();
  if (!state?.expectedFirmwareSourceCommit) return;
  enqueue(async () => {
    await post("/record", { state });
    if (state.running && state.heartbeatSuppressed) {
      await post("/fault", { kind: "heartbeats_suppressed", running: true, visibility: document.visibilityState, heartbeatSuppressed: true, generation: state.qualification?.generation });
    }
  });
}).observe(output, { childList: true, subtree: true, characterData: true });
document.addEventListener("visibilitychange", () => {
  const state = current();
  if (document.visibilityState !== "hidden" || !state?.running) return;
  // Snapshot the real DOM event before the SDK's disconnect callback settles.
  enqueue(() => post("/fault", { kind: "visibility_hidden", running: true, visibility: "hidden", heartbeatSuppressed: state.heartbeatSuppressed, generation: state.qualification?.generation }));
});
finish.addEventListener("click", () => enqueue(async () => {
  const result = await post("/advance", {});
  note.textContent = result.next_window < 3 ? `Window validated. Close, reconnect, then prepare window ${result.next_window}.` : "Three browser reports validated. Hardware qualification still requires the operator's real-device evidence review.";
}));

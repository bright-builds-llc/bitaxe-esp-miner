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
let recordFailed = false;
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
  queue = queue.then(operation).catch(() => { recordFailed = true; note.textContent = "Supervisor record rejected; inspect the bounded local result before continuing."; });
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
  note.textContent = result.next_window < 3 ? `Window validated. Close, reconnect, then prepare window ${result.next_window}.` : "Available window reports validated. Original failed windows remain unverified; hardware evidence requires review.";
}));

const review = document.createElement("button");
review.id = "review-campaign-budget";
review.textContent = "Review remaining campaign budget";
document.body.append(review);
review.addEventListener("click", () => enqueue(async () => {
  await window.workerAcceptance.submitBudgetReview();
  note.textContent = "Device budget reviewed. The original reservation remains consumed; prepare the remaining window promptly.";
}));

// The recovery hook flushes the published checkpoint before cutting the channel.
async function flush() {
  await Promise.resolve();
  await queue;
  if (recordFailed) throw new Error("supervisor_record_rejected");
  return { records_flushed: true };
}
async function recordTrace(stage) {
  await flush();
  if (stage !== "loss") {
    const trace = await window.workerAcceptance.deviceSerialTraceReview();
    await flush();
    await post("/recovery-trace", { stage, source: "device", trace });
  }
  const trace = window.workerAcceptance.exportBrowserSerialTrace();
  await flush();
  return post("/recovery-trace", { stage, source: "browser", trace });
}
Object.assign(window, { recoverySupervisor: { flush, recordTrace } });

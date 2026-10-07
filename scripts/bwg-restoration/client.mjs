// Page agent for the local restoration supervisor. It has no imports, so the served module graph is this file
// alone. `restorationSupervisor.run(name)` calls one `window.workerRestoration` operation and records its
// closed result with the published page state; `recoverySupervisor.flush()` is what submitCompletion awaits.
// The Gate page owns all USB. Connect stays bound to the page's own trusted #connect button.
const OPERATIONS = ["reconnect", "prepareStart", "loadScenarioLease", "startScenarioLease", "renewOnce", "pause", "cancel",
  "restoreChallengeSatisfied", "triggerClockDiscontinuity", "clockDiscontinuityStimulusReview", "authorizationRejectionReview", "statusReview",
  "replayArtifact", "beginPhysicalWindow", "armPhysicalWindow", "physicalWindowState", "close"];
const VALUED = new Set(["triggerClockDiscontinuity", "clockDiscontinuityStimulusReview", "authorizationRejectionReview", "statusReview",
  "replayArtifact", "beginPhysicalWindow", "armPhysicalWindow", "physicalWindowState"]);
const TOKEN = /^[a-z][a-z0-9_]{0,63}$/u;
const note = document.createElement("p");
note.id = "supervisor-status";
note.textContent = "Local restoration supervisor connected.";
document.body.append(note);
let queue = Promise.resolve();
let recordFailed = false;

async function post(route, body) {
  const response = await fetch(route, { method: "POST", headers: { "Content-Type": "application/json" }, cache: "no-store", credentials: "omit",
    body: JSON.stringify(body) });
  if (!response.ok) throw new Error("supervisor_request_rejected");
  return response.json();
}

function record(body) {
  queue = queue.then(() => post("/record", body)).catch(() => {
    recordFailed = true;
    note.textContent = "Restoration record rejected; this attempt cannot pass. Inspect the local result.";
  });
  return queue;
}

/** Run one page operation and record its closed outcome; the page's own error is rethrown unchanged. */
async function run(operation) {
  const page = window.workerRestoration;
  if (!OPERATIONS.includes(operation) || typeof page?.[operation] !== "function") throw new Error("operation_unknown");
  let value;
  try {
    value = await page[operation]();
  } catch (error) {
    const message = error instanceof Error ? error.message : "";
    await record({ operation, outcome: "error", error: TOKEN.test(message) ? message : "operation_failed", result: null, state: page.state() });
    throw error;
  }
  await record({ operation, outcome: "ok", result: VALUED.has(operation) ? value ?? null : null, state: page.state() });
  return value;
}

async function flush() {
  await Promise.resolve();
  await queue;
  if (recordFailed) throw new Error("supervisor_record_rejected");
  return { records_flushed: true };
}

Object.assign(window, { recoverySupervisor: { flush }, restorationSupervisor: { run, flush } });

const actions = [
  ["accounting", "Record initial accounting"], ["configure", "Configure candidate"],
  ["cycle", "Record next cycle"], ["run", "Run admitted qualification"],
  ["restore", "Record restoration"], ["close", "Close and flush journal"],
];
const safe = state => state?.status === "ready" && state.connected === true && state.running === false &&
  !state.failure && state.deviceBaselineConfirmed === true && state.deviceLeaseInactive === true &&
  state.preservation?.device_identity_match === true && state.preservation?.settings_match === true &&
  state.preservation?.mine_on_boot === false;

/** Fixed native controls retain progress only in this document; no baseline leaves the Gate. */
export function installNativeControls(document, supervisor, gate, published, abort = () => {}) {
  const section = document.createElement("section"), output = document.createElement("pre");
  section.id = "v2-native-controls"; output.id = "v2-native-result";
  output.setAttribute("role", "status"); output.setAttribute("aria-live", "polite");
  const buttons = new Map();
  let ready = false, busy = false, failed = false, accounted = false, configured = false;
  let cycles = 0, consumed = false, completed = false, restored = false;
  let result = { status: "initializing", action: null, code: null };
  function allowed(action) {
    if (busy || !ready) return false;
    if (action === "close") return true;
    if (failed) return false;
    const state = published();
    if (action === "accounting") return !accounted && !configured && safe(state);
    if (action === "configure") return accounted && !configured && state?.status === "closed" && state.serialOwnershipReleased === true;
    if (action === "cycle") return configured && cycles < 4 && !consumed && safe(state);
    if (action === "run") return accounted && configured && cycles === 4 && !consumed && safe(state);
    return completed && !restored && safe(state);
  }
  function render() {
    for (const [action, button] of buttons) button.disabled = !allowed(action);
    buttons.get("cycle").textContent = cycles < 4 ? `Record next cycle ${cycles + 1} of 4` : "Four cycles recorded";
    output.textContent = JSON.stringify({ schema: "str005-v2-native-ui-v1", ...result,
      ready, busy, failure_observed: failed, initial_accounting_recorded: accounted, candidate_configured: configured,
      cycles_recorded: cycles, run_consumed: consumed, restoration_recorded: restored });
  }
  async function execute(action) {
    // Consume admission synchronously, before any promise can allow a second click.
    if (!allowed(action)) return;
    busy = true; result = { status: "pending", action, code: null };
    if (action === "run") consumed = true;
    render();
    try {
      let value;
      if (action === "accounting") { value = await supervisor.recordAccounting("before-install"); if (value?.accounting_saved !== true) throw Error(); accounted = true; }
      else if (action === "configure") { value = await supervisor.configureCandidate(); if (value?.configured !== true) throw Error(); configured = true; }
      else if (action === "cycle") { value = await supervisor.recordCycle(cycles + 1); if (value?.cycle_verified !== true || value.index !== cycles + 1) throw Error(); cycles++; }
      else if (action === "restore") { value = await supervisor.restoreAndRecord(); if (value?.restoration_recorded !== true || value.accounting_recorded !== true) throw Error(); restored = true; }
      else if (action === "close") {
        await gate.close(); await supervisor.flush();
        const state = published();
        if (state?.status !== "closed" || state.connected !== false || state.running !== false || state.serialOwnershipReleased !== true) throw Error();
        value = { closed: true, disconnected: true, serial_ownership_released: true, journal_flushed: true };
      } else {
        value = await supervisor.run();
        if (value?.phase_complete !== true || !["channel", "share"].includes(value.scope) || value.hardware_qualified !== false ||
          value.requires_native_reconnect !== true || !Number.isSafeInteger(value.restoration_wait_remaining_ms) ||
          value.restoration_wait_remaining_ms < 0 || value.restoration_wait_remaining_ms > 145000) throw Error();
        completed = true;
      }
      result = { status: "succeeded", action, code: null };
      if (action === "run") Object.assign(result, { scope: value.scope, phase_complete: true, hardware_qualified: false,
        requires_native_reconnect: true, restoration_wait_remaining_ms: value.restoration_wait_remaining_ms });
      if (action === "close") Object.assign(result, value);
    } catch {
      failed = true; result = { status: "failed", action, code: "operation_failed" };
    } finally { busy = false; render(); }
  }
  for (const [action, label] of actions) {
    const button = document.createElement("button"); button.id = `v2-${action}`; button.type = "button";
    button.textContent = label; button.addEventListener("click", () => execute(action));
    buttons.set(action, button); section.append(button);
  }
  section.append(output); document.body.append(section);
  function observe(state) {
    if (["configured", "ready"].includes(state?.status) && state.expectedFirmwareSourceCommit) {
      ready = true;
      if (result.status === "initializing") result = { status: "ready", action: null, code: null };
    }
    render();
  }
  document.getElementById("stop")?.addEventListener("click", () => {
    failed = true; abort();
    result = { status: "failed", action: "emergency-stop", code: "operation_failed" }; render();
  }, { capture: true });
  render();
  return { observe };
}

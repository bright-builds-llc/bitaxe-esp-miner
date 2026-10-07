// Imperative owner of the physical checkpoint windows: one presence watcher per window, started before any
// removal instruction and released once the window settles. The pure state machine lives in checkpoint.mjs.
import { resolve } from "node:path";
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { armRestore, beginWindow, failCheckpoint, view, watcherEvent } from "./checkpoint.mjs";
import { WATCHER_PRESENT_TIMEOUT_MS } from "./contract.mjs";
import { createPresenceWatcher } from "./watcher.mjs";

const SETTLED = ["rearm_required", "failed", "reconnected"];

export function createPhysicalOwner({ campaign, context, root, now, hostEvent, announce, serial, onFailed }, operations = {}) {
  const createWatcher = operations.createWatcher ?? createPresenceWatcher;
  let maybeWatcher;
  const checkpoint = () => campaign.scenario.maybeCheckpoint;
  const requireCheckpoint = () => { requireCondition(checkpoint() !== null, "not_physical_scenario"); return checkpoint(); };

  async function stopWatcher() {
    const watcher = maybeWatcher; maybeWatcher = undefined;
    if (!watcher) return;
    const result = await watcher.stop();
    await hostEvent("watcher_stopped", { stopped_on_request: result?.stopped_on_request === true });
  }

  /** After any checkpoint change: release the watcher once its window is settled, and stop on a typed failure. */
  async function settle() {
    const state = checkpoint();
    if (!state) return;
    view(state, now());
    if (SETTLED.includes(state.checkpoint)) await stopWatcher();
    if (state.checkpoint === "failed") await onFailed(state.maybeFailure);
  }

  /** Begin needs the lease signed after this readiness and already delivered; the watcher must prove presence. */
  async function begin() {
    const state = requireCheckpoint();
    requireCondition(campaign.scenario.signedSinceReady === 1 && campaign.scenario.maybePending === null, "physical_window_lease_missing");
    beginWindow(state, now());
    const watcher = createWatcher({ binary: context.watcher, physicalIdentity: context.physical_identity_sha256,
      journalPath: resolve(root, "watcher.jsonl"), stderrPath: resolve(root, "watcher.stderr.log"),
      onEvent: (event, at) => {
        if (checkpoint() !== state) return;
        watcherEvent(state, event, at);
        serial.queue = serial.queue.then(settle).catch(() => undefined);
      } }, operations);
    maybeWatcher = watcher;
    try { await watcher.start(); } catch (error) { failCheckpoint(state, "watcher_start_failed", now()); await settle(); throw error; }
    await hostEvent("watcher_started");
    const deadline = now() + WATCHER_PRESENT_TIMEOUT_MS;
    while (state.checkpoint === "watcher_starting" && now() < deadline) await new Promise((done) => setTimeout(done, 25));
    if (state.checkpoint === "watcher_starting") failCheckpoint(state, "watcher_present_timeout", now());
    await hostEvent("checkpoint", { checkpoint: state.checkpoint });
    await settle();
    return { checkpoint: view(state, now()).checkpoint };
  }

  /** The restore token is announced before the restore instruction, which only `/supervisor-state` publishes. */
  async function arm() {
    const state = requireCheckpoint();
    const token = armRestore(state, now());
    if (token) {
      announce(`action_token=${token.action_token} response_required=false scenario=${token.scenario}`);
      await hostEvent("restore_watcher_armed", { response_required: false });
    }
    return { checkpoint: view(state, now()).checkpoint };
  }

  return { begin, arm, settle, stopWatcher, checkpoint, requireCheckpoint };
}

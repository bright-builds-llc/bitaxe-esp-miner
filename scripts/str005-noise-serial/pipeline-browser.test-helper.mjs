import vm from "node:vm";
import { readFile, writeFile } from "node:fs/promises";
import { state, ledger, original } from "./test-fixture.mjs";
import { admitted, example } from "./fixtures.mjs";
import { canonical, digest } from "./files.mjs";
export async function pipelineBrowser(f, origin) {
  let phase = "before", current = state(f.context, phase), maybeObserver, boot = 1, epoch = 1, binding = 0, maybeStart = null, maybeAccepted;
  const output = { textContent: JSON.stringify(current) };
  function publish() { output.textContent = JSON.stringify(current); if (maybeObserver) queueMicrotask(maybeObserver); }
  const qualification = { schema: "worker-qualification-v1", generation: 0, active_ms: 0, generation_elapsed_ms: 0, budget_reserved_ms: 0,
    submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 0, last_valid_heartbeat_ms: 0,
    budget_complete: true, safe_stop_complete: true, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true,
    watchdog_alive: true, mine_on_boot: false, voltage_volts: 5.2, power_watts: 1, chip_temp_celsius: 30, fan_rpm: 3200,
    gate_closed_ms: 0, shutdown_started_ms: 0, safe_stop_stage: "fan_paused", revocation_reason: "none", active_limit_ms: null,
    shutdown_budget_ms: 15550, work_gate_remaining_ms: null };
  function observation() { return { bootOrdinal: boot, workerGeneration: 7, transportEpoch: epoch, observedAtUs: maybeStart ? 11000 : 100,
    stationIpv4: "192.168.1.10", wifiConnected: true }; }
  const api = {
    configure: async (config) => { phase = config.noiseQualification; boot++; current = state(f.context, phase, true); current.status = "configured"; publish(); },
    refresh: async () => { publish(); }, state: () => structuredClone(current),
    reviewQualificationAttempts: async () => { binding++; return ledger; }, reviewBudget: async () => { binding++; return original; },
    noiseDiagnosticPossession: async () => `binding-${++binding}`,
    noiseDiagnosticStatus: async (id, expected) => {
      if (expected !== `binding-${binding}`) throw new Error("stale synthetic binding");
      if (id === null) return { schema: "worker-noise-diagnostic-status-v2", state: "idle", observation: observation(), job: null };
      if (!maybeAccepted) {
        maybeAccepted = example().device; maybeAccepted.schema = "worker-noise-diagnostic-status-v2";
        Object.assign(maybeAccepted.job, { attemptId: maybeStart.attemptId, inputSha256: digest(canonical(maybeStart)), bootOrdinal: boot, transportEpoch: epoch });
        maybeAccepted.job.resources.deadlineAtUs = maybeAccepted.job.authorityDeadlineUs + 5000000;
        await writeFile(`${f.root}/synthetic-proof.ready`, "synthetic", { flag: "wx", mode: 0o600 });
      }
      return { ...structuredClone(maybeAccepted), observation: observation() };
    },
    noiseDiagnosticStart: async (input) => {
      maybeStart = input;
      const value = admitted(); value.schema = "worker-noise-diagnostic-status-v2";
      Object.assign(value.job, { attemptId: input.attemptId, inputSha256: digest(canonical(input)), bootOrdinal: boot, transportEpoch: epoch });
      value.observation = { ...observation(), observedAtUs: 1000 }; return value;
    },
    stop: async () => { current.status = "baseline_confirmed"; publish(); },
    close: async () => { current.status = "closed"; current.connected = false; current.serialOwnershipReleased = true; publish(); },
    probe: async () => { current.probe = { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }; publish(); return current.probe; },
  };
  const sandbox = { window: { workerAcceptance: api }, document: { getElementById: () => null, createElement: () => ({}), body: { append() {} }, querySelector: () => output },
    MutationObserver: class { constructor(fn) { maybeObserver = fn; } observe() {} }, performance, setTimeout,
    fetch: (path, init) => fetch(`${origin}${path}`, { ...init, headers: { ...init.headers, Origin: origin } }),
  };
  vm.runInNewContext(await readFile(new URL("./client.mjs", import.meta.url), "utf8"), sandbox);
  return { driver: sandbox.window.noiseSupervisor, api,
    async connect() { epoch++; current = { ...state(f.context, phase), ...(phase === "candidate" ? { qualification } : {}) }; publish(); await sandbox.window.noiseSupervisor.flush(); } };
}

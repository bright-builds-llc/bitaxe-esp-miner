import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import vm from "node:vm";
import { restartFixture, installRestartFixture, restartPacket } from "./reset-origin-restart-fixtures.mjs";
import { createRestartSupervisor } from "./reset-origin-restart-server.mjs";
import { judgeRestart, readRestartResult } from "./reset-origin-restart-judge.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { resetDiagnostics } from "./reset-origin-fixtures.mjs";
import { inspectStartupSources } from "./cadence-startup-context.mjs";
import { writeNew } from "./contract.mjs";

async function pipeline(t, { largeEvidence = false, deferConsume = false } = {}) {
  let closeServer = async () => {};
  t.after(() => closeServer());
  const f = await restartFixture(t);
  let clock = 100,
    state,
    config,
    sequence = 0;
  const timers = new Map(),
    observers = [],
    calls = [],
    stateNode = { textContent: "" };
  let pauseVerification = false,
    entered,
    release;
  const verificationEntered = new Promise((resolve) => {
    entered = resolve;
  });
  const verificationReleased = new Promise((resolve) => {
    release = resolve;
  });
  const server = await createRestartSupervisor(f.options, {
    ...f.operations,
    now: () => clock,
    inspectSources: async (options, operations) => {
      const value = await inspectStartupSources(options, operations);
      if (pauseVerification) {
        pauseVerification = false;
        entered();
        await verificationReleased;
      }
      return value;
    },
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const origin = `http://127.0.0.1:${server.address().port}`;
  closeServer = async () => {
    release();
    if (!server.listening) return;
    try {
      await server.closeQualificationResources();
    } finally {
      await new Promise((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
    }
  };
  function publish() {
    stateNode.textContent = JSON.stringify(state);
    for (const notify of observers) queueMicrotask(notify);
  }
  const configuredState = () =>
    recoveryState({
      gate_commit: config.expectedGateCommit,
      firmware_commit: config.expectedFirmwareSourceCommit,
      app_elf_sha256: config.expectedAppElfSha256,
    });
  const worker = {
    configure(input) {
      assert.deepEqual(
        Object.keys(input).sort(),
        ["expectedAppElfSha256", "expectedFirmwareSourceCommit", "expectedGateCommit", "restartQualification", "trust"].sort(),
      );
      assert.equal(input.restartQualification, true);
      config = input;
      state = { ...configuredState(), status: "configured", connected: false };
      publish();
    },
    state: () => state,
    connect: async () => {
      state = configuredState();
      publish();
    },
    refresh: async () => {
      publish();
    },
    close: async () => {
      state = { ...state, status: "closed", connected: false, serialOwnershipReleased: true };
      publish();
    },
    reviewQualificationAttempts: async () => f.ledger,
    reviewBudget: async () => f.original,
    qualificationRestart: async (request) => {
      calls.push("qualification_restart");
      state = { ...state, status: "restarting", deviceBaselineConfirmed: false };
      publish();
      await new Promise((resolve) => setImmediate(resolve));
      const evidence = restartPacket(f.context, { ordinal: request.expectedBootOrdinal });
      if (largeEvidence) {
        evidence.observations = evidence.observations.slice(0, 2);
        for (let record = 4; record <= 511; record++)
          evidence.observations.push({
            record,
            atMs: 100,
            diagnostic: {
              category: "startup",
              authoritative: false,
              stage: "runtime_ready",
              state: "complete",
              first_failure: "none",
              uptime_ms: record * 10,
            },
          });
        evidence.summary.records = 512;
        evidence.summary.bytes = 200000;
        evidence.lifecycle.at(-2).record = 511;
        evidence.lifecycle.at(-2).atMs = 120;
        evidence.lifecycle.at(-1).record = 512;
        assert(Buffer.byteLength(JSON.stringify(evidence)) > 65536);
      }
      clock += evidence.summary.durationMs;
      state = { ...configuredState(), restart: evidence.summary };
      publish();
      return evidence;
    },
  };
  const diagnosticNode = {
    get textContent() {
      return JSON.stringify(
        resetDiagnostics(f.context, clock + 1000).map((d) =>
          d.category === "boot" ? { ...d, boot_ordinal: 7, reset_reason: "software_cpu" } : d,
        ),
      );
    },
  };
  function schedule(callback, delay) {
    const id = ++sequence;
    timers.set(id, { callback, at: clock + delay });
    if (delay <= 250)
      setImmediate(() => {
        const value = timers.get(id);
        if (!value) return;
        clock = value.at;
        for (const [key, timer] of [...timers]) if (timer.at <= clock && timers.delete(key)) timer.callback();
      });
    return id;
  }
  const sandbox = vm.createContext({
    window: { workerAcceptance: worker },
    document: {
      getElementById: () => null,
      createElement: () => ({}),
      body: { append: () => {} },
      querySelector: (name) => (name === "#state" ? stateNode : name === "#diagnostics" ? diagnosticNode : undefined),
    },
    MutationObserver: class {
      constructor(callback) {
        observers.push(callback);
      }
      observe() {}
    },
    fetch: async (path, options = {}) => {
      calls.push(path);
      if (deferConsume && path === "/restart/consume") pauseVerification = true;
      const response = await fetch(`${origin}${path}`, { ...options, headers: { ...options.headers, Origin: origin } });
      if (path === "/restart/consume" && response.ok) calls.push("restart_permit_returned");
      return response;
    },
    performance: { now: () => clock },
    setTimeout: schedule,
    clearTimeout: (id) => timers.delete(id),
    AbortController,
    AggregateError,
    console,
  });
  vm.runInContext(await readFile(new URL("./no-mining-client.mjs", import.meta.url), "utf8"), sandbox);
  vm.runInContext(`{${await readFile(new URL("./reset-origin-restart-client.mjs", import.meta.url), "utf8")}\n}`, sandbox);
  // Model Gate's actual one-time /context -> configure autoload before operator actions.
  const autoload = await fetch(`${origin}/context`);
  worker.configure(await autoload.json());
  return {
    ...f,
    worker,
    client: sandbox.window.restartSupervisor,
    server,
    origin,
    calls,
    verificationEntered,
    releaseVerification: release,
  };
}

test("real HTTP server and both browser clients preserve installation baseline through capture/restart/accounting/final judge", async (t) => {
  // Arrange
  const f = await pipeline(t, { largeEvidence: true });
  // Act
  await f.client.configureBeforeInstall();
  await f.worker.connect();
  await f.client.prepareInstallation();
  await installRestartFixture(f);
  await f.client.configureAfterInstall();
  await f.worker.connect();
  assert.equal((await f.client.observeAndRestart()).captured, true);
  await f.server.closeQualificationResources();
  await new Promise((resolve) => f.server.close(resolve));
  const cleanup = resolve(f.root, "host-cleanup.json");
  await writeNew(cleanup, {
    schema: "worker-restart-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  const result = await judgeRestart(f.root, cleanup, f.operations);
  const reviewed = await readRestartResult(resolve(f.root, "result.json"), f.operations);
  // Assert
  assert.equal(result.result, "controlled_restart_verified");
  assert.equal(result.qualification_pass, false);
  assert.equal(reviewed.ledger.next_ordinal, 17);
  assert.equal(reviewed.ledger.total_charged_ms, 1380000);
  assert.equal(f.calls.filter((value) => value === "qualification_restart").length, 1);
});

test("restart HTTP routes reject foreign origins and never offer signing, pool, lease or arbitrary controls", async (t) => {
  // Arrange
  const f = await pipeline(t);
  // Act / Assert
  for (const path of ["/authorization-context", "/sign", "/grant", "/pool", "/control"])
    assert.equal((await fetch(`${f.origin}${path}`, { method: "POST", headers: { Origin: f.origin }, body: "{}" })).status, 404);
  assert.equal(
    (await fetch(`${f.origin}/activate`, { method: "POST", headers: { Origin: "http://foreign.invalid" }, body: "{}" })).status,
    400,
  );
});

test("post-install accounting rejects replacement of the private baseline before any restart claim", async (t) => {
  // Arrange
  const f = await pipeline(t);
  await f.client.configureBeforeInstall();
  await f.worker.connect();
  await f.client.prepareInstallation();
  await installRestartFixture(f);
  await f.client.configureAfterInstall();
  await f.worker.connect();
  const state = structuredClone(f.worker.state());
  state.preservation.baseline_id = Buffer.alloc(16, 9).toString("base64url");
  const post = (path, value) =>
    fetch(`${f.origin}${path}`, {
      method: "POST",
      headers: { Origin: f.origin, "Content-Type": "application/json" },
      body: JSON.stringify(value),
    });
  // Act
  await post("/record", { state });
  const response = await post("/accounting", { stage: "before", ledger: f.ledger, original_budget: f.original, state });
  // Assert
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "restart_install_preservation_changed");
  await assert.rejects(readFile(resolve(f.root, "restart-consumed.json")), { code: "ENOENT" });
});

test("post-install prime rejects a boot that changed during the ownership handoff", async (t) => {
  // Arrange
  const f = await pipeline(t);
  await f.client.configureBeforeInstall();
  await f.worker.connect();
  await f.client.prepareInstallation();
  await installRestartFixture(f);
  await f.client.configureAfterInstall();
  await f.worker.connect();
  // Act
  const response = await fetch(`${f.origin}/diagnostic-export`, {
    method: "POST",
    headers: { Origin: f.origin, "Content-Type": "application/json" },
    body: JSON.stringify({ schema: "worker-diagnostic-export-v1", observations: resetDiagnostics(f.context, 1000) }),
  });
  // Assert
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "restart_install_boot_changed");
  await assert.rejects(readFile(resolve(f.root, "restart-consumed.json")), { code: "ENOENT" });
});

test("unclaimed completed restart metadata is rejected before installation", async (t) => {
  // Arrange
  const f = await pipeline(t);
  await f.client.configureBeforeInstall();
  await f.worker.connect();
  const state = { ...f.worker.state(), restart: restartPacket(f.context).summary };
  // Act
  const response = await fetch(`${f.origin}/record`, {
    method: "POST",
    headers: { Origin: f.origin, "Content-Type": "application/json" },
    body: JSON.stringify({ state }),
  });
  // Assert
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "restart_unclaimed_state");
});

test("terminal failure during deferred consume verification cannot return a permit or restart", async (t) => {
  // Arrange
  const f = await pipeline(t, { deferConsume: true });
  await f.client.configureBeforeInstall();
  await f.worker.connect();
  await f.client.prepareInstallation();
  await installRestartFixture(f);
  await f.client.configureAfterInstall();
  await f.worker.connect();
  const run = f.client.observeAndRestart();
  const rejectedRun = assert.rejects(run);
  await Promise.race([
    f.verificationEntered,
    run.then(() => {
      throw Error("restart flow ended before verification gate");
    }),
  ]);
  // Act
  try {
    const invalid = await fetch(`${f.origin}/activate`, { method: "POST", headers: { Origin: "http://foreign.invalid" }, body: "{}" });
    assert.equal(invalid.status, 400);
  } finally {
    f.releaseVerification();
  }
  await rejectedRun;
  // Assert
  assert.equal(f.calls.filter((value) => value === "restart_permit_returned").length, 0);
  assert.equal(f.calls.filter((value) => value === "qualification_restart").length, 0);
  await assert.rejects(readFile(resolve(f.root, "restart-consumed.json")), { code: "ENOENT" });
  const state = await (await fetch(`${f.origin}/supervisor-state`)).json();
  assert.equal(state.phase, "failed");
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";
const source = await readFile(
  new URL("./reset-origin-client.mjs", import.meta.url),
  "utf8",
);
function fixture({
  failExport = 0,
  failRecord = false,
  failClose = false,
  diagnostics = true,
  hangExport = 0,
  hangAccounting = false,
  hangFlush = false,
} = {}) {
  let clock = 0,
    exports = 0,
    closes = 0,
    timerId = 0;
  const events = [],
    note = {},
    timers = new Map();
  function advanceTo(time) {
    clock = time;
    for (const [id, timer] of [...timers].sort((a, b) => a[1].at - b[1].at)) {
      if (timer.at <= clock && timers.delete(id)) timer.callback();
    }
  }
  function timeout(callback, ms) {
    const id = ++timerId;
    timers.set(id, { at: clock + ms, callback });
    if (ms <= 250)
      queueMicrotask(() => {
        if (timers.has(id)) advanceTo(timers.get(id).at);
      });
    return id;
  }
  const pendingOperation = () =>
    new Promise(() => {
      queueMicrotask(() =>
        advanceTo(Math.min(...[...timers.values()].map((timer) => timer.at))),
      );
    });
  const state = {
    status: "ready",
    connected: true,
    running: false,
    deviceLeaseInactive: true,
    deviceBaselineConfirmed: true,
    preservation: {
      device_identity_match: true,
      settings_match: true,
      authorization_high_water_match: true,
      mine_on_boot: false,
    },
  };
  const diagnosticValues = diagnostics
    ? [
        {
          category: "boot",
          authoritative: false,
          boot_ordinal: 2,
          reset_reason: "panic",
          uptime_ms: 1000,
        },
        {
          category: "runtime_identity",
          authoritative: false,
          firmware_commit: "a".repeat(40),
          app_elf_sha256: "b".repeat(64),
        },
        {
          category: "storage_http_status",
          authoritative: false,
          spiffs_available: "true",
          http_ready: "true",
        },
        {
          category: "startup",
          authoritative: false,
          stage: "runtime_ready",
          state: "complete",
          first_failure: "none",
          uptime_ms: 1000,
        },
      ]
    : [];
  const diag = { textContent: JSON.stringify(diagnosticValues) };
  const worker = {
    state: () => structuredClone(state),
    refresh: async () => {
      events.push({ kind: "refresh", at: clock });
    },
    exportDiagnostics: async () => {
      throw Error("unbounded_legacy_export_called");
    },
    close: async () => {
      closes++;
      events.push({ kind: "close", at: clock });
      if (failClose) throw Error("cleanup_failed_fixture");
      Object.assign(state, { status: "closed", connected: false });
    },
  };
  const window = {
    workerAcceptance: worker,
    noMiningSupervisor: {
      recordAccounting: async (when) => {
        events.push({ kind: `accounting_${when}`, at: clock });
        if (hangAccounting) return pendingOperation();
      },
      flush: async () => {
        events.push({ kind: "flush", at: clock });
        if (hangFlush) return pendingOperation();
      },
    },
  };
  const fetch = async (path, options) => {
    events.push({ kind: path, at: clock, input: JSON.parse(options.body) });
    if (path === "/diagnostic-export") {
      exports++;
      if (exports === failExport) throw Error("export_failed_fixture");
      if (exports === hangExport)
        return new Promise((_resolve, reject) => {
          options.signal.addEventListener(
            "abort",
            () => {
              events.push({ kind: "aborted", at: clock });
              reject(Error("request_aborted_fixture"));
            },
            { once: true },
          );
          queueMicrotask(() =>
            advanceTo(
              Math.min(...[...timers.values()].map((timer) => timer.at)),
            ),
          );
        });
    }
    return {
      ok: !(path === "/reset-origin/failure" && failRecord),
      json: async () =>
        path === "/diagnostic-export"
          ? {
              diagnostic_export_saved: true,
              review_file: `diagnostic-export-${exports}.json`,
            }
          : {},
    };
  };
  vm.runInNewContext(source, {
    window,
    document: {
      createElement: () => note,
      body: { append: () => {} },
      querySelector: (selector) => (selector === "#diagnostics" ? diag : null),
    },
    performance: { now: () => clock },
    setTimeout: timeout,
    clearTimeout: (id) => timers.delete(id),
    AbortController,
    fetch,
    AggregateError,
    Error,
    JSON,
  });
  return {
    observer: window.resetOriginObserver,
    events,
    state,
    note,
    diagnosticValues,
    timers,
    closes: () => closes,
  };
}

test("actual client primes, polls continuously, refreshes, finishes accounting and closes exactly once", async () => {
  // Arrange
  const f = fixture();
  // Act
  const result = await f.observer.run();
  // Assert
  assert.equal(result.observation_collected, true);
  assert.equal(result.recovery_authorized, false);
  const kinds = f.events.map((event) => event.kind),
    start = kinds.indexOf("/reset-origin/start"),
    end = kinds.indexOf("/reset-origin/end");
  assert(kinds.indexOf("accounting_before") < start);
  assert(kinds.indexOf("/diagnostic-export") < start);
  assert(end > start);
  assert(kinds.indexOf("accounting_after") > end);
  assert(kinds.indexOf("close") > kinds.indexOf("accounting_after"));
  const batches = f.events
    .slice(start + 1, end)
    .filter((event) => event.kind === "/diagnostic-export");
  assert(batches.length >= 500);
  assert(
    batches.every(
      (event, index) => index === 0 || event.at - batches[index - 1].at === 250,
    ),
  );
  assert(f.events.filter((event) => event.kind === "refresh").length >= 120);
  assert.equal(f.closes(), 1);
  assert.deepEqual(
    f.events.find((event) => event.kind === "/diagnostic-export").input
      .observations,
    f.diagnosticValues,
  );
  assert.equal(f.timers.size, 0);
  assert.equal(f.observer.state().stage, "closed");
  assert.equal(f.observer.state().cleanupFailed, false);
  await assert.rejects(f.observer.run(), /reset_origin_already_started/u);
  assert.equal(f.closes(), 1);
  assert(!kinds.some((kind) => /sign|restart|install|lease|pool/u.test(kind)));
});

test("export failure is preserved before failure recording and cleanup failures", async () => {
  // Arrange
  const f = fixture({ failExport: 3, failRecord: true, failClose: true });
  // Act / Assert
  await assert.rejects(f.observer.run(), (error) => {
    assert.equal(error.message, "reset_origin_observation_failed");
    assert.equal(error.errors[0].message, "export_failed_fixture");
    assert.equal(error.errors[1].message, "reset_origin_request_rejected");
    assert.equal(error.errors[2].message, "cleanup_failed_fixture");
    return true;
  });
  assert.equal(f.observer.state().stage, "failed");
  assert.equal(f.observer.state().cleanupFailed, true);
  assert.equal(f.closes(), 1);
  assert(!f.events.some((event) => event.kind === "/reset-origin/end"));
});

test("missing required diagnostics fails within the preparation bound and closes the connection", async () => {
  // Arrange
  const f = fixture({ diagnostics: false });
  // Act / Assert
  await assert.rejects(
    f.observer.run(),
    (error) => error.errors[0].message === "reset_origin_diagnostics_missing",
  );
  assert.equal(f.closes(), 1);
  assert.equal(f.observer.state().stage, "failed");
  assert(
    !f.events.some(
      (event) =>
        event.kind === "accounting_before" ||
        event.kind === "/reset-origin/start",
    ),
  );
});

test("loss of ready baseline during polling stops further observation and closes", async () => {
  // Arrange
  const f = fixture();
  const original = f.state.preservation;
  Object.defineProperty(f.state, "preservation", {
    get() {
      return f.events.filter((event) => event.kind === "/diagnostic-export")
        .length > 3
        ? { ...original, settings_match: false }
        : original;
    },
    enumerable: true,
  });
  // Act / Assert
  await assert.rejects(
    f.observer.run(),
    (error) => error.errors[0].message === "reset_origin_baseline",
  );
  assert.equal(f.closes(), 1);
  assert(f.events.some((event) => event.kind === "/reset-origin/failure"));
});

test("hung diagnostic POST aborts at its bound, records failure and closes the Worker", async () => {
  // Arrange
  const f = fixture({ hangExport: 2 });
  // Act / Assert
  await assert.rejects(
    f.observer.run(),
    (error) => error.errors[0].message === "request_aborted_fixture",
  );
  const aborted = f.events.find((event) => event.kind === "aborted");
  assert.equal(aborted.at, 5000);
  assert.equal(f.closes(), 1);
  assert.equal(f.timers.size, 0);
  assert(f.events.some((event) => event.kind === "/reset-origin/failure"));
  assert(!f.events.some((event) => event.kind === "/reset-origin/end"));
});

test("hung accounting reaches its 60-second bound and then actually closes the Worker", async () => {
  // Arrange
  const f = fixture({ hangAccounting: true });
  // Act / Assert
  await assert.rejects(
    f.observer.run(),
    (error) => error.errors[0].message === "reset_origin_accounting_timeout",
  );
  assert.equal(f.events.find((event) => event.kind === "close").at, 60000);
  assert.equal(f.closes(), 1);
  assert.equal(f.observer.state().stage, "failed");
  assert.equal(f.state.connected, false);
  assert.equal(f.timers.size, 0);
  assert(!f.events.some((event) => event.kind === "/reset-origin/start"));
});

test("a hung record flush preserves explicit cleanup failure after closing", async () => {
  // Arrange
  const f = fixture({ hangFlush: true });
  // Act / Assert
  await assert.rejects(f.observer.run(), (error) => {
    assert.equal(error.errors[0].message, "reset_origin_record_timeout");
    assert.equal(error.errors.at(-1).message, "reset_origin_record_timeout");
    return true;
  });
  assert.equal(f.closes(), 1);
  assert.equal(f.state.connected, false);
  assert.equal(f.observer.state().cleanupFailed, true);
  assert.equal(f.timers.size, 0);
});

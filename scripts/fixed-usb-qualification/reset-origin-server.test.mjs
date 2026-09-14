import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import {
  createResetOriginSupervisor,
  selectResetOriginDiagnostics,
} from "./reset-origin-server.mjs";

const boot = {
  category: "boot",
  authoritative: false,
  boot_ordinal: 2,
  reset_reason: "panic",
  uptime_ms: 1000,
};
const startup = {
  category: "startup",
  authoritative: false,
  stage: "runtime_ready",
  state: "complete",
  first_failure: "none",
  uptime_ms: 1000,
};
const exportOf = (observations) => ({
  schema: "worker-diagnostic-export-v1",
  observations,
});

test("selected diagnostic metadata is validated and unrelated allowed metadata is not persisted", () => {
  // Arrange
  const observations = [
    boot,
    startup,
    { category: "memory", private: "discarded-fixture" },
    { category: "worker_admission", stage: "idle", first_failure: "none" },
    {
      category: "worker_preparation_receipt",
      authoritative: false,
      status: "corrupt",
      origin: "previous_boot",
    },
  ];
  // Act / Assert
  assert.deepEqual(selectResetOriginDiagnostics(exportOf(observations)), [
    boot,
    startup,
    observations[4],
  ]);
  assert.throws(
    () =>
      selectResetOriginDiagnostics(exportOf([{ ...boot, private: "fixture" }])),
    /object_fields/u,
  );
  assert.throws(
    () =>
      selectResetOriginDiagnostics(
        exportOf([{ ...boot, authoritative: true }]),
      ),
    /reset_origin_authority/u,
  );
  assert.throws(
    () => selectResetOriginDiagnostics(exportOf(Array(41).fill(boot))),
    /reset_origin_export_shape/u,
  );
});

test("failure diagnostics and work/preparation activity cannot be silently filtered away", () => {
  // Arrange / Act / Assert
  for (const category of [
    "control_failure",
    "serial_rx_failure",
    "serial_tx_failure",
    "network_failure",
    "startup_failure",
    "storage_http_failure",
  ]) {
    assert.throws(
      () =>
        selectResetOriginDiagnostics(
          exportOf([{ category, private: "fixture" }]),
        ),
      /reset_origin_failure_diagnostic/u,
    );
  }
  assert.throws(
    () =>
      selectResetOriginDiagnostics(
        exportOf([
          {
            category: "worker_admission",
            stage: "active",
            first_failure: "none",
          },
        ]),
      ),
    /reset_origin_unexpected_worker_activity/u,
  );
  assert.throws(
    () =>
      selectResetOriginDiagnostics(
        exportOf([{ category: "worker_preparation_receipt", status: "valid" }]),
      ),
    /reset_origin_preparation_receipt_review_required/u,
  );
  assert.throws(
    () => selectResetOriginDiagnostics(exportOf([{ category: "unknown" }])),
    /reset_origin_unsupported_diagnostic/u,
  );
});

test("credentials and caller-supplied context are rejected before loading any observation root", async () => {
  // Arrange / Act / Assert
  for (const forbidden of [
    { authorityDirectory: "/must-not-read" },
    { poolCredentials: "/must-not-read" },
    { context: {} },
  ]) {
    await assert.rejects(
      createResetOriginSupervisor({
        privateRoot: "/absent-fixture",
        ...forbidden,
      }),
      { code: "reset_origin_credentials_forbidden" },
    );
  }
});

async function serverFixture(t) {
  const { resetOriginFixture, resetDiagnostics, recordResetAccounting } =
    await import("./reset-origin-fixtures.mjs");
  let server;
  t.after(async () => {
    if (!server) return;
    try {
      await server.closeQualificationResources();
    } finally {
      await new Promise((done) => {
        server.close(done);
        server.closeAllConnections();
      });
    }
  });
  const f = await resetOriginFixture(t);
  let clock = 1000;
  const operations = { ...f.operations, now: () => clock };
  server = await createResetOriginSupervisor(
    { privateRoot: f.root },
    operations,
  );
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const origin = `http://127.0.0.1:${server.address().port}`;
  const request = async (path, input, extraHeaders = {}) => {
    const response = await fetch(origin + path, {
      method: input === undefined ? "GET" : "POST",
      headers: {
        Origin: origin,
        "Content-Type": "application/json",
        ...extraHeaders,
      },
      body: input === undefined ? undefined : JSON.stringify(input),
      signal: AbortSignal.timeout(30000),
    });
    return { status: response.status, body: await response.json() };
  };
  const prime = async () => {
    const state = await recordResetAccounting(f, "before");
    assert.equal(
      (
        await request(
          "/diagnostic-export",
          exportOf(resetDiagnostics(f.context, 1000)),
        )
      ).status,
      200,
    );
    return state;
  };
  return {
    ...f,
    operations,
    server,
    request,
    prime,
    diagnostics: (uptime) => exportOf(resetDiagnostics(f.context, uptime)),
    tick: (value) => {
      clock = value;
    },
  };
}

test("real server primes, starts, stores bounded batches, ends and preserves one-shot evidence", async (t) => {
  // Arrange
  const f = await serverFixture(t),
    state = await f.prime();
  // Act
  const started = await f.request("/reset-origin/start", {});
  assert.equal(started.status, 200);
  f.tick(3000);
  assert.equal(
    (await f.request("/diagnostic-export", f.diagnostics(3000))).status,
    200,
  );
  assert.equal((await f.request("/record", { state })).status, 200);
  f.tick(121000);
  const ended = await f.request("/reset-origin/end", {});
  // Assert
  assert.equal(ended.status, 200);
  assert.equal(ended.body.observation_finished, true);
  assert.equal(ended.body.recovery_authorized, false);
  const start = JSON.parse(
    await readFile(resolve(f.root, "reset-origin-start.json"), "utf8"),
  );
  const endBytes = await readFile(
      resolve(f.root, "reset-origin-end.json"),
      "utf8",
    ),
    end = JSON.parse(endBytes);
  assert.equal(start.hostMonotonicMs, 1000);
  assert.equal(end.hostMonotonicMs, 121000);
  assert(end.observed_sequence > start.observed_sequence);
  const batch = JSON.parse(
    await readFile(resolve(f.root, "diagnostic-export-0001.json"), "utf8"),
  );
  assert.equal(batch.sequence, 1);
  assert.equal(batch.hostMonotonicMs, 3000);
  assert.deepEqual(batch.observations, f.diagnostics(3000).observations);
  assert.equal((await f.request("/reset-origin/start", {})).status, 400);
  assert.equal(
    await readFile(resolve(f.root, "reset-origin-end.json"), "utf8"),
    endBytes,
  );
});

test("exclusive server claim prevents a second clock domain before any samples exist", async (t) => {
  // Arrange
  const f = await serverFixture(t);
  const claim = await readFile(
    resolve(f.root, "reset-origin-server-claim.json"),
    "utf8",
  );
  // Act / Assert
  await assert.rejects(
    createResetOriginSupervisor({ privateRoot: f.root }, f.operations),
    { code: "EEXIST" },
  );
  assert.equal(
    await readFile(resolve(f.root, "reset-origin-server-claim.json"), "utf8"),
    claim,
  );
  assert(
    !(await readdir(f.root)).some((name) =>
      name.startsWith("diagnostic-export-"),
    ),
  );
});

test("real HTTP failure preserves the first closed reason and never writes rejected private metadata", async (t) => {
  // Arrange
  const f = await serverFixture(t);
  await f.prime();
  await f.request("/reset-origin/start", {});
  // Act
  const rejected = await f.request(
    "/diagnostic-export",
    exportOf([{ ...boot, private: "private-rejection-fixture" }]),
  );
  const first = await readFile(
    resolve(f.root, "reset-origin-failure.json"),
    "utf8",
  );
  const reported = await f.request("/reset-origin/failure", {
    code: "observer_client_failed",
  });
  // Assert
  assert.equal(rejected.status, 400);
  assert.equal(reported.status, 200);
  assert.equal(
    await readFile(resolve(f.root, "reset-origin-failure.json"), "utf8"),
    first,
  );
  assert(!first.includes("private-rejection-fixture"));
  assert.equal(JSON.parse(first).code, "object_fields");
  assert.equal(
    (await f.request("/diagnostic-export", f.diagnostics(2000))).status,
    400,
  );
  await assert.rejects(
    readFile(resolve(f.root, "diagnostic-export-0001.json")),
    { code: "ENOENT" },
  );
});

test("read-only server exposes no signer, grant, reset or installation route", async (t) => {
  // Arrange
  const f = await serverFixture(t),
    before = await readdir(f.root);
  // Act / Assert
  for (const path of [
    "/sign",
    "/grant",
    "/window",
    "/restart",
    "/reset",
    "/install",
    "/read-only-interruption",
    "/authorization-context",
  ]) {
    const response = await f.request(path, {});
    assert.equal(response.status, 404);
    assert.equal(response.body.error, "route_unavailable");
  }
  const state = await f.request("/supervisor-state");
  assert.equal(state.body.signing_available, false);
  assert.equal(state.body.mining_authorized, false);
  assert.deepEqual(await readdir(f.root), before);
});

test("incomplete or stale prime and early end fail without manufacturing a completed observation", async (t) => {
  // Arrange / Act / Assert
  for (const scenario of [
    "missing_prime",
    "stale_prime",
    "early_end",
    "late_batch",
  ]) {
    const f = await serverFixture(t);
    if (scenario !== "missing_prime") await f.prime();
    if (scenario === "stale_prime") f.tick(7001);
    if (scenario === "late_batch") {
      assert.equal((await f.request("/reset-origin/start", {})).status, 200);
      f.tick(136001);
      assert.equal(
        (await f.request("/diagnostic-export", f.diagnostics(136001))).status,
        400,
      );
    } else if (scenario === "early_end") {
      assert.equal((await f.request("/reset-origin/start", {})).status, 200);
      f.tick(120999);
      assert.equal((await f.request("/reset-origin/end", {})).status, 400);
    } else
      assert.equal((await f.request("/reset-origin/start", {})).status, 400);
    await assert.rejects(readFile(resolve(f.root, "reset-origin-end.json")), {
      code: "ENOENT",
    });
  }
});

test("server closure before end retains an explicit failure and clears observation ownership", async (t) => {
  // Arrange
  const f = await serverFixture(t);
  await f.prime();
  await f.request("/reset-origin/start", {});
  // Act
  await f.server.closeQualificationResources();
  // Assert
  const failure = JSON.parse(
    await readFile(resolve(f.root, "reset-origin-failure.json"), "utf8"),
  );
  assert.equal(failure.code, "reset_origin_server_closed_before_end");
  assert.equal((await f.request("/reset-origin/end", {})).status, 400);
});

test("an unchanged failed startup snapshot cannot be hidden beside healthy runtime readiness", () => {
  // Arrange
  const failed = { ...startup, stage: "hardware", state: "failed", first_failure: "hardware" };
  // Act / Assert
  assert.throws(() => selectResetOriginDiagnostics(exportOf([startup, failed])), { code: "reset_origin_startup_failure" });
});

test("real page response contains executable HTML rather than a JSON string", async (t) => {
  // Arrange
  const f = await serverFixture(t);
  const original = await readFile(resolve(f.context.no_mining_context.gate_root, f.context.gate_page_relative_path), "utf8");
  // Act
  const response = await fetch(`http://127.0.0.1:${f.server.address().port}/`);
  const page = await response.text();
  // Assert
  assert.equal(response.headers.get("content-type"), "text/html");
  assert.equal(page, `${original}\n<script type="module" src="/no-mining-client.mjs"></script>\n<script type="module" src="/reset-origin-client.mjs"></script>`);
  assert(!page.includes('src=\\"'));
});

test("unattributable preparation metadata is preserved without claiming a current failure", () => {
  // Arrange
  const previous = { category: "worker_preparation_receipt", authoritative: false, origin: "previous_boot", status: "wrong_firmware" };
  const current = { ...previous, origin: "current_boot", status: "unavailable" };
  // Act / Assert
  assert.deepEqual(selectResetOriginDiagnostics(exportOf([previous, current])), [previous, current]);
  for (const changed of [{ ...previous, status: "valid" }, { ...previous, status: "incomplete" },
    { ...previous, origin: "current_boot" }, { ...previous, private: "fixture" }, { ...previous, authoritative: true }]) {
    assert.throws(() => selectResetOriginDiagnostics(exportOf([changed])));
  }
});

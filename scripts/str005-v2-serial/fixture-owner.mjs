import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";
import { networkInterfaces } from "node:os";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { fileDigest, missing } from "../fixed-usb-qualification/contract.mjs";
import { selectInterface } from "../str005-noise-serial/fixture-owner.mjs";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { parseConnection, parseReady } from "./fixture.mjs";
import { readPrivateRecord, writePrivateRecord } from "./private-pipe.mjs";
import { check, ipv4, PROFILE, SCOPES, sha256 } from "./values.mjs";

/** Own a fixture child without placing pool inputs in argv, environment, logs or receipts. */
export async function startFixture(root, context, station, fail, operations = {}) {
  ipv4(station); check(SCOPES.includes(context.scope), "v2_scope");
  const selected = selectInterface(station, (operations.networkInterfaces ?? networkInterfaces)());
  const directory = resolve(root, "fixture-run"); await missing(directory);
  check(await fileDigest(context.fixture_binary) === context.fixture_sha256, "v2_fixture_changed");
  const now = operations.now ?? (() => Math.floor(performance.now())), began = now();
  const contextSha256 = sha256(JSON.stringify(context)), lifetimeMs = context.scope === "channel" ? 150000 : 300000;
  await writeNew(resolve(root, "fixture-start.claim.json"), { schema: "str005-v2-fixture-claim-v1", contextSha256,
    scope: context.scope, attemptId: context.attemptId, binarySha256: context.fixture_sha256, atHostMs: began });
  const child = (operations.spawn ?? spawn)(context.fixture_binary,
    ["--mode", "v2-serial", "--scope", context.scope, "--private-root", directory, "--attempt-id", context.attemptId], {
      detached: true, stdio: ["pipe", "pipe", "pipe", "pipe"],
      env: { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C", RUST_BACKTRACE: "0" },
    });
  let maybeExit = null, maybeOwner = null, maybeReady = null, maybeClosing = null, maybeWriteFailure = null;
  let stderrBytes = 0, readyAtMs = null, exitWrite = Promise.resolve(), resolveExit;
  const exited = new Promise((resolvePromise) => { resolveExit = resolvePromise; });
  function markFailure(code) { fail(code); }
  child.on("error", () => markFailure("v2_fixture_spawn_failed"));
  child.stderr.on("data", (chunk) => {
    stderrBytes += chunk.length;
    if (stderrBytes > 4096) {
      markFailure("v2_fixture_stderr_bound");
      close().catch(() => markFailure("v2_fixture_cleanup_pending"));
    }
  });
  child.once("close", (code, signal) => {
    maybeExit = { code, signal, atHostMs: now() };
    exitWrite = writeNew(resolve(root, "fixture-exit.json"), { schema: "str005-v2-fixture-exit-v1", contextSha256,
      ...maybeExit, owner: maybeOwner, stderrBytes, lifetimeMs: readyAtMs === null ? null : maybeExit.atHostMs - readyAtMs })
      .catch((error) => { maybeWriteFailure = error; markFailure("v2_fixture_exit_evidence_failed"); });
    resolveExit(maybeExit);
    if (code !== 0 && maybeClosing === null) markFailure("v2_fixture_failed");
  });
  const readyOutcome = readPrivateRecord(child.stdout, { deadlineMs: began + 5000, now }).then((value) => ({ value }), () => {
    markFailure("v2_fixture_ready_unobserved"); return { failure: "v2_fixture_ready_unobserved" };
  });
  const connectionRead = readPrivateRecord(child.stdio[3], { deadlineMs: began + 5000 + lifetimeMs, now });
  // Install rejection handling immediately; collection may occur much later than ready.
  const connectionOutcome = connectionRead.then((value) => ({ value }), () => {
    markFailure("v2_fixture_connection_unobserved"); return { failure: "v2_fixture_connection_unobserved" };
  });
  const timer = setTimeout(() => {
    markFailure("v2_fixture_lifetime_expired"); close().catch(() => markFailure("v2_fixture_cleanup_pending"));
  }, lifetimeMs + 5000);
  timer.unref();
  function signal(name) {
    if (maybeExit || !child.pid) return;
    try { process.kill(-child.pid, name); }
    catch (error) { if (error.code !== "ESRCH") throw error; }
  }
  function waitExit(milliseconds) {
    return new Promise((resolveWait) => {
      const timeout = setTimeout(() => resolveWait(null), milliseconds);
      exited.then((value) => { clearTimeout(timeout); resolveWait(value); });
    });
  }
  async function reap() {
    const startedAtMs = now(), natural = maybeExit !== null;
    if (!natural) signal("SIGTERM");
    let result = await waitExit(natural ? 0 : 1500);
    if (!result) { signal("SIGKILL"); result = await waitExit(Math.max(0, 5000 - (now() - startedAtMs))); }
    check(result && (natural || now() - startedAtMs <= 5000), "v2_fixture_cleanup_pending");
    await exitWrite; if (maybeWriteFailure) throw maybeWriteFailure;
    await writeNew(resolve(root, "fixture-reap.json"), { schema: "str005-v2-fixture-reap-v1", contextSha256,
      kind: natural ? "natural_exit" : "requested_stop", requestedAtHostMs: natural ? null : startedAtMs,
      completedAtHostMs: now(), durationMs: natural ? null : now() - startedAtMs });
    return { exited: true, code: result.code };
  }
  function close() {
    clearTimeout(timer); if (maybeClosing === null) maybeClosing = reap(); return maybeClosing;
  }
  try {
    const rows = await (operations.processSnapshot ?? processSnapshot)();
    maybeOwner = rows.find((row) => row.pid === child.pid && row.pgid === child.pid);
    check(maybeOwner, "v2_fixture_owner_missing");
    await writeNew(resolve(root, "fixture-owner.json"), { schema: "str005-v2-fixture-owner-v1", contextSha256,
      owner: maybeOwner, binarySha256: context.fixture_sha256, atHostMs: began });
    const userIdentity = randomBytes(16).toString("base64url");
    await writePrivateRecord(child.stdin, { schema: "str005-v2-fixture-input-v1", scope: context.scope,
      attemptId: context.attemptId, listenIpv4: selected.address, expectedPeerIpv4: station, userIdentity });
    const readiness = await readyOutcome; check(!readiness.failure, "v2_fixture_ready_unobserved");
    maybeReady = parseReady(readiness.value); readyAtMs = now();
    check(readyAtMs - began <= 5000 && maybeReady.attemptId === context.attemptId && maybeReady.scope === context.scope &&
      maybeReady.listenIpv4 === selected.address && maybeExit === null, "v2_fixture_ready_binding");
    await writeNew(resolve(root, "fixture-ready.json"), { schema: "str005-v2-fixture-ready-facts-v1", contextSha256,
      scope: context.scope, attemptId: context.attemptId, instanceId: maybeReady.instanceId,
      authorityPublicKeySha256: sha256(Buffer.from(maybeReady.authorityPublicKey, "base64url")), owner: maybeOwner, readyAtMs });
    return {
      ready: Object.freeze(maybeReady),
      stratum: Object.freeze({ profile: PROFILE, endpoint: `stratum+tcp://${maybeReady.listenIpv4}:${maybeReady.listenPort}/`,
        authorityPublicKey: maybeReady.authorityPublicKey, userIdentity }),
      validateStation(value) { ipv4(value); check(value === station, "v2_fixture_station_changed"); },
      alive() {
        check(maybeExit === null && now() - readyAtMs < lifetimeMs, "v2_fixture_not_live");
      },
      requireStartWindow() { check(maybeExit === null && now() - readyAtMs <= 10000, "v2_fixture_start_deadline"); },
      async connection() {
        const outcome = await connectionOutcome; check(!outcome.failure, "v2_fixture_connection_unobserved");
        const observed = parseConnection(outcome.value);
        check(observed.scope === context.scope && observed.attemptId === context.attemptId && observed.instanceId === maybeReady.instanceId &&
          observed.peerIpv4 === station && observed.localIpv4 === maybeReady.listenIpv4 && observed.localPort === maybeReady.listenPort,
        "v2_fixture_connection_binding");
        return observed;
      },
      async finish() {
        const result = await waitExit(5000);
        check(result && result.code === 0 && stderrBytes === 0 && result.atHostMs - readyAtMs <= lifetimeMs, "v2_fixture_exit_unproved");
        return close();
      },
      close,
    };
  } catch (error) {
    markFailure(typeof error.code === "string" && /^v2_[a-z_]+$/u.test(error.code) ? error.code : "v2_fixture_failed");
    try { await close(); }
    catch {
      await writeNew(resolve(root, "fixture-cleanup-failure.json"), { schema: "str005-v2-fixture-cleanup-failure-v1",
        contextSha256, cause: "v2_fixture_cleanup_pending" });
    }
    await readyOutcome;
    throw error;
  }
}

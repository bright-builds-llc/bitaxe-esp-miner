import { spawn } from "node:child_process";
import { networkInterfaces } from "node:os";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { fileDigest, missing } from "../fixed-usb-qualification/contract.mjs";
import { parseFixtureReady, parseFixtureTerminal } from "./fixture.mjs";
import { privateIpv4 } from "./contract-v2.mjs";
import { check, digest, proof, writeNew } from "./files.mjs";

function ipNumber(ip) { return ip.split(".").reduce((v, part) => ((v * 256) + Number(part)) >>> 0, 0); }
export function selectInterface(station, interfaces = networkInterfaces()) {
  privateIpv4(station);
  const candidates = [];
  for (const [name, rows] of Object.entries(interfaces)) for (const row of rows ?? []) {
    if (row.family !== "IPv4" || row.internal) continue;
    try { privateIpv4(row.address); } catch { continue; }
    const mask = ipNumber(row.netmask);
    if ((ipNumber(station) & mask) === (ipNumber(row.address) & mask)) candidates.push({ name, address: row.address, netmask: row.netmask });
  }
  check(candidates.length === 1, "noise_fixture_interface_ambiguous");
  return candidates[0];
}
const delay = (ms) => new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
/** Own exactly one canonical child; persist closed output classifications only. */
export async function startFixture(root, context, station, fail, operations = {}) {
  const selected = selectInterface(station, (operations.networkInterfaces ?? networkInterfaces)());
  const path = resolve(root, "fixture-run"); await missing(path);
  check(await fileDigest(context.fixture_binary) === context.fixture_sha256, "noise_fixture_changed");
  const began = (operations.now ?? (() => Math.floor(performance.now())))();
  const args = ["--mode", "noise-serial", "--private-root", path, "--listen-address", `${selected.address}:0`,
    "--expected-peer-address", station, "--attempt-id", context.attempt_id,
    "--accept-timeout-seconds", "120", "--read-timeout-seconds", "10", "--lifetime-seconds", "150"];
  await writeNew(resolve(root, "fixture-start.claim.json"), { schema: "noise-serial-fixture-claim-v2", contextSha256: digest(JSON.stringify(context)),
    binarySha256: context.fixture_sha256, atHostMs: began, selected, station });
  const child = (operations.spawn ?? spawn)(context.fixture_binary, args, {
    detached: true, stdio: ["ignore", "pipe", "pipe"],
    env: { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C", RUST_BACKTRACE: "0" },
  });
  const clock = operations.now ?? (() => Math.floor(performance.now()));
  const output = { stdoutBytes: 0, stderrBytes: 0, codes: [] };
  let exit = null, settled = false, ready = null, readyAt = null, maybeClosing = null, maybeOwner = null;
  let exitWrite = Promise.resolve(), reapWrite = null, maybeExitWriteFailure = null;
  let resolveExit;
  const exited = new Promise((resolvePromise) => { resolveExit = resolvePromise; });
  function signal(name) {
    if (settled || !child.pid) return;
    try { process.kill(-child.pid, name); }
    catch (error) { if (error.code !== "ESRCH") throw error; }
  }
  function waitExit(milliseconds) {
    return new Promise((resolveWait) => {
      const timer = setTimeout(() => resolveWait(null), milliseconds);
      exited.then((value) => { clearTimeout(timer); resolveWait(value); });
    });
  }
  async function reap() {
    const started = clock();
    signal("SIGTERM");
    let result = await waitExit(1500);
    if (!result) {
      signal("SIGKILL");
      result = await waitExit(Math.max(0, 5000 - (clock() - started)));
    }
    check(result && clock() - started <= 5000, "noise_fixture_cleanup_pending");
    await exitWrite;
    if (maybeExitWriteFailure) throw maybeExitWriteFailure;
    if (!reapWrite) reapWrite = writeNew(resolve(root, "fixture-reap.json"), {
      schema: "noise-serial-fixture-reap-v2", contextSha256: digest(JSON.stringify(context)),
      startedAtHostMs: started, completedAtHostMs: clock(), durationMs: clock() - started });
    await reapWrite;
    return { exited: true, code: result.code, cleanupMs: clock() - started };
  }
  function close() {
    clearTimeout(timeout);
    if (!maybeClosing) maybeClosing = reap();
    return maybeClosing;
  }
  async function cleanupOnFailure(primary) {
    fail(primary.code ?? "noise_fixture_failed");
    try { await close(); }
    catch (cleanupError) {
      await writeNew(resolve(root, "fixture-cleanup-failure.json"), {
        schema: "noise-serial-fixture-cleanup-failure-v2", contextSha256: digest(JSON.stringify(context)),
        primary: primary.code ?? "noise_fixture_failed", cleanup: cleanupError.code ?? "noise_fixture_cleanup_pending" });
    }
    throw primary;
  }
  for (const name of ["stdout", "stderr"]) child[name].on("data", (bytes) => {
    output[`${name}Bytes`] += bytes.length;
    if (output[`${name}Bytes`] > 1048576) {
      fail("noise_fixture_output_bound");
      close().catch(() => fail("noise_fixture_cleanup_pending"));
    }
    const known = bytes.toString("utf8").match(/noise_serial_(?:fixture_rejected|lifetime_expired|startup_expired)/gu) ?? [];
    for (const code of known) if (!output.codes.includes(code)) output.codes.push(code);
  });
  child.on("error", () => { fail("noise_fixture_spawn_failed"); });
  child.once("close", (code, receivedSignal) => {
    exit = { code, signal: receivedSignal, atHostMs: clock() };
    settled = true;
    exitWrite = writeNew(resolve(root, "fixture-exit.json"), {
      schema: "noise-serial-fixture-exit-v2", contextSha256: digest(JSON.stringify(context)), ...exit,
      owner: maybeOwner, output, lifetimeMs: readyAt === null ? null : exit.atHostMs - readyAt })
      .catch((error) => { maybeExitWriteFailure = error; fail("noise_fixture_exit_evidence_failed"); });
    resolveExit(exit);
    if (code !== 0 && !maybeClosing) fail("noise_fixture_failed");
  });
  const timeout = setTimeout(() => {
    fail("noise_fixture_lifetime_expired");
    close().catch(() => fail("noise_fixture_cleanup_pending"));
  }, 155000);
  timeout.unref();
  try {
    const { processSnapshot } = await import("./host-resources.mjs");
    const rows = await (operations.processSnapshot ?? processSnapshot)();
    const owner = rows.find((row) => row.pid === child.pid && row.pgid === child.pid);
    check(owner, "noise_fixture_identity_missing"); maybeOwner = owner;
    await (operations.writeOwner ?? writeNew)(resolve(root, "fixture-owner.json"), {
      schema: "noise-serial-fixture-owner-v2", contextSha256: digest(JSON.stringify(context)),
      pid: child.pid, group: child.pid, owner, atHostMs: began, binarySha256: context.fixture_sha256 });
    while (clock() - began < 5000) {
      check(!settled, "noise_fixture_not_ready");
      try { ready = parseFixtureReady((await proof(root, "fixture-run/ready.json")).value); break; }
      catch (error) { if (error.code !== "ENOENT") throw error; }
      await delay(10);
    }
    check(ready && ready.attemptId === context.attempt_id && ready.listenIpv4 === selected.address, "noise_fixture_ready_mismatch");
    readyAt = clock(); check(readyAt - began <= 5000, "noise_fixture_ready_timeout");
    await writeNew(resolve(root, "fixture-ready-observation.json"), { contextSha256: digest(JSON.stringify(context)), atHostMs: readyAt,
      readySha256: (await proof(root, "fixture-run/ready.json")).sha256 });
  } catch (error) { return cleanupOnFailure(error); }
  return {
    ready,
    alive() { check(!settled, "noise_fixture_not_live"); check(clock() - readyAt < 150000, "noise_fixture_expired"); },
    async finish() {
      const waitStarted = clock();
      try {
        const result = await waitExit(5000);
        check(result && result.code === 0 && result.atHostMs - readyAt <= 150000, "noise_fixture_exit_unproved");
        clearTimeout(timeout);
        const terminal = parseFixtureTerminal((await proof(root, "fixture-run/terminal.json")).value);
        check(terminal.outcome === "accepted" && terminal.attemptId === context.attempt_id, "noise_fixture_result");
        await exitWrite;
        if (maybeExitWriteFailure) throw maybeExitWriteFailure;
        if (!reapWrite) reapWrite = writeNew(resolve(root, "fixture-reap.json"), {
          schema: "noise-serial-fixture-reap-v2", contextSha256: digest(JSON.stringify(context)),
          startedAtHostMs: waitStarted, completedAtHostMs: clock(), durationMs: clock() - waitStarted });
        await reapWrite;
        return terminal;
      } catch (error) { return cleanupOnFailure(error); }
    },
    close,
  };
}

import test from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { mkdtemp, mkdir, chmod, readFile, writeFile, realpath, rm } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { randomBytes } from "node:crypto";
import { exchange } from "./operator-ipc.mjs";
import { digest, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
const HERE = dirname(fileURLToPath(import.meta.url)), execute = promisify(execFile), pause = ms => new Promise(done => setTimeout(done, ms));
async function eventually(action) {
  const deadline = Date.now() + 10000;
  while (true) { const value = await action(); if (value) return value; assert.ok(Date.now() < deadline, "bounded rehearsal wait"); await pause(25); }
}
async function fixture(t) {
  const base = await mkdtemp(resolve(await realpath("/tmp"), "v2op-rehearsal-")); await chmod(base, 0o700);
  const root = resolve(base, "scratch/str005-v2-serial/channel-005"), modules = resolve(base, "scripts/str005-v2-serial"), noise = resolve(base, "scripts/str005-noise-serial");
  for (const path of [root, modules, noise]) await mkdir(path, { recursive: true, mode: 0o700 });
  const copy = async (from, to) => writeFile(resolve(modules, to), await readFile(resolve(HERE, from)), { mode: 0o600 });
  for (const name of ["operator-client.mjs", "operator-daemon.mjs", "operator-ipc.mjs", "operator-state.mjs", "operator-parent.mjs", "operator-rehearsal-launch.mjs", "operator-rehearsal-request.mjs"]) await copy(name, name);
  for (const name of ["cleanup.mjs", "cleanup-session.mjs", "values.mjs", "journal.mjs", "host-resources.mjs", "operator-disposition.mjs"]) await writeFile(resolve(modules, name), `export * from ${JSON.stringify(new URL(name, import.meta.url).href)};\n`, { mode: 0o600 });
  for (const name of ["files.mjs", "host-resources.mjs", "node-runtime.mjs"]) await writeFile(resolve(noise, name), `export * from ${JSON.stringify(new URL(`../str005-noise-serial/${name}`, import.meta.url).href)};\n`, { mode: 0o600 });
  await copy("operator-rehearsal-context.mjs", "context.mjs"); await copy("operator-rehearsal-supervisor.mjs", "operator-supervisor.mjs"); await copy("operator-rehearsal-install.mjs", "operator.mjs");
  const context = { schema: "str005-v2-serial-context-v5", scope: "channel", firmware_root: base, hostOrdinal: 5, install_indices: [0], evaluator: [{ path: "scripts/str005-v2-serial/operator-daemon.mjs", sha256: digest(await readFile(resolve(HERE, "operator-daemon.mjs"))) }] };
  await writeNew(resolve(root, "synthetic-only.json"), { schema: "operator-loss-software-only-v1" }); await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  t.after(async () => {
    for (const path of [[`${root}.operator`, "locator.json"], [root, "server-owner.json"]]) {
      let owner;
      try { owner = (await proof(...path)).value.owner; } catch (error) { if (error.code !== "ENOENT") throw error; }
      if (owner && (await processSnapshot()).some(row => sameProcess(row, owner))) {
        process.kill(-owner.pgid, "SIGKILL"); await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, owner)));
      }
    }
    try {
      const locator = (await proof(`${root}.operator`, "locator.json")).value;
      await rm(dirname(locator.socketPath), { recursive: true, force: true });
    } catch (error) { if (error.code !== "ENOENT") throw error; }
    await rm(base, { recursive: true, force: true });
  });
  return { root, modules, contextSha256: digest(JSON.stringify(context)) };
}
test("daemon survives launcher EOF and exit, retains child observation across disconnected operation client", { timeout: 20000 }, async t => {
  // Arrange: copied unchanged production daemon/client, synthetic device admission only.
  const f = await fixture(t);
  const launch = await execute(process.execPath, [resolve(f.modules, "operator-rehearsal-launch.mjs"), f.root], { timeout: 10000 }).catch(async error => {
    error.message += await readFile(resolve(`${f.root}.operator`, "daemon.stderr.log"), "utf8"); throw error;
  });
  assert.equal(JSON.parse(launch.stdout).phase, "initializing");
  const locator = (await proof(`${f.root}.operator`, "locator.json")).value;
  const query = { schema: "str005-v2-operator-query-v1", contextSha256: f.contextSha256, maybeRequestId: null };
  await eventually(async () => (await exchange(locator.socketPath, query)).phase === "ready");
  const client = await import(new URL(`file://${f.modules}/operator-client.mjs`));
  assert.equal((await client.operatorStatus({ privateRoot: f.root })).phase, "ready");
  // Act: each short client exits; duplicate IDs query the same in-flight operation.
  const request = { schema: "str005-v2-operator-request-v1", contextSha256: f.contextSha256, requestId: randomBytes(16).toString("hex"), action: "install", payload: { index: 0 } };
  const requestFile = resolve(f.root, "synthetic-request.json"); await writeNew(requestFile, request);
  await execute(process.execPath, [resolve(f.modules, "operator-rehearsal-request.mjs"), locator.socketPath, requestFile], { timeout: 5000 });
  assert.equal((await client.operatorRequest({ privateRoot: f.root, requestFile })).status, "pending");
  await eventually(async () => (await exchange(locator.socketPath, { ...query, maybeRequestId: request.requestId })).status === "succeeded");
  assert.ok((await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  const finish = { ...request, requestId: randomBytes(16).toString("hex"), action: "finish-cleanup", payload: { browserWitness: null } };
  await exchange(locator.socketPath, finish);
  await eventually(async () => (await exchange(locator.socketPath, { ...query, maybeRequestId: finish.requestId })).status === "failed");
  // Assert: missing UI witness stays a failure, but actual supervisor zero exit is retained.
  const exit = (await proof(f.root, "parent-cleanup-supervisor.json")).value;
  assert.equal(exit.code, 0); assert.equal(exit.source, "parent-observed");
  const stop = { ...request, requestId: randomBytes(16).toString("hex"), action: "stop", payload: {} };
  await exchange(locator.socketPath, stop);
  await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  assert.equal((await proof(`${f.root}.operator`, "stopped.json")).value.phase, "stopped");
  assert.equal((await client.operatorStatus({ privateRoot: f.root })).phase, "stopped");
});

test("closed PTY does not own daemon lifetime; daemon death cannot be adopted or restarted", { timeout: 20000 }, async t => {
  const f = await fixture(t);
  await execute("python3", [resolve(HERE, "operator-rehearsal-pty.py"), process.execPath, resolve(f.modules, "operator-rehearsal-launch.mjs"), f.root], { timeout: 15000 });
  const locator = (await proof(`${f.root}.operator`, "locator.json")).value;
  const query = { schema: "str005-v2-operator-query-v1", contextSha256: f.contextSha256, maybeRequestId: null };
  await eventually(async () => (await exchange(locator.socketPath, query)).phase === "ready");
  assert.ok((await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  process.kill(locator.owner.pid, "SIGKILL");
  await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  const client = await import(new URL(`file://${f.modules}/operator-client.mjs`));
  await assert.rejects(client.operatorStatus({ privateRoot: f.root }));
  await assert.rejects(client.operatorStart({ privateRoot: f.root }), { code: "EEXIST" });
  await assert.rejects(proof(`${f.root}.operator`, "stopped.json"), { code: "ENOENT" });
});
test("handled daemon stop waits for admitted bounded operation and retains actual child exit", { timeout: 20000 }, async t => {
  const f = await fixture(t);
  await execute(process.execPath, [resolve(f.modules, "operator-rehearsal-launch.mjs"), f.root], { timeout: 10000 });
  const locator = (await proof(`${f.root}.operator`, "locator.json")).value;
  const query = { schema: "str005-v2-operator-query-v1", contextSha256: f.contextSha256, maybeRequestId: null };
  await eventually(async () => (await exchange(locator.socketPath, query)).phase === "ready");
  const request = { schema: "str005-v2-operator-request-v1", contextSha256: f.contextSha256, requestId: randomBytes(16).toString("hex"), action: "install", payload: { index: 0 } };
  await exchange(locator.socketPath, request);
  process.kill(locator.owner.pid, "SIGTERM");
  await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  assert.equal((await proof(`${f.root}.operator`, `result-${request.requestId}.json`)).value.status, "succeeded");
  assert.equal((await proof(f.root, "parent-cleanup-supervisor.json")).value.code, 0);
  assert.equal((await proof(`${f.root}.operator`, "disposition.json")).value.cleanupRecorded, false);
});
for (const stopMode of ["request", "signal"]) test(`result write failure cannot strand the real daemon owners: ${stopMode}`, { timeout: 20000 }, async t => {
  const f = await fixture(t);
  await execute(process.execPath, [resolve(f.modules, "operator-rehearsal-launch.mjs"), f.root], { timeout: 10000 });
  const locator = (await proof(`${f.root}.operator`, "locator.json")).value;
  const query = { schema: "str005-v2-operator-query-v1", contextSha256: f.contextSha256, maybeRequestId: null };
  await eventually(async () => (await exchange(locator.socketPath, query)).phase === "ready");
  const input = { schema: "str005-v2-operator-request-v1", contextSha256: f.contextSha256, requestId: randomBytes(16).toString("hex"), action: "install", payload: { index: 0 } };
  await exchange(locator.socketPath, input);
  // Real filesystem conflict after acceptance, while the bounded operation runs.
  await mkdir(resolve(`${f.root}.operator`, `result-${input.requestId}.json`), { mode: 0o700 });
  await eventually(async () => (await exchange(locator.socketPath, query)).phase === "failed");
  if (stopMode === "signal") process.kill(locator.owner.pid, "SIGTERM");
  else {
    const cleanup = { ...input, requestId: randomBytes(16).toString("hex"), action: "finish-cleanup", payload: { browserWitness: null } };
    await exchange(locator.socketPath, cleanup);
    await eventually(async () => {
      const result = await exchange(locator.socketPath, { ...query, maybeRequestId: cleanup.requestId });
      if (result.status === "pending") return false;
      assert.equal(result.code, "v2_operator_failed"); return true;
    });
    await exchange(locator.socketPath, { ...input, requestId: randomBytes(16).toString("hex"), action: "stop", payload: {} });
  }
  await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, locator.owner)));
  assert.equal((await proof(f.root, "parent-cleanup-supervisor.json")).value.code, 0);
  assert.equal((await proof(`${f.root}.operator`, "disposition.json")).value.cleanupRecorded, false);
  const client = await import(new URL(`file://${f.modules}/operator-client.mjs`));
  await assert.rejects(client.operatorStatus({ privateRoot: f.root }), "missing result cannot become complete evidence");
});

test("parent loss between IPC admission and startup gate cannot create a supervisor listener", { timeout: 15000 }, async t => {
  const f = await fixture(t);
  await writeNew(resolve(f.root, "synthetic-startup-loss.json"), { enabled: true });
  await execute(process.execPath, [resolve(f.modules, "operator-rehearsal-launch.mjs"), f.root], { timeout: 10000 });
  const locator = (await proof(`${f.root}.operator`, "locator.json")).value;
  let owner;
  await eventually(async () => {
    try { owner = (await proof(f.root, "synthetic-startup-owner.json")).value; return true; }
    catch (error) { if (error.code !== "ENOENT") throw error; return false; }
  });
  process.kill(locator.owner.pid, "SIGKILL");
  await eventually(async () => !(await processSnapshot()).some(row => sameProcess(row, owner)));
  await assert.rejects(proof(f.root, "server-owner.json"), { code: "ENOENT" });
  await assert.rejects(proof(f.root, "parent-cleanup-supervisor.json"), { code: "ENOENT" });
});

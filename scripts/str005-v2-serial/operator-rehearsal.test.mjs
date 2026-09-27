import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import test from "node:test";
import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";
import { mkdtemp, mkdir, chmod, readFile, writeFile, realpath, rm, lstat, readdir } from "node:fs/promises";
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
  const root = resolve(base, "scratch/str005-v2-serial/channel-006"), modules = resolve(base, "scripts/str005-v2-serial"), noise = resolve(base, "scripts/str005-noise-serial");
  for (const path of [root, modules, noise]) await mkdir(path, { recursive: true, mode: 0o700 });
  const copy = async (from, to) => writeFile(resolve(modules, to), await readFile(resolve(HERE, from)), { mode: 0o600 });
  for (const name of ["operator-client.mjs", "operator-daemon.mjs", "operator-ipc.mjs", "operator-state.mjs", "operator-parent.mjs", "operator-rehearsal-launch.mjs", "operator-rehearsal-request.mjs"]) await copy(name, name);
  for (const name of ["cleanup.mjs", "cleanup-session.mjs", "values.mjs", "journal.mjs", "host-resources.mjs", "operator-disposition.mjs"]) await writeFile(resolve(modules, name), `export * from ${JSON.stringify(new URL(name, import.meta.url).href)};\n`, { mode: 0o600 });
  for (const name of ["files.mjs", "host-resources.mjs", "node-runtime.mjs"]) await writeFile(resolve(noise, name), `export * from ${JSON.stringify(new URL(`../str005-noise-serial/${name}`, import.meta.url).href)};\n`, { mode: 0o600 });
  await copy("operator-rehearsal-context.mjs", "context.mjs"); await copy("operator-rehearsal-supervisor.mjs", "operator-supervisor.mjs"); await copy("operator-rehearsal-install.mjs", "operator.mjs");
  const context = { schema: "str005-v2-serial-context-v6", scope: "channel", firmware_root: base, hostOrdinal: 6, install_indices: [0], evaluator: [{ path: "scripts/str005-v2-serial/operator-daemon.mjs", sha256: digest(await readFile(resolve(HERE, "operator-daemon.mjs"))) }] };
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

async function observedDaemon(t, f, { locatorFailure = false, acknowledgmentFailure = false } = {}) {
  const directory = `${f.root}.operator`; await mkdir(directory, { mode: 0o700 });
  await writeNew(resolve(directory, "start.claim.json"), { schema: "str005-v2-operator-start-v1", contextSha256: f.contextSha256 });
  if (locatorFailure) await mkdir(resolve(directory, "locator.json"), { mode: 0o700 });
  let program = resolve(f.modules, "operator-daemon.mjs");
  if (locatorFailure || acknowledgmentFailure) {
    // Entry and caught-error witnesses prevent a launcher failure from passing this test.
    program = resolve(f.modules, "synthetic-preack-failure.mjs");
    await writeFile(program, `import {runOperator} from './operator-daemon.mjs';
import {writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
const directory=process.argv[2]+'.operator';
const record=(name,value)=>writeFile(resolve(directory,name),JSON.stringify(value),{flag:'wx',mode:0o600});
let acknowledgmentAttempted=false;
${acknowledgmentFailure ? "process.send = (_value, done) => {acknowledgmentAttempted=true;done(Object.assign(Error('synthetic IPC failure'), {code:'EIO'}));};" : ""}
process.once('message', async input => {
  await record('synthetic-entered.json',{schema:'v2-daemon-fault-test-v1',entered:true});
  try {await runOperator(process.argv[2],input);}
  catch(error) {await record('synthetic-fault.json',{schema:'v2-daemon-fault-test-v1',code:error.code,acknowledgmentAttempted});process.exitCode=1;if(process.connected)process.disconnect();}
});\n`, { mode: 0o600 });
  }
  const socketParent = resolve(f.modules,"../..","sockets"); await mkdir(socketParent,{recursive:true,mode:0o700});
  const child = spawn(process.execPath, [program, f.root], { detached: true, stdio: ["ignore", "ignore", "pipe", "ipc"], env: {PATH:process.env.PATH,HOME:process.env.HOME,LANG:"C",LC_ALL:"C",TMPDIR:socketParent,...nodeRuntimeEnvironment()} });
  let stderr=""; child.stderr.on("data", chunk=>{stderr+=chunk;});
  const closed = new Promise(done => child.once("exit", (code, signal) => done({code, signal})));
  t.after(async () => { if (child.exitCode === null && child.signalCode === null) { child.kill("SIGKILL"); await bounded(closed); }
    if (locatorFailure) await rm(resolve(directory, "locator.json"), { recursive: true, force: true }); });
  const ready = new Promise((done,reject) => {child.once("message",done);child.once("error",reject);child.once("close",()=>reject(Error(`daemon closed before ack: ${stderr}`)));});
  // Pre-ack fault cases assert actual exit instead of awaiting acknowledgment.
  void ready.catch(error => ({error}));
  child.send({ schema: "str005-v2-operator-bootstrap-v1", maybeAuthorityDirectory: null });
  return { child, closed, ready, directory };
}
async function bounded(promise, ms = 10000) {
  let timer; try { return await Promise.race([promise, new Promise((_, reject) => {timer=setTimeout(()=>reject(Error("daemon-fault deadline")),ms);})]); }
  finally { clearTimeout(timer); }
}
for (const fault of ["stop-result", "supervisor-receipt"]) test(`real V2 daemon releases socket despite ${fault} evidence failure`, { timeout: 30000 }, async t => {
  // Arrange
  const f = await fixture(t), daemon = await observedDaemon(t, f); await bounded(daemon.ready); daemon.child.disconnect();
  const locator = (await proof(daemon.directory, "locator.json")).value;
  const query = id => exchange(locator.socketPath, {schema:"str005-v2-operator-query-v1",contextSha256:f.contextSha256,maybeRequestId:id});
  await eventually(async () => { const value=await query(null); assert.notEqual(value.phase,"failed",JSON.stringify(value)); return value.phase === "ready"; });
  if (fault === "supervisor-receipt") await mkdir(resolve(daemon.directory,"supervisor-close.json"),{mode:0o700});
  const request = (id, action, payload) => ({schema:"str005-v2-operator-request-v1",contextSha256:f.contextSha256,requestId:id.repeat(32),action,payload});
  // Act: actual exclusive filesystem writes fail; cleanup has a real supervisor to reap.
  await exchange(locator.socketPath,request("1","finish-cleanup",{browserWitness:null}));
  await eventually(async () => (await query("1".repeat(32))).status !== "pending");
  if (fault === "stop-result") await mkdir(resolve(daemon.directory,`result-${"2".repeat(32)}.pending.json`),{mode:0o700});
  await exchange(locator.socketPath,request("2","stop",{}));
  const exit = await bounded(daemon.closed);
  // Assert
  assert.deepEqual(exit,{code:1,signal:null});
  assert.equal((await processSnapshot()).some(row=>sameProcess(row,locator.owner)),false);
  await assert.rejects(lstat(locator.socketPath),{code:"ENOENT"});
  const disposition=(await proof(daemon.directory,"disposition.json")).value;
  assert.equal(disposition.hostStopped,true);assert.equal(disposition.cleanupRecorded,false);
  assert.notEqual((await proof(daemon.directory,"stopped.json")).value.maybeCode,null);
  if(fault==="stop-result") await assert.rejects(lstat(resolve(daemon.directory,`result-${"2".repeat(32)}.json`)),{code:"ENOENT"});
});
for (const fault of ["locator", "acknowledgment"]) test(`real V2 ${fault} failure closes pre-supervisor listener`, {timeout:20000}, async t=>{
  // Arrange
  const f=await fixture(t), parent=resolve(f.modules,"../..","sockets");
  await mkdir(parent,{mode:0o700}); const before=new Set(await readdir(parent));
  const daemon=await observedDaemon(t,f,{locatorFailure:fault==="locator",acknowledgmentFailure:fault==="acknowledgment"});
  // Act
  const exit=await bounded(daemon.closed);
  // Assert
  assert.deepEqual(exit,{code:1,signal:null});
  assert.deepEqual((await proof(daemon.directory,"synthetic-entered.json")).value,{schema:"v2-daemon-fault-test-v1",entered:true});
  const witness=(await proof(daemon.directory,"synthetic-fault.json")).value;
  assert.equal(witness.schema,"v2-daemon-fault-test-v1");
  assert.equal(witness.acknowledgmentAttempted,fault==="acknowledgment");
  if(fault==="locator") assert.ok(["EEXIST","EISDIR"].includes(witness.code));
  else assert.equal(witness.code,"EIO");
  assert.deepEqual((await readdir(parent)).filter(name=>name.startsWith("v2op-")&&!before.has(name)),[]);
  assert.equal((await processSnapshot()).some(row=>row.pid===daemon.child.pid||row.ppid===daemon.child.pid||row.pgid===daemon.child.pid),false);
  await assert.rejects(lstat(resolve(daemon.directory,"supervisor-start.json")),{code:"ENOENT"});
  if(fault==="locator") await rm(resolve(daemon.directory,"locator.json"),{recursive:true});
});

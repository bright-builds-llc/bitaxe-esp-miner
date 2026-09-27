import { spawn } from "node:child_process";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import { randomBytes } from "node:crypto";
import { chmod, mkdtemp, open, readFile, realpath, rmdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { privateRoot, proof, writeNew, digest } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireGone, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { createCleanupSession } from "./cleanup-session.mjs";
import { requireHostStopped } from "./cleanup.mjs";
import { install as installCandidate } from "./install.mjs";
import { readJournal } from "./journal.mjs";
import { serveOperatorSocket } from "../str005-v2-serial/operator-ipc.mjs";
import { createRequestStore, operatorCode, queryShape, requestShape } from "./operator-state.mjs";
import { check, object, sha256 } from "./values.mjs";
const pause = ms => new Promise(done => setTimeout(done, ms));
const bound = async (promise, ms) => { let timer; try { return await Promise.race([promise, new Promise(done => { timer = setTimeout(() => done(false), ms); })]); } finally { clearTimeout(timer); } };
const own = async () => (await processSnapshot()).find(row => row.pid === process.pid);
/** The daemon, never a reconnecting CLI, remains the real supervisor parent. */
export async function runOperator(root, bootstrap, operations = {}) {
  object(bootstrap, ["schema"]);
  check(bootstrap.schema === "usb-bootstrap-measure-operator-bootstrap-v1", "bootstrap_operator_request");
  const { loadOperatorContext, loadEffectContext, verifyCleanupInputs } = await import("./context.mjs");
  const context = await loadOperatorContext(root, operations), contextSha256 = sha256(JSON.stringify(context)), directory = `${root}.operator`;
  await privateRoot(directory);
  const owner = await own(); check(owner && owner.pid === owner.pgid, "bootstrap_operator_owner");
  const socketDirectory = await mkdtemp(resolve(await realpath(tmpdir()), "ubmop-"));
  try { await chmod(socketDirectory, 0o700); }
  catch (error) { try { await rmdir(socketDirectory); } catch (cleanupError) { error.cause ??= cleanupError; } throw error; }
  const socketPath = resolve(socketDirectory, "s"), store = createRequestStore(directory, contextSha256, context.installIndices);
  let phase = "initializing", maybeRequest = null, maybeCode = null, maybeOrigin = null, lastSequence = null, lastStateSha256 = null;
  let maybeCleanup, maybeChild, maybeChildOwner, maybeActive, maybeCloseWrite, maybeChildClosed, closing = false, stoppingRequested = false, cleanupRecorded = false;
  let finishInitialization; const initialized = new Promise(done => { finishInitialization = done; });
  const status = () => ({ schema: "usb-bootstrap-measure-operator-status-v1", contextSha256, phase, maybeRequestId: maybeRequest?.requestId ?? null,
    maybeAction: maybeRequest?.action ?? null, maybeIndex: maybeRequest?.payload.index ?? null, maybeCode,
    maybeSupervisorOrigin: maybeOrigin, lastSequence, lastStateSha256 });
  async function refreshJournal() {
    try { const last = (await readJournal(root, context)).at(-1); if (last) { lastSequence = last.sequence; lastStateSha256 = digest(JSON.stringify(last)); } }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  let server;
  try { server = await serveOperatorSocket(socketPath, async input => {
    await refreshJournal();
    if (input.schema === "usb-bootstrap-measure-operator-query-v1") {
      queryShape(input, contextSha256); return input.maybeRequestId === null ? status() : store.get(input.maybeRequestId);
    }
    requestShape(input, contextSha256, context.installIndices);
    const accepted = await store.accept(input);
    if (!accepted.first) return accepted.value;
    if (stoppingRequested || maybeActive || !["ready", "failed"].includes(phase) || (phase === "failed" && !["finish-cleanup", "stop"].includes(input.action))) {
      return store.finish(input, "bootstrap_operator_busy");
    }
    maybeRequest = input; phase = "busy";
    maybeActive = (async () => {
      let stopAdmitted = false;
      try {
        let maybeOperationCode = null;
        try { await execute(input); stopAdmitted = input.action === "stop"; }
        catch (error) { maybeOperationCode = operatorCode(error); maybeCode ??= maybeOperationCode; phase = "failed"; }
        await store.finish(input, maybeOperationCode);
        if (phase === "busy") phase = "ready";

      } catch (error) { recordFailure(error); }
      finally {
        if (stopAdmitted) { try { await close(); } catch (error) { recordFailure(error); } }
        maybeActive = null;
      }
    })();
    return accepted.value;
  }); } catch (error) {
    try { await rmdir(socketDirectory); } catch (cleanupError) { error.cause ??= cleanupError; }
    throw error;
  }
  try {
    await writeNew(resolve(directory, "locator.json"), { schema: "usb-bootstrap-measure-operator-locator-v1", contextSha256, owner, socketPath });
    if (process.connected) await new Promise((done, reject) => process.send(status(), error => {
      // Losing the reconnectable launcher does not revoke the durable owner.
      if (error && error.code !== "ERR_IPC_CHANNEL_CLOSED") reject(error); else done();
    }));
  } catch (error) {
    // No supervisor exists yet. Release the only live resource even when its locator is missing.
    recordFailure(error);
    try { await new Promise(done => server.close(done)); } catch (cleanupError) { recordFailure(cleanupError); }
    try { await rmdir(socketDirectory); } catch (cleanupError) { recordFailure(cleanupError); }
    if (process.connected) process.disconnect();
    throw error;
  }
  async function execute(input) {
    check((await processSnapshot()).some(row => sameProcess(row, owner)), "bootstrap_operator_owner");
    if (input.action === "install") { await loadEffectContext(root, operations); const result = await (operations.installCandidate ?? installCandidate)(root, input.payload.index, operations); check(result.install_measured === true, "bootstrap_operator_failed"); }

    if (input.action === "finish-cleanup") {
      try { await verifyCleanupInputs(context, operations); }
      catch (error) { try { await maybeCleanup.finish(null); } catch { /* Its persisted failure cannot upgrade source drift. */ } throw error; }
      const result = await maybeCleanup.finish(input.payload.browserWitness); cleanupRecorded = result.cleanup_recorded === true;
    }
    if (input.action === "stop") { await requireHostStopped(root, context, operations); phase = "stopping"; }
  }
  function recordFailure(error) {
    maybeCode ??= operatorCode(error); phase = "failed"; cleanupRecorded = false;
    process.exitCode = 1;
  }
  async function close() {
    if (closing) return;
    // Failed evidence cannot strand a stopped owner, but live children still block closure.
    await requireHostStopped(root, context, operations);
    closing = true;
    if (maybeCloseWrite) { try { await maybeCloseWrite; } catch (error) { recordFailure(error); } }
    await new Promise(done => server.close(done));
    try { await rmdir(socketDirectory); } catch (error) { recordFailure(error); }
    try {
      try { await refreshJournal(); } catch (error) { recordFailure(error); }
      phase = "stopped";
      try { await writeNew(resolve(directory, "stopped.json"), status()); } catch (error) { recordFailure(error); }
      let maybeObservationSha256 = null;
      try { maybeObservationSha256 = (await proof(root, "parent-cleanup-supervisor-observation.json")).sha256; }
      catch (error) { if (error.code !== "ENOENT") recordFailure(error); }
      try {
        await writeNew(resolve(directory, "disposition.json"), { schema: "usb-bootstrap-measure-operator-disposition-v1", contextSha256, owner,
          sourceSha256: digest(await readFile(fileURLToPath(import.meta.url))), hostStopped: true, cleanupRecorded,
          supervisorObservationSha256: maybeObservationSha256 });
      } catch (error) { recordFailure(error); }
    } finally {
      process.removeListener("SIGTERM", signalStop); process.removeListener("SIGINT", signalStop);
      if (process.connected) process.disconnect();
    }
  }
  async function signalStop() {
    if (closing || stoppingRequested) return;
    stoppingRequested = true;
    try {
      await initialized;
      if (maybeActive) { try { await maybeActive; } catch (error) { maybeCode ??= operatorCode(error); phase = "failed"; } }
      if (maybeCleanup && !cleanupRecorded) { try { await maybeCleanup.finish(null); } catch { phase = "failed"; } } await close(); }
    catch { phase = "failed"; maybeCode ??= "bootstrap_operator_failed"; }
  }
  process.once("SIGTERM", signalStop); process.once("SIGINT", signalStop);
  try {
    const nonce = randomBytes(32).toString("hex");
    await writeNew(resolve(directory, "supervisor-start.json"), { contextSha256, nonceSha256: sha256(nonce) });
    const out = await open(resolve(directory, "supervisor.stdout.log"), "wx", 0o600), err = await open(resolve(directory, "supervisor.stderr.log"), "wx", 0o600);
    maybeChild = spawn(process.execPath, [(operations.supervisorProgram ?? fileURLToPath(new URL("./operator-supervisor.mjs", import.meta.url))), root],
      { detached: true, stdio: ["ignore", out.fd, err.fd, "ipc"], env: { PATH: process.env.PATH, HOME: process.env.HOME, LANG: "C", LC_ALL: "C", ...(process.env.TZ ? { TZ: process.env.TZ } : {}), ...nodeRuntimeEnvironment() } });
    let exited = false, finishChildClose;
    maybeChildClosed = new Promise(done => { finishChildClose = done; });
    // Attach before the first await after spawn. Even startup exits are real events.
    maybeChild.once("close", (code, signal) => {
      exited = true; finishChildClose();
      if (!closing && !stoppingRequested && maybeRequest?.action !== "finish-cleanup") {
        maybeCode ??= "bootstrap_operator_failed"; phase = "failed";
      }
      maybeCloseWrite = writeNew(resolve(directory, "supervisor-close.json"), {
        schema: "usb-bootstrap-measure-operator-child-exit-v1", source: "parent-observed", contextSha256,
        parent: owner, childPid: maybeChild.pid, code, signal, observedAtUnixMs: Date.now(),
      });
      maybeCloseWrite.catch(recordFailure);
    });
    maybeChild.once("error", () => { exited = true; });
    operations.onSupervisorSpawn?.(maybeChild);
    await out.close(); await err.close();
    for (let i = 0; i < 30 && !maybeChildOwner && !exited; i++) { maybeChildOwner = (await processSnapshot()).find(row => row.pid === maybeChild.pid && row.pgid === maybeChild.pid); if (!maybeChildOwner) await pause(50); }
    check(maybeChildOwner && !exited, "bootstrap_operator_initialization");
    await writeNew(resolve(directory, "supervisor-owner.json"), { schema: "usb-bootstrap-measure-operator-supervisor-owner-v1", contextSha256, owner: maybeChildOwner });
    maybeCleanup = createCleanupSession(root, context, { child: maybeChild, owner: maybeChildOwner, operations });
    maybeChild.send({ binding: { schema: "usb-bootstrap-measure-operator-child-start-v1", contextSha256, nonce } });
    const deadline = performance.now() + 900000;
    while (true) {
      check(!exited && performance.now() < deadline, "bootstrap_operator_initialization");
      const output = await readFile(resolve(directory, "supervisor.stdout.log"), "utf8"); check(Buffer.byteLength(output) <= 65536, "bootstrap_operator_initialization");
      if (/^qualification_url=http:\/\/127\.0\.0\.1:[1-9][0-9]*\/\n/mu.test(output)) break;
      await pause(100);
    }
    await loadEffectContext(root, operations);
    const supervisor = (await proof(root, "server-owner.json")).value; check(sameProcess(supervisor.owner, maybeChildOwner), "bootstrap_operator_owner");
    maybeOrigin = supervisor.origin; phase = "ready";
  } catch (error) {
    phase = "failed"; maybeCode ??= operatorCode(error);
    if (maybeCleanup) { try { await maybeCleanup.finish(null); } catch { /* The helper persists its real failure and reaps actual owners. */ } }
    else if (maybeChild) {
      // No startup IPC was sent before ownership and the cleanup observer existed.
      // This child cannot have admitted hardware; stop the actual spawned handle.
      maybeChild.kill("SIGTERM");
      const stopped = await bound(maybeChildClosed.then(() => true), 3000);
      if (!stopped) { maybeChild.kill("SIGKILL"); await maybeChildClosed; }
      if (maybeChildOwner) await requireGone([maybeChildOwner]);
    }
  } finally { finishInitialization(); }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.umask(0o077);
  check(process.connected && process.argv.length === 3, "bootstrap_operator_owner");
  const timer = setTimeout(() => process.exit(1), 9000);
  process.once("message", input => { clearTimeout(timer); runOperator(resolve(process.argv[2]), input).catch(() => { process.exitCode = 1; if (process.connected) process.disconnect(); }); });
}

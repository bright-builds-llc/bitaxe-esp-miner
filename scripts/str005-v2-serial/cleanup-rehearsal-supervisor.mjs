// Test-only child. No Serial API or hardware command exists in this process.
import { once } from "node:events";
import { createSupervisor } from "./server.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { rehearsalOperations } from "./cleanup-rehearsal-operations.mjs";

if (!process.send) throw Error("rehearsal_ipc_required");
let sequence = 0, maybeServer, stopping = false, maybeClosing;
const pending = new Map();
const request = (method, args) => new Promise((done, reject) => {
  const id = ++sequence; pending.set(id, { done, reject }); process.send({ kind: "prerequisite", id, method, args });
});
process.on("message", message => {
  if (message.kind !== "prerequisite-result") return;
  const entry = pending.get(message.id); if (!entry) return;
  pending.delete(message.id);
  if (message.error) entry.reject(Object.assign(Error(message.error), { code: message.error })); else entry.done(message.value);
});
async function close(code) {
  if (maybeClosing) return maybeClosing;
  stopping = true;
  // A killed test runner cannot leave a helper awaiting unfinished IPC forever.
  // This is failed test-only teardown, never a successful qualification exit.
  setTimeout(() => process.exit(1), 6000).unref();
  maybeClosing = (async () => {
    try { if (maybeServer) await maybeServer.closeQualificationResources(); }
    catch { code = 1; }
    process.exit(code);
  })();
  return maybeClosing;
}
process.once("SIGTERM", () => { void close(0); });
process.once("disconnect", () => { void close(1); });
const watchdog = setTimeout(() => { void close(1); }, 120000); watchdog.unref();
try {
  const [input] = await once(process, "message");
  if (input.kind !== "initialize") throw Error("rehearsal_initialize");
  const { options, context } = input;
  await syntheticRoot(options.privateRoot, context);
  const operations = rehearsalOperations(context, request);
  maybeServer = await createSupervisor(options, operations);
  if (stopping) await close(1);
  maybeServer.listen(0, "127.0.0.1"); await once(maybeServer, "listening"); await maybeServer.qualificationReady;
  process.send({ kind: "ready" });
} catch (error) {
  const code = /^(?:v2|noise|iterative)_[a-z_]+$/u.test(error?.code) ? error.code : "rehearsal_supervisor_failed";
  if (process.connected) process.send({ kind: "failed", code });
  await close(1);
}

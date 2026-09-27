// Test-only daemon shell: only cold prerequisites cross the runner IPC boundary.
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { runOperator } from "./operator-daemon.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { rehearsalOperations } from "./cleanup-rehearsal-operations.mjs";
import { simulatedInstall } from "./cleanup-rehearsal-inputs.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { writeFile } from "node:fs/promises";
if (!process.connected || process.argv.length !== 3) throw Error("rehearsal_ipc_required");
process.umask(0o077);
const root = resolve(process.argv[2]), context = await syntheticRoot(root);
let sequence = 0;
const pending = new Map();
const request = (method, args) => new Promise((done, reject) => {
  const id = ++sequence;
  const timer = setTimeout(() => { pending.delete(id); reject(Error("rehearsal_prerequisite_deadline")); }, 10000);
  pending.set(id, { done, reject, timer }); process.send({ kind: "prerequisite", id, method, args });
});
process.on("message", message => {
  if (message.kind !== "prerequisite-result") return;
  const entry = pending.get(message.id); if (!entry) return; pending.delete(message.id); clearTimeout(entry.timer);
  if (message.error) entry.reject(Object.assign(Error(message.error), { code: message.error })); else entry.done(message.value);
});
const operations = rehearsalOperations(context, request);
operations.supervisorProgram = fileURLToPath(new URL("./operator-composition-supervisor.mjs", import.meta.url));
operations.onSupervisorSpawn = child => child.on("message", async message => {
  if (message.kind !== "prerequisite") return;
  try { const value = await request(message.method, message.args); if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, value }); }
  catch (error) { if (child.connected) child.send({ kind: "prerequisite-result", id: message.id, error: error.code ?? "v2_rehearsal_failed" }); }
});
operations.installCandidate = async (selected, index) => {
  await syntheticRoot(selected, context);
  await simulatedInstall({ root, context, operations: { ...operations, unixNow: Date.now }, put: (path, bytes) => writeFile(path, bytes, { mode: 0o600 }) }, index);
  const owner = (await proof(root, "server-owner.json")).value;
  const response = await fetch(`${owner.origin}/install/review`, { method: "POST", signal: AbortSignal.timeout(10000),
    headers: { Origin: owner.origin, "Content-Type": "application/json" }, body: JSON.stringify({ index }) });
  const result = await response.json(); if (!response.ok) throw Object.assign(Error(result.error), { code: result.error }); return result;
};
const watchdog = setTimeout(() => process.kill(process.pid, "SIGTERM"), 120000); watchdog.unref();
await runOperator(root, { schema: "str005-v2-operator-bootstrap-v1", maybeAuthorityDirectory: null }, operations);

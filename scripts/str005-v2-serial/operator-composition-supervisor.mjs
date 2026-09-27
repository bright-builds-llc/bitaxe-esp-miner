// Test-only private entry: actual parent admission and main/managed supervisor.
import { resolve } from "node:path";
import { once } from "node:events";
import { main } from "./main.mjs";
import { admitOperatorParent } from "./operator-parent.mjs";
import { syntheticRoot } from "./cleanup-rehearsal-guard.mjs";
import { rehearsalOperations } from "./cleanup-rehearsal-operations.mjs";
if (!process.connected || process.argv.length !== 3) throw Error("rehearsal_ipc_required");
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
const [input] = await once(process, "message");
await admitOperatorParent(root, input.binding);
if (input.maybeAuthorityDirectory !== null) throw Error("rehearsal_authority_forbidden");
const watchdog = setTimeout(() => process.kill(process.pid, "SIGTERM"), 120000); watchdog.unref();
await main(["serve", "--private-root", root], rehearsalOperations(context, request));
clearTimeout(watchdog); process.disconnect();

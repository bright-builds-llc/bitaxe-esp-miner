import { monotonicHostMs } from "../str005-noise-serial/operator.mjs";
import { resolve } from "node:path";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireGone, processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { browserWitness, requireHostStopped } from "./cleanup.mjs";
import { contextHash } from "./context.mjs";
import { fail } from "./failure.mjs";
import { check, schema } from "./values.mjs";
const bound = async (promise, ms) => { let timer; try { return await Promise.race([promise, new Promise(done => { timer = setTimeout(() => done(null), ms); })]); } finally { clearTimeout(timer); } };
/** Child exit observation is attached by its actual parent; evidence failure never skips stop. */
export function createCleanupSession(root, context, { child, owner, operations = {} }) {
  let maybeFinish, stopRequestedAtMs = null;
  const hash = contextHash(context);
  const exited = new Promise(done => child.once("close", (code, signal) => done({ code, signal, exitedAtMs: monotonicHostMs() })));
  async function finish(witness) {
    let maybeFailure = null, valid = false;
    try { await browserWitness(root, context, witness); await writeNew(resolve(root, "browser-closure.json"), witness); valid = true; }
    catch (error) { maybeFailure = error; }
    stopRequestedAtMs = monotonicHostMs();
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
    let result = await bound(exited, 5000);
    if (!result) {
      maybeFailure ??= Object.assign(Error("bootstrap_supervisor_timeout"), { code: "bootstrap_supervisor_timeout" });
      const rows = await (operations.processSnapshot ?? processSnapshot)();
      if (rows.some(row => sameProcess(row, owner))) process.kill(-owner.pgid, "SIGKILL");
      result = await bound(exited, 5000);
    }
    check(result, "bootstrap_supervisor_live"); await requireGone([owner], operations);
    const observation = { schema: schema("supervisor-exit"), source: "parent-observed", contextSha256: hash, owner, code: result.code, signal: result.signal,
      observedAtUnixMs: Date.now(), clock: "node-hrtime-ms-v1", stopRequestedAtMs, exitedAtMs: result.exitedAtMs };
    try { await writeNew(resolve(root, "parent-cleanup-supervisor-observation.json"), observation); } catch (error) { maybeFailure ??= error; }
    try { await requireHostStopped(root, context, operations); } catch (error) { maybeFailure ??= error; }
    const complete = valid && !maybeFailure && result.code === 0 && result.signal === null && result.exitedAtMs >= stopRequestedAtMs && result.exitedAtMs - stopRequestedAtMs <= 5000;
    try { await writeNew(resolve(root, "cleanup-facts.json"), { schema: schema("cleanup-facts"), contextSha256: hash, browserWitness: valid ? witness : null,
      supervisorObservationSha256: (await proof(root, "parent-cleanup-supervisor-observation.json")).sha256, complete }); } catch (error) { maybeFailure ??= error; }
    if (maybeFailure) { await fail(root, context, "cleanup", "cleanup", maybeFailure); throw maybeFailure; }
    return { cleanup_recorded: complete };
  }
  return { finish(witness) { maybeFinish ??= finish(witness); return maybeFinish; } };
}

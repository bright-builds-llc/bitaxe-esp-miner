import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fileDigest } from "../fixed-usb-qualification/contract.mjs";
import { prepareCleanup } from "./cleanup.mjs";
import { readJournal } from "./journal.mjs";
import { observeOwnedExit, monotonicHostMs } from "../str005-noise-serial/operator.mjs";
import { processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { check, object, sha256, uint } from "./values.mjs";
import { PARENT_CLEANUP_CODES, PARENT_CLEANUP_STAGES, parentCleanupCode } from "./cleanup-session-values.mjs";
export { PARENT_CLEANUP_CODES, PARENT_CLEANUP_STAGES };

/** One parent owns the private cleanup capability until it records completion or
 * a durable typed failure. The browser witness must come from actual parent
 * observation; the helper never fabricates browser closure or device state.
 */
export function createCleanupSession(root, context, { child, owner, operations = {} }) {
  check(context.schema === "str005-v2-serial-context-v3", "v2_parent_cleanup_state");
  const contextSha256 = sha256(JSON.stringify(context));
  check(child.pid === owner.pid && owner.pgid === owner.pid && child.exitCode === null && child.signalCode === null, "v2_parent_supervisor_identity");
  const exitObserver = observeOwnedExit(child, contextSha256, owner, "supervisor");
  let closed = false, maybeRawExit, maybeStoppedAt = null, maybePrepared, maybePrepare, maybeFinish, maybeStop, maybeFirstFailure, maybeFailureWriteError;
  child.once("close", (code, signal) => {
    closed = true;
    maybeRawExit = { schema: "str005-v2-parent-process-observation-v1", source: "parent-observed", contextSha256,
      owner, code, signal, clock: "node-hrtime-ms-v1", stopRequestedAtMs: maybeStoppedAt,
      exitedAtMs: monotonicHostMs(), observedAtUnixMs: Date.now() };
  });
  async function failure(stage, error) {
    if (maybeFirstFailure) return maybeFirstFailure;
    const code = parentCleanupCode(error);
    const value = { schema: "str005-v2-parent-cleanup-failure-v1", source: "parent-observed", contextSha256,
      stage, code, observedAtUnixMs: Date.now() };
    maybeFirstFailure = Object.assign(new Error(code), { code });
    try { await writeNew(resolve(root, "parent-cleanup-failure.json"), value); }
    catch (writeError) {
      try {
        if (writeError.code !== "EEXIST") throw writeError;
        const previous = (await proof(root, "parent-cleanup-failure.json")).value;
        object(previous, ["schema", "source", "contextSha256", "stage", "code", "observedAtUnixMs"]);
        check(previous.schema === value.schema && previous.source === value.source && previous.contextSha256 === contextSha256 &&
          PARENT_CLEANUP_STAGES.includes(previous.stage) && PARENT_CLEANUP_CODES.includes(previous.code), "v2_parent_cleanup_failed");
        uint(previous.observedAtUnixMs);
        maybeFirstFailure = Object.assign(new Error(previous.code), { code: previous.code });
      } catch {
        // A failed evidence write must not skip real owned-process cleanup.
        maybeFailureWriteError = Object.assign(new Error("v2_parent_cleanup_failed"), { code: "v2_parent_cleanup_failed" });
      }
    }
    return maybeFirstFailure;
  }
  async function browser(witness) {
    object(witness, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
    const last = (await readJournal(root, context)).at(-1);
    uint(witness.observedAtUnixMs);
    check(witness.schema === "noise-serial-browser-closure-v2" && witness.source === "parent-observed" &&
      witness.contextSha256 === contextSha256 && witness.closed === true && witness.observedAtUnixMs <= Date.now() &&
      last && witness.lastSequence === last.sequence && witness.lastStateSha256 === sha256(JSON.stringify(last)) &&
      last.state.status === "closed" && last.state.connected === false && last.state.serialOwnershipReleased === true,
    "v2_parent_browser_unproved");
    await writeNew(resolve(root, "parent-cleanup-browser.json"), witness);
  }
  async function reapAfterFailure() {
    const owners = [owner];
    try {
      const fixture = (await proof(root, "fixture-owner.json")).value;
      check(fixture.contextSha256 === contextSha256 && fixture.binarySha256 === context.fixture_sha256 &&
        fixture.owner.pid === fixture.owner.pgid, "v2_parent_supervisor_identity");
      owners.push(fixture.owner);
    } catch (error) { if (error.code !== "ENOENT") throw error; }
    const snapshot = operations.processSnapshot ?? processSnapshot;
    for (const value of owners) {
      const rows = await snapshot();
      if (rows.some(row => sameProcess(row, value))) {
        try { process.kill(-value.pgid, "SIGKILL"); }
        catch (error) { if (error.code !== "ESRCH") throw error; }
      }
    }
    const deadline = monotonicHostMs() + 5000;
    while (true) {
      const rows = await snapshot();
      if (!rows.some(row => owners.some(value => sameProcess(row, value) || row.ppid === value.pid || row.pgid === value.pgid))) return;
      check(monotonicHostMs() < deadline, "v2_parent_owner_remains");
      await new Promise(done => setTimeout(done, 25));
    }
  }
  function stop() {
    if (maybeStop) return maybeStop;
    maybeStop = (async () => {
      let deadline;
      try {
        const current = await (operations.processSnapshot ?? processSnapshot)();
        check(!closed && current.some(row => sameProcess(row, owner)), "v2_parent_supervisor_identity");
        maybeStoppedAt = monotonicHostMs(); exitObserver.markStopRequested();
        check(child.kill("SIGTERM"), "v2_parent_supervisor_stop_failed");
        const receipt = await Promise.race([exitObserver.receipt().catch(() => {
          check(false, "v2_parent_supervisor_exit_failed");
        }), new Promise((_done, reject) => { deadline = setTimeout(() => reject(Object.assign(Error("v2_parent_supervisor_timeout"),
          { code: "v2_parent_supervisor_timeout" })), 5000); })]);
        await writeNew(resolve(root, "parent-cleanup-supervisor.json"), receipt);
        check(receipt.code === 0, "v2_parent_supervisor_exit_failed");
        return receipt;
      } catch (error) {
        // The successful close deadline is already lost. Preserve that failure
        // before forced host cleanup; escalation can never create a passing exit.
        try { await failure("supervisor_stop", error); } finally { await reapAfterFailure(); }
        throw error;
      } finally {
        clearTimeout(deadline);
        if (maybeRawExit) await writeNew(resolve(root, "parent-cleanup-supervisor-observation.json"), maybeRawExit);
      }
    })();
    return maybeStop;
  }
  function prepare() {
    if (maybePrepare) return maybePrepare;
    maybePrepare = (async () => {
      try {
        check(!maybeFinish && !closed && !maybeFirstFailure, "v2_parent_cleanup_state");
        check(await fileDigest(fileURLToPath(import.meta.url)) === context.evaluator.find(item => item.path === "scripts/str005-v2-serial/cleanup-session.mjs")?.sha256,
          "v2_cleanup_helper_changed");
        await writeNew(resolve(root, "parent-cleanup.claim.json"), { schema: "str005-v2-parent-cleanup-claim-v1", contextSha256, owner });
        maybePrepared = await prepareCleanup(root, context, operations);
        return { cleanup_prepared: true };
      } catch (error) { await failure("prepare", error); throw maybeFailureWriteError ?? maybeFirstFailure; }
    })();
    return maybePrepare;
  }
  function finish(witness) {
    if (maybeFinish) return maybeFinish;
    maybeFinish = (async () => {
      let maybeSupervisor;
      if (maybePrepare) { try { await maybePrepare; } catch { /* First cause was persisted by prepare. */ } }
      try { check(maybePrepared && !maybeFirstFailure, "v2_parent_cleanup_state"); await browser(witness); }
      catch (error) { await failure("browser_witness", error); }
      // Even an invalid browser witness cannot strand the owned host children.
      try { maybeSupervisor = await stop(); } catch (error) { await failure("supervisor_stop", error); }
      if (maybeFirstFailure) throw maybeFailureWriteError ?? maybeFirstFailure;
      try {
        const result = await maybePrepared.record({ browser: witness, supervisor: maybeSupervisor });
        check(result.cleanup_recorded && result.device_resources_released && result.device_baseline_confirmed,
          "v2_parent_cleanup_failed");
        return result;
      } catch (error) { await failure("record", error); throw maybeFailureWriteError ?? maybeFirstFailure; }
    })();
    return maybeFinish;
  }
  return Object.freeze({ prepare, finish });
}

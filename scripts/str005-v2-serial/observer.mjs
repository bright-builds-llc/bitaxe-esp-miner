import { resolve } from "node:path";
import { createCadenceObserver } from "../fixed-usb-qualification/cadence-observer.mjs";
import { nonce } from "../fixed-usb-qualification/contract.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { bytes, check, ipv4, object, port, sha256, uint } from "./values.mjs";
import { baseline } from "./journal.mjs";

/** Reuse the qualified passive PlainWebSocket owner; no device control request is introduced. */
export function createObserverRoutes(root, context, { now, ready, failed, fail = () => {}, journal, operations = {} }) {
  const observer = (operations.createObserver ?? createCadenceObserver)(root, context, operations);
  let maybeChallenge, used = false, maybeFinish, maybeStart, maybeFaultAtMs = null, maybeBinding;
  let monitoring = false, finishing = false, maybePoll;
  function observeFailure() {
    if (!monitoring || finishing) return;
    const status = observer.status();
    if (!status.connected || !status.alive || status.failed) fail("v2_observer_disconnected");
  }
  function alive() {
    observeFailure();
    const status = observer.status();
    check(used && status.connected && status.alive && !status.failed, "v2_observer_not_live");
  }
  return {
    alive, observeFailure,
    binding: () => maybeBinding,
    markFault(confirmedAtHostMs) {
      alive(); uint(confirmedAtHostMs);
      check(maybeFaultAtMs === null, "v2_observer_fault_duplicate");
      check(confirmedAtHostMs <= now(), "v2_observer_fault_clock");
      maybeFaultAtMs = confirmedAtHostMs;
    },
    async handle(path, input) {
      if (!["/observer/context", "/observer/start", "/observer/finish"].includes(path)) return undefined;
      check(context.scope === "share", "v2_observer_scope");
      if (path === "/observer/finish") {
        object(input, []); check(maybeFaultAtMs !== null && now() - maybeFaultAtMs >= 5000, "v2_observer_fault_tail");
        alive(); return this.finish();
      }
      ready(); check(!finishing, "v2_observer_stopping"); baseline(journal.lastState()?.state);
      if (path === "/observer/context") {
        object(input, []); check(!used && maybeChallenge === undefined, "v2_observer_consumed");
        maybeChallenge = { nonce: nonce(), atHostMs: now() }; return { nonce: maybeChallenge.nonce };
      }
      object(input, ["nonce", "endpoint", "state"]);
      const challenge = maybeChallenge; maybeChallenge = undefined;
      check(challenge && input.nonce === challenge.nonce && now() - challenge.atHostMs <= 5000 && !used, "v2_observer_freshness");
      const e = input.endpoint;
      object(e, ["schema", "ipv4", "httpPort", "observedAtUs", "bootOrdinal", "generation", "controlSessionBindingSha256"]);
      check(e.schema === "worker-telemetry-endpoint-v1", "v2_observer_endpoint");
      ipv4(e.ipv4); port(e.httpPort); bytes(e.controlSessionBindingSha256, 32);
      for (const key of ["observedAtUs", "bootOrdinal", "generation"]) uint(e[key]);
      check(e.bootOrdinal > 0, "v2_observer_boot");
      baseline(input.state); await journal.state("candidate", input.state, now()); ready();
      used = true; maybeBinding = { bootOrdinal: e.bootOrdinal, workerGeneration: e.generation,
        controlSessionBindingSha256: e.controlSessionBindingSha256 };
      // Join the complete pending start before finishing: the reused owner may
      // await binary/file checks before spawning its child.
      maybeStart = (async () => {
        await writeNew(resolve(root, "observer-start.claim.json"), { schema: "str005-v2-observer-claim-v1",
          contextSha256: sha256(JSON.stringify(context)), atHostMs: now(), sourceObservedAtDeviceUs: e.observedAtUs,
          bootOrdinal: e.bootOrdinal, workerGeneration: e.generation, stateSequence: journal.lastState().sequence,
          binarySha256: context.cadence_observer.sha256, lifetimeMs: 360000 });
        ready(); check(!finishing, "v2_observer_stopping");
        const result = await observer.start({ ipv4: e.ipv4, httpPort: e.httpPort });
        if (!finishing) {
          monitoring = true;
          maybePoll = (operations.setInterval ?? setInterval)(observeFailure, 100);
          maybePoll?.unref?.();
        }
        return result;
      })().catch((error) => { fail("v2_observer_start_failed"); throw error; });
      const result = await maybeStart;
      ready(); check(!finishing, "v2_observer_stopping"); alive(); return result;
    },
    finish() {
      if (maybeFinish) return maybeFinish;
      observeFailure(); finishing = true;
      if (maybePoll !== undefined) (operations.clearInterval ?? clearInterval)(maybePoll);
      maybeFinish = (async () => {
        const requestedAtHostMs = now();
        let maybeStartFailure;
        if (maybeStart) {
          try { await maybeStart; } catch (error) { maybeStartFailure = error; }
        }
        // A rejected start may still own a child; cleanup is always attempted.
        const result = await observer.finish(), completedAtHostMs = now();
        if (!used) return { observer_started: false };
        await writeNew(resolve(root, "observer-stop.json"), { schema: "str005-v2-observer-stop-v1",
          contextSha256: sha256(JSON.stringify(context)), requestedAtHostMs, completedAtHostMs,
          tailMs: maybeFaultAtMs === null ? null : requestedAtHostMs - maybeFaultAtMs,
          requestedCleanupMs: completedAtHostMs - requestedAtHostMs });
        if (maybeStartFailure) throw maybeStartFailure;
        check(result.cleanupComplete && result.closed && completedAtHostMs - requestedAtHostMs <= 5000, "v2_observer_cleanup");
        if (!failed()) check(result.reason === "requested" && result.exitCode === 0, "v2_observer_outcome");
        return { observer_closed: true, tail_ms: maybeFaultAtMs === null ? null : requestedAtHostMs - maybeFaultAtMs };
      })();
      return maybeFinish;
    },
  };
}

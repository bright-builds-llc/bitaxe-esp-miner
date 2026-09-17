import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { nonce } from "../fixed-usb-qualification/contract.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { validateCycle } from "../fixed-usb-qualification/judge.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { parseStatus } from "./device.mjs";
import { compareConnection } from "./fixture.mjs";
import { checkedState, healthy, readJournal, restoredBaseline } from "./journal.mjs";
import { binding, createReceiptWriter, selectedFacts } from "./execution-receipts.mjs";
import { faultFacts } from "./execution-restoration.mjs";
import { inspectLiveSelection } from "./execution-selection.mjs";
import { decodeJob } from "./job-proof.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** Only the independently armed heartbeat fault can explain this specific liveness loss. */
export function expectedHeartbeatLoss(observed, generation, armed) {
  return armed && observed?.heartbeatSuppressed === true && observed.running === false && !observed.ownerResourceFailure &&
    observed.qualification?.generation === generation && observed.serialFailureCategory === "liveness_lost" &&
    (observed.failure === undefined || observed.failure === "window_control_failed");
}

/** Effect permits and source observations are separate; no route returns a hardware verdict. */
export function createExecutionRoutes(runtime) {
  const { root, context, contextSha256, now, ready, verify, journal, failures, observer } = runtime;
  const receipt = createReceiptWriter(root, context);
  let startClaimed = false, maybeConnection = null, maybeJob = null, maybeSelection = null, maybeFaultClaim = null, maybeFaultConfirmed = null;
  let faultUsed = false, protocolCompleted = false;
  function live() { ready(); check(!failures.failed() && runtime.phase() === "candidate", "v2_execution_admission"); }
  function owner() { const value = runtime.fixture(); check(value, "v2_fixture_missing"); return value; }
  function actualStatus(input) {
    const status = parseStatus(input), last = journal.lastDevice();
    check(status.scope === context.scope && status.record?.attemptId === context.attemptId && last && equal(last.record, status.record), "v2_execution_unrecorded_device");
    return { status, last };
  }
  async function requireCycles() {
    const last = journal.lastState(); check(last?.phase === "candidate", "v2_execution_candidate"); healthy(last.state);
    let previous;
    for (let index = 1; index <= 4; index++) {
      const value = (await proof(root, `cycle-${index}.json`)).value;
      check(value.schema === "str005-v2-cycle-v1" && value.contextSha256 === contextSha256 && value.afterSequence <= last.sequence &&
        value.report.baseline_id === last.state.preservation.baseline_id, "v2_execution_cycles");
      previous = validateCycle(value.report, context, previous);
    }
    const accounting = (await proof(root, "accounting-before.json")).value;
    check(accounting.contextSha256 === contextSha256 && accounting.stage === "before" && accounting.observedSequence <= last.sequence &&
      accounting.state.preservation.baseline_id === last.state.preservation.baseline_id, "v2_execution_accounting");
    requireIdleLedger(accounting.ledger, 18, 1560000); requireExhaustedOriginal(accounting.original_budget);
    return last;
  }
  async function claimStart(input) {
    object(input, ["status"]); live(); check(context.scope === "channel" && !startClaimed, "v2_channel_start_consumed");
    const status = parseStatus(input.status);
    check(status.scope === "channel" && status.state === "idle" && status.observation.wifiConnected && status.observation.stationIpv4 !== null, "v2_channel_idle_required");
    const last = await requireCycles(); await verify(); live(); const fixture = owner();
    fixture.alive(); fixture.validateStation(status.observation.stationIpv4); fixture.requireStartWindow();
    const fixtureReady = await proof(root, "fixture-ready.json"); live(); fixture.requireStartWindow();
    startClaimed = true;
    const value = { schema: "str005-v2-start-claim-v1", contextSha256, atHostMs: now(), observedStateSequence: last.sequence,
      bootOrdinal: status.observation.bootOrdinal, workerGeneration: status.observation.workerGeneration,
      serialTransportEpoch: status.observation.serialTransportEpoch, networkObservedAtDeviceUs: status.observation.observedAtUs,
      fixtureReadySha256: fixtureReady.sha256, clientSha256: context.client_sha256, attemptId: context.attemptId };
    await writeNew(resolve(root, "start.claim.json"), value); live(); fixture.alive(); fixture.requireStartWindow();
    return { schema: "worker-stratum-v2-channel-start-v1", attemptId: context.attemptId,
      expectedBootOrdinal: value.bootOrdinal, networkObservedAtUs: value.networkObservedAtDeviceUs, stratum: fixture.stratum };
  }
  async function connect(input) {
    object(input, ["status"]); check(maybeConnection === null, "v2_connection_consumed");
    const { status, last } = actualStatus(input.status), fixture = owner();
    const comparisonStartedAtMs = now(), observed = await fixture.connection();
    const compared = compareConnection(status, observed, fixture.ready), readiness = await proof(root, "fixture-ready.json");
    // fd3 is emitted only after successful bounded single-peer selection. Final inventory is checked independently.
    const facts = { attemptId: context.attemptId, instanceId: compared.instanceId, connectionId: compared.connectionId,
      bootOrdinal: status.record.bootOrdinal, ...binding(status.record), expectedPeerMatch: true, tupleMatch: true,
      expectedPeerCount: 1, unexpectedPeerCount: 0, candidateOverflow: false, readinessSha256: readiness.sha256,
      deviceObservationSequence: last.sequence, comparisonStartedAtMs, comparisonCompletedAtMs: now() };
    maybeConnection = await receipt("connection.json", "supervisor-ms", facts.comparisonCompletedAtMs, facts);
    return { connection_recorded: true };
  }
  async function job() {
    if (maybeJob) return maybeJob;
    check(maybeConnection, "v2_connection_missing");
    const source = await proof(root, "fixture-run/job.json"), decoded = decodeJob(source.value);
    check(decoded.connectionId === maybeConnection.value.facts.connectionId, "v2_job_connection");
    const record = journal.lastDevice()?.record, event = record?.events.find(row => row.kind === "work_ready");
    check(record?.jobCommitment === decoded.jobCommitment && event?.atDeviceUs !== null && event?.atDeviceUs !== undefined, "v2_native_job_unobserved");
    maybeJob = await receipt("job-receipt.json", "device-us", event.atDeviceUs, source.value); return maybeJob;
  }
  async function select(input) {
    object(input, ["status"]); live(); check(context.scope === "share", "v2_share_selection_scope"); observer.alive();
    const { status, last } = actualStatus(input.status);
    if (maybeSelection) return { eligible: true, submissionSequence: maybeSelection.fact.submissionSequence, selectedDeviceAckSha256: maybeSelection.ack.sha256 };
    check(status.state !== "terminal", "v2_selection_after_terminal");
    const selected = await inspectLiveSelection(root, context, status.record); live(); observer.alive();
    if (!selected) return { eligible: false };
    await job(); live(); observer.alive();
    const fact = selected.fact;
    let facts = selectedFacts(status.record, selected.job, fact, selected.share);
    await receipt("selected-dispatch.json", "device-us", fact.dispatchedAtDeviceUs, facts.dispatch);
    const nonce = await receipt("selected-nonce.json", "device-us", fact.nonceAtDeviceUs, facts.nonce);
    facts = selectedFacts(status.record, selected.job, fact, selected.share, nonce.sha256);
    const submission = await receipt("selected-submission.json", "device-us", fact.writeCompletedAtDeviceUs, facts.submission);
    facts = selectedFacts(status.record, selected.job, fact, selected.share, nonce.sha256, submission.sha256);
    const ack = await receipt("selected-device-ack.json", "device-us", fact.ackAtDeviceUs, facts.deviceAck);
    await receipt("selected-fixture-ack.json", "fixture-us", selected.share.writeCompletedAtFixtureUs, facts.fixtureAck);
    live(); observer.alive(); maybeSelection = { ...selected, ack, deviceSequence: last.sequence };
    return { eligible: true, submissionSequence: fact.submissionSequence, selectedDeviceAckSha256: ack.sha256 };
  }
  async function claimFault(input) {
    object(input, ["selectedDeviceAckSha256"]); live(); observer.alive();
    check(context.scope === "share" && maybeSelection && !faultUsed && input.selectedDeviceAckSha256 === maybeSelection.ack.sha256, "v2_fault_selection");
    const last = journal.lastState(); check(last?.state.running && !last.state.heartbeatSuppressed && last.state.qualification?.generation === maybeSelection.record.workerGeneration, "v2_fault_running");
    faultUsed = true;
    const value = { schema: "str005-v2-fault-claim-v1", contextSha256, nonce: nonce(), requestedAtHostMs: now(),
      selectedDeviceAckSha256: maybeSelection.ack.sha256, selectionDeviceSequence: maybeSelection.deviceSequence, stateSequence: last.sequence };
    await writeNew(resolve(root, "fault.claim.json"), value); live(); observer.alive();
    maybeFaultClaim = await proof(root, "fault.claim.json"); return { nonce: value.nonce };
  }
  async function confirmFault(input) {
    object(input, ["nonce", "headroom", "state"]); live(); observer.alive();
    check(maybeFaultClaim && !maybeFaultConfirmed && input.nonce === maybeFaultClaim.value.nonce && now() >= maybeFaultClaim.value.requestedAtHostMs && now() - maybeFaultClaim.value.requestedAtHostMs <= 45000, "v2_fault_confirmation");
    const h = input.headroom;
    object(h, ["schema", "workerGeneration", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs"]);
    for (const key of ["workerGeneration", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs"]) uint(h[key]);
    check(h.schema === "worker-v2-fault-headroom-v1" && h.workerGeneration === maybeSelection.record.workerGeneration && h.leaseRemainingMs >= 5000 && h.leaseRemainingMs <= 60000 && h.workGateRemainingMs >= 5000 && h.workGateRemainingMs <= 164450 &&
      h.headroomObservedAtDeviceUs >= Math.floor(maybeSelection.fact.ackAtDeviceUs / 1000) * 1000, "v2_fault_headroom");
    checkedState(input.state, context, "candidate");
    const q = input.state.qualification;
    check(input.state.running && input.state.heartbeatSuppressed && !input.state.failure && q?.generation === h.workerGeneration && q.revocation_reason === "none" && !q.safe_stop_complete && q.work_gate_remaining_ms === h.workGateRemainingMs, "v2_fault_state");
    const saved = await journal.state("candidate", input.state, now()); live(); observer.alive();
    const value = { schema: "str005-v2-fault-confirmed-v1", contextSha256, claimSha256: maybeFaultClaim.sha256,
      confirmedAtHostMs: now(), headroom: h, observedStateSequence: saved.sequence };
    await writeNew(resolve(root, "fault-confirmed.json"), value); live(); observer.alive();
    observer.markFault(value.confirmedAtHostMs); maybeFaultConfirmed = value;
    return { fault_confirmed: true };
  }
  async function complete(input) {
    object(input, []); check(!protocolCompleted, "v2_protocol_completion_consumed");
    if (context.scope === "share") check(maybeFaultConfirmed, "v2_fault_confirmation_missing");
    const last = journal.lastDevice(); check(last && maybeConnection, "v2_protocol_device_missing");
    if (context.scope === "channel") check(last.record.state === "terminal" && last.record.outcome === "accepted" &&
      last.record.resources.socketClosed && last.record.resources.workerQuiescent && !last.record.resources.fenceRetained, "v2_channel_completion");
    await owner().finish(); await job();
    const terminal = await proof(root, "fixture-run/fixture-terminal.json"), shares = await proof(root, "fixture-run/shares.json");
    check(terminal.value.outcome === "accepted" && terminal.value.firstFailure === null && terminal.value.peerClosed && terminal.value.socketClosed && terminal.value.listenerClosed, "v2_fixture_incomplete");
    await writeNew(resolve(root, "protocol-complete.json"), { schema: "str005-v2-protocol-complete-v1", contextSha256, atHostMs: now(),
      observedStateSequence: journal.lastState().sequence, observedDeviceSequence: last.sequence, fixtureTerminalSha256: terminal.sha256, fixtureSharesSha256: shares.sha256 });
    protocolCompleted = true; return { protocol_recorded: true };
  }
  async function restore(input) {
    object(input, ["status", "state"]); const { status, last } = actualStatus(input.status);
    check(status.record.state === "terminal" && status.record.resources.socketClosed && status.record.resources.workerQuiescent && !status.record.resources.fenceRetained, "v2_restoration_resource");
    checkedState(input.state, context, "candidate"); restoredBaseline(input.state, context);
    check(status.observation.serialTransportEpoch !== status.record.serialTransportEpoch, "v2_restoration_session_not_fresh");
    const rows = await readJournal(root, context), before = (await proof(root, "accounting-before.json")).value;
    const closed = rows.filter(row => row.sequence > before.observedSequence && row.state.status === "closed" && !row.state.connected && row.state.serialOwnershipReleased).at(-1);
    check(closed && input.state.preservation.baseline_id === before.state.preservation.baseline_id, "v2_restoration_continuity");
    if (maybeFaultConfirmed) check(now() - maybeFaultConfirmed.confirmedAtHostMs >= 145000, "v2_restoration_wait");
    const maybeFault = context.scope === "share" && maybeFaultConfirmed
      ? faultFacts(status.record, input.state, maybeFaultClaim.value, maybeFaultConfirmed, (await proof(root, "observer-stop.json")).value)
      : null;
    const saved = await journal.state("candidate", input.state, now());
    await writeNew(resolve(root, "restoration.json"), { schema: "str005-v2-restoration-v1", contextSha256, closedSequence: closed.sequence,
      observedSequence: saved.sequence, deviceSequence: last.sequence, state: input.state, recordSha256: sha256(JSON.stringify(status.record)) });
    if (maybeFault) await receipt("fault.json", "device-us", maybeFault.shutdownStartedAtDeviceUs, maybeFault);
    return { restoration_recorded: true };
  }
  return {
    async handle(path, input) {
      if (path === "/restoration/context") { object(input, []); return { attemptId: context.attemptId }; }
      if (path === "/start/claim") return claimStart(input);
      if (path === "/protocol/connection") return connect(input);
      if (path === "/share/select") return select(input);
      if (path === "/fault/claim") return claimFault(input);
      if (path === "/fault/confirm") return confirmFault(input);
      if (path === "/protocol/complete") return complete(input);
      if (path === "/restoration") return restore(input);
      return undefined;
    },
    expectedFault(state) {
      if (!maybeFaultClaim || !maybeSelection) return false;
      const observed = state ?? journal.lastState()?.state;
      return expectedHeartbeatLoss(observed, maybeSelection.record.workerGeneration, true);
    },
  };
}

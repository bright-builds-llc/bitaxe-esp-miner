import assert from "node:assert/strict";
import { createV2Coordinator } from "./client.mjs";
export const ATTEMPT = Buffer.alloc(16, 1).toString("base64url"), NONCE = Buffer.alloc(16, 2).toString("base64url");
export const BINDING = Buffer.alloc(32, 3).toString("base64url");
const IP = "192.168.77.4", KEY = Buffer.alloc(32, 4).toString("base64url");
export function clientFixture(scope = "channel") {
  let time = 0, maybeCoordinator, failed = false, terminal = false, polls = 0, maybeFaultTime = null, maybeThrowAt, maybeRecordFailure = false;
  const calls = [], durable = [], notices = [], sleeps = [], counts = { channelStart: 0, shareStart: 0, connect: 0, cancel: 0, close: 0, proof: 0, suppress: 0 };
  const state = { schema: "worker-serial-acceptance-v1", expectedFirmwareSourceCommit: "a".repeat(40), status: "ready", connected: true,
    running: false, heartbeatSuppressed: false, deviceLeaseInactive: true, deviceBaselineConfirmed: true, serialOwnershipReleased: false, renewalsConfirmed: 0,
    qualification: { generation: 7 }, preservation: { baseline_id: NONCE, settings_match: true, device_identity_match: true, authorization_high_water_match: true, mine_on_boot: false } };
  const publish = () => maybeCoordinator?.observe(structuredClone(state));
  const status = (active = false) => ({ schema: "worker-stratum-v2-status-v1", scope, state: active ? (terminal ? "terminal" : "running") : "idle",
    observation: { bootOrdinal: 1, workerGeneration: 7, serialTransportEpoch: 2, observedAtUs: time * 1000, clockValid: true, stationIpv4: IP, wifiConnected: true, socket: null },
    connection: active ? { socket: { localIpv4: IP, localPort: 40000, remoteIpv4: "192.168.77.5", remotePort: 3333 } } : null,
    record: active ? { schema: "worker-v2-serial-evidence-v1", scope, attemptId: ATTEMPT, state: terminal ? "terminal" : "running", outcome: terminal ? "accepted" : null, firstFailure: null, secondaryFailures: [] } : null });
  const event = name => { calls.push({ name }); if (name === maybeThrowAt) throw Object.assign(new Error(`private ${IP} ${KEY} raw payload`), { category: "io" }); };
  const gate = {
    state: () => structuredClone(state),
    async refresh() { event("refresh"); publish(); return structuredClone(state); },
    async reviewQualificationAttempts() { event("ledger"); return { pending: false }; },
    async reviewBudget() { event("budget"); return { pending: false }; },
    async configure() { event("configure"); state.status = "configured"; publish(); },
    async stratumV2Possession() { event("possession"); counts.proof++; return BINDING; },
    async stratumV2Status(requestedScope, id, binding) {
      event(id === null ? "idle" : "status"); assert.equal(requestedScope, scope); assert.equal(binding, BINDING);
      if (id === null) return status(); assert.equal(id, ATTEMPT); polls++;
      if (scope === "channel" || state.status === "ready" && counts.close > 0) terminal = true;
      return status(true);
    },
    async stratumV2ChannelStart(input, binding) { event("channel_start"); counts.channelStart++; assert.equal(binding, BINDING); assert.equal(input.attemptId, ATTEMPT); const result = status(true); result.connection = null; result.state = result.record.state = "admitted"; return result; },
    async stratumV2ChannelCancel() { event("cancel"); counts.cancel++; terminal = true; return status(true); },
    async probe() { event("probe"); const probe = { paddingBytes: 65000, requestPayloadBytes: 65536, responsePayloadBytes: 65536 }; state.probe = probe; publish(); return probe; },
    async submitCoolingReview() { event("cooling"); return { cooling_review_saved: true }; },
    async submitBudgetReview() { event("reviewed_binding"); return { pending: false }; },
    async stratumV2TelemetryEndpoint(...args) { event("endpoint"); assert.equal(args.length, 0); return { schema: "worker-telemetry-endpoint-v1", ipv4: IP, httpPort: 80, observedAtUs: time * 1000, bootOrdinal: 1, generation: 7, controlSessionBindingSha256: BINDING }; },
    async prepareStartAuthorization() { event("sign"); return { controlSessionBindingSha256: BINDING }; },
    async loadSignedWindow() { event("load"); state.status = "window_loaded"; publish(); return structuredClone(state); },
    async startWindow() { event("share_start"); counts.shareStart++; state.running = true; state.status = "running"; state.deviceLeaseInactive = false; state.deviceBaselineConfirmed = false; state.preservation.authorization_high_water_match = false; publish(); return structuredClone(state); },
    async suppressHeartbeats() { event("suppress"); counts.suppress++; state.heartbeatSuppressed = true; state.authorizationRecovery = { matched: null, generation: 7 }; publish();
      return { schema: "worker-v2-fault-headroom-v1", workerGeneration: 7, headroomObservedAtDeviceUs: 1234000, leaseRemainingMs: 47000, workGateRemainingMs: 123000 }; },
    async stop() { event("stop"); state.running = false; state.status = "baseline_confirmed"; state.deviceLeaseInactive = true; state.deviceBaselineConfirmed = true; publish(); return structuredClone(state); },
    async close() { event("close"); counts.close++; state.status = "closed"; state.connected = false; state.running = false; state.serialOwnershipReleased = true; publish(); },
    async connect() { counts.connect++; throw Error("client_must_not_connect"); },
  };
  const request = async (path, input, method) => {
    calls.push({ path, input: structuredClone(input), method });
    if (path === maybeThrowAt) throw Error(`private ${IP} ${KEY}`);
    if (path === "/supervisor-state") return { scope, phase: "candidate", failed };
    if (path === "/record") { if (maybeRecordFailure) throw Error("record_failed"); durable.push(input.state); return { recorded: true }; }
    if (path === "/accounting-context") return { campaignId: NONCE };
    if (path === "/accounting") { durable.push({ stage: input.stage }); return { accounting_saved: true, stage: input.stage }; }
    if (path === "/candidate-context") return { candidate: true };
    if (path === "/probe/claim") return { probe_nonce: NONCE };
    if (path === "/probe/complete") return { probe_recorded: true };
    if (path === "/cycle") return { cycle_verified: true, index: input.index };
    if (path === "/fixture/start") return { fixture_ready: true, attemptId: ATTEMPT };
    if (path === "/start/claim") return { attemptId: ATTEMPT, stratum: { endpoint: `stratum+tcp://${IP}:3333/`, authorityPublicKey: KEY, userIdentity: "private-fixture-user" } };
    if (path === "/device/record") { durable.push({ record: input.status.record }); return { recorded: true }; }
    if (path === "/protocol/connection" || path === "/protocol/complete") return { recorded: true };
    if (path === "/observer/context") return { nonce: NONCE };
    if (path === "/observer/start") return { observer_connected: true };
    if (path === "/start/network") return { network_reviewed: true };
    if (path === "/share/start-observed") return { start_observed: true };
    if (path === "/share/select") return polls >= 2 ? { eligible: true, submissionSequence: 1, selectedDeviceAckSha256: "f".repeat(64) } : { eligible: false };
    if (path === "/fault/claim") return { nonce: NONCE };
    if (path === "/fault/confirm") { maybeFaultTime = time; return { confirmed: true }; }
    if (path === "/observer/finish") { assert.ok(time - maybeFaultTime >= 5000); return { observer_closed: true }; }
    if (path === "/restoration/context") return { attemptId: ATTEMPT };
    if (path === "/restoration") return { restoration_recorded: true };
    if (path === "/client-failure") { failed = true; durable.push(input); return { recorded: true }; }
    throw Error("unexpected_route");
  };
  maybeCoordinator = createV2Coordinator({ gate, request, published: () => structuredClone(state), now: () => time,
    sleep: async ms => { sleeps.push(ms); time += ms; }, notice: text => notices.push(text) });
  return { api: maybeCoordinator.supervisor, calls, counts, durable, notices, sleeps, state, gate, request,
    advance(ms) { time += ms; }, at: () => time,
    throwAt(name) { maybeThrowAt = name; }, recordFailure() { maybeRecordFailure = true; },
    nativeReconnect() { state.status = "ready"; state.connected = true; state.serialOwnershipReleased = false; state.running = false;
      state.heartbeatSuppressed = false; state.deviceBaselineConfirmed = true; state.deviceLeaseInactive = true;
      if (scope === "share") state.authorizationRecovery = { matched: true, generation: 7 }; publish(); },
  };
}

import { channelFixture } from '../str005-v2-serial/protocol-judge.test-helper.mjs';
export function fixture() {
  const initial = channelFixture().deviceRecords.at(-1);
  Object.assign(initial, { scope: 'share', state: 'running', observedAtUs: 10000000, outcome: null, terminalAtDeviceUs: null,
    authorityDeadlineDeviceUs: 180001000, observationDeadlineDeviceUs: null });
  initial.resources = { socketClosed: false, workerQuiescent: false, fenceRetained: true, socketClosedAtUs: null, workerQuiescentAtUs: null };
  initial.events = initial.events.filter(e => !['socket_closed', 'worker_quiescent'].includes(e.kind));
  initial.events.push({ ...initial.events.at(-1), sequence: initial.events.length + 1, kind: 'asic_dispatch', atDeviceUs: 9900000 });
  const record = structuredClone(initial);
  Object.assign(record, { state: 'terminal', outcome: 'rejected', observedAtUs: 20000000, terminalAtDeviceUs: 15000000,
    firstFailure: { stage: 'worker_quiescent', category: 'authority', atDeviceUs: 12802000 } });
  for (const [kind, time] of [['revoked', 12801000], ['shutdown', 12802000], ['socket_closed', 12900000], ['worker_quiescent', 14000000], ['cooled', 14500000]])
    record.events.push({ ...record.events.at(-1), sequence: record.events.length + 1, kind, atDeviceUs: time });
  record.resources = { socketClosed: true, workerQuiescent: true, fenceRetained: false, socketClosedAtUs: 12900000, workerQuiescentAtUs: 14000000 };
  const q = { generation: 2, revocation_reason: 'heartbeat_timeout', last_valid_heartbeat_ms: 10000, gate_closed_ms: 12800, shutdown_started_ms: 12801 };
  const run = { firstFailure: null, observedStart: true, startInvokedAt: 0, startRepliedAt: 10000, suppressionRequestedAt: 10500,
    suppressionConfirmedAt: 10600, stopRequestedAt: 18600,
    dispatchStatus: { schema: 'str005-recovery-status-v1', scope: 'share', state: 'running', record: initial,
      observation: { bootOrdinal: 1, workerGeneration: 2, serialTransportEpoch: 3, observedAtUs: 10000000,
        clockValid: true } },
    headroom: { schema: 'worker-v2-fault-headroom-v1', workerGeneration: 2, headroomObservedAtDeviceUs: 10000000, leaseRemainingMs: 50000, workGateRemainingMs: 100000 },
    suppressedState: { running: true, heartbeatSuppressed: true, renewalsConfirmed: 0,
      qualification: { generation: 2, work_dispatched: 1 }, authorizationRecovery: { generation: 2, matched: null } } };
  return { run, record, q };
}

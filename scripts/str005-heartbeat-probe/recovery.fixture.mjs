import { fixture } from './fault.fixture.mjs';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
import { qualification } from '../str005-v2-serial/completed-fixture.mjs';
export function recoveryFixture() {
  const f = fixture(), context = { scope: 'share', attemptId: f.record.attemptId, firmware_commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64) };
  const q = { ...qualification(), ...f.q, active_ms: 2800, generation_elapsed_ms: 9000, budget_reserved_ms: 240000,
    work_dispatched: 1, active_limit_ms: 180000, work_gate_remaining_ms: 0,
    attempt: { schema: 'worker-qualification-observation-v1', ordinal: ledger.next_ordinal, purpose: 'normal', maximum_active_ms: 180000,
      reserved_ms: 180000, complete: true, active_ms: 2800 },
    owner_resources: { schema: 'worker-owner-resources-v1', generation: 2, phase: 'shutdown_complete', observed_at_ms: '20000', heap_free_bytes: 10000, heap_largest_bytes: 5000, stack_free_bytes: 6000 } };
  const checkpoint = { schema: 'worker-authorization-recovery-v1', checkpointId: Buffer.alloc(16, 4).toString('base64url'), generation: 2, matched: true };
  const restored = { ...state(context), qualification: q, authorizationRecovery: checkpoint };
  const closed = { ...state(context, 'candidate', true), qualification: q, authorizationRecovery: checkpoint };
  f.run.suppressedState = { ...state(context), status: 'running', running: true, heartbeatSuppressed: true,
    qualification: { ...q, safe_stop_complete: false, revocation_reason: 'none', gate_closed_ms: null, shutdown_started_ms: null,
      safe_stop_stage: 'not_started' }, authorizationRecovery: { ...checkpoint, matched: null } };
  const recovery = { state: restored, closed, ledger: { ...ledger, next_ordinal: ledger.next_ordinal + 1,
    last_completed_ordinal: ledger.next_ordinal, total_charged_ms: ledger.total_charged_ms + 180000 }, original_budget: original,
    status: { schema: 'str005-recovery-status-v1', scope: 'share', state: 'terminal', record: f.record,
      observation: { bootOrdinal: 1, workerGeneration: 3, serialTransportEpoch: 4, observedAtUs: 21000000, clockValid: true } },
    diagnostics: { schema: 'str005-recovery-diagnostics-v1', observations: [], omitted_count: 0, authoritative: false }, finished: { failures: [] } };
  return { context, parts: { before: { ledger, original_budget: original, state: state(context), attempt: { id: f.record.attemptId } }, run: f.run, recovery, hostReleased: true } };
}

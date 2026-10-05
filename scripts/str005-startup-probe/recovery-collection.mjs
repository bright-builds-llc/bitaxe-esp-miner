import { discoverCurrentStatus } from './retained-status.mjs';
import { RECOVERY_ERRORS_V2, recoveryErrorRow } from './recovery-error-row.mjs';
const categories = new Set(['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed',
  'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active']);
export function recoveryFailure(phase, error) {
  return { phase, category: categories.has(error?.category) ? error.category : 'operation_failed' };
}
/** One independently bounded collection; begin proves a fresh possession binding, not retained status. */
export function createRecoveryCollection({ gate, begin, save, beforeStage, afterStage, readMs = 30000, cleanupMs = 150000 }) {
  let consumed = false;
  return async () => {
    if (consumed) throw Error('recovery_collection_consumed'); consumed = true;
    const errors = []; let admission, stopped = false;
    async function collect(phase, operation, persist = true, limit = readMs) {
      let active = true, timer, ticket;
      const scoped = admission && ['ledger', 'original_budget', 'diagnostics', 'state', 'status'].includes(phase);
      try {
        return await Promise.race([Promise.resolve().then(async () => {
          if (scoped && beforeStage) ticket = await beforeStage(phase, limit);
          if (!active) return;
          return operation(() => active);
        }).then(async value => {
          if (!active) return;
          if (persist) await save(phase, value, ticket);
          return value;
        }), new Promise((_, reject) => { timer = setTimeout(() => reject(Object.assign(Error('timeout'), { category: 'timeout' })), limit); })]);
      } catch (error) { errors.push(recoveryErrorRow(phase, error)); }
      finally {
        active = false; clearTimeout(timer);
        if (ticket !== undefined && afterStage) {
          let closingTimer;
          try { await Promise.race([afterStage(phase, ticket), new Promise((_, reject) => { closingTimer = setTimeout(() => reject(Object.assign(Error('timeout'), { category: 'timeout' })), readMs); })]); }
          catch (error) { errors.push(recoveryErrorRow(phase, error)); }
          finally { clearTimeout(closingTimer); }
        }
      }
    }
    try {
      admission = await collect('begin', begin, false);
      if (admission) {
        await collect('ledger', () => gate.reviewQualificationAttempts());
        await collect('original_budget', () => gate.reviewBudget(admission.campaignId));
        await collect('diagnostics', () => gate.exportDiagnostics(), false);
        stopped = true; await collect('stop', () => gate.stop(), false, cleanupMs);
        await collect('state', async () => { await gate.refresh(); return gate.state(); });
        await collect('status', async active => {
          // Stop invalidates the controller's prepared context; collection admission is not status authority.
          const binding = await gate.stratumV2Possession();
          if (!active()) return;
          if (admission.statusMode === 'discover_current') return discoverCurrentStatus(gate, admission.attemptId, binding);
          if (admission.statusMode === 'confirmed' && typeof admission.attemptId === 'string') return gate.stratumV2Status('share', admission.attemptId, binding);
          if (admission.statusMode === 'not_invoked') return gate.stratumV2Status('share', null, binding);
          throw Error('recovery_unknown_start');
        });
      }
    } finally {
      if (!stopped) await collect('stop', () => gate.stop(), false, cleanupMs);
      await collect('closed', () => gate.close(), false, cleanupMs);
      await collect('closed', () => gate.state());
    }
    // Preserve the earliest bounded category independently of legacy evidence formats.
    const firstFailure = errors[0] ?? null;
    await collect('errors', () => save('errors', { schema: RECOVERY_ERRORS_V2, firstFailure, errors: [...errors] }), false);
    const failures = [...new Set(errors.map(row => ['begin', 'stop', 'errors', 'finished'].includes(row.phase) ? 'state' : row.phase))];
    await collect('finished', () => save('finished', { failures }), false);
    return { complete: errors.length === 0, firstFailure: errors[0] ?? null, errors, currentOnly: admission?.statusMode === 'discover_current', qualificationComplete: false };
  };
}

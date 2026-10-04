/** Bounded read-only review loop: the same Gate reviews the panicked baseline issued. */
export const OPERATIONS = Object.freeze(['ledger', 'original_budget', 'possession', 'status']);
export const CATEGORIES = Object.freeze(['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed',
  'read_failed', 'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active', 'operation_failed']);
export const LIMITS = Object.freeze({ iterations: 300, operationMs: 5000, progressEvery: 25 });

function bounded(operation, milliseconds) {
  let timer;
  return Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
    timer = setTimeout(() => reject(Object.assign(Error('timeout'), { category: 'timeout' })), milliseconds);
  })]).finally(() => clearTimeout(timer));
}
/** The Gate's own serial category when it names one in the closed vocabulary, otherwise a generic failure. */
export function failureCategory(error, state) {
  const named = error?.category ?? state?.serialFailureCategory;
  return CATEGORIES.includes(named) ? named : 'operation_failed';
}
/**
 * Runs up to `iterations` rounds and stops at the first failed operation.
 * Progress rows carry only counts and closed categories, never device payloads.
 */
export async function runLoop({ gate, campaignId, record, iterations = LIMITS.iterations, operationMs = LIMITS.operationMs,
  progressEvery = LIMITS.progressEvery }) {
  const calls = {
    ledger: () => gate.reviewQualificationAttempts(),
    original_budget: () => gate.reviewBudget(campaignId),
    possession: async () => { binding = await gate.stratumV2Possession(); },
    status: () => gate.stratumV2Status('share', null, binding),
  };
  let binding = null;
  for (let iteration = 1; iteration <= iterations; iteration++) {
    for (const operation of OPERATIONS) {
      try { await bounded(calls[operation], operationMs); }
      catch (error) {
        const failure = { iteration, operation, category: failureCategory(error, gate.state()) };
        await record({ kind: 'failure', completed: iteration - 1, ...failure });
        return { completed: iteration - 1, failure };
      }
    }
    if (iteration % progressEvery === 0) await record({ kind: 'progress', completed: iteration });
  }
  await record({ kind: 'complete', completed: iterations });
  return { completed: iterations, failure: null };
}

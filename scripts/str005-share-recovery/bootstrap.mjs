/** Baseline admission failures still own independent Stop/Close and durable errors. */
export async function prepareRecovery({ gate, configuration, record, saveFailure, readMs = 30000, cleanupMs = 150000 }) {
  const errors = []; let before, closed;
  async function attempt(phase, operation, limit) {
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Object.assign(Error('timeout'), { category: 'timeout' })), limit);
    })]); }
    catch (error) { errors.push({ phase, category: error?.category === 'timeout' ? 'timeout' : 'operation_failed' }); }
    finally { clearTimeout(timer); }
  }
  try {
    before = await attempt('state', () => {
      const value = gate.state(); if (value.status !== 'ready' || !value.connected || value.running || !value.deviceLeaseInactive) throw Error('baseline'); return value;
    }, readMs);
  } finally {
    await attempt('stop', () => gate.stop(), cleanupMs);
    await attempt('closed', () => gate.close(), cleanupMs);
    closed = await attempt('closed', () => gate.state(), readMs);
  }
  if (!errors.length && !(closed?.status === 'closed' && !closed.connected && closed.serialOwnershipReleased)) errors.push({ phase: 'closed', category: 'operation_failed' });
  if (!errors.length) await attempt('begin', () => record({ before, closed }), readMs);
  if (!errors.length) await attempt('begin', () => gate.configure(configuration), readMs);
  if (errors.length) {
    await attempt('errors', () => saveFailure({ closed, errors: { schema: 'str005-recovery-errors-v1', firstFailure: errors[0], errors: [...errors] },
      finished: { failures: ['state'] } }), readMs);
  }
  return { prepared: errors.length === 0, firstFailure: errors[0] ?? null };
}

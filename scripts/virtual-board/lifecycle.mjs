/** Earliest failure survives independent bounded collection and unconditional cleanup. */
export async function lifecycle({ execute, collect = [], stop, close, release, seal }, timeoutMs = 10000) {
  const facts = [], cleanupFailures = [];
  let earliest = null, generation = 0, released = false;
  const stage = async (phase, operation, cleanup = false) => {
    if (!operation) return;
    const token = ++generation;
    let timer;
    try {
      const value = await Promise.race([
        Promise.resolve().then(() => operation({ active: () => token === generation })),
        new Promise((_, reject) => { timer = setTimeout(() => reject(Error('deadline')), timeoutMs); }),
      ]);
      if (token === generation) facts.push({ phase, value });
      return value;
    } catch (error) {
      const failure = { phase, category: error.message === 'deadline' ? 'deadline' : 'operation_rejected' };
      if (cleanup) cleanupFailures.push(failure);
      else earliest ??= failure;
    } finally { clearTimeout(timer); ++generation; }
  };
  try {
    await stage('execution', execute);
    for (const [phase, operation] of collect) await stage(phase, operation);
  } finally {
    await stage('stop', stop, true);
    await stage('close', close, true);
    released = await stage('release', release, true) === true;
  }
  // Natural completion, release, and historical proof are independent conclusions.
  const result = { earliest_failure: earliest, cleanup_failures: cleanupFailures, facts,
    qualification_success: !earliest && cleanupFailures.length === 0 && released,
    current_resources_released: released, historical_retained_resource_proof: false };
  if (released && cleanupFailures.length === 0) await stage('seal', () => seal?.(result), true);
  return { ...result, cleanup_failures: cleanupFailures, qualification_success: result.qualification_success && cleanupFailures.length === 0 };
}

/** One conservative corpus; unknown changes select every profile. */
export const VERSION = 'ultra205-validation-v1';
export const SEEDS = Object.freeze([1, 19, 205]);
export const SCENARIOS = Object.freeze([
  'healthy-lifecycle', 'stale-safety-step5', 'zero-fan-step5', 'unsafe-reading-step5',
  'heartbeat-loss', 'heap-fragmented', 'internal-allocation-failure', 'max-status',
  'status-allocation-failure', 'delayed-i2c', 'queue-saturation', 'cancel-preparation',
  'cancel-shutdown', 'reboot-before-status', 'retained-record-missing',
  'persistence-failure', 'replay-challenge', 'stop-rejection', 'close-rejection',
  'fixture-completion-failure', 'live-writer',
]);
export const REQUIRED_COVERAGE = Object.freeze([
  'encrypted-fixture-share-ack', 'gate-page-http-seams', 'bounded-real-allocation-paths',
  'all-step-cancellation', 'descendant-listener-release', 'model-conformance-mutations',
  'target-panic-core-decode', 'target-reset-persistence', 'target-resolved-memory-routing',
  'host-target-semantic-agreement',
]);
const subsystemRules = [
  [/^(crates\/bitaxe-(asic|stratum)|firmware\/bitaxe\/src\/asic)/, ['healthy-lifecycle']],
  [/^(crates\/bitaxe-safety|firmware\/bitaxe\/src\/.*sensor)/, ['stale-safety-step5', 'zero-fan-step5', 'unsafe-reading-step5', 'delayed-i2c']],
];
export function requiredProfiles(changedPaths) {
  const selected = new Set(['healthy-lifecycle']);
  if (!changedPaths.length) return [...SCENARIOS];
  for (const path of changedPaths) {
    const rule = subsystemRules.find(([pattern]) => pattern.test(path));
    if (!rule) return [...SCENARIOS];
    for (const profile of rule[1]) selected.add(profile);
  }
  return SCENARIOS.filter(name => selected.has(name));
}

/** Unsupported, omitted, dirty, or failed evidence cannot activate a gate. */
export function qualify(rows, coverage, bindings) {
  const blockers = [];
  for (const backend of ['host', 'qemu']) for (const scenario of SCENARIOS) for (const seed of SEEDS) {
    const matches = rows.filter(row => row.backend === backend && row.scenario === scenario && row.seed === seed);
    if (matches.length !== 1) blockers.push({ backend, scenario, seed, category: 'result_missing_or_duplicate' });
    else if (matches[0].status !== 'passed') blockers.push({ backend, scenario, seed, category: matches[0].status });
  }
  for (const id of REQUIRED_COVERAGE) {
    const matches = coverage.filter(row => row.id === id);
    if (matches.length !== 1 || matches[0].status !== 'passed') blockers.push({ profile: id, category: matches[0]?.status ?? 'unsupported' });
  }
  if (!bindings?.production?.app_elf_sha256) blockers.push({ category: 'production_package_missing' });
  if (bindings?.source_dirty !== false) blockers.push({ category: 'source_not_frozen' });
  return { qualified: blockers.length === 0, blockers, mode: blockers.length ? 'observation' : 'qualified', hardware_qualified: false };
}

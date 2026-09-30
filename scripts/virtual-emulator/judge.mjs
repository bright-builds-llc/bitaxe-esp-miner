import { PROFILE } from './build.mjs';

/** Check observed target values and shared scenario verdicts, never event presence alone. */
export function judgeTargetEvents(events, manifest, { commands = [], scenario, seed = 1 } = {}) {
  const boot = events.find(value => value.event === 'boot');
  const checks = [
    { id: 'target_boot_identity', status: boot?.execution_profile === PROFILE && boot?.compiled_source_sha256 === manifest.compiled_source_sha256 && boot?.heartbeat_cutoff_ms === 2800 ? 'passed' : 'failed' },
    { id: 'target_task_join', status: events.some(value => value.event === 'task' && value.joined === true && value.stack_bytes === 8192 && value.minimum_stack_free_bytes > 0 && value.minimum_stack_free_bytes <= value.stack_bytes && [0, 1].includes(value.core)) ? 'passed' : 'failed' },
  ];
  if (!commands.includes('restart') && !commands.includes('panic') && events.filter(value => value.event === 'boot').length > 1) checks.push({ id: 'target_unexpected_reset', status: 'failed' });
  if (commands.includes('allocation')) {
    const allocation = events.find(value => value.event === 'allocation');
    checks.push({ id: 'target_internal_allocation_release', status: allocation?.released === true && allocation.bytes === 8192 && allocation.before - allocation.during >= 8192 && allocation.after === allocation.before ? 'passed' : 'failed' });
  }
  if (commands.includes('status')) {
    const status = events.find(value => value.event === 'status');
    checks.push({ id: 'target_allocator_observations', status: status?.internal_free > 0 && status.internal_largest > 0 && status.psram_free > 0 && status.internal_largest <= status.internal_free && status.boot === boot?.boot ? 'passed' : 'failed' });
  }
  if (commands.includes('restart')) {
    const boots = events.filter(value => value.event === 'boot');
    checks.push({ id: 'target_reset_persistence', status: boots.length === 2 && boots[1].boot === boots[0].boot + 1 ? 'passed' : 'failed' });
  }
  if (scenario) {
    const margin = events.find(value => value.event === 'scenario_margin');
    checks.push({ id: 'target_main_stack_margin', status: Number.isInteger(margin?.minimum_main_stack_free_bytes) && margin.minimum_main_stack_free_bytes >= 2048 && margin.minimum_main_stack_free_bytes <= 16896 && margin.required_margin_bytes === 2048 && margin.configured_main_stack_bytes === 16384 ? 'passed' : 'failed' });
    const scenarioResult = events.find(value => value.event === 'scenario')?.result;
    checks.push({ id: 'shared_scenario_identity', status: scenarioResult?.scenario === scenario && scenarioResult.seed === seed && scenarioResult.hardware_qualified === false ? 'passed' : 'failed' });
    if (scenarioResult?.checks) checks.push(...scenarioResult.checks.map(check => ({ id: `scenario:${check.id}`, status: check.status })));
  }
  return checks;
}

import { check, object } from '../str005-v2-serial/values.mjs';
const categories = new Set(['panic_detector_stale', 'panic_detector_physical', 'panic_detector_ambiguous', 'panic_detector_identity', 'panic_detector_port',
  'startup_source_changed', 'startup_tool_changed', 'startup_preparation_changed', 'startup_capture_changed', 'startup_prepared_admission',
  'startup_physical_changed', 'startup_review_stale', 'startup_accounting_changed', 'startup_fixture_stale', 'startup_signature_shape',
  'startup_delivery_consumed', 'startup_start_binding', 'startup_start_session', 'startup_effect_admission', 'startup_baseline_stale',
  'startup_prepared_baseline', 'v2_authority_failed', 'v2_authority_unavailable', 'v2_signer_failed',
  'status_repro_record_unverified', 'status_repro_observation_consumed', 'status_repro_source_changed',
  'status_repro_late_completion', 'startup_operation_rejected']);
const phases = new Map([
  ['/cooling-review-context', 'cooling'], ['/cooling-review', 'cooling'], ['/budget-review-context', 'budget'], ['/budget-review', 'budget'],
  ['/startup/fixture', 'fixture'], ['/authorization-context', 'signing'], ['/window-artifacts', 'delivery'], ['/startup/start-admit', 'start_admission'],
  ['/startup/baseline-begin', 'baseline'], ['/startup/baseline', 'baseline'], ['/startup/candidate', 'baseline'],
  ['/startup/release', 'cleanup'], ['/diagnostic-export', 'diagnostics'], ['/status-repro/observe', 'status'],
  ['/status-repro/late-completion', 'cleanup'],
]);
export function failureRecord(path, error) {
  const category = categories.has(error?.code) ? error.code : 'startup_operation_rejected';
  const age = error?.detectorAgeMs;
  return { schema: 'str005-startup-failure-v1', phase: phases.get(path) ?? (path.startsWith('/startup/recovery') ? 'recovery' : 'other'), category,
    detector_age_ms: category === 'panic_detector_stale' && Number.isSafeInteger(age) && age >= 0 && age <= 86400000 ? age : null };
}
export function validateFailure(value) {
  object(value, ['schema', 'phase', 'category', 'detector_age_ms']);
  check(value.schema === 'str005-startup-failure-v1' && [...phases.values(), 'recovery', 'other'].includes(value.phase) && categories.has(value.category) &&
    (value.detector_age_ms === null || (value.category === 'panic_detector_stale' && Number.isSafeInteger(value.detector_age_ms) && value.detector_age_ms >= 0 && value.detector_age_ms <= 86400000)),
  'startup_failure_shape'); return value;
}

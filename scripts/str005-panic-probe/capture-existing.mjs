/** Capture admission uses fresh healthy diagnostics, never a synthetic install pass. */
import { baselineConclusion, currentProof } from './model.mjs';
import { check } from '../str005-v2-serial/values.mjs';

export function reviewExistingCapture(parts, context, observedAtUnixMs = Date.now()) {
  check(context.captureExisting === true && context.installEnabled === false && context.selfTestEnabled === true &&
    context.before_source.firmware_commit === context.firmware_commit && context.before_source.app_elf_sha256 === context.app_elf_sha256 &&
    baselineConclusion(parts).complete, 'panic_existing_capture_baseline');
  currentProof(context, parts, observedAtUnixMs);
  const rows = parts.diagnostics.observations, boot = parts.status.observation.bootOrdinal;
  const preserved = context.corePreservation;
  check(preserved?.schema === 'str005-existing-core-preservation-v1' && preserved.empty_core_dump === true && preserved.bytes === 974848 &&
    ['recovery_seal_sha256', 'result_sha256', 'dump_sha256', 'partition_table_sha256'].every(key => /^[a-f0-9]{64}$/u.test(preserved[key])) &&
    preserved.firmware_commit === context.firmware_commit && preserved.app_elf_sha256 === context.app_elf_sha256 &&
    boot === preserved.expected_boot_ordinal, 'panic_existing_capture_core_preservation');
  const boots = rows.filter(row => row.category === 'boot');
  const afterBoot = rows.slice(rows.findIndex(row => row.category === 'boot') + 1);
  const identities = afterBoot.filter(row => row.category === 'runtime_identity');
  check(boots.length > 0 && new Set(boots.map(row => row.reset_reason)).size === 1 && boots.every(row => row.boot_ordinal === boot && ['software_cpu', 'other'].includes(row.reset_reason)) && identities.length > 0 &&
    identities.every(row => row.firmware_commit === context.firmware_commit && row.app_elf_sha256 === context.app_elf_sha256), 'panic_existing_capture_identity');
  const startup = afterBoot.filter(row => row.category === 'startup');
  check(startup.length > 0 && startup.every(row => row.first_failure === 'none' && row.state !== 'failed') &&
    !rows.some(row => row.category === 'panic' || /failure/u.test(row.category) ||
      (row.category === 'statistics_startup' && ['spawn_failed', 'config_failed', 'cancelled'].includes(row.state))), 'panic_existing_capture_failure');
  const ready = startup.filter(row => row.stage === 'runtime_ready' && row.state === 'complete');
  check(ready.length >= 2 && ready.every((row, index) => Number.isSafeInteger(row.uptime_ms) && row.uptime_ms >= 0 &&
    (index === 0 || row.uptime_ms >= ready[index - 1].uptime_ms)) && new Set(ready.map(row => row.uptime_ms)).size >= 2,
  'panic_existing_capture_readiness');
  return { schema: 'str005-existing-capture-review-v1', capture_admitted: true, installation_complete: false,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, boot_ordinal: boot,
    reset_reason_unresolved: boots.some(row => row.reset_reason === 'other'), core_dump_sha256: preserved.dump_sha256, core_read_result_sha256: preserved.result_sha256,
    observed_at_unix_ms: observedAtUnixMs, continuity_basis: 'current-session-only', historical_resource_proof: false, parity_promotion: false };
}
export function requireCurrentCaptureReview(review, status, now = Date.now()) {
  check(review?.schema === 'str005-existing-capture-review-v1' && review.capture_admitted === true &&
    status.observation.bootOrdinal === review.boot_ordinal && Number.isSafeInteger(review.observed_at_unix_ms) &&
    now >= review.observed_at_unix_ms && now - review.observed_at_unix_ms <= 120000, 'panic_existing_capture_review_stale');
}

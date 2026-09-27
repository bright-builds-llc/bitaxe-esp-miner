/** Capture admission uses fresh healthy diagnostics, never a synthetic install pass. */
import { baselineConclusion, currentProof } from './model.mjs';
import { validateCaptureDiagnosticPair } from './capture-diagnostics.mjs';
import { check } from '../str005-v2-serial/values.mjs';

export function reviewExistingCapture(parts, context, observedAtUnixMs = Date.now()) {
  check(context.captureExisting === true && context.installEnabled === false && context.selfTestEnabled === true &&
    context.before_source.firmware_commit === context.firmware_commit && context.before_source.app_elf_sha256 === context.app_elf_sha256 &&
    baselineConclusion(parts).complete, 'panic_existing_capture_baseline');
  currentProof(context, parts, observedAtUnixMs);
  const boot = parts.status.observation.bootOrdinal;
  const preserved = context.corePreservation;
  check(preserved?.schema === 'str005-existing-core-preservation-v1' && preserved.empty_core_dump === true && preserved.bytes === 974848 &&
    ['recovery_seal_sha256', 'result_sha256', 'dump_sha256', 'partition_table_sha256'].every(key => /^[a-f0-9]{64}$/u.test(preserved[key])) &&
    preserved.firmware_commit === context.firmware_commit && preserved.app_elf_sha256 === context.app_elf_sha256 &&
    boot === preserved.expected_boot_ordinal, 'panic_existing_capture_core_preservation');
  const samples = validateCaptureDiagnosticPair(parts, context);
  return { schema: 'str005-existing-capture-review-v1', capture_admitted: true, installation_complete: false,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, boot_ordinal: boot,
    ...samples, core_dump_sha256: preserved.dump_sha256, core_read_result_sha256: preserved.result_sha256,
    observed_at_unix_ms: observedAtUnixMs, continuity_basis: 'current-session-only', historical_resource_proof: false, parity_promotion: false };
}
export function requireCurrentCaptureReview(review, status, now = Date.now()) {
  check(review?.schema === 'str005-existing-capture-review-v1' && review.capture_admitted === true &&
    status.observation.bootOrdinal === review.boot_ordinal && Number.isSafeInteger(review.observed_at_unix_ms) &&
    now >= review.observed_at_unix_ms && now - review.observed_at_unix_ms <= 120000, 'panic_existing_capture_review_stale');
}

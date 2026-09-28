import { check, object, sha256 } from '../str005-v2-serial/values.mjs';
import { canonical } from '../str005-noise-serial/files.mjs';
import { validateRecoveryParts, recoveryConclusion } from '../str005-v2-serial/recovery-evidence.mjs';
import { baselineConclusion, currentProof } from '../str005-panic-probe/model.mjs';
export const STAGES = ['state', 'ledger', 'original_budget', 'diagnostics', 'status', 'closed', 'finished', 'errors'];
export function finished(value) {
  object(value, ['failures']); check(Array.isArray(value.failures) && new Set(value.failures).size === value.failures.length &&
    value.failures.every(stage => ['begin', 'state', 'ledger', 'original_budget', 'diagnostics', 'status', 'stop', 'closed'].includes(stage)), 'share_recovery_failure_shape');
  return value;
}
export function validateErrors(value) {
  object(value, ['schema', 'firstFailure', 'errors']);
  check(value.schema === 'str005-recovery-errors-v1' && Array.isArray(value.errors) && value.errors.length <= 24, 'share_recovery_errors');
  for (const row of value.errors) {
    object(row, ['phase', 'category']);
    check(['begin', 'state', 'ledger', 'original_budget', 'diagnostics', 'status', 'stop', 'closed', 'errors', 'finished'].includes(row.phase) &&
      ['timeout', 'command_rejected', 'closed', 'shape', 'session', 'io', 'write_failed', 'read_failed', 'v2_idle_correlation', 'v2_attempt_correlation', 'v2_possession', 'not_ready', 'operation_active', 'operation_failed'].includes(row.category), 'share_recovery_errors');
  }
  check(canonical(value.firstFailure) === canonical(value.errors[0] ?? null), 'share_recovery_first_failure'); return value;
}
export function conclusion(parts, context, hostReleased) {
  if (parts.errors) validateErrors(parts.errors);

  const projected = Object.fromEntries(Object.entries(parts).filter(([key]) => !['errors'].includes(key)));
  if (projected.finished) projected.finished = { failures: finished(projected.finished).failures.filter(stage => stage !== 'begin' && stage !== 'stop') };
  validateRecoveryParts(projected, context);
  const idle = parts.status?.state === 'idle' && parts.status.record === null;
  const checked = idle ? baselineConclusion(projected) : recoveryConclusion(projected);
  const blockers = [...checked.blockers];
  if (parts.finished?.failures.length) blockers.push('collection_failed');
  if (!hostReleased) blockers.push('host_resources_unreleased');
  if (!parts.errors || parts.errors.errors.length) blockers.push('collection_errors');
  if (!(parts.status?.observation.bootOrdinal > 15)) blockers.push('post_failure_boot_unconfirmed');
  const boots = parts.diagnostics?.observations?.filter(row => row.category === 'boot') ?? [];
  if (!boots.length || boots.at(-1).boot_ordinal !== parts.status?.observation.bootOrdinal) blockers.push('current_boot_correlation_missing');
  const preservation = parts.state?.preservation, closed = parts.closed;
  if (!(parts.state?.deviceRestorationConfirmed && closed?.deviceRestorationConfirmed && preservation?.authorization_high_water_match &&
    closed?.preservation?.authorization_high_water_match && preservation?.baseline_id === closed?.preservation?.baseline_id)) blockers.push('current_preservation_unconfirmed');
  return { schema: 'str005-share-current-recovery-v1', current_safe_recovery: blockers.length === 0, blockers,
    accounting_measured: Boolean(parts.ledger && parts.original_budget), current_v2_idle: idle,
    retained_resource_proof: !idle && checked.device_resources_released === true,
    historical_resource_unavailable: !(!idle && checked.device_resources_released === true), qualification_complete: false, parity_promotion: false,
    first_failure: parts.errors?.firstFailure ?? null, host_resources_released: hostReleased };
}
export function recoveryProof(parts, context, startedAtUnixMs, now = Date.now()) {
  check(Number.isSafeInteger(startedAtUnixMs) && now >= startedAtUnixMs && now - startedAtUnixMs <= 120000 &&
    conclusion(parts, context, true).current_safe_recovery, 'share_recovery_proof_incomplete');
  const normalized = { ...parts, finished: { failures: [] } }; delete normalized.errors;
  const identity = { ...context, commit: context.source_commit, before_source: context, detector: { physical: context.physical } };
  if (parts.status.state === 'idle') return currentProof(identity, normalized, startedAtUnixMs);
  const boots = parts.diagnostics.observations.filter(row => row.category === 'boot');
  check(boots.length > 0 && boots.at(-1).boot_ordinal === parts.status.observation.bootOrdinal, 'share_recovery_boot_correlation');
  return { schema: 'str005-current-recovery-proof-v2', source_commit: context.source_commit, gate_commit: context.gate_commit,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, physical_identity_sha256: context.physical,
    observed_at_unix_ms: startedAtUnixMs, ledger: parts.ledger, original_budget: parts.original_budget,
    safe_baseline: true, restoration_confirmed: true, device_lease_inactive: true, serial_ownership_released: true,
    preservation_matches: true, mine_on_boot: false, current_v2_released: true,
    retained_attempt_id: context.attemptId, retained_status_sha256: sha256(canonical(parts.status)) };
}

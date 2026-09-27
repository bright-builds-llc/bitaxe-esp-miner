import { validateSelfTestSummary } from './self-test-evidence.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';

export const BASELINE_PARTS = ['state', 'ledger', 'original_budget', 'diagnostics', 'status', 'closed'];
export function baselineIdentity(context) {
  return { ...context.before_source, gate_commit: context.gate_commit };
}
export function baselineReady(state) {
  const p = state?.preservation;
  return state?.status === 'ready' && state.connected === true && state.running === false &&
    state.deviceBaselineConfirmed === true && state.deviceLeaseInactive === true && state.serialOwnershipReleased === false &&
    state.heartbeatSuppressed === false && p?.settings_match === true && p.device_identity_match === true &&
    p.authorization_high_water_match === true && p.mine_on_boot === false;
}
export function validatePart(stage, value, context) {
  check(BASELINE_PARTS.includes(stage) && stage !== 'diagnostics', 'panic_stage');
  if (stage === 'status') return projectRecoveryPart(stage, value, context);
  if (stage === 'ledger') validateLedger(value);
  else if (stage === 'original_budget') requireExhaustedOriginal(value);
  else validateState(value, baselineIdentity(context));
  return structuredClone(value);
}
export function validateFinished(value) {
  object(value, ['failures']);
  check(Array.isArray(value.failures) && value.failures.length <= BASELINE_PARTS.length &&
    new Set(value.failures).size === value.failures.length && value.failures.every(stage => BASELINE_PARTS.includes(stage)), 'panic_failures');
  return structuredClone(value);
}
export function baselineConclusion(parts) {
  const blockers = BASELINE_PARTS.filter(stage => !parts[stage]).map(stage => `missing_${stage}`);
  if (!parts.finished) blockers.push('missing_finished');
  else { validateFinished(parts.finished); blockers.push(...parts.finished.failures.map(stage => `failed_${stage}`)); }
  if (parts.state && !baselineReady(parts.state)) blockers.push('baseline_unconfirmed');
  if (parts.ledger?.pending) blockers.push('accounting_pending');
  if (parts.status && !(parts.status.state === 'idle' && parts.status.record === null && parts.status.observation.clockValid === true)) blockers.push('current_idle_unconfirmed');
  const closed = parts.closed;
  if (closed && !(closed.status === 'closed' && closed.connected === false && closed.running === false &&
    closed.serialOwnershipReleased === true && closed.deviceLeaseInactive === true && closed.deviceRestorationConfirmed === true &&
    closed.preservation?.baseline_id === parts.state?.preservation?.baseline_id &&
    closed.preservation?.settings_match === true && closed.preservation?.device_identity_match === true &&
    closed.preservation?.authorization_high_water_match === true && closed.preservation?.mine_on_boot === false)) blockers.push('release_or_restoration_unconfirmed');
  return { schema: 'str005-panic-baseline-result-v1', complete: blockers.length === 0, blockers,
    next_ordinal: parts.ledger?.next_ordinal ?? null, historical_resource_proof: false, parity_promotion: false };
}

export function currentProof(context, parts, observedAtUnixMs = Date.now()) {
  const result = baselineConclusion(parts);
  check(result.complete, 'panic_baseline_incomplete');
  const expected = baselineIdentity(context);
  check(/^[0-9a-f]{40}$/u.test(expected.firmware_commit) && /^[0-9a-f]{64}$/u.test(expected.app_elf_sha256) &&
    ['state', 'closed'].every(key => parts[key].gateCommit === expected.gate_commit &&
      parts[key].expectedFirmwareSourceCommit === expected.firmware_commit && parts[key].expectedAppElfSha256 === expected.app_elf_sha256),
  'panic_current_proof_identity');
  validateLedger(parts.ledger); requireExhaustedOriginal(parts.original_budget);
  const boot = parts.status.observation.bootOrdinal;
  const boots = parts.diagnostics.observations.filter(row => row.category === 'boot');
  check(boots.length > 0 && boots.at(-1).boot_ordinal === boot, 'panic_boot_correlation');
  return { schema: 'str005-current-recovery-proof-v1', source_commit: context.commit, gate_commit: context.gate_commit,
    firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
    physical_identity_sha256: context.detector.physical, observed_at_unix_ms: observedAtUnixMs,
    ledger: parts.ledger, original_budget: parts.original_budget, safe_baseline: true, restoration_confirmed: true,
    device_lease_inactive: true, serial_ownership_released: true, preservation_matches: true, mine_on_boot: false, current_v2_idle: true };
}

/** Strip only explicitly validated new Gate metadata for the existing state validator. */
export function validateCandidateState(value, context, maybeRequest) {
  const base = { ...value };
  if (Object.hasOwn(base, 'restart')) {
    check(maybeRequest !== undefined, 'panic_unclaimed_restart');
    validateSelfTestSummary(base.restart, maybeRequest); delete base.restart;
  }
  if (base.failure === 'core_dump_self_test_failed') delete base.failure;
  validateState(base, context);
  return structuredClone(value);
}

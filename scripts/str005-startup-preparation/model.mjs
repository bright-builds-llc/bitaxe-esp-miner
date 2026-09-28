import { isDeepStrictEqual as equal } from 'node:util';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { validateRestartSummary } from '../fixed-usb-qualification/reset-origin-restart-state.mjs';
import { validateRestartEvidence } from '../fixed-usb-qualification/reset-origin-restart-evidence.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { check, object, uint, sha256 } from '../str005-v2-serial/values.mjs';
export const FRESH_MS = 120000;
export const STAGES = ['before_state', 'before_ledger', 'before_budget', 'after_state', 'after_ledger', 'after_budget', 'closed'];
export function checkedState(value, context) {
  const { restart, ...common } = value;
  if (restart !== undefined) validateRestartSummary(restart);
  const checked = { ...common, status: common.status === 'restarting' ? 'ready' : common.status };
  if (checked.failure === 'qualification_restart_failed') delete checked.failure;
  validateState(checked, context);
  check(value.running === false && value.renewalsConfirmed === 0 && !value.heartbeatSuppressed, 'preparation_work_forbidden');
  return structuredClone(value);
}
export function baseline(value, context) {
  checkedState(value, context); const p = value.preservation;
  check(value.status === 'ready' && value.connected && value.deviceLeaseInactive && value.deviceBaselineConfirmed && !value.failure &&
    p?.settings_match && p.device_identity_match && p.authorization_high_water_match && !p.mine_on_boot, 'preparation_baseline');
}
export function part(stage, value, context) {
  check(STAGES.includes(stage), 'preparation_stage');
  if (stage.endsWith('_ledger')) validateLedger(value);
  else if (stage.endsWith('_budget')) requireExhaustedOriginal(value);
  else checkedState(value, context);
  return structuredClone(value);
}
export function readyDiagnostics(value, context, expectedBoot) {
  check(value.schema === 'worker-diagnostic-export-v1' && Array.isArray(value.observations), 'preparation_diagnostics');
  const boots = value.observations.filter(row => row.category === 'boot');
  check(boots.length === 1 && boots[0].boot_ordinal === expectedBoot, 'preparation_boot_changed');
  const identities = value.observations.filter(row => row.category === 'runtime_identity');
  check(identities.length === 1 && identities[0].firmware_commit === context.firmware_commit && identities[0].app_elf_sha256 === context.app_elf_sha256,
    'preparation_identity');
  check(value.observations.some(row => row.category === 'startup' && row.stage === 'runtime_ready' && row.state === 'complete' && row.first_failure === 'none'),
    'preparation_runtime_not_ready');
}
/** Gate validates every diagnostic; the established restart validator judges its supported health subset. */
export async function restartPacket(value, context, request, operations = {}) {
  object(value, ['summary', 'ack', 'observations', 'lifecycle']);
  validateRestartSummary(value.summary);
  check(value.summary.expectedBootOrdinal === request.expectedBootOrdinal && value.summary.nextBootOrdinal === request.expectedBootOrdinal + 1, 'preparation_restart_boot');
  if (value.ack !== null) { object(value.ack, ['schema', 'requestNonceSha256', 'bootOrdinal', 'nextBootOrdinal']);
    check(value.ack.schema === 'worker-qualification-restart-v1' && value.ack.requestNonceSha256 === sha256(request.requestNonce) &&
      value.ack.bootOrdinal === request.expectedBootOrdinal && value.ack.nextBootOrdinal === request.expectedBootOrdinal + 1, 'preparation_restart_ack'); }
  check(Array.isArray(value.lifecycle) && value.lifecycle.length <= 16, 'preparation_lifecycle');
  for (const row of value.lifecycle) { object(row, ['record', 'atMs', 'event']); uint(row.record, value.summary.records); uint(row.atMs, value.summary.durationMs);
    check(['prearmed', 'acknowledged', 'stream_interrupted', 'same_port_reopened', 'hello_started', 'complete', 'failed'].includes(row.event), 'preparation_lifecycle'); }
  check(Array.isArray(value.observations) && value.observations.length <= 512, 'preparation_evidence_bound');
  const observations = [];
  for (let index = 0; index < value.observations.length; index += 40) {
    const chunk = value.observations.slice(index, index + 40);
    for (const row of chunk) { object(row, ['record', 'atMs', 'diagnostic']); uint(row.record, value.summary.records); uint(row.atMs, value.summary.durationMs); }
    const parsed = await (operations.validateDiagnostics ?? validateDiagnosticExport)({ schema: 'worker-diagnostic-export-v1', observations: chunk.map(row => row.diagnostic) }, context.gate_root);
    observations.push(...chunk.map((row, i) => ({ ...row, diagnostic: parsed.observations[i] })));
  }
  const retained = { summary: value.summary, ack: value.ack, lifecycle: value.lifecycle, observations };
  return retained;
}
export async function restartEvidence(value, context, request, operations = {}) {
  const retained = await restartPacket(value, context, request, operations);
  const observations = retained.observations;
  const judged = { ...retained, observations: observations.filter(row => ['boot', 'startup', 'runtime_identity', 'panic', 'allocation_failure',
    'allocation_context', 'statistics_startup', 'storage_http_status', 'worker_admission', 'memory'].includes(row.diagnostic.category)) };
  validateRestartEvidence(judged, { ...context, request_nonce: request.requestNonce,
    statistics_startup_required: observations.some(row => row.diagnostic.category === 'statistics_startup' && row.diagnostic.state === 'prepared') }, request.expectedBootOrdinal);
  return retained;
}
export function conclusion(parts, context, hostReleased) {
  const blockers = STAGES.filter(stage => !parts[stage]).map(stage => `missing_${stage}`);
  if (parts.firstFailure) {
    object(parts.firstFailure, ['schema', 'phase', 'category']);
    check(parts.firstFailure.schema === 'str005-preparation-failure-v1' && ['begin', 'part', 'diagnostics', 'claim', 'evidence', 'finish', 'other'].includes(parts.firstFailure.phase) &&
      /^preparation_[a-z_]+$/u.test(parts.firstFailure.category), 'preparation_failure_shape');
    blockers.push('owner_request_failed');
  }
  if (!parts.evidence || parts.evidenceVerified !== true) blockers.push('restart_evidence_unverified');
  if (!parts.finished || parts.finished.failures.length) blockers.push('collection_incomplete');
  if (!hostReleased) blockers.push('resources_unreleased');
  for (const stage of STAGES) if (parts[stage]) part(stage, parts[stage], context);
  for (const stage of ['before_state', 'after_state']) if (parts[stage]) {
    try { baseline(parts[stage], context); } catch (error) { if (error.code !== 'preparation_baseline') throw error; blockers.push(`${stage}_unconfirmed`); }
  }
  const before = parts.before_state?.preservation, after = parts.after_state?.preservation, closed = parts.closed;
  if (!before || !after || before.baseline_id !== after.baseline_id || !equal(before, after)) blockers.push('same_page_preservation_unproved');
  if (!closed || closed.status !== 'closed' || closed.connected || !closed.serialOwnershipReleased || !closed.deviceRestorationConfirmed ||
    !closed.deviceLeaseInactive || !equal(closed.preservation, before)) blockers.push('close_unproved');
  if (!parts.before_ledger || !parts.after_ledger || parts.before_ledger.pending || parts.after_ledger.pending ||
    !equal(parts.before_ledger, context.expectedLedger) || !equal(parts.before_ledger, parts.after_ledger)) blockers.push('accounting_changed');
  if (!equal(parts.before_budget, parts.after_budget) || !equal(parts.before_budget, context.originalBudget)) blockers.push('original_budget_changed');
  return { schema: 'str005-startup-preparation-result-v1', first_failure: parts.firstFailure ?? null, complete: blockers.length === 0, blockers, evidence_failure: parts.evidenceFailure ?? null,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, gate_commit: context.gate_commit,
    historical_gate_commit: context.historical_gate_commit, physical: context.physical, attemptId: context.attemptId,
    parentRoots: context.parentRoots, parentSeals: context.parentSeals, before_boot_ordinal: context.before_boot_ordinal,
    after_boot_ordinal: parts.evidenceVerified ? parts.evidence.summary.nextBootOrdinal : null, reset_reason: parts.evidenceVerified ? 'software_cpu' : null,
    ledger: parts.after_ledger ?? null, original_budget: parts.after_budget ?? null, same_page_preservation: !blockers.includes('same_page_preservation_unproved'),
    mining_started: false, authorization_issued: false, core_cleared: false, historical_startup_qualified: false, parity_promotion: false };
}

import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { baselineReady } from './model.mjs';

/** A Gate export is a latest-value snapshot, never a chronological journal. */
function snapshotHealth(value, context, boot) {
  const rows = value?.observations;
  check(value?.schema === 'worker-diagnostic-export-v1' && Array.isArray(rows), 'panic_capture_snapshot_missing');
  const boots = rows.filter(row => row.category === 'boot'), identities = rows.filter(row => row.category === 'runtime_identity');
  check(boots.length === 1 && boots[0].boot_ordinal === boot && ['software_cpu', 'other'].includes(boots[0].reset_reason) &&
    identities.length === 1 && identities[0].firmware_commit === context.firmware_commit && identities[0].app_elf_sha256 === context.app_elf_sha256,
  'panic_existing_capture_identity');
  const startup = rows.filter(row => row.category === 'startup');
  check(startup.length > 0 && startup.every(row => row.first_failure === 'none' && row.state !== 'failed') &&
    !rows.some(row => row.category === 'panic' || /failure/u.test(row.category) ||
      (row.category === 'statistics_startup' && ['spawn_failed', 'config_failed', 'cancelled'].includes(row.state))), 'panic_existing_capture_failure');
  const ready = startup.filter(row => row.stage === 'runtime_ready' && row.state === 'complete');
  check(ready.length === 1 && Number.isSafeInteger(ready[0].uptime_ms) && ready[0].uptime_ms >= 0, 'panic_existing_capture_readiness');
  return { uptime: ready[0].uptime_ms, reset: boots[0].reset_reason };
}
export function validateCaptureDiagnosticPair(parts, context) {
  const initial = parts.status, confirmation = parts.diagnostics_confirmation_status;
  check(initial && confirmation && baselineReady(confirmation.state) &&
    confirmation.state.preservation.baseline_id === parts.state?.preservation?.baseline_id &&
    confirmation.status?.state === 'idle' && confirmation.status.record === null && confirmation.status.observation.clockValid === true &&
    ['bootOrdinal', 'workerGeneration', 'serialTransportEpoch'].every(key => initial.observation[key] === confirmation.status.observation[key]) &&
    confirmation.status.observation.observedAtUs >= initial.observation.observedAtUs, 'panic_capture_snapshot_session');
  const first = snapshotHealth(parts.diagnostics, context, initial.observation.bootOrdinal);
  const second = snapshotHealth(parts.diagnostics_confirmation, context, initial.observation.bootOrdinal);
  check(second.uptime > first.uptime && second.reset === first.reset, 'panic_capture_snapshot_not_advancing');
  const fileHash = value => sha256(`${JSON.stringify(value, null, 2)}\n`);
  return { reset_reason_unresolved: first.reset === 'other', readiness_uptime_ms: [first.uptime, second.uptime],
    serial_transport_epoch: initial.observation.serialTransportEpoch,
    first_diagnostics_sha256: fileHash(parts.diagnostics), confirmation_diagnostics_sha256: fileHash(parts.diagnostics_confirmation),
    confirmation_status_sha256: fileHash(confirmation) };
}

/** Closed observations localize a store failure; they never grant mining authority. */
import { check } from '../str005-v2-serial/values.mjs';
export function sourceFingerprint(source) {
  check(/^[a-f0-9]{40}$/u.test(source), 'panic_store_source');
  let hash = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(source, 'ascii')) hash = BigInt.asUintN(64, (hash ^ BigInt(byte)) * 0x100000001b3n);
  return hash.toString(16).padStart(16, '0');
}
function boundReceipt(diagnostics, context, boot, origin) {
  const rows = diagnostics?.observations;
  check(Array.isArray(rows), 'panic_store_diagnostics_missing');
  const boots = rows.filter(row => row.category === 'boot');
  check(Number.isSafeInteger(boot) && boot >= 1 && boots.length === 1 &&
    boots[0].boot_ordinal === boot + (origin === 'previous_boot' ? 1 : 0), 'panic_store_boot_correlation');
  const identities = rows.filter(row => row.category === 'runtime_identity');
  check(identities.length === 1 && identities[0].firmware_commit === context.firmware_commit && identities[0].app_elf_sha256 === context.app_elf_sha256,
    'panic_store_runtime_identity');
  const receipts = rows.filter(row => row.category === 'core_dump_store_receipt' && row.origin === origin && row.status === 'valid' &&
    row.source_hash === sourceFingerprint(context.firmware_commit) && row.boot_ordinal === String(boot));
  check(receipts.length === 1, 'panic_store_receipt_missing_or_ambiguous');
  return receipts[0];
}
export function requireStoreReady(diagnostics, context, boot) {
  const row = boundReceipt(diagnostics, context, boot, 'current_boot');
  check(row.stage === 'ready' && row.capacity_bytes === 974848 && row.self_test_marked === false &&
    ['requested_bytes','prepared_bytes','init_result','prepare_result','start_result','end_result','store_result'].every(key => row[key] === 'unavailable'),
  'panic_store_not_ready');
  return { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, boot_ordinal: boot, capacity_bytes: row.capacity_bytes };
}
export function classifyStoreResult(diagnostics, context, panicBoot) {
  const row = boundReceipt(diagnostics, context, panicBoot, 'previous_boot');
  check(row.self_test_marked === true, 'panic_store_self_test_unmarked');
  let category = 'store_progress_incomplete';
  if (row.stage === 'store_returned' && row.store_result === 0) category = 'store_reported_success';
  else if (row.init_result !== 'unavailable' && row.init_result !== 0) category = 'store_initialization_rejected';
  else if (row.init_result === 0 && row.prepare_result === 0x101 && Number.isInteger(row.requested_bytes) &&
    row.prepared_bytes === row.requested_bytes && Math.ceil(row.requested_bytes / 32) * 32 + 32 > row.capacity_bytes) category = 'store_capacity_rejected';
  else if (row.store_result !== 'unavailable' && row.store_result !== 0) category = 'store_failed';
  return { schema: 'str005-core-store-observation-v1', ...row, outcome: category, core_capture_verified: false };
}

export function storeObservation(diagnostics, context, panicBoot) {
  try { return classifyStoreResult(diagnostics, context, panicBoot); }
  catch { return { schema: 'str005-core-store-observation-v1', outcome: 'receipt_unavailable', core_capture_verified: false }; }
}

export function recoveryStoreObservation(diagnostics, status, context, maybeRequest) {
  if (!maybeRequest || status?.observation?.bootOrdinal <= maybeRequest.expectedBootOrdinal) return undefined;
  if (status?.observation?.bootOrdinal > maybeRequest.expectedBootOrdinal + 1)
    return { schema: 'str005-core-store-observation-v1', outcome: 'outside_panic_boot', core_capture_verified: false };
  return storeObservation(diagnostics, context, maybeRequest.expectedBootOrdinal);
}

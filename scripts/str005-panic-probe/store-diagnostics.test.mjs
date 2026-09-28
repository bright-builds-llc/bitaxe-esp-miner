import test from 'node:test';
import assert from 'node:assert/strict';
import { sourceFingerprint, requireStoreReady, classifyStoreResult, recoveryStoreObservation } from './store-diagnostics.mjs';
const context = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64) };
function fixture(previous = false) {
  return { observations: [ { category: 'boot', boot_ordinal: previous ? 8 : 7 }, { category: 'runtime_identity', ...context },
    { category: 'core_dump_store_receipt', authoritative: false, origin: previous ? 'previous_boot' : 'current_boot', status: 'valid',
      source_hash: sourceFingerprint(context.firmware_commit), boot_ordinal: '7', stage: 'ready', capacity_bytes: 974848,
      requested_bytes: 'unavailable', prepared_bytes: 'unavailable', init_result: 'unavailable', prepare_result: 'unavailable',
      start_result: 'unavailable', end_result: 'unavailable', store_result: 'unavailable', self_test_marked: previous } ] };
}
test('fresh current ready receipt is required before a fault', () => {
  assert.equal(requireStoreReady(fixture(), context, 7).capacity_bytes, 974848);
  for (const change of [d => { d.observations[2].status = 'corrupt'; }, d => { d.observations[2].boot_ordinal = '6'; },
    d => { d.observations[2].source_hash = '0'.repeat(16); }, d => { d.observations[2].capacity_bytes = 0; },
    d => { d.observations[2].store_result = 0; }, d => { d.observations[0].boot_ordinal = 9; }]) {
    const d=fixture();change(d);assert.throws(()=>requireStoreReady(d,context,7));
  }
});
test('measured size with unchanged rejected length identifies the SDK capacity boundary', () => {
  // Arrange
  const d = fixture(true); Object.assign(d.observations[2], { stage: 'store_returned', init_result: 0, prepare_result: 257,
    requested_bytes: 1000000, prepared_bytes: 1000000, store_result: 257 });
  // Act
  const result = classifyStoreResult(d,context,7);
  // Assert
  assert.equal(result.outcome,'store_capacity_rejected');assert.equal(result.core_capture_verified,false);
  d.observations[2].prepared_bytes = 1000064;
  assert.equal(classifyStoreResult(d,context,7).outcome,'store_failed');
});
test('initialization failure remains distinct from capacity failure', () => {
  const d=fixture(true);Object.assign(d.observations[2],{stage:'store_returned',init_result:-1,store_result:-1});
  assert.equal(classifyStoreResult(d,context,7).outcome,'store_initialization_rejected');
});
test('SDK success alone cannot verify a captured dump', () => {
  const d=fixture(true);Object.assign(d.observations[2],{stage:'store_returned',init_result:0,prepare_result:0,start_result:0,end_result:0,store_result:0});
  const result=classifyStoreResult(d,context,7);assert.equal(result.outcome,'store_reported_success');assert.equal(result.core_capture_verified,false);
});
test('unmarked or wrong-boot historical data cannot classify the admitted self-test', () => {
  const d=fixture(true);d.observations[2].self_test_marked=false;assert.throws(()=>classifyStoreResult(d,context,7));
  assert.throws(()=>classifyStoreResult(fixture(true),context,8));
});

test('later managed-reset recovery cannot substitute for the immediate panic receipt', () => {
  const request = { expectedBootOrdinal: 7 };
  assert.equal(recoveryStoreObservation(fixture(), { observation: { bootOrdinal: 7 } }, context, request), undefined);
  assert.equal(recoveryStoreObservation(fixture(), { observation: { bootOrdinal: 9 } }, context, request).outcome, 'outside_panic_boot');
  assert.equal(recoveryStoreObservation(fixture(true), { observation: { bootOrdinal: 8 } }, context, request).outcome, 'store_progress_incomplete');
});

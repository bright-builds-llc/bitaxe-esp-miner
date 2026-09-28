import test from 'node:test';
import assert from 'node:assert/strict';
import { validatePartialInstall } from './partial-install.mjs';
function fixture() {
  return { context: { schema: 'str005-panic-probe-v1', renewSuccessor: true, ownerTask: 'task-str005-v2-accepted-share-probe',
    installEnabled: true, selfTestEnabled: false, firmware_commit: '8d4e470e4cd3e18be510d3912586aeda65960275',
    app_elf_sha256: 'd0dd775335e55e9d009a0a779bc938bebf64e76fe78d8d7c8144a06ef1c9d88e' },
  result: { schema: 'str005-panic-baseline-result-v1', complete: false, installation_complete: true, installation_attempted: true,
    candidate_install_reviewed: true, baseline_complete: true, host_resources_released: true,
    blockers: ['candidate_recovery_missing'], candidate_recoveries: [] } };
}
test('successful write with missing candidate recovery is only an identity anchor', () => {
  // Arrange
  const f = fixture();
  // Act
  const review = validatePartialInstall(f.context, f.result);
  // Assert
  assert.equal(review.write_verified, true); assert.equal(review.installation_verified, false);
  assert.equal(review.current_effect_authority, false); assert.equal(review.historical_preservation_verified, false);
});
test('unrelated identities writes failures resources or promoted outcomes are rejected', () => {
  // Arrange / Act / Assert
  for (const change of [f => { f.context.app_elf_sha256 = 'f'.repeat(64); }, f => { f.context.firmware_commit = 'a'.repeat(40); },
    f => { f.result.installation_complete = false; }, f => { f.result.host_resources_released = false; },
    f => { f.result.complete = true; }, f => { f.result.blockers.push('other'); }, f => { f.result.candidate_recoveries.push({ complete: false }); }]) {
    const f = fixture(); change(f); assert.throws(() => validatePartialInstall(f.context, f.result));
  }
});

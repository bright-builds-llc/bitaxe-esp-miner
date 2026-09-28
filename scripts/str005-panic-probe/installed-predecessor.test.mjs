import test from 'node:test';
import assert from 'node:assert/strict';
import { validateInstalledAnchor } from './installed-predecessor.mjs';
import { resolvePreflightSources } from './preflight-selection.mjs';
import { argumentsFor } from './main.mjs';
import { applyRecoveryOnlyOutcome } from './model.mjs';
import { ledger, original } from '../str005-noise-serial/test-fixture.mjs';

function fixture() {
  const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'c'.repeat(40), app_elf_sha256: 'd'.repeat(64), detector: { physical: 'e'.repeat(64) } };
  const candidate = { schema: 'str005-current-recovery-proof-v1', source_commit: context.commit, gate_commit: context.gate_commit,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, physical_identity_sha256: context.detector.physical,
    observed_at_unix_ms: 1000, ledger, original_budget: original, safe_baseline: true, restoration_confirmed: true,
    device_lease_inactive: true, serial_ownership_released: true, preservation_matches: true, mine_on_boot: false, current_v2_idle: true };
  const result = { schema: 'str005-panic-baseline-result-v1', complete: true, installation_complete: true, candidate_install_reviewed: true,
    host_resources_released: true, baseline_complete: true, blockers: [], candidate_recoveries: [{ complete: true, blockers: [] }] };
  const closed = { status: 'closed', connected: false, serialOwnershipReleased: true, deviceRestorationConfirmed: true, deviceLeaseInactive: true,
    expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256,
    preservation: { settings_match: true, device_identity_match: true, authorization_high_water_match: true, mine_on_boot: false } };
  return { context, result, candidate, baseline: structuredClone(candidate), closed };
}
const validate = value => validateInstalledAnchor(value.context, value.result, value.candidate, value.baseline, value.closed);
test('successful candidate lineage is historical identity rather than current effect authority', () => {
  const result = validate(fixture());
  assert.equal(result.predecessor_installation_verified, true);
  assert.equal(result.current_effect_authority, false);
});
test('pre-update proof cannot stand in for the installed candidate', () => {
  const value = fixture(); value.candidate.firmware_commit = 'f'.repeat(40);
  assert.throws(() => validate(value), /prerequisite/);
});
test('incomplete preservation or recovery cannot anchor an installation', () => {
  for (const mutate of [v => { v.result.complete = false; }, v => { v.result.installation_complete = false; },
    v => { v.result.candidate_recoveries[0].complete = false; }, v => { v.result.host_resources_released = false; },
    v => { v.closed.preservation.settings_match = false; }, v => { v.candidate.ledger = { ...ledger, pending: true }; },
    v => { v.baseline.ledger = { ...ledger, total_charged_ms: ledger.total_charged_ms + 180000 }; }]) {
    const value = fixture(); mutate(value); assert.throws(() => validate(value));
  }
});
test('successful-install recovery selection cannot invoke failed-lineage or new-package paths', async () => {
  // Arrange
  const installed = { kind: 'verified_installation', context: fixture().context };
  const forbidden = async () => { throw Error('wrong source selected'); };
  // Act
  const result = await resolvePreflightSources({ '--recover-installed-root': '/installed', '--retained-manifest': '/retained' }, '/repo', 'host', {
    installedPredecessor: async path => { assert.equal(path, '/installed'); return installed; },
    retainedPackage: async (_manifest, source) => { assert.equal(source, installed); return { packaged: 'retained' }; },
    recoveryPredecessor: forbidden, packageSnapshot: forbidden, beforeRecovery: forbidden, verifyCorePreservation: forbidden,
  });
  // Assert
  assert.equal(result.recoveryOnly, true); assert.equal(result.installed, installed);
  assert.equal(result.failedInstall, undefined); assert.equal(result.packaged, 'retained');
});
test('retained source selectors are mutually exclusive', () => {
  const args = ['preflight','--private-root','/p','--gate-root','/g','--recover-installed-root','/i','--retained-manifest','/m'];
  assert.doesNotThrow(() => argumentsFor(args));
  for (const other of ['--recover-install-root','--capture-recovery-root']) assert.throws(() => argumentsFor([...args,other,'/other']));
});
test('read-only successor of successful installation does not falsely report predecessor failure', () => {
  const result = applyRecoveryOnlyOutcome({ complete: true }, false);
  assert.equal(result.predecessor_installation_failed, false);
  assert.equal(result.installation_complete, false);
  assert.equal(result.core_capture_verified, false);
});

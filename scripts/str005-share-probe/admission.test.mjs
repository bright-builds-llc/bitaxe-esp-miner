import test from 'node:test';
import assert from 'node:assert/strict';
import { state, ledger, original } from '../str005-noise-serial/test-fixture.mjs';
import { currentProof } from '../str005-panic-probe/model.mjs';
import { validateIdleRecovery } from '../str005-panic-probe/verified-idle-recovery.mjs';
function fixture() {
  const identity = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40) }, physical = 'd'.repeat(64);
  const context = { ...identity, schema: 'str005-panic-probe-v1', commit: 'e'.repeat(40), before_source: identity, detector: { physical },
    renewSuccessor: true, recoveryOnly: true, ownerTask: 'task-str005-v2-accepted-share-probe', installEnabled: false, selfTestEnabled: false };
  const parts = { state: state(identity), closed: state(identity, 'candidate', true), ledger, original_budget: original,
    status: { state: 'idle', record: null, observation: { bootOrdinal: 12, clockValid: true } },
    diagnostics: { observations: [{ category: 'boot', boot_ordinal: 12 }] }, finished: { failures: [], first_failure: null } };
  const begin = { schema: 'str005-renew-baseline-begin-v1', startedAtUnixMs: 1000 };
  const result = { schema: 'str005-panic-baseline-result-v1', complete: true, baseline_complete: true, host_resources_released: true,
    installation_complete: false, continuity_basis: 'current-session-only', blockers: [] };
  return { identity, physical, value: { context, result, begin, parts, proof: currentProof(context, parts, 1000) } };
}
test('post-clear proof is reconstructed from its exact successful idle producer', () => {
  const f = fixture(); const result = validateIdleRecovery(f.value, f.identity, f.physical);
  assert.equal(result.status.observation.bootOrdinal, 12); assert.equal(result.proof.observed_at_unix_ms, 1000);
});
for (const [label, mutate] of [
  ['failed after early proof', v => { v.result.complete = false; v.result.blockers.push('host_failure'); }],
  ['unreleased host', v => { v.result.host_resources_released = false; }],
  ['unrelated idle boot', v => { v.parts.status.observation.bootOrdinal++; }],
  ['new timestamp', v => { v.proof.observed_at_unix_ms++; }],
  ['wrong image', v => { v.context.before_source.app_elf_sha256 = 'f'.repeat(64); }],
  ['different original budget', v => { v.proof.original_budget = { ...v.proof.original_budget, campaign_id: 'other' }; }],
]) test(`post-clear admission rejects ${label}`, () => {
  const f = fixture(); mutate(f.value);
  assert.throws(() => validateIdleRecovery(f.value, f.identity, f.physical));
});

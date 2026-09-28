import { validatePreparationTimeline } from './prepared-evidence.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { validateCompatibility } from './compatibility.mjs';
import { argumentsFor, admitArguments } from './contract.mjs';
const historical = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40) };
const currentGate = 'd'.repeat(40);
const report = { schema: 'str005-normal-stop-compatibility-v1', firmwareCommit: historical.firmware_commit,
  elfSha256: historical.app_elf_sha256, historicalGateCommit: historical.gate_commit, currentGateCommit: currentGate,
  gateDiffSha256: 'e'.repeat(64), changedGatePaths: ['web/worker-normal-authorization.ts'] };
test('reviewed Gate-only compatibility preserves historical capture identity', () => {
  assert.equal(validateCompatibility(report, historical, currentGate), report);
  assert.equal(historical.gate_commit, 'c'.repeat(40));
});
for (const key of ['firmwareCommit', 'elfSha256', 'historicalGateCommit', 'currentGateCommit'])
  test(`compatibility rejects a changed ${key}`, () => {
    assert.throws(() => validateCompatibility({ ...report, [key]: '0'.repeat(report[key].length) }, historical, currentGate),
      { code: 'startup_gate_compatibility' });
  });
test('new startup admission cannot request another core clear or supply a flash capability', () => {
  const args = ['preflight', '--private-root', '/new/attempt', '--preparation-root', '/sealed/preparation',
    '--gate-root', '/qualified/gate', '--fixture-binary', '/canonical/fixture'];
  assert.equal(argumentsFor(args).action, 'preflight');
  assert.throws(() => admitArguments(['clear', '--private-root', '/new/attempt'], true), { code: 'startup_clear_consumed' });
  assert.throws(() => argumentsFor([...args, '--flash-binary', '/unused/flash']), { code: 'startup_arguments' });
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/new/attempt', '--bindings', '/old/bindings']), { code: 'startup_arguments' });
});

test('preparation admission rejects stale, reversed or missing collection timing', () => {
  validatePreparationTimeline(1000, 2000, 121000);
  for (const values of [[1000, 2000, 121001], [1000, 999, 2000], [1000, 2000, 1999], [undefined, 2000, 3000]])
    assert.throws(() => validatePreparationTimeline(...values), { code: 'startup_preparation_stale' });
});

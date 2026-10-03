import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, realpath, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { argumentsFor, ENABLED, ENABLED_LINE, PINS, taskEnabled, TASK } from './contract.mjs';
import { installation, previousStart } from './lineage.mjs';
import { cleanDiagnosticClose } from './fixture-close.mjs';
import { authorizationRestored } from '../str005-v2-serial/recovery-evidence.mjs';
import { step5Summary } from './summary.mjs';
import { maybeRejection } from './client-core.mjs';

const tasks = line => `## Active\n### ${TASK} | 2026-10-02 | synthetic\n\n${line}\n\n## Future\n`;
const receipt = { category: 'worker_preparation_receipt', authoritative: false, status: 'valid', origin: 'current_boot',
  interrupted: 'false', source_hash: '0123456789abcdef', boot_ordinal: '7', generation: 3, sequence: 9, uptime_ms: '68990',
  last_completed_step: 4, current_step: 5, outcome: 'failed', failure: 'cancelled', heap_free: 3451, heap_largest: 2176, stack_free: 9516 };
const detail = { category: 'worker_revocation_detail', authoritative: false, generation: 3, reason: 'unsafe_observation',
  trigger: 'unsafe_sample', fact: 'bus_voltage', state: 'out_of_range', value_milli: 5512, age_ms: 40, since_safe_ms: 100, closed_ms: 500 };
const parts = observations => ({ recovery: { diagnostics: { schema: 'str005-recovery-diagnostics-v1', observations, omitted_count: 0, authoritative: false } } });

test('effects need both the compiled flag and the exact active task line', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => taskEnabled(tasks(ENABLED_LINE), true));
  assert.throws(() => taskEnabled(tasks(ENABLED_LINE), false), /step5_disabled/u);
  assert.throws(() => taskEnabled(tasks('Step-5 diagnostic Start hardware: disabled.'), true), /step5_disabled/u);
  assert.throws(() => taskEnabled(`## Future\n### ${TASK} | x\n\n${ENABLED_LINE}\n`, true), /step5_disabled/u);
});

test('the compiled effect flag requires a pinned install seal', () => {
  // Arrange / Act / Assert
  assert.ok(!ENABLED || (PINS.installationResult !== null && PINS.installationSeal !== null));
});

test('arguments are absolute, exact and action-specific', () => {
  // Arrange / Act / Assert
  assert.deepEqual(argumentsFor(['finish', '--private-root', '/a']).options, { '--private-root': '/a' });
  assert.throws(() => argumentsFor(['serve', '--private-root', '/a']), /step5_arguments/u);
  assert.throws(() => argumentsFor(['finish', '--private-root', 'relative']), /step5_arguments/u);
  assert.throws(() => argumentsFor(['finish', '--private-root', '/a', '--pool-credentials', '/p']), /step5_arguments/u);
});

test('an unpinned or mismatched installation is never admitted', async t => {
  // Arrange
  const base = await realpath(await mkdtemp(resolve(tmpdir(), 'step5-install-')));
  t.after(() => rm(base, { recursive: true, force: true }));
  const root = resolve(base, 'attempt-001'); await mkdir(root, { mode: 0o700 });
  await writeFile(resolve(root, 'final-result.json'), '{}', { mode: 0o600 });
  // Act / Assert
  await assert.rejects(installation(root, { installationResult: null, installationSeal: null }), /step5_installation_unpinned/u);
  await assert.rejects(installation(root, { installationResult: 'a'.repeat(64), installationSeal: 'b'.repeat(64) }), /step5_installation_anchor/u);
});

test('a retained detail for the failed generation diagnoses the revocation', () => {
  // Arrange / Act
  const summary = step5Summary(parts([receipt, detail]), { complete: false }, { rejection: 'session_failed' }).step5;
  // Assert
  assert.equal(summary.outcome, 'revocation_diagnosed');
  assert.deepEqual([summary.revocation_detail.fact, summary.worker_rejection, summary.cause_proven], ['bus_voltage', 'session_failed', false]);
});

test('a detail from another generation does not diagnose this attempt', () => {
  // Arrange / Act
  const summary = step5Summary(parts([receipt, { ...detail, generation: 2 }]), { complete: false }, undefined).step5;
  // Assert
  assert.deepEqual([summary.outcome, summary.revocation_detail], ['unverified', null]);
});

test('a complete normal Start reports step 5 passed', () => {
  // Arrange / Act / Assert
  assert.equal(step5Summary(parts([]), { complete: true }, undefined).step5.outcome, 'step5_passed');
});

test('only a closed Worker rejection is kept, including inside a cleanup aggregate', () => {
  // Arrange
  const rejected = Object.assign(new Error('x'), { rejection: 'session_failed' });
  // Act / Assert
  assert.equal(maybeRejection(rejected), 'session_failed');
  assert.equal(maybeRejection(new AggregateError([new Error('stop'), rejected])), 'session_failed');
  assert.equal(maybeRejection(Object.assign(new Error('x'), { rejection: 'free text' })), null);
});

const facts = { natural_exit: true, exact_peer: true, closed: true, received_shares: 0, accepted_shares: 0, invalid_shares: 0,
  outcome: 'unverified', first_failure_stage: 'peer_eof' };
const optional = { schema: 'str005-startup-fixture-completion-v1', required: false, complete: null };

test('a status-only Start completes its fixture by a clean natural close without a share', () => {
  // Arrange / Act / Assert
  assert.equal(cleanDiagnosticClose(optional, facts), true);
});

test('a share-required, terminated, foreign-peer or invalid-share fixture is not a clean close', () => {
  // Arrange
  const cases = [[{ ...optional, required: true, complete: false }, facts], [optional, { ...facts, natural_exit: false }],
    [optional, { ...facts, exact_peer: false }], [optional, { ...facts, invalid_shares: 1 }],
    [optional, { ...facts, first_failure_stage: 'connect' }]];
  // Act / Assert
  assert.deepEqual(cases.map(([completion, value]) => cleanDiagnosticClose(completion, value)), [false, false, false, false, false]);
});

test('authorization is restored by the Start generation\'s own authenticated recovery match', () => {
  // Arrange
  const state = { preservation: { authorization_high_water_match: false }, qualification: { generation: 4 },
    authorizationRecovery: { matched: true, generation: 4 } };
  // Act / Assert
  assert.equal(authorizationRestored(state, 4), true);
  assert.equal(authorizationRestored(state, 3), false);
  assert.equal(authorizationRestored({ ...state, authorizationRecovery: { matched: null, generation: 4 } }, 4), false);
});

test('a rerun without its pinned previous Start is never admitted', async () => {
  // Arrange / Act / Assert
  await assert.rejects(previousStart('/nonexistent', {}, { previousStartResult: null, previousStartSeal: null }), /step5_previous_unpinned/u);
});

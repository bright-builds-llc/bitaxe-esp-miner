import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseArguments } from './main.mjs';
import { SCENARIOS, SEEDS, REQUIRED_COVERAGE, qualify, requiredProfiles, framedLinkCoverage } from './profiles.mjs';
import { lifecycle } from './lifecycle.mjs';
import { parityFromLedger } from './parity.mjs';

test('unknown changes select the full corpus', () => {
  assert.deepEqual(requiredProfiles(['tools/flash/src/main.rs']), [...SCENARIOS]);
  assert.deepEqual(requiredProfiles([]), [...SCENARIOS]);
});
test('missing and unsupported coverage prevent qualification', () => {
  const rows = ['host', 'qemu'].flatMap(backend => SCENARIOS.flatMap(scenario => SEEDS.map(seed => ({ backend, scenario, seed, status: 'passed' }))));
  const coverage = REQUIRED_COVERAGE.map(id => ({ id, status: 'passed' }));
  const bindings = { source_dirty: false, production: { app_elf_sha256: 'a'.repeat(64) } };
  assert.equal(qualify(rows, coverage, bindings).qualified, true);
  coverage[0].status = 'unsupported';
  assert.equal(qualify(rows, coverage, bindings).qualified, false);
  coverage[0].status = 'passed'; rows.pop();
  assert.equal(qualify(rows, coverage, bindings).qualified, false);
});
test('dirty source prevents freezing a candidate', () => {
  assert.equal(qualify([], [], { source_dirty: true }).qualified, false);
});
test('CLI rejects effect flags, bad seeds and unknown scenarios', () => {
  for (const args of [
    ['run', '--scenario', 'healthy-lifecycle', '--evidence-dir', 'new', '--flash', 'yes'],
    ['run', '--scenario', 'healthy-lifecycle', '--evidence-dir', 'new', '--seed', 'NaN'],
    ['run', '--scenario', 'unknown', '--evidence-dir', 'new'],
    ['preflash', '--evidence-dir', 'new'],
  ]) assert.throws(() => parseArguments(args));
});
test('failure preserves first phase and independently records cleanup rejection', async () => {
  const events = [];
  const fail = name => () => { events.push(name); throw Error('rejected'); };
  const result = await lifecycle({ execute: fail('execute'), collect: [['accounting', fail('accounting')]],
    stop: fail('stop'), close: () => events.push('close'), release: () => { events.push('release'); return true; }, seal: () => events.push('seal') });
  assert.deepEqual(events, ['execute', 'accounting', 'stop', 'close', 'release']);
  assert.equal(result.earliest_failure.phase, 'execution');
  assert.equal(result.cleanup_failures[0].phase, 'stop');
  assert.equal(result.current_resources_released, true);
  assert.equal(result.qualification_success, false);
});
test('expired collection rejects late writes and still closes and releases', async () => {
  let active, closed = false;
  const result = await lifecycle({ execute: () => true,
    collect: [['status', ({ active: check }) => { active = check; return new Promise(() => {}); }]],
    stop: () => true, close: () => { closed = true; }, release: () => true, seal: () => true }, 10);
  assert.equal(active(), false);
  assert.equal(closed, true);
  assert.equal(result.earliest_failure.category, 'deadline');
  assert.equal(result.facts.some(row => row.phase === 'status'), false);
});
test('failed natural completion remains failed after successful release', async () => {
  const result = await lifecycle({ execute: () => { throw Error('fixture failed'); }, stop: () => true, close: () => true, release: () => true });
  assert.equal(result.qualification_success, false);
  assert.equal(result.current_resources_released, true);
  assert.equal(result.historical_retained_resource_proof, false);
});

test('each collection failure leaves the other current facts available', async () => {
  for (const failed of ['accounting', 'diagnostics', 'status']) {
    const result = await lifecycle({ execute: () => true,
      collect: ['accounting', 'diagnostics', 'status'].map(phase => [phase, () => { if (phase === failed) throw Error('read failed'); return phase; }]),
      stop: () => true, close: () => true, release: () => true });
    assert.equal(result.earliest_failure.phase, failed);
    assert.equal(result.facts.filter(fact => ['accounting', 'diagnostics', 'status'].includes(fact.phase)).length, 2);
    assert.equal(result.current_resources_released, true);
  }
});

test('Close rejection is recorded after Stop rejection and prevents sealing', async () => {
  let sealed = false;
  const result = await lifecycle({ execute: () => true, stop: () => { throw Error('stop'); }, close: () => { throw Error('close'); },
    release: () => true, seal: () => { sealed = true; } });
  assert.deepEqual(result.cleanup_failures.map(failure => failure.phase), ['stop', 'close']);
  assert.equal(sealed, false);
});

import { classifyHost } from './compiler-bindings.mjs';
function admittedHost() {
  const expected = { scenario: 'healthy-lifecycle', seed: 1, binarySha: 'binary', source: { source_commit: 'source', source_dirty: false }, compiler: { compiled_inputs_sha256: 'compiled', worker_control_sha256: 'controller', fixture_sha256: 'fixture', cargo_lock_sha256: 'lock' }, production: null };
  const envelope = { schema: 'bitaxe_virtual_board_run_v3', scenario: expected.scenario, seed: 1, executable_sha256: 'binary', ...expected.source, ...expected.compiler, package_binding: null, passed: true, result: { checks: [{ status: 'passed' }] } };
  const life = { qualification_success: true, current_resources_released: true, earliest_failure: null, cleanup_failures: [] };
  return { expected, envelope, life };
}
test('a passing result cannot override failed natural completion', () => {
  const { expected, envelope, life } = admittedHost();
  life.qualification_success = false; life.earliest_failure = { phase: 'execution', category: 'operation_rejected' };
  assert.equal(classifyHost(envelope, life, expected), 'failed');
});
test('wrong invocation or compiled source identity rejects otherwise passing evidence', () => {
  for (const field of ['scenario', 'seed', 'compiled_inputs_sha256', 'worker_control_sha256', 'fixture_sha256', 'cargo_lock_sha256', 'executable_sha256']) {
    const { expected, envelope, life } = admittedHost(); envelope[field] = 'different';
    assert.equal(classifyHost(envelope, life, expected), 'failed', field);
  }
});
test('report parity comes from the latest ledger record', () => {
  // Arrange
  const ledger = [
    { schema_version: 'parity-progress-v1', verified_total: 90, active_total: 95 },
    { schema_version: 'parity-progress-v1', verified_total: 92, active_total: 96 },
  ].map(record => JSON.stringify(record)).join('\n') + '\n';

  // Act
  const parity = parityFromLedger(ledger);

  // Assert
  assert.equal(parity, '92/96');
});
test('an empty or malformed parity ledger fails visibly', () => {
  assert.throws(() => parityFromLedger('\n'), /parity_ledger_empty/);
  assert.throws(() => parityFromLedger('{"schema_version":"other","verified_total":1,"active_total":2}\n'), /parity_ledger_invalid/);
});
test('passing host link checks mark the framed serial link covered', () => {
  // Arrange
  const rows = [
    { backend: 'host', status: 'passed', result: { checks: [{ id: 'serial_link', status: 'passed' }, { id: 'serial_output', status: 'passed' }] } },
    { backend: 'host', status: 'passed', result: { checks: [{ id: 'link_heartbeat_timeout', status: 'passed' }] } },
    { backend: 'host', status: 'unsupported', result: { checks: [{ id: 'serial_link', status: 'passed' }, { id: 'live', status: 'unsupported' }] } },
    { backend: 'qemu', status: 'unsupported' },
  ];
  // Act
  const coverage = framedLinkCoverage(rows);
  // Assert
  assert.equal(coverage.status, 'passed');
});
test('a failed link check or failed host row blocks framed link coverage', () => {
  const failedCheck = [{ backend: 'host', status: 'passed', result: { checks: [{ id: 'serial_link', status: 'failed' }] } }];
  const failedRow = [{ backend: 'host', status: 'failed', result: { checks: [{ id: 'serial_link', status: 'passed' }] } }];
  assert.equal(framedLinkCoverage(failedCheck).status, 'failed');
  assert.equal(framedLinkCoverage(failedRow).status, 'failed');
});
test('host rows without link checks leave framed link coverage unsupported', () => {
  assert.equal(framedLinkCoverage([{ backend: 'host', status: 'passed', result: { checks: [] } }]).status, 'unsupported');
});
test('framed link coverage requires judged device output', () => {
  // Arrange
  const rows = [{ backend: 'host', status: 'passed', result: { checks: [{ id: 'serial_link', status: 'passed' }] } }];
  // Act
  const coverage = framedLinkCoverage(rows);
  // Assert
  assert.deepEqual(coverage, { status: 'unsupported', category: 'device_output_not_exercised' });
});
test('a failed device output check fails framed link coverage', () => {
  const rows = [{ backend: 'host', status: 'passed', result: { checks: [{ id: 'serial_link', status: 'passed' }, { id: 'serial_output', status: 'failed' }] } }];
  assert.equal(framedLinkCoverage(rows).status, 'failed');
});
test('Gate controller coverage is a required qualification row', () => {
  assert.ok(REQUIRED_COVERAGE.includes('gate-controller-serial'));
  assert.ok(REQUIRED_COVERAGE.includes('gate-page-http-seams'));
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, readFile, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { claimPrefix, judgePrefix, judgePrefixSnapshots, runNoisePrefix, PREFIX_STOPS, admitPrefixPackage, redactedPrefixFailure, validatePrefixCommit, qualificationFailure, prefixLogFacts } from './noise-prefix.mjs';

async function mkPrivateTemp(prefix) { return realpath(await mkdtemp(prefix)); }
const source = 'a'.repeat(64);
function rows(stop, released = true) {
  return [...Array.from({ length: stop - 100 }, (_, i) => i + 101), ...(released ? [114] : [])].map(phase => ({ phase, stage: 2,
    integrity: true, stack_pointer_inside: true, stack_span_kind: 'configured', configured_main_stack_bytes: 16384, stack_low_water: 4096 }));
}
function events(stop = 107) {
  return [{ event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: source, heartbeat_cutoff_ms: 2800 },
    { event: 'noise_prefix', result: { schema: 'bitaxe-noise-prefix-v1', seed: 1, selected_stop: stop,
      boundary_reached: true, resources_released: true, subset_only: true, authenticated: false, frame_round_trip: false,
      full_probe_qualified: false, hardware_qualified: false }, checkpoints: rows(stop), configured_main_stack_bytes: 16384,
      required_margin_bytes: 2048, minimum_main_stack_free_bytes: 4096 }];
}
function snapshot(kind, stop = 107) {
  return { kind, registers_available: true, checkpoint_memory_available: true, scope_violation: false,
    task_bounds: [{ task_label: 'main', bounds_available: true, captured_sp_inside_bounds: true, saved_tcb_top_inside_bounds: true }],
    prefix_argument_matches: true, prefix_reached_phase: stop, prefix_released_flag: kind === 'released' ? 1 : 0, prefix_flags_available: true,
    records: rows(stop, kind === 'released').map(row => ({ phase: row.phase, stage: row.stage, heap_integrity: true,
      heap_observation_available: true, stack_pointer_inside_configured_span: true, configured_stack_span_bytes: 16384, stack_low_water_bytes: 4096 })) };
}
function decoded(stop = 107) {
  return { schema: 'bitaxe-noise-live-snapshots-v1', cutoff_phase: stop, snapshots: [snapshot('prefix', stop), snapshot('released', stop)] };
}

test('disabled prefix gate rejects before source/package/process access', async () => {
  // Arrange
  const missing = '/nonexistent/prefix';
  // Act / Assert
  await assert.rejects(runNoisePrefix(missing, missing, missing, { stop: 107, disableEffects: true }), /noise_prefix_effect_gate_disabled/);
});

test('unsafe and unsupported prefix selectors fail before package access', async () => {
  // Arrange
  const rejected = [100, 104, 108, 111, 114, 0, NaN];
  // Act / Assert
  for (const stop of rejected) await assert.rejects(admitPrefixPackage('/missing', '/missing', source, stop), /noise_prefix_stop_unsupported/);
});

test('fixed prefix campaign admits each cutoff once with unchanged bytes', async () => {
  // Arrange
  const root = await mkPrivateTemp(join(tmpdir(), 'prefix-claims-')), binding = { package_sha256: source, validator_sha256: 'b'.repeat(64) };
  try {
    // Act
    for (const stop of PREFIX_STOPS) await claimPrefix(root, binding, stop, `${stop}`);
    // Assert
    assert.equal(JSON.parse(await readFile(join(root, 'cutoff-107.claim.json'))).selected_stop, 107);
    await assert.rejects(claimPrefix(root, binding, 107, '107'), /noise_prefix_series_exhausted/);
  } finally { await rm(root, { recursive: true }); }
});

test('an unchanged prefix retry is rejected by the durable claim', async () => {
  // Arrange
  const root = await mkPrivateTemp(join(tmpdir(), 'prefix-retry-'));
  try {
    await claimPrefix(root, { elf_sha256: source }, 107, 'audit-one');
    // Act / Assert
    await assert.rejects(claimPrefix(root, { elf_sha256: source }, 107, 'audit-one'), { code: 'EEXIST' });
  } finally { await rm(root, { recursive: true }); }
});

test('a new image cannot masquerade as one runtime-selector campaign', async () => {
  // Arrange
  const root = await mkPrivateTemp(join(tmpdir(), 'prefix-binding-'));
  try {
    await claimPrefix(root, { elf_sha256: source }, 107, 'audit-one');
    // Act / Assert
    await assert.rejects(claimPrefix(root, { elf_sha256: 'b'.repeat(64) }, 109, 'audit-two'), /noise_prefix_series_binding_changed/);
  } finally { await rm(root, { recursive: true }); }
});

test('completed prefix preserves subset status and does not qualify authentication', () => {
  // Arrange / Act
  const checks = judgePrefix(events(), 107, source);
  // Assert
  assert.equal(checks.every(check => check.status === 'passed'), true);
});

for (const [label, mutate, id] of [
  ['authentication credit', value => { value[1].result.authenticated = true; }, 'subset_only'],
  ['full-probe credit', value => { value[1].result.full_probe_qualified = true; }, 'subset_only'],
  ['wrong selected stop', value => { value[1].result.selected_stop = 110; }, 'selected_prefix_completed'],
  ['missing post-drop checkpoint', value => value[1].checkpoints.pop(), 'selected_journal_integrity'],
  ['missing measured margin', value => { delete value[1].minimum_main_stack_free_bytes; }, 'existing_stack_margin'],
]) {
  test(`prefix result rejects ${label}`, () => {
    // Arrange
    const value = events(); mutate(value);
    // Act
    const checks = judgePrefix(value, 107, source);
    // Assert
    assert.equal(checks.find(check => check.id === id).status, 'failed');
  });
}

test('pre-release proof and post-drop proof are independently required', () => {
  // Arrange / Act
  const facts = judgePrefixSnapshots(decoded(), 107);
  // Assert
  assert.deepEqual(facts, { prefix_state: 'safe', cleanup_state: 'safe' });
});

test('clean prefix followed by destructor panic preserves prefix but fails cleanup', () => {
  // Arrange
  const value = decoded(); value.snapshots[1] = { kind: 'panic' };
  // Act
  const facts = judgePrefixSnapshots(value, 107);
  // Assert
  assert.deepEqual(facts, { prefix_state: 'safe', cleanup_state: 'failed' });
});

test('missing first-fault RAM proof remains unknown', () => {
  // Arrange / Act
  const facts = judgePrefixSnapshots({ schema: 'bitaxe-noise-live-snapshots-v1', cutoff_phase: 107, snapshots: [] }, 107);
  // Assert
  assert.deepEqual(facts, { prefix_state: 'unknown', cleanup_state: 'unknown' });
});

test('heap metadata fault before the boundary cannot become safe after host termination', () => {
  // Arrange
  const value = decoded(); value.snapshots[0].records[6].heap_integrity = null;
  // Act
  const facts = judgePrefixSnapshots(value, 107);
  // Assert
  assert.equal(facts.prefix_state, 'unknown');
});

test('wrong live-prefix argument is rejected even when all journal rows look healthy', () => {
  // Arrange
  const value = decoded(); value.snapshots[0].prefix_argument_matches = false;
  // Act
  const facts = judgePrefixSnapshots(value, 107);
  // Assert
  assert.equal(facts.prefix_state, 'failed');
});


test('unknown errors cannot publish private paths through categories', () => {
  // Arrange
  const error = Error('read failed at /private/raw-debugger-and-secret');
  // Act
  const category = redactedPrefixFailure(error);
  // Assert
  assert.equal(category, 'noise_prefix_operation_failed');
});

test('prefix CLI rejects additional scenario and mining orchestration options before setup', async () => {
  // Arrange
  const { main } = await import('./main.mjs');
  // Act / Assert
  for (const key of ['--commands', '--scenario', '--mode']) {
    await assert.rejects(main(['noise-prefix', '--manifest', 'synthetic', '--audit', 'synthetic', '--stop', '107', key, 'synthetic']), /noise_prefix_arguments/);
  }
});


test('prefix package must be clean and bound to the current published source commit', () => {
  // Arrange
  const head = 'a'.repeat(40);
  // Act / Assert
  assert.doesNotThrow(() => validatePrefixCommit({ source_commit: head, source_dirty: false }, head));
  for (const manifest of [{ source_commit: head, source_dirty: true }, { source_commit: head },
    { source_commit: 'b'.repeat(40), source_dirty: false }]) {
    assert.throws(() => validatePrefixCommit(manifest, head), /noise_prefix_commit_binding/);
  }
});


test('captured target panic and later debugger timeout stay independent', () => {
  // Arrange
  const firstHost = { phase: 'debugger', category: 'emulator_command_failed' };
  const result = { first_host_failure: firstHost, first_target_fault: { phase: 'prefix', category: 'first_target_panic', snapshot_verified: true }, collection_failures: [] };
  // Act
  const failure = qualificationFailure(result);
  // Assert
  assert.deepEqual(failure, { phase: 'prefix', category: 'first_target_panic' });
  assert.deepEqual(result.first_host_failure, firstHost);
});


test('malformed guest JSON cannot suppress an independently observed SDK panic', () => {
  // Arrange
  const boot = { event: 'boot', compiled_source_sha256: source };
  const log = `VIRTUAL_U205 ${JSON.stringify(boot)}\nVIRTUAL_U205 {"event":"noise_prefix","result":\nGuru Meditation Error: Core 0 panic'ed (LoadProhibited)\n`;
  // Act
  const facts = prefixLogFacts(log);
  // Assert
  assert.equal(facts.unexpected_panic, true);
  assert.equal(facts.invalid_guest_json, true);
  assert.deepEqual(facts.events, [boot]);
  const observed = { phase: 'prefix', category: 'unexpected_target_panic', snapshot_verified: false };
  assert.deepEqual(qualificationFailure({ first_target_fault: observed }), { phase: 'prefix', category: 'unexpected_target_panic' });
  assert.equal(observed.exception_category, undefined);
});


test('a reached checkpoint without task bounds remains unknown', () => {
  // Arrange
  const value = decoded(); value.snapshots[0].task_bounds = null;
  // Act
  const facts = judgePrefixSnapshots(value, 107);
  // Assert
  assert.equal(facts.prefix_state, 'unknown');
  assert.equal(facts.cleanup_state, 'safe');
});

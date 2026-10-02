import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { claimPrefix, judgePrefix, prefixTelemetryEvents, runNoisePrefix, validatePrefixCommit } from './noise-prefix.mjs';

const source = 'a'.repeat(64);
function events(stop = 106) {
  return [{ event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: source, heartbeat_cutoff_ms: 2800 },
    { event: 'noise_prefix', result: { schema: 'bitaxe-noise-prefix-v1', seed: 1, selected_stop: stop,
      boundary_reached: true, resources_released: true, subset_only: true, authenticated: false,
      frame_round_trip: false, full_probe_qualified: false, hardware_qualified: false },
    checkpoints: [...Array.from({ length: stop - 100 }, (_, index) => 101 + index), 114].map(phase => ({
      phase, stage: 2, integrity: true, stack_pointer_inside: true, stack_span_kind: 'configured',
      configured_main_stack_bytes: 16384, stack_low_water: 4096 })),
    configured_main_stack_bytes: 16384, required_margin_bytes: 2048, minimum_main_stack_free_bytes: 4096 }];
}

test('ordinary records support only partial prefix checks', () => {
  // Arrange
  const lines = events().map(event => `VIRTUAL_U205 ${JSON.stringify(event)}\n`).join('');
  // Act
  const checks = judgePrefix(prefixTelemetryEvents(lines), 106, source);
  // Assert
  assert.equal(checks.every(check => check.status === 'passed'), true);
  assert.equal(events()[1].result.full_probe_qualified, false);
});

test('unprojected or truncated application records stop collection', () => {
  // Arrange / Act / Assert
  assert.throws(() => prefixTelemetryEvents('ordinary startup text\n'), /prefix_projection_invalid/);
  assert.throws(() => prefixTelemetryEvents('VIRTUAL_U205 {\n'), /prefix_application_record_invalid/);
  assert.throws(() => prefixTelemetryEvents('VIRTUAL_U205 null\n'), /prefix_application_record_invalid/);
});

test('missing completion cannot become a healthy prefix', () => {
  // Arrange
  const bootOnly = events().slice(0, 1);
  // Act
  const checks = judgePrefix(bootOnly, 106, source);
  // Assert
  assert.equal(checks.find(check => check.id === 'selected_prefix_completed').status, 'failed');
});

test('unsafe observations and lost release evidence fail independently', () => {
  // Arrange
  const value = events(); value[1].checkpoints[2].integrity = false;
  value[1].result.resources_released = false;
  // Act
  const checks = judgePrefix(value, 106, source);
  // Assert
  assert.equal(checks.find(check => check.id === 'selected_journal_integrity').status, 'failed');
  assert.equal(checks.find(check => check.id === 'crypto_owners_released').status, 'failed');
});

test('a one-use telemetry cutoff claim cannot be replayed', async () => {
  // Arrange
  const root = await realpath(await mkdtemp(join(tmpdir(), 'telemetry-claim-')));
  try {
    await claimPrefix(root, { elf_sha256: source }, 106, 'receipt');
    // Act / Assert
    await assert.rejects(claimPrefix(root, { elf_sha256: source }, 106, 'receipt'), { code: 'EEXIST' });
  } finally { await rm(root, { recursive: true }); }
});

test('an unpublished package is rejected without touching execution', () => {
  // Arrange
  const commit = 'a'.repeat(40);
  // Act / Assert
  assert.throws(() => validatePrefixCommit({ source_commit: commit, source_dirty: true }, commit), /commit_binding/);
});

test('the disabled effect seam rejects before any source or process access', async () => {
  // Arrange / Act / Assert
  await assert.rejects(runNoisePrefix('/missing', '/missing', '/missing', { stop: 106, disableEffects: true }), /effect_gate_disabled/);
});

test('active runner uses projected records and has no debugger or partition inspection calls', async () => {
  // Arrange / Act
  const code = await readFile(new URL('./noise-prefix.mjs', import.meta.url), 'utf8');
  // Assert
  assert.match(code, /outputLinePrefix: 'VIRTUAL_U205 '/);
  assert.match(code, /independent_task_bounds: 'unsupported'/);
  assert.doesNotMatch(code, /prepareNoiseDebugger|decodeNoiseDebugger|decodeCore|inspectCore|listenerReleased|first_target_fault/);
});

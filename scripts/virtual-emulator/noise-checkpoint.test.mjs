import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, rm, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkpointEvents, checkpointObservations, claimCheckpoint, judgeComposedCheckpoint, judgeFullCheckpoint,
  redactedCheckpointFailure, runNoiseCheckpoint } from './noise-checkpoint.mjs';

const source = 'a'.repeat(64);
function events() {
  return [{ event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: source, heartbeat_cutoff_ms: 2800 },
    { event: 'noise_probe', result: { schema: 'bitaxe-noise-probe-v1', seed: 1, authenticated: true, frame_round_trip: true,
      payload_bytes: 32, resources_released: true, hardware_qualified: false },
    checkpoints: Array.from({ length: 14 }, (_, index) => ({ phase: 101 + index, stage: 2, integrity: true,
      stack_pointer_inside: true, stack_span_kind: 'configured', configured_main_stack_bytes: 16384,
      stack_low_water: 3000, internal_free: 300000, internal_largest: 100000 })),
    configured_main_stack_bytes: 16384, required_margin_bytes: 2048, minimum_main_stack_free_bytes: 3000 }];
}
const status = (checks, id) => checks.find(check => check.id === id).status;

test('complete projected records pass every full-probe check', () => {
  // Arrange
  const lines = events().map(event => `VIRTUAL_U205 ${JSON.stringify(event)}\n`).join('');
  // Act
  const checks = judgeFullCheckpoint(checkpointEvents(lines), source);
  // Assert
  assert.deepEqual(checks.map(check => check.status), ['passed', 'passed', 'passed', 'passed', 'passed']);
});

test('a missing probe record stops inference instead of passing', () => {
  // Arrange
  const bootOnly = events().slice(0, 1);
  // Act
  const checks = judgeFullCheckpoint(bootOnly, source);
  // Assert
  assert.equal(status(checks, 'noise_authenticated_round_trip'), 'failed');
  assert.equal(status(checks, 'noise_stack_margin'), 'failed');
  assert.deepEqual(checkpointObservations(bootOnly), { probe_event: 'absent' });
});

test('a rejected handshake fails authentication and keeps only a grammar-valid category', () => {
  // Arrange
  const value = events();
  value[1] = { ...value[1], event: 'noise_probe_rejected', category: 'noise_completion_Certificate', result: undefined };
  // Act
  const checks = judgeFullCheckpoint(value, source);
  const observations = checkpointObservations(value);
  // Assert
  assert.equal(status(checks, 'noise_authenticated_round_trip'), 'failed');
  assert.equal(observations.probe_event, 'rejected');
  assert.equal(observations.maybe_rejection_category, 'noise_completion_Certificate');
});

test('one phase below the unchanged margin fails the stack check', () => {
  // Arrange
  const value = events(); value[1].checkpoints[9].stack_low_water = 2047;
  // Act
  const checks = judgeFullCheckpoint(value, source);
  // Assert
  assert.equal(status(checks, 'noise_stack_margin'), 'failed');
  assert.equal(status(checks, 'noise_phase_integrity'), 'passed');
});

test('a heap integrity failure or a missing phase fails integrity', () => {
  // Arrange
  const corrupted = events(); corrupted[1].checkpoints[11].integrity = false;
  const truncated = events(); truncated[1].checkpoints.pop();
  // Act
  const corruptedChecks = judgeFullCheckpoint(corrupted, source);
  const truncatedChecks = judgeFullCheckpoint(truncated, source);
  // Assert
  assert.equal(status(corruptedChecks, 'noise_phase_integrity'), 'failed');
  assert.equal(status(truncatedChecks, 'noise_phase_integrity'), 'failed');
});

test('unprojected or truncated application records stop collection', () => {
  // Arrange / Act / Assert
  assert.throws(() => checkpointEvents('ordinary startup text\n'), /checkpoint_projection_invalid/);
  assert.throws(() => checkpointEvents('VIRTUAL_U205 {\n'), /checkpoint_application_record_invalid/);
});

test('a one-use contract claim cannot be replayed', async () => {
  // Arrange
  const root = await realpath(await mkdtemp(join(tmpdir(), 'full-checkpoint-claim-')));
  const claim = join(root, 'private', 'claim.json');
  try {
    await claimCheckpoint(claim, { elf_sha256: source });
    // Act / Assert
    await assert.rejects(claimCheckpoint(claim, { elf_sha256: source }), /noise_checkpoint_claim_consumed/);
  } finally { await rm(root, { recursive: true }); }
});

test('unknown failures are redacted to a fixed category', () => {
  // Arrange / Act / Assert
  assert.equal(redactedCheckpointFailure(Error('/private/path leaked')), 'noise_checkpoint_operation_failed');
  assert.equal(redactedCheckpointFailure(Error('noise_checkpoint_claim_consumed')), 'noise_checkpoint_claim_consumed');
});

test('the disabled effect seam rejects before any source or process access', async () => {
  // Arrange / Act / Assert
  await assert.rejects(runNoiseCheckpoint('/missing', '/missing', '/missing', { auditPath: '/missing', disableEffects: true }), /effect_gate_disabled/);
});

test('active runner uses projected records and has no debugger, panic text or core inspection', async () => {
  // Arrange / Act
  const code = await readFile(new URL('./noise-checkpoint.mjs', import.meta.url), 'utf8');
  // Assert
  assert.match(code, /outputLinePrefix: 'VIRTUAL_U205 '/);
  assert.match(code, /independent_task_bounds: 'unsupported'/);
  assert.doesNotMatch(code, /from '\.\/noise\.mjs'|core-dump\/main|decodeCore|NoiseDebugger|Guru Meditation|0xf12000|first_target_fault/);
});

function composedEvents() {
  return [{ event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: source, heartbeat_cutoff_ms: 2800, boot: 1 },
    { event: 'task', joined: true, stack_bytes: 8192, minimum_stack_free_bytes: 7000, core: 0 },
    { event: 'status', boot: 1, internal_free: 300000, internal_largest: 100000, psram_free: 8000000 },
    { event: 'allocation', released: true, bytes: 8192, before: 300000, during: 291800, after: 300000 },
    { event: 'scenario', result: { scenario: 'healthy-lifecycle', seed: 1, hardware_qualified: false, actual_outcome: 'started',
      checks: [{ id: 'controller_start', status: 'passed' }, { id: 'strict_live_profile_share', status: 'unsupported' }] } },
    { event: 'scenario_margin', minimum_main_stack_free_bytes: 6000, required_margin_bytes: 2048, configured_main_stack_bytes: 16384,
      handshake_minimum_stack_free_bytes: 4000, handshake_configured_stack_bytes: 16384, heap_integrity: true }];
}
const accepted = check => ['passed', 'expected_unsupported'].includes(check.status);

test('a healthy composed run accepts only the expected coverage gap', () => {
  // Arrange / Act
  const checks = judgeComposedCheckpoint(composedEvents(), source);
  // Assert
  assert.equal(checks.every(accepted), true);
  assert.equal(status(checks, 'scenario:strict_live_profile_share'), 'expected_unsupported');
});

test('any other unsupported composed check is a failure', () => {
  // Arrange
  const value = composedEvents(); value[4].result.checks[0].status = 'unsupported';
  // Act
  const checks = judgeComposedCheckpoint(value, source);
  // Assert
  assert.equal(status(checks, 'scenario:controller_start'), 'failed');
});

test('a thin helper stack or damaged heap fails the composed run', () => {
  // Arrange
  const thin = composedEvents(); thin[5].handshake_minimum_stack_free_bytes = 2047;
  const damaged = composedEvents(); damaged[5].heap_integrity = false;
  // Act
  const thinChecks = judgeComposedCheckpoint(thin, source);
  const damagedChecks = judgeComposedCheckpoint(damaged, source);
  // Assert
  assert.equal(status(thinChecks, 'handshake_stack_margin'), 'failed');
  assert.equal(status(damagedChecks, 'heap_integrity_after_scenario'), 'failed');
});

test('a missing scenario record stops inference', () => {
  // Arrange
  const value = composedEvents().slice(0, 4);
  // Act
  const checks = judgeComposedCheckpoint(value, source);
  // Assert
  assert.equal(status(checks, 'scenario_started'), 'failed');
  assert.equal(status(checks, 'handshake_stack_margin'), 'failed');
});

test('an unknown profile is rejected only after the effect gate', async () => {
  // Arrange / Act / Assert
  await assert.rejects(runNoiseCheckpoint('/missing', '/missing', '/missing', { auditPath: '/missing', profile: 'other', disableEffects: true }), /effect_gate_disabled/);
});

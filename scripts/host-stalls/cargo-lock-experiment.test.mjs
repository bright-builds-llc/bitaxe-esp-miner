import assert from 'node:assert/strict';
import test from 'node:test';
import { classifyCargoLockTrial } from './cargo-lock-experiment.mjs';

const shared = { mode: 'shared', commandsSucceeded: true, cleanupComplete: true, statusRecompiled: false, holderEnteredMs: 10000, holderExitedMs: 15000, statusRequestedMs: 10020, statusEnteredMs: 15030, lockMessageObserved: true };

test('actual lock message plus post-holder entry proves shared contention', () => {
  assert.equal(classifyCargoLockTrial(shared), 'shared_lock_observed');
});

test('isolated status progresses while holder is still executing', () => {
  // Arrange
  const trial = { ...shared, mode: 'isolated', statusEnteredMs: 10100, lockMessageObserved: false };
  // Act
  const result = classifyCargoLockTrial(trial);
  // Assert
  assert.equal(result, 'isolated_progress_observed');
});

for (const [reason, changes] of [
  ['command_failure', { commandsSucceeded: false }],
  ['cleanup', { cleanupComplete: false }],
  ['status_recompiled', { statusRecompiled: true }],
  ['missing_marker', { holderEnteredMs: null }],
  ['holder_duration', { holderExitedMs: 17000 }],
  ['late_request', { statusRequestedMs: 10800 }],
  ['lock_not_observed', { lockMessageObserved: false }],
  ['status_startup', { statusEnteredMs: 17000 }],
  ['status_startup', { statusEnteredMs: 14999 }],
  ['unknown_mode', { mode: 'other' }],
  ['unexpected_lock', { mode: 'isolated' }],
  ['status_startup', { mode: 'isolated', lockMessageObserved: false, statusEnteredMs: 13000 }],
  ['status_startup', { mode: 'isolated', lockMessageObserved: false, statusEnteredMs: 10000 }],
]) {
  test(`classifies ${reason} as confounded: ${JSON.stringify(changes)}`, () => {
    assert.equal(classifyCargoLockTrial({ ...shared, ...changes }), `confounded_${reason}`);
  });
}

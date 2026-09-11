import assert from 'node:assert/strict';
import test from 'node:test';
import { classifyCargoJobsTrial } from './cargo-jobs-experiment.mjs';

const serial = { jobs: 1, commandSucceeded: true, cleanupComplete: true, intervals: [{ enteredMs: 1000, exitedMs: 1500 }, { enteredMs: 1550, exitedMs: 2050 }] };
const parallel = { ...serial, jobs: 2, intervals: [{ enteredMs: 1000, exitedMs: 1500 }, { enteredMs: 1020, exitedMs: 1520 }] };

test('one job executes independent build scripts serially', () => {
  assert.equal(classifyCargoJobsTrial(serial), 'serial_scheduling_observed');
});

test('two jobs execute independent build scripts concurrently', () => {
  assert.equal(classifyCargoJobsTrial(parallel), 'parallel_scheduling_observed');
});

for (const [reason, changes] of [
  ['command_failure', { commandSucceeded: false }],
  ['cleanup', { cleanupComplete: false }],
  ['missing_marker', { intervals: [] }],
  ['missing_marker', { intervals: [{ enteredMs: null, exitedMs: 1500 }, serial.intervals[1]] }],
  ['build_script_duration', { intervals: [{ enteredMs: 1000, exitedMs: 4000 }, serial.intervals[1]] }],
  ['unexpected_overlap', { intervals: parallel.intervals }],
  ['parallel_overlap_missing', { jobs: 2 }],
  ['unknown_job_count', { jobs: 4 }],
]) {
  test(`invalid comparison remains confounded: ${reason}`, () => {
    assert.equal(classifyCargoJobsTrial({ ...serial, ...changes }), `confounded_${reason}`);
  });
}

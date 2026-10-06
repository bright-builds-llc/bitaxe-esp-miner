import test from 'node:test';
import assert from 'node:assert/strict';
import { deviceRecordAttemptId } from './start-record.mjs';
import { WORKER_REJECTIONS, recoveryErrorRow } from './recovery-error-row.mjs';
import { validateRecoveryErrors } from './recovery-errors.mjs';
import { CONTROL_REJECTIONS } from '../str005-v2-serial/safety-diagnostics.mjs';

const record = id => ({ attemptId: id, scope: 'share' });
const evidence = (issued, dispatched = issued, retained = issued) => ({ before: { attempt: { id: issued } },
  run: { dispatchStatus: { record: record(dispatched) } }, recoveryStatus: { state: 'terminal', record: record(retained) } });

test('the device record attempt is the issued attempt that dispatch and recovery both reported', () => {
  // Arrange / Act / Assert
  assert.equal(deviceRecordAttemptId(evidence('device-attempt')), 'device-attempt');
});

test('disagreeing dispatch or recovery records are refused', () => {
  // Arrange / Act / Assert
  assert.throws(() => deviceRecordAttemptId(evidence('device-attempt', 'other')), /start_record_attempt_mismatch/u);
  assert.throws(() => deviceRecordAttemptId(evidence('device-attempt', 'device-attempt', 'other')), /start_record_attempt_mismatch/u);
});

test('a share run binds by its proven generation when it keeps no dispatch record', () => {
  // Arrange
  const share = generation => ({ before: { attempt: { id: 'device-attempt' } }, run: { proof: { generation } },
    recoveryStatus: { state: 'terminal', record: { ...record('device-attempt'), workerGeneration: 3 } } });
  // Act / Assert
  assert.equal(deviceRecordAttemptId(share(3)), 'device-attempt');
  assert.throws(() => deviceRecordAttemptId(share(4)), /start_record_attempt_mismatch/u);
  assert.throws(() => deviceRecordAttemptId({ ...share(3), run: {} }), /start_record_attempt_mismatch/u);
});

test('a share run without a share binds through its in-run status read of the same record', () => {
  // Arrange
  const status = (id, generation) => ({ state: 'terminal', record: { ...record(id), workerGeneration: generation } });
  const unshared = (earlier, observedStart = true) => ({ before: { attempt: { id: 'device-attempt' } }, run: { observedStart, proof: null },
    recoveryStatus: status('device-attempt', 3), maybeEarlierStatus: earlier });
  // Act / Assert
  assert.equal(deviceRecordAttemptId(unshared(status('device-attempt', 3))), 'device-attempt');
  assert.throws(() => deviceRecordAttemptId(unshared(status('device-attempt', 4))), /start_record_attempt_mismatch/u);
  assert.throws(() => deviceRecordAttemptId(unshared(status('other', 3))), /start_record_attempt_mismatch/u);
  assert.throws(() => deviceRecordAttemptId(unshared(status('device-attempt', 3), false)), /start_record_attempt_mismatch/u);
  assert.throws(() => deviceRecordAttemptId(unshared(null)), /start_record_attempt_mismatch/u);
});

test('a Start without a retained terminal record is refused', () => {
  // Arrange
  const idle = { ...evidence('device-attempt'), recoveryStatus: { state: 'idle', record: null } };
  // Act / Assert
  assert.throws(() => deviceRecordAttemptId(idle), /start_record_attempt_mismatch/u);
});

test('the browser-safe rejection list matches the Worker control vocabulary', () => {
  // Arrange / Act / Assert
  assert.deepEqual([...WORKER_REJECTIONS], [...CONTROL_REJECTIONS]);
});

test('error rows keep a closed Worker rejection and drop anything else', () => {
  // Arrange
  const rejected = Object.assign(Error('private'), { category: 'command_rejected', rejection: 'invalid_transition' });
  const unknown = Object.assign(Error('private'), { category: 'command_rejected', rejection: 'device payload' });
  // Act
  const rows = [recoveryErrorRow('status', rejected), recoveryErrorRow('status', unknown)];
  // Assert
  assert.deepEqual(rows, [{ phase: 'status', category: 'command_rejected', rejection: 'invalid_transition' },
    { phase: 'status', category: 'command_rejected', rejection: null }]);
});

test('v2 error files accept closed rejections and v1 files stay valid', () => {
  // Arrange
  const row = { phase: 'status', category: 'command_rejected', rejection: 'invalid_transition' };
  const v1 = { schema: 'str005-recovery-errors-v1', firstFailure: { phase: 'status', category: 'timeout' }, errors: [{ phase: 'status', category: 'timeout' }] };
  // Act / Assert
  assert.doesNotThrow(() => validateRecoveryErrors({ schema: 'str005-recovery-errors-v2', firstFailure: row, errors: [row] }));
  assert.doesNotThrow(() => validateRecoveryErrors(v1));
  const invented = { ...row, rejection: 'made_up' };
  assert.throws(() => validateRecoveryErrors({ schema: 'str005-recovery-errors-v2', firstFailure: invented, errors: [invented] }),
    /startup_recovery_errors/u);
});

import assert from 'node:assert/strict';
import test from 'node:test';
import { sha256 } from '../str005-v2-serial/values.mjs';
import { validateSelfTest, validateSelfTestSummary, validateCompleteEvidence } from './self-test-evidence.mjs';
const request = { requestNonce: Buffer.alloc(16, 1).toString('base64url'), expectedBootOrdinal: 5 };
const identity = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64) };
function complete() {
  return { summary: { schema: 'worker-qualification-core-dump-self-test-observation-v1', stage: 'complete', ackMatched: true,
    expectedBootOrdinal: 5, nextBootOrdinal: 6, bootObserved: true, runtimeReadyObserved: true, softwareResetObserved: false,
    panicResetObserved: true, identityObserved: true, identityMatched: true, records: 8, bytes: 1500, durationMs: 600,
    portReopens: 0, streamInterrupted: false, continuity: 'uninterrupted' },
  ack: { schema: 'worker-qualification-core-dump-self-test-v1', requestNonceSha256: sha256(request.requestNonce), bootOrdinal: 5, nextBootOrdinal: 6 },
  lifecycle: [{ event: 'prearmed', record: 0, atMs: 0 }, { event: 'acknowledged', record: 1, atMs: 10 },
    { event: 'hello_started', record: 6, atMs: 500 }, { event: 'complete', record: 8, atMs: 600 }],
  observations: [
    { record: 2, atMs: 100, diagnostic: { category: 'boot', boot_ordinal: 6, reset_reason: 'panic', uptime_ms: 1 } },
    { record: 3, atMs: 110, diagnostic: { category: 'runtime_identity', ...identity } },
    { record: 4, atMs: 200, diagnostic: { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 100 } },
    { record: 5, atMs: 300, diagnostic: { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 200 } },
  ] };
}
test('complete self-test requires every positive observation flag', () => {
  for (const field of ['ackMatched', 'bootObserved', 'runtimeReadyObserved', 'identityObserved', 'identityMatched', 'panicResetObserved']) {
    const value = complete(); value.summary[field] = false;
    assert.throws(() => validateSelfTestSummary(value.summary, request));
  }
});
test('complete self-test requires matching ACK and fresh correlated panic, identity and two ready observations', () => {
  assert.doesNotThrow(() => validateCompleteEvidence(complete(), request, identity));
  for (const mutate of [v => { v.ack = null; }, v => { v.ack.requestNonceSha256 = '0'.repeat(64); },
    v => { v.observations[0].diagnostic.reset_reason = 'software_cpu'; }, v => { v.observations[0].diagnostic.boot_ordinal = 7; },
    v => { v.observations[1].diagnostic.app_elf_sha256 = '0'.repeat(64); }, v => { v.observations.pop(); },
    v => { v.observations[3].diagnostic.uptime_ms = 100; }, v => { v.lifecycle[2].record = 1; },
    v => { v.observations[2].diagnostic.first_failure = 'failed'; }, v => { v.observations[1].record = 1; }]) {
    const value = complete(); mutate(value); assert.throws(() => validateCompleteEvidence(value, request, identity));
  }
});
test('failed partial self-test with no ACK or observations remains inspectable without promotion', async () => {
  const value = complete(); value.summary.stage = 'failed';
  for (const field of ['ackMatched', 'bootObserved', 'runtimeReadyObserved', 'identityObserved', 'identityMatched', 'panicResetObserved']) value.summary[field] = false;
  value.ack = null; value.observations = []; value.summary.records = 513; value.summary.bytes = 262145; value.summary.durationMs = 40000;
  value.lifecycle = [{ event: 'failed', record: 513, atMs: 40000 }];
  const inspected = await validateSelfTest(value, '/unused', request, identity);
  assert.equal(inspected.summary.stage, 'failed'); assert.equal(inspected.ack, null); assert.deepEqual(inspected.observations, []);
});

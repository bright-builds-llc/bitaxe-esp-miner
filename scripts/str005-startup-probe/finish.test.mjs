import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, rm, chmod } from 'node:fs/promises';
import { resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { finalize } from './finish.mjs';
import { writeNew } from '../str005-noise-serial/files.mjs';
test('failure before fixture or authority launch still seals a truthful partial result', async t => {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'startup-finish-test-'))); await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeNew(resolve(root, 'run.json'), { firstFailure: 'prepare', observedStart: false, startInvokedAt: null, startRepliedAt: null,
    stopRequestedAt: 10, proof: null });
  const result = await finalize(root, { detector: { port: '/dev/test-only' } }, { requireNoHolders() {} });
  assert.equal(result.complete, false); assert.equal(result.first_failure, 'prepare'); assert.equal(result.sealed, true);
  assert.ok(result.blockers.includes('startup_not_proven'));
});

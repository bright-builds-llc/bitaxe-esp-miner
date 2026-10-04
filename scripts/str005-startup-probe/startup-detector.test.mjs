import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { readStartupDetector } from './startup-detector.mjs';
import { failureRecord } from './failure.mjs';

test('a missing startup detector is a named failure the owner records', async t => {
  // Arrange
  const directory = await mkdtemp(resolve(tmpdir(), 'startup-detector-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  // Act
  const error = await readStartupDetector(resolve(directory, 'startup-detector.stdout.log'), 'physical').catch(value => value);
  // Assert
  assert.equal(error.code, 'startup_detector_missing');
  assert.equal(failureRecord('/cooling-review', error).category, 'startup_detector_missing');
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { judgeSeries, parseSamples } from './series.mjs';
import { argumentsFor } from './main.mjs';

const line = (uptime, free, largest, blocks = 1000) =>
  `internal_heap_sample schema=v1 uptime_ms=${uptime} free_bytes=${free} allocated_bytes=400000 ` +
  `largest_block_bytes=${largest} minimum_free_bytes=1295 allocated_blocks=${blocks} free_blocks=40 revoked=false redacted=true`;
const thresholds = { minFreeBytes: 16384, minLargestBlockBytes: 8192, minSamples: 2 };

test('only well-formed sample lines are read from a mixed serial stream', () => {
  // Arrange
  const text = ['usb_startup schema=v1 stage=runtime_ready', line(60000, 9000, 1792), 'internal_heap_sample schema=v1 free_bytes=x redacted=true',
    `${line(120000, 9100, 1792)}\r`].join('\n');
  // Act
  const samples = parseSamples(text);
  // Assert
  assert.deepEqual(samples.map(sample => sample.uptime_ms), [60000, 120000]);
  assert.equal(samples[0].revoked, false);
});

test('a series passes only when every sample keeps the free and largest-block minimums', () => {
  // Arrange
  const healthy = parseSamples([line(0, 40000, 20000), line(60000, 30000, 12000)].join('\n'));
  // Act
  const result = judgeSeries(healthy, thresholds);
  // Assert
  assert.equal(result.passed, true);
  assert.deepEqual([result.least_free_bytes, result.least_largest_block_bytes, result.span_minutes], [30000, 12000, 1]);
});

test('one exhausted dip fails the series', () => {
  // Arrange: the candidate's measured idle state before the fix.
  const exhausted = parseSamples([line(0, 11555, 1792), line(60000, 2631, 1728, 1233)].join('\n'));
  // Act
  const result = judgeSeries(exhausted, thresholds);
  // Assert
  assert.deepEqual(result.failures, ['free_below_minimum', 'largest_block_below_minimum']);
  assert.equal(result.allocated_blocks_delta, 233);
});

test('too short a capture cannot pass', () => {
  // Arrange / Act
  const result = judgeSeries(parseSamples(line(0, 40000, 20000)), thresholds);
  // Assert
  assert.deepEqual(result.failures, ['too_few_samples']);
});

test('all three thresholds and at least one capture are required', () => {
  // Arrange / Act / Assert
  assert.throws(() => argumentsFor(['--min-free-bytes', '1', '--min-samples', '2', 'c.log']), /internal_heap_series_arguments/u);
  assert.throws(() => argumentsFor(['--min-free-bytes', '1', '--min-largest-block-bytes', '2', '--min-samples', '3']), /internal_heap_series_arguments/u);
  assert.deepEqual(argumentsFor(['--min-free-bytes', '1', '--min-largest-block-bytes', '2', '--min-samples', '3', 'a', 'b']).captures, ['a', 'b']);
});

// Pure parsing and judgement of the firmware's idle `internal_heap_sample` lines.

const FIELDS = ['uptime_ms', 'free_bytes', 'allocated_bytes', 'largest_block_bytes', 'minimum_free_bytes',
  'allocated_blocks', 'free_blocks'];

/** Parses every well-formed sample line; anything else on the serial stream is ignored. */
export function parseSamples(text) {
  const samples = [];
  for (const line of text.split(/\r?\n/u)) {
    const fields = line.trimEnd().split(' ');
    if (fields[0] !== 'internal_heap_sample' || fields[1] !== 'schema=v1' || fields.at(-1) !== 'redacted=true') continue;
    const values = Object.fromEntries(fields.slice(2, -1).map(pair => pair.split('=')));
    if (!FIELDS.every(key => /^\d+$/u.test(values[key] ?? '')) || !['true', 'false'].includes(values.revoked)) continue;
    samples.push({ ...Object.fromEntries(FIELDS.map(key => [key, Number(values[key])])), revoked: values.revoked === 'true' });
  }
  return samples;
}

/** Summarises a series and judges it against the minimum free bytes and largest free block every sample must keep. */
export function judgeSeries(samples, { minFreeBytes, minLargestBlockBytes, minSamples }) {
  const least = key => Math.min(...samples.map(sample => sample[key]));
  const summary = samples.length === 0 ? { samples: 0 } : {
    samples: samples.length,
    span_minutes: Math.round((samples.at(-1).uptime_ms - samples[0].uptime_ms) / 600) / 100,
    least_free_bytes: least('free_bytes'),
    least_largest_block_bytes: least('largest_block_bytes'),
    lifetime_minimum_free_bytes: samples.at(-1).minimum_free_bytes,
    allocated_blocks_delta: samples.at(-1).allocated_blocks - samples[0].allocated_blocks,
  };
  const failures = [];
  if (samples.length < minSamples) failures.push('too_few_samples');
  if (samples.length > 0 && summary.least_free_bytes < minFreeBytes) failures.push('free_below_minimum');
  if (samples.length > 0 && summary.least_largest_block_bytes < minLargestBlockBytes) failures.push('largest_block_below_minimum');
  return { ...summary, passed: failures.length === 0, failures };
}

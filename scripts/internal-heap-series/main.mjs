// Usage: internal-heap-series --min-free-bytes <n> --min-largest-block-bytes <n> --min-samples <n> <capture>...
// Prints only numeric categories, so the output is safe to promote; the captures themselves stay private.
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { judgeSeries, parseSamples } from './series.mjs';

const OPTIONS = { '--min-free-bytes': 'minFreeBytes', '--min-largest-block-bytes': 'minLargestBlockBytes', '--min-samples': 'minSamples' };

export function argumentsFor(argv) {
  const thresholds = {}, captures = [];
  for (let index = 0; index < argv.length; index++) {
    const key = OPTIONS[argv[index]];
    if (!key) { captures.push(argv[index]); continue; }
    const value = argv[++index];
    if (!/^\d+$/u.test(value ?? '') || key in thresholds) throw Error('internal_heap_series_arguments');
    thresholds[key] = Number(value);
  }
  if (Object.keys(thresholds).length !== 3 || captures.length === 0) throw Error('internal_heap_series_arguments');
  return { thresholds, captures };
}

export async function main(argv) {
  const { thresholds, captures } = argumentsFor(argv);
  const base = process.env.BUILD_WORKING_DIRECTORY ?? process.cwd();
  const texts = await Promise.all(captures.map(path => readFile(resolve(base, path), 'latin1')));
  return judgeSeries(texts.flatMap(parseSamples), thresholds);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => {
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exitCode = result.passed ? 0 : 1;
  }).catch(error => {
    process.stdout.write(`${JSON.stringify({ passed: false, error: error.message })}\n`);
    process.exitCode = 2;
  });
}

#!/usr/bin/env node
// Gate: no default (production) firmware ELF contains a test variant's boot marker (ADR-0040,
// ADR-0041). Under `bazel test //...` the `//firmware/bitaxe:firmware_variant` flag is
// `production`, so every ELF passed here is a production build: the Ultra 205's and the
// Gamma 601's.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// The boot markers, and the core-voltage check's record, which only its variant compiles.
export const TEST_VARIANT_MARKERS = [
  'firmware_variant=overheat-trip-test',
  'firmware_variant=gamma601-vout-check',
  'usb_gamma601_vout_check',
];

export function main(elves) {
  if (elves.length === 0) throw Error('firmware_variant_isolation_arguments');
  const contaminated = elves.filter((elf) => {
    const bytes = readFileSync(elf);
    if (bytes.length === 0) throw Error('firmware_variant_isolation_empty_elf');
    return TEST_VARIANT_MARKERS.some((marker) => bytes.includes(marker));
  });
  const result = contaminated.length === 0 ? 'production_elf_isolated' : 'test_marker_in_production_elf';
  return { result, elves: elves.length, contaminated: contaminated.length };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = main(process.argv.slice(2));
  console.log(JSON.stringify(result));
  if (result.result !== 'production_elf_isolated') process.exitCode = 1;
}

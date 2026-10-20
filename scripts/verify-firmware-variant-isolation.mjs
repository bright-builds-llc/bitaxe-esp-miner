#!/usr/bin/env node
// Gate: the default (production) firmware ELF must not contain the overheat-trip-test boot marker
// (ADR-0040). Under `bazel test //...` the `//firmware/bitaxe:firmware_variant` flag is `production`.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const OVERHEAT_TRIP_TEST_MARKER = 'firmware_variant=overheat-trip-test';

export function main([elf, ...rest]) {
  if (!elf || rest.length) throw Error('firmware_variant_isolation_arguments');
  const bytes = readFileSync(elf);
  if (bytes.length === 0) throw Error('firmware_variant_isolation_empty_elf');
  const result = bytes.includes(OVERHEAT_TRIP_TEST_MARKER) ? 'test_marker_in_production_elf' : 'production_elf_isolated';
  return { result, elf_bytes: bytes.length };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = main(process.argv.slice(2));
  console.log(JSON.stringify(result));
  if (result.result !== 'production_elf_isolated') process.exitCode = 1;
}

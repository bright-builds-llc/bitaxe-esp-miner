#!/usr/bin/env node
// Gate: the built firmware ELF must have no hardware compare-exchange outside the allowed native
// functions (ADR-0039). It runs in `bazel test //...` beside the stack audits, with the same objdump.
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { auditPsramAtomics } from './psram-atomics-audit.mjs';
import { managedObjdump } from './verify-firmware-stack-audits.mjs';
import { boundToFunctions, functionSizes } from './virtual-emulator/noise-stack-audit.mjs';

/**
 * The ELF's disassembly, not demangled, with each body bounded by its symbol size so a literal pool
 * after a function never decodes as its code.
 */
export function boundedDisassembly(objdump, elf) {
  const options = { encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, timeout: 300000 };
  return boundToFunctions(execFileSync(objdump, ['-d', elf], options), functionSizes(execFileSync(objdump, ['-t', elf], options)));
}

export function main([elf, ...rest]) {
  if (!elf || rest.length) throw Error('psram_atomics_gate_arguments');
  return auditPsramAtomics(boundedDisassembly(managedObjdump(), elf));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = main(process.argv.slice(2));
  console.log(JSON.stringify(result));
  if (result.result !== 'no_unexpected_hardware_atomics') process.exitCode = 1;
}

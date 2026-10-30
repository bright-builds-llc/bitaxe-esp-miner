// Usage: access-gate-check --monitor-log <live capture>
// Reads the device origin from the capture and sends three read-only requests to it.
// It adds no authority: device traffic needs the active task contract.
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { requestStatus, runCheck } from './check.mjs';

export function argumentsFor(argv) {
  if (argv.length !== 2 || argv[0] !== '--monitor-log' || !argv[1]) throw Error('access_gate_check_arguments');
  return { monitorLog: argv[1] };
}

export async function main(argv) {
  const { monitorLog } = argumentsFor(argv);
  const path = resolve(process.env.BUILD_WORKING_DIRECTORY ?? process.cwd(), monitorLog);
  return runCheck({ readLog: () => readFile(path, 'latin1'), requestStatus });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => {
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exitCode = result.passed ? 0 : 1;
  }).catch(error => {
    // Only fixed codes: a network error message would carry the device address.
    const code = error.message.startsWith('access_gate_check_') ? error.message : 'access_gate_check_request_failed';
    process.stdout.write(`${JSON.stringify({ passed: false, error: code })}\n`);
    process.exitCode = 2;
  });
}

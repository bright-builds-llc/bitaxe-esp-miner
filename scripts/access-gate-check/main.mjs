// Usage: access-gate-check --monitor-log <live capture> --expect-peer private|zero
// Reads the device origin from the capture, sends three read-only requests, and judges the denied line.
// It adds no authority: device traffic needs the active task contract.
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import { requestStatus, runCheck } from './check.mjs';

const EXPECTED_PEERS = new Set(['private', 'zero']);

export function argumentsFor(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 2) {
    const value = argv[index + 1];
    if (argv[index] === '--monitor-log') parsed.monitorLog = value;
    else if (argv[index] === '--expect-peer') parsed.expectPeer = value;
    else throw Error('access_gate_check_arguments');
  }
  if (!parsed.monitorLog || !EXPECTED_PEERS.has(parsed.expectPeer)) throw Error('access_gate_check_arguments');
  return parsed;
}

export async function main(argv) {
  const { monitorLog, expectPeer } = argumentsFor(argv);
  const path = resolve(process.env.BUILD_WORKING_DIRECTORY ?? process.cwd(), monitorLog);
  return runCheck({ readLog: () => readFile(path, 'latin1'), requestStatus, sleep, expectPeer });
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

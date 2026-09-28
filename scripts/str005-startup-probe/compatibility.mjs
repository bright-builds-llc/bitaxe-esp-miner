import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { git, cleanPushed } from '../fixed-usb-qualification/contract.mjs';
import { check, object, sha256 } from '../str005-v2-serial/values.mjs';
export const COMPATIBILITY = 'docs/hardware/str005-normal-stop-compatibility.json';
export const UNCHANGED_GATE_PATHS = [
  'conformance/bwg-worker-controller-0.4/fixtures.json', 'conformance/bwg-worker-possession-0.2/fixtures.json', 'docs/protocol/bwg-worker-serial-0.2.md',
  'web/worker-serial-controller-runtime.ts', 'web/worker-v2-serial-control.ts', 'web/worker-serial.ts',
  'Cargo.toml', 'Cargo.lock', 'package.json',
];
export function validateCompatibility(report, historical, currentGate) {
  object(report, ['schema', 'firmwareCommit', 'elfSha256', 'historicalGateCommit', 'currentGateCommit', 'gateDiffSha256', 'changedGatePaths']);
  check(report.schema === 'str005-normal-stop-compatibility-v1' &&
    report.firmwareCommit === historical.firmware_commit && report.elfSha256 === historical.app_elf_sha256 &&
    report.historicalGateCommit === historical.gate_commit && report.currentGateCommit === currentGate &&
    /^[a-f0-9]{40}$/u.test(currentGate) && currentGate !== report.historicalGateCommit &&
    /^[a-f0-9]{64}$/u.test(report.gateDiffSha256) && Array.isArray(report.changedGatePaths) && report.changedGatePaths.length > 0 &&
    report.changedGatePaths.every(path => typeof path === 'string' && /^[A-Za-z0-9_./-]+$/u.test(path)) &&
    new Set(report.changedGatePaths).size === report.changedGatePaths.length, 'startup_gate_compatibility');
  return report;
}
/** The reviewed current Gate is distinct from immutable historical capture identity. */
export async function reviewedGate(firmwareRoot, gateRoot, historical) {
  const currentGate = git(gateRoot, ['rev-parse', 'HEAD']); cleanPushed(gateRoot, currentGate);
  const report = validateCompatibility(JSON.parse(await readFile(resolve(firmwareRoot, COMPATIBILITY), 'utf8')), historical, currentGate);
  const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === currentGate, 'startup_gate_pin');
  const revisions = [report.historicalGateCommit, currentGate];
  const changed = git(gateRoot, ['diff', '--no-ext-diff', '--name-only', ...revisions]).split('\n').filter(Boolean).sort();
  const diff = git(gateRoot, ['diff', '--no-ext-diff', '--no-textconv', '--binary', ...revisions]);
  check(JSON.stringify(changed) === JSON.stringify([...report.changedGatePaths].sort()) && sha256(diff) === report.gateDiffSha256,
    'startup_gate_review_changed');
  for (const path of UNCHANGED_GATE_PATHS)
    check(git(gateRoot, ['rev-parse', `${revisions[0]}:${path}`]) === git(gateRoot, ['rev-parse', `${revisions[1]}:${path}`]), 'startup_gate_protocol_changed');
  return { gate_commit: currentGate, historical_gate_commit: historical.gate_commit, compatibilitySha256: sha256(JSON.stringify(report)) };
}

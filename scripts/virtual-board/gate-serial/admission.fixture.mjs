// Runs the pinned Gate serial controller (real code, Bun) against a virtual device child.
import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { childPort, releaseDevice, startDevice, DEVICE_FILTER } from './child-port.mjs';

const options = Object.fromEntries(process.argv.slice(2).map(argument => {
  const match = /^--([a-z-]+)=(.*)$/u.exec(argument);
  if (!match) throw Error('fixture_arguments');
  return [match[1], match[2]];
}));
const load = name => import(pathToFileURL(resolve(options['gate-root'], `web/${name}.ts`)).href);
const { workerSerialTestRuntime } = await load('webserial-worker-port');
const { createWebSerialWorkerController } = await load('webserial-worker-controller');
const { workerSerialFailureCategory } = await load('worker-serial-errors');
const trust = JSON.parse(await readFile(options.trust, 'utf8'));

const outcomePath = join(options.work, 'outcome.json');
const device = startDevice({ binary: options.device, seed: 7, capability: options.capability, outcome: outcomePath,
  stderrPath: join(options.work, 'device.stderr') });
let maybeStored;
const runtime = {
  serial: { requestPort: async () => childPort(device) },
  foreground: () => true,
  userActivation: () => true,
  now: () => performance.now(),
  acquireLock: async () => () => {},
  subscribeForegroundLoss: () => () => {},
  every(milliseconds, listener) { const timer = setInterval(listener, milliseconds); return () => clearInterval(timer); },
};
const continuity = {
  challengeBindingSha256: async () => 'A'.repeat(43),
  maybeExpectedFingerprint: async () => maybeStored,
  establish: async value => { maybeStored = value; },
  clear: async () => { maybeStored = undefined; },
};
const controller = createWebSerialWorkerController({
  deviceFilter: DEVICE_FILTER,
  trustedUpdateKeys: trust.updateAuthority.keys,
  continuityScope: { challengeId: 'challenge_virtual_board_gate_serial', retentionExpiryUnixSeconds: 2_000_000_000 },
  expectedFirmwareSourceCommit: 'a'.repeat(40),
  expectedAppElfSha256: 'b'.repeat(64),
  [workerSerialTestRuntime]: { runtime, continuity },
});

const result = { mode: options.mode, timings_ms: {} };
const started = performance.now();
const mark = phase => { result.timings_ms[phase] = Math.round(performance.now() - started); };
try {
  try {
    result.permission = (await controller.requestPermission()).status;
  } catch (error) {
    result.permission = 'rejected';
    result.category = workerSerialFailureCategory(error);
  }
  mark('permission');
  if (result.permission === 'ready' && options.mode === 'loss') {
    process.kill(-device.child.pid, 'SIGKILL');
    try { await controller.status(); result.status = 'unexpected'; } catch (error) { result.category = workerSerialFailureCategory(error); }
  } else if (result.permission === 'ready') {
    result.state = (await controller.status()).state;
  }
} finally {
  mark('commands');
  await controller.close();
  mark('close');
  result.device = await releaseDevice(device);
  mark('release');
}
try { result.outcome = JSON.parse(await readFile(outcomePath, 'utf8')); } catch (error) {
  // A killed device writes no outcome; anything else is a harness failure.
  if (error.code !== 'ENOENT') throw error;
  result.outcome = null;
}
process.stdout.write(`${JSON.stringify(result)}\n`);

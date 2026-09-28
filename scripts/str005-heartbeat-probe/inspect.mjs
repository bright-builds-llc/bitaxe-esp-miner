import { fixtureReleaseComplete } from '../str005-startup-probe/server-release.mjs';
import { readFile, stat, readdir } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { argumentsFor, STARTUP_SEAL } from './contract.mjs';
import { judgeHeartbeat } from './evidence.mjs';
import { judgeObserver } from '../str005-v2-serial/safety-judge.mjs';
import { sealed } from '../str005-startup-probe/capture.mjs';
import { proof, privateRoot } from '../str005-noise-serial/files.mjs';
import { requireGone, requireLsofAbsent, requireNoHolders, signerExitProofs } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { isDeepStrictEqual as equal } from 'node:util';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { check } from '../str005-v2-serial/values.mjs';
/** Read-only audit of actual producer files; this command exposes no effect path. */
export async function main(argv) {
  const { root } = argumentsFor(argv); await privateRoot(root);
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-heartbeat-context-v1' && context.scope === 'share', 'heartbeat_context');
  check(await sealed(context.startupRoot) === STARTUP_SEAL && (await proof(context.startupRoot, 'result.json')).value.complete === true,
    'heartbeat_startup_prerequisite');
  const owner = (await proof(root, 'server-owner.json')).value, fixture = (await proof(root, 'fixture-owner.json')).value;
  await requireGone([owner.owner, fixture.owner]);
  requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  await signerExitProofs(root, context);
  for (const file of ['fixture-exit.json', 'fixture-reap.json']) await proof(root, file);
  check(fixtureReleaseComplete((await proof(root, 'fixture-release.json')).value), 'heartbeat_fixture_release');
  const detectorPath = resolve(dirname(root), 'final-detector.stdout.log');
  const detected = parseDetector(await readFile(detectorPath, 'utf8'), context.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  for (const port of new Set([context.detector.port, detected.port])) requireNoHolders(port);
  const rounds = (await readdir(root)).filter(name => /^recovery-[1-4]-session\.json$/u.test(name)).sort();
  const round = rounds.at(-1)?.split('-')[1]; check(round, 'heartbeat_fresh_recovery_missing');
  const recovery = {}; for (const stage of ['state', 'ledger', 'original_budget', 'status', 'diagnostics', 'closed', 'finished'])
    recovery[stage] = (await proof(root, `recovery-${round}-${stage}.json`)).value;
  const parts = { before: (await proof(root, 'before.json')).value, run: (await proof(root, 'run.json')).value,
    recovery, hostReleased: true };
  const result = judgeHeartbeat(parts, context);
  const rows = (await proof(root, 'observer-state-rows.json')).value, devices = (await proof(root, 'observer-device-rows.json')).value;
  check(Array.isArray(devices) && devices.length === 1 && equal(devices[0].record, parts.run.dispatchStatus.record), 'heartbeat_observer_device_join');
  check(Array.isArray(rows) && rows.length >= 2 && rows.length <= 4, 'heartbeat_observer_states');
  for (const row of rows) validateState(row.state, context);
  const confirmed = (await proof(root, 'suppression-confirmed.json')).value;
  check(equal(confirmed.headroom, parts.run.headroom) && equal(confirmed.state, parts.run.suppressedState), 'heartbeat_suppression_join');
  check(confirmed.clientConfirmedAtMs === parts.run.suppressionConfirmedAt, 'heartbeat_observer_confirmation');
  const stop = (await proof(root, 'observer-stop.json')).value;
  check(stop.requestedAtHostMs - confirmed.confirmedAtHostMs >= 8000, 'heartbeat_observer_tail');
  const claim = (await proof(root, 'observer-start.claim.json')).value;
  const startClaim = (await proof(root, 'heartbeat-start-claim.json')).value;
  check(startClaim.contextSha256 === claim.contextSha256 && claim.atHostMs <= startClaim.atHostMs && claim.bootOrdinal === parts.run.dispatchStatus.observation.bootOrdinal &&
    claim.workerGeneration === parts.run.dispatchStatus.record.workerGeneration, 'heartbeat_observer_prearmed');
  const observer = await judgeObserver(root, context, rows, devices, confirmed);
  return { ...result, observer };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value =>
  process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ complete: false, blocker: /^heartbeat_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'heartbeat_evidence_unverified', parityPromotion: false })}\n`);
  process.exitCode = 1;
});

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { stripTypeScriptTypes } from 'node:module';
import { createContext, runInContext } from 'node:vm';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { maybeReadWindowSource, windowContext, windowFunctions, windowSourceDigest } from '../str005-startup-probe/gate-window-source.mjs';
function body(source, name) {
  const start = source.indexOf(`function ${name}(`); check(start >= 0, 'startup_gate_function_missing');
  const opening = source.indexOf('{', start); let depth = 0;
  for (let index = opening; index < source.length; index++) {
    if (source[index] === '{') depth++;
    if (source[index] === '}' && --depth === 0) return source.slice(source.slice(Math.max(0, start - 6), start) === 'async ' ? start - 6 : start, index + 1);
  }
  check(false, 'startup_gate_function_shape');
}
/** Executes actual Gate load/start/wrapper functions; parser/controller/checkpoint dependencies are test doubles. */
export async function assessGateSource(source, authorizationSource, maybeWindowSource) {
  const converted = stripTypeScriptTypes(source, { mode: 'transform' });
  const functions = ['loadWindow', 'loadSignedWindow', 'startWindow', 'tick'].map(name => body(converted, name));
  functions.push(...windowFunctions(source, maybeWindowSource));
  // Gate 26ab3ab moved the shared renewal step out of `tick`; older sources keep it inline.
  if (converted.includes('function renewNext(')) functions.push(body(converted, 'renewNext'));
  functions.push(body(stripTypeScriptTypes(authorizationSource, { mode: 'transform' }), 'runWorkerNormalAuthorization'));
  let time = 0, starts = 0, claims = 0, observations = 0, renewals = 0;
  const authorizationToken = {};
  const grant = { qualificationAttempt: { purpose: 'normal', maximumActiveMilliseconds: 180000 }, durationMilliseconds: 60000,
    renewAfterMilliseconds: 20000, leaseId: 'lease_synthetic', stratum: { profile: 'bwg-worker-stratum-v2-standard/0.1' } };
  const context = createContext({ ...windowContext, maybeConfiguration: { stratumV2Qualification: 'candidate', stratumV2Scope: 'share' }, running: false, polling: false,
    maybeWindow: undefined, maybeQualification: undefined, maybeOwnerResourceFailure: undefined, began: 0, nextRenew: 0, maybeTimer: undefined, status: '',
    parseWorkerLeaseGrant: value => value, parseWorkerLeaseRenewal: value => value, cadence: { requireWindow() {}, shouldSuppress() { return false; } },
    requireWorkerV2ShareMode: value => check(value.stratumV2Scope === 'share', 'startup_gate_scope'),
    isWorkerV2Stratum: value => value.profile === grant.stratum.profile,
    localJson: async path => { check(path === '/window-artifacts', 'startup_gate_artifacts_route'); return { grant, renewals: [grant, grant] }; },
    state: () => ({}), publish() {}, v2ShareStartClaim: { consume() { check(++claims === 1, 'startup_gate_duplicate'); } },
    controller: () => ({ async startLease() { starts++; time = 7000; return { qualification: { generation: 7 } }; } }),
    authorizationRecovery: { beginAuthorizedOperation: () => authorizationToken, completeAuthorizedOperation: token => { check(token === authorizationToken, 'startup_gate_checkpoint_token'); observations++; }, cancelAuthorizedOperation() { throw Error('startup_gate_checkpoint_cancelled'); } },
    close: async () => { throw Error('startup_gate_unexpected_close'); },
    renewalProgress: { beginWindow() {}, async renew() { renewals++; } },
    acceptanceMaximumActiveMilliseconds: () => 180000, acceptancePurposeWindow: () => 'normal', acceptanceWindowShouldStop: () => false,
    refresh: async () => {}, stop: async () => { context.running = false; }, fail: async () => { throw Error('share_gate_failure'); }, performance: { now: () => time }, enforceRunningHeadroom: async () => true,
    diagnosticInitialWorkCaptured: () => false, setInterval: () => 1, finishDiagnosticWork() {},
  });
  runInContext(functions.join('\n'), context, { timeout: 1000 });
  await context.loadSignedWindow();
  check(context.maybeWindow.renewals.length === 2, 'share_gate_two_renewals');
  await context.startWindow();
  check(starts === 1 && observations === 1 && context.began === 7000 && context.nextRenew === 27000 && context.running === true, 'startup_gate_renew_clock');
  time = 26999; await context.tick(); check(renewals === 0, 'share_gate_early_renewal');
  time = 27000; await context.tick(); check(renewals === 1, 'share_gate_first_renewal');
  time = 47000; await context.tick(); check(renewals === 2, 'share_gate_second_renewal');
  time = 52000; await context.stop(); time = 67000; await context.tick();
  check(renewals === 2 && context.running === false && observations === 3, 'share_gate_stop_bound');
  return { ...windowSourceDigest(maybeWindowSource), schema: 'str005-share-gate-compatibility-v1', sourceSha256: sha256(source), authorizationSourceSha256: sha256(authorizationSource), functions: ['loadWindow', 'loadSignedWindow', 'startWindow'],
    twoRenewalsAccepted: true, actualAutomaticRenewals: renewals, renewalOrigin: 'completed-controller-start', renewAfterMilliseconds: 20000, hardwareExercised: false,
    parserAndControllerMocked: true, privateCheckpointMocked: true };
}
export async function verifyGateCompatibility(gateRoot) {
  return assessGateSource(await readFile(resolve(gateRoot, 'web/worker-serial-acceptance.ts'), 'utf8'),
    await readFile(resolve(gateRoot, 'web/worker-normal-authorization.ts'), 'utf8'), await maybeReadWindowSource(gateRoot));
}

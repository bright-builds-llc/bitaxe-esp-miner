import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { stripTypeScriptTypes } from 'node:module';
import { createContext, runInContext } from 'node:vm';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { maybeReadWindowSource, windowContext, windowFunctions, windowSourceDigest } from './gate-window-source.mjs';
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
  const functions = ['loadWindow', 'loadSignedWindow', 'startWindow'].map(name => body(converted, name));
  functions.push(...windowFunctions(source, maybeWindowSource));
  functions.push(body(stripTypeScriptTypes(authorizationSource, { mode: 'transform' }), 'runWorkerNormalAuthorization'));
  let time = 0, starts = 0, claims = 0, observations = 0;
  const authorizationToken = {};
  const grant = { qualificationAttempt: { purpose: 'normal', maximumActiveMilliseconds: 180000 }, durationMilliseconds: 60000,
    renewAfterMilliseconds: 20000, leaseId: 'lease_synthetic', stratum: { profile: 'bwg-worker-stratum-v2-standard/0.1' } };
  const context = createContext({ ...windowContext, maybeConfiguration: { stratumV2Qualification: 'candidate', stratumV2Scope: 'share' }, running: false,
    maybeWindow: undefined, maybeQualification: undefined, maybeOwnerResourceFailure: undefined, began: 0, nextRenew: 0, maybeTimer: undefined, status: '',
    parseWorkerLeaseGrant: value => value, parseWorkerLeaseRenewal: value => value, cadence: { requireWindow() {} },
    requireWorkerV2ShareMode: value => check(value.stratumV2Scope === 'share', 'startup_gate_scope'),
    isWorkerV2Stratum: value => value.profile === grant.stratum.profile,
    localJson: async path => { check(path === '/window-artifacts', 'startup_gate_artifacts_route'); return { grant, renewals: [] }; },
    state: () => ({}), publish() {}, v2ShareStartClaim: { consume() { check(++claims === 1, 'startup_gate_duplicate'); } },
    controller: () => ({ async startLease() { starts++; time = 7000; return { qualification: { generation: 7 } }; } }),
    authorizationRecovery: { beginAuthorizedOperation: () => authorizationToken, completeAuthorizedOperation: token => { check(token === authorizationToken, 'startup_gate_checkpoint_token'); observations++; }, cancelAuthorizedOperation() { throw Error('startup_gate_checkpoint_cancelled'); } },
    close: async () => { throw Error('startup_gate_unexpected_close'); },
    renewalProgress: { beginWindow() {} }, performance: { now: () => time }, enforceRunningHeadroom: async () => true,
    diagnosticInitialWorkCaptured: () => false, setInterval: () => 1, tick() {}, finishDiagnosticWork() {},
  });
  runInContext(functions.join('\n'), context, { timeout: 1000 });
  await context.loadSignedWindow();
  check(context.maybeWindow.renewals.length === 0, 'startup_gate_zero_renewals');
  await context.startWindow();
  check(starts === 1 && observations === 1 && context.began === 7000 && context.nextRenew === 27000 && context.running === true, 'startup_gate_renew_clock');
  return { ...windowSourceDigest(maybeWindowSource), schema: 'str005-startup-gate-compatibility-v2', sourceSha256: sha256(source), authorizationSourceSha256: sha256(authorizationSource), functions: ['loadWindow', 'loadSignedWindow', 'startWindow'],
    zeroRenewalsAccepted: true, renewalOrigin: 'completed-controller-start', renewAfterMilliseconds: 20000, hardwareExercised: false,
    parserAndControllerMocked: true, privateCheckpointMocked: true };
}
export async function verifyGateCompatibility(gateRoot) {
  return assessGateSource(await readFile(resolve(gateRoot, 'web/worker-serial-acceptance.ts'), 'utf8'),
    await readFile(resolve(gateRoot, 'web/worker-normal-authorization.ts'), 'utf8'), await maybeReadWindowSource(gateRoot));
}

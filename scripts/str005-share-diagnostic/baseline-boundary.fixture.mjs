// Exact Gate hook/Stop/Close bodies and actual SDK-independent restoration helper;
// only the device response and page environment are simulated.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { once } from 'node:events';
import { createProbeServer } from '../str005-panic-probe/server.mjs';
import { createBaselineCollector } from '../str005-panic-probe/client.mjs';
import { baselineConclusion, currentProof } from '../str005-panic-probe/model.mjs';
import { ledger, original, state } from '../str005-noise-serial/test-fixture.mjs';
import { BEFORE_READ_BASELINE_POLICY } from './baseline-policy.mjs';
const gateRoot = process.argv[2], load = file => import(pathToFileURL(resolve(gateRoot, file)).href);
const { workerDeviceBaselineConfirmed } = await load('web/worker-device-baseline.ts');
const { restoreAcceptanceBaseline } = await load('web/worker-serial-acceptance-actions.ts');
const source = await readFile(resolve(gateRoot, 'web/worker-serial-acceptance.ts'), 'utf8');
const hookStart = source.indexOf('observeStatus: (value) => {'), hookEnd = source.indexOf('\n  },', hookStart);
const stopStart = source.indexOf('async function stop() {'), closeStart = source.indexOf('async function close()', stopStart);
const closeEnd = source.indexOf('\n}\n', closeStart) + 2;
assert.ok(hookStart >= 0 && hookEnd > hookStart && stopStart >= 0 && closeStart > stopStart && closeEnd > closeStart);
const hook = source.slice(hookStart, hookEnd).replace('observeStatus:', 'const observeStatus =') + '};';
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) }, scope: 'share',
  diagnosticSuccessor: true, renewSuccessor: true, recoveryOnly: true, selfTestEnabled: false, ...BEFORE_READ_BASELINE_POLICY };
const initial = state(context, 'before');
const page = new Function('workerDeviceBaselineConfirmed', 'restoreAcceptanceBaseline', 'initial', `
 let status='ready',connected=true,running=false,serialOwnershipReleased=false,deviceBaselineConfirmed=false,deviceLeaseInactive=false,deviceRestorationConfirmed=false;
 let maybeQualification,maybeWindow,maybeReviewedContext,maybeFailure,maybeConfiguration;
 const authorizationRecovery={observeStatus(){},captureNormalStop(){}},cadence={observe(){}},stopTimer=()=>{},publish=()=>{};
 ${hook}
 const baseline = restoration => ({state:'baseline',restoration:{status:restoration}});
 const actualController={restore:async reason=>{if(reason!=='cancelled')throw Error('reason');const response=baseline('confirmed');observeStatus(response);return response;},close:async()=>{observeStatus(baseline('confirmed'));serialOwnershipReleased=true;}};
 let maybeController=actualController;const controller=()=>actualController;
 const state=()=>({...initial,status,connected,running,serialOwnershipReleased,deviceBaselineConfirmed,deviceLeaseInactive,deviceRestorationConfirmed});
 ${source.slice(stopStart, closeStart)}
 ${source.slice(closeStart, closeEnd)}
 observeStatus(baseline('not_required'));return {state,stop,close};
`)(workerDeviceBaselineConfirmed, restoreAcceptanceBaseline, initial);
assert.equal(page.state().status, 'ready'); assert.equal(page.state().deviceBaselineConfirmed, true); assert.equal(page.state().deviceRestorationConfirmed, false);
const parts = {}, proofs = [];
const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
  now: () => 1000, persist: async (stage, value) => { parts[stage] = value; }, persistBegin: async () => {}, persistProof: async value => proofs.push(value),
  validateDiagnostics: async value => value });
server.listen(0, '127.0.0.1'); await once(server, 'listening');
const origin = `http://127.0.0.1:${server.address().port}`;
const post = async (path, value) => { const response = await fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) }); assert.equal(response.status, 200, path); return response.json(); };
const gate = { ...page, refresh: async () => {}, reviewQualificationAttempts: async () => ledger, reviewBudget: async () => original,
  exportDiagnostics: () => post('/diagnostic-export', { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 10 }] }),
  stratumV2Possession: async () => 'synthetic', stratumV2Status: async () => ({ schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
    observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }) };
try {
 const collected = await createBaselineCollector({ gate, published: page.state, campaignId: 'fixture', begin: () => post('/baseline-begin', {}),
   save: (stage, value) => stage === 'diagnostics' ? Promise.resolve() : post('/part', { stage, value }) })();
 assert.equal(collected.complete, true); assert.equal(proofs.length, 1);
 assert.equal(parts.state.status, 'ready'); assert.equal(parts.state.deviceRestorationConfirmed, false);
 assert.equal(parts.closed.status, 'closed'); assert.equal(parts.closed.deviceRestorationConfirmed, true);
 assert.equal(baselineConclusion(parts, context).complete, true);
 assert.deepEqual(currentProof(context, parts, 1000), proofs[0]);
 const failed = structuredClone(parts); failed.closed.deviceRestorationConfirmed = false;
 assert.equal(baselineConclusion(failed, context).complete, false); assert.throws(() => currentProof(context, failed, 1000));
 assert.equal(baselineConclusion(parts, {...context,allowConfirmedBaseline:true}).complete,false);
} finally { await server.release(); }
process.stdout.write('diagnostic_pre_stop_boundary_passed\n');

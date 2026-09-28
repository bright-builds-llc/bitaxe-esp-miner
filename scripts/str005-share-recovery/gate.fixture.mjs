import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { configuration } from '../str005-v2-serial/server-assets.mjs';
import { createRecoveryCollection } from '../str005-startup-probe/recovery-collection.mjs';
const root = process.argv[2], load = relative => import(pathToFileURL(resolve(root, relative)).href);
const config = await load('web/worker-serial-acceptance-config.ts');
const { WorkerV2SerialControl } = await load('web/worker-v2-serial-control.ts');
const context = { firmware_commit: 'f000872f2e436aa7cdaa8cbfa41eee965a27731e', app_elf_sha256: 'a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2',
  gate_commit: '9643e87664397a321c715a3a1b1bb6c1183b83ea', scope: 'share' }; context.before_source = context;
const trust = JSON.parse(await readFile(resolve(root, 'conformance/bwg-worker-deployment-trust-0.2/trust.json'), 'utf8'));
const before = config.parseWorkerSerialAcceptanceConfiguration(configuration(context, 'before', trust), context.gate_commit);
const candidate = config.parseWorkerSerialAcceptanceConfiguration(configuration(context, 'candidate', trust), context.gate_commit);
assert.throws(() => config.requireWorkerAcceptanceModeTransition(undefined, candidate), /v2_before_configuration_required/u);
config.requireWorkerAcceptanceModeTransition(undefined, before); config.requireWorkerAcceptanceModeTransition(before, candidate);
const idle = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 16, workerGeneration: 0, serialTransportEpoch: 2, observedAtUs: 10000, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
const { createWorkerV2PageOperations } = await load('web/worker-v2-page.ts');
let phase = 'before';
const binding = Buffer.alloc(32, 1).toString('base64url'), requests = [], events = [];
const control = new WorkerV2SerialControl({ requireScope(scope, effect) { assert.equal(scope, 'share'); assert.equal(effect, false); },
  maybeBinding: () => binding, possessionFresh: () => true, request: async (command, payload) => { requests.push({ command, payload }); return idle; } });
const page = createWorkerV2PageOperations({ serializeRead: operation => operation(), changed() {}, phase: () => phase, scope: () => 'share',
  connected: () => true, idle: () => true, maybeReviewedBinding: () => undefined, maybePreservation: () => undefined,
  controller: () => ({ prepareWorkerLeaseAuthorizationContext: async () => ({ controlSessionBindingSha256: binding }),
    stratumV2Status: (...args) => control.status(...args) }) });
await assert.rejects(page.stratumV2Possession(), /v2_page_admission/u);
phase = 'candidate'; assert.equal(await page.stratumV2Possession(), binding);
const gate = { ...page, reviewQualificationAttempts: async () => ({}), reviewBudget: async () => ({}), exportDiagnostics: async () => {},
  stop: async () => events.push('stop'), refresh: async () => {}, state: () => ({}), close: async () => events.push('close'),
  stratumV2Status: (...args) => page.stratumV2Status(...args) };
const saved = [];
const result = await createRecoveryCollection({ gate, begin: async () => ({ binding, attemptId: Buffer.alloc(16, 2).toString('base64url'),
  campaignId: Buffer.alloc(16, 3).toString('base64url'), statusMode: 'discover_current' }), save: async (stage, value) => saved.push({ stage, value }) })();
assert.equal(result.complete, true); assert.deepEqual(events, ['stop', 'close']); assert.equal(requests.length, 1);
assert.equal(requests[0].command, 'stratum_v2_status'); assert.equal(requests[0].payload.attemptId, null);
assert.equal(saved.find(row => row.stage === 'status').value.observation.bootOrdinal, 16);
const diagnostics = await load('web/worker-serial-diagnostics.ts'), exporter = await load('web/worker-diagnostic-export.ts');
const boot = diagnostics.maybeWorkerSerialDiagnostic('usb_reboot_discriminator schema=v1 boot_ordinal=16 reset_reason=panic uptime_ms=2000 redacted=true');
assert.equal(exporter.parseWorkerDiagnosticExport({ schema: 'worker-diagnostic-export-v1', observations: [boot] }).observations[0].boot_ordinal, 16);
// Execute the production page's exact receipt consumer, not a copied validation regex.
const production = await readFile(resolve(root, 'web/worker-serial-acceptance.ts'), 'utf8');
const marker = 'async function exportDiagnostics() {';
assert.equal(production.split(marker).length, 2);
const start = production.indexOf(marker), end = production.indexOf('\n}\n', start) + 2;
assert.ok(end > start);
let receipt = process.argv[3] ? JSON.parse(await readFile(process.argv[3], 'utf8')) :
  { diagnostic_export_saved: true, review_file: 'diagnostic-export-recovery.json' };
const consume = new Function('parseWorkerDiagnosticExport', 'localJson', 'localDiagnostics',
  `${production.slice(start, end)}; return exportDiagnostics;`)(exporter.parseWorkerDiagnosticExport,
  async (path, body) => { assert.equal(path, '/diagnostic-export'); assert.equal(body.schema, 'worker-diagnostic-export-v1'); return receipt; },
  { values: () => [boot] });
assert.deepEqual(await consume(), receipt);
receipt = { diagnostic_export_saved: true, review_file: 'diagnostics.json' };
await assert.rejects(consume(), /diagnostic_export_receipt/u);
// Execute the actual Gate Stop producer, then the production recovery judge.
const stopMarker = 'async function stop() {';
assert.equal(production.split(stopMarker).length, 2);
const stopStart = production.indexOf(stopMarker), stopEnd = production.indexOf('\nasync function close()', stopStart);
assert.ok(stopEnd > stopStart);
const { restoreAcceptanceBaseline } = await load('web/worker-serial-acceptance-actions.ts');
const { postStopState } = await import('./post-stop.fixture.mjs');
const { conclusion, recoveryProof } = await import('./model.mjs');
const { projectRecoveryPart } = await import('../str005-v2-serial/recovery-evidence.mjs');
const initial = { ...postStopState(context), status: 'ready' };
const stop = new Function('restoreAcceptanceBaseline', 'initial', 'assert', `
  let status = initial.status, running = false, maybeQualification = initial.qualification, maybeWindow, maybeConfiguration;
  let deviceBaselineConfirmed = true, deviceRestorationConfirmed = true;
  const stopTimer = () => {}, publish = () => {}, authorizationRecovery = {captureNormalStop() {}};
  const controller = () => ({restore: async reason => {assert.equal(reason, 'cancelled'); return {qualification: initial.qualification};}});
  const state = () => ({...initial,status,running,qualification:maybeQualification,deviceBaselineConfirmed,deviceRestorationConfirmed});
  ${production.slice(stopStart, stopEnd)}; return stop;
`)(restoreAcceptanceBaseline, initial, assert);
const stopped = await stop(); assert.equal(stopped.status, 'baseline_confirmed');
const recorded = JSON.stringify(stopped);
const proofContext = {...context,source_commit:'d'.repeat(40),physical:'e'.repeat(64)};
const observed = {state:stopped,closed:postStopState(context,true),
  ledger:{schema:'worker-qualification-ledger-v1',next_ordinal:21,last_completed_ordinal:20,total_charged_ms:2100000,pending:false},
  original_budget:{schema:'worker-budget-review-v1',campaign_match:true,reserved_mask:7,completed_mask:7,charged_ms:240000,pending:false},
  status:projectRecoveryPart('status',idle,context),
  diagnostics:projectRecoveryPart('diagnostics',{schema:'worker-diagnostic-export-v1',observations:[boot]},context),
  errors:{schema:'str005-recovery-errors-v1',firstFailure:null,errors:[]},finished:{failures:[]}};
assert.equal(conclusion(observed,proofContext,true).current_safe_recovery,true);
assert.equal(recoveryProof(observed,proofContext,1000,2000).current_v2_idle,true);
assert.equal(JSON.stringify(stopped),recorded);
process.stdout.write('share_recovery_gate_boundary_passed\n');

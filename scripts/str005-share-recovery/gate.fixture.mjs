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
process.stdout.write('share_recovery_gate_boundary_passed\n');

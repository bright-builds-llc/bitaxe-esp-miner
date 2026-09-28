// Exact production restore/possession method bodies plus actual V2 page/control; only device wire is simulated.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createRecoveryCollection } from './recovery-collection.mjs';
const root = process.argv[2], load = relative => import(pathToFileURL(resolve(root, relative)).href);
const { WorkerV2SerialControl } = await load('web/worker-v2-serial-control.ts');
const { createWorkerV2PageOperations } = await load('web/worker-v2-page.ts');
const { restoreAcceptanceBaseline } = await load('web/worker-serial-acceptance-actions.ts');
const { workerRestoredStatusMatches } = await load('web/worker-postconditions.ts');
const { serialFailure } = await load('web/worker-serial-errors.ts');
const source = new Bun.Transpiler({ loader: 'ts' }).transformSync(await readFile(resolve(root, 'web/worker-serial-controller-runtime.ts'), 'utf8')).replaceAll('async#', 'async #');
function method(name) {
  const start = source.indexOf(`async ${name}(`); assert.ok(start >= 0);
  const opening = source.indexOf('{', start); let depth = 0, end;
  for (let i = opening; i < source.length; i++) { if (source[i] === '{') depth++; if (source[i] === '}' && --depth === 0) { end = i + 1; break; } }
  assert.ok(end);
  const code = source.slice(start, end).replace(`async ${name}(`, `async function ${name.replace('#', '')}(`).replaceAll('this.#', 'this.');
  return Function('workerRestoredStatusMatches', 'serialFailure', `return (${code});`)(workerRestoredStatusMatches, serialFailure);
}
const restore = method('#restoreCommand'), possess = method('prepareWorkerLeaseAuthorizationContext');
const binding = Buffer.alloc(32, 1).toString('base64url'); let proofs = 0;
const status = { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 16, workerGeneration: 0, serialTransportEpoch: 2, observedAtUs: 10000, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } };
const controller = { activeLease: false, maybePossession: undefined, requireReady() {}, lost(error) { throw error; },
  async prove() { proofs++; return { controlSessionBindingSha256: binding }; },
  async statusRequest() { return { protocolVersion: 'bwg-worker-controller/0.4', state: 'baseline', monotonicMilliseconds: 1, restoration: { status: 'confirmed', reason: 'cancelled' } }; } };
controller.restore = reason => restore.call(controller, 'restore', reason);
controller.prepareWorkerLeaseAuthorizationContext = operation => possess.call(controller, operation);
const control = new WorkerV2SerialControl({ requireScope() {}, maybeBinding: () => controller.maybePossession?.controlSessionBindingSha256,
  possessionFresh: () => controller.maybePossession !== undefined, request: async () => status });
controller.stratumV2Status = (...args) => control.status(...args);
const page = createWorkerV2PageOperations({ serializeRead: operation => operation(), changed() {}, phase: () => 'candidate', scope: () => 'share',
  connected: () => true, idle: () => true, maybeReviewedBinding: () => undefined, maybePreservation: () => undefined, controller: () => controller });
const original = await page.stratumV2Possession(); await restoreAcceptanceBaseline(controller);
await assert.rejects(page.stratumV2Status('share', null, original), error => error.category === 'v2_possession');
const saved = new Map(); const gate = { ...page, state: () => ({}), refresh: async () => {}, stop: () => restoreAcceptanceBaseline(controller),
  close: async () => {}, reviewQualificationAttempts: async () => ({}), reviewBudget: async () => ({}), exportDiagnostics: async () => {} };
const result = await createRecoveryCollection({ gate, begin: async () => ({ binding: await gate.stratumV2Possession(), campaignId: 'synthetic',
  attemptId: Buffer.alloc(16, 2).toString('base64url'), statusMode: 'discover_current' }), save: async (stage, value) => saved.set(stage, value) })();
assert.equal(result.complete, true); assert.equal(saved.get('status').state, 'idle'); assert.equal(proofs, 3);
process.stdout.write('post_stop_possession_boundary_passed\n');

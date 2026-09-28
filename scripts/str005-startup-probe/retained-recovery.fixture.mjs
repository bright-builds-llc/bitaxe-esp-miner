// Production Gate command decoder, V2 controller and page; simulated firmware wire only.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { collectRecovery } from './client.mjs';
import { createPage } from './page.mjs';
const gateRoot = process.argv[2];
const load = relative => import(pathToFileURL(resolve(gateRoot, relative)).href);
const { WorkerV2SerialControl } = await load('web/worker-v2-serial-control.ts');
const { createWorkerV2PageOperations } = await load('web/worker-v2-page.ts');
const { requestWorkerSerialCommand } = await load('web/worker-serial-command.ts');
const { serialFailure } = await load('web/worker-serial-errors.ts');
const { v2Accepted, v2Input } = await load('web/worker-v2-serial.fixture.ts');
function fixture() {
  let live = true, sequence = 0; const ids = [], requests = [], saved = new Map();
  const binding = Buffer.alloc(32, 1).toString('base64url');
  const status = v2Accepted(); status.scope = status.record.scope = 'share'; status.record.outcome = 'cancelled';
  status.record.authorityDeadlineDeviceUs = 180001000; status.record.observationDeadlineDeviceUs = null;
  const control = new WorkerV2SerialControl({ requireScope(scope, effect) { assert.equal(scope, 'share'); assert.equal(effect, false); },
    maybeBinding: () => binding, possessionFresh: () => true,
    request: (command, payload) => requestWorkerSerialCommand({ command, maybePayload: payload, requestId: `fixture_${++sequence}`, fenced: control.fenced,
      exchange: async request => {
        assert.equal(request.command, 'stratum_v2_status'); ids.push(request.payload.attemptId);
        if (!live) throw serialFailure('closed');
        // Exact firmware prepare_v2 match: (None, Some(record)) rejects before Gate sees a status.
        // USB process_frame revokes the epoch after this non-restoration_pending rejection.
        if (request.payload.attemptId !== v2Input.attemptId) {
          live = false; return { protocolVersion: 'bwg-worker-controller/0.4', requestId: request.requestId, ok: false,
            error: { code: 'command_rejected', message: 'invalid_transition' } };
        }
        return { protocolVersion: 'bwg-worker-controller/0.4', requestId: request.requestId, ok: true, result: status };
      } }),
  });
  const page = createWorkerV2PageOperations({ serializeRead: run => run(), changed() {}, phase: () => 'candidate', scope: () => 'share',
    connected: () => live, idle: () => true, maybeReviewedBinding: () => undefined, maybePreservation: () => undefined,
    controller: () => ({ prepareWorkerLeaseAuthorizationContext: async () => ({ controlSessionBindingSha256: binding }),
      stratumV2Status: (...args) => control.status(...args) }) });
  const gate = { ...page, state: () => ({ connected: live, running: false }), refresh: async () => {}, reviewQualificationAttempts: async () => ({}),
    reviewBudget: async () => ({}), exportDiagnostics: async () => {}, stop: async () => {}, close: async () => { live = false; } };
  const post = async (path, input) => {
    requests.push(path);
    if (path === '/startup/context') return { originalCampaignId: 'fixture', attemptId: v2Input.attemptId, startState: 'confirmed' };
    if (path === '/startup/recovery-begin') { assert.equal(input.status.record.attemptId, v2Input.attemptId); return { sequence: 1 }; }
    if (path === '/startup/recovery') saved.set(input.stage, input.value);
    return { recorded: true };
  };
  return { gate, post, ids, saved, requests, live: () => live };
}
// Establish the regression's failure category at the actual Gate decoding boundary.
const bad = fixture();
await assert.rejects(bad.gate.stratumV2Status('share', null, await bad.gate.stratumV2Possession()),
  error => error.category === 'command_rejected' && error.rejection === 'invalid_transition');
assert.equal(bad.live(), false);
// Both production recovery entrypoints must query the known attempt without an idle probe.
const same = fixture();
await collectRecovery({ gate: same.gate, campaignId: 'fixture', attemptId: v2Input.attemptId, statusMode: 'confirmed',
  save: async (stage, value) => same.saved.set(stage, value) });
assert.deepEqual(same.ids, [v2Input.attemptId]); assert.equal(same.live(), true); assert.equal(same.saved.get('status').record.attemptId, v2Input.attemptId);
const fresh = fixture();
await createPage(fresh.gate, fresh.post, () => {}).recoverFresh();
assert.ok(fresh.requests.includes('/startup/recovery-begin'));
assert.deepEqual(fresh.ids, [v2Input.attemptId, v2Input.attemptId]); assert.ok(fresh.saved.has('status')); assert.ok(fresh.saved.has('closed'));
process.stdout.write('retained_recovery_boundary_passed\n');

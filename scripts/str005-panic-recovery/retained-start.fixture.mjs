// End to end through the recovery owner after a sealed Start: Start evidence -> predecessor -> status mode
// -> the server's begin reply -> the shared collector -> the production Gate decoder -> firmware keyed by
// the device record attempt. Only the firmware wire is simulated.
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { deviceRecordAttemptId } from '../str005-startup-probe/start-record.mjs';
import { createRecoveryCollection } from '../str005-startup-probe/recovery-collection.mjs';
import { retainedAttempt } from './control-diagnostic.mjs';
import { statusModeFor } from '../str005-share-recovery/main.mjs';
import { beginReply } from '../str005-share-recovery/server.mjs';
const gateRoot = process.argv[2];
const load = relative => import(pathToFileURL(resolve(gateRoot, relative)).href);
const { WorkerV2SerialControl } = await load('web/worker-v2-serial-control.ts');
const { createWorkerV2PageOperations } = await load('web/worker-v2-page.ts');
const { requestWorkerSerialCommand } = await load('web/worker-serial-command.ts');
const { serialFailure } = await load('web/worker-serial-errors.ts');
const { v2Accepted, v2Input } = await load('web/worker-v2-serial.fixture.ts');

const deviceAttempt = v2Input.attemptId, ownerNonce = Buffer.alloc(16, 9).toString('base64url');
assert.notEqual(ownerNonce, deviceAttempt);
const install = { firmware_commit: 'c'.repeat(40), app_elf_sha256: 'e'.repeat(64), physical: 'p'.repeat(64) };
const ledger = { schema: 'worker-qualification-ledger-v1', next_ordinal: 27, last_completed_ordinal: 26, total_charged_ms: 3180000, pending: false };
const shareRecord = { attemptId: deviceAttempt, scope: 'share' };
// The heartbeat-shaped sealed Start: the owner context carries a nonce; the issued attempt keys the record.
const start = { context: { ...install, attemptId: ownerNonce }, before: { attempt: { id: deviceAttempt } },
  run: { dispatchStatus: { record: shareRecord } }, recoveryStatus: { state: 'terminal', record: shareRecord } };

function firmware() {
  let live = true, sequence = 0; const ids = [], saved = new Map();
  const binding = Buffer.alloc(32, 1).toString('base64url');
  const status = v2Accepted(); status.scope = status.record.scope = 'share'; status.record.outcome = 'cancelled';
  status.record.authorityDeadlineDeviceUs = 180001000; status.record.observationDeadlineDeviceUs = null;
  const control = new WorkerV2SerialControl({ requireScope(scope, effect) { assert.equal(scope, 'share'); assert.equal(effect, false); },
    maybeBinding: () => binding, possessionFresh: () => true,
    request: (command, payload) => requestWorkerSerialCommand({ command, maybePayload: payload, requestId: `fixture_${++sequence}`, fenced: control.fenced,
      exchange: async request => {
        ids.push(request.payload.attemptId);
        if (!live) throw serialFailure('closed');
        // Firmware prepare_v2: only a matching attempt reads a retained record; anything else is an
        // invalid transition, after which USB process_frame revokes the session.
        if (request.payload.attemptId !== deviceAttempt) {
          live = false;
          return { protocolVersion: 'bwg-worker-controller/0.4', requestId: request.requestId, ok: false,
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
  return { gate, ids, saved };
}

async function collect(context) {
  const wire = firmware();
  const reply = beginReply({ collectionId: 'fixture' }, { ...context, original_campaign_id: 'fixture' });
  const result = await createRecoveryCollection({ gate: wire.gate, begin: async () => ({ ...reply, binding: 'fixture' }),
    save: async (stage, value) => wire.saved.set(stage, value) })();
  return { result, ids: wire.ids, saved: wire.saved };
}

// The owner path: sealed evidence yields the device attempt, the predecessor marks it retained.
const before = retainedAttempt(install, { attemptId: 'install-attempt', ledger },
  { context: start.context, ledger, deviceRecordAttemptId: deviceRecordAttemptId(start) });
const owner = await collect({ attemptId: before.attempt.id, statusMode: statusModeFor(before.attempt) });
assert.equal(owner.result.firstFailure, null);
assert.deepEqual(owner.ids, [deviceAttempt]);
assert.equal(owner.saved.get('status').record.attemptId, deviceAttempt);

// Recovery011's mistake: the owner nonce is rejected, and the rejection now names its reason.
const nonce = await collect({ attemptId: ownerNonce, statusMode: 'confirmed' });
assert.deepEqual([nonce.result.firstFailure, nonce.ids], [{ phase: 'status', category: 'command_rejected', rejection: 'invalid_transition' }, [ownerNonce]]);
assert.equal(nonce.saved.get('errors').schema, 'str005-recovery-errors-v2');

// Recoveries 009 and 010: discovery's null query is rejected before any id is sent.
const discovered = await collect({ attemptId: deviceAttempt, statusMode: 'discover_current' });
assert.deepEqual([discovered.result.firstFailure, discovered.ids], [{ phase: 'status', category: 'command_rejected', rejection: 'invalid_transition' }, [null]]);
process.stdout.write('retained_start_recovery_passed\n');

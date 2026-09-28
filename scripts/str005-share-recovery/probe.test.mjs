import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { requireEnabled, SHARE_SEAL, argumentsFor } from './contract.mjs';
import { conclusion, recoveryProof } from './model.mjs';
import { createRecoveryServer } from './server.mjs';
import { projectRecoveryPart } from '../str005-v2-serial/recovery-evidence.mjs';
import { ledger, original, state } from '../str005-noise-serial/test-fixture.mjs';
import { channelFixture } from '../str005-v2-serial/protocol-judge.test-helper.mjs';
import { createRecoveryCollection } from '../str005-startup-probe/recovery-collection.mjs';
const context = { source_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), gate_commit: 'd'.repeat(40),
  physical: 'e'.repeat(64), scope: 'share', attemptId: Buffer.alloc(16, 1).toString('base64url'), original_campaign_id: Buffer.alloc(16, 2).toString('base64url') };
function idle() { return { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 16, workerGeneration: 2, serialTransportEpoch: 3, observedAtUs: 15000, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }; }
function parts() {
  return { state: state(context), closed: state(context, 'candidate', true), ledger: structuredClone(ledger), original_budget: original,
    status: projectRecoveryPart('status', idle(), context), diagnostics: projectRecoveryPart('diagnostics', { schema: 'worker-diagnostic-export-v1', observations: [
      { category: 'boot', authoritative: false, boot_ordinal: 16, reset_reason: 'panic', uptime_ms: 2000 }] }, context),
    finished: { failures: [] }, errors: { schema: 'str005-recovery-errors-v1', firstFailure: null, errors: [] } };
}
test('closed owner gate needs published enable and exact active seal', () => {
  // Arrange
  const tasks = `## Active\n### task-str005-v2-accepted-share-probe | fixture\nShare failure recovery hardware: enabled.\nShare failure recovery seal: ${SHARE_SEAL}.\n`;
  // Act / Assert
  assert.throws(() => requireEnabled(tasks, false)); requireEnabled(tasks, true);
  assert.throws(() => requireEnabled(tasks + tasks, true)); assert.throws(() => requireEnabled(tasks.replace('## Active', '## Future'), true));
  assert.throws(() => requireEnabled(tasks.replace(SHARE_SEAL, '0'.repeat(64)), true));
  assert.throws(() => argumentsFor(['preflight', '--private-root', '/r', '--gate-root', '/g', '--share-root', '/s'], false));
  assert.throws(() => argumentsFor(['serve', '--private-root', '/r', '--authority-directory', '/secret'], true));
});
test('idle current proof measures arbitrary actual ledger and makes no historical claim', () => {
  // Arrange
  const observed = parts(); observed.ledger = { ...ledger, next_ordinal: 42, last_completed_ordinal: 41, total_charged_ms: 6000000 };
  // Act
  const result = conclusion(observed, context, true), proof = recoveryProof(observed, context, 1000, 2000);
  // Assert
  assert.equal(result.current_safe_recovery, true); assert.equal(result.historical_resource_unavailable, true);
  assert.equal(result.qualification_complete, false); assert.equal(proof.ledger.next_ordinal, 42);
  assert.equal(proof.schema, 'str005-current-recovery-proof-v1'); assert.equal(proof.observed_at_unix_ms, 1000);
  assert.throws(() => recoveryProof(observed, context, 1000, 121001));
});
test('known retained release creates V2 without synthetic idle', () => {
  // Arrange
  const observed = parts(), input = idle(), record = channelFixture().deviceRecords.at(-1);
  Object.assign(record, { scope: 'share', bootOrdinal: 16, authorityDeadlineDeviceUs: 180001000, observationDeadlineDeviceUs: null,
    poolSessionGeneration: 2, poolTransportEpoch: 3 });
  record.events = record.events.filter(row => row.kind !== 'connected').map((row, index) => ({ ...row, sequence: index + 1 }));
  input.state = 'terminal'; input.record = record; observed.status = projectRecoveryPart('status', input, context);
  // Act
  const proof = recoveryProof(observed, context, 1000, 2000);
  // Assert
  assert.equal(proof.schema, 'str005-current-recovery-proof-v2'); assert.equal(proof.current_v2_released, true);
  assert.equal(Object.hasOwn(proof, 'current_v2_idle'), false);
  observed.status.record.resources.fenceRetained = true; assert.throws(() => recoveryProof(observed, context, 1000, 2000));
});
test('missing cleanup errors pending accounting and old boot cannot create a safe proof', () => {
  for (const mutate of [p => { p.closed.serialOwnershipReleased = false; }, p => { p.state.deviceRestorationConfirmed = false; },
    p => { p.ledger.pending = true; }, p => { p.status.observation.bootOrdinal = 15; }, p => { delete p.errors; }]) {
    // Arrange
    const observed = parts(); mutate(observed);
    // Act / Assert
    assert.equal(conclusion(observed, context, true).current_safe_recovery, false);
    assert.throws(() => recoveryProof(observed, context, 1000, 2000));
  }
  assert.equal(conclusion(parts(), context, false).current_safe_recovery, false);
});
test('typed-only current status fallback and independent failures preserve cleanup', async () => {
  // Arrange
  const ids = [], saved = []; let closed = false;
  const gate = { stratumV2Possession: async () => 'fresh-after-stop', reviewQualificationAttempts: async () => { throw Object.assign(Error('ledger'), { category: 'io' }); }, reviewBudget: async () => original,
    exportDiagnostics: async () => {}, stop: async () => {}, refresh: async () => {}, state: () => state(context, 'candidate', closed),
    stratumV2Status: async (_scope, id) => { ids.push(id); throw Object.assign(Error('reject'), { category: 'command_rejected' }); }, close: async () => { closed = true; } };
  // Act
  const result = await createRecoveryCollection({ gate, begin: async () => ({ binding: 'fixture', attemptId: context.attemptId,
    campaignId: context.original_campaign_id, statusMode: 'discover_current' }), save: async (stage, value) => saved.push({ stage, value }) })();
  // Assert
  assert.deepEqual(ids, [null]); assert.equal(closed, true); assert.equal(result.complete, false);
  assert.deepEqual(result.firstFailure, { phase: 'ledger', category: 'io' }); assert.ok(saved.some(row => row.stage === 'closed'));
});
test('actual server uses challenge and stage ticket; late writes reject but cleanup survives', async t => {
  // Arrange
  let now = 1000; const saved = [];
  const server = createRecoveryServer({ root: '/unused', context, assets: { page: '', bundle: Buffer.from(''), trust: {} }, verify: async () => {} }, {
    now: () => now, persist: async (stage, value) => saved.push({ stage, value }), validateDiagnostics: async value => value });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  // Act / Assert
  assert.equal((await post('/recovery-prepare', { before: state(context), closed: state(context, 'candidate', true) })).status, 200);
  const challenge = await (await post('/recovery-challenge', {})).json();
  const binding = Buffer.alloc(32, 1).toString('base64url');
  assert.equal((await post('/recovery-begin', { nonce: 'wrong', binding, state: state(context) })).status, 400);
  const begin = await (await post('/recovery-begin', { nonce: challenge.nonce, binding, state: state(context) })).json();
  const ticket = await (await post('/stage-begin', { collectionId: begin.collectionId, phase: 'ledger' })).json();
  now += 30001;
  assert.equal((await post('/recovery-part', { collectionId: begin.collectionId, stage: 'ledger', ticket: ticket.token, value: ledger })).status, 400);
  assert.equal((await post('/stage-end', { collectionId: begin.collectionId, phase: 'ledger', token: ticket.token })).status, 200);
  now += 120000;
  assert.equal((await post('/stage-begin', { collectionId: begin.collectionId, phase: 'status' })).status, 200);
  assert.equal((await post('/recovery-part', { collectionId: begin.collectionId, stage: 'closed', ticket: null, value: state(context, 'candidate', true) })).status, 200);
  assert.equal((await post('/recovery-part', { collectionId: begin.collectionId, stage: 'finished', ticket: null, value: { failures: ['ledger'] } })).status, 200);
  assert.equal((await post('/recovery-part', { collectionId: begin.collectionId, stage: 'state', ticket: null, value: state(context) })).status, 400);
  for (const path of ['/start', '/grant', '/install', '/core-dump-read', '/self-test-claim']) assert.equal((await post(path, {})).status, 400);
  assert.equal(saved.find(row => row.stage === 'collection-begin').value.bindingSha256.length, 64);
});

test('actual pinned Gate parser and transitions admit current idle recovery without Start', async () => {
  // Arrange
  const { execFile } = await import('node:child_process'), { promisify } = await import('node:util');
  const { dirname, resolve } = await import('node:path'), { fileURLToPath } = await import('node:url');
  const here = dirname(fileURLToPath(import.meta.url));
  const gateRoot = process.argv[2] ? dirname(resolve(process.argv[2])) : resolve(here, '../../../bitaxe-turnstile-system');
  // Act
  const result = await promisify(execFile)('bun', [resolve(here, 'gate.fixture.mjs'), gateRoot], { timeout: 15000 });
  // Assert
  assert.equal(result.stdout.trim(), 'share_recovery_gate_boundary_passed'); assert.equal(result.stderr, '');
});

test('bootstrap admission or stalled Stop still closes and persists its earliest failure', async () => {
  // Arrange
  const { prepareRecovery } = await import('./bootstrap.mjs');
  const calls = [], records = []; let closed = false;
  const gate = { state: () => closed ? state(context, 'candidate', true) : { ...state(context), running: true },
    stop: async () => { calls.push('stop'); await new Promise(() => {}); }, close: async () => { calls.push('close'); closed = true; }, configure: async () => calls.push('configure') };
  // Act
  const result = await prepareRecovery({ gate, configuration: {}, record: async () => calls.push('prepare'), saveFailure: async value => records.push(value), cleanupMs: 5 });
  // Assert
  assert.equal(result.prepared, false); assert.deepEqual(calls, ['stop', 'close']);
  assert.deepEqual(records[0].errors.firstFailure, { phase: 'state', category: 'operation_failed' });
  assert.equal(records[0].closed.serialOwnershipReleased, true);
});

test('live writer prevents sealing; after its actual exit partial evidence can seal', async t => {
  // Arrange
  const { spawn } = await import('node:child_process'), { mkdtemp, realpath, rm, access } = await import('node:fs/promises');
  const { tmpdir } = await import('node:os'), { resolve } = await import('node:path');
  const { processSnapshot } = await import('../str005-v2-serial/host-resources.mjs');
  const { writeNew, proof, verifyInventory } = await import('../str005-noise-serial/files.mjs');
  const { finish } = await import('./main.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'share-recovery-seal-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const child = spawn(process.execPath, ['-e', 'setInterval(()=>{},1000)'], { stdio: 'ignore' });
  await once(child, 'spawn'); t.after(() => child.kill('SIGKILL'));
  const owner = (await processSnapshot()).find(row => row.pid === child.pid); assert.ok(owner);
  await writeNew(resolve(root, 'server-owner.json'), { owner, port: 1, serialPort: '/dev/test-only', physical: context.physical });
  // Act / Assert
  const live = await finish(root, context);
  assert.equal(live.sealed, false); await assert.rejects(access(resolve(root, 'sealed-inventory.json')));
  child.kill('SIGTERM'); await once(child, 'exit');
  const partial = await finish(root, context);
  assert.equal(partial.current_safe_recovery, false);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
});

test('empty or mismatched diagnostic boot becomes an explicit partial blocker', () => {
  // Arrange / Act / Assert
  for (const observations of [[], [{ category: 'boot', authoritative: false, boot_ordinal: 17, reset_reason: 'panic', uptime_ms: 1 }]]) {
    const observed = parts(); observed.diagnostics.observations = observations;
    const result = conclusion(observed, context, true);
    assert.equal(result.current_safe_recovery, false);
    assert.ok(result.blockers.includes('current_boot_correlation_missing'));
    assert.throws(() => recoveryProof(observed, context, 1000, 2000), { code: 'share_recovery_proof_incomplete' });
  }
});

test('actual diagnostic HTTP receipt names its private artifact and passes the production Gate consumer', async t => {
  // Arrange
  const { mkdtemp, realpath, rm } = await import('node:fs/promises'), { tmpdir } = await import('node:os');
  const { dirname, resolve } = await import('node:path'), { fileURLToPath } = await import('node:url');
  const { execFile } = await import('node:child_process'), { promisify } = await import('node:util');
  const { writeNew, proof } = await import('../str005-noise-serial/files.mjs');
  const { DIAGNOSTIC_FILE, readRecoveryPart } = await import('./diagnostics.mjs');
  const here = dirname(fileURLToPath(import.meta.url)), gateRoot = process.argv[2] ? dirname(resolve(process.argv[2])) : resolve(here, '../../../bitaxe-turnstile-system');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'share-diagnostic-receipt-')));
  const server = createRecoveryServer({ root, context: { ...context, gate_root: gateRoot }, assets: { page: '', bundle: Buffer.from(''), trust: {} }, verify: async () => {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(async () => { await server.release(); await rm(root, { recursive: true, force: true }); });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, body) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  await post('/recovery-prepare', { before: state(context), closed: state(context, 'candidate', true) });
  const challenge = await (await post('/recovery-challenge', {})).json();
  const begin = await (await post('/recovery-begin', { nonce: challenge.nonce, binding: Buffer.alloc(32, 1).toString('base64url'), state: state(context) })).json();
  await post('/stage-begin', { collectionId: begin.collectionId, phase: 'diagnostics' });
  // Act
  const response = await post('/diagnostic-export', { schema: 'worker-diagnostic-export-v1', observations: [
    { category: 'boot', authoritative: false, boot_ordinal: 16, reset_reason: 'panic', uptime_ms: 2000 }] });
  assert.equal(response.status, 200);
  const receipt = await response.json();
  await writeNew(resolve(root, 'receipt.json'), receipt);
  const consumed = await promisify(execFile)('bun', [resolve(here, 'gate.fixture.mjs'), gateRoot, resolve(root, 'receipt.json')], { timeout: 15000 });
  // Assert
  assert.deepEqual(receipt, { diagnostic_export_saved: true, review_file: DIAGNOSTIC_FILE });
  assert.equal((await proof(root, receipt.review_file)).value.observations[0].boot_ordinal, 16);
  assert.equal((await readRecoveryPart(root, 'diagnostics')).observations[0].boot_ordinal, 16);
  assert.equal(consumed.stdout.trim(), 'share_recovery_gate_boundary_passed'); assert.equal(consumed.stderr, '');
  await writeNew(resolve(root, 'diagnostics.json'), { historical: true });
  await assert.rejects(readRecoveryPart(root, 'diagnostics'), { code: 'share_recovery_ambiguous_diagnostics' });
});

test('legacy diagnostic artifact is readable without rewriting historical evidence', async t => {
  // Arrange
  const { mkdtemp, realpath, rm, readFile } = await import('node:fs/promises'), { tmpdir } = await import('node:os'), { resolve } = await import('node:path');
  const { writeNew } = await import('../str005-noise-serial/files.mjs');
  const { readRecoveryPart } = await import('./diagnostics.mjs');
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'share-diagnostic-legacy-'))); t.after(() => rm(root, { recursive: true, force: true }));
  await writeNew(resolve(root, 'diagnostics.json'), { fixture: true }); const before = await readFile(resolve(root, 'diagnostics.json'));
  // Act / Assert
  assert.deepEqual(await readRecoveryPart(root, 'diagnostics'), { fixture: true });
  assert.deepEqual(await readFile(resolve(root, 'diagnostics.json')), before);
});

test('recorded post-Stop baseline_confirmed passes the production judge without changing state', async () => {
  // Arrange
  const { postStopState } = await import('./post-stop.fixture.mjs');
  const { baselineConclusion, currentProof } = await import('../str005-panic-probe/model.mjs');
  const { validateState } = await import('../fixed-usb-qualification/judge.mjs');
  const observed = parts(); observed.state = postStopState(context); observed.closed = postStopState(context, true);
  const before = JSON.stringify(observed);
  const legacy = { ...observed }; delete legacy.errors;
  // Act
  validateState(observed.state, context); validateState(observed.closed, context);
  const result = conclusion(observed, context, true), proof = recoveryProof(observed, context, 1000, 2000);
  // Assert
  assert.equal(result.current_safe_recovery, true); assert.equal(proof.current_v2_idle, true);
  assert.equal(JSON.stringify(observed), before); assert.equal(observed.state.status, 'baseline_confirmed');
  assert.equal(baselineConclusion(legacy).complete, false);
  assert.throws(() => currentProof({ ...context, commit: context.source_commit, before_source: context, detector: { physical: context.physical } }, legacy, 1000));
});

test('post-Stop opt-in retains restoration preservation authority and status guards', async () => {
  // Arrange / Act / Assert
  const { postStopState } = await import('./post-stop.fixture.mjs');
  for (const status of ['ready', 'baseline_confirmed']) {
    const observed = parts(); observed.state = { ...postStopState(context), status }; observed.closed = postStopState(context, true);
    assert.equal(conclusion(observed, context, true).current_safe_recovery, true);
    for (const mutate of [p => { p.state.deviceRestorationConfirmed = false; }, p => { p.state.deviceLeaseInactive = false; },
      p => { p.state.deviceBaselineConfirmed = false; }, p => { p.state.running = true; }, p => { p.state.heartbeatSuppressed = true; },
      p => { p.state.preservation.settings_match = false; }, p => { p.state.preservation.authorization_high_water_match = false; },
      p => { p.state.preservation.device_identity_match = false; }, p => { p.state.preservation.mine_on_boot = true; },
      p => { p.state.status = 'failed'; }]) {
      const rejected = structuredClone(observed); mutate(rejected);
      assert.throws(() => recoveryProof(rejected, context, 1000, 2000));
    }
  }
});

test('fresh finalization emits proof for unchanged recorded post-Stop state shape', async t => {
  // Arrange
  const { mkdtemp, realpath, mkdir, rm, writeFile, readFile } = await import('node:fs/promises');
  const { tmpdir } = await import('node:os'), { resolve } = await import('node:path');
  const { spawn } = await import('node:child_process'), { createServer } = await import('node:net');
  const { processSnapshot } = await import('../str005-v2-serial/host-resources.mjs');
  const { writeNew, proof, verifyInventory } = await import('../str005-noise-serial/files.mjs');
  const { postStopState } = await import('./post-stop.fixture.mjs');
  const { DIAGNOSTIC_FILE } = await import('./diagnostics.mjs');
  const { finish } = await import('./main.mjs');
  const parent = await realpath(await mkdtemp(resolve(tmpdir(), 'post-stop-finalize-'))), root = resolve(parent, 'attempt');
  t.after(() => rm(parent, { recursive: true, force: true })); await mkdir(root, { mode: 0o700 });
  const child = spawn(process.execPath, ['-e', 'setInterval(()=>{},1000)'], { stdio: 'ignore' }); await once(child, 'spawn'); t.after(() => child.kill('SIGKILL'));
  const owner = (await processSnapshot()).find(row => row.pid === child.pid); assert.ok(owner); child.kill('SIGTERM'); await once(child, 'exit');
  const listener = createServer(); listener.listen(0, '127.0.0.1'); await once(listener, 'listening'); const port = listener.address().port; listener.close(); await once(listener, 'close');
  await writeNew(resolve(root, 'server-owner.json'), { owner, port, serialPort: '/dev/test-only', physical: context.physical });
  await writeFile(resolve(parent, 'final-detector.stdout.log'), `port: /dev/test-only\nphysical_identity_sha256: ${context.physical}\nusb_profile: serial_jtag_runtime\n`, { mode: 0o600 });
  const observed = parts(); observed.state = postStopState(context); observed.closed = postStopState(context, true);
  observed.ledger = { ...ledger, next_ordinal: 21, last_completed_ordinal: 20, total_charged_ms: 2100000 };
  for (const [stage, value] of Object.entries(observed)) await writeNew(resolve(root, stage === 'diagnostics' ? DIAGNOSTIC_FILE : `${stage}.json`), value);
  await writeNew(resolve(root, 'collection-begin.json'), { schema: 'str005-share-recovery-begin-v1', startedAtUnixMs: Date.now() });
  const unchanged = await readFile(resolve(root, 'state.json'));
  // Act
  const result = await finish(root, context, { requireNoHolders: () => {} }); // Only serial absence is simulated; no device opens.
  // Assert
  assert.equal(result.current_safe_recovery, true); assert.equal(result.fresh_effect_proof, true);
  assert.equal((await proof(root, 'current-recovery.json')).value.ledger.next_ordinal, 21);
  assert.equal((await proof(root, 'state.json')).value.status, 'baseline_confirmed');
  assert.deepEqual(await readFile(resolve(root, 'state.json')), unchanged);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
});

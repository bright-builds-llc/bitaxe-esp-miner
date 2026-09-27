import assert from 'node:assert/strict';
import test from 'node:test';
import { once } from 'node:events';
import { createBaselineCollector } from './client.mjs';
import { createProbeServer } from './server.mjs';
import { baselineConclusion, currentProof, validatePart, applyInstallationOutcome, applyRecoveryOnlyOutcome } from './model.mjs';
import { ledger, original, state } from '../str005-noise-serial/test-fixture.mjs';
const context = { commit: 'a'.repeat(40), gate_commit: 'b'.repeat(40), firmware_commit: 'a'.repeat(40), app_elf_sha256: 'c'.repeat(64),
  before_source: { firmware_commit: 'd'.repeat(40), app_elf_sha256: 'e'.repeat(64) }, detector: { physical: 'f'.repeat(64) }, scope: 'share' };
const diagnostics = { schema: 'worker-diagnostic-export-v1', observations: [{ category: 'boot', authoritative: false, boot_ordinal: 3, reset_reason: 'software_cpu', uptime_ms: 10 }] };
function preservedCore() { return { schema: 'str005-existing-core-preservation-v1', empty_core_dump: true, bytes: 974848,
  expected_boot_ordinal: 3, firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
  recovery_seal_sha256: '1'.repeat(64), result_sha256: '2'.repeat(64), dump_sha256: '3'.repeat(64), partition_table_sha256: '4'.repeat(64) }; }
function idle() { return { schema: 'worker-stratum-v2-status-v1', scope: 'share', state: 'idle', connection: null, record: null,
  observation: { bootOrdinal: 3, workerGeneration: 0, serialTransportEpoch: 1, observedAtUs: 100, clockValid: true, stationIpv4: null, wifiConnected: false, socket: null } }; }
function parts() { return { state: state(context, 'before'), closed: state(context, 'before', true), ledger, original_budget: original,
  diagnostics, status: validatePart('status', idle(), context), finished: { failures: [] } }; }

test('current proof measures an arbitrary fresh ordinal without claiming historical resources', () => {
  // Arrange
  const observed = parts(); observed.ledger = { ...ledger, next_ordinal: 41, last_completed_ordinal: 40, total_charged_ms: 6600000 };
  // Act
  const result = baselineConclusion(observed), proof = currentProof(context, observed, 1000);
  // Assert
  assert.equal(result.complete, true); assert.equal(result.historical_resource_proof, false);
  assert.equal(proof.ledger.next_ordinal, 41); assert.equal(proof.current_v2_idle, true);
});

test('pending accounting, unclosed serial or mismatched boot cannot authorize a current proof', () => {
  for (const mutate of [p => { p.ledger = { ...ledger, pending: true }; }, p => { p.closed.serialOwnershipReleased = false; },
    p => { p.diagnostics = { ...diagnostics, observations: [{ ...diagnostics.observations[0], boot_ordinal: 4 }] }; }]) {
    const value = structuredClone(parts()); mutate(value); assert.throws(() => currentProof(context, value));
  }
});

test('collector always closes after failed accounting and does not issue any effects', async () => {
  // Arrange
  const steps = [], saved = [];
  const gate = { refresh: async () => {}, reviewQualificationAttempts: async () => { throw Error('failed'); },
    reviewBudget: async () => original, exportDiagnostics: async () => {}, stratumV2Possession: async () => ({}),
    stratumV2Status: async () => idle(), stop: async () => steps.push('stop'), close: async () => steps.push('close') };
  const collect = createBaselineCollector({ gate, published: () => state(context, 'before'), campaignId: 'original', save: async (stage, value) => saved.push({ stage, value }) });
  // Act
  const result = await collect();
  // Assert
  assert.deepEqual(steps, ['stop', 'close']); assert.deepEqual(result.failures, ['ledger']);
  assert.equal(saved.at(-1).stage, 'finished'); await assert.rejects(collect(), /consumed/);
});

test('real loopback server denies Start, candidate, grant and self-test routes while retaining full validated diagnostics', async t => {
  // Arrange
  const saved = [], proofs = [];
  const server = createProbeServer({ root: '/unused', context, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} },
    { persist: async (stage, value) => saved.push({ stage, value }), persistProof: async value => proofs.push(value), validateDiagnostics: async value => value });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = async (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  // Act / Assert
  for (const path of ['/grant', '/start', '/candidate', '/core-dump-self-test', '/flash']) assert.equal((await post(path, {})).status, 400);
  assert.equal((await post('/diagnostic-export', diagnostics)).status, 200);
  assert.deepEqual(saved[0].value, diagnostics);
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle(), closed: state(context, 'before', true), finished: { failures: [] } })) {
    assert.equal((await post('/part', { stage, value })).status, 200, stage);
  }
  assert.equal(proofs.length, 1); assert.equal(proofs[0].schema, 'str005-current-recovery-proof-v1');
  assert.equal((await post('/part', { stage: 'ledger', value: ledger })).status, 400);
});

test('stop failure still persists observed closure and marks the collection incomplete', async () => {
  // Arrange
  const saved = [];
  const gate = { refresh: async () => {}, reviewQualificationAttempts: async () => ledger, reviewBudget: async () => original,
    exportDiagnostics: async () => {}, stratumV2Possession: async () => ({}), stratumV2Status: async () => idle(),
    stop: async () => { throw Error('stop'); }, close: async () => {} };
  // Act
  const result = await createBaselineCollector({ gate, published: () => state(context, 'before', true), campaignId: 'original',
    save: async (stage, value) => saved.push({ stage, value }) })();
  // Assert
  assert.deepEqual(result.failures, ['closed']); assert.equal(saved.find(row => row.stage === 'closed').value.serialOwnershipReleased, true);
});

test('candidate transition consumes actual install review and self-test claim is one-use', async t => {
  // Arrange
  const c = { ...context, installEnabled: true, selfTestEnabled: true }, stored = [];
  let installChecks = 0;
  const server = createProbeServer({ root: '/unused', context: c, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} }, {
    persist: async () => {}, persistProof: async () => {}, validateDiagnostics: async value => value,
    verifyNativeAudit: async () => {},
    inspectInstall: async () => { installChecks += 1; return { installation_verified: true }; },
    persistCandidate: async value => stored.push(value), persistClaim: async value => stored.push(value),
    createRecoveryRound: async path => stored.push(path), persistCandidatePart: async (path, value) => stored.push({ path, value }),
    persistCandidateProof: async (path, value) => stored.push({ path, value }), persistCandidateDiagnostics: async () => {},
  });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = async (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  // Act / Assert
  assert.equal((await post('/candidate', {})).status, 400); assert.equal(installChecks, 0);
  await post('/diagnostic-export', diagnostics);
  for (const [stage, value] of Object.entries({ state: state(context, 'before'), ledger, original_budget: original, status: idle(), closed: state(context, 'before', true), finished: { failures: [] } })) await post('/part', { stage, value });
  const response = await post('/candidate', {}); assert.equal(response.status, 200);
  const config = await response.json(); assert.equal(config.expectedAppElfSha256, context.app_elf_sha256); assert.equal(installChecks, 1);
  assert.equal((await post('/candidate', {})).status, 400);
  const input = { state: state(context), ledger, status: idle() };
  const claim = await post('/self-test-claim', input); assert.equal(claim.status, 200);
  assert.equal((await claim.json()).expectedBootOrdinal, 3);
  assert.equal((await post('/self-test-claim', input)).status, 400);
  for (let sequence = 1; sequence <= 8; sequence += 1) {
    const observed = idle(); observed.observation.bootOrdinal = sequence + 2;
    const beginning = await post('/candidate-recovery-begin', { state: state(context), status: observed });
    assert.equal(beginning.status, 200); const round = await beginning.json(); assert.equal(round.sequence, sequence);
    assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: observed })).status, 400);
    await post('/diagnostic-export', { ...diagnostics, observations: [{ ...diagnostics.observations[0], boot_ordinal: sequence + 2 }] });
    for (const [stage, value] of Object.entries({ state: state(context), ledger, original_budget: original, status: observed,
      closed: state(context, 'candidate', true), finished: { failures: [] } })) {
      assert.equal((await post('/candidate-part', { sequence, stage, value })).status, 200, `${sequence}:${stage}`);
    }
    assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: observed })).status, 400);
    assert.ok(stored.some(row => row.path === `candidate-recovery-${String(sequence).padStart(3, '0')}/current-recovery.json`));
  }
  const ninth = idle(); ninth.observation.bootOrdinal = 11;
  assert.equal((await post('/candidate-recovery-begin', { state: state(context), status: ninth })).status, 400);
});

test('current proof independently rejects mismatched published runtime identity despite positive booleans', () => {
  // Arrange / Act / Assert
  for (const key of ['state', 'closed']) {
    const value = parts(); value[key].expectedAppElfSha256 = '0'.repeat(64);
    assert.throws(() => currentProof(context, value), { code: 'panic_current_proof_identity' });
  }
});

test('failed installation without candidate review cannot inherit a successful baseline result', () => {
  // Arrange
  const result = { ...baselineConclusion(parts()), baseline_complete: true };
  // Act
  applyInstallationOutcome(result, { claim: {}, runner: { schema: 'str005-panic-install-runner-v1', code: 1,
    spawn_failed: false, timed_out: false, interrupted: false, serial_holders_absent: true } });
  // Assert
  assert.equal(result.baseline_complete, true); assert.equal(result.complete, false); assert.equal(result.installation_complete, false);
  assert.deepEqual(result.blockers, ['installation_failed']);
});
test('missing installation exit remains blocked independently of the completed baseline', () => {
  const result = { ...baselineConclusion(parts()), baseline_complete: true };
  applyInstallationOutcome(result, { claim: {} });
  assert.equal(result.complete, false); assert.deepEqual(result.blockers, ['installation_result_missing']);
});

test('recovery-only server stays before-phase and denies every effect even after complete baseline', async t => {
  // Arrange
  const c = { ...context, recoveryOnly: true, installEnabled: true, selfTestEnabled: true,
    firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256 };
  const proofs = [];
  const server = createProbeServer({ root: '/unused', context: c, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} },
    { persist: async () => {}, persistProof: async value => proofs.push(value), validateDiagnostics: async value => value,
      inspectInstall: async () => { throw Error('must not inspect effects'); } });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  // Act
  const config = await (await fetch(`${origin}/context`)).json();
  await post('/diagnostic-export', diagnostics);
  for (const [stage, value] of Object.entries({ state: state(c, 'before'), ledger, original_budget: original, status: idle(), closed: state(c, 'before', true), finished: { failures: [] } })) assert.equal((await post('/part', { stage, value })).status, 200);
  // Assert
  assert.equal(config.stratumV2Qualification, 'before'); assert.equal(config.expectedFirmwareSourceCommit, c.firmware_commit);
  assert.equal(proofs.length, 1); assert.equal(proofs[0].source_commit, context.commit); assert.equal(proofs[0].firmware_commit, c.firmware_commit);
  for (const path of ['/candidate', '/self-test-claim', '/self-test-result', '/install', '/start', '/grant', '/candidate-recovery-begin']) assert.equal((await post(path, {})).status, 400);
});

test('recovery-only client never acquires a self-test capability or renders effect controls', async () => {
  // Arrange
  const previous = { window: globalThis.window, document: globalThis.document, fetch: globalThis.fetch };
  const rendered = [], gate = { state: () => ({ status: 'ready', connected: true, running: false }) };
  for (const name of ['refresh', 'reviewQualificationAttempts', 'reviewBudget', 'exportDiagnostics', 'stratumV2Possession', 'stratumV2Status', 'stop', 'close']) gate[name] = async () => {};
  Object.defineProperty(gate, 'coreDumpSelfTest', { configurable: true, get() { throw Error('recovery acquired effect'); } });
  Object.defineProperty(gate, 'exportCoreDumpSelfTestEvidence', { configurable: true, get() { throw Error('recovery acquired self-test evidence'); } });
  globalThis.window = { workerAcceptance: gate };
  globalThis.document = { createElement: () => ({ addEventListener() {}, setAttribute() {} }), getElementById: () => null,
    body: { append: (...elements) => rendered.push(...elements) } };
  globalThis.fetch = async () => ({ ok: true, json: async () => ({ recoveryOnly: true, originalCampaignId: 'fixture' }) });
  try {
    // Act
    await import('./client.mjs?recovery-only-client-test');
    // Assert
    assert.deepEqual(rendered.map(element => element.id), ['capture-panic-baseline', 'panic-probe-result']);
    assert.equal(Object.hasOwn(gate, 'coreDumpSelfTest'), false);
  } finally {
    for (const [key, value] of Object.entries(previous)) { if (value === undefined) delete globalThis[key]; else globalThis[key] = value; }
  }
});


test('successful current-only recovery preserves failed installation and zero historical promotion', () => {
  const result = applyRecoveryOnlyOutcome({ ...baselineConclusion(parts()), baseline_complete: true });
  assert.equal(result.complete, true); assert.equal(result.baseline_complete, true);
  assert.equal(result.installation_complete, false); assert.equal(result.predecessor_installation_failed, true);
  assert.equal(result.self_test_permitted, false); assert.equal(result.core_capture_verified, false);
  assert.equal(result.historical_resource_proof, false); assert.equal(result.parity_promotion, false);
});


test('recovery mode flags cannot be silently ignored on an install action', async () => {
  const { main } = await import('./main.mjs');
  await assert.rejects(main(['install', '--private-root', '/unused', '--recover-install-root', '/sealed']), /preflight_arguments_only/);
  await assert.rejects(main(['preflight', '--private-root', '/unused', '--gate-root', '/gate', '--manifest', '/candidate', '--retained-manifest', '/retained']), /retained_manifest_mode/);
});

test('existing capture requires advancing healthy readiness and exact current identity', async () => {
  const { reviewExistingCapture, requireCurrentCaptureReview } = await import('./capture-existing.mjs');
  const c = { ...context, firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
    captureExisting: true, installEnabled: false, selfTestEnabled: true, corePreservation: preservedCore() };
  const value = structuredClone(parts());
  value.diagnostics.observations.push({ category: 'runtime_identity', firmware_commit: c.firmware_commit, app_elf_sha256: c.app_elf_sha256 },
    { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 100 },
    { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 200 });
  for (const core of [undefined, { ...preservedCore(), empty_core_dump: false }, { ...preservedCore(), expected_boot_ordinal: 4 }, { ...preservedCore(), result_sha256: 'bad' }]) assert.throws(() => reviewExistingCapture(value, { ...c, corePreservation: core }, 1000));
  const other = structuredClone(value); other.diagnostics.observations[0].reset_reason = 'other';
  const otherReview = reviewExistingCapture(other, c, 1000); assert.equal(otherReview.reset_reason_unresolved, true);
  assert.equal(otherReview.core_dump_sha256, c.corePreservation.dump_sha256);
  const review = reviewExistingCapture(value, c, 1000);
  assert.equal(review.reset_reason_unresolved, false);
  assert.equal(review.installation_complete, false); assert.equal(Object.hasOwn(review, 'installation_verified'), false);
  assert.doesNotThrow(() => requireCurrentCaptureReview(review, idle(), 121000));
  assert.throws(() => requireCurrentCaptureReview(review, idle(), 121001));
  assert.throws(() => requireCurrentCaptureReview(review, { observation: { bootOrdinal: 4 } }, 1000));
  for (const mutate of [v => { v.diagnostics.observations.pop(); }, v => { v.diagnostics.observations.at(-1).uptime_ms = 100; },
    v => { v.diagnostics.observations.at(-1).first_failure = 'storage_http'; }, v => { v.diagnostics.observations[1].app_elf_sha256 = '0'.repeat(64); },
    v => { v.diagnostics.observations[0].reset_reason = 'panic'; }, v => { v.diagnostics.observations.push({ category: 'storage_http_failure' }); }, v => { v.diagnostics.observations.push(v.diagnostics.observations.shift()); }, v => { v.ledger = { ...ledger, pending: true }; }]) {
    const bad = structuredClone(value); mutate(bad); assert.throws(() => reviewExistingCapture(bad, c, 1000));
  }
});

test('existing-image capture transitions without install review only after fresh healthy baseline', async t => {
  for (const healthy of [false, true]) {
    const c = { ...context, firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
      captureExisting: true, installEnabled: false, selfTestEnabled: true, corePreservation: preservedCore() }, reviews = [], claims = [];
    const observed = structuredClone(diagnostics);
    observed.observations.push({ category: 'runtime_identity', firmware_commit: c.firmware_commit, app_elf_sha256: c.app_elf_sha256 },
      { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: healthy ? 'none' : 'storage_http', uptime_ms: 100 },
      { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: healthy ? 'none' : 'storage_http', uptime_ms: 200 });
    const server = createProbeServer({ root: '/unused', context: c, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} },
      { persist: async () => {}, persistProof: async () => {}, validateDiagnostics: async value => value, verifyNativeAudit: async () => {},
        persistCapture: async value => reviews.push(value), persistClaim: async value => claims.push(value),
        inspectInstall: async () => { throw Error('installation must not run'); }, persistCandidate: async () => { throw Error('must not synthesize installation review'); } });
    server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
    const origin = `http://127.0.0.1:${server.address().port}`;
    const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
    assert.equal((await post('/self-test-claim', {})).status, 400);
    await post('/diagnostic-export', observed);
    for (const [stage, value] of Object.entries({ state: state(c, 'before'), ledger, original_budget: original, status: idle(), closed: state(c, 'before', true), finished: { failures: [] } })) assert.equal((await post('/part', { stage, value })).status, 200);
    assert.equal((await post('/candidate', {})).status, healthy ? 200 : 400);
    assert.equal((await post('/self-test-claim', { state: state(c), ledger, status: idle() })).status, healthy ? 200 : 400);
    assert.equal(reviews.length, healthy ? 1 : 0); assert.equal(claims.length, healthy ? 1 : 0);
    assert.equal((await post('/install', {})).status, 400);
    if (healthy) assert.equal(reviews[0].installation_complete, false);
  }
});


test('normal, predecessor-before, recovery-only and capture-existing modes select distinct package sources', async () => {
  const { argumentsFor } = await import('./main.mjs');
  const { resolvePreflightSources } = await import('./preflight-selection.mjs');
  for (const mode of ['normal', 'before', 'recovery', 'capture']) {
    const args = ['preflight', '--private-root', '/new', '--gate-root', '/gate'];
    if (mode === 'normal' || mode === 'before') args.push('--manifest', '/canonical');
    if (mode === 'before') args.push('--before-recovery-root', '/before');
    if (mode === 'recovery') args.push('--recover-install-root', '/failed', '--retained-manifest', '/retained');
    if (mode === 'capture') args.push('--capture-recovery-root', '/capture', '--retained-manifest', '/retained');
    const { options } = argumentsFor(args), calls = [], predecessor = { context: { firmware_commit: 'old' } };
    const result = await resolvePreflightSources(options, '/repo', 'current', {
      packageSnapshot: async (_repo, manifest, commit) => { calls.push(['package', manifest, commit]); return { source: 'current' }; },
      beforeRecovery: async root => { calls.push(['before', root]); return { predecessor, root, context: { synthetic: true } }; },
      recoveryPredecessor: async root => { calls.push(['failed', root]); return predecessor; },
      verifyCorePreservation: async (root, value) => { assert.equal(value.synthetic, true); calls.push(['preserved', root]); return preservedCore(); },
      retainedPackage: async (manifest, value) => { assert.equal(value, predecessor); calls.push(['retained', manifest]); return { packaged: { source: 'retained' } }; },
    });
    assert.equal(result.captureExisting, mode === 'capture'); assert.equal(result.recoveryOnly, mode === 'recovery');
    if (mode === 'normal') assert.deepEqual(calls, [['package', '/canonical', 'current']]);
    if (mode === 'before') assert.deepEqual(calls, [['before', '/before'], ['package', '/canonical', 'current']]);
    if (mode === 'recovery') assert.deepEqual(calls, [['failed', '/failed'], ['retained', '/retained']]);
    if (mode === 'capture') assert.deepEqual(calls, [['before', '/capture'], ['retained', '/retained'], ['preserved', '/capture']]);
  }
});


test('capture review cannot refresh an expired baseline or extend its original observation age', async t => {
  for (const candidateExpired of [true, false]) {
  let now = 1000;
  const c = { ...context, firmware_commit: context.before_source.firmware_commit, app_elf_sha256: context.before_source.app_elf_sha256,
    captureExisting: true, installEnabled: false, selfTestEnabled: true, corePreservation: preservedCore() };
  const observed = structuredClone(diagnostics), saved = [];
  observed.observations.push({ category: 'runtime_identity', firmware_commit: c.firmware_commit, app_elf_sha256: c.app_elf_sha256 },
    { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 100 },
    { category: 'startup', stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms: 200 });
  const server = createProbeServer({ root: '/unused', context: c, page: '', bundle: Buffer.from(''), client: Buffer.from(''), trust: {} },
    { now: () => now, persist: async () => {}, persistProof: async () => {}, validateDiagnostics: async value => value,
      verifyNativeAudit: async () => {}, persistCapture: async value => saved.push(value) });
  server.listen(0, '127.0.0.1'); await once(server, 'listening'); t.after(() => server.release());
  const origin = `http://127.0.0.1:${server.address().port}`;
  const post = (path, value) => fetch(`${origin}${path}`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' }, body: JSON.stringify(value) });
  await post('/diagnostic-export', observed);
  for (const [stage, value] of Object.entries({ state: state(c, 'before'), ledger, original_budget: original, status: idle(), closed: state(c, 'before', true), finished: { failures: [] } })) assert.equal((await post('/part', { stage, value })).status, 200);
  now = candidateExpired ? 121001 : 120000;
  assert.equal((await post('/candidate', {})).status, candidateExpired ? 400 : 200); assert.equal(saved.length, candidateExpired ? 0 : 1);
  if (!candidateExpired) assert.equal(saved[0].observed_at_unix_ms, 1000);
  now = 121001;
  assert.equal((await post('/self-test-claim', { state: state(c), ledger, status: idle() })).status, 400);
  }
});

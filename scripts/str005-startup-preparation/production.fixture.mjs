// Actual Gate restart adapter, page operation, observer, preservation and parser; fake serial wire only.
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { restartEvidence } from './model.mjs';
import { restartConfiguration } from './server.mjs';
const gateRoot = process.argv[2], load = name => import(pathToFileURL(resolve(gateRoot, `web/${name}.ts`)).href);
const { serialHarness } = await load('worker-serial.test-support');
const { createWebSerialWorkerController, workerSerialQualificationHook } = await load('webserial-worker-controller');
const { createWorkerRestartPageOperations } = await load('worker-restart-page');
const { WorkerPreservationBaseline } = await load('worker-preservation');
const { parseWorkerDiagnosticExport } = await load('worker-diagnostic-export');
const { parseWorkerSerialAcceptanceConfiguration, requireWorkerAcceptanceModeTransition } = await load('worker-serial-acceptance-config');
const { parseWorkerDeploymentTrust } = await load('worker-deployment-trust');
const trustModule = await import(pathToFileURL(resolve(gateRoot, 'conformance/bwg-worker-deployment-trust-0.2/trust.json')).href);
const context = { firmware_commit: 'a'.repeat(40), app_elf_sha256: 'b'.repeat(64), gate_commit: 'c'.repeat(40), gate_root: gateRoot };
const config = restartConfiguration(context, parseWorkerDeploymentTrust(trustModule.default));
const parsed = parseWorkerSerialAcceptanceConfiguration(config, context.gate_commit);
assert.equal(parsed.restartQualification, true); assert.equal(parsed.stratumV2Qualification, undefined);
const pair = { firmwareSourceCommit: context.firmware_commit, appElfSha256: context.app_elf_sha256 };
assert.throws(() => parseWorkerSerialAcceptanceConfiguration({ ...config, stratumV2Qualification: 'before', stratumV2Scope: 'share',
  stratumV2Identities: { before: pair, candidate: pair } }, context.gate_commit), /configuration_invalid/u);
assert.throws(() => requireWorkerAcceptanceModeTransition(parsed, { expectedGateCommit: context.gate_commit,
  expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256, trust: config.trust }));
for (const mode of ['same_stream', 'reopen_once']) {
  const h = await serialHarness(); h.setRestartScenario(mode); const baseline = new WorkerPreservationBaseline();
  const controller = createWebSerialWorkerController({ ...h.input, [workerSerialQualificationHook]: {
    suppressHeartbeats: false, memoryOnlyContinuity: true, allowQualificationRestart: true, observePreservation: value => baseline.observe(value) } });
  await controller.requestPermission(); const before = baseline.maybePublicState();
  const page = createWorkerRestartPageOperations({ enabled: () => true, idle: () => true, maybeController: () => controller,
    before() {}, succeeded() {}, failed() {} });
  const request = { requestNonce: 'A'.repeat(22), expectedBootOrdinal: 1 };
  try {
    await page.qualificationRestart(request); const evidence = page.exportRestartEvidence();
    const checked = await restartEvidence(evidence, context, request, { validateDiagnostics: async value => parseWorkerDiagnosticExport(value) });
    assert.equal(checked.summary.softwareResetObserved, true); assert.equal(checked.summary.nextBootOrdinal, 2);
    assert.equal(checked.summary.portReopens, mode === 'same_stream' ? 0 : 1);
    await controller.status(); assert.deepEqual(baseline.maybePublicState(), before);
    await assert.rejects(page.qualificationRestart({ ...request, expectedBootOrdinal: 2 }));
    assert.equal(h.received.filter(row => row.command === 'qualification_restart').length, 1);
    assert.equal(h.received.some(row => ['start_lease', 'renew_lease', 'qualification_core_dump_self_test', 'stratum_v2_channel_start'].includes(row.command)), false);
  } finally { await controller.close(); }
  assert.equal(h.counts().locked, false);
}
process.stdout.write('preparation_production_boundary_passed\n');

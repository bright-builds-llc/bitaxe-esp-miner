import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat, realpath, readdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { BUNDLE, PAGE, cleanPushed, git, ignored, missing, protectedPath, packageSnapshot, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { inventory, privateRoot, proof, verifyInventory, writeNew, retain } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireLsofAbsent, requireNoHolders, sameProcess } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector, cleanupPorts } from './detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { validateSelfTest } from './self-test-evidence.mjs';
import { main as runNativeAudit } from '../core-dump/native-audit.mjs';
import { validateNativeAudit, verifyNativeAudit } from './audit.mjs';
import { install } from './install.mjs';
import { createProbeServer } from './server.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { BASELINE_PARTS, baselineConclusion, validatePart, validateFinished, validateCandidateState, currentProof } from './model.mjs';

const task = 'task-str005-start-panic-diagnosis';
const oldSeal = '14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950';
const oldContext = 'a453de1753acc78bfa3eaeebfd9f42339528ec4fb41704fd50d390a1cd1ff5c4';
const contract = 'docs/hardware/str005-panic-probe.md';
async function source(root) {
  const commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, commit);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8');
  const active = tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '';
  check(active.includes(`### ${task} |`) && tasks.split(`### ${task} |`).length === 2, 'panic_task_inactive');
  const block = active.split(`### ${task} |`)[1]?.split(/^### /mu)[0] ?? '';
  const lines = block.split(/\r?\n/u).map(line => line.trim());
  check(lines.includes('Development panic probe: stage A enabled.'), 'panic_stage_a_disabled');
  return { commit,
    selfTestEnabled: lines.includes('Development panic probe: self-test enabled.'),
    installEnabled: lines.includes('Development panic probe: installation enabled.'),
    contractSha256: sha256(await readFile(resolve(root, contract))) };
}
async function predecessor(root) {
  const oldRoot = resolve(root, 'scratch/str005-v2-serial/share-002');
  const sealed = await proof(oldRoot, 'sealed-inventory.json');
  check(sealed.sha256 === oldSeal, 'panic_predecessor_seal');
  await verifyInventory(oldRoot, sealed.value.files, new Set(['sealed-inventory.json', 'projection.json']));
  const wrapped = (await proof(oldRoot, 'context.json')).value;
  check(wrapped.sha256 === oldContext && sha256(JSON.stringify(wrapped.context)) === oldContext, 'panic_predecessor_context');
  const detector = (await proof(oldRoot, 'install-4.claim.json')).value.detector;
  return { before: wrapped.context, detector };
}
function argumentsFor(argv) {
  const [action, ...rest] = argv, options = {};
  check(['preflight', 'serve', 'finish', 'install'].includes(action), 'panic_action');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(['--private-root', '--gate-root', '--manifest', '--flash-binary'].includes(key) && !Object.hasOwn(options, key) && typeof value === 'string' && resolve(value) === value, 'panic_arguments');
    options[key] = value;
  }
  check(options['--private-root'] && (action !== 'preflight' || (options['--gate-root'] && options['--manifest'])), 'panic_arguments');
  return { action, options };
}
export async function main(argv) {
  const { action, options } = argumentsFor(argv), root = options['--private-root'];
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  ignored(firmwareRoot, root);
  const published = await source(firmwareRoot), prior = await predecessor(firmwareRoot);
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root));
    const gateRoot = options['--gate-root'], gateCommit = git(gateRoot, ['rev-parse', 'HEAD']);
    cleanPushed(gateRoot, gateCommit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === gateCommit, 'panic_gate_pin');
    const packaged = await packageSnapshot(firmwareRoot, options['--manifest'], published.commit);
    const page = await readFile(resolve(gateRoot, PAGE)), bundle = await readFile(resolve(gateRoot, BUNDLE));
    check([gateCommit, 'stratumV2Status', 'coreDumpSelfTestQualification', 'coreDumpSelfTest'].every(marker => bundle.includes(marker)), 'panic_gate_capability');
    const trust = JSON.parse(await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json'), 'utf8'));
    const flashBinary = await realpath(resolve(firmwareRoot, 'bazel-bin/tools/flash/flash'));
    check(!options['--flash-binary'] || await realpath(options['--flash-binary']) === flashBinary, 'panic_flash_binary');
    const context = { schema: 'str005-panic-probe-v1', ...published, ...packaged, firmware_commit: published.commit,
      firmware_root: firmwareRoot, manifest: options['--manifest'], flashBinary, flashBinarySha256: await fileDigest(flashBinary), gate_root: gateRoot, gate_commit: gateCommit, scope: 'share',
      before_source: { firmware_commit: prior.before.firmware_commit, app_elf_sha256: prior.before.app_elf_sha256 },
      original_campaign_id: prior.before.original_campaign_id, attemptId: prior.before.attemptId,
      detector: prior.detector, predecessorSeal: oldSeal, predecessorContext: oldContext,
      gatePageSha256: sha256(page), gateBundleSha256: sha256(bundle), trustSha256: sha256(JSON.stringify(trust)) };
    await mkdir(root, { mode: 0o700 });
    context.candidateElf = resolve(dirname(options['--manifest']), 'bitaxe-ultra205.elf');
    const nativeAudit = await runNativeAudit(['--elf', context.candidateElf, '--output', resolve(root, 'native-audit.json')]);
    validateNativeAudit(nativeAudit, context.app_elf_sha256);
    context.nativeAuditSha256 = await fileDigest(resolve(root, 'native-audit.json'));
    await writeNew(resolve(root, 'context.json'), context);
    await retain(resolve(root, 'gate-page.html'), page); await retain(resolve(root, 'gate-bundle.js'), bundle);
    return { preflight: 'passed', stage: 'baseline', device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-panic-probe-v1' && context.commit === published.commit && context.contractSha256 === published.contractSha256 &&
    context.predecessorSeal === oldSeal && context.predecessorContext === oldContext, 'panic_source_changed');
  cleanPushed(context.gate_root, context.gate_commit);
  if (action === 'finish') return finish(root, context);
  await verifyNativeAudit(root, context);
  if (action === 'install') { check(published.installEnabled && context.installEnabled, 'panic_install_disabled'); return install(root, context); }
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'panic_protected_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'panic_distinct_output');
  await missing(resolve(root, 'server-owner.json'));
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detected = await readFile(detectorPath, 'utf8');
  const currentDetector = parseDetector(detected, context.detector.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(currentDetector.port);
  const runtimeContext = { ...context, detector: { ...context.detector, ...currentDetector } };
  const page = await readFile(resolve(root, 'gate-page.html')), bundle = await readFile(resolve(root, 'gate-bundle.js'));
  check(sha256(page) === context.gatePageSha256 && sha256(bundle) === context.gateBundleSha256, 'panic_asset_changed');
  const trust = JSON.parse(await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json'), 'utf8'));
  check(sha256(JSON.stringify(trust)) === context.trustSha256, 'panic_trust_changed');
  const server = createProbeServer({ root, context: runtimeContext, page, bundle, trust, client: await readFile(resolve(firmwareRoot, 'scripts/str005-panic-probe/client.mjs')) });
  const done = new Promise(resolveDone => server.once('close', resolveDone)); let maybeRelease;
  const stop = () => { maybeRelease ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'panic_owner_missing');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port, serialPort: currentDetector.port, physicalIdentitySha256: currentDetector.physical,
      detectorSha256: sha256(detected), startedAtUnixMs: Date.now() });
    process.stdout.write(`panic_probe_url=http://127.0.0.1:${server.address().port}/\n`); await done;
  } finally { stop(); await maybeRelease; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
async function finish(root, context) {
  const server = (await proof(root, 'server-owner.json')).value, current = await processSnapshot();
  check(!current.some(row => sameProcess(row, server.owner) || row.ppid === server.owner.pid), 'panic_server_live');
  requireLsofAbsent(['-nP', `-iTCP:${server.port}`, '-sTCP:LISTEN', '-t']);
  const detectorPath = resolve(dirname(root), 'cleanup-detector.stdout.log'); await protectedPath(detectorPath);
  const fresh = parseDetector(await readFile(detectorPath, 'utf8'), context.detector.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  let maybeInstallPort;
  try { maybeInstallPort = (await proof(root, 'install-claim.json')).value.port; }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  for (const port of cleanupPorts(fresh.port, server.serialPort, maybeInstallPort)) requireNoHolders(port);
  const parts = {};
  for (const stage of [...BASELINE_PARTS, 'finished']) {
    try {
      const value = (await proof(root, `baseline-${stage}.json`)).value;
      parts[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) :
        stage === 'finished' ? validateFinished(value) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status : validatePart(stage, value, context);
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const result = { ...baselineConclusion(parts), host_resources_released: true, predecessor_unchanged: true };
  if (result.complete) {
    try { check((await proof(root, 'current-recovery.json')).value.schema === 'str005-current-recovery-proof-v1', 'panic_current_proof_missing'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; result.complete = false; result.blockers.push('missing_current_proof'); }
  }
  result.baseline_complete = result.complete;
  result.core_capture_verified = false;
  try {
    await proof(root, 'candidate-install-review.json');
    result.candidate_install_reviewed = true;
    const rounds = (await readdir(root)).filter(name => /^candidate-recovery-[0-9]{3}$/u.test(name)).sort();
    check(rounds.length <= 8 && rounds.every((name, index) => name === `candidate-recovery-${String(index + 1).padStart(3, '0')}`), 'panic_recovery_round_inventory');
    result.candidate_recoveries = [];
    let maybeRequest;
    try { maybeRequest = (await proof(root, 'self-test-claim.json')).value.request; } catch (error) { if (error.code !== 'ENOENT') throw error; }
    for (const round of rounds) {
      const collected = {}, candidateContext = { ...context, before_source: context };
      for (const stage of [...BASELINE_PARTS, 'finished']) {
        try {
          const value = (await proof(root, `${round}/${stage}.json`)).value;
          collected[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) :
            stage === 'finished' ? validateFinished(value) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status :
              ['state', 'closed'].includes(stage) ? validateCandidateState(value, context, maybeRequest) : validatePart(stage, value, candidateContext);
        } catch (error) { if (error.code !== 'ENOENT') throw error; }
      }
      const assessed = baselineConclusion(collected);
      try {
        const saved = (await proof(root, `${round}/current-recovery.json`)).value;
        check(JSON.stringify(saved) === JSON.stringify(currentProof(candidateContext, collected, saved.observed_at_unix_ms)), 'panic_candidate_proof_changed');
      } catch (error) { if (error.code !== 'ENOENT') throw error; assessed.complete = false; assessed.blockers.push('missing_current_proof'); }
      result.candidate_recoveries.push({ round, ...assessed });
      if (!assessed.complete) { result.complete = false; result.blockers.push(`${round}_incomplete`); }
    }
    if (rounds.length === 0) { result.complete = false; result.blockers.push('candidate_recovery_missing'); }

    try {
      const evidence = await validateSelfTest((await proof(root, 'self-test-result.json')).value, context.gate_root,
        (await proof(root, 'self-test-claim.json')).value.request, context);
      result.self_test_reset_observed = evidence.summary?.stage === 'complete' && evidence.summary?.panicResetObserved === true;
      if (!result.self_test_reset_observed) { result.complete = false; result.blockers.push('self_test_reset_incomplete'); }
    } catch (error) { if (error.code !== 'ENOENT') throw error; result.complete = false; result.blockers.push('self_test_result_missing'); }
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify(value)}\n`)).catch(() => {
    process.stdout.write('{"error":"panic_operation_rejected"}\n'); process.exitCode = 1;
  });
}

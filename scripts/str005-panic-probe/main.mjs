import { applyReadinessFailure } from './readiness.mjs';
import { applyCandidateFailure, sealProbeResult } from './candidate-failure.mjs';
import { main as auditSignedStart } from '../audit-signed-start-stack.mjs';
import { main as auditSignedRenew } from '../audit-signed-renew-stack.mjs';
import { renewSource, startupPredecessor } from './renew-successor.mjs';
import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat, realpath, readdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { BUNDLE, PAGE, cleanPushed, git, ignored, missing, protectedPath, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { inventory, privateRoot, proof, verifyInventory, writeNew, retain } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireLsofAbsent, requireNoHolders, sameProcess } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector, cleanupPorts } from './detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { validateSelfTest } from './self-test-evidence.mjs';
import { main as runNativeAudit } from '../core-dump/native-audit.mjs';
import { main as runStoreAudit } from '../core-dump/store-audit.mjs';
import { validateNativeAudit, verifyNativeAudit, validateStoreAudit } from './audit.mjs';
import { checkCommand, verifyCommandCheck } from './command-check.mjs';
import { install, admitRecovery, INSTALL_TIMEOUT_MS } from './install.mjs';
import { resolvePreflightSources } from './preflight-selection.mjs';
import { recoveryPredecessor, retainedPackage, beforeRecovery } from './recovery-predecessor.mjs';
import { recoveryStoreObservation } from './store-diagnostics.mjs';
import { installedPredecessor } from './installed-predecessor.mjs';
import { verifyCorePreservation } from './core-preservation.mjs';
import { validateCaptureDiagnosticPair } from './capture-diagnostics.mjs';
import { reviewExistingCapture } from './capture-existing.mjs';
import { createProbeServer } from './server.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { BASELINE_PARTS, baselineConclusion, validatePart, validateFinished, validateCandidateState, currentProof, applyInstallationOutcome, applyRecoveryOnlyOutcome, applySelfTestScope } from './model.mjs';

const task = 'task-str005-start-panic-diagnosis';
const oldSeal = '14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950';
const oldContext = 'a453de1753acc78bfa3eaeebfd9f42339528ec4fb41704fd50d390a1cd1ff5c4';
const contract = 'docs/hardware/str005-panic-probe.md';
async function source(root, renewSuccessor = false) {
  const commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, commit);
  if (renewSuccessor) return { commit, ...await renewSource(root) };
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8');
  const active = tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '';
  check(active.includes(`### ${task} |`) && tasks.split(`### ${task} |`).length === 2, 'panic_task_inactive');
  const block = active.split(`### ${task} |`)[1]?.split(/^### /mu)[0] ?? '';
  const lines = block.split(/\r?\n/u).map(line => line.trim());
  check(lines.includes('Development panic probe: stage A enabled.'), 'panic_stage_a_disabled');
  return { commit,
    selfTestEnabled: lines.includes('Development panic probe: self-test enabled.'),
    installEnabled: lines.includes('Development panic probe: installation enabled.'),
    storeAuditRequired: lines.includes('Development panic probe: store diagnostics required.'),
    cutoffUserRegionRequired: lines.includes('Development panic probe: task-stack capture required.'),
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
  return { before: wrapped.context, detector, seal: oldSeal, contextDigest: oldContext };
}
export function argumentsFor(argv) {
  const [requestedAction, ...rest] = argv, options = {};
  const renewSuccessor = requestedAction?.startsWith('renew-') === true;
  const action = renewSuccessor ? requestedAction.slice(6) : requestedAction;
  check(['preflight', 'serve', 'finish', 'install'].includes(action), 'panic_action');
  for (let i = 0; i < rest.length; i += 2) {
    const key = rest[i], value = rest[i + 1];
    check(['--private-root', '--gate-root', '--manifest', '--flash-binary', '--recover-install-root', '--recover-installed-root', '--retained-manifest', '--before-recovery-root', '--capture-recovery-root'].includes(key) && !Object.hasOwn(options, key) && typeof value === 'string' && resolve(value) === value, 'panic_arguments');
    options[key] = value;
  }
  const retainedModes = ['--recover-install-root', '--recover-installed-root', '--capture-recovery-root'].filter(key => options[key]);
  check(retainedModes.length <= 1, 'panic_exclusive_retained_mode');
  const retainedMode = retainedModes.length === 1;
  check(options['--private-root'] && (action !== 'preflight' || (options['--gate-root'] && (retainedMode ? options['--retained-manifest'] &&
    !options['--manifest'] && !options['--flash-binary'] && !options['--before-recovery-root'] && !(options['--recover-install-root'] && options['--capture-recovery-root']) : options['--manifest']))), 'panic_arguments');
  check(action === 'preflight' || Object.keys(options).length === 1, 'panic_preflight_arguments_only');
  check(!options['--retained-manifest'] || retainedMode, 'panic_retained_manifest_mode');
  return { action, options, ...(renewSuccessor ? { renewSuccessor } : {}) };
}
export async function main(argv) {
  if (argv[0]?.startsWith('renew-clear')) return (await import('./renew-clear.mjs')).renewClearMain(argv);
  const { action, options, renewSuccessor = false } = argumentsFor(argv), root = options['--private-root'];
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  ignored(firmwareRoot, root);
  const published = await source(firmwareRoot, renewSuccessor), prior = renewSuccessor ? await startupPredecessor(firmwareRoot) : await predecessor(firmwareRoot);
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root));
    const gateRoot = options['--gate-root'], gateCommit = git(gateRoot, ['rev-parse', 'HEAD']);
    cleanPushed(gateRoot, gateCommit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === gateCommit, 'panic_gate_pin');
    const { corePreservation, recoveryOnly, captureExisting, before, failedInstall, installed, retainedSource, retained, packaged } = await resolvePreflightSources(options, firmwareRoot, published.commit);
    check(!renewSuccessor || retainedSource || !before || before.kind === 'partial_verified_write', 'renew_before_mode');
    check(!renewSuccessor || !published.selfTestEnabled || captureExisting || recoveryOnly, 'renew_capture_requires_preserved_image');
    const page = await readFile(resolve(gateRoot, PAGE)), bundle = await readFile(resolve(gateRoot, BUNDLE));
    check([gateCommit, 'stratumV2Status', 'coreDumpSelfTestQualification', 'coreDumpSelfTest', ...(published.storeAuditRequired ? ['core_dump_store_receipt'] : [])].every(marker => bundle.includes(marker)), 'panic_gate_capability');
    const trust = JSON.parse(await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json'), 'utf8'));
    const flashBinary = retainedSource ? null : await realpath(resolve(firmwareRoot, 'bazel-bin/tools/flash/flash'));
    check(!options['--flash-binary'] || await realpath(options['--flash-binary']) === flashBinary, 'panic_flash_binary');
    const context = { schema: 'str005-panic-probe-v1', ...published, ...packaged, firmware_commit: retainedSource?.context.firmware_commit ?? published.commit,
      ...(renewSuccessor && !retainedSource && !before ? { retainedBaseline: prior.retainedBaseline } : {}),
      recoveryOnly, captureExisting, storeAuditRequired: published.storeAuditRequired && !recoveryOnly, cutoffUserRegionRequired: published.cutoffUserRegionRequired && !recoveryOnly, ...(captureExisting ? { corePreservation } : {}), ...(retainedSource ? { installEnabled: false, selfTestEnabled: captureExisting && published.selfTestEnabled, continuity_basis: 'current-session-only', retainedManifest: retained.retainedManifest } : {}),
      ...(failedInstall ? { failedInstall: { root: failedInstall.root, seal_sha256: failedInstall.seal_sha256, context_sha256: failedInstall.context_sha256 } } : {}),
      ...(installed ? { installedAnchor: { root: installed.root, seal_sha256: installed.seal_sha256, context_sha256: installed.context_sha256, candidate_proof_sha256: installed.candidate_proof_sha256 } } : {}),
      ...(before ? { beforeRecovery: { root: before.root, seal_sha256: before.seal_sha256, context_sha256: before.context_sha256 } } : {}),
      ...(retainedSource ? {} : { installTimeoutMs: INSTALL_TIMEOUT_MS }),
      firmware_root: firmwareRoot, manifest: options['--manifest'], flashBinary, flashBinarySha256: flashBinary ? await fileDigest(flashBinary) : null, gate_root: gateRoot, gate_commit: gateCommit, scope: 'share',
      before_source: retainedSource ? { firmware_commit: retainedSource.context.firmware_commit, app_elf_sha256: retainedSource.context.app_elf_sha256 } : before?.context.before_source ?? { firmware_commit: prior.before.firmware_commit, app_elf_sha256: prior.before.app_elf_sha256 },
      original_campaign_id: prior.before.original_campaign_id, attemptId: prior.before.attemptId,
      detector: retainedSource?.context.detector ?? before?.context.detector ?? prior.detector, predecessorSeal: prior.seal, predecessorContext: prior.contextDigest,
      gatePageSha256: sha256(page), gateBundleSha256: sha256(bundle), trustSha256: sha256(JSON.stringify(trust)) };
    await mkdir(root, { mode: 0o700 });
    context.candidateElf = retained?.candidateElf ?? resolve(dirname(options['--manifest']), 'bitaxe-ultra205.elf');
    if (retainedSource) await retain(resolve(root, 'native-audit.json'), retainedSource.audit.bytes);
    if (installed) await writeNew(resolve(root, 'installed-anchor-review.json'), { ...installed.review, ...context.installedAnchor });
    if (failedInstall) {
      await writeNew(resolve(root, 'failed-install-review.json'), { ...failedInstall.review, seal_sha256: failedInstall.seal_sha256, claim_sha256: failedInstall.claim_sha256, runner_sha256: failedInstall.runner_sha256, receipt_sha256: failedInstall.receipt_sha256, log_sha256: failedInstall.log_sha256, stdout_sha256: failedInstall.stdout_sha256 });
    }
    const nativeAudit = retainedSource ? retainedSource.audit.value : await runNativeAudit(['--elf', context.candidateElf, '--output', resolve(root, 'native-audit.json')]);
    validateNativeAudit(nativeAudit, context.app_elf_sha256, context.cutoffUserRegionRequired);
    context.nativeAuditSha256 = await fileDigest(resolve(root, 'native-audit.json'));
    if (renewSuccessor) {
      context.signedPathAudits = {};
      for (const [name, audit] of [['start', auditSignedStart], ['renew', auditSignedRenew]]) {
        const path = resolve(root, `native-${name}-audit.json`);
        const result = await audit(['--elf', context.candidateElf, '--output', path]);
        check(result.result === 'selected_path_with_headroom' && result.elf_sha256 === context.app_elf_sha256, 'renew_native_path_audit');
        context.signedPathAudits[name] = await fileDigest(path);
      }
    }
    if (context.storeAuditRequired) {
      const store = await runStoreAudit(['--elf', context.candidateElf, '--output', resolve(root, 'store-audit.json')]);
      validateStoreAudit(store, context.app_elf_sha256);
      context.storeAuditSha256 = await fileDigest(resolve(root, 'store-audit.json'));
    }
    if (!retainedSource) context.commandCheckSha256 = (await checkCommand(root, context)).runner_sha256;
    await writeNew(resolve(root, 'context.json'), context);
    await retain(resolve(root, 'gate-page.html'), page); await retain(resolve(root, 'gate-bundle.js'), bundle);
    return { preflight: 'passed', stage: 'baseline', device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-panic-probe-v1' && context.commit === published.commit && context.contractSha256 === published.contractSha256 &&
    context.predecessorSeal === prior.seal && context.predecessorContext === prior.contextDigest && Boolean(context.renewSuccessor) === renewSuccessor, 'panic_source_changed');
  cleanPushed(context.gate_root, context.gate_commit);
  if (context.recoveryOnly || context.captureExisting) {
    const anchor = context.installedAnchor ?? context.failedInstall;
    check(anchor && !(context.installedAnchor && context.failedInstall), 'panic_recovery_anchor');
    const predecessor = context.installedAnchor ? await installedPredecessor(anchor.root, firmwareRoot) : await recoveryPredecessor(anchor.root, firmwareRoot);
    check(predecessor.seal_sha256 === anchor.seal_sha256 && predecessor.context_sha256 === anchor.context_sha256, 'panic_recovery_predecessor_changed');
    if (context.installedAnchor) check(predecessor.candidate_proof_sha256 === anchor.candidate_proof_sha256, 'panic_installed_anchor_proof_changed');
    await retainedPackage(context.retainedManifest, predecessor, firmwareRoot);
    check(context.installEnabled === false && (!context.recoveryOnly || context.selfTestEnabled === false) && action !== 'install', 'panic_recovery_only');
  }
  if (context.beforeRecovery) {
    const before = await beforeRecovery(context.beforeRecovery.root, firmwareRoot);
    check(before.seal_sha256 === context.beforeRecovery.seal_sha256 && before.context_sha256 === context.beforeRecovery.context_sha256, 'panic_before_recovery_changed');
    if (context.captureExisting) check(JSON.stringify(await verifyCorePreservation(before.root, before.context)) === JSON.stringify(context.corePreservation), 'panic_core_preservation_changed');
  }
  if (action === 'finish') return finish(root, context);
  if (renewSuccessor) for (const name of ['start', 'renew']) {
    const saved = await proof(root, `native-${name}-audit.json`);
    check(saved.sha256 === context.signedPathAudits?.[name] && saved.value.result === 'selected_path_with_headroom' &&
      saved.value.elf_sha256 === context.app_elf_sha256, 'renew_native_path_audit');
  }
  await verifyNativeAudit(root, context);
  if (!context.recoveryOnly && !context.captureExisting) await verifyCommandCheck(root, context);
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
  const server = createProbeServer({ root, context: runtimeContext, page, bundle, trust, client: await readFile(resolve(firmwareRoot, 'scripts/str005-panic-probe/client.mjs')), readinessClient: await readFile(resolve(firmwareRoot, 'scripts/str005-panic-probe/self-test-readiness.mjs')) }, {
    verifyEffect: async () => {
      const current = await source(firmwareRoot, renewSuccessor);
      check(JSON.stringify(current) === JSON.stringify(published), 'renew_source_changed');
      cleanPushed(context.gate_root, context.gate_commit);
    },
  });
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
export async function finish(root, context) {
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
        stage === 'finished' ? validateFinished(value, context) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status : validatePart(stage, value, context);
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  if (context.captureExisting) {
    try {
      parts.diagnostics_confirmation = await validateDiagnosticExport((await proof(root, 'baseline-diagnostics_confirmation.json')).value, context.gate_root);
      const confirmation = (await proof(root, 'baseline-diagnostics_confirmation_status.json')).value;
      validatePart('state', confirmation.state, context);
      parts.diagnostics_confirmation_status = { state: confirmation.state, status: validateRecoveryParts({ status: confirmation.status }, context).status };
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const result = { ...baselineConclusion(parts, context), ...(context.renewSuccessor ? { first_failure: parts.finished?.first_failure ?? null } : {}), host_resources_released: true, predecessor_unchanged: true };
  if (context.captureExisting) {
    try { validateCaptureDiagnosticPair(parts, context); }
    catch { result.complete = false; result.blockers.push('capture_diagnostics_pair_incomplete'); }
  }

  if (result.complete) {
    try { check((await proof(root, 'current-recovery.json')).value.schema === (context.retainedBaseline ? 'str005-current-recovery-proof-v2' : 'str005-current-recovery-proof-v1'), 'panic_current_proof_missing'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; result.complete = false; result.blockers.push('missing_current_proof'); }
  }
  result.baseline_complete = result.complete;
  try { applyCandidateFailure(result, (await proof(root, 'candidate-recovery-failure.json')).value); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  await applyReadinessFailure(root, result);
  result.core_capture_verified = false;
  if (context.recoveryOnly) {
    applyRecoveryOnlyOutcome(result, context.predecessorInstallationFailed ?? (context.installedAnchor === undefined));
    if (result.complete) {
      const current = (await proof(root, 'current-recovery.json')).value;
      check(JSON.stringify(current) === JSON.stringify(currentProof(context, parts, current.observed_at_unix_ms)), 'panic_recovery_proof_changed');
      admitRecovery(current, context, current.observed_at_unix_ms);
    }
    return sealProbeResult(root, result);
  }
  if (context.captureExisting) {
    result.continuity_basis = 'current-session-only'; result.installation_complete = false;
    result.predecessor_installation_failed = context.installedAnchor === undefined;
  }
  const installation = {};
  for (const [key, name] of [['claim', 'install-claim.json'], ['runner', 'install-runner.json'], ['review', 'candidate-install-review.json']]) {
    try { installation[key] = (await proof(root, name)).value; } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  applyInstallationOutcome(result, installation);
  try {
    const review = (await proof(root, context.captureExisting ? 'capture-review.json' : 'candidate-install-review.json')).value;
    if (context.captureExisting) {
      check(JSON.stringify(review) === JSON.stringify(reviewExistingCapture(parts, context, review.observed_at_unix_ms)), 'panic_capture_review_changed');
      result.capture_admission_reviewed = true;
    }
    else result.candidate_install_reviewed = true;
    const rounds = (await readdir(root)).filter(name => /^candidate-recovery-[0-9]{3}$/u.test(name)).sort();
    check(rounds.length <= 8 && rounds.every((name, index) => name === `candidate-recovery-${String(index + 1).padStart(3, '0')}`), 'panic_recovery_round_inventory');
    result.candidate_recoveries = [];
    let maybeRequest;
    try { maybeRequest = (await proof(root, 'self-test-claim.json')).value.request; } catch (error) { if (error.code !== 'ENOENT') throw error; }
    for (const round of rounds) {
      const collected = {}, candidateContext = { ...context, retainedBaseline: undefined, before_source: context };
      for (const stage of [...BASELINE_PARTS, 'finished']) {
        try {
          const value = (await proof(root, `${round}/${stage}.json`)).value;
          collected[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) :
            stage === 'finished' ? validateFinished(value, context) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status :
              ['state', 'closed'].includes(stage) ? validateCandidateState(value, context, maybeRequest) : validatePart(stage, value, candidateContext);
        } catch (error) { if (error.code !== 'ENOENT') throw error; }
      }
      const assessed = baselineConclusion(collected, candidateContext);
      try {
        const saved = (await proof(root, `${round}/current-recovery.json`)).value;
        if (context.renewSuccessor) {
          const begin = (await proof(root, `${round}/collection-begin.json`)).value;
          check(begin.schema === 'str005-renew-candidate-begin-v1' && begin.startedAtUnixMs === saved.observed_at_unix_ms, 'renew_candidate_collection_binding');
        }
        check(JSON.stringify(saved) === JSON.stringify(currentProof(candidateContext, collected, saved.observed_at_unix_ms)), 'panic_candidate_proof_changed');
      } catch (error) { if (error.code !== 'ENOENT') throw error; assessed.complete = false; assessed.blockers.push('missing_current_proof'); }
      const measured = context.storeAuditRequired ? recoveryStoreObservation(collected.diagnostics, collected.status, context, maybeRequest) : undefined;
      if (measured) {
        let maybeSaved;
        try { maybeSaved = (await proof(root, `${round}/store-observation.json`)).value; }
        catch (error) { if (error.code !== 'ENOENT') throw error; }
        if (!maybeSaved) { result.complete = false; result.blockers.push(`${round}_store_observation_missing`); }
        else check(JSON.stringify(maybeSaved) === JSON.stringify(measured), 'panic_store_observation_changed');
        assessed.store_outcome = maybeSaved ? measured.outcome : 'receipt_unavailable';
        if (measured.outcome === 'receipt_unavailable') { result.complete = false; result.blockers.push(`${round}_store_diagnostics_missing`); }
      }
      result.candidate_recoveries.push({ round, ...assessed });
      if (!assessed.complete) { result.complete = false; result.blockers.push(`${round}_incomplete`); }
    }
    if (rounds.length === 0) { result.complete = false; result.blockers.push('candidate_recovery_missing'); }
    if (context.storeAuditRequired && maybeRequest && !result.candidate_recoveries.some(row => row.store_outcome && !['receipt_unavailable','outside_panic_boot'].includes(row.store_outcome))) {
      result.complete = false; result.blockers.push('store_diagnostics_missing');
    }

    await finalizeSelfTestEvidence(root, context, result);
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (context.captureExisting && !result.capture_admission_reviewed) { result.complete = false; result.blockers.push('capture_review_missing'); }
  return sealProbeResult(root, result);
}
/** Re-read immutable artifacts at finalization; disabled fault scope is not failed capture. */
export async function finalizeSelfTestEvidence(root, context, result) {
  let maybeClaim, maybeEvidence;
  try { maybeClaim = await proof(root, 'self-test-claim.json'); } catch (error) { if (error.code !== 'ENOENT') throw error; }
  try { maybeEvidence = await proof(root, 'self-test-result.json'); } catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (!applySelfTestScope(result, context, maybeClaim !== undefined, maybeEvidence !== undefined)) return;
  if (!maybeClaim || !maybeEvidence) { result.complete = false; result.blockers.push('self_test_result_missing'); return; }
  const evidence = await validateSelfTest(maybeEvidence.value, context.gate_root, maybeClaim.value.request, context);
  result.self_test_reset_observed = evidence.summary?.stage === 'complete' && evidence.summary?.panicResetObserved === true;
  if (!result.self_test_reset_observed) { result.complete = false; result.blockers.push('self_test_reset_incomplete'); }
}
export function failureCategory(error) {
  return /^(?:panic|renew)_[a-z_]+$/u.test(error?.code ?? '') ? error.code : 'panic_operation_rejected';
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
    process.stdout.write(`${JSON.stringify({ error: failureCategory(error) })}\n`); process.exitCode = 1;
  });
}

import { retainedPackage } from '../str005-panic-probe/recovery-predecessor.mjs';
import { captureArchive } from './archive-clear.mjs';
import { BEFORE_READ_BASELINE_POLICY } from './baseline-policy.mjs';
import { mkdir, readFile, realpath } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { PAGE, BUNDLE, cleanPushed, ignored, missing, packageSnapshot, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew, retain } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { main as cutoffAudit } from '../core-dump/native-audit.mjs';
import { main as storeAudit } from '../core-dump/store-audit.mjs';
import { main as startAudit } from '../audit-signed-start-stack.mjs';
import { main as renewAudit } from '../audit-signed-renew-stack.mjs';
import { main as provenanceAudit } from '../audit-fault-provenance.mjs';
import { checkCommand } from '../str005-panic-probe/command-check.mjs';
import { INSTALL_TIMEOUT_MS } from '../str005-panic-probe/install.mjs';
import { verifyCorePreservation } from '../str005-panic-probe/core-preservation.mjs';
import { oldAnchors } from './anchors.mjs';
import { installationAnchor } from './installation.mjs';
import { clearAnchor, freshRecovery, recoveryAnchor } from './clear.mjs';
/** Old empty-core evidence binds the image/boot; the browser obtains new authority. */
export async function capturePrerequisite(root, repo, identity, physical, operations = { recoveryAnchor, verifyCorePreservation }) {
  const recovered = await operations.recoveryAnchor(root, repo, identity, physical);
  const corePreservation = await operations.verifyCorePreservation(root, recovered.context);
  return { recovered, corePreservation };
}
/** Capture audits use the sealed installation's retained package, never mutable build outputs. */
export async function capturePackage(manifest, installed, repo) {
  const retained = await retainedPackage(manifest, installed, repo);
  return { candidateElf: retained.candidateElf, retainedManifest: retained.retainedManifest,
    retainedManifestSha256: retained.packaged.manifest_sha256 };
}
export async function preflight(repo, root, options, published) {
  ignored(repo, root); await missing(root); await privateRoot(dirname(root));
  const old = await oldAnchors(repo), stage = published.stage;
  check(options['--gate-root'], 'diagnostic_gate_required');
  const gateRoot = options['--gate-root'], gateCommit = old.context.gate_commit; cleanPushed(gateRoot, gateCommit);
  const pins = [...(await readFile(resolve(repo, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === gateCommit, 'diagnostic_gate_pin');
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)), trust: await readFile(resolve(repo, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check(assets.bundle.includes(gateCommit), 'diagnostic_gate_bundle');
  const maybeInstalled = options['--installation-root'] ? await installationAnchor(options['--installation-root'], repo) : undefined;
  const before = maybeInstalled?.context ?? old.context;
  const context = { schema: 'str005-panic-probe-v1', ...published, ownerTask: 'task-str005-v2-accepted-share-probe',
    diagnosticSuccessor: true, renewSuccessor: true, ...BEFORE_READ_BASELINE_POLICY, predecessorInstallationFailed: false,
    firmware_root: repo, firmware_commit: before.firmware_commit, app_elf_sha256: before.app_elf_sha256,
    before_source: { firmware_commit: before.firmware_commit, app_elf_sha256: before.app_elf_sha256 },
    gate_root: gateRoot, gate_commit: gateCommit, scope: 'share', attemptId: old.share.before.attempt.id,
    original_campaign_id: old.context.original_campaign_id, detector: { physical: old.context.physical },
    archivePath: resolve(old.archiveRoot, 'core-dump.private.bin'), archiveSha: old.archiveSha,
    ...(maybeInstalled ? { diagnosticInstallation: { root: maybeInstalled.root, seal: maybeInstalled.seal } } : {}),
    gatePageSha256: sha256(assets.page), gateBundleSha256: sha256(assets.bundle), trustSha256: sha256(JSON.stringify(JSON.parse(assets.trust))) };
  if (stage === 'installation') {
    check(options['--manifest'] && options['--clear-root'] && !maybeInstalled, 'diagnostic_install_inputs');
    const cleared = await clearAnchor(options['--clear-root'], repo);
    check(cleared.context.firmware_commit === before.firmware_commit && cleared.context.app_elf_sha256 === before.app_elf_sha256 &&
      cleared.context.detector.physical === old.context.physical, 'diagnostic_clear_image');
    Object.assign(context, await packageSnapshot(repo, options['--manifest'], published.commit), {
      firmware_commit: published.commit, manifest: options['--manifest'], candidateElf: resolve(dirname(options['--manifest']), 'bitaxe-ultra205.elf'),
      diagnosticClear: { root: cleared.root, seal: cleared.seal }, installTimeoutMs: INSTALL_TIMEOUT_MS });
  }
  if (stage === 'capture') {
    check(maybeInstalled && options['--recovery-root'], 'diagnostic_capture_inputs');
    const { recovered, corePreservation } = await capturePrerequisite(options['--recovery-root'], repo, before, old.context.physical);
    context.corePreservation = corePreservation;
    Object.assign(context, await capturePackage(options['--retained-manifest'], maybeInstalled, repo));
    context.reference_commit = before.reference_commit;
    context.diagnosticRecovery = { root: options['--recovery-root'], seal: recovered.seal };
  }
  if (stage === 'clear') {
    check(options['--recovery-root'] && !maybeInstalled, 'diagnostic_clear_inputs');
    const recovered = await freshRecovery(options['--recovery-root'], repo, context, old.context.physical, published.commit);
    context.recoveryRoot = options['--recovery-root']; context.recoverySeal = recovered.seal;
  }
  if (stage === 'archive-clear') {
    check(options['--capture-root'] && options['--recovery-root'] && !maybeInstalled, 'diagnostic_archive_clear_inputs');
    const captured = await captureArchive(options['--capture-root'], repo);
    check(captured.context.detector.physical === old.context.physical && captured.context.gate_commit === gateCommit,
      'diagnostic_archive_clear_identity');
    Object.assign(context, { firmware_commit: captured.context.firmware_commit, app_elf_sha256: captured.context.app_elf_sha256,
      before_source: { firmware_commit: captured.context.firmware_commit, app_elf_sha256: captured.context.app_elf_sha256 },
      archivePath: captured.dumpPath, archiveSha: captured.dumpSha,
      diagnosticCapture: { root: captured.root, seal: captured.seal, archiveSha: captured.dumpSha } });
    const recovered = await freshRecovery(options['--recovery-root'], repo, context, old.context.physical, published.commit);
    context.recoveryRoot = options['--recovery-root']; context.recoverySeal = recovered.seal;
  }
  if (['installation', 'clear', 'archive-clear'].includes(stage)) {
    context.flashBinary = await realpath(resolve(repo, 'bazel-bin/tools/flash/flash'));
    context.flashBinarySha256 = await fileDigest(context.flashBinary);
  }
  await mkdir(root, { mode: 0o700 });
  if (['installation', 'capture'].includes(stage)) {
    context.storeAuditRequired = true; context.cutoffUserRegionRequired = true; context.signedPathAudits = {};
    for (const [name, run] of [['native', cutoffAudit], ['store', storeAudit], ['start', startAudit], ['renew', renewAudit], ['provenance', provenanceAudit]]) {
      const path = resolve(root, `${name}-audit.json`), report = await run(['--elf', context.candidateElf, '--output', path]);
      check(report.elf_sha256 === context.app_elf_sha256, 'diagnostic_audit_image');
      if (['start', 'renew'].includes(name)) { check(report.result === 'selected_path_with_headroom', 'diagnostic_stack_audit'); context.signedPathAudits[name] = await fileDigest(path); }
      if (name === 'provenance') check(report.native_abi_verified && report.hot_call_closure_cache_safe && report.forbidden_calls_absent, 'diagnostic_provenance_audit');
    }
    context.nativeAuditSha256 = await fileDigest(resolve(root, 'native-audit.json'));
    context.storeAuditSha256 = await fileDigest(resolve(root, 'store-audit.json'));
    context.provenanceAuditSha256 = await fileDigest(resolve(root, 'provenance-audit.json'));
    if (stage === 'installation') context.commandCheckSha256 = (await checkCommand(root, context)).runner_sha256;
  }
  await writeNew(resolve(root, 'context.json'), context);
  for (const [key, value] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), value);
  return { preflight: 'passed', stage, device_effects: false };
}

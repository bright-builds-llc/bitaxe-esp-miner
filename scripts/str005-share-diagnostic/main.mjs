import { finalizeDiagnostic } from './finalize.mjs';
import { fstatSync } from 'node:fs';
import { readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { git, cleanPushed, ignored, missing, protectedPath, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew, canonical } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { createProbeServer } from '../str005-panic-probe/server.mjs';
import { finish as finishProbe } from '../str005-panic-probe/main.mjs';
import { install } from '../str005-panic-probe/install.mjs';
import { verifyCommandCheck } from '../str005-panic-probe/command-check.mjs';
import { verifyNativeAudit } from '../str005-panic-probe/audit.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { processSnapshot, requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { argumentsFor, currentSource } from './contract.mjs';
import { oldAnchors, sealed } from './anchors.mjs';
import { installationAnchor } from './installation.mjs';
import { retainedPackage } from '../str005-panic-probe/recovery-predecessor.mjs';
import { captureArchive } from './archive-clear.mjs';
import { preflight } from './prepare.mjs';
import { clearAnchor, runClear, finishClear } from './clear.mjs';
export async function verifyCandidate(root, context) {
  if (!['installation', 'capture'].includes(context.stage)) return;
  await verifyNativeAudit(root, context);
  for (const name of ['start', 'renew', 'provenance']) {
    const saved = await proof(root, `${name}-audit.json`), expected = name === 'provenance' ? context.provenanceAuditSha256 : context.signedPathAudits[name];
    check(saved.sha256 === expected && saved.value.elf_sha256 === context.app_elf_sha256 &&
      (name === 'provenance' ? saved.value.native_abi_verified && saved.value.hot_call_closure_cache_safe : saved.value.result === 'selected_path_with_headroom'), 'diagnostic_audit_changed');
  }
  if (context.stage === 'installation') await verifyCommandCheck(root, context);
}
export async function main(argv) {
  const { action, options } = argumentsFor(argv), root = options['--private-root'];
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']); ignored(repo, root);
  if (action === 'preflight') return preflight(repo, root, options, await currentSource(repo, options['--stage']));
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.diagnosticSuccessor === true && context.ownerTask === 'task-str005-v2-accepted-share-probe', 'diagnostic_context');
  const verify = async () => {
    const current = await currentSource(repo, context.stage);
    check(current.commit === context.commit && current.contractSha256 === context.contractSha256 && current.installEnabled === context.installEnabled && current.selfTestEnabled === context.selfTestEnabled, 'diagnostic_source_changed');
    cleanPushed(context.gate_root, context.gate_commit); await oldAnchors(repo); await verifyCandidate(root, context);
    if (context.diagnosticClear) check(await sealed(context.diagnosticClear.root, repo) === context.diagnosticClear.seal, 'diagnostic_clear_changed');
    if (context.diagnosticInstallation) check(await sealed(context.diagnosticInstallation.root, repo) === context.diagnosticInstallation.seal, 'diagnostic_installation_changed');
    if (context.diagnosticRecovery) check(await sealed(context.diagnosticRecovery.root, repo) === context.diagnosticRecovery.seal, 'diagnostic_recovery_changed');
    if (context.diagnosticCapture) {
      const captured = await captureArchive(context.diagnosticCapture.root, repo);
      check(captured.seal === context.diagnosticCapture.seal && captured.dumpSha === context.archiveSha &&
        captured.dumpSha === context.diagnosticCapture.archiveSha, 'diagnostic_capture_changed');
    }
    if (context.retainedManifest) {
      const installed = await installationAnchor(context.diagnosticInstallation.root, repo);
      const retained = await retainedPackage(context.retainedManifest, installed, repo);
      check(retained.candidateElf === context.candidateElf && retained.packaged.manifest_sha256 === context.retainedManifestSha256,
        'diagnostic_retained_package_changed');
    }
  };
  await verify();
  if (action === 'finish') return ['clear', 'archive-clear'].includes(context.stage) ? finishClear(root, context) : finishProbe(root, context, (target, result) => finalizeDiagnostic(target, context, result));
  if (action === 'clear') { check(['clear', 'archive-clear'].includes(context.stage), 'diagnostic_clear_disabled'); return runClear(root, context); }
  if (action === 'install') {
    check(context.stage === 'installation' && context.installEnabled, 'diagnostic_install_disabled');
    await clearAnchor(context.diagnosticClear.root, repo);
    const measured = (await proof(root, 'current-recovery.json')).value;
    const clearedAt = (await proof(context.diagnosticClear.root, 'clear-exit.json')).value.finishedAtUnixMs;
    check(measured.observed_at_unix_ms > clearedAt, 'diagnostic_postclear_baseline_required');
    return install(root, context);
  }
  check(action === 'serve' && context.stage !== 'clear', 'diagnostic_serve_stage');
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'diagnostic_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'diagnostic_distinct_output');
  await missing(resolve(root, 'server-owner.json'));
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detectorBytes = await readFile(detectorPath);
  const detector = parseDetector(detectorBytes.toString('utf8'), context.detector.physical, Date.now() - (await stat(detectorPath)).mtimeMs); requireNoHolders(detector.port);
  const page = await readFile(resolve(root, 'gate-page')), bundle = await readFile(resolve(root, 'gate-bundle')), trust = JSON.parse(await readFile(resolve(root, 'gate-trust')));
  check(sha256(page) === context.gatePageSha256 && sha256(bundle) === context.gateBundleSha256 && sha256(JSON.stringify(trust)) === context.trustSha256, 'diagnostic_assets_changed');
  const server = createProbeServer({ root, context: { ...context, detector: { ...context.detector, ...detector } }, page, bundle, trust,
    client: await readFile(resolve(repo, 'scripts/str005-panic-probe/client.mjs')),
    readinessClient: await readFile(resolve(repo, 'scripts/str005-panic-probe/self-test-readiness.mjs')) }, { verifyEffect: verify });
  let maybeClosing; const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'diagnostic_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port, serialPort: detector.port,
      physicalIdentitySha256: detector.physical, detectorSha256: sha256(detectorBytes), startedAtUnixMs: Date.now() });
    process.stdout.write(`diagnostic_url=http://127.0.0.1:${server.address().port}/\n`); await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ error: /^(diagnostic|panic)_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'diagnostic_failed' })}\n`); process.exitCode = 1;
});

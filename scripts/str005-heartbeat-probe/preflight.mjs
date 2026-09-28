import { mkdir, readFile, stat, realpath } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { PAGE, BUNDLE, missing, protectedPath, cleanPushed, nonce, fileDigest, git } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, retain, writeNew } from '../str005-noise-serial/files.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { canonicalFixture } from '../str005-startup-probe/tools.mjs';
import { check, object, sha256 } from '../str005-v2-serial/values.mjs';
import { admittedImage } from '../str005-share-probe/admission.mjs';
import { inspectPreparation } from './prepared.mjs';
export async function preflight(firmwareRoot, root, options, source) {
  await missing(root); await privateRoot(dirname(root));
  const prepared = await inspectPreparation(options['--preparation-root']);
  check(prepared.seal === source.preparationSealSha256, 'heartbeat_preparation_pin');
  git(firmwareRoot, ['merge-base', '--is-ancestor', prepared.context.source_commit, source.source_commit]);
  const contract = execFileSync('git', ['-C', firmwareRoot, 'show', `${prepared.context.source_commit}:scripts/str005-heartbeat-probe/README.md`]);
  check(sha256(contract) === prepared.context.contractSha256, 'heartbeat_preparation_contract');
  const manifest = await proof(prepared.context.parentRoots.share, 'admission.json');
  const admitted = await admittedImage(firmwareRoot, manifest.value), gateRoot = options['--gate-root'];
  check(admitted.identity.firmware_commit === prepared.context.firmware_commit && admitted.identity.app_elf_sha256 === prepared.context.app_elf_sha256 &&
    admitted.identity.gate_commit === prepared.context.gate_commit && admitted.physical === prepared.context.physical,
  'heartbeat_installed_preparation');
  cleanPushed(gateRoot, prepared.context.gate_commit);
  const tools = await canonicalFixture(firmwareRoot, options['--fixture-binary'], source.source_commit);
  const observerPath = await realpath(resolve(firmwareRoot, 'bazel-bin/tools/http-transport/cadence_observer'));
  const receiptPath = resolve(firmwareRoot, 'bazel-bin/tools/http-transport/v2-observer-build-identity.json');
  const receiptBytes = await readFile(receiptPath), built = JSON.parse(receiptBytes);
  const receipt = { sha256: sha256(receiptBytes) };
  object(built, ['schema', 'sourceCommit', 'sourceDirty', 'observerSha256', 'writerSha256']);
  check(built.schema === 'str005-v2-observer-build-v1' && built.sourceCommit === source.source_commit && built.sourceDirty === false &&
    built.observerSha256 === await fileDigest(observerPath) && built.writerSha256 === await fileDigest(resolve(firmwareRoot, 'scripts/str005-v2-serial/observer-build-identity.mjs')),
  'heartbeat_observer_provenance');
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), admitted.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(detector.port);
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check([prepared.context.gate_commit, 'suppressHeartbeats', 'startWindow'].every(marker => assets.bundle.includes(marker)), 'heartbeat_gate_bundle');
  await mkdir(root, { mode: 0o700 });
  for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
  await retain(resolve(root, 'admission.json'), manifest.bytes);
  const identity = admitted.identity;
  const context = { schema: 'str005-heartbeat-context-v1', admission: 'prepared-heartbeat-v1', ...source, ...identity,
    firmware_root: firmwareRoot, gate_root: gateRoot, before_source: identity, scope: 'share', attemptId: nonce(),
    preparationRoot: options['--preparation-root'], expectedBootOrdinal: prepared.result.after_boot_ordinal, expectedLedger: prepared.result.ledger,
    startupRoot: manifest.value.startupRoot, admissionSha256: manifest.sha256, cadence_observer: { path: observerPath, sha256: built.observerSha256 },
    observer_build_receipt_sha256: receipt.sha256, observer_receipt_path: resolve(firmwareRoot, 'bazel-bin/tools/http-transport/v2-observer-build-identity.json'),
    physical: admitted.physical, detector, captureVerified: true, original_campaign_id: admitted.originalCampaignId, ...tools,
    assetHashes: Object.fromEntries(Object.entries(assets).map(([key, bytes]) => [key, sha256(bytes)])) };
  await writeNew(resolve(root, 'context.json'), context);
  return { preflight: 'passed', start_issued: false, effects_performed: false };
}

import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve, basename } from 'node:path';
import { PAGE, BUNDLE, missing, protectedPath, cleanPushed, nonce } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, retain, writeNew } from '../str005-noise-serial/files.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { canonicalFixture } from '../str005-startup-probe/tools.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { verifyGateCompatibility } from './gate-compatibility.mjs';
import { requireGatePin } from './contract.mjs';
import { main as auditRenew } from '../audit-signed-renew-stack.mjs';
import { main as auditStart } from '../audit-signed-start-stack.mjs';
import { admittedImage } from './admission.mjs';
export async function preflight(firmwareRoot, root, options, source) {
  await missing(root); await privateRoot(dirname(root));
  const manifest = await proof(dirname(options['--admission']), basename(options['--admission']));
  check(manifest.sha256 === source.admissionSha256, 'share_admission_changed');
  const admitted = await admittedImage(firmwareRoot, manifest.value), gateRoot = options['--gate-root'];
  requireGatePin(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8'), manifest.value.gateCommit);
  cleanPushed(gateRoot, manifest.value.gateCommit);
  const tools = await canonicalFixture(firmwareRoot, options['--fixture-binary'], source.source_commit);
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), admitted.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(detector.port);
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check([manifest.value.gateCommit, 'startWindow', 'loadSignedWindow'].every(marker => assets.bundle.includes(marker)), 'share_gate_bundle');
  await mkdir(root, { mode: 0o700 });
  for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
  await retain(resolve(root, 'admission.json'), manifest.bytes);
  await writeNew(resolve(root, 'gate-compatibility.json'), await verifyGateCompatibility(gateRoot));
  const identity = admitted.identity;
  for (const [name, audit] of [['start', auditStart], ['renew', auditRenew]]) {
    const result = await audit(['--elf', resolve(manifest.value.captureBindings.decoderRoot, 'firmware.elf'), '--output', resolve(root, `native-${name}-stack.json`)]);
    check(result.elf_sha256 === identity.app_elf_sha256 && result.result === 'selected_path_with_headroom', 'share_native_stack');
  }
  const context = { schema: 'str005-share-context-v1', admission: 'qualified-renew-share-v1', ...source, ...identity,
    firmware_root: firmwareRoot, gate_root: gateRoot, before_source: identity, scope: 'share', attemptId: nonce(),
    expectedBootOrdinal: admitted.expectedBootOrdinal, expectedLedger: admitted.recovery.ledger,
    physical: admitted.physical, detector, captureVerified: true, original_campaign_id: admitted.originalCampaignId, ...tools,
    assetHashes: Object.fromEntries(Object.entries(assets).map(([key, bytes]) => [key, sha256(bytes)])) };
  await writeNew(resolve(root, 'context.json'), context);
  return { preflight: 'passed', start_issued: false, effects_performed: false };
}

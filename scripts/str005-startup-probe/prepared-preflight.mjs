import { execFileSync } from 'node:child_process';
import { main as auditSignedStart } from '../audit-signed-start-stack.mjs';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { PAGE, BUNDLE, missing, nonce, protectedPath, fileDigest, git } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, writeNew, retain } from '../str005-noise-serial/files.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { main as decode } from '../core-dump/main.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { captureEvidence } from './capture.mjs';
import { preparedEvidence } from './prepared-evidence.mjs';
import { reviewedGate } from './compatibility.mjs';
import { canonicalFixture } from './tools.mjs';
import { verifyGateCompatibility } from './gate-compatibility.mjs';
export async function preparedPreflight(firmwareRoot, root, options, source) {
  await missing(root); await privateRoot(dirname(root));
  const preparationRoot = options['--preparation-root'], preparation = await preparedEvidence(preparationRoot);
  check(/^[a-f0-9]{40}$/u.test(preparation.context.source_commit), 'startup_preparation_source');
  git(firmwareRoot, ['merge-base', '--is-ancestor', preparation.context.source_commit, source.source_commit]);
  const producerContract = execFileSync('git', ['-C', firmwareRoot, 'show', `${preparation.context.source_commit}:docs/hardware/str005-startup-preparation.md`]);
  check(sha256(producerContract) === preparation.context.contractSha256, 'startup_preparation_contract');
  const historical = preparation.historical, bindings = historical.bindings;
  const captured = await captureEvidence(firmwareRoot, bindings, source.captureSealSha256);
  const gateRoot = options['--gate-root'], compatible = await reviewedGate(firmwareRoot, gateRoot, captured.identity);
  check(preparation.context.gate_commit === compatible.gate_commit && preparation.context.gate_root === gateRoot &&
    preparation.context.physical === captured.physical, 'startup_preparation_gate');
  const tools = await canonicalFixture(firmwareRoot, options['--fixture-binary'], source.source_commit);
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), captured.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(detector.port);
  await mkdir(root, { mode: 0o700 });
  const freshDecode = await decode(['verify-cutoff', '--dump', resolve(bindings.archiveRoot, bindings.archiveRelative),
    '--elf', resolve(bindings.decoderRoot, 'firmware.elf'), '--elf-sha256', captured.identity.app_elf_sha256,
    '--private-root', resolve(root, 'capture-verification')]);
  check(freshDecode.status === 'passed' && freshDecode.self_test_marked === true && freshDecode.dump_sha256 === captured.archiveSha,
    'startup_fresh_decode');
  const elfPath = resolve(bindings.decoderRoot, 'firmware.elf');
  const stack = await auditSignedStart(['--elf', elfPath, '--output', resolve(root, 'native-start-audit.json')]);
  check(stack.elf_sha256 === captured.identity.app_elf_sha256 && stack.result === 'selected_path_with_headroom', 'startup_native_stack');
  const symbolsPath = resolve(firmwareRoot, 'scripts/verify-native-usb-symbols.mjs');
  const symbols = execFileSync(process.execPath, [symbolsPath, elfPath], { encoding: 'utf8', timeout: 30000, maxBuffer: 65536 });
  check(symbols.trim() === 'native_usb_symbols=verified', 'startup_native_usb_symbols');
  await writeNew(resolve(root, 'native-usb-symbols.json'), { schema: 'str005-installed-usb-symbol-audit-v1',
    elf_sha256: stack.elf_sha256, verifier_sha256: await fileDigest(symbolsPath), verified: true });
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check([compatible.gate_commit, 'startWindow', 'loadSignedWindow'].every(marker => assets.bundle.includes(marker)), 'startup_gate_bundle');
  await writeNew(resolve(root, 'gate-compatibility.json'), await verifyGateCompatibility(gateRoot));
  for (const [name, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${name}`), bytes);
  const identity = { ...captured.identity, gate_commit: compatible.gate_commit };
  const context = { schema: 'str005-startup-context-v2', admission: 'prepared-normal-stop-v1', ...source, ...identity,
    historical_gate_commit: captured.identity.gate_commit, gateCompatibilitySha256: compatible.compatibilitySha256,
    firmware_root: firmwareRoot, gate_root: gateRoot, before_source: identity, scope: 'share', attemptId: nonce(),
    preparationRoot, preparation_source_commit: preparation.context.source_commit, preparationSealSha256: preparation.seal, expectedBootOrdinal: preparation.result.after_boot_ordinal,
    expectedLedger: preparation.result.ledger, coreClearRoot: preparation.coreClearRoot,
    bindings, captureSeals: captured.seals, archiveSha: captured.archiveSha, physical: captured.physical, detector, captureVerified: true,
    original_campaign_id: captured.originalCampaignId, ...tools,
    assetHashes: Object.fromEntries(Object.entries(assets).map(([key, value]) => [key, sha256(value)])) };
  await writeNew(resolve(root, 'context.json'), context);
  return { preflight: 'passed', start_issued: false, firmware_reflashed: false, core_cleared: false };
}

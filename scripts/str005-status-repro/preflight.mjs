import { execFileSync } from 'node:child_process';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { PAGE, BUNDLE, cleanPushed, ignored, missing, protectedPath, fileDigest, nonce } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, retain, writeNew } from '../str005-noise-serial/files.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { canonicalFixture } from '../str005-startup-probe/tools.mjs';
import { installationAnchor } from '../str005-share-diagnostic/installation.mjs';
import { captureArchive } from '../str005-share-diagnostic/archive-clear.mjs';
import { archiveClearAnchor, recoveryAnchor } from '../str005-share-diagnostic/clear.mjs';
import { capturePackage } from '../str005-share-diagnostic/prepare.mjs';
import { main as cutoffAudit } from '../core-dump/native-audit.mjs';
import { main as storeAudit } from '../core-dump/store-audit.mjs';
import { main as startAudit } from '../audit-signed-start-stack.mjs';
import { main as provenanceAudit } from '../audit-fault-provenance.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { PINS } from './contract.mjs';
import { verifyGateCompatibility } from '../str005-startup-probe/gate-compatibility.mjs';

export async function lineage(repo, options, source) {
  const installed = await installationAnchor(options['--installation-root'], repo);
  const captured = await captureArchive(options['--capture-root'], repo);
  const cleared = await archiveClearAnchor(options['--clear-root'], repo);
  const identity = { firmware_commit: installed.context.firmware_commit, app_elf_sha256: installed.context.app_elf_sha256,
    gate_commit: installed.context.gate_commit };
  const physical = installed.context.detector.physical;
  check(installed.seal === PINS.installation && captured.seal === PINS.capture && cleared.seal === PINS.clear &&
    identity.firmware_commit === PINS.firmware && identity.app_elf_sha256 === PINS.elf && identity.gate_commit === PINS.gate,
  'status_repro_fixed_lineage');
  check(captured.context.firmware_commit === identity.firmware_commit && captured.context.app_elf_sha256 === identity.app_elf_sha256 &&
    captured.context.gate_commit === identity.gate_commit && captured.context.detector.physical === physical &&
    cleared.context.firmware_commit === identity.firmware_commit && cleared.context.app_elf_sha256 === identity.app_elf_sha256 &&
    cleared.context.diagnosticCapture?.seal === captured.seal && cleared.context.archiveSha === captured.dumpSha,
  'status_repro_lineage');
  const recovered = await recoveryAnchor(options['--recovery-root'], repo, identity, physical);
  check(recovered.seal === PINS.recovery, 'status_repro_fixed_recovery');
  const clearExit = (await proof(cleared.root, 'clear-exit.json')).value;
  const status = (await proof(options['--recovery-root'], 'baseline-status.json')).value;
  const current = recovered.current.value;
  check(recovered.context.commit === cleared.context.commit && recovered.context.commit !== source.commit &&
    current.observed_at_unix_ms > clearExit.finishedAtUnixMs && current.ledger?.pending === false &&
    current.original_budget?.pending === false && current.current_v2_idle === true && current.restoration_confirmed === true &&
    current.serial_ownership_released === true && status.state === 'idle' && Number.isSafeInteger(status.observation?.bootOrdinal),
  'status_repro_postclear_recovery');
  return { installed, captured, cleared, recovered, identity, physical, status, current };
}

export async function preflight(repo, root, options, source) {
  ignored(repo, root); await missing(root); await privateRoot(dirname(root));
  const chain = await lineage(repo, options, source), gateRoot = options['--gate-root'];
  check(gateRoot === chain.installed.context.gate_root && chain.identity.gate_commit === chain.captured.context.gate_commit,
    'status_repro_gate');
  cleanPushed(gateRoot, chain.identity.gate_commit);
  const pin = (await readFile(resolve(repo, 'MODULE.bazel'), 'utf8')).match(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/u);
  check(pin?.[1] === chain.identity.gate_commit, 'status_repro_gate_pin');
  const packageInfo = await capturePackage(chain.captured.context.retainedManifest, chain.installed, repo);
  const tools = await canonicalFixture(repo, options['--fixture-binary'], source.commit);
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(repo, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check([chain.identity.gate_commit, 'startWindow', 'loadSignedWindow'].every(marker => assets.bundle.includes(marker)), 'status_repro_gate_bundle');
  const compatibility = await verifyGateCompatibility(gateRoot);
  check(compatibility.zeroRenewalsAccepted === true && compatibility.renewAfterMilliseconds === 20000 &&
    compatibility.renewalOrigin === 'completed-controller-start', 'status_repro_gate_timer');
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), chain.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(detector.port);
  await mkdir(root, { mode: 0o700 });
  for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
  await writeNew(resolve(root, 'gate-compatibility.json'), compatibility);
  const audits = {};
  for (const [name, run] of [['native', cutoffAudit], ['store', storeAudit], ['start', startAudit], ['provenance', provenanceAudit]]) {
    const path = resolve(root, `${name}-audit.json`), report = await run(['--elf', packageInfo.candidateElf, '--output', path]);
    check(report.elf_sha256 === chain.identity.app_elf_sha256 && (name !== 'start' || report.result === 'selected_path_with_headroom') &&
      (name !== 'provenance' || report.native_abi_verified === true && report.hot_call_closure_cache_safe === true), 'status_repro_audit');
    audits[name] = await fileDigest(path);
  }
  const symbolsPath = resolve(repo, 'scripts/verify-native-usb-symbols.mjs');
  const symbols = execFileSync(process.execPath, [symbolsPath, packageInfo.candidateElf], { encoding: 'utf8', timeout: 30000, maxBuffer: 65536 });
  check(symbols.trim() === 'native_usb_symbols=verified', 'status_repro_native_usb');
  const context = { schema: 'str005-status-repro-context-v1', admission: 'diagnostic-status-v1', source_commit: source.commit,
    contractSha256: source.contractSha256, ...chain.identity, firmware_root: repo, gate_root: gateRoot,
    before_source: chain.identity, scope: 'share', attemptId: nonce(), expectedBootOrdinal: chain.status.observation.bootOrdinal,
    expectedLedger: chain.current.ledger, original_campaign_id: chain.captured.context.original_campaign_id,
    physical: chain.physical, detector, ...tools, candidateElf: packageInfo.candidateElf,
    retainedManifest: packageInfo.retainedManifest, retainedManifestSha256: packageInfo.retainedManifestSha256,
    anchors: { installation: { root: chain.installed.root, seal: chain.installed.seal }, capture: { root: chain.captured.root, seal: chain.captured.seal },
      clear: { root: chain.cleared.root, seal: chain.cleared.seal }, recovery: { root: options['--recovery-root'], seal: chain.recovered.seal } },
    auditSha256: audits, gateCompatibilitySha256: await fileDigest(resolve(root, 'gate-compatibility.json')),
    symbolVerifierSha256: await fileDigest(symbolsPath),
    assetHashes: Object.fromEntries(Object.entries(assets).map(([name, bytes]) => [name, sha256(bytes)])) };
  await writeNew(resolve(root, 'context.json'), context);
  return { preflight: 'passed', device_effects: false, attempt_issued: false };
}

import { execFileSync } from 'node:child_process';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { PAGE, BUNDLE, cleanPushed, ignored, missing, protectedPath, fileDigest, nonce } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, retain, writeNew } from '../str005-noise-serial/files.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { canonicalFixture } from '../str005-startup-probe/tools.mjs';
import { main as cutoffAudit } from '../core-dump/native-audit.mjs';
import { main as storeAudit } from '../core-dump/store-audit.mjs';
import { main as startAudit } from '../audit-signed-start-stack.mjs';
import { main as provenanceAudit } from '../audit-fault-provenance.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { verifyGateCompatibility } from '../str005-startup-probe/gate-compatibility.mjs';
import { PINS } from './contract.mjs';
import { currentRecovery, installation, previousStart, restartAfter } from './lineage.mjs';

export const SCHEMA = 'str005-step5-diagnostic-context-v1';
export const ADMISSION = 'diagnostic-step5-v1';

export const STEP5_PROFILE = Object.freeze({ pins: PINS, schema: SCHEMA, admission: ADMISSION });

/** Effect-free admission of one Start against the sealed install, previous Start and restart. */
export async function preflight(repo, root, options, source, operations = {}, profile = STEP5_PROFILE) {
  const pins = profile.pins;
  ignored(repo, root); await missing(root); await privateRoot(dirname(root));
  const installed = await (operations.installation ?? installation)(options['--installation-root'], pins);
  const maybePrevious = options['--previous-start-root']
    ? await (operations.previousStart ?? previousStart)(options['--previous-start-root'], installed, pins) : null;
  check((maybePrevious !== null) === Boolean(pins.previousStartResult), 'step5_previous_required');
  const maybeRestart = options['--restart-root'] && maybePrevious
    ? await (operations.restartAfter ?? restartAfter)(options['--restart-root'], maybePrevious, pins) : null;
  check((maybeRestart !== null) === Boolean(pins.restartResult), 'step5_restart_required');
  // A current recovery re-bases an install-only lineage after reboots that left no retained record.
  const maybeRecovery = options['--current-recovery-root'] && !maybePrevious
    ? await (operations.currentRecovery ?? currentRecovery)(options['--current-recovery-root'], installed, pins) : null;
  check((maybeRecovery !== null) === Boolean(pins.currentRecoveryResult), 'step5_recovery_required');
  const current = maybeRecovery ?? maybeRestart ?? maybePrevious;
  const gateRoot = options['--gate-root'];
  check(gateRoot === installed.gate_root && installed.identity.gate_commit === pins.gate, 'step5_gate');
  cleanPushed(gateRoot, installed.identity.gate_commit);
  const pin = (await readFile(resolve(repo, 'MODULE.bazel'), 'utf8')).match(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/u);
  check(pin?.[1] === installed.identity.gate_commit, 'step5_gate_pin');
  const tools = await canonicalFixture(repo, options['--fixture-binary'], source.commit);
  const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
    trust: await readFile(resolve(repo, 'firmware/bitaxe/bwg/deployment-trust.json')) };
  check([installed.identity.gate_commit, 'startWindow', 'loadSignedWindow', 'worker_revocation_detail', ...(profile.bundleMarkers ?? [])]
    .every(marker => assets.bundle.includes(marker)), 'step5_gate_bundle');
  const compatibility = await verifyGateCompatibility(gateRoot);
  check(compatibility.zeroRenewalsAccepted === true && compatibility.renewAfterMilliseconds === 20000 &&
    compatibility.renewalOrigin === 'completed-controller-start', 'step5_gate_timer');
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), installed.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  requireNoHolders(detector.port);
  await mkdir(root, { mode: 0o700 });
  for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
  await writeNew(resolve(root, 'gate-compatibility.json'), compatibility);
  const audits = {};
  for (const [name, run] of [['native', cutoffAudit], ['store', storeAudit], ['start', startAudit], ['provenance', provenanceAudit]]) {
    const path = resolve(root, `${name}-audit.json`), report = await run(['--elf', installed.candidateElf, '--output', path]);
    check(report.elf_sha256 === installed.identity.app_elf_sha256 && (name !== 'start' || report.result === 'selected_path_with_headroom') &&
      (name !== 'provenance' || report.native_abi_verified === true && report.hot_call_closure_cache_safe === true), 'step5_audit');
    audits[name] = await fileDigest(path);
  }
  const symbolsPath = resolve(repo, 'scripts/verify-native-usb-symbols.mjs');
  const symbols = execFileSync(process.execPath, [symbolsPath, installed.candidateElf], { encoding: 'utf8', timeout: 30000, maxBuffer: 65536 });
  check(symbols.trim() === 'native_usb_symbols=verified', 'step5_native_usb');
  const context = { schema: profile.schema, admission: profile.admission, source_commit: source.commit, contractSha256: source.contractSha256,
    // Only a renewal probe's context carries a minimum; every other profile's context is unchanged.
    ...(profile.minimumRenewals ? { minimum_renewals: profile.minimumRenewals } : {}),
    ...installed.identity, firmware_root: repo, gate_root: gateRoot, before_source: installed.identity, scope: 'share',
    attemptId: nonce(), expectedBootOrdinal: current?.expectedBootOrdinal ?? installed.expectedBootOrdinal,
    expectedLedger: current?.ledger ?? installed.ledger,
    original_campaign_id: installed.original_campaign_id, physical: installed.physical, detector, ...tools,
    candidateElf: installed.candidateElf, retainedManifest: installed.retainedManifest,
    retainedManifestSha256: installed.retainedManifestSha256,
    anchors: { installation: { root: installed.root, seal: installed.seal },
      ...(maybePrevious ? { previousStart: { root: maybePrevious.root, seal: maybePrevious.seal } } : {}),
      ...(maybeRestart ? { restart: { root: maybeRestart.root, seal: maybeRestart.seal } } : {}),
      ...(maybeRecovery ? { currentRecovery: { root: maybeRecovery.root, seal: maybeRecovery.seal } } : {}) },
    auditSha256: audits, gateCompatibilitySha256: await fileDigest(resolve(root, 'gate-compatibility.json')),
    symbolVerifierSha256: await fileDigest(symbolsPath),
    assetHashes: Object.fromEntries(Object.entries(assets).map(([name, bytes]) => [name, sha256(bytes)])) };
  if (operations.extraContext) Object.assign(context, await operations.extraContext(repo, source));
  await writeNew(resolve(root, 'context.json'), context);
  return { preflight: 'passed', device_effects: false, attempt_issued: false };
}

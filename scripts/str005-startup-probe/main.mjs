import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { PAGE, BUNDLE, git, cleanPushed, ignored, missing, fileDigest, nonce, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew, retain, inventory } from '../str005-noise-serial/files.mjs';
import { requireNoHolders, processSnapshot } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { main as decode } from '../core-dump/main.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { HARDWARE_ENABLED, TASK, CONTRACT, admitArguments, requireEnabled } from './contract.mjs';
import { captureEvidence, sealed } from './capture.mjs';
import { runClear } from './clear.mjs';
import { createServerOwner } from './server.mjs';
import { finalize } from './finish.mjs';
import { canonicalTools } from './tools.mjs';
import { verifyGateCompatibility } from './gate-compatibility.mjs';

async function currentSource(root, effect) {
  if (effect) check(HARDWARE_ENABLED, 'startup_hardware_disabled');
  const source = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8');
  if (effect) requireEnabled(tasks);
  const block = tasks.split(`### ${TASK} |`)[1]?.split(/^### /mu)[0] ?? '';
  const pins = [...block.matchAll(/^Startup capture seal: ([a-f0-9]{64})\.$/gmu)];
  check(pins.length === 1, 'startup_capture_pin');
  return { source_commit: source, captureSealSha256: pins[0][1], contractSha256: await fileDigest(resolve(root, CONTRACT)) };
}
export async function main(argv) {
  if (argv[0]?.startsWith('recover-')) return (await import('./recovery-main.mjs')).recoveryMain(argv);
  const { action, options } = admitArguments(argv);
  // Reject before touching private inputs, authority directories, discovery or subprocesses.
  if (action !== 'finish') check(HARDWARE_ENABLED, 'startup_hardware_disabled');
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const root = options['--private-root']; ignored(firmwareRoot, root);
  const source = await currentSource(firmwareRoot, action !== 'finish');
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root)); await protectedPath(options['--bindings']);
    const bindings = JSON.parse(await readFile(options['--bindings'], 'utf8'));
    const tools = await canonicalTools(firmwareRoot, options, source.source_commit);
    const captured = await captureEvidence(firmwareRoot, bindings, source.captureSealSha256), gateRoot = options['--gate-root'];
    cleanPushed(gateRoot, captured.identity.gate_commit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === captured.identity.gate_commit, 'startup_gate_pin');
    const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
    const detector = parseDetector(await readFile(detectorPath, 'utf8'), captured.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
    requireNoHolders(detector.port);
    await mkdir(root, { mode: 0o700 });
    const freshDecode = await decode(['verify-cutoff', '--dump', resolve(bindings.archiveRoot, bindings.archiveRelative),
      '--elf', resolve(bindings.decoderRoot, 'firmware.elf'), '--elf-sha256', captured.identity.app_elf_sha256, '--private-root', resolve(root, 'capture-verification')]);
    check(freshDecode.status === 'passed' && freshDecode.self_test_marked === true && freshDecode.dump_sha256 === captured.archiveSha, 'startup_fresh_decode');
    const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
      trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
    check([captured.identity.gate_commit, 'startWindow', 'loadSignedWindow'].every(marker => assets.bundle.includes(marker)), 'startup_gate_bundle');
    const compatibility = await verifyGateCompatibility(gateRoot);
    await writeNew(resolve(root, 'gate-compatibility.json'), compatibility);
    for (const [name, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${name}`), bytes);
    const context = { schema: 'str005-startup-context-v1', ...source, ...captured.identity, firmware_root: firmwareRoot, gate_root: gateRoot,
      before_source: captured.identity, scope: 'share', attemptId: nonce(), bindings, captureSeals: captured.seals,
      archiveSha: captured.archiveSha, physical: captured.physical, detector, captureVerified: true,
      original_campaign_id: captured.originalCampaignId, ...tools,
      assetHashes: Object.fromEntries(Object.entries(assets).map(([key, value]) => [key, sha256(value)])),
      compatibilitySha256: await fileDigest(resolve(root, 'gate-compatibility.json')) };
    await writeNew(resolve(root, 'context.json'), context); return { preflight: 'passed', start_issued: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-startup-context-v1' && context.source_commit === source.source_commit && context.contractSha256 === source.contractSha256,
    'startup_source_changed');
  const verify = async () => {
    await currentSource(firmwareRoot, true); cleanPushed(context.gate_root, context.gate_commit);
    check(await fileDigest(context.fixture_binary) === context.fixture_sha256 && await fileDigest(context.fixture_receipt_path) === context.fixture_receipt_sha256 &&
      await fileDigest(context.flash_binary) === context.flash_sha256, 'startup_tool_changed');
    for (const [key, hash] of Object.entries(context.captureSeals)) check(await sealed(context.bindings[key]) === hash, 'startup_capture_changed');
  };
  if (action === 'clear') { await verify(); await runClear(root, context); return { clear_complete: true, start_issued: false }; }
  if (action === 'finish') return finalize(root, context);
  await verify();
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'startup_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'startup_distinct_output');
  requireNoHolders(context.detector.port); await missing(resolve(root, 'server-owner.json'));
  const assets = {};
  for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'startup_asset_changed'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust); assets.pageClient = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/page.mjs'));
  assets.coordinator = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/client.mjs'));
  await missing(resolve(root, 'serve-claim.json'));
  const admissionOwner = (await processSnapshot()).find(row => row.pid === process.pid); check(admissionOwner, 'startup_server_owner');
  await writeNew(resolve(root, 'serve-claim.json'), { schema: 'str005-startup-serve-claim-v1', owner: admissionOwner, contextSha256: sha256(JSON.stringify(context)) });
  const server = await createServerOwner({ root, context, assets, authorityDirectory: options['--authority-directory'], verify });
  let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'startup_server_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port });
    process.stdout.write(`startup_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ complete: false, blocker: /^startup_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'startup_operation_rejected' })}\n`); process.exitCode = 1; });

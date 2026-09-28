import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { PAGE, BUNDLE, cleanPushed, git, ignored, missing, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, writeNew, retain, inventory } from '../str005-noise-serial/files.mjs';
import { processSnapshot, sameProcess, requireNoHolders, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { currentSource, predecessor, argumentsFor, SHARE_SEAL } from './contract.mjs';
import { STAGES, conclusion, recoveryProof } from './model.mjs';
import { createRecoveryServer } from './server.mjs';
async function detect(root, physical, final = false) {
  const path = resolve(dirname(root), `${final ? 'final-' : ''}detector.stdout.log`); await protectedPath(path);
  return parseDetector(await readFile(path, 'utf8'), physical, Date.now() - (await stat(path)).mtimeMs);
}
export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const root = options['--private-root']; ignored(firmwareRoot, root);
  const source = await currentSource(firmwareRoot, action !== 'finish');
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root)); ignored(firmwareRoot, options['--share-root']);
    const prior = await predecessor(firmwareRoot, options['--share-root']), before = prior.context;
    const gateRoot = options['--gate-root']; cleanPushed(gateRoot, before.gate_commit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === before.gate_commit, 'share_recovery_gate_pin');
    const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
      trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
    check(assets.bundle.includes(before.gate_commit), 'share_recovery_gate_bundle');
    for (const [key, bytes] of Object.entries(assets)) check(sha256(bytes) === before.assetHashes[key], 'share_recovery_historical_asset');
    const context = { schema: 'str005-share-failure-recovery-context-v1', ...source, firmware_root: firmwareRoot, gate_root: gateRoot,
      shareRoot: options['--share-root'], shareSeal: SHARE_SEAL, firmware_commit: before.firmware_commit, app_elf_sha256: before.app_elf_sha256,
      gate_commit: before.gate_commit, physical: before.physical, before_source: { firmware_commit: before.firmware_commit, app_elf_sha256: before.app_elf_sha256 },
      scope: 'share', attemptId: prior.before.attempt.id, original_campaign_id: before.original_campaign_id,
      historical_before_ledger: prior.before.ledger, assetHashes: before.assetHashes };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    for (const [key, bytes] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), bytes);
    return { preflight: 'passed', device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-share-failure-recovery-context-v1' && context.source_commit === source.source_commit &&
    context.contractSha256 === source.contractSha256 && context.shareSeal === SHARE_SEAL, 'share_recovery_source_changed');
  const prior = await predecessor(firmwareRoot, context.shareRoot);
  check(context.firmware_commit === prior.context.firmware_commit && context.app_elf_sha256 === prior.context.app_elf_sha256 &&
    context.gate_commit === prior.context.gate_commit && context.attemptId === prior.before.attempt.id && context.physical === prior.context.physical,
  'share_recovery_context_changed');
  if (action === 'finish') return finish(root, context);
  const verify = async () => { const current = await currentSource(firmwareRoot); check(current.source_commit === context.source_commit && current.contractSha256 === context.contractSha256,
    'share_recovery_source_changed'); cleanPushed(context.gate_root, context.gate_commit); };
  await verify(); const selected = await detect(root, context.physical); requireNoHolders(selected.port);
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'share_recovery_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'share_recovery_distinct_output');
  await missing(resolve(root, 'server-owner.json'));
  const assets = { modules: {} };
  for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'share_recovery_asset_changed'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust);
  for (const [url, relative] of [['/bootstrap.mjs', 'str005-share-recovery/bootstrap.mjs'], ['/recovery-page.mjs', 'str005-share-recovery/page.mjs'], ['/recovery-collection.mjs', 'str005-startup-probe/recovery-collection.mjs'], ['/retained-status.mjs', 'str005-startup-probe/retained-status.mjs']])
    assets.modules[url] = await readFile(resolve(firmwareRoot, 'scripts', relative));
  const server = createRecoveryServer({ root, context, assets, verify }); let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'share_recovery_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, port: server.address().port, serialPort: selected.port, physical: selected.physical });
    process.stdout.write(`share_recovery_url=http://127.0.0.1:${server.address().port}/\n`);
    await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true };
}
export async function finish(root, context) {
  let owner;
  try {
    owner = (await proof(root, 'server-owner.json')).value;
    const processes = await processSnapshot();
    check(owner.owner && !processes.some(row => sameProcess(row, owner.owner) || row.ppid === owner.owner.pid), 'share_recovery_owner_live');
  } catch {
    return { schema: 'str005-share-recovery-finish-blocked-v1', current_safe_recovery: false,
      blockers: ['writer_exit_unproved'], sealed: false, qualification_complete: false, parity_promotion: false };
  }
  let hostReleased = false, releasePhase = 'listener_release';
  try {
    requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
    releasePhase = 'final_detector';
    const selected = await detect(root, context.physical, true);
    releasePhase = 'serial_release';
    for (const port of new Set([owner.serialPort, selected.port])) requireNoHolders(port);
    hostReleased = true;
  } catch { /* A missing or failed host release check is an explicit blocker. */ }
  const parts = {};
  for (const stage of STAGES) { try { parts[stage] = (await proof(root, `${stage}.json`)).value; } catch (error) { if (error.code !== 'ENOENT') throw error; } }
  const result = { ...conclusion(parts, context, hostReleased), release_blocker: hostReleased ? null : releasePhase };
  if (result.current_safe_recovery) {
    try {
      const begin = (await proof(root, 'collection-begin.json')).value;
      check(begin.schema === 'str005-share-recovery-begin-v1', 'share_recovery_begin');
      const current = recoveryProof(parts, context, begin.startedAtUnixMs);
      await writeNew(resolve(root, 'current-recovery.json'), current); result.fresh_effect_proof = true;
    } catch (error) {
      if (error.code !== 'ENOENT' && !/^(?:share_recovery|panic)_[a-z_]+$/u.test(error.code ?? '')) throw error;
      result.fresh_effect_proof = false; result.blockers.push('fresh_effect_proof_unavailable');
    }
  } else result.fresh_effect_proof = false;
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: /^share_recovery_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'share_recovery_failed' })}\n`); process.exitCode = 1; });

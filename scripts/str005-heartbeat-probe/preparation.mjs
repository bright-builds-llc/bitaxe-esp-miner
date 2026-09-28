import { inspectPredecessor } from './predecessor.mjs';
import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { once } from 'node:events';
import { isDeepStrictEqual as equal } from 'node:util';
import { PAGE, BUNDLE, cleanPushed, git, ignored, missing, fileDigest, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory, writeNew, retain, inventory } from '../str005-noise-serial/files.mjs';
import { processSnapshot, requireGone, requireNoHolders, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { currentConclusion } from '../str005-startup-probe/recovery-main.mjs';
import { createCurrentRecoveryServer, RECOVERY_STAGES } from '../str005-startup-probe/recovery-server.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { createRestartServer } from '../str005-startup-preparation/server.mjs';
import { conclusion, restartEvidence, STAGES, FRESH_MS } from '../str005-startup-preparation/model.mjs';
export const ENABLED = false;
const TASK = 'task-str005-heartbeat-shutdown-probe', CONTRACT = 'scripts/str005-heartbeat-probe/README.md';
export function argumentsFor(argv, enabled = ENABLED) {
  const [action, ...args] = argv, options = {}; check(['preflight', 'serve', 'finish'].includes(action) && args.length % 2 === 0, 'preparation_arguments');
  if (action !== 'finish') check(enabled, 'preparation_disabled');
  for (let i = 0; i < args.length; i += 2) { const key = args[i], value = args[i + 1];
    check(['--private-root', '--share-root', '--gate-root', '--stage'].includes(key) && !options[key] && typeof value === 'string' &&
      (key === '--stage' ? ['recovery', 'restart'].includes(value) : resolve(value) === value), 'preparation_arguments'); options[key] = value; }
  check(options['--private-root'] && (action === 'preflight' ? options['--share-root'] && options['--gate-root'] && Object.keys(options).length === 3 : options['--stage'] && Object.keys(options).length === 2), 'preparation_arguments');
  return { action, options };
}
async function sealed(root, expected) { await privateRoot(root); const seal = await proof(root, 'sealed-inventory.json');
  check(!expected || seal.sha256 === expected, 'preparation_parent_seal'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json'])); return seal.sha256; }
async function source(root, effect) {
  const source_commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source_commit);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8'), blocks = tasks.split(`### ${TASK} |`);
  check(blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${TASK} |`), 'preparation_task');
  if (effect) check(ENABLED && blocks[1].split(/^### /mu)[0].split(/\r?\n/u).includes('Heartbeat preparation hardware: enabled.'), 'preparation_disabled');
  return { source_commit, contractSha256: await fileDigest(resolve(root, CONTRACT)) };
}
async function parts(root, names) { const result = {}; for (const name of names) { try { result[name] = (await proof(root, `${name}.json`)).value; }
  catch (error) { if (error.code !== 'ENOENT') throw error; } } return result; }
async function parents(roots) {
  const admitted = await inspectPredecessor(roots.share), parent = admitted.context, recovered = admitted.recovery;
  return { firmware_commit: parent.firmware_commit, app_elf_sha256: parent.app_elf_sha256, historical_gate_commit: parent.historical_gate_commit,
    physical: parent.physical, attemptId: admitted.before.attempt.id, originalGeneration: recovered.status.record.workerGeneration,
    expectedLedger: recovered.ledger, originalBudget: recovered.original_budget, original_campaign_id: parent.original_campaign_id,
    before_boot_ordinal: recovered.status.observation.bootOrdinal, parentRoots: roots, parentSeals: { share: admitted.seal },
    predecessorGateCommit: parent.gate_commit };
}
async function detector(root, context, stage, final = false) {
  const path = resolve(dirname(root), `${stage}-${final ? 'final-' : ''}detector.stdout.log`); await protectedPath(path);
  return parseDetector(await readFile(path, 'utf8'), context.physical, Date.now() - (await stat(path)).mtimeMs);
}
async function recoveryReady(root, context, fresh) {
  const child = resolve(root, 'recovery'); const hash = await sealed(child);
  const result = (await proof(child, 'result.json')).value, status = (await proof(child, 'status.json')).value;
  check(result.current_recovery_complete === true && status.observation.bootOrdinal === context.before_boot_ordinal &&
    status.record?.attemptId === context.attemptId, 'preparation_recovery_incomplete');
  const collection = (await proof(child, 'collection-begin.json')).value;
  check(collection.schema === 'str005-recovery-collection-v1' && collection.startedAtUnixMs === result.startedAtUnixMs, 'preparation_collection_binding');
  if (fresh) check(Date.now() >= result.startedAtUnixMs && Date.now() - result.startedAtUnixMs <= FRESH_MS, 'preparation_recovery_stale');
  return hash;
}
export async function main(argv) {
  const { action, options } = argumentsFor(argv), root = options['--private-root'];
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']); ignored(firmwareRoot, root);
  const published = await source(firmwareRoot, action !== 'finish');
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root)); const bound = await parents({ share: options['--share-root'] });
    const gateRoot = options['--gate-root'], gateCommit = git(gateRoot, ['rev-parse', 'HEAD']); cleanPushed(gateRoot, gateCommit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === gateCommit && gateCommit === bound.predecessorGateCommit, 'preparation_gate_pin');
    const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
      trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
    check(assets.bundle.includes(gateCommit) && assets.bundle.includes('qualificationRestart'), 'preparation_gate_bundle');
    const context = { schema: 'str005-heartbeat-preparation-context-v1', ...published, ...bound, firmware_root: firmwareRoot,
      gate_root: gateRoot, gate_commit: gateCommit, scope: 'share', assetHashes: Object.fromEntries(Object.entries(assets).map(([key, value]) => [key, sha256(value)])) };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    for (const [key, value] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), value);
    return { preflight: 'passed', mining_started: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json')); const context = (await proof(root, 'context.json')).value;
  const prior = await parents(context.parentRoots); check(context.schema === 'str005-heartbeat-preparation-context-v1' &&
    context.source_commit === published.source_commit && context.contractSha256 === published.contractSha256 &&
    Object.entries(prior).every(([key, value]) => equal(context[key], value)), 'preparation_context_changed');
  cleanPushed(context.gate_root, context.gate_commit); const stage = options['--stage'], child = resolve(root, stage);
  if (action === 'finish') return finish(root, child, stage, context);
  await missing(child); if (stage === 'restart') await recoveryReady(root, context, true);
  const detected = await detector(root, context, stage); requireNoHolders(detected.port);
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'preparation_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'preparation_distinct_output');
  await mkdir(child, { mode: 0o700 }); const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'preparation_owner');
  await writeNew(resolve(child, 'serve-claim.json'), { owner, startedAtUnixMs: Date.now(), contextSha256: sha256(JSON.stringify(context)) });
  const assets = {}; for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'preparation_asset_changed'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust);
  const verify = async () => { const current = await source(firmwareRoot, true);
    cleanPushed(context.gate_root, context.gate_commit);
    check(current.source_commit === context.source_commit && current.contractSha256 === context.contractSha256, 'preparation_source_changed'); if (stage === 'restart') await recoveryReady(root, context, true); };
  let server;
  if (stage === 'recovery') {
    assets.client = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/recovery-client.mjs'));
    assets.retainedStatus = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/retained-status.mjs'));
    server = createCurrentRecoveryServer({ root: child, context, assets });
  } else { assets.client = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-preparation/restart-client.mjs')); server = createRestartServer({ root: child, context, assets, verify }); }
  let maybeClosing; const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try { server.listen(0, '127.0.0.1'); await once(server, 'listening'); await writeNew(resolve(child, 'server-owner.json'), { owner, detector: detected, port: server.address().port });
    process.stdout.write(`preparation_url=http://127.0.0.1:${server.address().port}/\n`); await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { stage, released: true };
}
async function finish(root, child, stage, context) {
  await missing(resolve(child, 'sealed-inventory.json')); const owner = (await proof(child, 'server-owner.json')).value;
  const current = await detector(root, context, stage, true); await requireGone([owner.owner]);
  for (const port of new Set([owner.detector.port, current.port])) requireNoHolders(port); requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  if (stage === 'recovery') {
    const observed = await parts(child, [...RECOVERY_STAGES, 'finished']), result = currentConclusion(observed, context, true);
    const collection = await parts(child, ['collection-begin']);
    result.startedAtUnixMs = collection['collection-begin']?.startedAtUnixMs ?? null;
    if (collection['collection-begin']?.schema !== 'str005-recovery-collection-v1' || !Number.isSafeInteger(result.startedAtUnixMs))
      result.blockers.push('preparation_collection_unclaimed');
    if (observed.status?.observation.bootOrdinal !== context.before_boot_ordinal) result.blockers.push('preparation_boot_changed');
    if (result.startedAtUnixMs === null || Date.now() < result.startedAtUnixMs || Date.now() - result.startedAtUnixMs > FRESH_MS) result.blockers.push('preparation_recovery_stale');
    result.current_recovery_complete = result.blockers.length === 0; await writeNew(resolve(child, 'result.json'), result);
    await writeNew(resolve(child, 'sealed-inventory.json'), { files: await inventory(child) });
    if (!result.current_recovery_complete) {
      const failed = { ...conclusion({}, context, true), blockers: result.blockers, failed_stage: 'recovery' };
      await writeNew(resolve(root, 'result.json'), failed); await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
    }
    return result;
  }
  const observed = await parts(child, [...STAGES, 'finished', 'evidence', 'first-failure']), claim = await parts(child, ['restart-claim']);
  if (observed.evidence && claim['restart-claim']) {
    try { observed.evidence = await restartEvidence(observed.evidence, context, claim['restart-claim'].request); observed.evidenceVerified = true; }
    catch (error) { observed.evidenceVerified = false; observed.evidenceFailure = /^(restart|statistics|preparation)_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'restart_evidence_unverified'; }
  }
  observed.firstFailure = observed['first-failure'] ?? null;
  const result = conclusion(observed, context, true); result.recoverySealSha256 = await recoveryReady(root, context, false);
  result.restartEvidenceSha256 = observed.evidence ? await fileDigest(resolve(child, 'evidence.json')) : null;
  await writeNew(resolve(child, 'result.json'), result); await writeNew(resolve(child, 'sealed-inventory.json'), { files: await inventory(child) });
  result.restartSealSha256 = await fileDigest(resolve(child, 'sealed-inventory.json'));
  await writeNew(resolve(root, 'result.json'), result); await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) }); return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify({ complete: value.complete ?? value.current_recovery_complete ?? null, preflight: value.preflight ?? null, stage: value.stage ?? null, released: value.released ?? null, blockers: value.blockers ?? [], mining_started: false })}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ complete: false, blocker: /^preparation_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'preparation_rejected' })}\n`); process.exitCode = 1;
});

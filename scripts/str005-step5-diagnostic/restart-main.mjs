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
import { createCurrentRecoveryServer, RECOVERY_STAGES, recoveryClientModules } from '../str005-startup-probe/recovery-server.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { createRestartServer } from '../str005-startup-preparation/server.mjs';
import { conclusion, restartEvidence, STAGES, FRESH_MS } from '../str005-startup-preparation/model.mjs';
import { RESTART } from './restart-config.mjs';
import { startGeneration } from './lineage.mjs';
/** One no-mining qualification restart that clears the previous Start's retained V2 record. */
export const ENABLED = true;
const TASK = RESTART.task, CONTRACT = RESTART.contract;
const ENABLED_LINE = RESTART.enabledLine;
// The origin that holds the Ultra 205 Web Serial grant.
const PORT = 48765;
export function argumentsFor(argv, enabled = ENABLED) {
  const [action, ...args] = argv, options = {}; check(['preflight', 'serve', 'finish', 'review'].includes(action) && args.length % 2 === 0, 'preparation_arguments');
  if (!['finish', 'review'].includes(action)) check(enabled, 'preparation_disabled');
  for (let i = 0; i < args.length; i += 2) { const key = args[i], value = args[i + 1];
    check(['--private-root', '--start-root', '--gate-root', '--stage'].includes(key) && !options[key] && typeof value === 'string' &&
      (key === '--stage' ? ['recovery', 'restart'].includes(value) : resolve(value) === value), 'preparation_arguments'); options[key] = value; }
  check(options['--private-root'] && (action === 'preflight' ? options['--start-root'] && options['--gate-root'] && Object.keys(options).length === 3
    : action === 'review' ? Object.keys(options).length === 1 : options['--stage'] && Object.keys(options).length === 2), 'preparation_arguments');
  return { action, options };
}
async function sealed(root, expected) { await privateRoot(root); const seal = await proof(root, 'sealed-inventory.json');
  check(!expected || seal.sha256 === expected, 'preparation_parent_seal'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json'])); return seal.sha256; }
async function source(root, effect) {
  const source_commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, source_commit);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8'), blocks = tasks.split(`### ${TASK} |`);
  check(blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${TASK} |`), 'preparation_task');
  if (effect) check(ENABLED && blocks[1].split(/^### /mu)[0].split(/\r?\n/u).includes(ENABLED_LINE), 'preparation_disabled');
  return { source_commit, contractSha256: await fileDigest(resolve(root, CONTRACT)) };
}
async function parts(root, names) { const result = {}; for (const name of names) { try { result[name] = (await proof(root, `${name}.json`)).value; }
  catch (error) { if (error.code !== 'ENOENT') throw error; } } return result; }
/** The pinned sealed step-5 Start: its attempt, generation, fresh-recovery ledger and boot. */
async function parents(roots) {
  await sealed(roots.start, RESTART.startSeal);
  check(await fileDigest(resolve(roots.start, 'result.json')) === RESTART.startResult, 'preparation_parent_seal');
  const start = (await proof(roots.start, 'context.json')).value, before = (await proof(roots.start, 'before.json')).value;
  const run = (await proof(roots.start, 'run.json')).value, recovered = await parts(roots.start,
    ['recovery-1-ledger', 'recovery-1-original_budget', 'recovery-1-status']);
  const maybeEarlier = run.proof || run.dispatchStatus ? null : (await parts(roots.start, ['recovery-0-status']))['recovery-0-status'] ?? null;
  const status = recovered['recovery-1-status'], boot = status?.observation?.bootOrdinal, generation = startGeneration(run, maybeEarlier);
  check(run.observedStart === true && Number.isSafeInteger(generation) && status?.state === 'terminal' &&
    status.record?.attemptId === before.attempt.id && status.record.workerGeneration === generation &&
    recovered['recovery-1-ledger']?.pending === false && Number.isSafeInteger(boot), 'preparation_parent_binding');
  return { firmware_commit: start.firmware_commit, app_elf_sha256: start.app_elf_sha256, historical_gate_commit: start.gate_commit,
    physical: start.physical, attemptId: before.attempt.id, originalGeneration: generation,
    expectedLedger: recovered['recovery-1-ledger'], originalBudget: recovered['recovery-1-original_budget'],
    original_campaign_id: start.original_campaign_id, before_boot_ordinal: boot, parentRoots: roots,
    parentSeals: { start: RESTART.startSeal } };
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
  if (action === 'review') return review(root);
  const published = await source(firmwareRoot, action !== 'finish');
  if (action === 'preflight') {
    await missing(root); await privateRoot(dirname(root)); const bound = await parents({ start: options['--start-root'] });
    const gateRoot = options['--gate-root'], gateCommit = git(gateRoot, ['rev-parse', 'HEAD']); cleanPushed(gateRoot, gateCommit);
    const pins = [...(await readFile(resolve(firmwareRoot, 'MODULE.bazel'), 'utf8')).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
    check(pins.length === 1 && pins[0][1] === gateCommit, 'preparation_gate_pin');
    const assets = { page: await readFile(resolve(gateRoot, PAGE)), bundle: await readFile(resolve(gateRoot, BUNDLE)),
      trust: await readFile(resolve(firmwareRoot, 'firmware/bitaxe/bwg/deployment-trust.json')) };
    check(assets.bundle.includes(gateCommit) && assets.bundle.includes('qualificationRestart'), 'preparation_gate_bundle');
    const context = { schema: 'str005-step5-restart-context-v1', ...published, ...bound, prior_attempt_completed: true, firmware_root: firmwareRoot,
      gate_root: gateRoot, gate_commit: gateCommit, scope: 'share', assetHashes: Object.fromEntries(Object.entries(assets).map(([key, value]) => [key, sha256(value)])) };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    for (const [key, value] of Object.entries(assets)) await retain(resolve(root, `gate-${key}`), value);
    return { preflight: 'passed', mining_started: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json')); const context = (await proof(root, 'context.json')).value;
  const prior = await parents(context.parentRoots); check(context.schema === 'str005-step5-restart-context-v1' &&
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
  const verify = async () => { await source(firmwareRoot, true); if (stage === 'restart') await recoveryReady(root, context, true); };
  let server;
  if (stage === 'recovery') {
    assets.modules = await recoveryClientModules(firmwareRoot);
    server = createCurrentRecoveryServer({ root: child, context, assets });
  } else { assets.client = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-preparation/restart-client.mjs')); server = createRestartServer({ root: child, context, assets, verify }); }
  let maybeClosing; const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try { requireLsofAbsent(['-nP', `-iTCP:${PORT}`, '-sTCP:LISTEN', '-t']); server.listen(PORT, '127.0.0.1'); await once(server, 'listening'); await writeNew(resolve(child, 'server-owner.json'), { owner, detector: detected, port: server.address().port });
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
/** Effect-free re-judgment of a sealed restart's immutable evidence with the corrected
 * prior-attempt rule; it writes a sibling `-review` root and never touches the original. */
async function review(root) {
  await privateRoot(root); const rootSeal = await sealed(root);
  const context = (await proof(root, 'context.json')).value, child = resolve(root, 'restart'), reviewRoot = `${root}-review`;
  await missing(reviewRoot); await sealed(child);
  check(context.schema === 'str005-step5-restart-context-v1', 'preparation_review_context');
  const observed = await parts(child, [...STAGES, 'finished', 'evidence', 'first-failure']), claim = await parts(child, ['restart-claim']);
  check(observed.evidence && claim['restart-claim'], 'preparation_review_evidence');
  observed.evidence = await restartEvidence(observed.evidence, { ...context, prior_attempt_completed: true }, claim['restart-claim'].request);
  observed.evidenceVerified = true; observed.firstFailure = observed['first-failure'] ?? null;
  // The page's only collection failure was the evidence verification this review now performs.
  const remaining = (observed.finished?.failures ?? ['missing']).filter(stage => stage !== 'evidence');
  if (remaining.length === 0) observed.finished = { failures: [] };
  const result = conclusion(observed, context, true);
  result.review = { schema: 'str005-step5-restart-review-v1', source_seal: rootSeal, rule: 'prior_attempt_completed' };
  await mkdir(reviewRoot, { mode: 0o700 });
  await writeNew(resolve(reviewRoot, 'result.json'), result);
  await writeNew(resolve(reviewRoot, 'sealed-inventory.json'), { files: await inventory(reviewRoot) });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify({ complete: value.complete ?? value.current_recovery_complete ?? null, preflight: value.preflight ?? null, stage: value.stage ?? null, released: value.released ?? null, blockers: value.blockers ?? [], mining_started: false })}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ complete: false, blocker: /^preparation_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'preparation_rejected' })}\n`); process.exitCode = 1;
});

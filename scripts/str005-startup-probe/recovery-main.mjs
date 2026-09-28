import { isDeepStrictEqual as equal } from 'node:util';
import { fstatSync } from 'node:fs';
import { mkdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { once } from 'node:events';
import { cleanPushed, git, ignored, missing, fileDigest, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory, writeNew, retain, inventory } from '../str005-noise-serial/files.mjs';
import { requireGone, requireNoHolders, requireLsofAbsent, processSnapshot } from '../str005-v2-serial/host-resources.mjs';
import { validateRecoveryParts, recoveryConclusion } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateAttempt, validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { createCurrentRecoveryServer, RECOVERY_STAGES, validateFinished } from './recovery-server.mjs';
export const RECOVERY_ENABLED = true;
export const STARTUP_SEAL = '950a8e55b5efb468905a4edd2e739cfd02e47013797e026bf0218af955234cb7';
const TASK = 'task-str005-mining-startup-probe', CONTRACT = 'docs/hardware/str005-startup-current-recovery.md';
export function recoveryArguments(argv, enabled = RECOVERY_ENABLED) {
  const [action, ...rest] = argv, options = {};
  check(['recover-preflight', 'recover-serve', 'recover-finish'].includes(action) && rest.length % 2 === 0, 'startup_recovery_arguments');
  if (action !== 'recover-finish') check(enabled, 'startup_recovery_disabled');
  for (let index = 0; index < rest.length; index += 2) { const key = rest[index], value = rest[index + 1];
    check(['--private-root', '--startup-root'].includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'startup_recovery_arguments'); options[key] = value; }
  check(options['--private-root'] && (action === 'recover-preflight' ? Object.keys(options).length === 2 : Object.keys(options).length === 1), 'startup_recovery_arguments');
  return { action, options };
}
async function source(root, effect) {
  const commit = git(root, ['rev-parse', 'HEAD']); cleanPushed(root, commit);
  const tasks = await readFile(resolve(root, 'TASKS.md'), 'utf8'), blocks = tasks.split(`### ${TASK} |`);
  check(blocks.length === 2 && (tasks.split('## Active\n')[1]?.split(/^## /mu)[0] ?? '').includes(`### ${TASK} |`), 'startup_recovery_task');
  if (effect) check(RECOVERY_ENABLED && blocks[1].split(/^### /mu)[0].split(/\r?\n/u).includes('Startup recovery hardware: enabled.'), 'startup_recovery_disabled');
  return { source_commit: commit, contractSha256: await fileDigest(resolve(root, CONTRACT)) };
}
async function original(root) {
  await privateRoot(root); const seal = await proof(root, 'sealed-inventory.json');
  check(seal.sha256 === STARTUP_SEAL, 'startup_recovery_seal');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const context = (await proof(root, 'context.json')).value, before = (await proof(root, 'before.json')).value, run = (await proof(root, 'run.json')).value;
  validateAttempt(before.attempt);
  check(run.observedStart === true && run.firstFailure === null && run.proof?.workDispatched > run.proof?.initialWorkDispatched &&
    before.attempt.purpose === 'normal' && context.schema === 'str005-startup-context-v1', 'startup_recovery_confirmed_start');
  const ledger = (await proof(root, 'recovery-0-ledger.json')).value; validateLedger(ledger); check(!ledger.pending, 'startup_recovery_old_pending');
  return { context, before, run, ledger };
}
export function currentConclusion(parts, context, hostReleased) {
  const projected = Object.fromEntries(Object.entries(parts).filter(([key]) => !['stop', 'finished'].includes(key)));
  if (parts.finished) projected.finished = { failures: validateFinished(parts.finished).failures.filter(row => row.stage !== 'stop').map(row => row.stage) };
  validateRecoveryParts(projected, context); const result = recoveryConclusion(projected);
  const blockers = [...result.blockers];
  if (parts.stop?.requested !== true || parts.finished?.failures.some(row => row.stage === 'stop')) blockers.push('current_stop_unconfirmed');
  if (parts.ledger && !equal(parts.ledger, context.expectedLedger)) blockers.push('accounting_changed_since_startup_seal');
  if (parts.status?.record && parts.status.record.workerGeneration !== context.originalGeneration) blockers.push('retained_generation_mismatch');
  if (!hostReleased) blockers.push('host_release_unconfirmed');
  return { schema: 'str005-current-startup-recovery-v1', current_recovery_complete: blockers.length === 0, blockers,
    accounting_measured: result.accounting_measured, current_restoration_confirmed: result.restoration_confirmed,
    retained_resources_verified: result.device_resources_released, serial_released: result.serial_released,
    historical_authorization_checkpoint_verified: false, historical_preservation_verified: false,
    qualification_complete: false, parity_promotion: false, start_replayed: false, failures: parts.finished?.failures ?? [] };
}
export async function recoveryMain(argv) {
  const { action, options } = recoveryArguments(argv), root = options['--private-root'];
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  ignored(firmwareRoot, root); const published = await source(firmwareRoot, action !== 'recover-finish');
  if (action === 'recover-preflight') {
    await missing(root); await privateRoot(dirname(root)); const oldRoot = options['--startup-root'];
    const old = await original(oldRoot); cleanPushed(old.context.gate_root, old.context.gate_commit);
    const context = { schema: 'str005-current-startup-recovery-context-v1', ...published, startupRoot: oldRoot, startupSeal: STARTUP_SEAL,
      firmware_commit: old.context.firmware_commit, app_elf_sha256: old.context.app_elf_sha256, gate_commit: old.context.gate_commit, gate_root: old.context.gate_root,
      scope: 'share', attemptId: old.before.attempt.id, original_campaign_id: old.context.original_campaign_id,
      originalGeneration: old.run.proof.generation, expectedLedger: old.ledger, physical: old.context.physical,
      assetHashes: old.context.assetHashes };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(oldRoot, `gate-${key}`)); check(sha256(bytes) === hash, 'startup_recovery_asset'); await retain(resolve(root, `gate-${key}`), bytes); }
    return { recovery_preflight: 'passed', effects: 'read_stop_close_only', qualification_complete: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-current-startup-recovery-context-v1' && context.source_commit === published.source_commit &&
    context.contractSha256 === published.contractSha256 && context.startupSeal === STARTUP_SEAL, 'startup_recovery_context');
  const old = await original(context.startupRoot);
  check(context.attemptId === old.before.attempt.id && context.originalGeneration === old.run.proof.generation &&
    context.firmware_commit === old.context.firmware_commit && context.app_elf_sha256 === old.context.app_elf_sha256 &&
    context.gate_commit === old.context.gate_commit && context.gate_root === old.context.gate_root && context.physical === old.context.physical &&
    context.original_campaign_id === old.context.original_campaign_id && equal(context.assetHashes, old.context.assetHashes) &&
    equal(context.expectedLedger, old.ledger), 'startup_recovery_original_binding');
  if (action === 'recover-finish') {
    const owner = (await proof(root, 'server-owner.json')).value;
    const detectorPath = resolve(dirname(root), 'final-detector.stdout.log'); await protectedPath(detectorPath);
    const current = parseDetector(await readFile(detectorPath, 'utf8'), context.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
    await requireGone([owner.owner]); for (const port of new Set([owner.detector.port, current.port])) requireNoHolders(port); requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
    const parts = {};
    for (const stage of [...RECOVERY_STAGES, 'finished']) { try { parts[stage] = (await proof(root, `${stage}.json`)).value; } catch (error) { if (error.code !== 'ENOENT') throw error; } }
    const result = currentConclusion(parts, context, true); await writeNew(resolve(root, 'result.json'), result);
    await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) }); return result;
  }
  cleanPushed(context.gate_root, context.gate_commit); await missing(resolve(root, 'server-owner.json'));
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, 'startup_recovery_private_output');
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, 'startup_recovery_distinct_output');
  const detectorPath = resolve(dirname(root), 'detector.stdout.log'); await protectedPath(detectorPath);
  const detector = parseDetector(await readFile(detectorPath, 'utf8'), context.physical, Date.now() - (await stat(detectorPath)).mtimeMs); requireNoHolders(detector.port);
  const assets = {};
  for (const [key, hash] of Object.entries(context.assetHashes)) { const bytes = await readFile(resolve(root, `gate-${key}`)); check(sha256(bytes) === hash, 'startup_recovery_asset'); assets[key] = bytes; }
  assets.trust = JSON.parse(assets.trust); assets.client = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/recovery-client.mjs'));
  assets.retainedStatus = await readFile(resolve(firmwareRoot, 'scripts/str005-startup-probe/retained-status.mjs'));
  await missing(resolve(root, 'serve-claim.json'));
  const admissionOwner = (await processSnapshot()).find(row => row.pid === process.pid); check(admissionOwner, 'startup_recovery_owner');
  await writeNew(resolve(root, 'serve-claim.json'), { schema: 'str005-current-recovery-serve-claim-v1', owner: admissionOwner, contextSha256: sha256(JSON.stringify(context)) });
  const server = createCurrentRecoveryServer({ root, context, assets }); let maybeClosing;
  const stop = () => { maybeClosing ??= server.release(); };
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, stop);
  try {
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'startup_recovery_owner');
    await writeNew(resolve(root, 'server-owner.json'), { owner, detector, port: server.address().port });
    process.stdout.write(`recovery_url=http://127.0.0.1:${server.address().port}/\n`); await once(server, 'close');
  } finally { stop(); await maybeClosing; for (const signal of ['SIGINT', 'SIGTERM']) process.removeListener(signal, stop); }
  return { server_released: true, qualification_complete: false };
}

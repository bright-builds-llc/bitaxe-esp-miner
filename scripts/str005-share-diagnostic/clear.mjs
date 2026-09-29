import { readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { proof, writeNew, retain, inventory, canonical } from '../str005-noise-serial/files.mjs';
import { fileDigest, protectedPath, missing } from '../fixed-usb-qualification/contract.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { requireNoHolders, processSnapshot, sameProcess, checkedOwner } from '../str005-v2-serial/host-resources.mjs';
import { runClearChild } from './clear-supervisor.mjs';
import { validateClearResult } from '../str005-startup-probe/capture.mjs';
import { BASELINE_PARTS, currentProof, validatePart, validateFinished } from '../str005-panic-probe/model.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { sealed, OLD_DUMP_SHA256 } from './anchors.mjs';
export async function recoveryAnchor(root, repo, identity, physical) {
  const seal = await sealed(root, repo), context = (await proof(root, 'context.json')).value, result = (await proof(root, 'result.json')).value;
  check(context.diagnosticSuccessor && context.recoveryOnly && /^[a-f0-9]{40}$/u.test(context.commit) && context.firmware_commit === identity.firmware_commit &&
    context.app_elf_sha256 === identity.app_elf_sha256 && context.gate_commit === identity.gate_commit && context.detector.physical === physical &&
    result.complete && result.baseline_complete && result.host_resources_released && result.blockers.length === 0, 'diagnostic_fresh_recovery');
  const parts = {};
  for (const stage of [...BASELINE_PARTS, 'finished']) { const value = (await proof(root, `baseline-${stage}.json`)).value;
    parts[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status :
      stage === 'finished' ? validateFinished(value, context) : validatePart(stage, value, context); }
  const begin = (await proof(root, 'baseline-begin.json')).value, current = await proof(root, 'current-recovery.json');
  check(begin.schema === 'str005-renew-baseline-begin-v1' && Number.isSafeInteger(begin.startedAtUnixMs) && begin.startedAtUnixMs >= 0 &&
    canonical(current.value) === canonical(currentProof(context, parts, begin.startedAtUnixMs)), 'diagnostic_recovery_freshness');
  return { seal, context, current };
}
/** Historical lineage never renews effect authority; only this gate uses current time. */
export function requireFreshRecovery(recovered, commit, now = Date.now()) {
  const observed = recovered.current.value.observed_at_unix_ms;
  check(recovered.context.commit === commit && Number.isSafeInteger(observed) && now >= observed && now - observed <= 120000, 'diagnostic_recovery_freshness');
  return recovered;
}
export async function freshRecovery(root, repo, identity, physical, commit, now = Date.now()) {
  return requireFreshRecovery(await recoveryAnchor(root, repo, identity, physical), commit, now);
}
export function clearArguments(root, context, port) {
  return ['core-dump-clear', '--board', '205', '--port', port, '--expected-physical-sha256', context.detector.physical,
    '--expected-installed-source', context.firmware_commit, '--expected-installed-elf', context.app_elf_sha256,
    '--recovery-proof', resolve(root, 'current-recovery.json'), '--preserved-dump', context.archivePath, '--preserved-sha256', context.archiveSha, '--private-root', resolve(root, 'clear')];
}
export async function runClear(root, context) {
  await missing(resolve(root, 'clear-claim.json')); await missing(resolve(root, 'clear'));
  const recovered = await freshRecovery(context.recoveryRoot, context.firmware_root, context, context.detector.physical, context.commit);
  check(recovered.seal === context.recoverySeal && await fileDigest(context.flashBinary) === context.flashBinarySha256 &&
    await fileDigest(context.archivePath) === context.archiveSha && context.archiveSha === OLD_DUMP_SHA256, 'diagnostic_clear_binding');
  const path = resolve(dirname(root), 'clear-detector.stdout.log'); await protectedPath(path);
  const detected = parseDetector(await readFile(path, 'utf8'), context.detector.physical, Date.now() - (await stat(path)).mtimeMs); requireNoHolders(detected.port);
  await retain(resolve(root, 'current-recovery.json'), recovered.current.bytes);
  const args = clearArguments(root, context, detected.port);
  const owner = (await processSnapshot()).find(row => row.pid === process.pid); check(owner, 'diagnostic_clear_owner');
  await writeNew(resolve(root, 'clear-claim.json'), { schema: 'str005-diagnostic-clear-claim-v1', owner, contextSha256: sha256(JSON.stringify(context)), port: detected.port, args, startedAtUnixMs: Date.now() });
  const child = await runClearChild(context.flashBinary, args, root);
  check(child.code === 0 && child.first_failure === null && child.group_released && child.owner_recorded && child.worker_recorded, 'diagnostic_clear_child_failed');
  await writeNew(resolve(root, 'clear-exit.json'), { code: 0, finishedAtUnixMs: Date.now() });
  return { clear_returned: true, fresh_baseline_required: true };
}
export async function clearAnchor(root, repo) {
  const seal = await sealed(root, repo), context = (await proof(root, 'context.json')).value;
  check(context.diagnosticSuccessor && context.stage === 'clear' && context.archiveSha === OLD_DUMP_SHA256, 'diagnostic_clear_anchor');
  const result = (await proof(root, 'result.json')).value;
  check(result.schema === 'str005-diagnostic-clear-result-v1' && result.complete === true && result.host_resources_released === true && result.current_group_absence_verified === true && result.cleanup_failure === null, 'diagnostic_clear_failed');
  await verifyClear(root, context); return { root, seal, context };
}
export async function verifyClear(root, context) {
  const claim = (await proof(root, 'clear-claim.json')).value, exit = (await proof(root, 'clear-exit.json')).value;
  check(typeof claim.port === 'string' && /^\/dev\/[A-Za-z0-9._/-]+$/u.test(claim.port) && canonical(claim.args) === canonical(clearArguments(root, context, claim.port)) && claim.contextSha256 === sha256(JSON.stringify(context)) && exit.code === 0 && exit.finishedAtUnixMs >= claim.startedAtUnixMs && exit.finishedAtUnixMs - claim.startedAtUnixMs <= 1200000, 'diagnostic_clear_exit');
  validateClearResult((await proof(root, 'clear/result.private.json')).value, { ...context, source_commit: context.commit });
  const original = await readFile(resolve(root, 'clear/core-dump.private.bin')), after = await readFile(resolve(root, 'clear/cleared-core-dump.private.bin'));
  check(original.length === 974848 && after.length === 974848 && sha256(original) === context.archiveSha && after.every(byte => byte === 255), 'diagnostic_clear_bytes');
}
const CHILD_PHASES = new Set(['session_admission', 'physical_identity', 'rom_admission', 'partition_table_read', 'partition_table_validation',
  'dump_read', 'preserved_dump_match', 'dump_erase', 'dump_erase_readback', 'application_return_failed', 'cleanup_failed']);
export function clearRunnerSucceeded(value) {
  return value.code === 0 && value.signal === null && value.spawn_failed === false && value.timed_out === false && value.interrupted === false &&
    value.first_failure === null && value.group_released === true && value.owner_recorded === true && value.worker_recorded === true;
}
export async function finishClear(root, context, operations = {}) {
  const snapshot = operations.processSnapshot ?? processSnapshot;
  let claim, childOwner, workerOwner, childResult;
  try {
    claim = (await proof(root, 'clear-claim.json')).value;
    childOwner = (await proof(root, 'clear-process-owner.json')).value;
    workerOwner = (await proof(root, 'clear-worker-owner.json')).value;
    childResult = (await proof(root, 'clear-process-result.json')).value;
    check(claim.owner && childOwner.schema === 'str005-clear-process-owner-v1' && workerOwner.schema === 'str005-clear-worker-owner-v1' &&
      childResult.schema === 'str005-clear-process-result-v1' && childResult.owner_recorded === true && childResult.worker_recorded === true && typeof childResult.group_released === 'boolean' &&
      childOwner.claimSha256 === (await proof(root, 'clear-claim.json')).sha256 && childOwner.owner.pgid === workerOwner.owner.pgid &&
      childOwner.argsSha256 === sha256(JSON.stringify(claim.args)) && childOwner.programSha256 === context.flashBinarySha256, 'diagnostic_clear_owner_binding');
    checkedOwner(claim.owner); checkedOwner(childOwner.owner); checkedOwner(workerOwner.owner);
    check(childOwner.owner.pgid === childOwner.owner.pid && childOwner.owner.pid !== workerOwner.owner.pid, 'diagnostic_clear_owner_binding');
    const live = await snapshot();
    check(!live.some(row => sameProcess(row, claim.owner) || row.ppid === claim.owner.pid ||
      sameProcess(row, childOwner.owner) || sameProcess(row, workerOwner.owner) || row.pgid === childOwner.owner.pgid), 'diagnostic_clear_writer_live');
  } catch { return { complete: false, sealed: false, blockers: ['clear_writer_exit_unproved'] }; }
  let producerFailure = null;
  try { await verifyClear(root, context); } catch { producerFailure = 'diagnostic_clear_unverified'; }
  if (!clearRunnerSucceeded(childResult)) producerFailure ??= 'diagnostic_clear_runner_unverified';
  const runnerFailures = new Set(['group_kill_failed', 'interrupted', 'timeout', 'child_failed', 'child_protocol', 'child_identity_unproved', 'supervisor_spawn_failed', 'child_exit_unproved', 'group_release_unproved']);
  if (producerFailure && runnerFailures.has(childResult.first_failure)) producerFailure = `diagnostic_clear_${childResult.first_failure}`;
  try {
    const child = (await proof(root, 'clear/result.private.json')).value;
    if (child.schema_version === 'bitaxe-development-core-dump-clear-v1' && child.source_commit === context.commit &&
      child.expected_installed_source === context.firmware_commit && child.expected_installed_elf === context.app_elf_sha256 &&
      CHILD_PHASES.has(child.first_failure_stage)) producerFailure = `diagnostic_clear_child_${child.first_failure_stage}`;
  } catch (error) { if (error.code !== 'ENOENT' && !(error instanceof SyntaxError)) throw error; }
  let cleanupFailure = null;
  try {
    const path = resolve(dirname(root), 'cleanup-detector.stdout.log'); await protectedPath(path);
    const selected = parseDetector(await readFile(path, 'utf8'), context.detector.physical, Date.now() - (await stat(path)).mtimeMs);
    for (const port of new Set([claim.port, selected.port])) (operations.requireNoHolders ?? requireNoHolders)(port);
  } catch { cleanupFailure = 'diagnostic_clear_cleanup_unproved'; }
  if (cleanupFailure) return { complete: false, sealed: false, first_failure: producerFailure,
    cleanup_failure: cleanupFailure, blockers: [...(producerFailure ? [producerFailure] : []), cleanupFailure] };
  const result = { schema: 'str005-diagnostic-clear-result-v1', complete: producerFailure === null,
    blockers: producerFailure ? [producerFailure] : [], first_failure: producerFailure, cleanup_failure: null,
    host_resources_released: true, current_group_absence_verified: true, original_group_release_verified: childResult.group_released, fresh_post_clear_baseline_required: true, historical_qualification: false, parity_promotion: false };
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) }); return result;
}

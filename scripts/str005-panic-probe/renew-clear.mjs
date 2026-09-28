/** The successor owns a new clear wrapper; no consumed startup gate is reactivated. */
import { mkdir, realpath } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { isDeepStrictEqual as equal } from 'node:util';
import { git, cleanPushed, ignored, missing, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, retain, writeNew, inventory } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { renewSource, RENEW_TASK } from './renew-successor.mjs';
import { verifiedIdleRecovery } from './verified-idle-recovery.mjs';
import { captureEvidence, sealed, validateRecovery } from '../str005-startup-probe/capture.mjs';
import { runClear, verifyClear } from '../str005-startup-probe/clear.mjs';
export function clearArguments(argv) {
  const [action, ...args] = argv, options = {};
  check(['renew-clear-preflight', 'renew-clear', 'renew-clear-finish'].includes(action) && args.length % 2 === 0, 'renew_clear_arguments');
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i], value = args[i + 1];
    check(['--private-root', '--capture-root', '--recovery-root'].includes(key) && !options[key] && typeof value === 'string' && resolve(value) === value, 'renew_clear_arguments');
    options[key] = value;
  }
  check(options['--private-root'] && (action === 'renew-clear-preflight' ? options['--capture-root'] && options['--recovery-root'] && Object.keys(options).length === 3 : Object.keys(options).length === 1), 'renew_clear_arguments');
  return { action, options };
}
export function validateClearRecovery(recovered, recoveryContext, recoveryResult, capture, sourceCommit, maybeNow) {
  validateRecovery(recovered, capture.identity, capture.physical, maybeNow);
  check(recoveryContext.renewSuccessor === true && recoveryContext.ownerTask === RENEW_TASK && recoveryContext.recoveryOnly === true &&
    recoveryContext.commit === sourceCommit && recovered.source_commit === sourceCommit && recoveryResult.complete === true &&
    recoveryResult.host_resources_released === true && recoveryResult.baseline_complete === true && recoveryResult.blockers?.length === 0 &&
    equal(recovered.ledger, capture.recovery.ledger) && equal(recovered.original_budget, capture.recovery.original_budget), 'renew_clear_recovery');
}
async function captureBindings(root) {
  const result = (await proof(root, 'result.json')).value;
  const rounds = result.candidate_recoveries;
  check(Array.isArray(rounds) && rounds.length >= 1 && rounds.every(row => row.complete && row.blockers?.length === 0), 'renew_clear_capture_recovery');
  const round = rounds.at(-1).round;
  check(/^candidate-recovery-[0-9]{3}$/u.test(round), 'renew_clear_capture_recovery');
  return { schema: 'str005-startup-proof-inputs-v1', captureRoot: root, archiveRoot: root, archiveRelative: 'self-test-core/core-dump.private.bin',
    decoderRoot: resolve(root, 'cutoff-review'), recoveryRoot: root, recoveryRelative: `${round}/current-recovery.json` };
}
export async function renewClearMain(argv) {
  const { action, options } = clearArguments(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ['rev-parse', '--show-toplevel']);
  const published = await renewSource(firmwareRoot), commit = git(firmwareRoot, ['rev-parse', 'HEAD']); cleanPushed(firmwareRoot, commit);
  check(published.clearEnabled, 'renew_clear_disabled');
  const root = options['--private-root']; ignored(firmwareRoot, root);
  if (action === 'renew-clear-preflight') {
    await missing(root); await privateRoot(dirname(root));
    ignored(firmwareRoot, options['--capture-root']); ignored(firmwareRoot, options['--recovery-root']);
    const bindings = await captureBindings(options['--capture-root']);
    const capture = await captureEvidence(firmwareRoot, bindings, published.captureSealSha256);
    const captureContext = (await proof(bindings.captureRoot, 'context.json')).value;
    check(captureContext.renewSuccessor === true && captureContext.ownerTask === RENEW_TASK, 'renew_clear_capture_owner');
    const recoveryRoot = options['--recovery-root'], recoverySeal = await sealed(recoveryRoot);
    const recoveryProducer = await verifiedIdleRecovery(recoveryRoot, capture.identity, capture.physical);
    const recoveryContext = recoveryProducer.context;
    const recovered = await proof(recoveryRoot, 'current-recovery.json');
    validateClearRecovery(recovered.value, recoveryContext, recoveryProducer.result, capture, commit, Date.now());
    const flash = await realpath(resolve(firmwareRoot, 'bazel-bin/tools/flash/flash'));
    const context = { schema: 'str005-renew-clear-context-v1', ownerTask: RENEW_TASK, source_commit: commit,
      contractSha256: published.contractSha256, ...capture.identity, physical: capture.physical,
      detector: recoveryContext.detector, bindings, archiveSha: capture.archiveSha, captureSealSha256: published.captureSealSha256,
      recoveryRoot, recoverySeal, recoverySha256: recovered.sha256, flash_binary: flash, flash_sha256: await fileDigest(flash) };
    await mkdir(root, { mode: 0o700 }); await writeNew(resolve(root, 'context.json'), context);
    await retain(resolve(root, 'current-recovery.json'), recovered.bytes);
    return { clear_preflight: 'passed', device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, 'sealed-inventory.json'));
  const context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-renew-clear-context-v1' && context.ownerTask === RENEW_TASK && context.source_commit === commit &&
    context.contractSha256 === published.contractSha256 && context.captureSealSha256 === published.captureSealSha256, 'renew_clear_source_changed');
  const verifyInputs = async () => {
    const capture = await captureEvidence(firmwareRoot, context.bindings, context.captureSealSha256);
    check(capture.archiveSha === context.archiveSha && equal(capture.identity, { firmware_commit: context.firmware_commit,
    app_elf_sha256: context.app_elf_sha256, gate_commit: context.gate_commit }) && capture.physical === context.physical &&
    await sealed(context.recoveryRoot) === context.recoverySeal && await fileDigest(resolve(root, 'current-recovery.json')) === context.recoverySha256, 'renew_clear_input_changed');
  };
  if (action === 'renew-clear-finish') return finalizeClear(root, context, verifyInputs);
  await verifyInputs();
  if (action === 'renew-clear') {
    await runClear(root, context);
    return { clear_command: 'verified', hardware_baseline_verified: false };
  }
  throw Object.assign(Error('renew_clear_action'), { code: 'renew_clear_action' });
}

async function maybeProof(root, relative) {
  try { return (await proof(root, relative)).value; }
  catch (error) {
    if (error instanceof SyntaxError) return { malformed: true };
    if (error.code !== 'ENOENT') throw error;
    return undefined;
  }
}
const CHILD_FAILURES = new Set(['session_admission', 'physical_identity', 'rom_admission', 'partition_table_read',
  'partition_table_validation', 'dump_read', 'preserved_dump_match', 'dump_erase', 'dump_erase_readback',
  'application_return_failed', 'cleanup_failed']);
const LAUNCH_FAILURES = new Set(['decoder_timeout', 'decoder_interrupted', 'decoder_failed', 'decoder_spawn', 'clear_launch_failed']);

/** Finalize actual partial producer output without converting it into qualification. */
export async function finalizeClear(root, context, verifyInputs = async () => {}) {
  await missing(resolve(root, 'result.json')); await missing(resolve(root, 'sealed-inventory.json'));
  let verified, blocker;
  try { await verifyInputs(); verified = await verifyClear(root, context); }
  catch (error) {
    if (error.code !== 'ENOENT' && !(error instanceof SyntaxError) && !/^(?:startup|renew)_[a-z_]+$/u.test(error.code ?? '')) throw error;
    blocker = error.code === 'ENOENT' ? 'renew_clear_receipt_missing' : error instanceof SyntaxError ? 'renew_clear_receipt_invalid' : error.code;
  }
  const result = { schema: 'str005-renew-clear-result-v1', complete: verified !== undefined, ...(verified ?? {}),
    blockers: blocker ? [blocker] : [], first_failure: blocker ?? null, hardware_baseline_verified: false,
    cleanup_receipt_verified: verified !== undefined, host_serial_release_verified: false,
    fresh_post_clear_baseline_required: true, parity_promotion: false };
  if (!verified) {
    const failure = await maybeProof(root, 'clear-failure.json');
    const child = await maybeProof(root, 'clear/result.private.json');
    const launcher = failure?.schema === 'str005-startup-clear-failure-v1' && LAUNCH_FAILURES.has(failure.firstFailure);
    const matchingChild = child?.schema_version === 'bitaxe-development-core-dump-clear-v1' &&
      child.source_commit === context.source_commit && child.expected_installed_source === context.firmware_commit &&
      child.expected_installed_elf === context.app_elf_sha256 && child.terminal_category !== 'complete';
    if (child?.malformed || failure?.malformed) result.first_failure = 'renew_clear_receipt_invalid';
    else if (matchingChild && CHILD_FAILURES.has(child.first_failure_stage)) result.first_failure = `renew_clear_child_${child.first_failure_stage}`;
    else if (launcher) result.first_failure = `renew_clear_${failure.firstFailure}`;
    result.cleanup_receipt_verified = Boolean(matchingChild && child.cleanup_complete === true);
    result.host_serial_release_verified = Boolean(launcher && failure.serialOwnershipReleased === true);
    if (!result.blockers.includes(result.first_failure)) result.blockers.push(result.first_failure);
  }
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return result;
}

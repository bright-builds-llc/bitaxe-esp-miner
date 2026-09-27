/** Failed installation admits observation only; it never becomes a successful install. */
import { readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileDigest, protectedPath, ignored, inspectPackage, missing } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { flashArguments, admitRecovery } from './install.mjs';
import { validateNativeAudit } from './audit.mjs';

function validateInstallBinding({ context, contextDigest, claim, runner, ownerDigest, owner, recoveryDigest }, timedOut) {
  check(context.schema === 'str005-panic-probe-v1' && /^[a-f0-9]{40}$/u.test(context.firmware_commit) &&
    /^[a-f0-9]{64}$/u.test(context.app_elf_sha256) && /^[a-f0-9]{40}$/u.test(context.reference_commit), 'panic_failed_install_identity');
  check(claim.schema === 'str005-panic-install-claim-v1' && claim.context_sha256 === contextDigest &&
    claim.recovery_sha256 === recoveryDigest && claim.server_owner_sha256 === ownerDigest &&
    claim.physical_identity_sha256 === context.detector.physical && owner.physicalIdentitySha256 === context.detector.physical &&
    typeof claim.port === 'string' && /^\/dev\/[A-Za-z0-9._/-]+$/u.test(claim.port), 'panic_failed_install_claim');
  const args = flashArguments(claim.root, { ...context, detector: { ...context.detector, port: claim.port } });
  check(runner.schema === 'str005-panic-install-runner-v1' &&
    (claim.timeout_ms ?? 480000) === (context.installTimeoutMs ?? 480000) && (runner.timeout_ms ?? 480000) === (claim.timeout_ms ?? 480000) &&
    [480000, 1200000].includes(context.installTimeoutMs ?? 480000) && (timedOut ? runner.code === null : runner.code === 1) && runner.spawn_failed === false &&
    runner.timed_out === timedOut && runner.interrupted === false && runner.serial_holders_absent === true &&
    Number.isSafeInteger(runner.started_at_unix_ms) && runner.started_at_unix_ms === claim.started_at_unix_ms &&
    Number.isSafeInteger(runner.finished_at_unix_ms) && runner.finished_at_unix_ms >= runner.started_at_unix_ms &&
    runner.binary_sha256 === context.flashBinarySha256 && claim.binary_sha256 === runner.binary_sha256 && claim.program === context.flashBinary &&
    JSON.stringify(claim.argv) === JSON.stringify(args) && claim.command_sha256 === sha256(JSON.stringify(args)) &&
    runner.command_sha256 === claim.command_sha256, 'panic_failed_install_runner');
}

export function validateFailedInstallation(input) {
  validateInstallBinding(input, false);
  const { context, runner, receipt, logDigest, log } = input;
  const a = receipt.fixed_serial_assessment;
  check(receipt.command_kind === 'flash-monitor' && receipt.board === '205' && receipt.flash_status === 'completed' &&
    receipt.firmware_commit === context.firmware_commit && receipt.observed_firmware_commit === context.firmware_commit &&
    receipt.reference_commit === context.reference_commit && [undefined, null, 'Unavailable', 'unavailable', context.reference_commit].includes(receipt.observed_reference_commit) && receipt.nvs_seed_status === 'not_provided' && receipt.redaction_mode === 'dual' &&
    receipt.capture_mode === 'noninteractive' && receipt.capture_timeout_seconds === 360 && receipt.manifest_path === context.manifest &&
    receipt.private_log_role === 'classifier-input-private' && receipt.private_monitor_log_sha256 === logDigest &&
    typeof receipt.timestamp === 'string' && /^[0-9]+$/u.test(receipt.timestamp) &&
    Number(receipt.timestamp) * 1000 + 999 >= runner.started_at_unix_ms && Number(receipt.timestamp) * 1000 <= runner.finished_at_unix_ms &&
    receipt.trusted_output === false && a?.execution_present === true && a.safe_baseline_confirmed === true &&
    a.startup_complete === false && a.startup_failed === true && a.stable_boot === true && a.retained_failure_history === false && Array.isArray(a.issues) &&
    a.issues.length > 0 && a.issues.includes('startup_failed') && a.issues.every(issue => ['error_diagnostic', 'startup_failed', 'startup_incomplete'].includes(issue)),
  'panic_failed_install_receipt');
  const lines = log.split(/\r?\n/u);
  const failures = lines.filter(line => line.startsWith('usb_startup ') && !line.includes('first_failure=none'));
  const errors = lines.filter(line => /^(?:[a-z_]*failure(?:_receipt|_context)?|rust_panic_receipt)(?: |$)/u.test(line));
  check(errors.length > 0 && errors.every(line => line === 'storage_http_failure schema=v1 phase=http_server error=http_task redacted=true') &&
    failures.length > 0 && failures.every(line => /^usb_startup schema=v1 stage=(?:storage_http|network|worker_control|statistics|runtime_ready) state=(?:entered|failed|complete) first_failure=storage_http uptime_ms=[0-9]+ redacted=true$/u.test(line)), 'panic_failed_install_http_boundary');
  return { installation_verified: false, installation_complete: false, recovery_only: true,
    continuity_basis: 'current-session-only', historical_resource_proof: false, parity_promotion: false };
}

export function validateTimedOutInstallation(input) {
  validateInstallBinding(input, true);
  const lines = input.stdout.split(/\r?\n/u);
  const exit = lines.indexOf('application_exit_transport: serial_jtag_runtime');
  check(exit >= 0 && lines.lastIndexOf('application_exit_transport: serial_jtag_runtime') === exit &&
    ['manifest: protected-operational', 'flash_image: protected-operational', 'flash_command: protected-operational'].every(marker => lines.indexOf(marker) >= 0 && lines.indexOf(marker) < exit) &&
    !lines.some(line => line.startsWith('nvs_seed_')), 'panic_timed_out_application_return');
  return { installation_verified: false, installation_complete: false, recovery_only: true,
    predecessor_failure: 'monitor_timeout_after_reported_application_return', application_return_reported: true,
    installed_identity_authenticated: false, continuity_basis: 'current-session-only', historical_resource_proof: false, parity_promotion: false };
}

export async function recoveryPredecessor(root, firmwareRoot) {
  ignored(firmwareRoot, root); await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const context = await proof(root, 'context.json'), claim = await proof(root, 'install-claim.json');
  const runner = await proof(root, 'install-runner.json'), owner = await proof(root, 'server-owner.json');
  const recovery = await proof(root, 'current-recovery.json');
  admitRecovery(recovery.value, context.value, claim.value.started_at_unix_ms);
  const input = { context: context.value, contextDigest: context.sha256, claim: { ...claim.value, root },
    runner: runner.value, ownerDigest: owner.sha256, owner: owner.value, recoveryDigest: recovery.sha256 };
  let review, receiptDigest = null, logDigest, stdoutDigest = null;
  if (runner.value.timed_out === true) {
    await missing(resolve(root, 'install/flash-command-evidence.private.json'));
    await missing(resolve(root, 'install/flash-monitor.classifier-input.log'));
    const stdoutPath = resolve(root, 'install.stdout.log'); await protectedPath(stdoutPath);
    check((await stat(stdoutPath)).size <= 8_388_608, 'panic_install_stdout_bound');
    const stdout = await readFile(stdoutPath, 'utf8'); stdoutDigest = sha256(stdout); logDigest = stdoutDigest;
    review = validateTimedOutInstallation({ ...input, stdout });
  } else {
    const receipt = await proof(root, 'install/flash-command-evidence.private.json'); receiptDigest = receipt.sha256;
    const logPath = resolve(root, 'install/flash-monitor.classifier-input.log'); await protectedPath(logPath);
    logDigest = await fileDigest(logPath);
    review = validateFailedInstallation({ ...input, receipt: receipt.value, logDigest, log: await readFile(logPath, 'utf8') });
  }
  const audit = await proof(root, 'native-audit.json');
  check(audit.sha256 === context.value.nativeAuditSha256, 'panic_failed_install_audit_digest');
  validateNativeAudit(audit.value, context.value.app_elf_sha256);
  return { context: context.value, audit, review, root, seal_sha256: seal.sha256, context_sha256: context.sha256,
    claim_sha256: claim.sha256, runner_sha256: runner.sha256, receipt_sha256: receiptDigest, log_sha256: logDigest, stdout_sha256: stdoutDigest };
}


export async function retainedPackage(manifest, predecessor, firmwareRoot) {
  ignored(firmwareRoot, manifest); await privateRoot(dirname(manifest)); await protectedPath(manifest);
  const expected = predecessor.context;
  check(await fileDigest(manifest) === expected.manifest_sha256, 'panic_retained_manifest_digest');
  const packaged = await inspectPackage(manifest, expected.firmware_commit, kind => kind === 'partition_table' ? firmwareRoot : dirname(manifest));
  check(packaged.app_elf_sha256 === expected.app_elf_sha256 && packaged.reference_commit === expected.reference_commit, 'panic_retained_package_identity');
  const raw = JSON.parse(await readFile(manifest, 'utf8'));
  for (const artifact of raw.artifacts.filter(item => item.kind !== 'partition_table')) await protectedPath(resolve(dirname(manifest), artifact.path));
  const elf = raw.artifacts.find(item => item.kind === 'firmware_elf');
  return { packaged, candidateElf: resolve(dirname(manifest), elf.path), retainedManifest: manifest };
}

export async function beforeRecovery(root, firmwareRoot) {
  ignored(firmwareRoot, root); await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const saved = await proof(root, 'context.json'), result = (await proof(root, 'result.json')).value;
  const current = (await proof(root, 'current-recovery.json')).value, context = saved.value;
  check(context.recoveryOnly === true && result.complete === true && result.baseline_complete === true &&
    result.continuity_basis === 'current-session-only' && result.installation_complete === false &&
    current.source_commit === context.commit && current.firmware_commit === context.before_source.firmware_commit &&
    current.app_elf_sha256 === context.before_source.app_elf_sha256 && current.physical_identity_sha256 === context.detector.physical,
  'panic_before_recovery_proof');
  admitRecovery(current, context, current.observed_at_unix_ms);
  const predecessor = await recoveryPredecessor(context.failedInstall.root, firmwareRoot);
  check(current.firmware_commit === predecessor.context.firmware_commit && current.app_elf_sha256 === predecessor.context.app_elf_sha256, 'panic_before_recovery_installed_identity');
  check(predecessor.seal_sha256 === context.failedInstall.seal_sha256 && predecessor.context_sha256 === context.failedInstall.context_sha256,
    'panic_before_recovery_predecessor');
  return { context, predecessor, root, seal_sha256: seal.sha256, context_sha256: saved.sha256 };
}

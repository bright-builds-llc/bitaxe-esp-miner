import { spawn } from 'node:child_process';
import { open, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileDigest, missing, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { proof, writeNew } from '../str005-noise-serial/files.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from './detector.mjs';
import { validateLedger, requireExhaustedOriginal } from '../fixed-usb-qualification/iterative-contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';

export function admitRecovery(value, context, now = Date.now()) {
  check(value.schema === 'str005-current-recovery-proof-v1' && value.source_commit === context.commit &&
    value.gate_commit === context.gate_commit && value.firmware_commit === context.before_source.firmware_commit &&
    value.app_elf_sha256 === context.before_source.app_elf_sha256 && value.physical_identity_sha256 === context.detector.physical &&
    Number.isSafeInteger(value.observed_at_unix_ms) && now >= value.observed_at_unix_ms && now - value.observed_at_unix_ms <= 120000 &&
    ['safe_baseline', 'restoration_confirmed', 'device_lease_inactive', 'serial_ownership_released', 'preservation_matches', 'current_v2_idle']
      .every(key => value[key] === true) && value.mine_on_boot === false, 'panic_recovery_prerequisite');
  validateLedger(value.ledger); requireExhaustedOriginal(value.original_budget);
  check(value.ledger.pending === false, 'panic_pending_accounting');
}
export function flashArguments(root, context) {
  return ['flash-monitor', '--board', '205', '--port', context.detector.port, '--expected-physical-sha256', context.detector.physical, '--manifest', context.manifest,
    '--evidence-dir', resolve(root, 'install'), '--evidence-mode', 'dual', '--capture-timeout-seconds', '360'];
}
/** Bounded child ownership captures every byte privately and kills descendants on every exit. */
export async function runChild(program, args, root, timeoutMs = 480000) {
  const stdout = await open(resolve(root, 'install.stdout.log'), 'wx', 0o600);
  let maybeStderr;
  try {
    maybeStderr = await open(resolve(root, 'install.stderr.log'), 'wx', 0o600);
    return await new Promise(resolveResult => {
      const child = spawn(program, args, { cwd: root, detached: true, stdio: ['ignore', stdout.fd, maybeStderr.fd] });
      let timedOut = false, interrupted = false, settled = false;
      const kill = () => { if (child.pid) { try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; } } };
      const interrupt = () => { interrupted = true; kill(); };
      const timer = setTimeout(() => { timedOut = true; kill(); }, timeoutMs);
      process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
      const done = (code, spawnFailed) => {
        if (settled) return; settled = true; clearTimeout(timer); kill();
        process.removeListener('SIGINT', interrupt); process.removeListener('SIGTERM', interrupt);
        resolveResult({ code, spawn_failed: spawnFailed, timed_out: timedOut, interrupted, pid: child.pid ?? null });
      };
      child.once('error', () => done(null, true)); child.once('exit', code => done(code, false));
    });
  } finally { await stdout.close(); await maybeStderr?.close(); }
}
export async function install(root, context) {
  check(context.installEnabled === true, 'panic_install_disabled');
  const recovery = await proof(root, 'current-recovery.json'); admitRecovery(recovery.value, context);
  await missing(resolve(root, 'install')); await missing(resolve(root, 'install-runner.json'));
  const detectorPath = resolve(dirname(root), 'install-detector.stdout.log'); await protectedPath(detectorPath);
  const detector = await readFile(detectorPath, 'utf8');
  const fresh = parseDetector(detector, context.detector.physical, Date.now() - (await stat(detectorPath)).mtimeMs);
  const owner = (await proof(root, 'server-owner.json')).value;
  check(owner.physicalIdentitySha256 === fresh.physical, 'panic_install_server_identity');
  const runtimeContext = { ...context, detector: { ...context.detector, ...fresh } };
  requireNoHolders(fresh.port);
  check(await fileDigest(context.flashBinary) === context.flashBinarySha256 && await fileDigest(context.manifest) === context.manifest_sha256, 'panic_install_input_changed');
  const args = flashArguments(root, runtimeContext), started = Date.now();
  await writeNew(resolve(root, 'install-claim.json'), { schema: 'str005-panic-install-claim-v1', started_at_unix_ms: started,
    context_sha256: (await proof(root, 'context.json')).sha256, recovery_sha256: recovery.sha256,
    detector_sha256: sha256(detector), physical_identity_sha256: fresh.physical, port: fresh.port, server_owner_sha256: (await proof(root, 'server-owner.json')).sha256, program: context.flashBinary, argv: args, command_sha256: sha256(JSON.stringify(args)), binary_sha256: context.flashBinarySha256 });
  const outcome = await runChild(context.flashBinary, args, root);
  let released = false; try { requireNoHolders(fresh.port); released = true; } catch { /* The terminal record must retain failed cleanup. */ }
  await writeNew(resolve(root, 'install-runner.json'), { schema: 'str005-panic-install-runner-v1', ...outcome,
    started_at_unix_ms: started, finished_at_unix_ms: Date.now(), command_sha256: sha256(JSON.stringify(args)),
    binary_sha256: context.flashBinarySha256, serial_holders_absent: released });
  check(outcome.code === 0 && !outcome.spawn_failed && !outcome.timed_out && !outcome.interrupted && released, 'panic_install_failed');
  return inspectInstall(root, context);
}
export function validateFlashReceipt(f, context, runner, logDigest) {
  const a = f.fixed_serial_assessment;
  check(f.command_kind === 'flash-monitor' && f.board === '205' && f.flash_status === 'completed' &&
    f.capture_mode === 'noninteractive' && ['completed', 'timed_out_after_trusted_output'].includes(f.capture_status) &&
    f.monitor_evidence_status === 'trusted' && f.trusted_output === true && f.firmware_commit === context.firmware_commit &&
    f.observed_firmware_commit === context.firmware_commit && f.reference_commit === context.reference_commit &&
    f.observed_reference_commit === context.reference_commit && f.trust_basis === 'fixed_serial' && f.nvs_seed_status === 'not_provided' &&
    f.redaction_mode === 'dual' && f.capture_timeout_seconds === 360 && f.manifest_path === context.manifest &&
    f.private_log_role === 'classifier-input-private' && f.private_monitor_log_sha256 === logDigest &&
    typeof f.timestamp === 'string' && /^[0-9]+$/u.test(f.timestamp) && Number(f.timestamp) * 1000 + 999 >= runner.started_at_unix_ms &&
    Number(f.timestamp) * 1000 <= runner.finished_at_unix_ms && a?.execution_present === true && a.safe_baseline_confirmed === true &&
    a.startup_complete === true && a.startup_failed === false && a.stable_boot === true && Array.isArray(a.issues) && a.issues.length === 0,
  'panic_install_receipt');
}
export async function inspectInstall(root, context) {
  const claim = (await proof(root, 'install-claim.json')).value, runner = (await proof(root, 'install-runner.json')).value;
  const owner = await proof(root, 'server-owner.json');
  check(claim.physical_identity_sha256 === context.detector.physical && owner.value.physicalIdentitySha256 === context.detector.physical &&
    claim.server_owner_sha256 === owner.sha256 && typeof claim.port === 'string' && /^\/dev\/[A-Za-z0-9._/-]+$/u.test(claim.port), 'panic_install_physical_binding');
  const runtimeContext = { ...context, detector: { ...context.detector, port: claim.port } };
  check(claim.schema === 'str005-panic-install-claim-v1' && claim.context_sha256 === (await proof(root, 'context.json')).sha256 &&
    claim.recovery_sha256 === (await proof(root, 'current-recovery.json')).sha256 && runner.schema === 'str005-panic-install-runner-v1' &&
    runner.code === 0 && runner.spawn_failed === false && runner.timed_out === false && runner.interrupted === false &&
    runner.serial_holders_absent === true && runner.started_at_unix_ms === claim.started_at_unix_ms &&
    runner.finished_at_unix_ms >= runner.started_at_unix_ms && runner.binary_sha256 === context.flashBinarySha256 &&
    runner.command_sha256 === sha256(JSON.stringify(flashArguments(root, runtimeContext))) && claim.command_sha256 === runner.command_sha256,
  'panic_install_runner');
  const receipt = await proof(root, 'install/flash-command-evidence.private.json');
  const logPath = resolve(root, 'install/flash-monitor.classifier-input.log'); await protectedPath(logPath);
  const logDigest = await fileDigest(logPath); validateFlashReceipt(receipt.value, context, runner, logDigest);
  requireNoHolders(claim.port);
  return { installation_verified: true, flash_receipt_sha256: receipt.sha256, log_sha256: logDigest, exact_runtime_elf_verified: false };
}

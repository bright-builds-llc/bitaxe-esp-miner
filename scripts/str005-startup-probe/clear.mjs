import { readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileDigest, missing, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { proof, writeNew, inventory, retain } from '../str005-noise-serial/files.mjs';
import { privateProcess } from '../core-dump/process.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateRecovery, validateClearResult, privateBytes, sealed } from './capture.mjs';
import { requireNoHolders } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { LIMITS } from './contract.mjs';

export function clearArguments(root, context, port = context.detector.port) {
  return ['core-dump-clear', '--board', '205', '--port', port,
    '--expected-physical-sha256', context.physical, '--expected-installed-source', context.firmware_commit,
    '--expected-installed-elf', context.app_elf_sha256, '--recovery-proof', resolve(root, 'current-recovery.json'),
    '--preserved-dump', resolve(context.bindings.archiveRoot, context.bindings.archiveRelative), '--preserved-sha256', context.archiveSha,
    '--private-root', resolve(root, 'clear')];
}

/** Binds the official command's actual arguments and exit without changing its result schema. */
export async function runClear(root, context, operations = {}) {
  await missing(resolve(root, 'clear-launch.json')); await missing(resolve(root, 'clear'));
  const recoveryPath = resolve(root, 'current-recovery.json');
  const recovery = (await proof(root, 'current-recovery.json')).value;
  validateRecovery(recovery, context, context.physical, Date.now());
  check(recovery.source_commit === context.source_commit, 'startup_clear_recovery_source');
  const archive = resolve(context.bindings.archiveRoot, context.bindings.archiveRelative);
  check(await fileDigest(archive) === context.archiveSha && await fileDigest(context.flash_binary) === context.flash_sha256, 'startup_clear_inputs_changed');
  const detectorPath = resolve(dirname(root), 'clear-detector.stdout.log'); await protectedPath(detectorPath);
  const detectorBytes = await readFile(detectorPath), detectorTime = (await stat(detectorPath)).mtimeMs;
  const selected = parseDetector(detectorBytes.toString('utf8'), context.physical, Date.now() - detectorTime);
  (operations.requireNoHolders ?? requireNoHolders)(selected.port);
  await retain(resolve(root, 'clear-detector.log'), detectorBytes);
  const args = clearArguments(root, context, selected.port);
  const claim = { schema: 'str005-startup-clear-launch-v1', contextSha256: sha256(JSON.stringify(context)), source_commit: context.source_commit,
    physical_identity_sha256: context.physical, recoveryProofSha256: await fileDigest(recoveryPath), archiveSha256: context.archiveSha,
    binarySha256: context.flash_sha256, arguments: args, selectedPort: selected.port, detectorSha256: sha256(detectorBytes), detectorObservedAtUnixMs: detectorTime, startedAtUnixMs: Date.now() };
  await writeNew(resolve(root, 'clear-launch.json'), claim);
  try {
    await (operations.run ?? privateProcess)(context.flash_binary, args, root, 'clear-command', {
      PATH: process.env.PATH ?? '/usr/bin:/bin', HOME: process.env.HOME ?? '', LANG: 'C', LC_ALL: 'C',
    }, LIMITS.clearMs);
    await writeNew(resolve(root, 'clear-exit.json'), { schema: 'str005-startup-clear-exit-v1', launchSha256: await fileDigest(resolve(root, 'clear-launch.json')),
      code: 0, completedAtUnixMs: Date.now() });
  } catch (error) {
    let serialOwnershipReleased = false;
    try { for (const port of new Set([context.detector.port, selected.port])) (operations.requireNoHolders ?? requireNoHolders)(port); serialOwnershipReleased = true; } catch { serialOwnershipReleased = false; }
    await writeNew(resolve(root, 'clear-failure.json'), { schema: 'str005-startup-clear-failure-v1', launchSha256: await fileDigest(resolve(root, 'clear-launch.json')),
      complete: false, serialOwnershipReleased, firstFailure: ['decoder_timeout', 'decoder_interrupted', 'decoder_failed', 'decoder_spawn'].includes(error.message) ? error.message : 'clear_launch_failed' }); throw error;
  }
  await verifyClear(root, context, true);
  await writeNew(resolve(root, 'clear', 'sealed-inventory.json'), { files: await inventory(resolve(root, 'clear')) });
}
export async function verifyClear(root, context, allowUnsealed = false) {
  if (!allowUnsealed) await sealed(resolve(root, 'clear'));
  const launch = await proof(root, 'clear-launch.json'), exit = (await proof(root, 'clear-exit.json')).value;
  check(launch.value.schema === 'str005-startup-clear-launch-v1' && launch.value.contextSha256 === sha256(JSON.stringify(context)) &&
    launch.value.source_commit === context.source_commit && launch.value.physical_identity_sha256 === context.physical &&
    launch.value.archiveSha256 === context.archiveSha && launch.value.binarySha256 === context.flash_sha256 &&
    JSON.stringify(launch.value.arguments) === JSON.stringify(clearArguments(root, context, launch.value.selectedPort)) &&
    exit.schema === 'str005-startup-clear-exit-v1' && exit.launchSha256 === launch.sha256 && exit.code === 0 &&
    exit.completedAtUnixMs >= launch.value.startedAtUnixMs && exit.completedAtUnixMs - launch.value.startedAtUnixMs <= LIMITS.clearMs,
  'startup_clear_launch_unverified');
  const detectorBytes = await readFile(resolve(root, 'clear-detector.log'));
  const selected = parseDetector(detectorBytes.toString('utf8'), context.physical, launch.value.startedAtUnixMs - launch.value.detectorObservedAtUnixMs);
  check(selected.port === launch.value.selectedPort && sha256(detectorBytes) === launch.value.detectorSha256, 'startup_clear_detector_changed');
  check(launch.value.recoveryProofSha256 === await fileDigest(resolve(root, 'current-recovery.json')), 'startup_clear_recovery_changed');
  const child = resolve(root, 'clear');
  validateClearResult((await proof(child, 'result.private.json')).value, context);
  const original = await privateBytes(child, 'core-dump.private.bin', 974848), cleared = await privateBytes(child, 'cleared-core-dump.private.bin', 974848);
  check(sha256(original) === context.archiveSha && cleared.every(byte => byte === 255), 'startup_clear_readback');
  return { launchSha256: launch.sha256, clearedSha256: sha256(cleared) };
}

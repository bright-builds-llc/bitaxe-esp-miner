import { mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { proof, writeNew } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { flashArguments, runChild } from './install.mjs';

/** Execute the actual pinned CLI parser and package path with hardware branches disabled. */
export async function checkCommand(root, context) {
  const directory = resolve(root, 'command-check');
  await mkdir(directory, { mode: 0o700 });
  check(await fileDigest(context.flashBinary) === context.flashBinarySha256 &&
    await fileDigest(context.manifest) === context.manifest_sha256, 'panic_command_check_identity');
  const args = [...flashArguments(directory, context), '--dry-run'];
  const result = await runChild(context.flashBinary, args, directory, 60000);
  await writeNew(resolve(directory, 'runner.json'), { schema: 'str005-panic-command-check-v1', ...result,
    command_sha256: sha256(JSON.stringify(args)), binary_sha256: context.flashBinarySha256, manifest_sha256: context.manifest_sha256,
    device_effects: false });
  check(result.code === 0 && !result.spawn_failed && !result.timed_out && !result.interrupted, 'panic_command_check_failed');
  const record = (await proof(directory, 'install/flash-command-evidence.private.json')).value;
  check(record.command_kind === 'flash-monitor' && record.flash_status === 'dry_run' && record.capture_status === 'dry_run' &&
    record.redaction_mode === 'dual' && record.nvs_seed_status === 'not_provided' && record.trusted_output === false,
  'panic_command_check_effect');
  check(await fileDigest(context.flashBinary) === context.flashBinarySha256 &&
    await fileDigest(context.manifest) === context.manifest_sha256, 'panic_command_check_identity');
  return { runner_sha256: (await proof(directory, 'runner.json')).sha256, device_effects: false };
}

export async function verifyCommandCheck(root, context) {
  const saved = await proof(root, 'command-check/runner.json'), value = saved.value;
  check(saved.sha256 === context.commandCheckSha256 && value.schema === 'str005-panic-command-check-v1' &&
    value.code === 0 && value.spawn_failed === false && value.timed_out === false && value.interrupted === false && value.device_effects === false &&
    value.binary_sha256 === context.flashBinarySha256 && value.manifest_sha256 === context.manifest_sha256 &&
    value.command_sha256 === sha256(JSON.stringify([...flashArguments(resolve(root, 'command-check'), context), '--dry-run'])) &&
    await fileDigest(context.flashBinary) === context.flashBinarySha256 && await fileDigest(context.manifest) === context.manifest_sha256,
  'panic_command_check_changed');
}

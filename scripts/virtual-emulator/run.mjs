import { readFile, copyFile, writeFile, chmod } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { createHash } from 'node:crypto';
import { main as decodeCore } from '../core-dump/main.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { EMULATOR } from './lock.mjs';
import { runPrivate } from './process.mjs';
import { judgeTargetEvents } from './judge.mjs';
import { MARKER, PROFILE } from './build.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

/** Pinned QEMU topology; no physical transport, networking, monitor socket or credentials. */
export function qemuArguments(image, efuse) {
  return ['-M', 'esp32s3', '-smp', '2', '-m', '8M',
    '-global', 'driver=ssi_psram,property=is_octal,value=true',
    '-drive', `file=${image},if=mtd,format=raw`,
    '-drive', `file=${efuse},if=none,format=raw,id=efuse`,
    '-global', 'driver=nvram.esp32s3.efuse,property=drive,value=efuse',
    '-global', 'driver=timer.esp32s3.timg,property=wdt_disable,value=true',
    '-nographic', '-serial', 'mon:stdio', '-nic', 'none'];
}

/** A reset-only private flash copy keeps each run independent from frozen packages. */
export async function runGuest(repo, packagePath, evidence, { commands = ['status', 'allocation'], timeoutMs = 10000, scenario, seed = 1 } = {}) {
  if (!Number.isSafeInteger(seed) || seed < 0 || (scenario !== undefined && (typeof scenario !== 'string' || !/^[a-z0-9-]{1,80}$/.test(scenario)))) throw Error('guest_scenario_arguments');
  if (commands.length > 16 || commands.some(command => !['status', 'allocation', 'restart', 'panic'].includes(command))) throw Error('guest_command_unsupported');
  const packageBytes = await readFile(packagePath);
  const manifest = JSON.parse(packageBytes);
  const packageRoot = dirname(packagePath);
  if (manifest.schema !== 'bitaxe-virtual-package-v1' || manifest.execution_profile !== PROFILE || manifest.hardware_eligible !== false) throw Error('guest_package_profile');
  const elf = await readFile(join(packageRoot, manifest.virtual_elf));
  if (![manifest.virtual_elf, manifest.flash_image].every(name => typeof name === 'string' && /^[a-zA-Z0-9_.-]+$/.test(name))) throw Error('guest_artifact_path');
  const image = await readFile(join(packageRoot, manifest.flash_image));
  if (!elf.includes(Buffer.from(MARKER)) || sha(elf) !== manifest.virtual_elf_sha256 || sha(image) !== manifest.image_sha256 || image.length !== 16 * 1024 * 1024) throw Error('guest_package_digest');
  await doctor(repo, evidence);
  const tools = managedPaths(repo);
  const efuse = await writeSdkEfuse(repo, evidence);
  const flash = join(evidence, 'virtual-flash-run.bin');
  await copyFile(join(packageRoot, manifest.flash_image), flash);
  const processResult = await runPrivate(tools.binary, qemuArguments(flash, efuse), evidence, 'qemu', {
    timeoutMs, allowTimeout: true, inputReadyMarker: '"event":"task"',
    input: [...commands.map(command => ({ command })), ...(scenario ? [{ command: 'scenario', scenario, seed }] : [])].map(value => JSON.stringify(value)).join('\n') + '\n' });
  const log = await readFile(join(evidence, 'qemu.stdout.log'), 'utf8');
  const events = log.split(/\r?\n/).filter(line => line.startsWith('VIRTUAL_U205 ')).map(line => JSON.parse(line.slice(13)));
  const boot = events.find(value => value.event === 'boot');
  const required = ['boot', 'task', ...(scenario ? ['scenario', 'scenario_margin'] : []), ...commands.filter(command => command !== 'panic'), ...(commands.includes('panic') ? ['panic_requested'] : [])];
  const missing = required.filter(event => !events.some(value => value.event === event));
  const checks = judgeTargetEvents(events, manifest, { commands, scenario, seed });
  let maybeCoreInspection = null;
  if (commands.includes('panic')) {
    const coreBytes = (await readFile(flash)).subarray(0xf12000, 0x1000000);
    if (coreBytes.every(byte => byte === 0xff)) {
      checks.push({ id: 'target_panic_core_decoding', status: 'failed', category: 'panic_partition_empty' });
    } else {
      const dump = join(evidence, 'virtual-core.raw');
      await writeFile(dump, coreBytes, { flag: 'wx', mode: 0o600 });
      const archivedElf = join(evidence, 'virtual-ultra205.elf');
      await copyFile(join(packageRoot, manifest.virtual_elf), archivedElf); await chmod(archivedElf, 0o600);
      try {
        maybeCoreInspection = await decodeCore(['analyze', '--dump', dump, '--elf', archivedElf,
          '--elf-sha256', manifest.virtual_elf_sha256, '--private-root', join(evidence, 'decoded-core')]);
        checks.push({ id: 'target_panic_core_decoding', status: maybeCoreInspection.checksum_verified && maybeCoreInspection.full_elf_identity_verified ? 'passed' : 'failed' });
      } catch (error) {
        checks.push({ id: 'target_panic_core_decoding', status: 'failed', category: 'private_core_decode_failed' });
      }
    }
  }
  const verdict = checks.some(check => check.status === 'failed') || missing.length ? 'failed' : checks.some(check => check.status === 'unsupported') ? 'unsupported' : 'passed';
  const result = { schema: 'bitaxe-virtual-target-result-v1', backend: 'qemu', emulator: EMULATOR.version,
    virtual_package_sha256: sha(packageBytes), paired_production_elf_sha256: manifest.production_elf_sha256,
    compiled_source_sha256: manifest.compiled_source_sha256, target_elf_sha256: manifest.virtual_elf_sha256,
    required_events: required, missing_events: missing, checks,
    target_boot_observed: boot?.execution_profile === PROFILE, actual_target_measurements: events,
    maybe_core_inspection: maybeCoreInspection, process_released: processResult.released, status: verdict,
    unexpected_panic: !commands.includes('panic') && /Guru Meditation|assert failed|Stack canary|stack overflow/.test(log),
    qualification: 'target_component_probe_only', full_board_qualified: false, hardware_eligible: false };
  await writeFile(join(evidence, 'target-result.json'), `${JSON.stringify(result, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  return result;
}

/** Use the exact pinned SDK default virtual eFuses, never physical identities. */
export async function writeSdkEfuse(repo, evidence) {
  const tools = managedPaths(repo);
  const extension = await readFile(join(tools.idf, 'tools/idf_py_actions/qemu_ext.py'), 'utf8');
  const section = extension.slice(extension.indexOf("'esp32s3': QemuTarget("));
  const hex = section.slice(section.indexOf('binascii.unhexlify('), section.indexOf("'-global driver=esp32s3.gpio"))
    .match(/'[a-f0-9]{40,}'/g)?.map(value => value.slice(1, -1)).join('');
  if (!hex || hex.length !== 2048) throw Error('guest_sdk_efuse_shape');
  const efuse = join(evidence, 'virtual-efuse.bin');
  await writeFile(efuse, Buffer.from(hex, 'hex'), { flag: 'wx', mode: 0o600 });
  return efuse;
}

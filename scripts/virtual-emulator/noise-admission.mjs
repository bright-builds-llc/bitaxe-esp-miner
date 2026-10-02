import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { MARKER, PROFILE } from './build.mjs';
import { validateNoiseAudit } from './noise-stack-audit.mjs';

const digest = value => createHash('sha256').update(value).digest('hex');
const configRequired = ['CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=2048',
  'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304', 'CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y'];

/** Read-only admission binds actual bytes and the current auditor, before any emulator effect. */
export async function admitNoisePackage(packagePath, auditPath, currentSourceSha256, objdumpPath) {
  const packageBytes = await readFile(packagePath), manifest = JSON.parse(packageBytes);
  if (manifest.schema !== 'bitaxe-virtual-package-v1' || manifest.execution_profile !== PROFILE ||
      manifest.hardware_eligible !== false || manifest.sdk !== 'v5.5.4') throw Error('noise_package_profile');
  if (![manifest.virtual_elf, manifest.flash_image].every(name => typeof name === 'string' && /^[a-zA-Z0-9_.-]+$/.test(name))) throw Error('noise_artifact_path');
  const root = dirname(packagePath);
  const elf = await readFile(join(root, manifest.virtual_elf)), image = await readFile(join(root, manifest.flash_image));
  const config = await readFile(join(root, 'virtual-ultra205.sdkconfig'));
  if (elf.length < 52 || elf.readUInt32BE(0) !== 0x7f454c46 || elf[4] !== 1 || elf[5] !== 1 || elf.readUInt16LE(18) !== 94 || !elf.includes(Buffer.from(MARKER)) || !elf.includes(Buffer.from(currentSourceSha256)) || digest(elf) !== manifest.virtual_elf_sha256 ||
      digest(image) !== manifest.image_sha256 || digest(config) !== manifest.virtual_sdkconfig_sha256 || image.length !== 16777216) throw Error('noise_package_digest');
  if (manifest.board_profile?.cores !== 2 || manifest.board_profile.flash_bytes !== 16777216 || manifest.board_profile.psram_bytes !== 8388608 || manifest.board_profile.psram_mode !== 'octal') throw Error('noise_board_profile');
  if (manifest.compiled_source_sha256 !== currentSourceSha256 || !/^[a-f0-9]{64}$/.test(currentSourceSha256)) throw Error('noise_source_identity');
  if (!image.subarray(0xf12000, 0x1000000).every(byte => byte === 0xff)) throw Error('noise_initial_core_not_empty');
  if (!configRequired.every(line => config.toString('utf8').split('\n').includes(line))) throw Error('noise_config_contract');
  const auditBytes = await readFile(auditPath), audit = JSON.parse(auditBytes);
  if (typeof objdumpPath !== 'string' || digest(await readFile(objdumpPath)) !== audit.bindings?.objdump_sha256) throw Error('noise_objdump_identity');
  const disassembly = await readFile(`${auditPath}.disassembly.private`, 'utf8');
  await validateNoiseAudit(audit, { elfSha256: digest(elf), sdkconfigSha256: digest(config), compiledSourceSha256: currentSourceSha256 }, { disassembly, sdkconfig: config.toString('utf8') });
  return { manifest, packageBytes, elf, image, config, audit, auditBytes };
}


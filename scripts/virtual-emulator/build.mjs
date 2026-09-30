import { createHash } from 'node:crypto';
import { readFile, writeFile, copyFile, mkdir } from 'node:fs/promises';
import { join, resolve, dirname } from 'node:path';
import { execFileSync } from 'node:child_process';
import { auditGuestFrames } from './frame-audit.mjs';
import { sourceSnapshot } from './identity.mjs';
import { managedPaths } from './setup.mjs';
import { runPrivate } from './process.mjs';

export const PROFILE = 'virtual-ultra205';
export const MARKER = 'BITAXE_EXECUTION_PROFILE=virtual-ultra205';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

/** Isolate the guest SDK configuration; production defaults and partitions are read-only. */
export async function buildGuest(repo, evidence, productionManifest) {
  const pairedBytes = await readFile(productionManifest);
  const paired = JSON.parse(pairedBytes);
  if (paired.image_metadata?.esp_idf_version !== 'v5.5.4' || paired.image_metadata?.board !== '205' ||
      !/^[a-f0-9]{64}$/.test(paired.app_elf_sha256) || paired.execution_profile === PROFILE) throw Error('guest_production_pair');
  const maybePairedElf = paired.artifacts?.find(artifact => artifact.kind === 'firmware_elf');
  if (!maybePairedElf || typeof maybePairedElf.path !== 'string') throw Error('guest_production_elf');
  const pairedElf = await readFile(resolve(dirname(productionManifest), maybePairedElf.path));
  if (digest(pairedElf) !== paired.app_elf_sha256 || maybePairedElf.sha256 !== paired.app_elf_sha256) throw Error('guest_production_elf_digest');
  const pairedConfig = await readFile(join(dirname(productionManifest), 'bitaxe-firmware.sdkconfig'), 'utf8');
  const source = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim();
  const snapshotBefore = await sourceSnapshot(repo);
  const compiledSourceDigest = snapshotBefore.sha256;
  await writeFile(join(evidence, 'compiled-inputs.json'), JSON.stringify(snapshotBefore), { flag: 'wx', mode: 0o600 });
  const base = await readFile(join(repo, 'firmware/bitaxe/sdkconfig.defaults'), 'utf8');
  const defaults = base.split('\n').filter(line => !line.startsWith('CONFIG_GPIO_ASIC_') && !line.startsWith('CONFIG_PARTITION_TABLE_CUSTOM_FILENAME=')).join('\n') +
    `\nCONFIG_PARTITION_TABLE_CUSTOM_FILENAME="${join(repo, 'firmware/bitaxe/partitions-ultra205.csv')}"\nCONFIG_APP_PROJECT_VER_FROM_CONFIG=y\nCONFIG_APP_PROJECT_VER="VIRTUAL-ultra205-${source.slice(0, 12)}"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n`;
  const defaultsFile = join(evidence, 'sdkconfig.defaults');
  await writeFile(defaultsFile, defaults, { flag: 'wx', mode: 0o600 });
  // espup's environment owns the Xtensa toolchain. No environment text enters evidence.
  const shellEntries = execFileSync('/bin/zsh', ['-lc', 'source "$HOME/export-esp.sh" >/dev/null 2>&1 && printf "%s\\0%s\\0" "$PATH" "$LIBCLANG_PATH"'], { encoding: 'utf8' });
  const [shellPath, shellLibclang, tail] = shellEntries.split('\0');
  if (!shellPath || !shellLibclang || tail !== '') throw Error('guest_esp_environment');
  const shellEnv = { PATH: shellPath, LIBCLANG_PATH: shellLibclang };
  const gccManifest = JSON.parse(await readFile(join(managedPaths(repo).idf, 'tools/tools.json'), 'utf8'));
  const gcc = gccManifest.tools.find(tool => tool.name === 'xtensa-esp-elf')?.versions.find(version => version.status === 'recommended');
  if (!gcc) throw Error('guest_sdk_compiler_missing');
  const espPath = `${join(repo, '.embuild/espressif/tools/xtensa-esp-elf', gcc.name, 'xtensa-esp-elf/bin')}:${shellEnv.PATH}`;
  const target = join(repo, 'target/virtual-ultra205');
  await mkdir(target, { recursive: true });
  const env = { BITAXE_VIRTUAL_SOURCE_DIGEST: compiledSourceDigest, PATH: espPath, LIBCLANG_PATH: shellEnv.LIBCLANG_PATH, HOME: process.env.HOME, MCU: 'esp32s3',
    CARGO_TARGET_DIR: target, ESP_IDF_SDKCONFIG: join(evidence, 'sdkconfig'),
    ESP_IDF_SDKCONFIG_DEFAULTS: defaultsFile, ESP_IDF_SYS_ROOT_CRATE: 'bitaxe-virtual-firmware',
    ESP_IDF_TOOLS_INSTALL_DIR: 'workspace', ESP_IDF_VERSION: 'tag:v5.5.4',
    CC_xtensa_esp32s3_espidf: 'xtensa-esp32s3-elf-gcc', CFLAGS_xtensa_esp32s3_espidf: '-mlongcalls',
    AR_xtensa_esp32s3_espidf: 'xtensa-esp32s3-elf-ar', CARGO_PROFILE_RELEASE_DEBUG: '2', CARGO_PROFILE_RELEASE_STRIP: 'none' };
  await runPrivate('cargo', ['build', '-p', 'bitaxe-virtual-firmware', '--release', '--target', 'xtensa-esp32s3-espidf', '--message-format=json-render-diagnostics'], evidence, 'build', { cwd: repo, env, timeoutMs: 1200000 });
  const elf = join(evidence, 'virtual-ultra205.elf');
  await copyFile(join(target, 'xtensa-esp32s3-espidf/release/bitaxe-virtual-firmware'), elf);
  const snapshotAfter = await sourceSnapshot(repo);
  if (snapshotAfter.sha256 !== compiledSourceDigest) {
    const beforeInputs = new Map(snapshotBefore.inputs.map(input => [input.path, input.sha256]));
    const changed = snapshotAfter.inputs.filter(input => beforeInputs.get(input.path) !== input.sha256).map(input => input.path);
    await writeFile(join(evidence, 'source-changes.json'), JSON.stringify({ changed, before: compiledSourceDigest, after: snapshotAfter.sha256 }), { flag: 'wx', mode: 0o600 });
    throw Error('guest_source_changed_during_build');
  }
  const elfBytes = await readFile(elf);
  if (!elfBytes.includes(Buffer.from(MARKER))) throw Error('guest_marker_missing');
  const messages = (await readFile(join(evidence, 'build.stdout.log'), 'utf8')).split('\n')
    .filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const matches = messages.filter(message => message.reason === 'build-script-executed' &&
    message.package_id.includes('esp-idf-sys') && typeof message.out_dir === 'string');
  if (matches.length !== 1) throw Error('guest_generated_sdk_ambiguous');
  const generated = matches[0].out_dir;
  await copyFile(join(generated, 'sdkconfig'), join(evidence, 'virtual-ultra205.sdkconfig'));
  await runPrivate('espflash', ['save-image', '--chip', 'esp32s3', '--flash-size', '16mb', elf, join(evidence, 'app.bin')], evidence, 'save-image', { cwd: repo, env });
  const tools = managedPaths(repo);
  // SDK merge operation alone controls image layout and FF padding; no custom binary merger.
  await runPrivate(tools.python, ['-m', 'esptool', '--chip', 'esp32s3', 'merge_bin', '--output', join(evidence, 'virtual-flash.bin'), '--fill-flash-size', '16MB',
    '0x0', join(generated, 'build/bootloader/bootloader.bin'), '0x8000', join(generated, 'build/partition_table/partition-table.bin'),
    '0x10000', join(evidence, 'app.bin'), '0xf10000', join(generated, 'build/ota_data_initial.bin')], evidence, 'merge', { cwd: repo, env });
  const config = await readFile(join(evidence, 'virtual-ultra205.sdkconfig'));
  const actualConfig = config.toString('utf8');
  await runPrivate('xtensa-esp32s3-elf-objdump', ['-d', '-C', elf], evidence, 'guest-disassembly', { cwd: repo, env, timeoutMs: 120000 });
  const guestFrameAudit = auditGuestFrames(await readFile(join(evidence, 'guest-disassembly.stdout.log'), 'utf8'), actualConfig);
  await writeFile(join(evidence, 'guest-frame-audit.json'), JSON.stringify(guestFrameAudit), { flag: 'wx', mode: 0o600 });
  for (const required of ['CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y', 'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304', 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384']) {
    if (!actualConfig.split('\n').includes(required) || !pairedConfig.split('\n').includes(required)) throw Error('guest_resolved_config_contract');
  }
  const parseConfig = text => new Map(text.split('\n').filter(line => /^CONFIG_[A-Z0-9_]+=/.test(line)).map(line => [line.slice(0, line.indexOf('=')), line.slice(line.indexOf('=') + 1)]));
  const beforeConfig = parseConfig(pairedConfig), afterConfig = parseConfig(actualConfig);
  const configDifferences = [...new Set([...beforeConfig.keys(), ...afterConfig.keys()])].sort()
    .filter(key => beforeConfig.get(key) !== afterConfig.get(key))
    .map(key => ({ key, production: beforeConfig.get(key) ?? null, virtual: afterConfig.get(key) ?? null }));
  const result = { schema: 'bitaxe-virtual-package-v1', execution_profile: PROFILE, source_commit: source, compiled_source_sha256: compiledSourceDigest,
    source_dirty: execFileSync('git', ['status', '--porcelain'], { cwd: repo, encoding: 'utf8' }).length > 0,
    build_identity: { channel: PROFILE }, sdk: 'v5.5.4', production_manifest_sha256: digest(pairedBytes),
    production_elf_sha256: paired.app_elf_sha256, virtual_elf_sha256: digest(elfBytes), virtual_sdkconfig_sha256: digest(config),
    image_sha256: digest(await readFile(join(evidence, 'virtual-flash.bin'))),
    config_differences: configDifferences, production_sdkconfig_sha256: digest(pairedConfig),
    target_io_adapters: { adc_self_calibration: 'synthetic_uncalibrated_offset_2048' },
    guest_frame_audit: guestFrameAudit,
    board_profile: { cores: 2, flash_bytes: 16777216, psram_bytes: 8388608, psram_mode: 'octal' },
    virtual_elf: 'virtual-ultra205.elf', flash_image: 'virtual-flash.bin', hardware_eligible: false };
  await writeFile(join(evidence, 'virtual-package.json'), `${JSON.stringify(result, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  return result;
}

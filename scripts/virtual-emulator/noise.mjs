import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, readdir, mkdir, stat, realpath } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { MARKER, PROFILE } from './build.mjs';
import { sourceDigest } from './identity.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { qemuArguments, writeSdkEfuse } from './run.mjs';
import { runPrivate } from './process.mjs';
import { main as decodeCore } from '../core-dump/main.mjs';
import { validateNoiseAudit, validateNativeNoiseAudit } from './noise-stack-audit.mjs';
import { EMULATOR } from './lock.mjs';

export const NOISE_BASELINE_EFFECTS_ENABLED = false;
const digest = value => createHash('sha256').update(value).digest('hex');
export async function noiseValidatorIdentity(repo) {
  const hash = createHash('sha256');
  const files = ['noise.mjs', 'noise-stack-audit.mjs', 'process.mjs', 'run.mjs', 'setup.mjs', 'lock.mjs', 'build.mjs', 'identity.mjs',
    '../core-dump/main.mjs', '../core-dump/files.mjs', '../core-dump/decode_core.py', '../core-dump/process.mjs'];
  for (const file of files) { hash.update(file); hash.update('\0'); hash.update(await readFile(join(repo, 'scripts/virtual-emulator', file))); hash.update('\0'); }
  return hash.digest('hex');
}
const claimName = 'scratch/virtual-noise-diagnostic/baseline001.claim.json';
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

/** Fixed new ABI. Stack facts exist at stage1; heap facts remain unavailable until stage2. */
export function parseNoiseCheckpoints(raw) {
  if (raw.length !== 768) throw Error('noise_checkpoint_length');
  const rows = [];
  for (let offset = 0; offset < raw.length; offset += 48) {
    const words = Array.from({ length: 12 }, (_, i) => raw.readUInt32LE(offset + i * 4));
    if (words[0] === 0) continue;
    if (words[0] !== 0x564e5031 || words[1] < 101 || words[1] > 114 || ![1, 2].includes(words[2])) throw Error('noise_checkpoint_format');
    const complete = words[2] === 2;
    rows.push({ phase: words[1], stage: words[2], integrity: complete ? words[3] === 1 : null,
      stack_span_kind: 'configured', configured_main_stack_bytes: words[6],
      stack_low_water: words[10], stack_pointer_inside: words[11] === 1,
      internal_free: complete ? words[8] : null, internal_largest: complete ? words[9] : null });
  }
  return rows;
}

/** Only bounded ELF32 little-endian tables and the exact retained journal symbol are admitted. */
export function extractNoiseCheckpoints(elf, core) {
  const requireElf = bytes => {
    if (bytes.length < 52 || bytes.readUInt32BE(0) !== 0x7f454c46 || bytes[4] !== 1 || bytes[5] !== 1 || bytes.readUInt16LE(18) !== 94) throw Error('noise_checkpoint_elf');
  };
  const bounded = (bytes, offset, size) => {
    if (!Number.isSafeInteger(offset) || !Number.isSafeInteger(size) || offset < 0 || size < 0 || offset + size > bytes.length) throw Error('noise_checkpoint_elf_bounds');
    return bytes.subarray(offset, offset + size);
  };
  requireElf(elf); requireElf(core);
  const sections = [], sectionOffset = elf.readUInt32LE(32), sectionSize = elf.readUInt16LE(46), sectionCount = elf.readUInt16LE(48);
  if (sectionSize !== 40 || sectionCount > 4096) throw Error('noise_checkpoint_elf_sections');
  for (let i = 0; i < sectionCount; i++) sections.push(bounded(elf, sectionOffset + i * sectionSize, sectionSize));
  const symbols = [];
  for (const section of sections.filter(item => item.readUInt32LE(4) === 2)) {
    const linked = sections[section.readUInt32LE(24)];
    if (!linked || section.readUInt32LE(36) !== 16) throw Error('noise_checkpoint_symbols');
    const strings = bounded(elf, linked.readUInt32LE(16), linked.readUInt32LE(20));
    const table = bounded(elf, section.readUInt32LE(16), section.readUInt32LE(20));
    for (let i = 0; i + 16 <= table.length; i += 16) {
      const nameOffset = table.readUInt32LE(i), end = strings.indexOf(0, nameOffset);
      if (end < nameOffset) throw Error('noise_checkpoint_symbols');
      if (strings.toString('utf8', nameOffset, end) === 'BITAXE_VIRTUAL_NOISE_CHECKPOINTS') symbols.push({ address: table.readUInt32LE(i + 4), size: table.readUInt32LE(i + 8) });
    }
  }
  if (symbols.length !== 1 || symbols[0].size !== 768) throw Error('noise_checkpoint_symbol_missing');
  const offset = core.readUInt32LE(28), size = core.readUInt16LE(42), count = core.readUInt16LE(44);
  if (size !== 32 || count > 4096) throw Error('noise_checkpoint_core_headers');
  for (let i = 0; i < count; i++) {
    const header = bounded(core, offset + i * size, size);
    if (header.readUInt32LE(0) !== 1) continue;
    const start = header.readUInt32LE(8), bytes = header.readUInt32LE(16);
    if (symbols[0].address >= start && symbols[0].address + 768 <= start + bytes) {
      return parseNoiseCheckpoints(bounded(core, header.readUInt32LE(4) + symbols[0].address - start, 768));
    }
  }
  throw Error('noise_checkpoint_memory_unavailable');
}

/** Presence is insufficient: admission, completed real crypto, integrity and existing margin must all agree. */
export function judgeNoiseProbe(events, records, manifest, processResult, unexpectedPanic) {
  const boot = events.filter(event => event.event === 'boot');
  const probes = events.filter(event => event.event === 'noise_probe');
  const probe = probes[0];
  const checks = [
    ['boot_identity', boot.length === 1 && boot[0].execution_profile === PROFILE && boot[0].compiled_source_sha256 === manifest.compiled_source_sha256 && boot[0].heartbeat_cutoff_ms === 2800],
    ['noise_authenticated_round_trip', probes.length === 1 && probe.result?.schema === 'bitaxe-noise-probe-v1' && probe.result.seed === 1 && probe.result.authenticated === true && probe.result.frame_round_trip === true && probe.result.payload_bytes === 32 && probe.result.hardware_qualified === false],
    ['noise_resources_released', probes.length === 1 && probe.result?.resources_released === true],
    ['noise_phase_integrity', records.length === 14 && records.every((record, i) => record.phase === 101 + i && record.stage === 2 && record.integrity === true && record.stack_pointer_inside === true && record.stack_span_kind === 'configured' && record.configured_main_stack_bytes === 16384 && Number.isSafeInteger(record.internal_free) && Number.isSafeInteger(record.internal_largest) && record.internal_largest >= 0 && record.internal_largest <= record.internal_free)],
    ['noise_stack_margin', probe?.configured_main_stack_bytes === 16384 && probe.required_margin_bytes === 2048 && Number.isSafeInteger(probe.minimum_main_stack_free_bytes) && probe.minimum_main_stack_free_bytes >= 2048 && records.length === 14 && records.every(record => Number.isSafeInteger(record.stack_low_water) && record.stack_low_water >= 2048)],
    ['emulator_released', processResult?.released === true && processResult.interrupted === false && (processResult.timedOut === true || processResult.code === 0)],
    ['no_unexpected_panic', unexpectedPanic === false],
  ];
  return checks.map(([id, passed]) => ({ id, status: passed ? 'passed' : 'failed' }));
}

export async function requireNoNoiseWriters(root, repo) {
  for (let current = resolve(root); current.startsWith(`${resolve(repo)}/`); current = dirname(current)) {
    if ((await readdir(current)).some(name => name.endsWith('.writer.json'))) throw Error('noise_live_writer');
  }
}

/** One published, exclusive baseline. Emergency physical recovery is outside this command. */
export async function runNoiseProbe(repo, packagePath, root, { auditPath, seed = 1, mode = 'valid', disableEffects = false } = {}) {
  if (!NOISE_BASELINE_EFFECTS_ENABLED || disableEffects) throw Error('noise_effect_gate_disabled');
  if (seed !== 1 || mode !== 'valid' || typeof auditPath !== 'string') throw Error('noise_arguments');
  const tasks = await readFile(join(repo, 'TASKS.md'), 'utf8');
  const task = tasks.split('### task-ultra205-virtual-board-validation |')[1]?.split(/\n### |\n## Future/)[0];
  if (!task?.includes('noise-probe-baseline-enabled: true')) throw Error('noise_task_gate_disabled');
  const git = args => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
  if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== git(['rev-parse', 'origin/main'])) throw Error('noise_contract_not_clean_pushed');
  if (await realpath(root) !== resolve(root) || !resolve(root).startsWith(`${resolve(repo)}/`) || ((await stat(root)).mode & 0o077) !== 0) throw Error('noise_private_evidence_root');
  try { git(['check-ignore', '--quiet', '--', root]); } catch { throw Error('noise_private_evidence_not_ignored'); }
  await requireNoNoiseWriters(root, repo);
  const currentSource = await sourceDigest(repo), validatorSha256 = await noiseValidatorIdentity(repo);
  const sourceCommit = git(['rev-parse', 'HEAD']);
  const tools = managedPaths(repo), sdkTools = JSON.parse(await readFile(join(tools.idf, 'tools/tools.json'), 'utf8'));
  const compiler = sdkTools.tools.find(tool => tool.name === 'xtensa-esp-elf')?.versions.find(version => version.status === 'recommended');
  if (compiler?.name !== 'esp-14.2.0_20260121') throw Error('noise_objdump_pin');
  const objdumpPath = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', compiler.name, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
  const admitted = await admitNoisePackage(packagePath, auditPath, currentSource, objdumpPath);
  if (admitted.manifest.source_commit !== git(["rev-parse", "HEAD"]) || admitted.manifest.source_dirty !== false) throw Error("noise_package_not_current_clean");
  await validateNativeNoiseAudit(admitted.audit, join(dirname(packagePath), admitted.manifest.virtual_elf), repo);
  const claim = join(repo, claimName); await mkdir(dirname(claim), { recursive: true, mode: 0o700 });
  await writeFile(claim, JSON.stringify({ schema: 'bitaxe-noise-baseline-claim-v1', source_commit: git(['rev-parse', 'HEAD']),
    compiled_source_sha256: currentSource, package_sha256: digest(admitted.packageBytes), audit_sha256: digest(admitted.auditBytes), seed, mode }), { flag: 'wx', mode: 0o600 });
  const result = { schema: 'bitaxe-noise-diagnostic-result-v1', seed, mode, status: 'failed',
    package_sha256: digest(admitted.packageBytes), elf_sha256: admitted.manifest.virtual_elf_sha256,
    sdkconfig_sha256: admitted.manifest.virtual_sdkconfig_sha256, compiled_source_sha256: currentSource, source_commit: sourceCommit, validator_sha256: validatorSha256,
    audit_sha256: digest(admitted.auditBytes), emulator: EMULATOR.version, checkpoint_schema: 'bitaxe-noise-checkpoint-v1', checks: [], records: [],
    maybe_earliest_failure: null, cleanup_failures: [], full_board_qualified: false, hardware_eligible: false };
  let processResult, unexpectedPanic = false;
  const flash = join(root, 'virtual-flash-run.bin'), elfPath = join(root, 'virtual-ultra205.elf');
  try {
    await doctor(repo, root);
    await writeFile(flash, admitted.image, { flag: 'wx', mode: 0o600 });
    await writeFile(elfPath, admitted.elf, { flag: 'wx', mode: 0o600 });
    const efuse = await writeSdkEfuse(repo, root);
    if (await sourceDigest(repo) !== currentSource || await noiseValidatorIdentity(repo) !== validatorSha256) throw Error('noise_source_changed_before_effect');
    processResult = await runPrivate(managedPaths(repo).binary, qemuArguments(flash, efuse), root, 'qemu', {
      timeoutMs: 30000, allowTimeout: true, inputReadyMarker: '"event":"task"', input: `${JSON.stringify({ command: 'noise_probe', seed, mode })}\n` });
  } catch (error) {
    result.maybe_earliest_failure = { phase: 'execution', category: error.message };
    result.cleanup_failures.push(...(error.cleanupFailures ?? []).map(category => ({ phase: 'release', category })));
    try { processResult = JSON.parse(await readFile(join(root, 'qemu.process.json'), 'utf8')); } catch (error) { if (error.code !== 'ENOENT') result.cleanup_failures.push({ phase: 'collection', category: 'process_record_unavailable' }); }
  }
  try {
    const log = await readFile(join(root, 'qemu.stdout.log'), 'utf8');
    unexpectedPanic = /Guru Meditation|assert failed|Stack canary|stack overflow|abort\(\)/.test(log);
    if (unexpectedPanic) result.maybe_earliest_failure ??= { phase: 'noise', category: 'unexpected_target_panic' };
    const events = log.split(/\r?\n/).filter(line => line.startsWith('VIRTUAL_U205 ')).map(line => JSON.parse(line.slice(13)));
    const rejected = events.find(event => event.event === 'noise_probe_rejected');
    if (rejected) result.maybe_earliest_failure ??= { phase: 'noise', category: typeof rejected.category === 'string' && /^noise_[A-Za-z_]{1,80}$/.test(rejected.category) ? rejected.category : 'noise_rejection_unclassified' };
    result.records = events.find(event => event.event === 'noise_probe' || event.event === 'noise_probe_rejected')?.checkpoints ?? [];
    result.checks = judgeNoiseProbe(events, result.records, admitted.manifest, processResult, unexpectedPanic);
  } catch (error) {
    result.maybe_earliest_failure ??= { phase: 'collection', category: 'noise_collection_failed' };
    result.collection_failure = error.message;
  }
  // Core preservation and decoding do not depend on successful event parsing.
  if (unexpectedPanic) {
    result.maybe_earliest_failure ??= { phase: 'noise', category: 'unexpected_target_panic' };
    try {
      const core = (await readFile(flash)).subarray(0xf12000, 0x1000000);
      if (core.every(byte => byte === 0xff)) throw Error('noise_panic_partition_empty');
      const dump = join(root, 'virtual-core.raw'); await writeFile(dump, core, { flag: 'wx', mode: 0o600 });
      result.core_sha256 = digest(core);
      const decoded = join(root, 'decoded-core');
      const inspection = await decodeCore(['inspect', '--dump', dump, '--elf', elfPath, '--elf-sha256', admitted.manifest.virtual_elf_sha256, '--private-root', decoded]);
      result.core_verified = inspection.checksum_verified === true && inspection.full_elf_identity_verified === true;
      if (!result.core_verified) throw Error('noise_core_identity');
      result.records = extractNoiseCheckpoints(admitted.elf, await readFile(join(decoded, 'core.elf')));
    } catch (error) {
      result.core_collection_failure = error.message;
    }
  }
  result.process_released = processResult?.released === true;
  result.emulator_termination = processResult?.timedOut ? 'bounded_collection_cutoff' : 'natural_exit';
  result.natural_process_completion = processResult?.timedOut === false && processResult?.code === 0;
  await writeFile(join(root, 'noise-partial-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  await finalizeNoiseResult(result, root, repo, async () => {
    if (await sourceDigest(repo) !== currentSource || await noiseValidatorIdentity(repo) !== validatorSha256) throw Error('noise_source_changed_after_effect');
  });
  return result;
}


/** Finalization proves release and identity; failed persistence cannot leave a passed conclusion. */
export async function finalizeNoiseResult(result, root, repo, verifyIdentity) {
  try {
    await verifyIdentity();
    await requireNoNoiseWriters(root, repo);
    if (result.checks.length === 7 && result.checks.every(check => check.status === 'passed') && !result.maybe_earliest_failure && result.cleanup_failures.length === 0) result.status = 'passed';
    else result.maybe_earliest_failure ??= { phase: 'validation', category: result.checks.find(check => check.status === 'failed')?.id ?? 'noise_required_facts_unavailable' };
    await writeFile(join(root, 'noise-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  } catch (error) {
    result.status = 'failed';
    result.maybe_earliest_failure ??= { phase: 'finalization', category: error.message };
    result.cleanup_failures.push({ phase: 'finalization', category: error.message });
    throw Object.assign(Error(result.maybe_earliest_failure.category), { result });
  }
  return result;
}

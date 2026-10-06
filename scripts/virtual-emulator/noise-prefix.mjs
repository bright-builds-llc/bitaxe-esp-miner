import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, stat, realpath, readdir } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { EMULATOR } from './lock.mjs';
import { sourceDigest } from './identity.mjs';
import { MARKER, PROFILE } from './build.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { qemuArguments, writeSdkEfuse } from './topology.mjs';
import { runPrivate } from './process.mjs';
import { validatePrefixNoiseAudit } from './noise-prefix-audit.mjs';
import { validateNativeNoiseAudit } from './noise-stack-audit.mjs';
export { judgePrefixSnapshots, qualificationFailure, prefixLogFacts } from './noise-prefix-history.mjs';

export const NOISE_PREFIX_EFFECTS_ENABLED = false;
export const PREFIX_STOPS = Object.freeze([101, 102, 103, 105, 106, 107, 109, 110]);
const sha = value => createHash('sha256').update(value).digest('hex');

const failureCategories = new Set(['noise_prefix_operation_failed', 'noise_prefix_source_changed_before_effect',
  'noise_prefix_source_changed_after_effect', 'noise_prefix_series_binding_changed', 'noise_prefix_series_exhausted',
  'noise_prefix_claim_permissions', 'noise_live_writer', 'emulator_manifest_version', 'emulator_manifest_digest',
  'emulator_binary_version', 'emulator_spawn', 'emulator_command_failed', 'emulator_output_bound',
  'emulator_line_bound', 'prefix_projection_invalid', 'prefix_application_record_invalid', 'emulator_process_group_alive', 'emulator_log_release_unproven', 'emulator_writer_release_unproven']);
export function redactedPrefixFailure(error) {
  return failureCategories.has(error?.message) ? error.message : 'noise_prefix_operation_failed';
}

export async function prefixValidatorIdentity(repo) {
  const hash = createHash('sha256');
  const files = ['noise-prefix.mjs', 'noise-prefix-audit.mjs', 'noise-stack-audit.mjs',
    'elf-symbols.mjs', 'identity.mjs', 'build.mjs', 'frame-audit.mjs', 'setup.mjs', 'noise-prefix-history.mjs',
    'verify_install.py', 'lock.mjs', 'topology.mjs', 'process.mjs', 'main.mjs', 'judge.mjs'];
  for (const name of files) {
    hash.update(name); hash.update('\0');
    hash.update(await readFile(join(repo, 'scripts/virtual-emulator', name))); hash.update('\0');
  }
  return hash.digest('hex');
}

async function requireNoPrefixWriters(root, repo) {
  for (let current = resolve(root); current.startsWith(`${resolve(repo)}/`); current = dirname(current)) {
    if ((await readdir(current)).some(name => name.endsWith('.writer.json'))) throw Error('noise_live_writer');
  }
}

export function validatePrefixCommit(manifest, sourceCommit) {
  if (manifest.source_dirty !== false || manifest.source_commit !== sourceCommit || !/^[a-f0-9]{40}$/.test(sourceCommit ?? '')) throw Error('noise_prefix_commit_binding');
}

/** Actual bytes, existing memory routing and cutoff-aware paths are admitted before ownership. */
export async function admitPrefixPackage(packagePath, auditPath, sourceSha256, stop, sourceCommit) {
  if (!PREFIX_STOPS.includes(stop)) throw Error('noise_prefix_stop_unsupported');
  const packageBytes = await readFile(packagePath), manifest = JSON.parse(packageBytes);
  if (manifest.schema !== 'bitaxe-virtual-package-v1' || manifest.execution_profile !== PROFILE || manifest.hardware_eligible !== false || manifest.sdk !== 'v5.5.4') throw Error('noise_prefix_package_profile');
  if (![manifest.virtual_elf, manifest.flash_image].every(name => typeof name === 'string' && /^[a-zA-Z0-9_.-]+$/.test(name))) throw Error('noise_prefix_artifact_path');
  const root = dirname(packagePath), elf = await readFile(join(root, manifest.virtual_elf)), image = await readFile(join(root, manifest.flash_image)), config = await readFile(join(root, 'virtual-ultra205.sdkconfig'));
  if (elf.length < 52 || elf.readUInt32BE(0) !== 0x7f454c46 || elf[4] !== 1 || elf[5] !== 1 || elf.readUInt16LE(18) !== 94 || !elf.includes(Buffer.from(MARKER)) || !elf.includes(Buffer.from(sourceSha256)) || sha(elf) !== manifest.virtual_elf_sha256 || sha(image) !== manifest.image_sha256 || sha(config) !== manifest.virtual_sdkconfig_sha256 || image.length !== 16777216) throw Error('noise_prefix_artifact_binding');
  validatePrefixCommit(manifest, sourceCommit);
  if (manifest.compiled_source_sha256 !== sourceSha256 || !/^[a-f0-9]{64}$/.test(sourceSha256)) throw Error('noise_prefix_source_binding');
  const required = ['CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0', 'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304', 'CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y'];
  if (!required.every(line => config.toString().split('\n').includes(line)) || manifest.board_profile?.cores !== 2 || manifest.board_profile.flash_bytes !== 16777216 || manifest.board_profile.psram_bytes !== 8388608 || manifest.board_profile.psram_mode !== 'octal') throw Error('noise_prefix_memory_contract');
  const auditBytes = await readFile(auditPath), audit = JSON.parse(auditBytes), disassembly = await readFile(`${auditPath}.disassembly.private`, 'utf8');
  await validatePrefixNoiseAudit(audit, { elfSha256: sha(elf), sdkconfigSha256: sha(config), compiledSourceSha256: sourceSha256 }, { disassembly, sdkconfig: config.toString(), elf }, stop);
  return { manifest, packageBytes, elf, image, config, audit, auditBytes };
}

/** A fixed series admits at most one execution per cutoff and rejects image/validator drift. */
export async function claimPrefix(seriesRoot, binding, stop, auditSha256) {
  if (!PREFIX_STOPS.includes(stop)) throw Error('noise_prefix_stop_unsupported');
  await mkdir(seriesRoot, { recursive: true, mode: 0o700 });
  if (await realpath(seriesRoot) !== resolve(seriesRoot) || ((await stat(seriesRoot)).mode & 0o077) !== 0) throw Error('noise_prefix_claim_permissions');
  const bytes = JSON.stringify({ schema: 'bitaxe-noise-prefix-series-v1', ...binding });
  const series = join(seriesRoot, 'series.claim.json');
  try { await writeFile(series, bytes, { flag: 'wx', mode: 0o600 }); }
  catch (error) {
    if (error.code !== 'EEXIST') throw error;
    if (await readFile(series, 'utf8') !== bytes) throw Error('noise_prefix_series_binding_changed');
  }
  const attempts = (await readdir(seriesRoot)).filter(name => /^cutoff-[0-9]+\.claim\.json$/.test(name));
  if (attempts.length >= 8) throw Error('noise_prefix_series_exhausted');
  await writeFile(join(seriesRoot, `cutoff-${stop}.claim.json`), JSON.stringify({ ...binding, selected_stop: stop, audit_sha256: auditSha256 }), { flag: 'wx', mode: 0o600 });
}

/** Runtime comparison never promotes incomplete crypto into authentication or full qualification. */
export function judgePrefix(events, stop, sourceSha256) {
  const boot = events.filter(event => event.event === 'boot'), completed = events.filter(event => event.event === 'noise_prefix'), event = completed[0], result = event?.result;
  const rows = event?.checkpoints ?? [], expected = [...Array.from({ length: stop - 100 }, (_, i) => i + 101), 114];
  return [
    ['boot_identity', boot.length === 1 && boot[0].execution_profile === PROFILE && boot[0].compiled_source_sha256 === sourceSha256 && boot[0].heartbeat_cutoff_ms === 2800],
    ['selected_prefix_completed', completed.length === 1 && result?.schema === 'bitaxe-noise-prefix-v1' && result.seed === 1 && result.selected_stop === stop && result.boundary_reached === true],
    ['subset_only', result?.subset_only === true && result.authenticated === false && result.frame_round_trip === false && result.full_probe_qualified === false && result.hardware_qualified === false],
    ['crypto_owners_released', result?.resources_released === true],
    ['selected_journal_integrity', rows.length === expected.length && rows.every((row, i) => row.phase === expected[i] && row.stage === 2 && row.integrity === true && row.stack_pointer_inside === true && row.stack_span_kind === 'configured' && row.configured_main_stack_bytes === 16384)],
    ['existing_stack_margin', event?.configured_main_stack_bytes === 16384 && event.required_margin_bytes === 2048 && Number.isSafeInteger(event.minimum_main_stack_free_bytes) && event.minimum_main_stack_free_bytes >= 2048 && rows.length === expected.length && rows.every(row => Number.isSafeInteger(row.stack_low_water) && row.stack_low_water >= 2048)],
  ].map(([id, passed]) => ({ id, status: passed ? 'passed' : 'failed' }));
}

/** Parse only application records already projected by the bounded process owner. */
export function prefixTelemetryEvents(text) {
  const lines = text.split(/\r?\n/).filter(Boolean);
  if (!lines.every(line => line.startsWith('VIRTUAL_U205 '))) throw Error('prefix_projection_invalid');
  try {
    return lines.map(line => {
      const record = JSON.parse(line.slice(13));
      if (!record || typeof record !== 'object' || Array.isArray(record) || typeof record.event !== 'string') throw Error('prefix_application_record_invalid');
      return record;
    });
  }
  catch { throw Error('prefix_application_record_invalid'); }
}

/** One bounded emulator; no debugger or post-run partition/memory inspection. */
export async function runNoisePrefix(repo, packagePath, root, { auditPath, stop, seed = 1, disableEffects = false } = {}) {
  if (!NOISE_PREFIX_EFFECTS_ENABLED || disableEffects) throw Error('noise_prefix_effect_gate_disabled');
  if (!PREFIX_STOPS.includes(stop) || seed !== 1 || typeof auditPath !== 'string') throw Error('noise_prefix_arguments');
  const task = (await readFile(join(repo, 'TASKS.md'), 'utf8')).split('### task-ultra205-virtual-board-validation |')[1]?.split(/\n### |\n## Future/)[0];
  if (!task?.includes('noise-prefix-telemetry-enabled: true')) throw Error('noise_prefix_task_gate_disabled');
  const git = args => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
  if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== git(['rev-parse', 'origin/main'])) throw Error('noise_prefix_contract_not_clean_pushed');
  if (await realpath(root) !== resolve(root) || !resolve(root).startsWith(`${resolve(repo)}/`) || ((await stat(root)).mode & 0o077) !== 0) throw Error('noise_prefix_private_root');
  try { git(['check-ignore', '--quiet', '--', root]); } catch { throw Error('noise_prefix_root_not_ignored'); }
  await requireNoPrefixWriters(root, repo);
  const sourceSha256 = await sourceDigest(repo), validatorSha256 = await prefixValidatorIdentity(repo);
  const admitted = await admitPrefixPackage(packagePath, auditPath, sourceSha256, stop, git(['rev-parse', 'HEAD']));
  await validateNativeNoiseAudit(admitted.audit, join(dirname(packagePath), admitted.manifest.virtual_elf), repo);
  const binding = { package_sha256: sha(admitted.packageBytes), elf_sha256: admitted.manifest.virtual_elf_sha256,
    compiled_source_sha256: sourceSha256, validator_sha256: validatorSha256, source_commit: git(['rev-parse', 'HEAD']) };
  const result = { schema: 'bitaxe-noise-prefix-result-v2', ...binding, audit_sha256: sha(admitted.auditBytes), selected_stop: stop, seed,
    status: 'failed', collection_mode: 'application_records_only', emulator: EMULATOR.version,
    comparison_mode: 'runtime_selector_same_elf', actual_prefix_handshake_frame_bytes: admitted.audit.actual_prefix_handshake_frame_bytes,
    baseline_reference_handshake_frame_bytes: admitted.audit.baseline_reference_handshake_frame_bytes,
    checks: [], maybe_earliest_failure: null, cleanup_failures: [], subset_only: true,
    independent_task_bounds: 'unsupported', full_probe_qualified: false, hardware_qualified: false };
  let maybeProcess;
  try {
    await doctor(repo, root);
    const flash = join(root, 'virtual-flash-run.bin');
    await writeFile(flash, admitted.image, { flag: 'wx', mode: 0o600 });
    const efuse = await writeSdkEfuse(repo, root);
    if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== binding.source_commit
      || await sourceDigest(repo) !== sourceSha256 || await prefixValidatorIdentity(repo) !== validatorSha256) throw Error('noise_prefix_source_changed_before_effect');
    await claimPrefix(join(repo, 'scratch/virtual-noise-diagnostic/prefix-bisection002'), binding, stop, sha(admitted.auditBytes));
    maybeProcess = await runPrivate(managedPaths(repo).binary, qemuArguments(flash, efuse), root, 'noise-qemu', {
      timeoutMs: 35000, allowTimeout: true, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      inputReadyMarker: '\"event\":\"task\"', input: `${JSON.stringify({ command: 'noise_prefix', seed, stop })}\n` });
  } catch (error) {
    result.maybe_earliest_failure ??= { phase: 'execution', category: redactedPrefixFailure(error) };
    result.cleanup_failures.push(...(error.cleanupFailures ?? []).map(message => ({ phase: 'release', category: redactedPrefixFailure({ message }) })));
  }
  try {
    result.checks = judgePrefix(prefixTelemetryEvents(await readFile(join(root, 'noise-qemu.stdout.log'), 'utf8')), stop, sourceSha256);
    result.guest_prefix_state = result.checks.every(check => check.status === 'passed') ? 'completed_with_checks' : 'unknown_or_failed_checks';
    if (!result.checks.every(check => check.status === 'passed')) result.maybe_earliest_failure ??= { phase: 'collection', category: 'prefix_application_checks_failed' };
  } catch (error) { result.maybe_earliest_failure ??= { phase: 'collection', category: redactedPrefixFailure(error) }; }
  if (!maybeProcess) {
    try { maybeProcess = JSON.parse(await readFile(join(root, 'noise-qemu.process.json'), 'utf8')); }
    catch { result.cleanup_failures.push({ phase: 'release', category: 'process_release_record_unavailable' }); }
  }
  result.qemu_released = maybeProcess?.released === true;
  result.collection_cutoff = maybeProcess?.timedOut === true;
  result.natural_emulator_completion = maybeProcess?.timedOut === false && maybeProcess?.code === 0;
  result.debugger_used = false;
  await writeFile(join(root, 'prefix-partial-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  try {
    if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== binding.source_commit
      || await sourceDigest(repo) !== sourceSha256 || await prefixValidatorIdentity(repo) !== validatorSha256) throw Error('noise_prefix_source_changed_after_effect');
    await requireNoPrefixWriters(root, repo);
    if (!result.maybe_earliest_failure && result.cleanup_failures.length === 0 && result.checks.length === 6 && result.checks.every(check => check.status === 'passed') && result.qemu_released) result.status = 'passed';
    else result.maybe_earliest_failure ??= { phase: 'validation', category: 'selected_prefix_or_release_failed' };
    await writeFile(join(root, 'prefix-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  } catch (error) {
    result.status = 'failed'; result.maybe_earliest_failure ??= { phase: 'finalization', category: redactedPrefixFailure(error) };
    throw Object.assign(Error(result.maybe_earliest_failure.category), { result });
  }
  return result;
}

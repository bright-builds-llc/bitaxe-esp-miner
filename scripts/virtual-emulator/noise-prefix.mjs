import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, stat, realpath, readdir } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { createServer, createConnection } from 'node:net';
import { setTimeout as wait } from 'node:timers/promises';
import { EMULATOR } from './lock.mjs';
import { sourceDigest } from './identity.mjs';
import { noiseValidatorIdentity, requireNoNoiseWriters } from './noise.mjs';
import { MARKER, PROFILE } from './build.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { qemuArguments, writeSdkEfuse } from './run.mjs';
import { runPrivate } from './process.mjs';
import { validatePrefixNoiseAudit } from './noise-prefix-audit.mjs';
import { validateNativeNoiseAudit } from './noise-stack-audit.mjs';
import { prepareNoiseDebugger, decodeNoiseDebugger, noiseDebuggerMaterials } from './noise-debugger.mjs';

export const NOISE_PREFIX_EFFECTS_ENABLED = true;
export const PREFIX_STOPS = Object.freeze([101, 102, 103, 105, 106, 107, 109, 110]);
const sha = value => createHash('sha256').update(value).digest('hex');

const failureCategories = new Set(['noise_prefix_operation_failed', 'noise_prefix_source_changed_before_effect',
  'noise_prefix_source_changed_after_effect', 'noise_prefix_series_binding_changed', 'noise_prefix_series_exhausted',
  'noise_prefix_claim_permissions', 'noise_debugger_preflight', 'noise_debugger_abi', 'noise_debugger_arguments',
  'noise_debugger_private_root', 'noise_live_writer', 'emulator_manifest_version', 'emulator_manifest_digest',
  'emulator_binary_version', 'emulator_spawn', 'emulator_command_failed', 'emulator_output_bound',
  'emulator_process_group_alive', 'emulator_log_release_unproven', 'emulator_writer_release_unproven']);
export function redactedPrefixFailure(error) {
  return failureCategories.has(error?.message) ? error.message : 'noise_prefix_operation_failed';
}

export async function prefixValidatorIdentity(repo) {
  const hash = createHash('sha256'); hash.update(await noiseValidatorIdentity(repo));
  for (const name of ['noise-prefix.mjs', 'noise-prefix-audit.mjs', 'frame-audit.mjs', '../core-dump/analysis.mjs', ...noiseDebuggerMaterials]) {
    hash.update(name); hash.update('\0'); hash.update(await readFile(join(repo, 'scripts/virtual-emulator', name))); hash.update('\0');
  }
  return hash.digest('hex');
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
  const required = ['CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=2048', 'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304', 'CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y'];
  if (!required.every(line => config.toString().split('\n').includes(line)) || !image.subarray(0xf12000).every(byte => byte === 0xff) || manifest.board_profile?.cores !== 2 || manifest.board_profile.flash_bytes !== 16777216 || manifest.board_profile.psram_bytes !== 8388608 || manifest.board_profile.psram_mode !== 'octal') throw Error('noise_prefix_memory_contract');
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

/** SDK panic observation is independent of damaged or truncated structured guest output. */
export function prefixLogFacts(log) {
  const unexpectedPanic = /Guru Meditation|assert failed|Stack canary|stack overflow|abort\(\)/.test(log);
  const events = [];
  let invalidGuestJson = false;
  for (const line of log.split(/\r?\n/).filter(line => line.startsWith('VIRTUAL_U205 '))) {
    try { events.push(JSON.parse(line.slice(13))); }
    catch { invalidGuestJson = true; }
  }
  return { unexpected_panic: unexpectedPanic, events, invalid_guest_json: invalidGuestJson };
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

/** First-boundary safety and post-drop safety remain separate, with missing snapshots unknown. */
export function judgePrefixSnapshots(decoded, stop) {
  if (decoded?.schema !== 'bitaxe-noise-live-snapshots-v1' || decoded.cutoff_phase !== stop || !Array.isArray(decoded.snapshots)) return { prefix_state: 'unknown', cleanup_state: 'unknown' };
  const safe = (snapshot, expected) => snapshot?.registers_available === true && snapshot.checkpoint_memory_available === true
    && snapshot.scope_violation === false && snapshot.task_bounds?.filter(task => task.task_label === 'main' && task.bounds_available === true && task.captured_sp_inside_bounds === true && task.saved_tcb_top_inside_bounds === true).length === 1
    && snapshot.records?.length === expected.length && snapshot.records.every((record, i) => record.phase === expected[i] && record.stage === 2
      && record.heap_integrity === true && record.heap_observation_available === true && record.stack_pointer_inside_configured_span === true
      && record.configured_stack_span_bytes === 16384 && Number.isSafeInteger(record.stack_low_water_bytes) && record.stack_low_water_bytes >= 2048);
  const before = decoded.snapshots[0], after = decoded.snapshots[1], phases = Array.from({ length: stop - 100 }, (_, i) => 101 + i);
  const state = (snapshot, kind, expected) => {
    if (!snapshot) return 'unknown';
    if (['panic', 'assert'].includes(snapshot.kind) || snapshot.scope_violation === true) return 'failed';
    if (snapshot.kind !== kind) return 'unknown';
    if (snapshot.records?.some(record => record.heap_integrity === false || record.stack_pointer_inside_configured_span === false
      || (Number.isSafeInteger(record.stack_low_water_bytes) && record.stack_low_water_bytes < 2048))) return 'failed';
    if (snapshot.task_bounds?.some(task => task.task_label === 'main' && task.bounds_available === true
      && (task.captured_sp_inside_bounds === false || task.saved_tcb_top_inside_bounds === false))) return 'failed';
    if (snapshot.prefix_flags_available !== true) return 'unknown';
    return safe(snapshot, expected) ? 'safe' : 'unknown';
  };
  const prefixState = state(before, 'prefix', phases);
  const cleanupState = state(after, 'released', [...phases, 114]);
  return { prefix_state: prefixState === 'safe' && (before.prefix_argument_matches !== true || before.prefix_reached_phase !== stop || before.prefix_released_flag !== 0 || before.prefix_flags_available !== true) ? 'failed' : prefixState,
    cleanup_state: cleanupState === 'safe' && (after.prefix_reached_phase !== stop || after.prefix_released_flag !== 1 || after.prefix_argument_matches !== true || after.prefix_flags_available !== true) ? 'failed' : cleanupState };
}

async function reservePort() {
  const server = createServer();
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const port = server.address().port;
  return { port, close: () => new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve())) };
}
export async function listenerReleased(port) {
  return new Promise(resolve => {
    const socket = createConnection({ host: '127.0.0.1', port }); socket.setTimeout(1000);
    socket.once('connect', () => { socket.destroy(); resolve(false); });
    socket.once('timeout', () => { socket.destroy(); resolve(false); });
    socket.once('error', error => resolve(error.code === 'ECONNREFUSED'));
  });
}

export function qualificationFailure(result) {
  if (result.first_target_fault) return { phase: result.first_target_fault.phase, category: result.first_target_fault.category };
  return result.first_host_failure ?? result.collection_failures?.[0] ?? null;
}

/** One fresh QEMU and one prearmed live debugger, with independent bounded release. */
export async function runNoisePrefix(repo, packagePath, root, { auditPath, stop, seed = 1, disableEffects = false } = {}) {
  if (!NOISE_PREFIX_EFFECTS_ENABLED || disableEffects) throw Error('noise_prefix_effect_gate_disabled');
  if (!PREFIX_STOPS.includes(stop) || seed !== 1 || typeof auditPath !== 'string') throw Error('noise_prefix_arguments');
  const task = (await readFile(join(repo, 'TASKS.md'), 'utf8')).split('### task-ultra205-virtual-board-validation |')[1]?.split(/\n### |\n## Future/)[0];
  if (!task?.includes('noise-prefix-bisect-enabled: true')) throw Error('noise_prefix_task_gate_disabled');
  const git = args => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
  if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== git(['rev-parse', 'origin/main'])) throw Error('noise_prefix_contract_not_clean_pushed');
  if (await realpath(root) !== resolve(root) || !resolve(root).startsWith(`${resolve(repo)}/`) || ((await stat(root)).mode & 0o077) !== 0) throw Error('noise_prefix_private_root');
  try { git(['check-ignore', '--quiet', '--', root]); } catch { throw Error('noise_prefix_root_not_ignored'); }
  await requireNoNoiseWriters(root, repo);
  const sourceSha256 = await sourceDigest(repo), validatorSha256 = await prefixValidatorIdentity(repo);
  const admitted = await admitPrefixPackage(packagePath, auditPath, sourceSha256, stop, git(['rev-parse', 'HEAD']));
  await validateNativeNoiseAudit(admitted.audit, join(dirname(packagePath), admitted.manifest.virtual_elf), repo);
  const binding = { package_sha256: sha(admitted.packageBytes), elf_sha256: admitted.manifest.virtual_elf_sha256,
    compiled_source_sha256: sourceSha256, validator_sha256: validatorSha256, source_commit: git(['rev-parse', 'HEAD']) };
  const result = { schema: 'bitaxe-noise-prefix-result-v1', ...binding, audit_sha256: sha(admitted.auditBytes), selected_stop: stop, seed, status: 'failed',
    emulator: EMULATOR.version, comparison_mode: 'runtime_selector_same_elf',
    actual_prefix_handshake_frame_bytes: admitted.audit.actual_prefix_handshake_frame_bytes, baseline_reference_handshake_frame_bytes: admitted.audit.baseline_reference_handshake_frame_bytes, checks: [], maybe_earliest_failure: null, first_host_failure: null, collection_failures: [], cleanup_failures: [], subset_only: true, full_probe_qualified: false, hardware_qualified: false };
  let reservation, qemuOutcome, debuggerOutcome;
  const flash = join(root, 'virtual-flash-run.bin'), elf = join(root, 'virtual-ultra205.elf');
  try {
    await doctor(repo, root);
    await writeFile(flash, admitted.image, { flag: 'wx', mode: 0o600 }); await writeFile(elf, admitted.elf, { flag: 'wx', mode: 0o600 });
    const efuse = await writeSdkEfuse(repo, root);
    reservation = await reservePort();
    const debuggerPlan = await prepareNoiseDebugger(repo, root, elf, reservation.port, stop);
    result.debugger_preflight = debuggerPlan.identity;
    if (await sourceDigest(repo) !== sourceSha256 || await prefixValidatorIdentity(repo) !== validatorSha256) throw Error('noise_prefix_source_changed_before_effect');
    await claimPrefix(join(repo, 'scratch/virtual-noise-diagnostic/prefix-bisection001'), binding, stop, sha(admitted.auditBytes));
    await reservation.close();
    const settled = (promise, phase) => promise.then(value => ({ value }), error => {
      const category = redactedPrefixFailure(error);
      result.first_host_failure ??= { phase, category };
      return { error: category, cleanupFailures: (error.cleanupFailures ?? []).map(category => redactedPrefixFailure({ message: category })) };
    });
    const qemu = settled(runPrivate(managedPaths(repo).binary, [...qemuArguments(flash, efuse), '-gdb', `tcp:127.0.0.1:${reservation.port}`, '-S'], root, 'noise-qemu', {
      timeoutMs: 35000, allowTimeout: true, maxOutputBytes: 2097152, inputReadyMarker: '"event":"task"', input: `${JSON.stringify({ command: 'noise_prefix', seed, stop })}\n` }), 'execution');
    await wait(200);
    const debuggerRun = settled(runPrivate(debuggerPlan.command, debuggerPlan.args, root, 'noise-gdb', { timeoutMs: 25000, maxOutputBytes: 2097152 }), 'debugger');
    [qemuOutcome, debuggerOutcome] = await Promise.all([qemu, debuggerRun]);
    for (const [phase, outcome] of [['execution', qemuOutcome], ['debugger', debuggerOutcome]]) {
      if (outcome.error) result.first_host_failure ??= { phase, category: outcome.error };
      result.cleanup_failures.push(...(outcome.cleanupFailures ?? []).map(category => ({ phase: 'release', category })));
    }
  } catch (error) { result.first_host_failure ??= { phase: 'preflight', category: redactedPrefixFailure(error) }; }
  finally {
    if (reservation) {
      try { await reservation.close(); } catch (error) { if (error.code !== 'ERR_SERVER_NOT_RUNNING') result.cleanup_failures.push({ phase: 'release', category: 'debugger_reservation_release_failed' }); }
    }
  }
  try {
    const log = await readFile(join(root, 'noise-qemu.stdout.log'), 'utf8');
    const facts = prefixLogFacts(log);
    result.unexpected_panic = facts.unexpected_panic;
    if (facts.unexpected_panic) result.first_target_fault = { category: 'unexpected_target_panic', phase: 'prefix', snapshot_verified: false };
    if (facts.invalid_guest_json) result.collection_failures.push({ phase: 'collection', category: 'prefix_guest_json_invalid' });
    result.checks = judgePrefix(facts.events, stop, sourceSha256);
  } catch { result.collection_failures.push({ phase: 'collection', category: 'prefix_guest_facts_unavailable' }); }
  try { result.debugger = await decodeNoiseDebugger(repo, root, stop); result.prefix_safety = judgePrefixSnapshots(result.debugger, stop);
    const fault = result.debugger.snapshots?.find(snapshot => snapshot.kind === 'panic' || snapshot.kind === 'assert');
    if (fault) { result.first_target_fault = { snapshot_index: fault.index, phase: fault.index === 0 ? 'prefix' : 'cleanup', snapshot_verified: true, category: fault.kind === 'assert' ? 'first_target_assert' : 'first_target_panic', exception_category: fault.exception_category, last_checkpoint_phase: fault.records?.at(-1)?.phase ?? null };
    }
  }
  catch { result.collection_failures.push({ phase: 'collection', category: 'prefix_debugger_facts_unavailable' }); }
  for (const [label, outcome] of [['noise-qemu', qemuOutcome], ['noise-gdb', debuggerOutcome]]) {
    if (outcome && !outcome.value) {
      try { outcome.process = JSON.parse(await readFile(join(root, `${label}.process.json`), 'utf8')); }
      catch { result.cleanup_failures.push({ phase: 'release', category: 'process_release_record_unavailable' }); }
    }
  }
  const qemuProcess = qemuOutcome?.value ?? qemuOutcome?.process, debuggerProcess = debuggerOutcome?.value ?? debuggerOutcome?.process;
  result.qemu_released = qemuProcess?.released === true;
  result.debugger_released = debuggerProcess?.released === true;
  result.listener_released = reservation ? await listenerReleased(reservation.port) : false;
  if (!result.listener_released) result.cleanup_failures.push({ phase: 'release', category: 'debugger_listener_release_unproven' });
  result.collection_cutoff = qemuProcess?.timedOut === true;
  result.natural_emulator_completion = qemuProcess?.timedOut === false && qemuProcess?.code === 0;
  // Host rejection order does not establish target causal order. Direct target facts remain independent.
  result.maybe_earliest_failure = qualificationFailure(result);
  await writeFile(join(root, 'prefix-partial-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  try {
    if (await sourceDigest(repo) !== sourceSha256 || await prefixValidatorIdentity(repo) !== validatorSha256) throw Error('noise_prefix_source_changed_after_effect');
    await requireNoNoiseWriters(root, repo);
    if (!result.maybe_earliest_failure && result.cleanup_failures.length === 0 && result.checks.length === 6 && result.checks.every(check => check.status === 'passed') && result.prefix_safety?.prefix_state === 'safe' && result.prefix_safety?.cleanup_state === 'safe' && result.qemu_released && result.debugger_released && result.listener_released && result.unexpected_panic === false) result.status = 'passed';
    else result.maybe_earliest_failure ??= { phase: 'validation', category: 'selected_prefix_or_release_failed' };
    await writeFile(join(root, 'prefix-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  } catch (error) { result.status = 'failed'; result.maybe_earliest_failure ??= { phase: 'finalization', category: redactedPrefixFailure(error) }; throw Object.assign(Error(result.maybe_earliest_failure.category), { result }); }
  return result;
}

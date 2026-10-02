// Checkpoint-only Noise runs: projected application records are the only evidence.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, stat, realpath, readdir } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { EMULATOR } from './lock.mjs';
import { sourceDigest } from './identity.mjs';
import { PROFILE } from './build.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { qemuArguments, writeSdkEfuse } from './topology.mjs';
import { runPrivate } from './process.mjs';
import { admitNoisePackage } from './noise-admission.mjs';
import { validateNativeNoiseAudit } from './noise-stack-audit.mjs';
import { judgeTargetEvents } from './judge.mjs';

export const NOISE_CHECKPOINT_EFFECTS_ENABLED = false;
const OBJDUMP_VERSION = 'esp-14.2.0_20260121';
const PHASES = Object.freeze(Array.from({ length: 14 }, (_, index) => 101 + index));
const sha = value => createHash('sha256').update(value).digest('hex');

const failureCategories = new Set(['noise_checkpoint_operation_failed', 'noise_checkpoint_claim_consumed',
  'noise_checkpoint_source_changed_before_effect', 'noise_checkpoint_source_changed_after_effect', 'noise_live_writer',
  'emulator_manifest_version', 'emulator_manifest_digest', 'emulator_binary_version', 'emulator_spawn',
  'emulator_command_failed', 'emulator_output_bound', 'emulator_line_bound', 'checkpoint_projection_invalid',
  'checkpoint_application_record_invalid', 'emulator_process_group_alive', 'emulator_log_release_unproven',
  'emulator_writer_release_unproven']);
export function redactedCheckpointFailure(error) {
  return failureCategories.has(error?.message) ? error.message : 'noise_checkpoint_operation_failed';
}

/** Binds every visible emulator and core-dump script, so no reachable validator can drift unnoticed. */
export async function checkpointValidatorIdentity(repo) {
  const files = execFileSync('git', ['ls-files', '-c', '-o', '--exclude-standard', '-z', '--', 'scripts/virtual-emulator', 'scripts/core-dump'],
    { cwd: repo, encoding: 'utf8' }).split('\0').filter(Boolean).sort();
  const hash = createHash('sha256');
  for (const file of new Set(files)) {
    hash.update(file); hash.update('\0'); hash.update(await readFile(join(repo, file))); hash.update('\0');
  }
  return hash.digest('hex');
}

async function requireNoCheckpointWriters(root, repo) {
  for (let current = resolve(root); current.startsWith(`${resolve(repo)}/`); current = dirname(current)) {
    if ((await readdir(current)).some(name => name.endsWith('.writer.json'))) throw Error('noise_live_writer');
  }
}

/** One execution per published contract; an existing claim is never overwritten or replayed. */
export async function claimCheckpoint(claimPath, binding) {
  await mkdir(dirname(claimPath), { recursive: true, mode: 0o700 });
  try {
    await writeFile(claimPath, JSON.stringify({ schema: 'bitaxe-noise-full-checkpoint-claim-v1', ...binding }), { flag: 'wx', mode: 0o600 });
  } catch (error) {
    if (error.code === 'EEXIST') throw Error('noise_checkpoint_claim_consumed');
    throw error;
  }
}

/** Parse only application records already projected by the bounded process owner. */
export function checkpointEvents(text) {
  const lines = text.split(/\r?\n/).filter(Boolean);
  if (!lines.every(line => line.startsWith('VIRTUAL_U205 '))) throw Error('checkpoint_projection_invalid');
  try {
    return lines.map(line => {
      const record = JSON.parse(line.slice(13));
      if (!record || typeof record !== 'object' || Array.isArray(record) || typeof record.event !== 'string') throw Error('invalid');
      return record;
    });
  } catch { throw Error('checkpoint_application_record_invalid'); }
}

/** Authentication, release, per-phase heap integrity and the unchanged stack margin must all agree. */
export function judgeFullCheckpoint(events, sourceSha256) {
  const boot = events.filter(event => event.event === 'boot');
  const probes = events.filter(event => event.event === 'noise_probe');
  const rejected = events.filter(event => event.event === 'noise_probe_rejected');
  const probe = probes[0], result = probe?.result, rows = probe?.checkpoints ?? [];
  return [
    ['boot_identity', boot.length === 1 && boot[0].execution_profile === PROFILE && boot[0].compiled_source_sha256 === sourceSha256 && boot[0].heartbeat_cutoff_ms === 2800],
    ['noise_authenticated_round_trip', probes.length === 1 && rejected.length === 0 && result?.schema === 'bitaxe-noise-probe-v1' && result.seed === 1 &&
      result.authenticated === true && result.frame_round_trip === true && result.payload_bytes === 32 && result.hardware_qualified === false],
    ['noise_resources_released', result?.resources_released === true],
    ['noise_phase_integrity', rows.length === PHASES.length && rows.every((row, index) => row.phase === PHASES[index] && row.stage === 2 &&
      row.integrity === true && row.stack_pointer_inside === true && row.stack_span_kind === 'configured' && row.configured_main_stack_bytes === 16384 &&
      Number.isSafeInteger(row.internal_free) && Number.isSafeInteger(row.internal_largest) && row.internal_largest >= 0 && row.internal_largest <= row.internal_free)],
    ['noise_stack_margin', probe?.configured_main_stack_bytes === 16384 && probe.required_margin_bytes === 2048 &&
      Number.isSafeInteger(probe.minimum_main_stack_free_bytes) && probe.minimum_main_stack_free_bytes >= 2048 &&
      rows.length === PHASES.length && rows.every(row => Number.isSafeInteger(row.stack_low_water) && row.stack_low_water >= 2048)],
  ].map(([id, passed]) => ({ id, status: passed ? 'passed' : 'failed' }));
}

// The host scenario deliberately reports this coverage gap; every other check must pass.
const EXPECTED_UNSUPPORTED = new Set(['scenario:strict_live_profile_share']);

/** The composed Start path: shared target checks, the helper stack and post-run heap integrity. */
export function judgeComposedCheckpoint(events, sourceSha256) {
  const target = judgeTargetEvents(events, { compiled_source_sha256: sourceSha256 },
    { commands: ['status', 'allocation'], scenario: 'healthy-lifecycle', seed: 1 });
  const margin = events.find(event => event.event === 'scenario_margin');
  const result = events.find(event => event.event === 'scenario')?.result;
  return [
    ...target.map(check => ({ id: check.id, status: check.status === 'unsupported' && EXPECTED_UNSUPPORTED.has(check.id) ? 'expected_unsupported'
      : check.status === 'passed' ? 'passed' : 'failed' })),
    { id: 'scenario_started', status: result?.actual_outcome === 'started' ? 'passed' : 'failed' },
    { id: 'handshake_stack_margin', status: Number.isSafeInteger(margin?.handshake_minimum_stack_free_bytes) &&
      margin.handshake_minimum_stack_free_bytes >= 2048 && margin.handshake_configured_stack_bytes === 16384 ? 'passed' : 'failed' },
    { id: 'heap_integrity_after_scenario', status: margin?.heap_integrity === true ? 'passed' : 'failed' },
  ];
}

export function composedObservations(events) {
  const margin = events.find(event => event.event === 'scenario_margin');
  const number = value => Number.isSafeInteger(value) ? value : null;
  return { scenario_outcome: typeof events.find(event => event.event === 'scenario')?.result?.actual_outcome === 'string'
      ? events.find(event => event.event === 'scenario').result.actual_outcome : null,
    minimum_main_stack_free_bytes: number(margin?.minimum_main_stack_free_bytes),
    handshake_minimum_stack_free_bytes: number(margin?.handshake_minimum_stack_free_bytes),
    heap_integrity: margin?.heap_integrity === true };
}

/** Each published contract owns one profile, one claim and one task gate. */
export const CHECKPOINT_PROFILES = Object.freeze({
  'noise-probe': { claim: 'scratch/virtual-noise-diagnostic/full-checkpoint001.claim.json',
    task: '### task-ultra205-virtual-board-validation |', gate: 'noise-full-checkpoint-enabled: true',
    input: [{ command: 'noise_probe', seed: 1, mode: 'valid' }], judge: judgeFullCheckpoint, observe: events => checkpointObservations(events) },
  'healthy-lifecycle': { claim: 'scratch/virtual-noise-diagnostic/composed-checkpoint001.claim.json',
    task: '### task-device-noise-worker-stack |', gate: 'noise-composed-checkpoint-enabled: true',
    input: [{ command: 'status' }, { command: 'allocation' }, { command: 'scenario', scenario: 'healthy-lifecycle', seed: 1 }],
    judge: judgeComposedCheckpoint, observe: events => composedObservations(events) },
});

/** Ordinary numeric facts only; a rejection category is kept only when it matches the guest grammar. */
export function checkpointObservations(events) {
  const event = events.find(item => item.event === 'noise_probe' || item.event === 'noise_probe_rejected');
  if (!event) return { probe_event: 'absent' };
  const rows = Array.isArray(event.checkpoints) ? event.checkpoints : [];
  const number = value => Number.isSafeInteger(value) ? value : null;
  return { probe_event: event.event === 'noise_probe' ? 'completed' : 'rejected',
    maybe_rejection_category: event.event === 'noise_probe_rejected' && /^noise_[A-Za-z0-9_]{1,80}$/.test(event.category ?? '') ? event.category : null,
    minimum_main_stack_free_bytes: number(event.minimum_main_stack_free_bytes),
    phases: rows.map(row => ({ phase: number(row.phase), stage: number(row.stage), integrity: row.integrity === true,
      stack_low_water: number(row.stack_low_water), internal_free: number(row.internal_free), internal_largest: number(row.internal_largest) })) };
}

const accepted = check => check.status === 'passed' || check.status === 'expected_unsupported';

/** One bounded emulator; no debugger, panic text, core or post-run partition inspection. */
export async function runNoiseCheckpoint(repo, packagePath, root, { auditPath, seed = 1, profile = 'noise-probe', disableEffects = false } = {}) {
  if (!NOISE_CHECKPOINT_EFFECTS_ENABLED || disableEffects) throw Error('noise_checkpoint_effect_gate_disabled');
  const selected = Object.hasOwn(CHECKPOINT_PROFILES, profile) ? CHECKPOINT_PROFILES[profile] : null;
  if (seed !== 1 || typeof auditPath !== 'string' || !selected) throw Error('noise_checkpoint_arguments');
  const task = (await readFile(join(repo, 'TASKS.md'), 'utf8')).split(selected.task)[1]?.split(/\n### |\n## Future/)[0];
  if (!task?.includes(selected.gate)) throw Error('noise_checkpoint_task_gate_disabled');
  const git = args => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
  if (git(['status', '--porcelain']) || git(['rev-parse', 'HEAD']) !== git(['rev-parse', 'origin/main'])) throw Error('noise_checkpoint_contract_not_clean_pushed');
  if (await realpath(root) !== resolve(root) || !resolve(root).startsWith(`${resolve(repo)}/`) || ((await stat(root)).mode & 0o077) !== 0) throw Error('noise_checkpoint_private_root');
  try { git(['check-ignore', '--quiet', '--', root]); } catch { throw Error('noise_checkpoint_root_not_ignored'); }
  await requireNoCheckpointWriters(root, repo);
  const sourceSha256 = await sourceDigest(repo), validatorSha256 = await checkpointValidatorIdentity(repo), sourceCommit = git(['rev-parse', 'HEAD']);
  const objdump = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', OBJDUMP_VERSION, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
  const admitted = await admitNoisePackage(packagePath, auditPath, sourceSha256, objdump);
  if (admitted.manifest.source_commit !== sourceCommit || admitted.manifest.source_dirty !== false) throw Error('noise_checkpoint_package_not_current_clean');
  await validateNativeNoiseAudit(admitted.audit, join(dirname(packagePath), admitted.manifest.virtual_elf), repo);
  const binding = { package_sha256: sha(admitted.packageBytes), elf_sha256: admitted.manifest.virtual_elf_sha256,
    sdkconfig_sha256: admitted.manifest.virtual_sdkconfig_sha256, compiled_source_sha256: sourceSha256,
    validator_sha256: validatorSha256, source_commit: sourceCommit, audit_sha256: sha(admitted.auditBytes) };
  const result = { schema: 'bitaxe-noise-checkpoint-result-v2', profile, ...binding, seed, status: 'failed',
    collection_mode: 'application_records_only', emulator: EMULATOR.version, checks: [], observations: null,
    maybe_earliest_failure: null, cleanup_failures: [], independent_task_bounds: 'unsupported',
    full_board_qualified: false, hardware_qualified: false };
  const stillBound = async () => !git(['status', '--porcelain']) && git(['rev-parse', 'HEAD']) === sourceCommit &&
    await sourceDigest(repo) === sourceSha256 && await checkpointValidatorIdentity(repo) === validatorSha256;
  let maybeProcess;
  try {
    await doctor(repo, root);
    const flash = join(root, 'virtual-flash-run.bin');
    await writeFile(flash, admitted.image, { flag: 'wx', mode: 0o600 });
    const efuse = await writeSdkEfuse(repo, root);
    if (!await stillBound()) throw Error('noise_checkpoint_source_changed_before_effect');
    await claimCheckpoint(join(repo, selected.claim), { profile, ...binding });
    maybeProcess = await runPrivate(managedPaths(repo).binary, qemuArguments(flash, efuse), root, 'noise-qemu', {
      timeoutMs: 60000, allowTimeout: true, maxOutputBytes: 2097152, outputLinePrefix: 'VIRTUAL_U205 ',
      inputReadyMarker: '"event":"task"', input: selected.input.map(value => `${JSON.stringify(value)}\n`).join('') });
  } catch (error) {
    result.maybe_earliest_failure ??= { phase: 'execution', category: redactedCheckpointFailure(error) };
    result.cleanup_failures.push(...(error.cleanupFailures ?? []).map(message => ({ phase: 'release', category: redactedCheckpointFailure({ message }) })));
  }
  try {
    const events = checkpointEvents(await readFile(join(root, 'noise-qemu.stdout.log'), 'utf8'));
    result.checks = selected.judge(events, sourceSha256);
    result.observations = selected.observe(events);
    if (!result.checks.every(accepted)) result.maybe_earliest_failure ??= { phase: 'collection', category: 'checkpoint_checks_failed' };
  } catch (error) { result.maybe_earliest_failure ??= { phase: 'collection', category: redactedCheckpointFailure(error) }; }
  if (!maybeProcess) {
    try { maybeProcess = JSON.parse(await readFile(join(root, 'noise-qemu.process.json'), 'utf8')); }
    catch { result.cleanup_failures.push({ phase: 'release', category: 'process_release_record_unavailable' }); }
  }
  result.qemu_released = maybeProcess?.released === true;
  result.collection_cutoff = maybeProcess?.timedOut === true;
  result.debugger_used = false;
  await writeFile(join(root, 'checkpoint-partial-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  try {
    if (!await stillBound()) throw Error('noise_checkpoint_source_changed_after_effect');
    await requireNoCheckpointWriters(root, repo);
    if (!result.maybe_earliest_failure && result.cleanup_failures.length === 0 && result.checks.length > 0 &&
        result.checks.every(accepted) && result.qemu_released) result.status = 'passed';
    else result.maybe_earliest_failure ??= { phase: 'validation', category: 'checkpoint_or_release_failed' };
    await writeFile(join(root, 'checkpoint-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  } catch (error) {
    result.status = 'failed'; result.maybe_earliest_failure ??= { phase: 'finalization', category: redactedCheckpointFailure(error) };
    throw Object.assign(Error(result.maybe_earliest_failure.category), { result });
  }
  return result;
}

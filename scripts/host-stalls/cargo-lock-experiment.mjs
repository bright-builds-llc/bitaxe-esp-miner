import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { createEvidenceRoot, writeEvidence } from './evidence.mjs';
import { runRecordedCommand } from './runner.mjs';

const fixtures = fileURLToPath(new URL('./fixtures/', import.meta.url));
const holdMs = 5000;
const startupAllowanceMs = 1500;

/** Distinguish observed Cargo lock contention from delayed or invalid trials. */
export function classifyCargoLockTrial(trial) {
  if (!trial.commandsSucceeded) return 'confounded_command_failure';
  if (!trial.cleanupComplete) return 'confounded_cleanup';
  if (trial.statusRecompiled) return 'confounded_status_recompiled';
  const { holderEnteredMs, holderExitedMs, statusRequestedMs, statusEnteredMs } = trial;
  if (![holderEnteredMs, holderExitedMs, statusRequestedMs, statusEnteredMs].every(Number.isSafeInteger)) return 'confounded_missing_marker';
  if (holderExitedMs - holderEnteredMs < holdMs - 100 || holderExitedMs - holderEnteredMs > holdMs + startupAllowanceMs) return 'confounded_holder_duration';
  if (statusRequestedMs < holderEnteredMs || statusRequestedMs - holderEnteredMs > 500) return 'confounded_late_request';
  if (trial.mode === 'shared') {
    if (!trial.lockMessageObserved) return 'confounded_lock_not_observed';
    if (statusEnteredMs < holderExitedMs || statusEnteredMs - holderExitedMs > startupAllowanceMs) return 'confounded_status_startup';
    return 'shared_lock_observed';
  }
  if (trial.mode !== 'isolated') return 'confounded_unknown_mode';
  if (trial.lockMessageObserved) return 'confounded_unexpected_lock';
  if (statusEnteredMs < statusRequestedMs || statusEnteredMs - statusRequestedMs > startupAllowanceMs || statusEnteredMs >= holderExitedMs) return 'confounded_status_startup';
  return 'isolated_progress_observed';
}

function createFixture(root) {
  for (let ancestor = path.dirname(root); ; ancestor = path.dirname(ancestor)) {
    if (['config', 'config.toml'].some(name => existsSync(path.join(ancestor, '.cargo', name)))) throw new Error('fixture_has_ancestor_cargo_config');
    if (ancestor === path.dirname(ancestor)) break;
  }
  for (const relative of ['fixture', 'fixture/holder', 'fixture/holder/src', 'fixture/status', 'fixture/status/src', 'cargo-home', 'markers']) mkdirSync(path.join(root, relative), { mode: 0o700 });
  const files = {
    'fixture/Cargo.toml': '[workspace]\nresolver = "2"\nmembers = ["holder", "status"]\n',
    'fixture/holder/Cargo.toml': '[package]\nname = "holder"\nversion = "0.0.0"\nedition = "2021"\n',
    'fixture/status/Cargo.toml': '[package]\nname = "status"\nversion = "0.0.0"\nedition = "2021"\n',
    'fixture/holder/src/main.rs': 'fn main() {}\n',
    'fixture/holder/build.rs': readFileSync(path.join(fixtures, 'cargo-lock-build.rs'), 'utf8'),
    'fixture/status/src/main.rs': readFileSync(path.join(fixtures, 'cargo-lock-status.rs'), 'utf8'),
  };
  for (const [relative, text] of Object.entries(files)) writeFileSync(path.join(root, relative), text, { flag: 'wx', mode: 0o600 });
  return Object.fromEntries(Object.entries(files).map(([relative, text]) => [relative, createHash('sha256').update(text).digest('hex')]));
}

function commandOptions(options, label, targetName, nonce, args, timeoutMs) {
  // These inputs are fixture-owned, never copied from credentials or arbitrary environment dumps.
  const command = ['/usr/bin/env'];
  for (const name of ['RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS']) command.push('-u', name);
  command.push(`CARGO_HOME=${path.join(options.root, 'cargo-home')}`, `RUSTC=${options.rustc}`, 'CARGO_NET_OFFLINE=true', 'CARGO_TERM_COLOR=never', `HOST_STALL_MARKER_ROOT=${path.join(options.root, 'markers')}`, `HOST_STALL_TRIAL=${nonce}`, options.cargo, ...args, '--offline', '--target-dir', path.join(options.root, targetName));
  return { command, cwd: path.join(options.root, 'fixture'), evidenceRoot: path.join(options.root, label), label, quietMs: 15000, captureIntervalMs: 10000, maxCaptures: 2, timeoutMs, signal: options.signal };
}

function commandSucceeded(result) {
  return result.outcome === 'success' && result.exitCode === 0 && result.failures.length === 0 && !result.stdout.truncated && !result.stderr.truncated;
}

function maybeMarker(root, nonce, phase) {
  try {
    const value = Number(readFileSync(path.join(root, 'markers', `${nonce}.${phase}`), 'utf8'));
    if (!Number.isSafeInteger(value) || value <= 0) throw new Error('invalid_fixture_marker');
    return value;
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
}

async function waitForMarker(root, nonce, holderState) {
  const until = Date.now() + 25000;
  while (!holderState.finished && Date.now() < until) {
    const maybeEntered = maybeMarker(root, nonce, 'entered');
    if (maybeEntered !== null) return maybeEntered;
    await delay(20);
  }
  return null;
}

async function runTrial(options, ordinal, mode) {
  const nonce = `trial-${ordinal}-${mode}`;
  const holderAbort = new AbortController();
  const holderOptions = { ...options, signal: AbortSignal.any([options.signal, holderAbort.signal]) };
  const holderState = { finished: false };
  const holderPromise = runRecordedCommand(commandOptions(holderOptions, `${nonce}-holder`, 'target-shared', nonce, ['build', '--locked', '-p', 'holder'], 30000))
    .then(result => ({ result }), error => ({ error }))
    .finally(() => { holderState.finished = true; });
  let holderEnteredMs;
  let maybeStatus;
  let statusRequestedMs;
  let holder;
  try {
    holderEnteredMs = await waitForMarker(options.root, nonce, holderState);
    statusRequestedMs = Date.now();
    if (holderEnteredMs !== null && !options.signal.aborted) maybeStatus = await runRecordedCommand(commandOptions(options, `${nonce}-status`, mode === 'shared' ? 'target-shared' : 'target-isolated', nonce, ['run', '--locked', '-p', 'status'], 30000));
    else holderAbort.abort();
    const settled = await holderPromise;
    if (settled.error) throw settled.error;
    holder = settled.result;
  } finally {
    if (!holderState.finished) holderAbort.abort();
    await holderPromise;
  }
  const statusStdout = maybeStatus ? readFileSync(path.join(maybeStatus.evidenceRoot, 'stdout.log'), 'utf8') : '';
  const statusStderr = maybeStatus ? readFileSync(path.join(maybeStatus.evidenceRoot, 'stderr.log'), 'utf8') : '';
  const maybeEnteredMatch = statusStdout.match(/^entered_main (\d+)$/m);
  const trial = {
    ordinal, mode, holderEnteredMs, holderExitedMs: maybeMarker(options.root, nonce, 'exited'), statusRequestedMs,
    statusEnteredMs: maybeEnteredMatch ? Number(maybeEnteredMatch[1]) : null,
    commandsSucceeded: commandSucceeded(holder) && Boolean(maybeStatus && commandSucceeded(maybeStatus)),
    cleanupComplete: holder.cleanup.complete && Boolean(maybeStatus && maybeStatus.cleanup.complete),
    statusRecompiled: /\bCompiling\b/.test(statusStderr),
    lockMessageObserved: /Blocking waiting for file lock on (?:build|artifact) directory/.test(statusStderr),
    holderEvidence: holder.evidenceRoot, statusEvidence: maybeStatus?.evidenceRoot ?? null,
  };
  trial.classification = classifyCargoLockTrial(trial);
  writeEvidence(options.root, `${nonce}.json`, trial);
  return trial;
}

/** Run four offline Cargo trials in a new private root, without touching existing build caches. */
export async function runCargoLockExperiment(input) {
  const options = { ...input };
  for (const name of ['root', 'cargo', 'rustc']) if (typeof options[name] !== 'string' || !path.isAbsolute(options[name])) throw new Error(`${name}_must_be_absolute`);
  options.cargo = realpathSync(options.cargo);
  options.rustc = realpathSync(options.rustc);
  const abortController = new AbortController();
  options.signal = abortController.signal;
  const cancel = () => abortController.abort();
  process.on('SIGINT', cancel);
  process.on('SIGTERM', cancel);
  const oldUmask = process.umask(0o077);
  try {
    options.root = createEvidenceRoot(options.root);
    const fixtureHashes = createFixture(options.root);
    const summary = { schema: 'host-stall-cargo-lock-experiment-v1', root: options.root, cargo: options.cargo, rustc: options.rustc, fixtureHashes, holdMs, startupAllowanceMs, warmups: [], trials: [], outcome: 'incomplete' };
    for (const targetName of ['target-shared', 'target-isolated']) {
      const label = `warm-${targetName}`;
      const result = await runRecordedCommand(commandOptions(options, label, targetName, label, ['build', '--workspace'], 120000));
      summary.warmups.push({ evidence: result.evidenceRoot, succeeded: commandSucceeded(result), cleanupComplete: result.cleanup.complete });
      if (!commandSucceeded(result) || !result.cleanup.complete) {
        summary.outcome = 'confounded_warmup';
        writeEvidence(options.root, 'experiment.json', summary);
        return summary;
      }
    }
    for (const [index, mode] of ['shared', 'isolated', 'shared', 'isolated'].entries()) {
      if (options.signal.aborted) break;
      const trial = await runTrial(options, index + 1, mode);
      summary.trials.push(trial);
      if (!trial.cleanupComplete || !trial.commandsSucceeded) break;
    }
    summary.outcome = summary.trials.length === 4 && summary.trials.every(trial => ['shared_lock_observed', 'isolated_progress_observed'].includes(trial.classification)) ? 'lock_amplification_demonstrated' : 'confounded';
    writeEvidence(options.root, 'experiment.json', summary);
    return summary;
  } finally {
    process.umask(oldUmask);
    process.off('SIGINT', cancel);
    process.off('SIGTERM', cancel);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const options = {};
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index]?.slice(2);
    if (!['root', 'cargo', 'rustc'].includes(key) || options[key] !== undefined || !args[index + 1] || args[index] !== `--${key}`) throw new Error('usage: --root ABS_FRESH_ROOT --cargo ABS_STABLE_CARGO --rustc ABS_STABLE_RUSTC');
    options[key] = args[index + 1];
  }
  const result = await runCargoLockExperiment(options);
  console.log(JSON.stringify({ outcome: result.outcome, evidence: path.join(result.root, 'experiment.json'), trials: result.trials.map(trial => ({ mode: trial.mode, classification: trial.classification })) }));
  if (result.outcome !== 'lock_amplification_demonstrated') process.exitCode = 1;
}

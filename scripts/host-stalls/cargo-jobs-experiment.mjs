import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createEvidenceRoot, writeEvidence } from './evidence.mjs';
import { runRecordedCommand } from './runner.mjs';

const buildScript = fileURLToPath(new URL('./fixtures/cargo-jobs-build.rs', import.meta.url));
const packages = ['independent_a', 'independent_b'];
const digest = value => createHash('sha256').update(value).digest('hex');

/** Classify actual independent build-script intervals, not command wall time. */
export function classifyCargoJobsTrial(trial) {
  if (!trial.commandSucceeded) return 'confounded_command_failure';
  if (!trial.cleanupComplete) return 'confounded_cleanup';
  if (trial.intervals.length !== 2) return 'confounded_missing_marker';
  for (const interval of trial.intervals) {
    if (![interval.enteredMs, interval.exitedMs].every(Number.isSafeInteger)) return 'confounded_missing_marker';
    const duration = interval.exitedMs - interval.enteredMs;
    if (duration < 490 || duration > 1500) return 'confounded_build_script_duration';
  }
  const overlap = Math.min(...trial.intervals.map(interval => interval.exitedMs)) - Math.max(...trial.intervals.map(interval => interval.enteredMs));
  if (trial.jobs === 1) return overlap <= 0 ? 'serial_scheduling_observed' : 'confounded_unexpected_overlap';
  if (trial.jobs === 2) return overlap >= 250 ? 'parallel_scheduling_observed' : 'confounded_parallel_overlap_missing';
  return 'confounded_unknown_job_count';
}

function createFixture(root) {
  for (let ancestor = path.dirname(root); ; ancestor = path.dirname(ancestor)) {
    if (['config', 'config.toml'].some(name => existsSync(path.join(ancestor, '.cargo', name)))) throw new Error('fixture_has_ancestor_cargo_config');
    if (ancestor === path.dirname(ancestor)) break;
  }
  for (const name of ['fixture', 'cargo-home', 'markers']) mkdirSync(path.join(root, name), { mode: 0o700 });
  const files = { 'fixture/Cargo.toml': `[workspace]\nresolver = "2"\nmembers = [${packages.map(name => `"${name}"`).join(', ')}]\n` };
  for (const name of packages) {
    mkdirSync(path.join(root, 'fixture', name), { mode: 0o700 });
    mkdirSync(path.join(root, 'fixture', name, 'src'), { mode: 0o700 });
    files[`fixture/${name}/Cargo.toml`] = `[package]\nname = "${name}"\nversion = "0.0.0"\nedition = "2021"\n`;
    files[`fixture/${name}/src/main.rs`] = 'fn main() {}\n';
    files[`fixture/${name}/build.rs`] = readFileSync(buildScript, 'utf8');
  }
  for (const [name, text] of Object.entries(files)) writeFileSync(path.join(root, name), text, { flag: 'wx', mode: 0o600 });
  return Object.fromEntries(Object.entries(files).map(([name, text]) => [name, digest(text)]));
}

function commandOptions(options, label, jobs, warmup) {
  const command = ['/usr/bin/env'];
  for (const name of ['RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_TARGET', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS']) command.push('-u', name);
  command.push(`CARGO_HOME=${path.join(options.root, 'cargo-home')}`, `RUSTC=${options.rustc}`, 'CARGO_NET_OFFLINE=true', 'CARGO_TERM_COLOR=never', `HOST_STALL_MARKER_ROOT=${path.join(options.root, 'markers')}`, `HOST_STALL_TRIAL=${label}`, options.cargo, 'build', '--workspace', '--offline', '--jobs', String(jobs), '--target-dir', path.join(options.root, 'target'));
  if (!warmup) command.push('--locked', '--timings');
  return { command, cwd: path.join(options.root, 'fixture'), evidenceRoot: path.join(options.root, label), label, quietMs: 15000, captureIntervalMs: 10000, maxCaptures: 2, timeoutMs: warmup ? 120000 : 30000, signal: options.signal };
}

function commandSucceeded(result) {
  return result.outcome === 'success' && result.exitCode === 0 && result.failures.length === 0 && !result.stdout.truncated && !result.stderr.truncated;
}

function maybeMarker(root, label, packageName, phase) {
  try {
    const value = Number(readFileSync(path.join(root, 'markers', `${label}-${packageName}.${phase}`), 'utf8'));
    return Number.isSafeInteger(value) && value > 0 ? value : null;
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
}

async function runTrial(options, ordinal, jobs) {
  const label = `trial-${ordinal}-jobs-${jobs}`;
  const result = await runRecordedCommand(commandOptions(options, label, jobs, false));
  const trial = {
    ordinal, jobs, commandSucceeded: commandSucceeded(result), cleanupComplete: result.cleanup.complete,
    intervals: packages.map(name => ({ package: name, enteredMs: maybeMarker(options.root, label, name, 'entered'), exitedMs: maybeMarker(options.root, label, name, 'exited') })),
    evidence: result.evidenceRoot, durationMs: result.durationMs,
  };
  if (trial.commandSucceeded) {
    const html = readFileSync(path.join(options.root, 'target/cargo-timings/cargo-timing.html'));
    const timings = path.join(result.evidenceRoot, 'cargo-timings.html');
    writeFileSync(timings, html, { flag: 'wx', mode: 0o600 });
    trial.timings = { path: timings, sha256: digest(html), bytes: html.length };
  }
  trial.classification = classifyCargoJobsTrial(trial);
  writeEvidence(options.root, `${label}.json`, trial);
  return trial;
}

/** Compare one and two Cargo jobs using one warmed private target and no dependencies. */
export async function runCargoJobsExperiment(input) {
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
    const summary = { schema: 'host-stall-cargo-jobs-experiment-v1', root: options.root, cargo: options.cargo, rustc: options.rustc, fixtureHashes: createFixture(options.root), trials: [], outcome: 'incomplete', nativeStallDiagnosis: 'not_established_by_scheduling_experiment' };
    const warmup = await runRecordedCommand(commandOptions(options, 'warmup', 2, true));
    summary.warmup = { evidence: warmup.evidenceRoot, succeeded: commandSucceeded(warmup), cleanupComplete: warmup.cleanup.complete };
    if (!summary.warmup.succeeded || !summary.warmup.cleanupComplete) {
      summary.outcome = 'confounded_warmup';
      writeEvidence(options.root, 'experiment.json', summary);
      return summary;
    }
    for (const [index, jobs] of [1, 2, 1, 2].entries()) {
      if (options.signal.aborted) break;
      const trial = await runTrial(options, index + 1, jobs);
      summary.trials.push(trial);
      if (!trial.commandSucceeded || !trial.cleanupComplete) break;
    }
    summary.outcome = summary.trials.length === 4 && summary.trials.every(trial => ['serial_scheduling_observed', 'parallel_scheduling_observed'].includes(trial.classification)) ? 'job_scheduling_difference_demonstrated' : 'confounded';
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
  const result = await runCargoJobsExperiment(options);
  console.log(JSON.stringify({ outcome: result.outcome, evidence: path.join(result.root, 'experiment.json'), nativeStallDiagnosis: result.nativeStallDiagnosis, trials: result.trials.map(trial => ({ jobs: trial.jobs, classification: trial.classification })) }));
  if (result.outcome !== 'job_scheduling_difference_demonstrated') process.exitCode = 1;
}

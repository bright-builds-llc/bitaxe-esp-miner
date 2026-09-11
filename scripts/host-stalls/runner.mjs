import { spawn } from 'node:child_process';
import { performance } from 'node:perf_hooks';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { captureProcesses, cleanupOwned, ownedProcesses, processOwnership, processSnapshot, processExecutable } from './capture.mjs';
import { createEvidenceRoot, environmentMetadata, outputSink, OVERRIDE_KEYS, writeEvidence } from './evidence.mjs';

function validate(options) {
  if (!Array.isArray(options.command) || options.command.length === 0 || options.command.some((item) => typeof item !== 'string' || item.includes('\0')) || !options.command[0]) throw new Error('invalid_command');
  if (!path.isAbsolute(options.cwd)) throw new Error('cwd_must_be_absolute');
  if (!/^[a-zA-Z0-9_-]{1,80}$/.test(options.label)) throw new Error('invalid_label');
  for (const name of ['quietMs', 'captureIntervalMs', 'maxOutputBytes', 'processPollMs']) if (!Number.isSafeInteger(options[name]) || options[name] < 1) throw new Error(`invalid_${name}`);
  if (!Number.isSafeInteger(options.maxCaptures) || options.maxCaptures < 0 || options.maxCaptures > 10) throw new Error('invalid_maxCaptures');
  if (options.timeoutMs !== undefined && (!Number.isSafeInteger(options.timeoutMs) || options.timeoutMs < 1)) throw new Error('invalid_timeoutMs');
  if (options.expectedStderrMarker !== undefined && (typeof options.expectedStderrMarker !== 'string' || !options.expectedStderrMarker || Buffer.byteLength(options.expectedStderrMarker) > 128 || options.expectedStderrMarker.includes('\0'))) throw new Error('invalid_expectedStderrMarker');
  for (const [key, value] of Object.entries(options.env ?? {})) if (!OVERRIDE_KEYS.has(key) || typeof value !== 'string') throw new Error(`invalid_environment_override:${key}`);
}

function recordFailure(failures, failure) {
  const maybeExisting = failures.find((entry) => entry.stage === failure.stage && entry.error === failure.error);
  if (maybeExisting) { maybeExisting.count += 1; return; }
  if (failures.length < 20) failures.push({ ...failure, count: 1 });
}

async function recorderAncestors(snapshot) {
  const rows = await snapshot();
  const result = [];
  let next = process.pid;
  while (next > 0 && result.length < 16) {
    const maybeRow = rows.find((row) => row.pid === next);
    if (!maybeRow || result.some((row) => row.pid === next)) break;
    result.push({ pid: next, ppid: maybeRow.ppid, startedAt: maybeRow.startedAt, executable: path.basename(await processExecutable(next)) });
    next = maybeRow.ppid;
  }
  return result;
}

function startChild(options, event, stdout, stderr, state) {
  event('spawn_requested');
  const child = spawn(options.command[0], options.command.slice(1), { cwd: options.cwd, env: { ...process.env, ...options.env }, shell: false, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  child.on('spawn', () => { state.pid = child.pid; event('spawned', { pid: child.pid }); });
  child.on('error', (error) => { state.spawnError = { code: error.code ?? null, message: error.message }; event('spawn_error', state.spawnError); });
  child.on('exit', (code, signal) => { state.exitCode = code; state.signal = signal; event('exit', { code, signal }); });
  child.on('close', (code, signal) => { state.exitCode = code; state.signal = signal; state.closed = true; event('close', { code, signal }); });
  for (const [name, sink] of [['stdout', stdout], ['stderr', stderr]]) child[name].on('data', (chunk) => {
    if (sink.result.totalBytes === 0) event(`first_${name}`);
    state.lastOutput = performance.now();
    if (name === 'stderr' && options.expectedStderrMarker && !state.markerObserved) {
      const marker = Buffer.from(options.expectedStderrMarker);
      const combined = Buffer.concat([state.markerTail, chunk]);
      if (combined.includes(marker)) { state.markerObserved = true; event('stderr_marker_observed'); }
      state.markerTail = Buffer.from(combined.subarray(Math.max(0, combined.length - marker.length + 1)));
    }
    try { sink.append(chunk); }
    catch (error) { recordFailure(state.failures, { stage: `write_${name}`, error: error.message }); state.stopReason = 'evidence_error'; }
  });
  return new Promise((resolve) => { child.once('spawn', () => resolve(child)); child.once('error', () => resolve(child)); });
}

async function observe(options, state, event, snapshot) {
  let lastCapture = -Infinity;
  let lastProcessPoll = -Infinity;
  const captures = [];
  let maybeCapture;
  while (!state.closed && !state.stopReason) {
    const now = performance.now();
    if (options.timeoutMs !== undefined && now - state.started >= options.timeoutMs) { state.stopReason = 'timeout'; event('timeout'); break; }
    if (state.pid && now - lastProcessPoll >= options.processPollMs) {
      lastProcessPoll = now;
      try {
        const rows = await snapshot();
        const ownership = processOwnership(rows, state.pid, state.known);
        const current = ownership.owned;
        state.known = current;
        if (ownership.unanchored.some((row) => !row.state.startsWith('Z'))) {
          recordFailure(state.failures, { stage: 'process_ownership', error: 'unanchored_process_group' });
          state.stopReason = 'evidence_error';
        }
        if (!maybeCapture && captures.length < options.maxCaptures && now - state.lastOutput >= options.quietMs && now - lastCapture >= options.captureIntervalMs) {
          const ordinal = captures.length + 1;
          lastCapture = now;
          event('quiet_capture_started', { ordinal });
          captures.push({ ordinal, status: 'pending' });
          maybeCapture = captureProcesses({ evidenceRoot: options.evidenceRoot, ordinal, processes: current })
            .then((capture) => { captures[ordinal - 1] = { ordinal, status: 'captured', processCount: capture.processes.length, diagnosticFailures: capture.diagnostics.filter((item) => item.error || item.timedOut || item.exitCode !== 0).length }; })
            .catch((error) => { captures[ordinal - 1] = { ordinal, status: 'failed', error: error.message }; })
            .finally(() => { maybeCapture = undefined; event('quiet_capture_finished', { ordinal }); });
        }
      } catch (error) { recordFailure(state.failures, { stage: 'process_snapshot', error: error.message }); }
    }
    await delay(50);
  }
  return { captures, pending: maybeCapture };
}

/** Record one host command. Quiet output triggers evidence, never termination. */
export async function runRecordedCommand(input, { snapshot = processSnapshot } = {}) {
  const options = { quietMs: 15000, maxOutputBytes: 1048576, maxCaptures: 2, captureIntervalMs: 30000, processPollMs: 500, ...Object.fromEntries(Object.entries(input).filter(([, value]) => value !== undefined)) };
  validate(options);
  options.evidenceRoot = createEvidenceRoot(options.evidenceRoot);
  const started = performance.now();
  const events = [];
  const event = (type, details = {}) => events.push({ type, utc: new Date().toISOString(), offsetMs: Number((performance.now() - started).toFixed(3)), ...details });
  const failures = [];
  let ancestors = [];
  try { ancestors = await recorderAncestors(snapshot); }
  catch (error) { recordFailure(failures, { stage: 'recorder_ancestors', error: error.message }); }
  writeEvidence(options.evidenceRoot, 'invocation.json', { schema: 'host-stall-invocation-v1', command: options.command, cwd: options.cwd, label: options.label, environment: environmentMetadata({ ...process.env, ...options.env }), ancestors, quietMs: options.quietMs, timeoutMs: options.timeoutMs ?? null, maxCaptures: options.maxCaptures, maxOutputBytes: options.maxOutputBytes });
  const stdout = outputSink(options.evidenceRoot, 'stdout.log', options.maxOutputBytes);
  const stderr = outputSink(options.evidenceRoot, 'stderr.log', options.maxOutputBytes);
  const state = { started, lastOutput: started, pid: null, known: [], closed: false, exitCode: null, signal: null, spawnError: null, stopReason: null, markerTail: Buffer.alloc(0), markerObserved: false, failures };
  const cancel = (signal) => { if (!state.stopReason) { state.stopReason = 'cancelled'; event('cancelled', { signal }); } };
  const abort = () => cancel('AbortSignal');
  const sigint = () => cancel('SIGINT');
  const sigterm = () => cancel('SIGTERM');
  process.on('SIGINT', sigint);
  process.on('SIGTERM', sigterm);
  options.signal?.addEventListener('abort', abort, { once: true });
  if (options.signal?.aborted) abort();
  let observation = { captures: [], pending: undefined };
  let cleanup;
  let maybeChild;
  try {
    if (state.stopReason) state.closed = true;
    else {
      state.started = performance.now();
      state.lastOutput = state.started;
      maybeChild = await startChild(options, event, stdout, stderr, state);
      if (state.pid) {
        try { state.known = ownedProcesses(await snapshot(), state.pid, [], true); }
        catch (error) { recordFailure(state.failures, { stage: 'initial_process_snapshot', error: error.message }); }
      }
    }
    observation = await observe(options, state, event, snapshot);
    cleanup = await cleanupOwned(state.pid, state.known, snapshot);
    const until = performance.now() + 3000;
    while (!state.closed && performance.now() < until) await delay(25);
    if (!state.closed) { cleanup.complete = false; cleanup.failures.push({ error: 'child_streams_not_closed' }); }
    if (observation.pending) await observation.pending;
  } finally {
    process.off('SIGINT', sigint);
    process.off('SIGTERM', sigterm);
    options.signal?.removeEventListener('abort', abort);
    if (maybeChild) {
      for (const name of ['stdout', 'stderr']) {
        maybeChild[name].removeAllListeners('data');
        maybeChild[name].destroy();
      }
      maybeChild.unref();
    }
    stdout.close();
    stderr.close();
  }
  const summary = { schema: 'host-stall-run-v1', label: options.label, evidenceRoot: options.evidenceRoot, outcome: !cleanup.complete ? 'cleanup_failed' : failures.length ? 'evidence_error' : state.stopReason ?? (state.spawnError ? 'spawn_error' : state.exitCode === 0 ? 'success' : 'nonzero'), pid: state.pid, exitCode: state.exitCode, signal: state.signal, spawnError: state.spawnError, events, ancestors, stdout: stdout.result, stderr: stderr.result, captures: observation.captures, cleanup, failures, durationMs: Number((performance.now() - started).toFixed(3)) };
  writeEvidence(options.evidenceRoot, 'summary.json', summary);
  return summary;
}

import { spawn } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';
import { writeEvidence } from './evidence.mjs';

/** Execute a small diagnostic with bounded duration and bounded retained output. */
export async function runDiagnostic(command, timeoutMs = 2500, cap = 131072) {
  return new Promise((resolve) => {
    const result = { command, exitCode: null, signal: null, error: null, timedOut: false, unreaped: false, stdout: '', stderr: '', truncated: false };
    const child = spawn(command[0], command.slice(1), { shell: false, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
    const retained = { stdout: 0, stderr: 0 };
    for (const name of ['stdout', 'stderr']) child[name].on('data', (chunk) => {
      const keep = chunk.subarray(0, Math.max(0, cap - retained[name]));
      retained[name] += keep.length;
      result[name] += keep.toString('utf8');
      if (keep.length < chunk.length) result.truncated = true;
    });
    let maybeReapTimer;
    let settled = false;
    const finish = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      clearTimeout(maybeReapTimer);
      resolve(result);
    };
    const timer = setTimeout(() => {
      result.timedOut = true;
      if (child.exitCode === null && child.signalCode === null && child.pid) {
        try { process.kill(-child.pid, 'SIGKILL'); }
        catch (error) { if (error.code !== 'ESRCH') result.error = { code: error.code ?? 'KILL_FAILED', message: 'Diagnostic group could not be signaled' }; }
      } else result.error = { code: 'CLEANUP_UNCONFIRMED', message: 'Diagnostic root exited before descendants closed the streams' };
      maybeReapTimer = setTimeout(() => {
        result.unreaped = true;
        child.stdout.destroy();
        child.stderr.destroy();
        child.unref();
        finish();
      }, 500);
    }, timeoutMs);
    child.on('error', (error) => { result.error = { code: error.code ?? null, message: error.message }; });
    child.on('close', (code, signal) => { result.exitCode = code; result.signal = signal; finish(); });
  });
}

/** Parse bounded numeric metadata; global process listings never include executable paths. */
export function parseProcessRows(output) {
  const lines = output.split('\n').filter(Boolean);
  if (lines.length > 8192) throw new Error('process_snapshot_row_limit');
  return lines.map((line) => {
    const maybeMatch = line.match(/^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\S+\s+\S+\s+\d+\s+[\d:]+\s+\d+)\s+(\S+)\s+([\d.]+)\s*$/);
    if (!maybeMatch) throw new Error('process_snapshot_parse_failed');
    return { pid: Number(maybeMatch[1]), ppid: Number(maybeMatch[2]), pgid: Number(maybeMatch[3]), startedAt: maybeMatch[4], state: maybeMatch[5], cpuPercent: Number(maybeMatch[6]) };
  });
}

/** Read compact identifiers and resource metadata with an explicit capacity bound. */
export async function processSnapshot() {
  const result = await runDiagnostic(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,lstart=,state=,%cpu='], 2500, 524288);
  if (result.error || result.exitCode !== 0 || result.timedOut || result.truncated) throw new Error(`process_snapshot_failed:${JSON.stringify({ code: result.error?.code ?? null, exitCode: result.exitCode, timedOut: result.timedOut, truncated: result.truncated })}`);
  return parseProcessRows(result.stdout);
}

/** Read a name only for an explicitly selected process, never for the whole machine. */
export async function processExecutable(pid) {
  const result = await runDiagnostic(['/bin/ps', '-p', String(pid), '-o', 'comm='], 2500, 16384);
  if (result.error || result.exitCode !== 0 || result.timedOut || result.truncated) throw new Error('selected_process_name_unavailable');
  return result.stdout.trim();
}

export const sameProcess = (left, right) => left.pid === right.pid && left.startedAt === right.startedAt && left.pgid === right.pgid;

/** Select only live identity anchors and their descendants; retired groups confer no authority. */
export function processOwnership(rows, pid, known, admitFreshRoot = false) {
  const liveKnown = rows.filter((row) => known.some((entry) => sameProcess(entry, row)));
  const groupAnchored = liveKnown.some((row) => row.pgid === pid)
    || (admitFreshRoot && rows.some((row) => row.pid === pid && row.pgid === pid));
  const owned = rows.filter((row) => liveKnown.includes(row) || (groupAnchored && row.pgid === pid));
  let changed = true;
  while (changed) {
    changed = false;
    for (const row of rows) {
      if (row.pgid === pid && !groupAnchored) continue;
      if (!owned.some((entry) => entry.pid === row.pid) && owned.some((entry) => entry.pid === row.ppid)) {
        owned.push(row);
        changed = true;
      }
    }
  }
  return { owned, unanchored: rows.filter((row) => row.pgid === pid && !owned.includes(row)) };
}

export function ownedProcesses(rows, pid, known, admitFreshRoot = false) {
  return processOwnership(rows, pid, known, admitFreshRoot).owned;
}

/** Prioritize blocked leaf work over shell/build wrappers within the capture budget. */
export function diagnosticProcesses(processes, limit = 3) {
  const alive = processes.filter((entry) => !entry.state.startsWith('Z'));
  const leaves = alive.filter((entry) => !alive.some((child) => child.ppid === entry.pid));
  const roots = alive.filter((entry) => !alive.some((parent) => parent.pid === entry.ppid));
  return [...new Set([...leaves, ...roots, ...alive])].slice(0, limit);
}

export async function captureProcesses({ evidenceRoot, ordinal, processes }) {
  const capture = { ordinal, utc: new Date().toISOString(), processes, diagnostics: [] };
  for (const processInfo of diagnosticProcesses(processes)) {
    const current = await processSnapshot();
    if (!current.some((entry) => sameProcess(entry, processInfo))) {
      capture.diagnostics.push({ pid: processInfo.pid, skipped: 'process_identity_changed' });
      continue;
    }
    const commands = [['/usr/sbin/lsof', '-nP', '-p', String(processInfo.pid)]];
    if (process.platform === 'darwin') commands.push(['/usr/bin/sample', String(processInfo.pid), '1', '10']);
    for (const command of commands) {
      if (!(await processSnapshot()).some((entry) => sameProcess(entry, processInfo))) {
        capture.diagnostics.push({ pid: processInfo.pid, skipped: 'process_identity_changed' });
        break;
      }
      const result = await runDiagnostic(command);
      capture.diagnostics.push({ pid: processInfo.pid, ...result });
    }
  }
  writeEvidence(evidenceRoot, `capture-${String(ordinal).padStart(3, '0')}.json`, capture);
  return capture;
}

/** Stop only revalidated identities; clean descendants even after their parent exits. */
export async function cleanupOwned(pid, known, snapshot = processSnapshot) {
  const result = { complete: false, signals: [], failures: [], remaining: [] };
  if (!pid) return { ...result, complete: true };
  const signalOwned = async (signal) => {
    const rows = await snapshot();
    const ownership = processOwnership(rows, pid, known);
    const current = ownership.owned.filter((row) => !row.state.startsWith('Z'));
    for (const row of ownership.unanchored.filter((entry) => !entry.state.startsWith('Z'))) {
      if (!result.failures.some((entry) => entry.pid === row.pid)) result.failures.push({ pid: row.pid, error: 'unanchored_process_group' });
    }
    known = current;
    for (const row of current) {
      const fresh = await snapshot();
      if (!fresh.some((entry) => sameProcess(entry, row))) continue;
      try { process.kill(row.pid, signal); result.signals.push({ pid: row.pid, signal }); }
      catch (error) { if (error.code !== 'ESRCH') result.failures.push({ pid: row.pid, error: error.message }); }
    }
  };
  try {
    await signalOwned('SIGTERM');
    await delay(150);
    await signalOwned('SIGKILL');
    await delay(50);
    const finalOwnership = processOwnership(await snapshot(), pid, known);
    result.remaining = [...finalOwnership.owned, ...finalOwnership.unanchored].filter((row) => !row.state.startsWith('Z'));
    result.complete = result.remaining.length === 0 && result.failures.length === 0;
  } catch (error) { result.failures.push({ error: error.message }); }
  return result;
}

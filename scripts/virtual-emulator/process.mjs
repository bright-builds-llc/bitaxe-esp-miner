import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { open, writeFile, rm, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

/** Bounded private command; a timeout is a failure unless deliberately collecting emulator output. */
export async function runPrivate(command, args, root, label, { cwd, env = process.env, timeoutMs = 120000, allowTimeout = false, input, inputDelayMs = 0, inputReadyMarker } = {}) {
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 1200000) throw Error('emulator_timeout_bound');
  const writer = join(root, `${label}.writer.json`);
  await writeFile(writer, JSON.stringify({ schema: 'bitaxe-private-writer-lease-v1', parent_pid: process.pid, label }), { flag: 'wx', mode: 0o600 });
  let released = true, maybeStdout, maybeStderr, maybeFirstFailure;
  try {
    maybeStdout = await open(join(root, `${label}.stdout.log`), 'wx', 0o600);
    maybeStderr = await open(join(root, `${label}.stderr.log`), 'wx', 0o600);
    const stdout = maybeStdout, stderr = maybeStderr;
    released = false;
    const result = await new Promise((resolve, reject) => {
      let timedOut = false, interrupted = false, maybeOwnerWrite, active = true;
      const child = spawn(command, args, { cwd, env, detached: true, stdio: ['pipe', stdout.fd, stderr.fd] });
      child.once('spawn', () => {
        const digest = value => createHash('sha256').update(value).digest('hex');
        maybeOwnerWrite = writeFile(join(root, `${label}.owner.json`), `${JSON.stringify({ schema: 'bitaxe-virtual-process-owner-v1',
          pid: child.pid, pgid: child.pid, parent_pid: process.pid, started_at: new Date().toISOString(),
          command_sha256: digest(command), arguments_sha256: digest(JSON.stringify(args)) })}\n`, { flag: 'wx', mode: 0o600 })
          .catch(() => { interrupted = true; kill(); });
      });
      const kill = () => { if (child.pid) { try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; } } };
      const interrupt = () => { interrupted = true; kill(); };
      process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
      const timer = setTimeout(() => { timedOut = true; kill(); }, timeoutMs);
      let maybeInputTimer;
      const cleanup = () => { active = false; clearTimeout(timer); clearTimeout(maybeInputTimer); process.removeListener('SIGINT', interrupt); process.removeListener('SIGTERM', interrupt); };
      child.stdin.on('error', error => { if (error.code !== 'EPIPE') { interrupted = true; kill(); } });
      // Keep UART open: EOF can suppress emulated boot output. Input is bounded before spawn.
      if (input) {
        const deliver = async () => {
          if (!active) return;
          if (inputReadyMarker) {
            try {
              const output = await readFile(join(root, `${label}.stdout.log`), 'utf8');
              if (!active) return;
              if (!output.includes(inputReadyMarker)) { maybeInputTimer = setTimeout(deliver, 25); return; }
            } catch { interrupted = true; kill(); return; }
          }
          if (active) child.stdin.write(input);
        };
        maybeInputTimer = setTimeout(deliver, inputDelayMs);
      }
      child.once('error', () => { cleanup(); if (!child.pid) released = true; reject(Error('emulator_spawn')); });
      child.once('close', async (code, signal) => {
        cleanup();
        try {
          await maybeOwnerWrite;
          kill();
          const deadline = Date.now() + 2000;
          for (;;) {
            try { process.kill(-child.pid, 0); } catch (error) { if (error.code === 'ESRCH') break; throw error; }
            if (Date.now() >= deadline) throw Error('emulator_process_group_alive');
            await wait(20);
          }
          released = true;
          resolve({ code, signal, timedOut, interrupted, released: true });
        } catch (error) { reject(error); }
      });
    });
    await writeFile(join(root, `${label}.process.json`), `${JSON.stringify(result)}\n`, { flag: 'wx', mode: 0o600 });
    if (result.interrupted || (!result.timedOut && result.code !== 0) || (result.timedOut && !allowTimeout)) throw Error('emulator_command_failed');
    return result;
  } catch (error) {
    maybeFirstFailure = error; throw error;
  } finally {
    const closes = await Promise.allSettled([maybeStdout?.close(), maybeStderr?.close()]);
    const failed = closes.filter(outcome => outcome.status === 'rejected');
    if (failed.length) {
      if (maybeFirstFailure) maybeFirstFailure.cleanupFailures = ['emulator_log_release_unproven'];
      else throw Error('emulator_log_release_unproven');
    } else if (released) {
      try { await rm(writer); }
      catch { if (maybeFirstFailure) maybeFirstFailure.cleanupFailures = ['emulator_writer_release_unproven']; else throw Error('emulator_writer_release_unproven'); }
    }
  }
}

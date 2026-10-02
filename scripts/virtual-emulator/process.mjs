import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { open, writeFile, rm, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

const LINE_BUFFER_BYTES = 64 * 1024;

// Projection compares raw bytes only; it never parses or interprets line contents.
function stdoutProjection(prefix) {
  const expected = Buffer.from(prefix);
  const partial = Buffer.alloc(LINE_BUFFER_BYTES);
  let used = 0;
  return {
    project(chunk) {
      const lines = [];
      let offset = 0;
      while (offset < chunk.length) {
        const newline = chunk.indexOf(10, offset);
        const end = newline < 0 ? chunk.length : newline + 1;
        const bytes = end - offset;
        if (used + bytes > LINE_BUFFER_BYTES) throw Error('emulator_line_bound');
        chunk.copy(partial, used, offset, end); used += bytes;
        if (newline >= 0) {
          if (used >= expected.length && partial.subarray(0, expected.length).equals(expected)) {
            lines.push(Buffer.from(partial.subarray(0, used)));
          }
          used = 0;
        }
        offset = end;
      }
      return Buffer.concat(lines);
    },
  };
}

/** Bounded private command; a timeout is a failure unless deliberately collecting emulator output. */
export async function runPrivate(command, args, root, label, { cwd, env = process.env, timeoutMs = 120000, allowTimeout = false, input, inputDelayMs = 0, inputReadyMarker, maxOutputBytes, outputLinePrefix } = {}) {
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 1200000) throw Error('emulator_timeout_bound');
  if (maxOutputBytes !== undefined && (!Number.isSafeInteger(maxOutputBytes) || maxOutputBytes < 1)) throw Error('emulator_output_bound_invalid');
  if (outputLinePrefix !== undefined && (maxOutputBytes === undefined || typeof outputLinePrefix !== 'string' ||
      outputLinePrefix.length === 0 || Buffer.byteLength(outputLinePrefix) > LINE_BUFFER_BYTES || /[\r\n\0]/.test(outputLinePrefix))) throw Error('emulator_output_projection_invalid');
  const writer = join(root, `${label}.writer.json`);
  await writeFile(writer, JSON.stringify({ schema: 'bitaxe-private-writer-lease-v1', parent_pid: process.pid, label }), { flag: 'wx', mode: 0o600 });
  let released = true, maybeStdout, maybeStderr, maybeFirstFailure;
  try {
    maybeStdout = await open(join(root, `${label}.stdout.log`), 'wx', 0o600);
    maybeStderr = await open(join(root, `${label}.stderr.log`), 'wx', 0o600);
    const stdout = maybeStdout, stderr = maybeStderr;
    released = false;
    const result = await new Promise((resolve, reject) => {
      let timedOut = false, interrupted = false, maybeOwnerWrite, active = true, maybeFailure;
      let outputLimitExceeded = false, outputBytes = 0, pendingWrites = Promise.resolve();
      let persistedOutputBytes = 0, outputLineLimitExceeded = false;
      const maybeProjection = outputLinePrefix === undefined ? undefined : stdoutProjection(outputLinePrefix);
      const capture = maxOutputBytes !== undefined;
      const child = spawn(command, args, { cwd, env, detached: true, stdio: ['pipe', capture ? 'pipe' : stdout.fd, capture ? 'pipe' : stderr.fd] });
      child.once('spawn', () => {
        const digest = value => createHash('sha256').update(value).digest('hex');
        maybeOwnerWrite = writeFile(join(root, `${label}.owner.json`), `${JSON.stringify({ schema: 'bitaxe-virtual-process-owner-v1',
          pid: child.pid, pgid: child.pid, parent_pid: process.pid, started_at: new Date().toISOString(),
          command_sha256: digest(command), arguments_sha256: digest(JSON.stringify(args)) })}\n`, { flag: 'wx', mode: 0o600 })
          .catch(() => { maybeFailure ??= Error('emulator_owner_persistence'); interrupted = true; kill(); });
      });
      const kill = () => { if (child.pid) { try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; } } };
      const interrupt = () => { interrupted = true; kill(); };
      if (capture) {
        const collect = (stream, file) => stream.on('data', chunk => {
          if (!active || outputLimitExceeded || maybeFailure) return;
          const accepted = chunk.subarray(0, Math.max(0, maxOutputBytes - outputBytes));
          outputBytes += accepted.length;
          let projected;
          try {
            projected = !maybeProjection ? accepted : file === stdout ? maybeProjection.project(accepted) : Buffer.alloc(0);
          } catch (error) {
            outputLineLimitExceeded = true; maybeFailure ??= error; kill(); return;
          }
          persistedOutputBytes += projected.length;
          // Serialize both streams so the shared byte budget and descriptor lifetime are bounded.
          if (projected.length) {
            stream.pause();
            pendingWrites = pendingWrites.then(async () => {
              let offset = 0;
              while (offset < projected.length) {
                const { bytesWritten } = await file.write(projected, offset, projected.length - offset);
                if (!bytesWritten) throw Error('emulator_log_write');
                offset += bytesWritten;
              }
            }).catch(() => { maybeFailure ??= Error('emulator_log_write'); interrupted = true; kill(); })
              .finally(() => { if (active) stream.resume(); });
          }
          if (accepted.length < chunk.length) { outputLimitExceeded = true; maybeFailure ??= Error('emulator_output_bound'); kill(); }
        });
        collect(child.stdout, stdout); collect(child.stderr, stderr);
      }
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
      child.once('error', () => { maybeFailure ??= Error('emulator_spawn'); });
      // Descendants may retain inherited pipes after the direct child has failed.
      child.once('exit', code => { if (!timedOut && code !== 0) maybeFailure ??= Error('emulator_command_failed'); });
      child.once('close', async (code, signal) => {
        cleanup();
        maybeFailure ??= interrupted || (!timedOut && code !== 0) || (timedOut && !allowTimeout) ? Error('emulator_command_failed') : undefined;
        try {
          await maybeOwnerWrite;
          await pendingWrites;
          kill();
          const deadline = Date.now() + 2000;
          while (child.pid) {
            try { process.kill(-child.pid, 0); } catch (error) { if (error.code === 'ESRCH') break; throw error; }
            if (Date.now() >= deadline) throw Error('emulator_process_group_alive');
            await wait(20);
          }
          released = true;
          const result = { code, signal, timedOut, interrupted, released: true };
          if (capture) Object.assign(result, { outputLimitExceeded, outputBytes, maxOutputBytes });
          if (maybeProjection) Object.assign(result, { outputLinePrefix, persistedOutputBytes,
            outputLineLimitExceeded, stdoutPartialLineLimitBytes: LINE_BUFFER_BYTES });
          await writeFile(join(root, `${label}.process.json`), `${JSON.stringify(result)}\n`, { flag: 'wx', mode: 0o600 });
          if (maybeFailure) reject(maybeFailure); else resolve(result);
        } catch (error) {
          if (maybeFailure) { maybeFailure.cleanupFailures = [error.message]; reject(maybeFailure); }
          else reject(error);
        }
      });
    });
    return result;
  } catch (error) {
    maybeFirstFailure = error; throw error;
  } finally {
    const closes = await Promise.allSettled([maybeStdout?.close(), maybeStderr?.close()]);
    const failed = closes.filter(outcome => outcome.status === 'rejected');
    if (failed.length) {
      if (maybeFirstFailure) maybeFirstFailure.cleanupFailures = [...(maybeFirstFailure.cleanupFailures ?? []), 'emulator_log_release_unproven'];
      else throw Error('emulator_log_release_unproven');
    } else if (released) {
      try { await rm(writer); }
      catch { if (maybeFirstFailure) maybeFirstFailure.cleanupFailures = [...(maybeFirstFailure.cleanupFailures ?? []), 'emulator_writer_release_unproven']; else throw Error('emulator_writer_release_unproven'); }
    }
  }
}

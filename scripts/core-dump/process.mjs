import { spawn } from 'node:child_process';
import { open } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

/** Confirm the detached decoder group cannot retain a writer after parent exit. */
export async function requireDecoderGroupGone(pid, options = {}) {
  if (!pid) return;
  const exists = options.exists ?? (() => { try { process.kill(-pid, 0); return true; } catch (error) { if (error.code === 'ESRCH') return false; throw error; } });
  const deadline = Date.now() + (options.limitMs ?? 2000);
  for (;;) {
    let present; try { present = exists(); } catch { throw Error('decoder_release_unproven'); }
    if (!present) return;
    if (Date.now() >= deadline) throw Error('decoder_release_unproven');
    await wait(Math.min(25, Math.max(1, deadline - Date.now())));
  }
}

/** Run an offline decoder in its own bounded process group; never expose vendor text. */
export async function privateProcess(command, args, root, label, env, timeoutMs = 120_000) {
  const stdout = await open(join(root, `${label}.stdout`), 'wx', 0o600);
  let maybeStderr;
  try {
    maybeStderr = await open(join(root, `${label}.stderr`), 'wx', 0o600);
    const stderr = maybeStderr;
    return await new Promise((resolve, reject) => {
      const child = spawn(command, args, { cwd: root, env, detached: true, stdio: ['ignore', stdout.fd, stderr.fd] });
      let timedOut = false, interrupted = false;
      const kill = () => {
        if (!child.pid) return;
        try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
      };
      const interrupt = () => { interrupted = true; kill(); };
      process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
      const clear = () => { clearTimeout(timer); process.removeListener('SIGINT', interrupt); process.removeListener('SIGTERM', interrupt); };
      const timer = setTimeout(() => { timedOut = true; kill(); }, timeoutMs);
      child.once('error', () => { clear(); reject(new Error('decoder_spawn')); });
      child.once('exit', async code => {
        clear();
        try {
          kill(); // Also release descendants if the immediate process exited first.
          await requireDecoderGroupGone(child.pid);
        } catch { reject(new Error('decoder_release_unproven')); return; }
        if (interrupted) reject(new Error('decoder_interrupted'));
        else if (timedOut) reject(new Error('decoder_timeout'));
        else if (code !== 0) reject(new Error('decoder_failed'));
        else resolve();
      });
    });
  } finally { await stdout.close(); await maybeStderr?.close(); }
}

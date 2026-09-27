import { spawn } from 'node:child_process';
import { open } from 'node:fs/promises';
import { join } from 'node:path';

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
      child.once('exit', code => {
        clear();
        kill(); // Also release descendants if the immediate process exited first.
        if (interrupted) reject(new Error('decoder_interrupted'));
        else if (timedOut) reject(new Error('decoder_timeout'));
        else if (code !== 0) reject(new Error('decoder_failed'));
        else resolve();
      });
    });
  } finally { await stdout.close(); await maybeStderr?.close(); }
}

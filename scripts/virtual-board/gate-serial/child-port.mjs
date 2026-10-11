// A Gate WorkerSerialPort over a virtual device child's stdio, plus owned cleanup.
import { spawn } from 'node:child_process';
import { openSync, closeSync } from 'node:fs';

export const DEVICE_FILTER = Object.freeze({ usbVendorId: 0x303a, usbProductId: 0x1001 });

/**
 * Starts the built device binary as its own process group; stderr goes to a private file.
 * `trust` is the deployment trust the device verifies leases with.
 */
export function startDevice({ binary, seed, capability, trust, outcome, stderrPath }) {
  const stderr = openSync(stderrPath, 'wx', 0o600);
  try {
    const child = spawn(binary, ['serve-serial', `--seed=${seed}`, `--capability=${capability}`, `--trust=${trust}`, `--outcome=${outcome}`],
      { detached: true, stdio: ['pipe', 'pipe', stderr] });
    const exited = new Promise(resolve => child.once('exit', (code, signal) => resolve({ code, signal })));
    return { child, exited };
  } finally { closeSync(stderr); }
}

/** The port the Gate opens; closing it ends the device's stdin. */
export function childPort({ child }) {
  let maybeReadable = null, maybeWritable = null;
  return {
    get readable() { return maybeReadable; },
    get writable() { return maybeWritable; },
    getInfo: () => ({ ...DEVICE_FILTER }),
    async open() {
      let ended = false;
      maybeReadable = new ReadableStream({
        start(controller) {
          child.stdout.on('data', data => { if (!ended) controller.enqueue(new Uint8Array(data)); });
          child.stdout.once('end', () => { if (!ended) { ended = true; controller.close(); } });
        },
        cancel() { ended = true; child.stdout.destroy(); },
      });
      maybeWritable = new WritableStream({
        write: chunk => new Promise((resolve, reject) => child.stdin.write(Buffer.from(chunk), error => error ? reject(error) : resolve())),
        close: () => new Promise(resolve => child.stdin.end(resolve)),
        abort: () => { child.stdin.destroy(); },
      });
    },
    async close() {
      maybeReadable = null; maybeWritable = null;
      if (!child.stdin.destroyed) child.stdin.end();
    },
  };
}

const groupAlive = pid => {
  try { process.kill(-pid, 0); return true; } catch (error) { if (error.code === 'ESRCH') return false; throw error; }
};

/** Waits for the device to exit, kills its group after the bound, and proves the group is gone. */
export async function releaseDevice({ child, exited }, boundMs = 5000) {
  let maybeTimer;
  const bound = new Promise(resolve => { maybeTimer = setTimeout(() => resolve(null), boundMs); });
  let maybeExit = await Promise.race([exited, bound]);
  clearTimeout(maybeTimer);
  const killed = maybeExit === null;
  if (killed) { process.kill(-child.pid, 'SIGKILL'); maybeExit = await exited; }
  for (const started = Date.now(); groupAlive(child.pid);) {
    if (Date.now() - started > 2000) throw Error('device_process_group_alive');
    await new Promise(resolve => setTimeout(resolve, 20));
  }
  return { code: maybeExit.code, signal: maybeExit.signal, killed, released: true };
}

// The durable parent claim precedes permission to launch the sole core tool.
import { spawn } from 'node:child_process';
let launched = false;
const terminate = () => { try { process.kill(-process.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') process.exit(1); } };
process.once('disconnect', terminate);
process.on('message', message => {
  if (launched || message?.kind !== 'run' || typeof message.program !== 'string' || !Array.isArray(message.args) ||
    !message.args.every(value => typeof value === 'string')) { terminate(); return; }
  launched = true;
  const child = spawn(message.program, message.args, { stdio: 'inherit', detached: false });
  child.once('spawn', () => process.send?.({ kind: 'worker_spawn', pid: child.pid }));
  child.once('error', () => process.send?.({ kind: 'worker_exit', code: null, signal: null, spawn_failed: true }));
  child.once('close', (code, signal) => process.send?.({ kind: 'worker_exit', code, signal, spawn_failed: false }));
  // Keep this group owner alive until its parent reaps the whole group.
});
process.send?.({ kind: 'waiting' });

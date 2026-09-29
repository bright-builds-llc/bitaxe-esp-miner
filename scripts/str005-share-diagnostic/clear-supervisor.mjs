import { nodeRuntimeEnvironment } from '../str005-noise-serial/node-runtime.mjs';
import { spawn } from 'node:child_process';
import { open } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { processSnapshot, sameProcess, checkedOwner } from '../str005-v2-serial/host-resources.mjs';
import { proof, writeNew } from '../str005-noise-serial/files.mjs';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export async function runClearChild(program, args, root, timeoutMs = 1200000) {
  const helper = fileURLToPath(new URL('./clear-child.mjs', import.meta.url));
  const stdout = await open(resolve(root, 'clear-command.stdout'), 'wx', 0o600); let stderr;
  try {
    stderr = await open(resolve(root, 'clear-command.stderr'), 'wx', 0o600);
    return await new Promise((resolveResult, reject) => {
      const child = spawn(process.execPath, [helper], { cwd: root, detached: true, stdio: ['ignore', stdout.fd, stderr.fd, 'ipc'],
        env: { ...nodeRuntimeEnvironment(), PATH: process.env.PATH ?? '/usr/bin:/bin', HOME: process.env.HOME ?? '', LANG: 'C', LC_ALL: 'C' } });
      let owner, workerOwner, workerExit, timedOut = false, interrupted = false, firstFailure = null, stopped = false, queue = Promise.resolve();
      const kill = () => { if (stopped || !child.pid) return; stopped = true;
        try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') firstFailure ??= 'group_kill_failed'; } };
      const fail = category => { firstFailure ??= category; kill(); };
      const interrupt = () => { interrupted = true; fail('interrupted'); };
      process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
      const timer = setTimeout(() => { timedOut = true; fail('timeout'); }, timeoutMs);
      child.on('message', message => { queue = queue.then(async () => {
        if (message.kind === 'waiting') {
          check(!owner && child.pid, 'diagnostic_child_state');
          owner = checkedOwner((await processSnapshot()).find(row => row.pid === child.pid));
          check(owner.pgid === child.pid, 'diagnostic_child_group');
          await writeNew(resolve(root, 'clear-process-owner.json'), { schema: 'str005-clear-process-owner-v1', owner,
            claimSha256: (await proof(root, 'clear-claim.json')).sha256, programSha256: await fileDigest(program),
            argsSha256: sha256(JSON.stringify(args)), supervisorSha256: await fileDigest(helper) });
          if (!stopped) child.send({ kind: 'run', program, args });
        } else if (message.kind === 'worker_spawn') {
          check(owner && !workerOwner && Number.isSafeInteger(message.pid), 'diagnostic_worker_state');
          workerOwner = checkedOwner((await processSnapshot()).find(row => row.pid === message.pid));
          check(workerOwner.pgid === owner.pgid, 'diagnostic_worker_group');
          await writeNew(resolve(root, 'clear-worker-owner.json'), { schema: 'str005-clear-worker-owner-v1', owner: workerOwner });
        } else if (message.kind === 'worker_exit') {
          workerExit = message; if (message.code !== 0 || message.signal !== null || message.spawn_failed) firstFailure ??= 'child_failed';
          kill();
        } else fail('child_protocol');
      }).catch(() => fail('child_identity_unproved')); });
      child.once('error', () => fail('supervisor_spawn_failed'));
      child.once('close', () => { clearTimeout(timer); process.removeListener('SIGINT', interrupt); process.removeListener('SIGTERM', interrupt);
        queue.then(async () => {
          let groupReleased = false;
          if (owner) {
            for (let attempt = 0; attempt < 50; attempt++) {
              const current = await processSnapshot();
              if (!current.some(row => row.pgid === owner.pgid || sameProcess(row, owner) || (workerOwner && sameProcess(row, workerOwner)))) { groupReleased = true; break; }
              await new Promise(done => setTimeout(done, 100));
            }
          }
          if (!workerExit) firstFailure ??= 'child_exit_unproved';
          if (!groupReleased) firstFailure ??= 'group_release_unproved';
          const result = { schema: 'str005-clear-process-result-v1', code: workerExit?.code ?? null,
            signal: workerExit?.signal ?? null, spawn_failed: workerExit?.spawn_failed ?? null,
            timed_out: timedOut, interrupted, first_failure: firstFailure, group_released: groupReleased,
            owner_recorded: Boolean(owner), worker_recorded: Boolean(workerOwner), completedAtUnixMs: Date.now() };
          await writeNew(resolve(root, 'clear-process-result.json'), result); resolveResult(result);
        }).catch(reject);
      });
    });
  } finally { await stdout.close(); await stderr?.close(); }
}

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, realpath, rm, writeFile, access, stat, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { writeNew, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { processSnapshot } from '../str005-v2-serial/host-resources.mjs';
import { runClearChild } from './clear-supervisor.mjs';
import { finishClear } from './clear.mjs';
async function fixture(t, args) {
  const parent = await realpath(await mkdtemp(resolve(tmpdir(), 'clear-owned-'))), root = resolve(parent, 'attempt');
  await mkdir(root, { mode: 0o700 }); t.after(() => rm(parent, { recursive: true, force: true }));
  const context = { commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), detector: { physical: 'f'.repeat(64) }, flashBinarySha256: await fileDigest(process.execPath) };
  await writeNew(resolve(root, 'clear-claim.json'), { schema: 'str005-diagnostic-clear-claim-v1',
    owner: { pid: 2147483000, pgid: 2147483000, startedAt: 'synthetic-gone-parent' }, port: '/dev/cu.fixture', args });
  return { root, parent, context };
}
async function waitWorker(root) {
  for (let i = 0; i < 100; i++) {
    try { return await proof(root, 'clear-worker-owner.json'); } catch (error) { if (error.code !== 'ENOENT') throw error; }
    await new Promise(done => setTimeout(done, 20));
  }
  throw Error('worker_not_recorded');
}
test('real managed child and supervisor identities are durable before completion and group release', async t => {
  // Arrange
  const args = ['-e', 'setTimeout(()=>process.exit(0),500)']; const f = await fixture(t, args);
  // Act
  const result = await runClearChild(process.execPath, args, f.root, 5000);
  // Assert
  assert.equal(result.code, 0, JSON.stringify({ result, stderr: (await readFile(resolve(f.root, 'clear-command.stderr'), 'utf8')).slice(0, 3000) })); assert.equal(result.first_failure, null); assert.equal(result.group_released, true);
  const owner = (await proof(f.root, 'clear-process-owner.json')).value.owner;
  const worker = (await proof(f.root, 'clear-worker-owner.json')).value.owner;
  assert.equal(owner.pgid, owner.pid); assert.equal(worker.pgid, owner.pgid); assert.notEqual(owner.pid, worker.pid);
  assert.equal((await processSnapshot()).some(row => row.pgid === owner.pgid), false);
  assert.equal((await stat(resolve(f.root, 'clear-command.stdout'))).mode & 0o777, 0o600);
});
test('live child never seals; failed child retains cause until independent physical cleanup permits sealing', async t => {
  // Arrange
  const record = { schema_version: 'bitaxe-development-core-dump-clear-v1', source_commit: 'a'.repeat(40), expected_installed_source: 'b'.repeat(40),
    expected_installed_elf: 'c'.repeat(64), first_failure_stage: 'dump_erase', cleanup_complete: false };
  const code = `const f=require('fs');f.mkdirSync('clear',{mode:448});f.writeFileSync('clear/result.private.json',${JSON.stringify(JSON.stringify(record))},{mode:384});setTimeout(()=>process.exit(7),750);`;
  const args = ['-e', code], f = await fixture(t, args);
  const running = runClearChild(process.execPath, args, f.root, 5000); await waitWorker(f.root);
  // Act / Assert
  const live = await finishClear(f.root, f.context); assert.equal(live.sealed, false);
  await assert.rejects(access(resolve(f.root, 'sealed-inventory.json')));
  const exited = await running; assert.equal(exited.code, 7); assert.equal(exited.group_released, true);
  const missingCleanup = await finishClear(f.root, f.context);
  assert.equal(missingCleanup.sealed, false); assert.equal(missingCleanup.first_failure, 'diagnostic_clear_child_dump_erase');
  assert.equal(missingCleanup.cleanup_failure, 'diagnostic_clear_cleanup_unproved');
  await writeFile(resolve(f.parent, 'cleanup-detector.stdout.log'), `port: /dev/cu.returned\nphysical_identity_sha256: ${f.context.detector.physical}\nusb_profile: serial_jtag_runtime\n`, { mode: 0o600 });
  const held = await finishClear(f.root, f.context, { requireNoHolders: () => { throw Error('synthetic-holder'); } });
  assert.equal(held.sealed, false); await assert.rejects(access(resolve(f.root, 'sealed-inventory.json')));
  const checked = []; const final = await finishClear(f.root, f.context, { requireNoHolders: port => checked.push(port) });
  assert.equal(final.complete, false); assert.equal(final.first_failure, 'diagnostic_clear_child_dump_erase');
  assert.equal(final.cleanup_failure, null); assert.equal(final.host_resources_released, true);
  assert.deepEqual(checked, ['/dev/cu.fixture', '/dev/cu.returned']);
  const seal = await proof(f.root, 'sealed-inventory.json'); await verifyInventory(f.root, seal.value.files, new Set(['sealed-inventory.json']));
});
test('real child timeout is reaped and cannot be reported as a successful operation', async t => {
  // Arrange
  const args = ['-e', 'setInterval(()=>{},1000)'], f = await fixture(t, args);
  // Act
  const result = await runClearChild(process.execPath, args, f.root, 1000);
  // Assert
  assert.equal(result.timed_out, true); assert.equal(result.first_failure, 'timeout'); assert.equal(result.group_released, true);
});

test('conflicting successful native clear and failed runner can seal only a failed outcome', async t => {
  // Arrange: no process or device effect; all files are synthetic producer receipts.
  const { clearArguments, clearRunnerSucceeded, verifyClear } = await import('./clear.mjs');
  const { sha256 } = await import('../str005-v2-serial/values.mjs');
  const good = { schema: 'str005-clear-process-result-v1', code: 0, signal: null, spawn_failed: false, timed_out: false, interrupted: false,
    first_failure: null, group_released: true, owner_recorded: true, worker_recorded: true };
  for (const change of [{ code: 7 }, { signal: 'SIGKILL' }, { spawn_failed: true }, { timed_out: true }, { interrupted: true },
    { first_failure: 'timeout' }, { group_released: false }, { owner_recorded: false }, { worker_recorded: false }]) {
    assert.equal(clearRunnerSucceeded({ ...good, ...change }), false);
  }
  const parent = await realpath(await mkdtemp(resolve(tmpdir(), 'clear-conflict-'))), root = resolve(parent, 'attempt');
  await mkdir(root, { mode: 0o700 }); await mkdir(resolve(root, 'clear'), { mode: 0o700 }); t.after(() => rm(parent, { recursive: true, force: true }));
  const original = Buffer.alloc(974848, 0x42);
  const context = { commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), detector: { physical: 'f'.repeat(64) },
    archivePath: '/private/original', archiveSha: sha256(original), flashBinarySha256: 'd'.repeat(64) };
  const owner = { pid: 2147483000, pgid: 2147483000, startedAt: 'synthetic-parent' }, child = { pid: 2147483001, pgid: 2147483001, startedAt: 'synthetic-child' },
    worker = { pid: 2147483002, pgid: 2147483001, startedAt: 'synthetic-worker' };
  const args = clearArguments(root, context, '/dev/cu.fixture');
  await writeNew(resolve(root, 'clear-claim.json'), { schema: 'str005-diagnostic-clear-claim-v1', owner, contextSha256: sha256(JSON.stringify(context)),
    port: '/dev/cu.fixture', args, startedAtUnixMs: 1000 });
  await writeNew(resolve(root, 'clear-process-owner.json'), { schema: 'str005-clear-process-owner-v1', owner: child,
    claimSha256: (await proof(root, 'clear-claim.json')).sha256, argsSha256: sha256(JSON.stringify(args)), programSha256: context.flashBinarySha256 });
  await writeNew(resolve(root, 'clear-worker-owner.json'), { schema: 'str005-clear-worker-owner-v1', owner: worker });
  await writeNew(resolve(root, 'clear-process-result.json'), { ...good, group_released: false, first_failure: 'group_release_unproved' });
  await writeNew(resolve(root, 'clear-exit.json'), { code: 0, finishedAtUnixMs: 2000 });
  await writeNew(resolve(root, 'clear/result.private.json'), { schema_version: 'bitaxe-development-core-dump-clear-v1', clearing_complete: true,
    terminal_category: 'complete', first_failure_stage: 'complete', source_commit: context.commit, rom_admitted: true, acquisition_complete: true,
    application_identity_restored: true, hardware_baseline_verified: false, cleanup_complete: true,
    expected_installed_source: context.firmware_commit, expected_installed_elf: context.app_elf_sha256 });
  await writeFile(resolve(root, 'clear/core-dump.private.bin'), original, { mode: 0o600 });
  await writeFile(resolve(root, 'clear/cleared-core-dump.private.bin'), Buffer.alloc(974848, 255), { mode: 0o600 });
  await writeFile(resolve(parent, 'cleanup-detector.stdout.log'), `port: /dev/cu.returned\nphysical_identity_sha256: ${context.detector.physical}\nusb_profile: serial_jtag_runtime\n`, { mode: 0o600 });
  await verifyClear(root, context); // Native receipt, exit and bytes are otherwise a pass.
  // Act
  const result = await finishClear(root, context, { requireNoHolders: () => {} });
  // Assert
  assert.equal(result.complete, false); assert.equal(result.first_failure, 'diagnostic_clear_group_release_unproved');
  assert.equal(result.original_group_release_verified, false); assert.equal(result.current_group_absence_verified, true);
  assert.equal(result.host_resources_released, true); assert.equal(result.cleanup_failure, null);
  const seal = await proof(root, 'sealed-inventory.json'); await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
});

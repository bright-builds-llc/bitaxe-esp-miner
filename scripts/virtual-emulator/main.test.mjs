import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { EMULATOR, selectRelease } from './lock.mjs';
import { qemuArguments } from './run.mjs';
import { runPrivate } from './process.mjs';

function manifest() {
  return { tools: [{ name: EMULATOR.tool, versions: [{ name: EMULATOR.version, status: 'recommended',
    'macos-arm64': { ...EMULATOR.platforms['darwin-arm64'], url: 'https://github.com/espressif/qemu/releases/download/esp-develop-9.2.2-20250817/qemu.tar.xz' } }] }] };
}

test('selects the official SDK recommended ARM64 release', () => {
  // Arrange
  const sdk = manifest();
  // Act
  const actual = selectRelease(sdk, 'darwin', 'arm64');
  // Assert
  assert.equal(actual.sha256, 'aa92e337461d482f5d9f31cd8efc0bd67b3de8fcfcfb567289cb43a59c184651');
});

test('rejects SDK archive identity drift', () => {
  // Arrange
  const sdk = manifest(); sdk.tools[0].versions[0]['macos-arm64'].sha256 = 'f'.repeat(64);
  // Act / Assert
  assert.throws(() => selectRelease(sdk, 'darwin', 'arm64'), /emulator_manifest_digest/);
});

test('rejects substitution of another recommended version', () => {
  // Arrange
  const sdk = manifest(); sdk.tools[0].versions[0].name = 'latest';
  // Act / Assert
  assert.throws(() => selectRelease(sdk, 'darwin', 'arm64'), /emulator_manifest_version/);
});

test('pins dual-core S3 and eight-megabyte octal PSRAM without network', () => {
  // Arrange / Act
  const args = qemuArguments('/private/virtual.bin', '/private/efuse.bin');
  // Assert
  assert.deepEqual(args.slice(0, 6), ['-M', 'esp32s3', '-smp', '2', '-m', '8M']);
  assert.ok(args.includes('driver=ssi_psram,property=is_octal,value=true'));
  assert.equal(args[args.indexOf('-nic') + 1], 'none');
  assert.ok(args.includes('driver=timer.esp32s3.timg,property=wdt_disable,value=true'));
});

test('bounded runner records timeout and releases the process group', async () => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'virtual-process-'));
  try {
    // Act
    const actual = await runPrivate(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], root, 'timeout', { timeoutMs: 50, allowTimeout: true });
    // Assert
    assert.equal(actual.timedOut, true); assert.equal(actual.released, true);
    assert.equal(JSON.parse(await readFile(join(root, 'timeout.process.json'), 'utf8')).released, true);
  } finally { await rm(root, { recursive: true }); }
});

test('failed completion does not become success because cleanup passed', async () => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'virtual-process-'));
  try {
    // Act / Assert
    await assert.rejects(runPrivate(process.execPath, ['-e', 'process.exit(7)'], root, 'failed', { timeoutMs: 1000 }), /emulator_command_failed/);
    const result = JSON.parse(await readFile(join(root, 'failed.process.json'), 'utf8'));
    assert.equal(result.code, 7); assert.equal(result.released, true);
  } finally { await rm(root, { recursive: true }); }
});

import { judgeTargetEvents } from './judge.mjs';
const guestManifest = { compiled_source_sha256: 'a'.repeat(64) };
const goodEvents = () => [
  { event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: 'a'.repeat(64), heartbeat_cutoff_ms: 2800, boot: 1 },
  { event: 'task', joined: true, core: 1, stack_bytes: 8192, minimum_stack_free_bytes: 6000 },
  { event: 'allocation', bytes: 8192, before: 100000, during: 91800, after: 100000, released: true },
  { event: 'status', boot: 1, internal_free: 100000, internal_largest: 60000, psram_free: 8000000 },
];

test('a task event with a failed join cannot qualify target execution', () => {
  // Arrange
  const events = goodEvents(); events[1].joined = false;
  // Act
  const checks = judgeTargetEvents(events, guestManifest);
  // Assert
  assert.equal(checks.find(check => check.id === 'target_task_join').status, 'failed');
});

test('allocator release requires observed free memory restoration', () => {
  // Arrange
  const events = goodEvents(); events[2].after = 99999;
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { commands: ['allocation'] });
  // Assert
  assert.equal(checks.find(check => check.id === 'target_internal_allocation_release').status, 'failed');
});

test('restart requires a second boot with the persisted next counter', () => {
  // Arrange
  const events = goodEvents(); events.push({ ...events[0], boot: 1 });
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { commands: ['restart'] });
  // Assert
  assert.equal(checks.find(check => check.id === 'target_reset_persistence').status, 'failed');
});

test('unsupported shared checks survive successful target execution', () => {
  // Arrange
  const events = goodEvents(); events.push({ event: 'scenario', result: { scenario: 'healthy-lifecycle', seed: 7, hardware_qualified: false, checks: [{ id: 'encrypted_share', status: 'unsupported' }] } });
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { scenario: 'healthy-lifecycle', seed: 7 });
  // Assert
  assert.equal(checks.find(check => check.id === 'scenario:encrypted_share').status, 'unsupported');
});

test('a guest built from a different source closure fails identity', () => {
  // Arrange
  const events = goodEvents(); events[0].compiled_source_sha256 = 'b'.repeat(64);
  // Act
  const checks = judgeTargetEvents(events, guestManifest);
  // Assert
  assert.equal(checks.find(check => check.id === 'target_boot_identity').status, 'failed');
});

import { createConnection } from 'node:net';
import { stat } from 'node:fs/promises';

test('natural failure releases a real descendant listener and the durable writer lease', async () => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'virtual-descendant-'));
  const ready = join(root, 'listener.json');
  const descendant = "const fs=require('fs');const net=require('net');const server=net.createServer();server.listen(0,'127.0.0.1',()=>fs.writeFileSync(process.argv[1],JSON.stringify({pid:process.pid,port:server.address().port})));";
  const parent = "const {spawn}=require('child_process');const fs=require('fs');spawn(process.execPath,['-e',process.argv[1],process.argv[2]],{stdio:'ignore'});const started=Date.now();const timer=setInterval(()=>{if(fs.existsSync(process.argv[2]))process.exit(7);if(Date.now()-started>1000)process.exit(8);},10);";
  try {
    // Act
    await assert.rejects(runPrivate(process.execPath, ['-e', parent, descendant, ready], root, 'descendant', { timeoutMs: 2000 }), /emulator_command_failed/);
    const listener = JSON.parse(await readFile(ready, 'utf8'));
    const released = await new Promise(resolve => {
      const socket = createConnection({ host: '127.0.0.1', port: listener.port });
      socket.once('connect', () => { socket.destroy(); resolve(false); });
      socket.once('error', error => resolve(error.code === 'ECONNREFUSED'));
    });
    // Assert
    assert.equal(released, true);
    await assert.rejects(stat(join(root, 'descendant.writer.json')), { code: 'ENOENT' });
    const outcome = JSON.parse(await readFile(join(root, 'descendant.process.json'), 'utf8'));
    assert.equal(outcome.code, 7); assert.equal(outcome.released, true);
  } finally { await rm(root, { recursive: true }); }
});

import { auditGuestFrames } from './frame-audit.mjs';

test('native audit rejects the old retained 4096-byte receive frame in dispatch', () => {
  // Arrange
  const old = '42000000 <bitaxe_virtual_firmware::guest::run>:\n42000000:  entry a1, 0x12c0\n';
  // Act / Assert
  assert.throws(() => auditGuestFrames(old, 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384'), /guest_receive_frame_not_released/);
});

test('native audit accepts separate bounded receive and dispatch frames', () => {
  // Arrange
  const separated = '42000000 <bitaxe_virtual_firmware::guest::run>:\n42000000:  entry a1, 832\n42001000 <bitaxe_virtual_firmware::guest::read_command>:\n42001000:  entry a1, 0x1070\n';
  // Act
  const actual = auditGuestFrames(separated, 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384');
  // Assert
  assert.equal(actual.dispatch_frame_bytes, 832); assert.equal(actual.receive_frame_bytes, 4208);
  assert.equal(actual.complete_call_closure_audited, false);
});

import { parseCheckpoints } from './checkpoints.mjs';

test('entered-only checkpoint keeps phase while reporting facts unavailable', () => {
  // Arrange
  const raw = Buffer.alloc(384); raw.writeUInt32LE(0x56484331, 0); raw.writeUInt32LE(3, 4); raw.writeUInt32LE(1, 8);
  // Act
  const records = parseCheckpoints(raw);
  // Assert
  assert.equal(records[0].phase, 3); assert.equal(records[0].integrity, null);
  assert.equal(records[0].stack_pointer_inside, null); assert.equal(records[0].internal_free, null);
});

test('completed checkpoint distinguishes an observed integrity failure', () => {
  // Arrange
  const raw = Buffer.alloc(384); raw.writeUInt32LE(0x56484331, 0); raw.writeUInt32LE(3, 4); raw.writeUInt32LE(2, 8);
  // Act
  const records = parseCheckpoints(raw);
  // Assert
  assert.equal(records[0].integrity, false); assert.equal(records[0].internal_free, 0);
});

test('checkpoint decoder rejects another ABI', () => {
  // Arrange
  const raw = Buffer.alloc(384); raw.writeUInt32LE(0x56484332, 0);
  // Act / Assert
  assert.throws(() => parseCheckpoints(raw), /checkpoint_format/);
});

import { writeFile } from 'node:fs/promises';

test('second log open failure releases the first log without starting a child', async () => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'virtual-log-failure-'));
  await writeFile(join(root, 'persist.stderr.log'), 'retained', { flag: 'wx' });
  try {
    // Act / Assert
    await assert.rejects(runPrivate(process.execPath, ['-e', 'process.exit(0)'], root, 'persist'), { code: 'EEXIST' });
    await assert.rejects(stat(join(root, 'persist.writer.json')), { code: 'ENOENT' });
    await assert.rejects(stat(join(root, 'persist.owner.json')), { code: 'ENOENT' });
    assert.equal(await readFile(join(root, 'persist.stderr.log'), 'utf8'), 'retained');
  } finally { await rm(root, { recursive: true }); }
});

test('scenario success requires an actual main task stack margin observation', () => {
  // Arrange
  const events = goodEvents();
  events.push({ event: 'scenario', result: { scenario: 'healthy-lifecycle', seed: 1, hardware_qualified: false, checks: [] } });
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { scenario: 'healthy-lifecycle' });
  // Assert
  assert.equal(checks.find(check => check.id === 'target_main_stack_margin').status, 'failed');
});

test('exact existing two-kibibyte target margin passes without increasing limits', () => {
  // Arrange
  const events = goodEvents();
  events.push({ event: 'scenario_margin', minimum_main_stack_free_bytes: 2048, required_margin_bytes: 2048, configured_main_stack_bytes: 16384 });
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { scenario: 'healthy-lifecycle' });
  // Assert
  assert.equal(checks.find(check => check.id === 'target_main_stack_margin').status, 'passed');
});

test('corrupted or mistyped target margin cannot qualify', () => {
  // Arrange
  const events = goodEvents();
  events.push({ event: 'scenario_margin', minimum_main_stack_free_bytes: '2048', required_margin_bytes: 2048, configured_main_stack_bytes: 16384 });
  // Act
  const checks = judgeTargetEvents(events, guestManifest, { scenario: 'healthy-lifecycle' });
  // Assert
  assert.equal(checks.find(check => check.id === 'target_main_stack_margin').status, 'failed');
});

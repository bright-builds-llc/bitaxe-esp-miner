import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, chmod, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { parseNoiseDebuggerAbi, noiseDebuggerScript, noiseObjectAbi } from './noise-debugger.mjs';

const ABI = { TCB_BYTES: 340, STACK_OFFSET: 48, END_OFFSET: 72, TOP_OFFSET: 0, NAME_OFFSET: 52,
  PANIC_BYTES: 36, FRAME_OFFSET: 28, EXCEPTION_BYTES: 112, JOURNAL_BYTES: 768, COUNT_BYTES: 4, CURRENT_BYTES: 8,
  REACHED_BYTES: 4, RELEASED_BYTES: 4 };
const stdout = Object.entries(ABI).map(([name, value]) => `ABI_${name} ${value}`).join('\n');

test('exact native SDK ABI is required before any remote attachment', () => {
  // Arrange / Act
  const actual = parseNoiseDebuggerAbi(stdout);
  // Assert
  assert.equal(actual.tcb_bytes, 340);
  assert.equal(actual.frame_offset, 28);
  assert.throws(() => parseNoiseDebuggerAbi(stdout.replace('ABI_TCB_BYTES 340', 'ABI_TCB_BYTES 344')), /abi/);
});

test('GDB script reads two bounded snapshots without inferior heap calls', async () => {
  // Arrange / Act
  const script = await noiseDebuggerScript({ root: '/private/synthetic', port: 23456, selectedStop: 106 });
  // Assert
  assert.match(script, /target remote 127\.0\.0\.1:23456/);
  assert.match(script, /break esp_panic_handler/);
  assert.match(script, /break bitaxe_virtual_noise_prefix_cutoff/);
  assert.match(script, /PREFIX_REACHED == 106 && \*\(unsigned\*\)&BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED == 0/);
  assert.match(script, /PREFIX_RELEASED == 1/);
  assert.match(script, /\+768/);
  assert.match(script, /\$snapshots >= 2/);
  assert.doesNotMatch(script, /^\s*call /m);
  assert.doesNotMatch(script, /@(?:ROOT|PORT|INDEX|CAPTURE)/);
});

function objectElf(journalBytes = 768) {
  const names = ['BITAXE_VIRTUAL_NOISE_CHECKPOINTS', 'BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT',
    'BITAXE_VIRTUAL_NOISE_PREFIX_REACHED', 'BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED'];
  const strings = Buffer.from(`\0${names.join('\0')}\0`), elf = Buffer.alloc(256 + strings.length);
  elf.writeUInt32BE(0x7f454c46); elf[4] = 1; elf[5] = 1; elf.writeUInt16LE(94, 18);
  elf.writeUInt32LE(52, 32); elf.writeUInt16LE(40, 46); elf.writeUInt16LE(3, 48);
  elf.writeUInt32LE(2, 96); elf.writeUInt32LE(176, 108); elf.writeUInt32LE(64, 112);
  elf.writeUInt32LE(2, 116); elf.writeUInt32LE(16, 128);
  elf.writeUInt32LE(3, 136); elf.writeUInt32LE(256, 148); elf.writeUInt32LE(strings.length, 152);
  strings.copy(elf, 256);
  let offset = 1;
  names.forEach((name, index) => {
    const start = 176 + index * 16;
    elf.writeUInt32LE(offset, start); elf.writeUInt32LE(0x3fc90000 + index * 1024, start + 4);
    elf.writeUInt32LE(index === 0 ? journalBytes : 4, start + 8); elf[start + 12] = 0x11;
    elf.writeUInt16LE(1, start + 14); offset += name.length + 1;
  });
  return elf;
}

test('Rust object ABI uses exact symbol sizes and rejects a changed journal', () => {
  // Arrange
  const valid = objectElf(), changed = objectElf(764);
  // Act
  const observed = noiseObjectAbi(valid);
  // Assert
  assert.match(observed, /^ABI_JOURNAL_BYTES 768/m);
  assert.throws(() => noiseObjectAbi(changed), /noise_debugger_object_abi/);
  assert.throws(() => noiseObjectAbi(valid.subarray(0, 60)), /native_elf_symbols_bounds/);
});

test('a command-bearing debugger path is rejected', async () => {
  // Arrange / Act / Assert
  await assert.rejects(noiseDebuggerScript({ root: '/private/invalid\ncontinue', port: 23456, selectedStop: 106 }), /arguments/);
});

async function decodeFixture(stage, withMemory) {
  const root = await mkdtemp(join(tmpdir(), 'noise-debugger-unit-')); await chmod(root, 0o700);
  const abi = Object.fromEntries(Object.entries(ABI).map(([name, value]) => [name.toLowerCase(), value]));
  await writeFile(join(root, 'gdb-preflight.json'), JSON.stringify({ abi, gdb_version: '16.3_20250913' }), { mode: 0o600 });
  await writeFile(join(root, 'stdout'), withMemory ? 'SNAPSHOT_0_KIND 3\nSNAPSHOT_0_PC 1073741824\nSNAPSHOT_0_SP 1070121216\nSNAPSHOT_0_ARG 101\n' : '', { mode: 0o600 });
  if (withMemory) {
    const raw = Buffer.alloc(768), words = [0x564e5031, 101, stage, stage === 2 ? 1 : 0, 0,
      0x3fc90000, 16384, 0x3fc92000, 10000, 5000, 4000, 1];
    words.forEach((word, index) => raw.writeUInt32LE(word, index * 4));
    await writeFile(join(root, 'snapshot0-journal.raw'), raw, { mode: 0o600 });
    await writeFile(join(root, 'snapshot0-count.raw'), Buffer.from([1, 0, 0, 0]), { mode: 0o600 });
  }
  const output = join(root, 'result.json');
  execFileSync('python3', ['scripts/virtual-emulator/noise-debugger-decode.py', root, join(root, 'stdout'), '101', output],
    { timeout: 25000, maxBuffer: 2 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  return JSON.parse(await readFile(output, 'utf8'));
}

test('missing snapshots remain unknown and never become safety success', async () => {
  // Arrange / Act
  const actual = await decodeFixture(0, false);
  // Assert
  assert.equal(actual.snapshot_memory_available, false);
  assert.equal(actual.unknown_reason, 'no_snapshot_captured');
  assert.equal(actual.full_noise_qualified, false);
});

test('incomplete heap checks retain stack facts but heap integrity is unknown', async () => {
  // Arrange / Act
  const actual = await decodeFixture(1, true);
  // Assert
  assert.equal(actual.snapshots[0].records[0].stack_low_water_bytes, 4000);
  assert.equal(actual.snapshots[0].records[0].heap_integrity, null);
  assert.equal(actual.snapshots[0].records[0].internal_free_bytes, null);
});

test('completed synthetic snapshot is decoded without exporting raw pointers', async () => {
  // Arrange / Act
  const actual = await decodeFixture(2, true);
  // Assert
  assert.equal(actual.snapshots[0].records[0].heap_integrity, true);
  assert.equal(actual.snapshots[0].task_bounds_available, false);
  assert.equal(actual.hardware_qualified, false);
  assert.doesNotMatch(JSON.stringify(actual), /0x3fc9|1070121216/);
});

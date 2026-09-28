import { createHash } from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest();
const address = 0x3fc90000;
const words = values => { const b = Buffer.alloc(values.length * 4); values.forEach((v, i) => b.writeUInt32LE(v, i * 4)); return b; };
function header(core, phnum, shoff = 0, shnum = 0) {
  const b = Buffer.alloc(52); Buffer.from([0x7f, 69, 76, 70, 1, 1, 1]).copy(b);
  b.writeUInt16LE(core ? 4 : 2, 16); b.writeUInt16LE(94, 18); b.writeUInt32LE(1, 20);
  b.writeUInt32LE(52, 28); b.writeUInt32LE(shoff, 32); b.writeUInt16LE(52, 40);
  b.writeUInt16LE(32, 42); b.writeUInt16LE(phnum, 44); b.writeUInt16LE(40, 46);
  b.writeUInt16LE(shnum, 48); b.writeUInt16LE(shnum ? 2 : 0, 50); return b;
}
function program(kind) {
  const strings = Buffer.from(`\0${kind === 'missing-symbol' ? 'ABSENT_CUTOFF_RECEIPT' : 'BITAXE_PANIC_CUTOFF_RECEIPT'}\0`);
  const symbol = Buffer.alloc(32); symbol.writeUInt32LE(1, 16); symbol.writeUInt32LE(address, 20);
  symbol.writeUInt32LE(28, 24); symbol[28] = 0x11; symbol.writeUInt16LE(1, 30);
  // This tempting program-only receipt must never substitute for captured PT_LOAD bytes.
  const initialized = kind === 'program-receipt-only' ? words([0x50434f32, 0x402, 0x402, 0x400, 7, (7 << 3) | 4, 0x53544631]) : Buffer.alloc(0);
  const stringOffset = 84 + initialized.length, symbolOffset = stringOffset + strings.length, shoff = symbolOffset + symbol.length;
  // Deliberately no initialized receipt. Program NOBITS cannot provide captured memory.
  return Buffer.concat([header(false, 1, shoff, 4), words([1, 84, address, address, initialized.length, 28, 6, 4]), initialized, strings, symbol,
    Buffer.alloc(40), words([0, initialized.length ? 1 : 8, 3, address, 84, 28, 0, 0, 4, 0]),
    words([0, 3, 0, 0, stringOffset, strings.length, 0, 0, 1, 0]),
    words([0, 2, 0, 0, symbolOffset, symbol.length, 2, 1, 4, 16])]);
}
/** Entirely synthetic native ELF32/Xtensa symbols, IDF identity note and captured DRAM. */
export function cutoffFixture(kind = 'valid') {
  const executable = program(kind), identity = hash(executable).toString('hex'), version = (9 << 16) | 0x103;
  const name = Buffer.from('ESP_CORE_DUMP_INFO\0');
  const descriptor = Buffer.concat([words([version]), Buffer.from(identity), Buffer.alloc(4)]);
  const note = Buffer.concat([words([name.length, descriptor.length, 8266]), name, Buffer.alloc((4 - name.length % 4) % 4), descriptor]);
  const values = [0x50434f32, 0x402, 0x402, 0x400, 7, (7 << 3) | 4, kind === 'ordinary' ? 0 : 0x53544631];
  if (kind === 'bad-magic') values[0] = 0;
  if (kind === 'unsafe') values[3] = 0x402;
  if (kind === 'unconfigured') values[1] = 0;
  if (kind === 'input-mode') values[2] = 0;
  if (kind === 'unrevoked') values[5] = (7 << 3) | 2;
  if (kind === 'generation-mismatch') values[4] = 8;
  if (kind === 'marker-invalid') values[6] = 1;
  const receipt = words(values), missing = ['missing-memory', 'program-receipt-only'].includes(kind), duplicate = kind === 'overlap';
  const phnum = missing ? 1 : duplicate ? 3 : 2, start = 52 + 32 * phnum;
  const data = kind === 'truncated' ? receipt.subarray(0, 24) : receipt;
  const loads = missing ? [] : [words([1, start + note.length, address, address, data.length, receipt.length, 6, 4])];
  if (duplicate) loads.push(loads[0]);
  const core = Buffer.concat([header(true, phnum), words([4, start, 0, 0, note.length, note.length, 0, 4]), ...loads, note, ...(missing ? [] : [data])]);
  const raw = Buffer.concat([words([24 + core.length + 32, version, 0, 0, 0, 0]), core]);
  const dump = Buffer.concat([raw, hash(raw)]);
  if (kind === 'checksum') dump[dump.length - 1] ^= 1;
  return { program: executable, identity, dump };
}

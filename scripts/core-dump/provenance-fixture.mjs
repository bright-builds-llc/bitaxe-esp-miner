import { createHash } from 'node:crypto';
const hash = b => createHash('sha256').update(b).digest();
const words = values => { const b = Buffer.alloc(values.length * 4); values.forEach((v, i) => b.writeUInt32LE(v >>> 0, i * 4)); return b; };
const fnv = values => { let n = 0x811c9dc5; for (const b of words(values)) n = Math.imul(n ^ b, 0x01000193) >>> 0; return n; };
const rolling = values => values.reduce((n, v) => (((n << 5) | (n >>> 27)) ^ v) >>> 0, 0x6d5a56a9);
function header(core, count, shoff = 0, shnum = 0) {
  const b = Buffer.alloc(52); Buffer.from([127, 69, 76, 70, 1, 1, 1]).copy(b); b.writeUInt16LE(core ? 4 : 2, 16);
  b.writeUInt16LE(94, 18); b.writeUInt32LE(1, 20); b.writeUInt32LE(52, 28); b.writeUInt32LE(shoff, 32);
  b.writeUInt16LE(52, 40); b.writeUInt16LE(32, 42); b.writeUInt16LE(count, 44); b.writeUInt16LE(40, 46); b.writeUInt16LE(shnum, 48); b.writeUInt16LE(shnum ? 2 : 0, 50); return b;
}
function note(name, type, desc) {
  const n = Buffer.from(`${name}\0`); return Buffer.concat([words([n.length, desc.length, type]), n, Buffer.alloc((4 - n.length % 4) % 4), desc, Buffer.alloc((4 - desc.length % 4) % 4)]);
}
/** Synthetic SDK ELF_SHA256_V2_1 core with exact Xtensa588byte NT_PRSTATUS. No device memory. */
export function provenanceFixture(kind = 'real') {
  const address = 0x3fc90000, pc = 0x42000000, tcb = 0x3fca0000, frame = kind === 'last-span' ? 0x3fd00000 - 112 : 0x3fca1000;
  const source = [0x12345678, 0x90abcdef], panic = Array(48).fill(0);
  Object.assign(panic, { 0: 0x50465231, 1: 1, 2: 48, 3: 31, 4: source[0], 5: source[1], 6: 17, 8: 0,
    15: frame, 18: pc, 21: frame + 112, 24: tcb, 25: frame, 26: frame + 512, 27: frame, 28: frame + 512,
    29: 1, 32: 1, 33: 1, 36: 0x3fc88000, 37: 0x3fd00000, 40: 65536, 41: 36, 42: 112, 43: 1, 44: 1, 45: 1 });
  if (kind === 'fake') Object.assign(panic, { 26: frame - 16, 27: 0x20000000, 28: 0x20000070, 30: 12, 31: 1 });
  if (kind === 'last-span') Object.assign(panic, { 21: 0x3fd00000 - 16, 26: 0x3fd00000, 27: 0x20000000, 28: 0x20000070, 30: 2, 31: 1, 45: 0 });
  if (kind === 'bad-pointer') panic[15] = 0;
  if (kind === 'bad-pc') panic[18] = 0x20000000;
  if (kind === 'wrong-source') panic[4]++;
  if (kind === 'wrong-boot') panic[6]++;
  panic[46] = fnv(panic.slice(1, 46)); panic[47] = ~panic[46] >>> 0;
  if (kind === 'torn') panic[47]++;
  const allocations = Array(384).fill(0);
  allocations.splice(0, 16, 0x42414c48, 1, 16, 20, 184, 2, 8, ...source, 17, 0, 0, 0xa110ca7e, 0, 0, 0);
  allocations[15] = rolling(allocations.slice(0, 11));
  if (kind.startsWith('allocation-')) {
    const count = 10; allocations[16] = count; allocations[18] = count - 8;
    for (let index = 0; index < 9; index++) {
      const sequence = index === 0 ? 1 : count - ((count - index) % 8);
      const record = [sequence, 0x414c4c31, sequence, 0, ...source, 17, 0, 1, 8192, 0x804, tcb, 42, pc, 1, 5, 2, pc, 1, 0];
      if (kind === 'allocation-early' && index === 0) { record[6] = 0; record[18] = 3; }
      record[19] = rolling(record.slice(1, 19));
      if (kind === 'allocation-torn' && index === 4) record[0] = 0;
      if (kind === 'allocation-checksum' && index === 4) record[19]++;
      allocations.splice(20 + index * 20, 20, ...record);
    }
  }
  const cutoff = [0x50434f32, 0x402, 0x402, 0x400, 7, 60, 0x53544631];
  const abi = [0x50464131, 1, 48, 36, 112, 12, 0, 4, 8, 12, 24, 28, 32, 0, 4, 8, 12, 16, 80, 84, 0, 4, 8, 0x3fc88000, 0x3fd00000, 0, 0, 0, 65536, 112, 0, 0x50465231];
  const blocks = [words(panic), words(allocations), words(cutoff), words(source), words(abi)];
  const names = ['BITAXE_PANIC_FRAME_RECORD', 'BITAXE_ALLOCATION_HISTORY', 'BITAXE_PANIC_CUTOFF_RECEIPT', 'BITAXE_FAULT_COMPILED_SOURCE', 'BITAXE_FAULT_PROVENANCE_ABI'];
  let strings = Buffer.from([0]); const symbols = [Buffer.alloc(16)]; let offset = 0;
  for (let i = 0; i < names.length; i++) {
    const sym = Buffer.alloc(16); sym.writeUInt32LE(strings.length); sym.writeUInt32LE(address + offset, 4); sym.writeUInt32LE(blocks[i].length, 8); sym[12] = 0x11; sym.writeUInt16LE(1, 14);
    symbols.push(sym); strings = Buffer.concat([strings, Buffer.from(names[i] + '\0')]); offset += blocks[i].length;
  }
  const data = Buffer.concat(blocks), code = Buffer.alloc(16), start = 116, stringOffset = start + data.length + code.length, symbolOffset = stringOffset + strings.length;
  const symbolData = Buffer.concat(symbols), shoff = symbolOffset + symbolData.length;
  const program = Buffer.concat([header(false, 2, shoff, 4), words([1, start, address, address, data.length, data.length, 6, 4]),
    words([1, start + data.length, pc, pc, code.length, code.length, 5, 4]), data, code, strings, symbolData, Buffer.alloc(40),
    words([0, 1, 3, address, start, data.length, 0, 0, 4, 0]), words([0, 3, 0, 0, stringOffset, strings.length, 0, 0, 1, 0]),
    words([0, 2, 0, 0, symbolOffset, symbolData.length, 2, 1, 4, 16])]);
  const identity = hash(program).toString('hex'), version = (9 << 16) | 0x103;
  const regs = Buffer.alloc(588); regs.writeUInt32LE(kind === 'bad-note' ? tcb + 4 : tcb, 24); regs.writeUInt32LE(panic[31] ? 0x20000000 : pc, 72);
  const notes = Buffer.concat([note('ESP_CORE_DUMP_INFO', 8266, Buffer.concat([words([version]), Buffer.from(identity), Buffer.alloc(4)])),
    note('CORE', 1, regs), note('EXTRA_INFO', 677, words([tcb]))]);
  const captured = kind === 'missing-memory' ? Buffer.alloc(0) : data;
  const stack = Buffer.alloc(panic[28] - panic[27]); stack.writeUInt32LE(panic[31] ? 0x20000000 : pc, 4); const phnum = kind === 'overlap' ? 4 : 3, coreStart = 52 + phnum * 32;
  const load = words([1, coreStart + notes.length, address, address, captured.length, data.length, 6, 4]);
  const core = Buffer.concat([header(true, phnum), words([4, coreStart, 0, 0, notes.length, notes.length, 0, 4]), load,
    words([1, coreStart + notes.length + captured.length, panic[27], panic[27], stack.length, stack.length, 6, 4]),
    ...(kind === 'overlap' ? [load] : []), notes, captured, stack]);
  const raw = Buffer.concat([words([24 + core.length + 32, version, 0, 0, 0, 0]), core]);
  return { program, identity, dump: Buffer.concat([raw, hash(raw)]) };
}

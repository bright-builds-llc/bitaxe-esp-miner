import { createHash } from 'node:crypto';
const check = (ok, code) => { if (!ok) throw Error(code); };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
/** One immutable SDK IRQ primitive, not a general ROM exception. */
export function verifyIrqRomLeaf(appEntries, romEntries, functions, romBytes, selector) {
  check(romBytes.length >= 84 && romBytes.subarray(0, 7).equals(Buffer.from([127, 69, 76, 70, 1, 1, 1])) && romBytes.readUInt16LE(18) === 94, 'fault_audit_rom_elf');
  check(Array.isArray(selector.esp32s3) && selector.esp32s3.length === 1 && selector.esp32s3[0].rev === 0, 'fault_audit_rom_selector');
  const find = (rows, name = '_xtos_set_intlevel') => { const matches = rows.filter(row => row.name === name); check(matches.length === 1, 'fault_audit_rom_symbol'); return matches[0]; };
  const app = find(appEntries), rom = find(romEntries), veneer = find(romEntries, '__call__xtos_set_intlevel'), fn = functions.get(rom.address);
  const thunk = functions.get(veneer.address);
  check(app.address === veneer.address && rom.size === 25 && fn && thunk, 'fault_audit_rom_address');
  const expected = [['entry', 'a1,16'], ['extui', 'a3,a2,0,4'], ['rsr.ps', 'a2'], ['movi.n', 'a4,-16'], ['and', 'a4,a4,a2'],
    ['or', 'a4,a4,a3'], ['wsr.ps', 'a4'], ['rsync', ''], ['retw.n', '']];
  check(fn.instructions.length === expected.length && fn.instructions.every((row, i) => row.op === expected[i][0] && row.args.replace(/\s+/gu, '') === expected[i][1]), 'fault_audit_rom_instructions');
  check(thunk.instructions.length === 2 && thunk.instructions[0].op === 'l32r' && thunk.instructions[0].args.startsWith('a9,') &&
    thunk.instructions[1].op === 'jx' && thunk.instructions[1].args === 'a9', 'fault_audit_rom_veneer');
  const literal = /,\s*([a-f0-9]+)/u.exec(thunk.instructions[0].args); check(literal, 'fault_audit_rom_veneer');
  const literalAddress = parseInt(literal[1], 16);
  const base = romBytes.readUInt32LE(28), count = romBytes.readUInt16LE(44), width = romBytes.readUInt16LE(42); const mappings = [];
  check(width === 32 && count < 256 && base + width * count <= romBytes.length, 'fault_audit_rom_elf');
  for (let i = 0; i < count; i++) {
    const at = base + i * width, address = romBytes.readUInt32LE(at + 8), size = romBytes.readUInt32LE(at + 16), file = romBytes.readUInt32LE(at + 4);
    if (romBytes.readUInt32LE(at) === 1 && address <= rom.address && address + size >= rom.address + rom.size) mappings.push(romBytes.subarray(file + rom.address - address, file + rom.address - address + rom.size));
  }
  let veneerTarget;
  for (let i = 0; i < count; i++) {
    const at = base + i * width, address = romBytes.readUInt32LE(at + 8), size = romBytes.readUInt32LE(at + 16), file = romBytes.readUInt32LE(at + 4);
    if (romBytes.readUInt32LE(at) === 1 && address <= literalAddress && address + size >= literalAddress + 4) {
      check(veneerTarget === undefined, 'fault_audit_rom_mapping'); veneerTarget = romBytes.readUInt32LE(file + literalAddress - address);
    }
  }
  check(veneerTarget === rom.address, 'fault_audit_rom_veneer');
  check(mappings.length === 1 && mappings[0].length === rom.size, 'fault_audit_rom_mapping');
  return { address: app.address, stackBytes: 16, symbol: '_xtos_set_intlevel', code_sha256: hash(mappings[0]), rom_elf_sha256: hash(romBytes),
    veneer_verified: true, chip: 'esp32s3', sdk_rom_selector_revision: 0, sdk_rom_selector_verified: true };
}

/** Read bounded ELF32 Xtensa symbol facts; no execution or device access. */
export function nativeElfSymbols(elf) {
  if (!Buffer.isBuffer(elf) || elf.length < 52 || elf.readUInt32BE(0) !== 0x7f454c46 ||
      elf[4] !== 1 || elf[5] !== 1 || elf.readUInt16LE(18) !== 94) throw Error('native_elf_symbols_format');
  const bounded = (offset, bytes) => {
    if (!Number.isSafeInteger(offset) || offset < 0 || offset + bytes > elf.length) throw Error('native_elf_symbols_bounds');
    return elf.subarray(offset, offset + bytes);
  };
  const sections = [], offset = elf.readUInt32LE(32), count = elf.readUInt16LE(48);
  if (elf.readUInt16LE(46) !== 40 || count > 4096) throw Error('native_elf_symbols_bounds');
  for (let index = 0; index < count; index++) sections.push(bounded(offset + index * 40, 40));
  const result = [];
  for (const section of sections.filter(value => value.readUInt32LE(4) === 2)) {
    if (section.readUInt32LE(36) !== 16) throw Error('native_elf_symbols_bounds');
    const linked = sections[section.readUInt32LE(24)];
    if (!linked) throw Error('native_elf_symbols_bounds');
    const strings = bounded(linked.readUInt32LE(16), linked.readUInt32LE(20));
    const symbols = bounded(section.readUInt32LE(16), section.readUInt32LE(20));
    for (let index = 0; index + 16 <= symbols.length; index += 16) {
      const nameOffset = symbols.readUInt32LE(index), end = strings.indexOf(0, nameOffset);
      if (end < nameOffset) throw Error('native_elf_symbols_bounds');
      const name = strings.toString('utf8', nameOffset, end);
      result.push({ name, address: symbols.readUInt32LE(index + 4), bytes: symbols.readUInt32LE(index + 8),
        type: symbols[index + 12] & 15, defined: symbols.readUInt16LE(index + 14) !== 0 });
    }
  }
  return result;
}

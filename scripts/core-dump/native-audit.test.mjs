import assert from 'node:assert/strict';
import test from 'node:test';
import { audit } from './native-audit.mjs';

function fixture() {
  const elf = Buffer.alloc(0x3000); Buffer.from([127, 69, 76, 70, 1, 1, 1]).copy(elf);
  elf.writeUInt16LE(94, 18); elf.writeUInt32LE(52, 28); elf.writeUInt16LE(32, 42); elf.writeUInt16LE(2, 44);
  for (const [index, address] of [0x40370000, 0x42000000].entries()) {
    const ph = 52 + index * 32; elf.writeUInt32LE(1, ph); elf.writeUInt32LE(0x1000 + index * 0x1000, ph + 4);
    elf.writeUInt32LE(address, ph + 8); elf.writeUInt32LE(0x1000, ph + 16);
  }
  [0x60004008, 0x6000400c, 0x3fc82000, 0x3fc81000, 0x50434f32, 0x42000100].forEach((value, i) => elf.writeUInt32LE(value, 0x1100 + i * 4));
  elf.writeUInt32LE(0x40370200, 0x2100);
  elf.writeUInt32LE(0x3fc81000, 0x2208); elf.writeUInt32LE(0x3fc8101c, 0x220c);
  const symbols = [
    '40370200 00000200 T __wrap_esp_panic_handler', '42000100 00000100 T esp_panic_handler',
    '3fc81000 A _coredump_dram_start', '3fc8101c A _coredump_dram_end', '42000200 00000010 r s_memory_sections',
    '3fc81000 0000001c D BITAXE_PANIC_CUTOFF_RECEIPT', '3fc82000 00000084 D revocation6global4GATE',
    '3fc83000 00000004 d panic_cutoff18CONFIGURED_OUTPUTS', '3fc83004 00000004 d panic_cutoff16SELF_TEST_MARKER',
  ].join('\n');
  const code = ['entry a1, 32', 'l32r a8, 40370100', 'movi a9, 0x400', 'memw', 's32i.n a9, a8, 0',
    'l32r a8, 40370104', 'movi.n a9, 2', 'memw', 's32i.n a9, a8, 0',
    'l32r a8, 40370108', 'l32i.n a9, a8, 0', 'movi a10, -8', 'and a9, a9, a10', 'movi a10, 4',
    'or a9, a9, a10', 's32i.n a9, a8, 0', 'l32r a10, 4037010c', 'l32r a9, 40370110', 's32i.n a9, a10, 0',
    'l32r a8, 40370114', 'callx8 a8', 'retw.n'];
  const wrapper = code.map((line, i) => `${(0x40370200 + i * 3).toString(16)}: 000000 ${line}`).join('\n');
  const port = '42000200: 000000 l32r a8, 42000100\n42000203: 000000 callx8 a8';
  return { elf, symbols, wrapper, port, run() { return audit(this.elf, this.symbols, this.wrapper, this.port); } };
}
test('native audit binds actual literal words, latch order and revocation expression', () => {
  const f = fixture(); const result = f.run();
  assert.equal(result.schema, 'str005-native-panic-cutoff-audit-v2'); assert.equal(result.receipt_user_region, true);
  assert.equal(result.safe_latches_before_delegate, true); assert.equal(result.hardware_verified, false);
});
test('native audit rejects reversed enable polarity', () => {
  const f = fixture(); f.wrapper = f.wrapper.replace('movi a9, 0x400', 'movi a9, 0');
  assert.throws(() => f.run(), /native_safe_latch_order/);
});
test('native audit rejects conditional or call paths before cutoff', () => {
  for (const op of ['beqz a9, 40370300', 'callx8 a8']) {
    const f = fixture(); f.wrapper = f.wrapper.replace('memw', op); assert.throws(() => f.run());
  }
});
test('native audit rejects flash-resident emergency data or code', () => {
  for (const old of ['3fc83000', '40370200']) {
    const f = fixture(); f.symbols = f.symbols.replace(old, '42000300'); assert.throws(() => f.run());
  }
});
test('native audit rejects missing generation revocation', () => {
  const f = fixture(); f.wrapper = f.wrapper.replace('movi a10, 4', 'movi a10, 1');
  assert.throws(() => f.run(), /native_generation_revoke/);
});
test('native audit rejects bypassed or missing wrapper route', () => {
  const f = fixture(); f.elf.writeUInt32LE(0x42000100, 0x2100);
  assert.throws(() => f.run(), /native_port_bypass/);
  f.port = ''; assert.throws(() => f.run(), /native_port_routing/);
});

test('receipt must fit wholly in the dedicated user region', () => {
  for (const symbols of [
    ['3fc81000 A _coredump_dram_start', '3fc81004 A _coredump_dram_start'],
    ['3fc8101c A _coredump_dram_end', '3fc81018 A _coredump_dram_end'],
    ['3fc8101c A _coredump_dram_end', '3fc81000 A _coredump_dram_end'],
  ]) {
    const f = fixture(); f.symbols = f.symbols.replace(...symbols);
    assert.throws(() => f.run(), /native_receipt_user_region/);
  }
});
test('SDK memory-region table must actually reference the receipt section', () => {
  const f = fixture(); f.elf.writeUInt32LE(0x3fc81020, 0x2208);
  assert.throws(() => f.run(), /native_sdk_user_regions/);
});

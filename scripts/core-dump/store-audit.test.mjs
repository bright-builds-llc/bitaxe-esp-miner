import assert from 'node:assert/strict';
import test from 'node:test';
import { auditStore, instructions, STORE_SEAMS, auditBootInitializer, elfView } from './store-audit-model.mjs';
function fixture() {
  const words = new Map(), entries = [{ name: 'BITAXE_CORE_DUMP_RTC', address: 0x50000000, size: 80 },
    { name: 'BITAXE_CORE_DUMP_CURRENT', address: 0x3fc81000, size: 80 }];
  const functions = new Map(), sdkCalls = new Map([['esp_core_dump_write_elf_and_check', []], ['esp_core_dump_store', []]]);
  let literal = 0x40370100;
  const word = value => { const at = literal; literal += 4; words.set(at, value); return at.toString(16); };
  const code = (address, rows) => instructions(rows.map((text, index) => `${(address + index * 3).toString(16)}: 000000 ${text}`).join('\n'));
  const helper = { name: 'bitaxe_core_dump_receipt_update', address: 0x40371000, size: 80 };
  helper.code = code(helper.address, ['entry a1, 32', `l32r a8, ${word(0x50000000)}`, 's32i.n a2, a8, 0', 'retw.n']); functions.set(helper.name, helper);
  for (const [index, name] of STORE_SEAMS.entries()) {
    const real = { name: `esp_core_dump_${name}`, address: 0x42001000 + index * 256, size: 80 };
    const wrapper = { name: `__wrap_esp_core_dump_${name}`, address: 0x40372000 + index * 256, size: 80 };
    wrapper.code = code(wrapper.address, ['entry a1, 32', 'movi a10, 1', 'movi a11, 0', `l32r a8, ${word(helper.address)}`, 'callx8 a8',
      'mov.n a10, a2', 'mov.n a11, a3', `l32r a8, ${word(real.address)}`, 'callx8 a8', 'mov.n a2, a10', 'movi a10, 2', `l32r a8, ${word(helper.address)}`, 'callx8 a8', 'retw.n']);
    functions.set(wrapper.name, wrapper); entries.push(real, wrapper);
    sdkCalls.get(name === 'store' ? 'esp_core_dump_write_elf_and_check' : 'esp_core_dump_store').push({ target: wrapper.address });
  }
  return { words, entries, functions, sdkCalls, elf: { word: at => { assert.ok(words.has(at)); return words.get(at); }, noLoad: () => true },
    run() { return auditStore(this); }, code, word };
}
test('all five routes preserve delegation and bounded own stores without hardware claim', () => {
  const f = fixture(), result = f.run();
  assert.equal(result.sdk_routes_wrapped, true); assert.equal(result.max_added_stack_bytes, 96); assert.equal(result.hardware_verified, false);
});
test('missing or bypassed SDK alias routes fail closed', () => {
  for (const bypass of [false, true]) {
    const f = fixture(); f.sdkCalls.set('esp_core_dump_write_elf_and_check', bypass ? [{ target: 0x42001000 }] : []);
    assert.throws(() => f.run(), /store_audit_sdk_route/);
  }
});
test('unmapped calls, allocation targets and return paths without delegate fail closed', () => {
  for (const mutate of [f => f.words.set(0x40370108, 0x42009999), f => { f.functions.get('__wrap_esp_core_dump_store').code[8].args = 'a7'; },
    f => { f.functions.get('__wrap_esp_core_dump_store').code[8].op = 'nop'; }]) {
    const f = fixture(); mutate(f); assert.throws(() => f.run());
  }
});
test('RTC overflow, GPIO writes, nonretained storage and oversized frames fail closed', () => {
  for (const mutate of [f => f.words.set(0x40370100, 0x50000050), f => f.words.set(0x40370100, 0x60004008),
    f => { f.elf.noLoad = () => false; }, f => { f.functions.get('bitaxe_core_dump_receipt_update').code[0].args = 'a1, 512'; }]) {
    const f = fixture(); mutate(f); assert.throws(() => f.run());
  }
});
test('fixed 20-word write loop is bounded but unknown-counter loop is rejected', () => {
  const f = fixture(), helper = f.functions.get('bitaxe_core_dump_receipt_update');
  helper.code = f.code(helper.address, ['entry a1, 32', `l32r a8, ${f.word(0x50000000)}`, 'movi a9, 20', 's32i.n a2, a8, 0',
    'addi a8, a8, 4', 'addi a9, a9, -1', `bnez a9, ${(helper.address + 9).toString(16)}`, 'retw.n']);
  assert.equal(f.run().bounded_diagnostic_writes, true);
  helper.code[2] = { ...helper.code[2], op: 'mov.n', args: 'a9, a6' };
  assert.throws(() => f.run());
});

test('added-stack budget includes the outer store wrapper retained during inner SDK phases', () => {
  const f = fixture(); f.functions.get('bitaxe_core_dump_receipt_update').code[0].args = 'a1, 224';
  assert.throws(() => f.run(), /store_audit_added_stack/);
});

test('normal initializer must be scheduled for secondary CPU0 and call the real void initializer', () => {
  const entries = [
    { name: 'esp_system_init_fn_init_coredump', address: 100, size: 8 },
    { name: '__esp_system_init_fn_init_coredump', address: 200, size: 10 },
    { name: 'esp_core_dump_init', address: 300, size: 96, binding: 'T' },
    { name: '_esp_system_init_fn_array_start', address: 100 }, { name: '_esp_system_init_fn_array_end', address: 108 },
  ];
  const elf = { word: at => at === 100 ? 200 : 0x10001 };
  assert.equal(auditBootInitializer(elf, entries, [{ target: 300 }]).normal_boot_init_runtime_verified, false);
  assert.throws(() => auditBootInitializer(elf, entries.map(row => row.name === 'esp_core_dump_init' ? { ...row, binding: 'W' } : row), [{ target: 300 }]), /store_audit_boot_route/);
  assert.throws(() => auditBootInitializer(elf, entries, []), /store_audit_boot_route/);
  assert.throws(() => auditBootInitializer({ word: () => 0 }, entries, [{ target: 300 }]), /store_audit_boot_route/);
});

test('IRAM helper cannot dereference a compiler-generated DROM jump table', () => {
  const f = fixture(), helper = f.functions.get('bitaxe_core_dump_receipt_update');
  // Actual rejected candidate sequence: stage10 indexes seven words into DROM at 0x3c254ec4.
  f.words.set(0x4037440c, 0x3c254ec4);
  helper.code = f.code(helper.address, ['entry a1, 32', 'movi a15, 7', 'l32r a10, 4037440c',
    'addx4 a10, a15, a10', 'l32i.n a10, a10, 0', 'jx a10']);
  assert.throws(() => f.run(), /store_audit_read_bound/);
});

test('delegate arguments and signed SDK return values cannot be replaced by wrapper bookkeeping', () => {
  for (const [name, index] of [['__wrap_esp_core_dump_write_prepare', 5], ['__wrap_esp_core_dump_store', 9]]) {
    const f = fixture(); f.functions.get(name).code[index] = { ...f.functions.get(name).code[index], op: 'movi', args: index === 5 ? 'a10, 0' : 'a2, 0' };
    assert.throws(() => f.run(), /store_audit_delegate_(?:arguments|result)/);
  }
});

test('RTC no-load placement must specifically use rtc_noinit rather than reset-cleared rtc_bss', () => {
  const bytes = Buffer.alloc(256); Buffer.from([127,69,76,70,1,1,1]).copy(bytes);
  bytes.writeUInt16LE(94, 18); bytes.writeUInt32LE(52, 28); bytes.writeUInt16LE(32, 42);
  bytes.writeUInt32LE(64, 32); bytes.writeUInt16LE(40, 46); bytes.writeUInt16LE(2, 48); bytes.writeUInt16LE(1, 50);
  const names = Buffer.from('\0.rtc_noinit\0.shstrtab\0'); names.copy(bytes, 160);
  bytes.writeUInt32LE(1, 64); bytes.writeUInt32LE(8, 68); bytes.writeUInt32LE(0x50000000, 76); bytes.writeUInt32LE(80, 84);
  bytes.writeUInt32LE(names.indexOf(Buffer.from('.shstrtab')), 104); bytes.writeUInt32LE(3, 108);
  bytes.writeUInt32LE(160, 120); bytes.writeUInt32LE(names.length, 124);
  assert.equal(elfView(bytes).noLoad({ address: 0x50000000, size: 80 }), true);
  Buffer.from('.rtc.bss\0').copy(bytes, 161);
  assert.equal(elfView(bytes).noLoad({ address: 0x50000000, size: 80 }), false);
});

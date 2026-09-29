import test from 'node:test';
import assert from 'node:assert/strict';
import { auditFaultProvenance, ROOTS } from './fault-provenance-audit.mjs';
function fixture() {
  const entries = [], functions = new Map(), literals = new Map(); let address = 0x40378000;
  const add = (name, size, where = address) => { const value = { name, size, address: where }; entries.push(value); address += 256; return value; };
  add('_coredump_dram_start', 0, 0x3fc90000); add('_coredump_dram_end', 0, 0x3fca0000);
  add('BITAXE_PANIC_FRAME_RECORD', 192, 0x3fc90000); add('BITAXE_ALLOCATION_HISTORY', 1536, 0x3fc90100);
  for (const [index, name] of ['owners', 'legacy_allocation', 'legacy_context', 'legacy_source_lo', 'legacy_source_hi', 'legacy_claim', 'identity', 'snapshot', 'checks', 'capture_active'].entries()) add(name, 4, 0x3fc91000 + index * 4);
  add('BITAXE_FAULT_COMPILED_SOURCE', 8, 0x3c010000); const abi = add('BITAXE_FAULT_PROVENANCE_ABI', 128, 0x3c010100);
  [0x50464131, 1, 48, 36, 112, 12, 0, 4, 8, 12, 24, 28, 32, 0, 4, 8, 12, 16, 80, 84, 0, 4, 8].forEach((x, i) => literals.set(abi.address + i * 4, x));
  literals.set(abi.address + 28 * 4, 65536); literals.set(abi.address + 31 * 4, 0x50465231);
  const delegates = ['esp_core_dump_check_task', 'esp_core_dump_port_set_crashed_tcb'].map(name => add(name, 32));
  const roots = ROOTS.map(name => add(name, 32)); const caller = add('esp_core_dump_get_task_snapshot', 32); const wrapper = add('__wrap_esp_panic_handler', 32);
  for (const value of [...roots, caller, wrapper]) {
    const instructions = [{ address: value.address, op: 'entry', args: 'a1, 32' }];
    const index = ROOTS.indexOf(value.name);
    if (index === 3) instructions.push({ address: value.address + 6, op: 'wsr.scompare1', args: 'a4' }, { address: value.address + 9, op: 's32c1i', args: 'a2, a3, 0' });
    if (index === 1 || index === 2) instructions.push({ address: value.address + 3, op: 'call8', args: `${delegates[index - 1].address.toString(16)} <${delegates[index - 1].name}>` });
    if (value === caller) for (const [i, target] of roots.slice(1, 3).entries()) instructions.push({ address: value.address + 3 + i * 3, op: 'call8', args: `${target.address.toString(16)} <${target.name}>` });
    instructions.push({ address: value.address + 12, op: 'retw.n', args: '' }); functions.set(value.address, { address: value.address, symbol: value.name, instructions });
  }
  return { entries, functions, word: value => { assert.ok(literals.has(value)); return literals.get(value); }, literals, roots };
}
test('provenance closure checks native ABI selected region SDK routes and bounded IRAM helpers', () => {
  const f = fixture(); const result = auditFaultProvenance(f); assert.equal(result.sdk_routes_wrappers, true); assert.equal(result.max_selected_added_stack_bytes, 64);
});
for (const [label, mutate] of [
  ['flash helper', f => { f.functions.get(f.roots[0].address).instructions[0].address = 0x42000000; }],
  ['unresolved indirect call', f => { f.functions.get(f.roots[0].address).instructions.splice(1, 0, { address: f.roots[0].address + 3, op: 'callx8', args: 'a8' }); }],
  ['dynamic branch', f => { f.functions.get(f.roots[0].address).instructions.splice(1, 0, { address: f.roots[0].address + 3, op: 'jx', args: 'a8' }); }],
  ['external owner registry', f => { f.entries.find(x => x.name === 'owners').address = 0x3c000000; }],
  ['changed SDK ABI', f => { f.literals.set(0x3c010100 + 4 * 3, 40); }],
]) test(`native provenance rejects ${label}`, () => { const f = fixture(); mutate(f); assert.throws(() => auditFaultProvenance(f)); });
import './fault-provenance-rom.test.mjs';

test('panic closure includes cutoff caller frame and cannot exceed128bytes', () => {
  const f = fixture(); f.functions.get(f.roots[0].address).instructions[0].args = 'a1, 112';
  assert.throws(() => auditFaultProvenance(f), /fault_audit_panic_stack_budget/u);
});
test('allocation closure cannot conceal nested frame cost above256bytes', () => {
  const f = fixture(), root = f.functions.get(f.roots[3].address), helper = f.roots[4];
  root.instructions[0].args = 'a1, 240'; root.instructions.splice(1, 0, { address: root.address + 3, op: 'call8', args: `${helper.address.toString(16)} <${helper.name}>` });
  assert.throws(() => auditFaultProvenance(f), /fault_audit_allocation_stack_budget/u);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { verifyIrqRomLeaf } from './fault-provenance-rom.mjs';
function fixture() {
  const base = 0x40000000, veneer = base + 16, implementation = base + 32, bytes = Buffer.alloc(148);
  Buffer.from([127, 69, 76, 70, 1, 1, 1]).copy(bytes); bytes.writeUInt16LE(94, 18); bytes.writeUInt32LE(52, 28); bytes.writeUInt16LE(32, 42); bytes.writeUInt16LE(1, 44);
  [1, 84, base, base, 64, 64, 5, 4].forEach((value, i) => bytes.writeUInt32LE(value, 52 + i * 4)); bytes.writeUInt32LE(implementation, 84 + 12);
  const ops = [['entry', 'a1, 16'], ['extui', 'a3, a2, 0, 4'], ['rsr.ps', 'a2'], ['movi.n', 'a4, -16'], ['and', 'a4, a4, a2'],
    ['or', 'a4, a4, a3'], ['wsr.ps', 'a4'], ['rsync', ''], ['retw.n', '']];
  const functions = new Map([[implementation, { instructions: ops.map(([op, args]) => ({ op, args })) }],
    [veneer, { instructions: [{ op: 'l32r', args: `a9, ${(base + 12).toString(16)} <literal>` }, { op: 'jx', args: 'a9' }] }]]);
  return { app: [{ name: '_xtos_set_intlevel', address: veneer }], rom: [{ name: '_xtos_set_intlevel', address: implementation, size: 25 },
    { name: '__call__xtos_set_intlevel', address: veneer }], functions, bytes, selector: { esp32s3: [{ rev: 0 }] }, veneer, implementation };
}
test('only the exact SDK-selected ROM IRQ veneer and immutable leaf are admitted', () => {
  const f = fixture(), result = verifyIrqRomLeaf(f.app, f.rom, f.functions, f.bytes, f.selector);
  assert.equal(result.stackBytes, 16); assert.equal(result.veneer_verified, true); assert.match(result.code_sha256, /^[a-f0-9]{64}$/u);
});
for (const [label, change] of [
  ['wrong application address', f => { f.app[0].address++; }], ['changed veneer literal', f => { f.bytes.writeUInt32LE(0, 96); }],
  ['wrong silicon selector', f => { f.selector.esp32s3[0].rev = 1; }],
  ['unexpected memory load', f => { f.functions.get(f.implementation).instructions[2].op = 'l32i'; }],
  ['wrong processor status operand', f => { f.functions.get(f.implementation).instructions[6].args = 'a5'; }],
]) test(`ROM exception rejects ${label}`, () => { const f = fixture(); change(f); assert.throws(() => verifyIrqRomLeaf(f.app, f.rom, f.functions, f.bytes, f.selector)); });

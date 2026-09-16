import test from 'node:test';
import assert from 'node:assert/strict';
import { noiseNativeCalls, noiseNativeInstructions } from './noise-native-calls.mjs';

const fn = rows => ({ address: 0x100, symbol: 'test', instructions: rows.map(([address, op, args = '']) => ({ address, op, args })) });
const spill = [
  [0x100, 'entry', 'a1, 64'], [0x103, 'l32r', 'a8, 80 <literal> (200 <callee>)'],
  [0x106, 's32i.n', 'a8, a1, 16'], [0x108, 'movi.n', 'a8, 0'],
];
const finish = [[0x120, 'l32i.n', 'a8, a1, 16'], [0x122, 'callx8', 'a8'], [0x125, 'retw.n']];

test('compiler local function spill resolves the reloaded target', () => {
  assert.deepEqual(noiseNativeCalls(fn([...spill, ...finish])), [{ target: 0x200, call_address: 0x122 }]);
});
test('overlapping byte write invalidates the complete spilled target', () => {
  assert.deepEqual(noiseNativeCalls(fn([...spill, [0x110, 's8i', 'a2, a1, 17'], ...finish])), []);
});
test('unrelated stack store preserves the private spill', () => {
  assert.equal(noiseNativeCalls(fn([...spill, [0x110, 's32i.n', 'a2, a1, 24'], ...finish]))[0].target, 0x200);
});
test('unknown store address cannot preserve a possible aliased spill', () => {
  assert.deepEqual(noiseNativeCalls(fn([...spill, [0x110, 's32i.n', 'a2, a3, 0'], ...finish])), []);
});
test('conditional atomic store invalidates a spilled function address', () => {
  const code = [...spill, [0x110, 'l32r', 'a2, 84 <literal> (300 <other>)'],
    [0x113, 's32c1i', 'a2, a1, 16'], ...finish];
  assert.deepEqual(noiseNativeCalls(fn(code)), []);
});
test('passing the spill region to a callee invalidates its retained constant', () => {
  assert.deepEqual(noiseNativeCalls(fn([...spill, [0x110, 'addi', 'a10, a1, 16'], [0x113, 'callx8', 'a9'], ...finish])), []);
});
test('passing a later buffer leaves an earlier private spill intact', () => {
  assert.equal(noiseNativeCalls(fn([...spill, [0x110, 'addi', 'a10, a1, 24'], [0x113, 'callx8', 'a9'], ...finish]))[0].target, 0x200);
});
test('merging a nullable stack pointer preserves its possible mutation of a spill', () => {
  const code = [...spill, [0x10a, 'movi.n', 'a10, 0'], [0x10c, 'bnez.n', 'a2, 112 <test+0x12>'],
    [0x10e, 'addi', 'a10, a1, 16'], [0x112, 'call8', '400 <mutate_if_nonnull>'], ...finish];
  assert.deepEqual(noiseNativeCalls(fn(code)), [{ target: 0x400, call_address: 0x112 }]);
});
test('narrow branch joins retain both possible spilled callees', () => {
  const code = [...spill, [0x110, 'bnez.n', 'a2, 120 <test+0x20>'],
    [0x112, 'l32r', 'a8, 84 <literal> (300 <other>)'], [0x115, 's32i.n', 'a8, a1, 16'], ...finish];
  assert.deepEqual(noiseNativeCalls(fn(code)), [{ target: 0x200, call_address: 0x122 }, { target: 0x300, call_address: 0x122 }]);
});
test('a nullable stack pointer remains tainted through a spill and reload', () => {
  const code = [...spill, [0x10a, 'movi.n', 'a10, 0'], [0x10c, 'bnez.n', 'a2, 113 <test+0x13>'],
    [0x10e, 'addi', 'a10, a1, 16'], [0x111, 's32i.n', 'a10, a1, 24'],
    [0x113, 'l32i.n', 'a10, a1, 24'], [0x115, 'call8', '400 <mutate_if_nonnull>'], ...finish];
  assert.deepEqual(noiseNativeCalls(fn(code)), [{ target: 0x400, call_address: 0x115 }]);
});
test('narrow branch reaches the positive path after an early return', () => {
  const code = [[0x100, 'entry', 'a1, 32'], [0x103, 'bnez.n', 'a2, 110 <test+0x10>'],
    [0x105, 'retw.n'], [0x110, 'l32r', 'a8, 80 <literal> (200 <callee>)'], [0x113, 'callx8', 'a8'], [0x116, 'retw.n']];
  assert.deepEqual(noiseNativeCalls(fn(code)), [{ target: 0x200, call_address: 0x113 }]);
});
test('a compiler-merged call site retains both known alternative callees', () => {
  const code = [[0x100, 'entry', 'a1, 32'], [0x103, 'l32r', 'a8, 80 <literal> (200 <first>)'],
    [0x106, 'bnez.n', 'a2, 110 <test+0x10>'], [0x108, 'l32r', 'a8, 84 <literal> (300 <second>)'],
    [0x10b, 'j', '110 <test+0x10>'], [0x110, 'callx8', 'a8'], [0x113, 'retw.n']];
  assert.deepEqual(noiseNativeCalls(fn(code)), [{ target: 0x200, call_address: 0x110 }, { target: 0x300, call_address: 0x110 }]);
});
test('an unknown alternative does not become a proof of the known callee', () => {
  const code = [[0x100, 'entry', 'a1, 32'], [0x103, 'bnez.n', 'a2, 110 <test+0x10>'],
    [0x106, 'l32r', 'a8, 80 <literal> (200 <first>)'], [0x110, 'callx8', 'a8'], [0x113, 'retw.n']];
  assert.deepEqual(noiseNativeCalls(fn(code)), []);
});
test('unreachable padding resembling loop instructions never becomes executable evidence', () => {
  const code = [[0x100, 'entry', 'a1, 32'], [0x103, 'retw.n'], [0x105, 'loop', 'a1, 500 <elsewhere>']];
  assert.equal(noiseNativeInstructions(fn(code)).length, 2);
});
test('a reachable unknown loop bound fails instead of truncating analysis', () => {
  assert.throws(() => noiseNativeCalls(fn([[0x100, 'entry', 'a1, 32'], [0x103, 'loop', 'a2, 500 <elsewhere>']])), /noise_native_loop_unknown/u);
});

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

const privateCalls = rows => noiseNativeCalls(fn(rows), { compilerPrivateSpills: true });
const hasSpill = rows => privateCalls(rows).some(edge => edge.target === 0x200 && edge.call_address === 0x122);
const restore = [[0x116, 'l32r', 'a8, 80 <literal> (200 <callee>)'], [0x119, 's32i.n', 'a8, a1, 16']];

test('private mode preserves an unexposed spill across an external store and loopback', () => {
  const rows = [...spill, [0x110, 's32i.n', 'a2, a3, 0'], [0x113, 'bnez.n', 'a2, 110 <loop>'], ...finish];
  assert(hasSpill(rows));
  assert.deepEqual(noiseNativeCalls(fn(rows)), []);
});
test('private mode preserves a lower spill when a separate later stack buffer escaped', () => {
  assert(hasSpill([...spill, [0x110, 'addi', 'a10, a1, 24'], [0x113, 'callx8', 'a9'],
    [0x116, 's32i.n', 'a2, a3, 0'], ...finish]));
});
test('unknown calls cannot mutate private locals that were never exposed', () => {
  assert(hasSpill([...spill, [0x110, 'callx8', 'a9'], ...finish]));
});
test('an externally stored stack pointer permanently exposes the spill tail', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 's32i.n', 'a2, a3, 0'],
    ...restore, [0x11c, 's32i.n', 'a4, a5, 0'], ...finish]));
});
test('call exposure cannot be undone by restoring a literal into the same slot', () => {
  assert(!hasSpill([...spill, [0x110, 'addi', 'a10, a1, 16'], [0x113, 'callx8', 'a9'],
    ...restore, [0x11c, 'callx8', 'a9'], ...finish]));
});
test('passing a pointer-bearing local exposes its recursively reachable lower slot', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 's32i.n', 'a2, a1, 24'],
    [0x110, 'addi', 'a10, a1, 24'], [0x113, 'callx8', 'a9'], ...finish]));
});
test('nested pointer spills are traversed before any facts are invalidated', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 's32i.n', 'a2, a1, 24'],
    [0x110, 'addi', 'a2, a1, 24'], [0x113, 's32i.n', 'a2, a1, 32'],
    [0x116, 'addi', 'a10, a1, 32'], [0x119, 'callx8', 'a9'], ...finish]));
});
test('pointer cycles terminate and still expose every reachable tail', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 24'], [0x10d, 's32i.n', 'a2, a1, 32'],
    [0x110, 'addi', 'a2, a1, 16'], [0x113, 's32i.n', 'a2, a1, 24'],
    [0x116, 'addi', 'a10, a1, 32'], [0x119, 'callx8', 'a9'], ...finish]));
});
test('a pointer written into a previously exposed local exposes its pointee immediately', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a10, a1, 24'], [0x10d, 'callx8', 'a9'],
    [0x110, 'addi', 'a2, a1, 16'], [0x113, 's32i.n', 'a2, a1, 24'], ...finish]));
});
test('a nullable stack pointer stored externally cannot be classified as a heap value', () => {
  assert(!hasSpill([...spill, [0x10a, 'movi.n', 'a2, 0'], [0x10c, 'bnez.n', 'a3, 112 <join>'],
    [0x10e, 'addi', 'a2, a1, 16'], [0x112, 's32i.n', 'a2, a3, 0'], ...finish]));
});
test('exposure from either branch persists at joins and loop backedges', () => {
  assert(!hasSpill([...spill, [0x10a, 'bnez.n', 'a3, 116 <restore>'],
    [0x10d, 'addi', 'a2, a1, 16'], [0x110, 's32i.n', 'a2, a3, 0'], ...restore,
    [0x11c, 'bnez.n', 'a3, 10a <loop>'], ...finish]));
});
test('partial pointer loads retain possible stack provenance', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 's32i.n', 'a2, a1, 24'],
    [0x110, 'l8ui', 'a10, a1, 25'], [0x113, 'callx8', 'a9'], ...finish]));
});
test('unknown stack-derived arithmetic cannot become an external nonaliasing address', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 'xor', 'a3, a2, a4'],
    [0x110, 's32i.n', 'a5, a3, 0'], ...finish]));
});
test('a conditional move retains a possible previous stack destination', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a10, a1, 16'], [0x10d, 'movnez', 'a10, a2, a3'],
    [0x110, 'callx8', 'a9'], ...finish]));
});
test('an exposed byte overlapping the end of a spill invalidates all four bytes', () => {
  assert(!hasSpill([...spill, [0x110, 'addi', 'a10, a1, 19'], [0x113, 'callx8', 'a9'], ...finish]));
});
test('unsupported conditional stores fail closed in private mode', () => {
  assert(!hasSpill([...spill, [0x110, 's32c1i', 'a2, a3, 0'], ...restore, ...finish]));
});
test('unsupported register-bank transfers cannot launder stack provenance', () => {
  assert(!hasSpill([...spill, [0x110, 'wfr', 'f0, a1'], ...restore, ...finish]));
});
test('a caller-owned slot beyond the entry frame is not a private compiler spill', () => {
  const rows = spill.map(row => row[1] === 'entry' ? [row[0], row[1], 'a1, 16'] : row);
  assert(!hasSpill([...rows, [0x110, 's32i.n', 'a2, a3, 0'], ...finish]));
});

test('a nullable local load cannot launder a pointer to an earlier private spill', () => {
  assert(!hasSpill([...spill, [0x10a, 'addi', 'a2, a1, 16'], [0x10d, 's32i.n', 'a2, a1, 24'],
    [0x110, 'movi.n', 'a3, 0'], [0x112, 'bnez.n', 'a5, 118 <join>'],
    [0x115, 'addi', 'a3, a1, 24'], [0x118, 'l32i.n', 'a4, a3, 0'],
    [0x11b, 's32i.n', 'a5, a4, 0'], ...finish]));
});
test('an inexact stack address plus a positive offset cannot narrow possible aliasing', () => {
  assert(!hasSpill([...spill, [0x110, 'sub', 'a3, a1, a4'],
    [0x113, 's32i.n', 'a2, a3, 100'], ...finish]));
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { parseNativeFunctions } from './telemetry-stack-audit.mjs';
import { noiseNativeCalls } from './noise-native-calls.mjs';
import { resolveNoiseInstructions } from './noise-native-disassembly.mjs';

const initial = `100 <bitaxe_noise_serial_owner_entry>:
 100: 004136 entry a1, 32
 103: 0000 bnez.n a2, 108 <bitaxe_noise_serial_owner_entry+0x8>
 105: f00d retw.n
 108: 000081 l32r a8, 80 <literal> (200 <rustsecp256k1_v0_9_2_test>)
 10b: 0008e0 callx8 a8
 10e: f00d retw.n
200 <rustsecp256k1_v0_9_2_test>:
 200: 004136 entry a1, 32
 203: f00d retw.n
300 <end>:
 300: 004136 entry a1, 32
 303: f00d retw.n
`;
const read = text => resolveNoiseInstructions(parseNativeFunctions(text), text, async () => { throw Error('unexpected_decode'); });

test('existing aligned instructions require no additional decoder invocation', async () => {
  const result = await read(initial);
  assert.equal(result.supplementalRanges, 0);
  assert.deepEqual(result.unresolvedJumps, []);
});
test('branch into an alignment gap is decoded at its exact address', async () => {
  // Arrange: the linear listing consumed one padding byte and missed the true start.
  const text = initial.replace(' 108: 000081 l32r a8, 80 <literal> (200 <rustsecp256k1_v0_9_2_test>)', ' 107: abcdef extui a10, a0, 17, 2');
  const requested = [];
  // Act
  const result = await resolveNoiseInstructions(parseNativeFunctions(text), text, async (start, end) => {
    requested.push({ start, end });
    return ' 108: 000081 l32r a8, 80 <literal> (200 <rustsecp256k1_v0_9_2_test>)\n';
  });
  // Assert
  assert.deepEqual(requested, [{ start: 0x108, end: 0x200 }]);
  assert.deepEqual(noiseNativeCalls(result.functions.get(0x100)), [{ target: 0x200, call_address: 0x10b }]);
  assert(!result.functions.get(0x100).instructions.some(row => row.address === 0x107));
});
test('a decoder that does not supply the requested instruction fails closed', async () => {
  const text = initial.replace('108: 000081', '107: 000081');
  await assert.rejects(resolveNoiseInstructions(parseNativeFunctions(text), text, async () => ''), /noise_native_instruction_missing/u);
});
test('a direct branch outside its function is not silently discarded', async () => {
  await assert.rejects(read(initial.replace('108 <bitaxe_noise_serial_owner_entry+0x8>', '300 <end>')), /noise_native_branch_outside_function/u);
});
test('indirect jump remains an explicit limitation rather than inventing a target', async () => {
  const result = await read(initial.replace('10b: 0008e0 callx8 a8', '10b: 0008e0 jx a8'));
  assert.deepEqual(result.unresolvedJumps, [{ symbol: 'bitaxe_noise_serial_owner_entry', address: 0x10b }]);
});
test('known diverging allocation error terminates before trailing padding', async () => {
  const text = initial.replace('200 <rustsecp256k1_v0_9_2_test>)', '400 <alloc::raw_vec::handle_error>)').replace(' 10e: f00d retw.n\n', '')
    + '400 <alloc::raw_vec::handle_error>:\n 400: 004136 entry a1, 32\n 403: 0000 ill\n';
  const result = await read(text);
  assert.equal(result.functions.get(0x100).instructions.at(-1).terminal, true);
});
test('a branch bypassing a panic load cannot hide the returning path', async () => {
  // Arrange: one predecessor retains a returning address; the other loads abort.
  const text = `100 <bitaxe_noise_serial_owner_entry>:
 100: 004136 entry a1, 32
 103: 000081 l32r a8, 80 <literal> (200 <rustsecp256k1_v0_9_2_test>)
 106: 0000 bnez.n a2, 10b <bitaxe_noise_serial_owner_entry+0xb>
 108: 000081 l32r a8, 84 <literal> (400 <abort>)
 10b: 0008e0 callx8 a8
 10e: 000081 l32r a8, 88 <literal> (500 <rustsecp256k1_v0_9_2_heavy>)
 111: 0008e0 callx8 a8
 114: f00d retw.n
200 <rustsecp256k1_v0_9_2_test>:
 200: 004136 entry a1, 32
 203: f00d retw.n
400 <abort>:
 400: 004136 entry a1, 32
 403: 0000 ill
500 <rustsecp256k1_v0_9_2_heavy>:
 500: 004136 entry a1, 4096
 503: f00d retw.n
600 <end>:
 600: 0000 ill
`;
  // Act
  const result = await read(text), root = result.functions.get(0x100);
  // Assert
  assert(root.instructions.some(row => row.address === 0x111));
  assert(noiseNativeCalls(root).some(edge => edge.target === 0x500));
  assert.equal(root.instructions.find(row => row.address === 0x10b).terminal, false);
});
test('an unknown last call cannot claim the diverging-call exception', async () => {
  const text = initial.replace(' 10e: f00d retw.n\n', '');
  await assert.rejects(read(text), /unexpected_decode/u);
});

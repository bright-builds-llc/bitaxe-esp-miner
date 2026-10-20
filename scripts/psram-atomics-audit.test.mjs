import assert from 'node:assert/strict';
import test from 'node:test';
import { allowedHardwareAtomic, auditPsramAtomics, hardwareAtomicFunctions } from './psram-atomics-audit.mjs';

const RUST = '_ZN3std3sys4sync8once_box16OnceBox$LT$T$GT$10initialize17h2b625b0cb8bd94c7E';
const CAS = ['wsr.scompare1\ta5', 's32c1i\ta2, a6, 0'];
const fn = (symbol, ops) => [`42069964 <${symbol}>:`, ...ops.map((op, index) => `${(0x42069964 + 3 * index).toString(16)}:\t${op}`), ''].join('\n');

test('a Rust function with a hardware compare-exchange fails the audit', () => {
  // Arrange
  const disassembly = fn(RUST, [...CAS, 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'unexpected_hardware_atomics');
  assert.deepEqual(result.unexpected_functions, [RUST]);
});

test('an export-named Rust function with a hardware compare-exchange fails the audit', () => {
  // Arrange
  const disassembly = fn('bitaxe_production_owner_entry', [...CAS, 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.deepEqual(result.unexpected_functions, ['bitaxe_production_owner_entry']);
});

test("ESP-IDF's PSRAM-aware libcall body is allowed", () => {
  // Arrange
  const disassembly = fn('__atomic_s32c1i_compare_exchange_4', [...CAS, 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'no_unexpected_hardware_atomics');
  assert.equal(result.allowed_native_functions, 1);
});

test('only the listed native functions are allowed', () => {
  // Act / Assert
  assert.equal(allowedHardwareAtomic('esp_cpu_compare_and_set'), true);
  assert.equal(allowedHardwareAtomic('__sync_s32c1i_lock_test_and_set_4'), true);
  assert.equal(allowedHardwareAtomic('bitaxe_allocation_failure_record'), true);
  assert.equal(allowedHardwareAtomic('spinlock_acquire'), false);
  assert.equal(allowedHardwareAtomic('__atomic_compare_exchange_4'), false);
});

test('a Rust function that calls the atomic libcall passes', () => {
  // Arrange
  const disassembly = fn(RUST, ['l32r\ta8, 42000000 <__atomic_compare_exchange_4>', 'callx8\ta8', 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'no_unexpected_hardware_atomics');
});

test('each function counts its own compare-exchanges', () => {
  // Arrange
  const disassembly = fn(RUST, [...CAS, ...CAS]) + fn('esp_cpu_compare_and_set', CAS);
  // Act
  const functions = hardwareAtomicFunctions(disassembly);
  // Assert
  assert.deepEqual(functions, [{ symbol: RUST, count: 2 }, { symbol: 'esp_cpu_compare_and_set', count: 1 }]);
});

test('an s32c1i in an address literal comment is not an instruction', () => {
  // Arrange
  const disassembly = fn(RUST, ['wsr.scompare1\ta5', 'l32r\ta8, 42000000 <__atomic_s32c1i_load_4>', 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'no_unexpected_hardware_atomics');
});

test('an opcode after raw instruction bytes is found', () => {
  // Arrange
  const disassembly = `42069964 <${RUST}>:\n4206998b:\t130300      \twsr.scompare1\ta5\n42069990:\t006632      \ts32c1i\ta2, a6, 0\n`;
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'unexpected_hardware_atomics');
});

test('an s32c1i with no SCOMPARE1 write before it is decoded data, not a compare-exchange', () => {
  // Arrange
  const disassembly = fn(RUST, ['l32i\ta8, a1, 0', 's32c1i\ta4, a0, 0x228', 'retw.n']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'no_unexpected_hardware_atomics');
});

test('the SCOMPARE1 write of one function never pairs with the next function', () => {
  // Arrange
  const disassembly = fn('esp_cpu_compare_and_set', ['wsr.scompare1\ta4']) + fn(RUST, ['s32c1i\ta2, a6, 0']);
  // Act
  const result = auditPsramAtomics(disassembly);
  // Assert
  assert.equal(result.result, 'no_unexpected_hardware_atomics');
});

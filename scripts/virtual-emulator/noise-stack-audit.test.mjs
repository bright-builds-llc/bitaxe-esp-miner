import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { auditNoiseStack, parseNoiseFrames, measureNoisePath, noiseAuditIdentity, validateNoiseAudit } from './noise-stack-audit.mjs';

const CONFIG = 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384\nCONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y\nCONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y\n# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set\n';
const names = ['bitaxe_virtual_firmware::main', 'bitaxe_virtual_firmware::guest::run',
  'bitaxe_virtual_firmware::noise_probe::run_and_emit', 'bitaxe_simulation::noise_probe::run',
  'bitaxe_simulation::noise_probe::handshake_and_frame', 'bitaxe_stratum::v2::noise::NoiseInitiator::complete_diagnostic',
  'noise_sv2::initiator::Initiator::step_2_with_now', 'noise_sv2::handshake::HandshakeOp::mix_hash',
  'bitaxe_simulation::noise_probe::frame_round_trip'];
const digest = text => createHash('sha256').update(text).digest('hex');

function fixture(sizes = [128, 912, 96, 32, 6368, 1408, 3472, 288, 512]) {
  return names.map((name, index) => {
    const address = 0x40000000 + index * 0x100;
    const next = index === 4 ? [5, 8] : index < 7 ? [index + 1] : [];
    return `${address.toString(16)} <${name}>:\n ${address.toString(16)}: 004136 entry a1, 0x${sizes[index].toString(16)}\n` +
      next.map(target => ` ${(address + 3).toString(16)}: 000005 call8 ${(0x40000000 + target * 0x100).toString(16)} <${names[target]}>\n`).join('');
  }).join('\n');
}

test('selected native paths fit without treating SDK extra stack as spendable margin', () => {
  // Arrange
  const disassembly = fixture();
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, true);
  assert.equal(result.available_selected_path_bytes, 14336);
  assert.equal(result.sdk_extra_stack_bytes, 512);
  assert.equal(result.complete_callgraph_bound, false);
  assert.equal(result.paths[0].frame_bytes, 12704);
});

test('retained composed callers and completion chain violate the unchanged stack margin', () => {
  // Arrange
  const disassembly = fixture([128, 928, 8464, 176, 7136, 1408, 3472, 288, 512]);
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, false);
  assert.equal(result.paths[0].frame_bytes, 22000);
});

test('aliases at one entry are not added twice like inline debugger frames', () => {
  // Arrange
  const disassembly = '40000000 <outer>:\n 40000000: 004136 entry a1, 288\n' +
    '40000000 <inline_alias>:\n 40000000: 004136 entry a1, 288\n';
  // Act
  const result = measureNoisePath(parseNoiseFrames(disassembly), ['outer', 'inline_alias']);
  // Assert
  assert.equal(result.frame_bytes, 288);
  assert.equal(result.native_frame_count, 1);
});

test('a missing mandatory completion symbol cannot pass by omission', () => {
  // Arrange
  const disassembly = fixture().replaceAll(names[7], 'missing_completion');
  // Act / Assert
  assert.throws(() => auditNoiseStack(disassembly, CONFIG), /required_symbol_missing/);
});

test('a symbol without a demonstrated direct call is not a selected path', () => {
  // Arrange
  const disassembly = fixture().replace(/call8 40000700 <[^>]+>/, 'callx8 a8');
  // Act / Assert
  assert.throws(() => auditNoiseStack(disassembly, CONFIG), /required_call_missing/);
});

test('Xtensa longcall is resolved only from the unchanged annotated literal register', () => {
  // Arrange
  const disassembly = fixture().replace('000005 call8 40000700 <' + names[7] + '>',
    '000081 l32r a8, 40009900 <literal> (40000700 <' + names[7] + '>)\n 40000604: 02ad mov.n a10, a2\n 40000606: 0008e0 callx8 a8');
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, true);
});

test('clobbering a longcall register leaves a mandatory call unresolved', () => {
  // Arrange
  const disassembly = fixture().replace('000005 call8 40000700 <' + names[7] + '>',
    '000081 l32r a8, 40009900 <literal> (40000700 <' + names[7] + '>)\n 40000604: 00a082 movi a8, 0\n 40000606: 0008e0 callx8 a8');
  // Act / Assert
  assert.throws(() => auditNoiseStack(disassembly, CONFIG), /required_call_missing/);
});

test('increased stack configuration cannot hide the failing path', () => {
  // Arrange
  const config = CONFIG.replace('16384', '32768');
  // Act / Assert
  assert.throws(() => auditNoiseStack(fixture(), config), /noise_stack_config/);
});

async function boundReceipt() {
  const disassembly = fixture(), receipt = auditNoiseStack(disassembly, CONFIG);
  const expected = { elfSha256: 'a'.repeat(64), sdkconfigSha256: digest(CONFIG), compiledSourceSha256: 'c'.repeat(64) };
  receipt.bindings = { elf_sha256: expected.elfSha256, sdkconfig_sha256: expected.sdkconfigSha256,
    compiled_source_sha256: expected.compiledSourceSha256, auditor_sha256: await noiseAuditIdentity(),
    objdump_version: 'esp-14.2.0_20260121', objdump_sha256: 'd'.repeat(64), disassembly_sha256: digest(disassembly) };
  return { receipt, expected, proof: { disassembly, sdkconfig: CONFIG } };
}

test('admission recomputes exact native proof rather than crediting declared fields', async () => {
  // Arrange
  const { receipt, expected, proof } = await boundReceipt();
  // Act
  const actual = await validateNoiseAudit(receipt, expected, proof);
  // Assert
  assert.equal(actual, receipt);
  await assert.rejects(validateNoiseAudit(receipt, expected), /native_proof/);
});

test('a stale native ELF binding is rejected', async () => {
  // Arrange
  const { receipt, expected, proof } = await boundReceipt();
  expected.elfSha256 = 'e'.repeat(64);
  // Act / Assert
  await assert.rejects(validateNoiseAudit(receipt, expected, proof), /noise_audit_binding/);
});

test('receipt path size mutation is rejected against native proof', async () => {
  // Arrange
  const { receipt, expected, proof } = await boundReceipt();
  receipt.paths[0].frame_bytes = 1;
  // Act / Assert
  await assert.rejects(validateNoiseAudit(receipt, expected, proof), /native_proof/);
});

test('a stale auditor closure is rejected', async () => {
  // Arrange
  const { receipt, expected, proof } = await boundReceipt();
  receipt.bindings.auditor_sha256 = '0'.repeat(64);
  // Act / Assert
  await assert.rejects(validateNoiseAudit(receipt, expected, proof), /noise_audit_invalid/);
});

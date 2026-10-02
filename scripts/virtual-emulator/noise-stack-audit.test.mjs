import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { auditNoiseStack, parseNoiseFrames, measureNoisePath, noiseAuditIdentity, validateNoiseAudit } from './noise-stack-audit.mjs';

const CONFIG = 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384\nCONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y\nCONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y\n# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set\n';
const digest = text => createHash('sha256').update(text).digest('hex');
const HANDSHAKE = 'bitaxe_simulation::noise_probe::handshake_and_frame';
const VERIFY = 'rustsecp256k1_v0_9_2_schnorrsig_verify';
const STRAUSS = 'rustsecp256k1_v0_9_2_ecmult_strauss_wnaf';
const RUN = 'bitaxe_virtual_firmware::noise_probe::run_and_emit';
// Corrected helper topology with synthetic frame sizes; [name, frame bytes, direct callees].
const GRAPH = [
  ['bitaxe_virtual_firmware::main', 128, ['bitaxe_virtual_firmware::guest::run']],
  ['bitaxe_virtual_firmware::guest::run', 928, [RUN]],
  [RUN, 96, ['bitaxe_simulation::noise_probe::run', 'bitaxe_virtual_firmware::noise_probe::emit_outcome']],
  ['bitaxe_virtual_firmware::noise_probe::emit_outcome', 576, []],
  ['bitaxe_simulation::noise_probe::run', 32, [HANDSHAKE]],
  [HANDSHAKE, 480, ['bitaxe_simulation::noise_probe::prepare_initiator', 'bitaxe_simulation::noise_probe::respond',
    'bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into',
    'bitaxe_simulation::noise_probe::frame_round_trip']],
  ['bitaxe_simulation::noise_probe::prepare_initiator', 208, ['noise_sv2::initiator::Initiator::new_with_rng',
    'bitaxe_stratum::v2::noise::NoiseInitiator::act_one']],
  ['noise_sv2::initiator::Initiator::new_with_rng', 512, ['rustsecp256k1_v0_9_2_ecmult_gen']],
  ['bitaxe_stratum::v2::noise::NoiseInitiator::act_one', 256, ['rustsecp256k1_v0_9_2_ellswift_encode']],
  ['rustsecp256k1_v0_9_2_ellswift_encode', 512, []],
  ['rustsecp256k1_v0_9_2_ecmult_gen', 1024, []],
  ['bitaxe_simulation::noise_probe::respond', 96, ['bitaxe_simulation::noise_probe::construct_responder',
    'bitaxe_simulation::noise_probe::step_responder']],
  ['bitaxe_simulation::noise_probe::construct_responder', 112, ['noise_sv2::responder::Responder::from_authority_kp_with_rng']],
  ['noise_sv2::responder::Responder::from_authority_kp_with_rng', 400, ['rustsecp256k1_v0_9_2_keypair_create']],
  ['rustsecp256k1_v0_9_2_keypair_create', 304, ['rustsecp256k1_v0_9_2_ecmult_gen']],
  ['bitaxe_simulation::noise_probe::step_responder', 1408, ['noise_sv2::responder::Responder::step_1_with_now_rng']],
  ['noise_sv2::responder::Responder::step_1_with_now_rng', 5152, ['rustsecp256k1_v0_9_2_ecmult_const', 'rustsecp256k1_v0_9_2_ecmult_gen']],
  ['rustsecp256k1_v0_9_2_ecmult_const', 2000, ['memcpy']],
  ['memcpy', 4000, []],
  ['bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into', 1168,
    ['noise_sv2::initiator::Initiator::step_2_with_now', 'bitaxe_stratum::v2::noise::completion::store_result']],
  ['bitaxe_stratum::v2::noise::completion::store_result', 48, []],
  ['noise_sv2::initiator::Initiator::step_2_with_now', 3472, ['noise_sv2::handshake::HandshakeOp::mix_hash', VERIFY]],
  ['noise_sv2::handshake::HandshakeOp::mix_hash', 288, []],
  [VERIFY, 2624, [STRAUSS]],
  [STRAUSS, 1536, ['rustsecp256k1_v0_9_2_ecmult_odd_multiples_table']],
  ['rustsecp256k1_v0_9_2_ecmult_odd_multiples_table', 688, []],
  ['bitaxe_simulation::noise_probe::frame_round_trip', 160, ['bitaxe_stratum::v2::noise::NoiseTransport::encrypt_frame']],
  ['bitaxe_stratum::v2::noise::NoiseTransport::encrypt_frame', 400, []],
];
const ADDRESS = new Map(GRAPH.map(([name], index) => [name, 0x40000000 + index * 0x100]));

function fixture(sizes = {}) {
  return GRAPH.map(([name, bytes, callees]) => {
    const address = ADDRESS.get(name).toString(16);
    return `${address} <${name}>:\n ${address}: 004136 entry a1, 0x${(sizes[name] ?? bytes).toString(16)}\n` +
      callees.map(callee => ` ${address}: 000005 call8 ${ADDRESS.get(callee).toString(16)} <${callee}>\n`).join('');
  }).join('\n');
}
const pathOf = (result, id) => result.paths.find(path => path.id === id);

test('corrected helper paths fit without treating SDK extra stack as spendable margin', () => {
  // Arrange
  const disassembly = fixture();
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, true);
  assert.equal(result.available_selected_path_bytes, 14336);
  assert.equal(result.sdk_extra_stack_bytes, 512);
  assert.equal(result.complete_callgraph_bound, false);
  assert.deepEqual(result.paths.map(path => [path.id, path.frame_bytes]), [
    ['initiator_constructor', 3408], ['act_one', 2640], ['responder_constructor', 3600],
    ['responder_ecdh_and_sign', 10320], ['completion', 11152], ['certificate_verification', 11152],
    ['encrypted_frame', 2224], ['outcome_emission', 1728]]);
});

test('certificate verification counts nested Strauss descent beyond its named boundary', () => {
  // Arrange
  const disassembly = fixture();
  // Act
  const path = pathOf(auditNoiseStack(disassembly, CONFIG), 'certificate_verification');
  // Assert
  assert.equal(path.crypto_boundary, VERIFY);
  assert.deepEqual(path.native_symbols.slice(-3), [VERIFY, STRAUSS, 'rustsecp256k1_v0_9_2_ecmult_odd_multiples_table']);
});

test('the original oversized caller frame keeps certificate verification over budget', () => {
  // Arrange
  const disassembly = fixture({ [HANDSHAKE]: 6368 });
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, false);
  assert.equal(pathOf(result, 'certificate_verification').frame_bytes, 17040);
  assert.equal(pathOf(result, 'certificate_verification').fit, false);
});

test('an oversized responder step frame fails even when completion fits', () => {
  // Arrange
  const disassembly = fixture({ 'bitaxe_simulation::noise_probe::step_responder': 5440 });
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, false);
  assert.equal(pathOf(result, 'responder_ecdh_and_sign').fit, false);
  assert.equal(pathOf(result, 'completion').fit, true);
});

test('calls outside the crypto family are reported as gaps rather than credited', () => {
  // Arrange
  const disassembly = fixture();
  // Act
  const path = pathOf(auditNoiseStack(disassembly, CONFIG), 'responder_ecdh_and_sign');
  // Assert
  assert.equal(path.outside_crypto_family_edges, 1);
  assert.equal(path.native_symbols.includes('memcpy'), false);
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

test('a missing certificate verification boundary cannot pass by omission', () => {
  // Arrange
  const disassembly = fixture().replaceAll(VERIFY, 'missing_verification');
  // Act / Assert
  assert.throws(() => auditNoiseStack(disassembly, CONFIG), /noise_crypto_boundary_missing/);
});

test('a symbol without a demonstrated direct call is not a selected path', () => {
  // Arrange
  const disassembly = fixture().replace(/call8 [a-f0-9]+ <noise_sv2::initiator::Initiator::step_2_with_now>/, 'callx8 a8');
  // Act / Assert
  assert.throws(() => auditNoiseStack(disassembly, CONFIG), /required_call_missing/);
});

const STEP_TWO = 'noise_sv2::initiator::Initiator::step_2_with_now';
const STEP_TWO_CALL = `000005 call8 ${ADDRESS.get(STEP_TWO).toString(16)} <${STEP_TWO}>`;

test('Xtensa longcall is resolved only from the unchanged annotated literal register', () => {
  // Arrange
  const disassembly = fixture().replace(STEP_TWO_CALL,
    `000081 l32r a8, 40009900 <literal> (${ADDRESS.get(STEP_TWO).toString(16)} <${STEP_TWO}>)\n 40001304: 02ad mov.n a10, a2\n 40001306: 0008e0 callx8 a8`);
  // Act
  const result = auditNoiseStack(disassembly, CONFIG);
  // Assert
  assert.equal(result.selected_path_budget_fit, true);
});

test('clobbering a longcall register leaves a mandatory call unresolved', () => {
  // Arrange
  const disassembly = fixture().replace(STEP_TWO_CALL,
    `000081 l32r a8, 40009900 <literal> (${ADDRESS.get(STEP_TWO).toString(16)} <${STEP_TWO}>)\n 40001304: 00a082 movi a8, 0\n 40001306: 0008e0 callx8 a8`);
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

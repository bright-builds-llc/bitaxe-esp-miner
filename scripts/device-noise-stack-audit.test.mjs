import test from 'node:test';
import assert from 'node:assert/strict';
import { auditDeviceNoiseStack } from './device-noise-stack-audit.mjs';

const COMPLETE = 'bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into';
const STEP_TWO = 'noise_sv2::initiator::Initiator::step_2_with_now';
const VERIFY = 'rustsecp256k1_v0_9_2_schnorrsig_verify';
const HELPER = 'bitaxe_firmware::noise_completion_stack::complete_in_helper';
const DIAGNOSTIC = 'bitaxe_stratum::v2::noise::diagnostic::run';
const SPAWN = 'bitaxe_firmware::noise_completion_stack::authenticate_on_psram_stack';
const SOURCES = {
  helper: 'pub(crate) const COMPLETION_STACK_BYTES: usize = 16 * 1024;\nconfig.stack_alloc_caps = sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT;\n',
  transport: 'const WORKER_STACK_BYTES: usize = 12 * 1024;\n',
};
// Corrected device topology with synthetic frame sizes; [name, frame bytes, direct callees].
function graph(diagnosticCallees = ['noise_sv2::initiator::Initiator::new_with_rng', SPAWN]) {
  return [
    ['bitaxe_firmware::production_mining_session::transport::run_worker', 112, ['bitaxe_firmware::production_mining_session::transport::v2::run']],
    ['bitaxe_firmware::production_mining_session::transport::v2::run', 128, ['bitaxe_firmware::v2_serial_runtime::run_share_transport']],
    ['bitaxe_firmware::v2_serial_runtime::run_share_transport', 80, ['bitaxe_firmware::v2_serial_runtime::observer::run_share']],
    ['bitaxe_firmware::v2_serial_runtime::observer::run_share', 224, [DIAGNOSTIC]],
    ['bitaxe_v2_channel_owner_entry', 48, ['bitaxe_firmware::v2_serial_runtime::observer::run_channel']],
    ['bitaxe_firmware::v2_serial_runtime::observer::run_channel', 240, [DIAGNOSTIC]],
    [DIAGNOSTIC, 256, diagnosticCallees],
    ['noise_sv2::initiator::Initiator::new_with_rng', 512, ['rustsecp256k1_v0_9_2_ecmult_gen']],
    ['rustsecp256k1_v0_9_2_ecmult_gen', 1024, []],
    [SPAWN, 224, []],
    [HELPER, 32, [COMPLETE]],
    [COMPLETE, 1168, [STEP_TWO]],
    [STEP_TWO, 3472, [VERIFY]],
    [VERIFY, 2624, ['rustsecp256k1_v0_9_2_ecmult_strauss_wnaf']],
    ['rustsecp256k1_v0_9_2_ecmult_strauss_wnaf', 1536, ['rustsecp256k1_v0_9_2_ecmult_odd_multiples_table']],
    ['rustsecp256k1_v0_9_2_ecmult_odd_multiples_table', 688, []],
  ];
}
function disassembly(rows) {
  const address = new Map(rows.map(([name], index) => [name, (0x40000000 + index * 0x100).toString(16)]));
  return rows.map(([name, bytes, callees]) => `${address.get(name)} <${name}>:\n ${address.get(name)}: 004136 entry a1, 0x${bytes.toString(16)}\n` +
    callees.map(callee => ` ${address.get(name)}: 000005 call8 ${address.get(callee)} <${callee}>\n`).join('')).join('\n');
}
const pathOf = (result, id) => result.paths.find(path => path.id === id);

test('the corrected device keeps completion on the PSRAM helper with worker headroom', () => {
  // Arrange / Act
  const result = auditDeviceNoiseStack(disassembly(graph()), SOURCES);
  // Assert
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.completion_only_on_helper, true);
  assert.equal(pathOf(result, 'helper_certificate').frame_bytes, 9520);
  assert.equal(pathOf(result, 'worker_share_crypto').frame_bytes, 2336);
  assert.equal(pathOf(result, 'worker_share_crypto').budget_bytes, 10240);
});

test('a removed helper spawn fails closed', () => {
  // Arrange
  const original = graph(['noise_sv2::initiator::Initiator::new_with_rng', COMPLETE]);
  // Act / Assert
  assert.throws(() => auditDeviceNoiseStack(disassembly(original), SOURCES), /required_call_missing/);
});

test('completion called directly on the worker is blocked', () => {
  // Arrange
  const original = graph(['noise_sv2::initiator::Initiator::new_with_rng', SPAWN, COMPLETE]);
  // Act
  const result = auditDeviceNoiseStack(disassembly(original), SOURCES);
  // Assert
  assert.equal(result.completion_only_on_helper, false);
  assert.equal(result.result, 'blocked');
  assert.equal(pathOf(result, 'worker_share_crypto').fit, false);
});

test('a changed stack contract cannot pass the audit', () => {
  // Arrange
  const sources = { ...SOURCES, transport: 'const WORKER_STACK_BYTES: usize = 16 * 1024;\n' };
  // Act / Assert
  assert.throws(() => auditDeviceNoiseStack(disassembly(graph()), sources), /device_noise_stack_contract/);
});

test('an internal helper stack placement cannot pass the audit', () => {
  // Arrange
  const sources = { ...SOURCES, helper: SOURCES.helper.replace('MALLOC_CAP_SPIRAM', 'MALLOC_CAP_INTERNAL') };
  // Act / Assert
  assert.throws(() => auditDeviceNoiseStack(disassembly(graph()), sources), /device_noise_stack_contract/);
});

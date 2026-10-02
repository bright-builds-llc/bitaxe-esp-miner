import test from 'node:test';
import assert from 'node:assert/strict';
import { auditNoisePrefixStack } from './noise-prefix-audit.mjs';

const config = 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384\nCONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y\nCONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y\n# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set\n';
const names = ['bitaxe_virtual_firmware::main', 'bitaxe_virtual_firmware::guest::run',
  'bitaxe_virtual_firmware::noise_probe::run_prefix_and_emit', 'bitaxe_simulation::noise_probe::prefix::run_until',
  'bitaxe_simulation::noise_probe::prefix::handshake_until_prefix', 'bitaxe_stratum::v2::noise::NoiseInitiator::new',
  'noise_sv2::initiator::Initiator::new_with_rng', 'bitaxe_stratum::v2::noise::NoiseInitiator::act_one',
  'secp256k1::ellswift::ElligatorSwift::shared_secret', 'rustsecp256k1_v0_9_2_ellswift_xdh',
  'rustsecp256k1_v0_9_2_ecmult_const$part$0', 'rustsecp256k1_v0_9_2_gej_add_ge',
  'secp256k1::schnorr::<impl secp256k1::Secp256k1<C>>::sign_schnorr_with_rng',
  'rustsecp256k1_v0_9_2_schnorrsig_sign_internal', 'bitaxe_virtual_noise_checkpoint',
  'bitaxe_virtual_noise_prefix_cutoff', 'bitaxe_virtual_noise_checkpoint_complete', 'bitaxe_virtual_noise_prefix_released',
  'rustsecp256k1_v0_9_2_keypair_create'];
function fixture(handshakeBytes = 6368) {
  return names.map((name, index) => {
    const address = 0x40000000 + index * 0x100;
    const children = index < 4 ? [index + 1] : index === 4 ? [5, 7, 8, 12, 18] : [5, 8, 9, 10, 12].includes(index) ? [index + 1] : [];
    return `${address.toString(16)} <${name}>:\n ${address.toString(16)}: 004136 entry a1, ${index === 4 ? handshakeBytes : 128}\n` +
      children.map(child => ` ${(address + 3).toString(16)}: 000005 call8 ${(0x40000000 + child * 0x100).toString(16)} <${names[child]}>\n`).join('');
  }).join('\n');
}
test('prefix excludes completion but reports actual caller frame instead of claiming baseline repair', () => {
  // Arrange / Act
  const report = auditNoisePrefixStack(fixture(), config, 110);
  // Assert
  assert.equal(report.selected_path_budget_fit, true);
  assert.equal(report.actual_prefix_handshake_frame_bytes, 6368);
  assert.equal(report.full_noise_eligible, false);
  assert.equal(report.excluded_certificate_branch.minimum_known_native_path_bytes, 17264);
});
test('known-red completion cannot be admitted by a prefix receipt', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditNoisePrefixStack(fixture(), config, 111), /scope/);
});
test('Entry already reserves the full native prefix helper frame', () => {
  // Arrange / Act
  const report = auditNoisePrefixStack(fixture(15000), config, 101);
  // Assert
  assert.equal(report.selected_path_budget_fit, false);
  assert.ok(report.paths[0].native_symbols.includes('bitaxe_simulation::noise_probe::prefix::handshake_until_prefix'));
});
test('a large constructor prefix is rejected at the same configured margin', () => {
  // Arrange / Act
  const report = auditNoisePrefixStack(fixture(15000), config, 103);
  // Assert
  assert.equal(report.selected_path_budget_fit, false);
});
test('removed compiled instructions are visible as changed caller footprint', () => {
  // Arrange / Act
  const report = auditNoisePrefixStack(fixture(2048), config, 110);
  // Assert
  assert.equal(report.prefix_frame_delta_from_baseline_bytes, -4320);
  assert.equal(report.baseline_helper_changed, true);
  assert.equal(report.full_noise_eligible, false);
});

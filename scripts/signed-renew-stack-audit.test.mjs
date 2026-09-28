import test from 'node:test';
import assert from 'node:assert/strict';
import { auditSignedRenew } from './signed-renew-stack-audit.mjs';

const names = ['pthread_task_func', 'std::sys::pal::unix::thread::Thread::new::thread_start',
  'core::ops::function::FnOnce::call_once{{vtable.shim}}', 'std::sys::backtrace::__rust_begin_short_backtrace',
  'bitaxe_firmware::bwg_worker_usb::run_owner',
  'bitaxe_worker_control::controller::frame::<impl WorkerControl<V,S>>::prepare_frame',
  'bitaxe_worker_control::controller::renew_dispatch::<impl WorkerControl<V,S>>::prepare_renew_controller',
  'WorkLeaseAuthorizationVerifier<S>::verify_renewal', 'ed25519_dalek::verifying::VerifyingKey::verify_strict',
  'ed25519_dalek::verifying::RCompute<CtxDigest>::compute',
  'curve25519_dalek::edwards::EdwardsPoint::vartime_double_scalar_mul_basepoint',
  'curve25519_dalek::backend::vartime_double_base_mul',
  '<curve25519_dalek::window::NafLookupTable5<ProjectiveNielsPoint> as From<T>>::from',
  'curve25519_dalek::backend::serial::curve_models::ProjectivePoint::double',
  'curve25519_dalek::backend::serial::u32::field::FieldElement2625::square_inner'];
function fixture({ historical = false, huge = false, missing = false, dynamic = false } = {}) {
  const sizes = [32,32,1856,32,688,432,historical ? 4592 : 1760,544,608,1472,32,2688,2096,544,384];
  if (huge) sizes[5] = 4944;
  return names.map((name,index) => {
    const address = 0x42000000 + index * 0x100;
    if (historical && index === 6) name = 'WorkerControl<V,S>::prepare_controller';
    const instructions = [`${address.toString(16)}: 000136 entry a1, ${sizes[index]}`];
    if (dynamic && index === 6) instructions.push(`${(address+3).toString(16)}: 000000 addi a1, a1, -16`);
    if (index >= 2 && index < names.length-1 && !(missing && index === 6)) instructions.push(`${(address+6).toString(16)}: 000000 call8 ${(address+0x100).toString(16)} <next>`);
    instructions.push(`${(address+9).toString(16)}: 000081 retw.n`);
    return `${address.toString(16)} <${name}>:\n${instructions.join('\n')}\n`;
  }).join('\n');
}
test('historical nested Renew chain fails required margin in the configured control stack', () => {
  const result = auditSignedRenew(fixture({ historical: true }));
  assert.equal(result.selected_fixed_entry_bytes, 16032);
  assert.equal(result.result, 'budget_exceeded');
});
test('outlined chain retains explicit unclaimed margin and limited claims', () => {
  const result = auditSignedRenew(fixture());
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.unclaimed_headroom_bytes, 3184);
  assert.equal(result.complete_callgraph_bound, false);
  assert.equal(result.hardware_verified, false);
});
test('dispatcher inlining into the caller fails the budget', () => {
  assert.equal(auditSignedRenew(fixture({ huge: true })).result, 'budget_exceeded');
});
test('missing signature edge fails closed', () => {
  assert.throws(() => auditSignedRenew(fixture({ missing: true })), /signed_renew/);
});
test('dynamic stack adjustment cannot claim fixed headroom', () => {
  assert.throws(() => auditSignedRenew(fixture({ dynamic: true })), /stack/);
});

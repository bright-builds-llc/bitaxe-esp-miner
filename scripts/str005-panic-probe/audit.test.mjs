import assert from 'node:assert/strict';
import test from 'node:test';
import { validateNativeAudit } from './audit.mjs';
const hash = 'a'.repeat(64);
const valid = { schema: 'str005-native-panic-cutoff-audit-v1', elf_sha256: hash, wrapper_iram: true, literals_iram: true,
  state_internal_dram: true, safe_latches_before_delegate: true, generation_revoked_before_delegate: true,
  no_calls_or_branches_before_cutoff: true, port_routes_wrapper: true, wrapper_instructions: 48, hardware_verified: false };
test('native audit binds the exact ELF and every prerequisite without promoting hardware evidence', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => validateNativeAudit(valid, hash));
  for (const key of Object.keys(valid).filter(key => valid[key] === true)) assert.throws(() => validateNativeAudit({ ...valid, [key]: false }, hash));
  assert.throws(() => validateNativeAudit(valid, 'b'.repeat(64)));
  assert.throws(() => validateNativeAudit({ ...valid, hardware_verified: true }, hash));
});

test('store audit requires every native prerequisite and a bounded added stack', async () => {
  const { validateStoreAudit } = await import('./audit.mjs');
  const report = { schema:'str005-native-core-store-audit-v1',elf_sha256:hash,wrappers_iram:true,receipt_rtc_noinit:true,current_metadata_internal:true,
    sdk_routes_wrapped:true,real_calls_preserved:true,diagnostic_call_closure:true,bounded_diagnostic_writes:true,normal_boot_initializer_linked:true,
    normal_boot_init_runtime_verified:false,hardware_verified:false,max_added_stack_bytes:240,added_stack_budget_bytes:256 };
  assert.doesNotThrow(()=>validateStoreAudit(report,hash));
  for(const key of Object.keys(report).filter(key=>report[key]===true)) assert.throws(()=>validateStoreAudit({...report,[key]:false},hash));
  assert.throws(()=>validateStoreAudit({...report,max_added_stack_bytes:272},hash));
  assert.throws(()=>validateStoreAudit(report,'b'.repeat(64)));
});

test('task-stack capture requires selected-user-region evidence while history remains readable', () => {
  // Arrange
  const selected = { ...valid, schema: 'str005-native-panic-cutoff-audit-v2', receipt_user_region: true };
  // Act / Assert
  assert.doesNotThrow(() => validateNativeAudit(valid,hash));
  assert.throws(() => validateNativeAudit(valid,hash,true));
  assert.doesNotThrow(() => validateNativeAudit(selected,hash,true));
  assert.throws(() => validateNativeAudit({...selected,receipt_user_region:false},hash,true));
});

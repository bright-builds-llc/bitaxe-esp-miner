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

import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { proof } from '../str005-noise-serial/files.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';

const flags = ['wrapper_iram', 'literals_iram', 'state_internal_dram', 'safe_latches_before_delegate',
  'generation_revoked_before_delegate', 'no_calls_or_branches_before_cutoff', 'port_routes_wrapper'];
export function validateNativeAudit(report, elfSha256) {
  object(report, ['schema', 'elf_sha256', ...flags, 'wrapper_instructions', 'hardware_verified']);
  check(report.schema === 'str005-native-panic-cutoff-audit-v1' && report.elf_sha256 === elfSha256 &&
    flags.every(key => report[key] === true) && report.hardware_verified === false &&
    Number.isSafeInteger(report.wrapper_instructions) && report.wrapper_instructions > 0 && report.wrapper_instructions <= 96,
  'panic_native_audit');
}
export async function verifyNativeAudit(root, context) {
  const saved = await proof(root, 'native-audit.json');
  check(saved.sha256 === context.nativeAuditSha256 && await fileDigest(context.candidateElf) === context.app_elf_sha256, 'panic_native_audit_changed');
  validateNativeAudit(saved.value, context.app_elf_sha256);
}

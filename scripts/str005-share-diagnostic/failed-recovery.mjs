import { proof } from '../str005-noise-serial/files.mjs';
import { validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { sealed } from './anchors.mjs';
const RECOVERY_SEAL = 'b5c2ebf1c0805f90498759f9fe798255f4a5f33892d33943b658e407663b1741';
/** The failed read and Stop are immutable context, never an admission or safety proof. */
export async function failedStatusRecovery(root, repo) {
  const seal = await sealed(root, repo);
  check(seal === RECOVERY_SEAL, 'diagnostic_failed_recovery_seal');
  const context = (await proof(root, 'context.json')).value, result = (await proof(root, 'result.json')).value;
  const ledger = (await proof(root, 'baseline-ledger.json')).value;
  const finished = (await proof(root, 'baseline-finished.json')).value;
  validateLedger(ledger);
  check(context.diagnosticSuccessor === true && context.stage === 'recovery' &&
    context.commit === '30487d6de59b96c7ae00096ddfd4c1fa4f4e3349' &&
    context.firmware_commit === 'ce8f015811b93385c5aab0bac7bcdf6307452b31' &&
    context.app_elf_sha256 === '453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31' &&
    result.complete === false && result.host_resources_released === true && result.current_safe_recovery === false &&
    result.first_failure?.phase === 'status' && finished.failures.includes('status') && finished.failures.includes('closed') &&
    ledger.next_ordinal === 22 && ledger.last_completed_ordinal === 21 && ledger.pending === false,
  'diagnostic_failed_recovery_binding');
  return { root, seal, context, result, ledger };
}

/** A successful write with missing candidate recovery is identity lineage, never qualification. */
import { proof, verifyInventory, privateRoot } from '../str005-noise-serial/files.mjs';
import { ignored } from '../fixed-usb-qualification/contract.mjs';
import { inspectInstall } from './install.mjs';
import { validateNativeAudit } from './audit.mjs';
import { check } from '../str005-v2-serial/values.mjs';
export const PARTIAL_INSTALL_SEAL = '8d6a89683e92b6d798b92a01b38882138b5db5f177058c3c0ed8f9a04b85a5ad';
export function validatePartialInstall(context, result) {
  check(context.schema === 'str005-panic-probe-v1' && context.renewSuccessor === true &&
    context.ownerTask === 'task-str005-v2-accepted-share-probe' && context.installEnabled === true && context.selfTestEnabled === false &&
    context.firmware_commit === '8d4e470e4cd3e18be510d3912586aeda65960275' &&
    context.app_elf_sha256 === 'd0dd775335e55e9d009a0a779bc938bebf64e76fe78d8d7c8144a06ef1c9d88e' &&
    result.schema === 'str005-panic-baseline-result-v1' && result.complete === false && result.installation_complete === true &&
    result.installation_attempted === true && result.candidate_install_reviewed === true && result.baseline_complete === true &&
    result.host_resources_released === true && JSON.stringify(result.blockers) === JSON.stringify(['candidate_recovery_missing']) &&
    Array.isArray(result.candidate_recoveries) && result.candidate_recoveries.length === 0, 'renew_partial_install_boundary');
  return { installation_verified: false, installation_complete: false, write_verified: true, recovery_only: true,
    predecessor_failure: 'candidate_recovery_missing', historical_preservation_verified: false,
    historical_resource_proof: false, current_effect_authority: false, parity_promotion: false };
}
export async function partialInstallPredecessor(root, firmwareRoot) {
  ignored(firmwareRoot, root); await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json'); check(seal.sha256 === PARTIAL_INSTALL_SEAL, 'renew_partial_install_seal');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const context = await proof(root, 'context.json'), result = (await proof(root, 'result.json')).value;
  const review = validatePartialInstall(context.value, result);
  const installed = await inspectInstall(root, context.value);
  const audit = await proof(root, 'native-audit.json');
  check(audit.sha256 === context.value.nativeAuditSha256, 'renew_partial_install_audit');
  validateNativeAudit(audit.value, context.value.app_elf_sha256, true);
  for (const name of ['start', 'renew']) {
    const measured = await proof(root, `native-${name}-audit.json`);
    check(measured.sha256 === context.value.signedPathAudits?.[name] && measured.value.elf_sha256 === context.value.app_elf_sha256 &&
      measured.value.result === 'selected_path_with_headroom', 'renew_partial_install_stack_audit');
  }
  return { kind: 'partial_verified_write', root, context: context.value, review, audit, seal_sha256: seal.sha256,
    context_sha256: context.sha256, claim_sha256: (await proof(root, 'install-claim.json')).sha256,
    runner_sha256: (await proof(root, 'install-runner.json')).sha256, receipt_sha256: installed.flash_receipt_sha256,
    log_sha256: installed.log_sha256, stdout_sha256: null };
}

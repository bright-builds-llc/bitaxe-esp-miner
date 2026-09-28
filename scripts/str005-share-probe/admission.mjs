import { isDeepStrictEqual as same } from 'node:util';
import { proof } from '../str005-noise-serial/files.mjs';
import { installedPredecessor } from '../str005-panic-probe/installed-predecessor.mjs';
import { captureEvidence, sealed } from '../str005-startup-probe/capture.mjs';
import { verifyClear } from '../str005-startup-probe/clear.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { verifiedIdleRecovery } from '../str005-panic-probe/verified-idle-recovery.mjs';
import { STARTUP_SEAL } from './contract.mjs';
/** Existing sealed producers establish lineage; no aggregate flag substitutes for their evidence. */
export async function admittedImage(firmwareRoot, input) {
  object(input, ['schema', 'installationRoot', 'installationSeal', 'startupRoot', 'captureBindings', 'captureSeal', 'clearRoot', 'clearSeal',
    'recoveryRoot', 'recoveryRelative', 'statusRelative', 'recoverySeal', 'gateCommit']);
  check(input.schema === 'str005-share-admission-v1' && /^[a-f0-9]{40}$/u.test(input.gateCommit), 'share_admission_schema');
  const installed = await installedPredecessor(input.installationRoot, firmwareRoot);
  check(installed.seal_sha256 === input.installationSeal && installed.context.renewSuccessor === true &&
    installed.context.ownerTask === 'task-str005-v2-accepted-share-probe', 'share_renew_image_required');
  check(await sealed(input.startupRoot) === STARTUP_SEAL && (await proof(input.startupRoot, 'result.json')).value.complete === true, 'share_startup_prerequisite');
  const captured = await captureEvidence(firmwareRoot, input.captureBindings, input.captureSeal);
  check(captured.identity.firmware_commit === installed.context.firmware_commit && captured.identity.app_elf_sha256 === installed.context.app_elf_sha256 &&
    captured.physical === installed.context.detector.physical && captured.identity.gate_commit === input.gateCommit, 'share_installed_capture_join');
  check(await sealed(input.clearRoot) === input.clearSeal, 'share_clear_seal');
  const clearContext = (await proof(input.clearRoot, 'context.json')).value;
  check(clearContext.firmware_commit === captured.identity.firmware_commit && clearContext.app_elf_sha256 === captured.identity.app_elf_sha256 &&
    clearContext.physical === captured.physical && clearContext.archiveSha === captured.archiveSha, 'share_clear_identity');
  await verifyClear(input.clearRoot, clearContext);
  check(await sealed(input.recoveryRoot) === input.recoverySeal, 'share_recovery_seal');
  check(input.recoveryRelative === 'current-recovery.json' && input.statusRelative === 'baseline-status.json', 'share_recovery_paths');
  const { proof: recovery, status } = await verifiedIdleRecovery(input.recoveryRoot, captured.identity, captured.physical);
  check(recovery.observed_at_unix_ms >= (await proof(input.clearRoot, 'clear-exit.json')).value.completedAtUnixMs &&
    same(recovery.ledger, captured.recovery.ledger) && same(recovery.original_budget, captured.recovery.original_budget), 'share_postclear_recovery');
  return { ...captured, recovery, expectedBootOrdinal: status.observation.bootOrdinal, admission: input };
}

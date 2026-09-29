import { assessCapture } from './capture-proof.mjs';
import { resolve } from 'node:path';
import { writeNew } from '../str005-noise-serial/files.mjs';
import { proof } from '../str005-noise-serial/files.mjs';
import { inspectInstallArtifacts } from '../str005-panic-probe/install.mjs';
import { sealProbeResult } from '../str005-panic-probe/candidate-failure.mjs';
/** A successful before baseline is not a completed installation stage. */
export async function finalizeDiagnostic(root, context, derived, operations = {}) {
  const result = { ...derived, blockers: [...derived.blockers] };
  if (context.stage === 'installation') {
    let installationVerified = false, maybeBlocker;
    try { await proof(root, 'install-claim.json'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; maybeBlocker = 'diagnostic_installation_not_attempted'; }
    if (!maybeBlocker) {
      try { await inspectInstallArtifacts(root, context); installationVerified = true; }
      catch { maybeBlocker = 'diagnostic_installation_unverified'; }
    }
    if (!maybeBlocker && !(derived.installation_complete === true && derived.candidate_install_reviewed === true &&
      Array.isArray(derived.candidate_recoveries) && derived.candidate_recoveries.length > 0 &&
      derived.candidate_recoveries.every(row => row.complete === true && row.blockers?.length === 0))) maybeBlocker = 'diagnostic_candidate_recovery_unverified';
    if (maybeBlocker) {
      result.installation_blocker = maybeBlocker;
      result.complete = false; result.installation_complete = installationVerified && derived.installation_complete === true;
      if (!result.blockers.includes(maybeBlocker)) result.blockers.push(maybeBlocker);
    }
  }
  if (context.stage === 'capture') {
    const capture = await (operations.assessCapture ?? assessCapture)(root, context, operations);
    result.capture_validation = capture; result.core_capture_verified = capture.complete;
    result.capture_first_failure = capture.first_failure;
    if (!capture.complete) {
      result.complete = false;
      for (const failure of capture.failures) result.blockers.push(`diagnostic_capture_${failure.phase}_${failure.category}`);
    }
    if (!capture.decoder_released) {
      await writeNew(resolve(root, 'capture-unsealed-result.json'), result);
      throw Object.assign(Error('diagnostic_decoder_release_unproven'), { code: 'diagnostic_decoder_release_unproven' });
    }
  }
  return sealProbeResult(root, result);
}

import { proof } from '../str005-noise-serial/files.mjs';
import { inspectInstallArtifacts } from '../str005-panic-probe/install.mjs';
import { sealProbeResult } from '../str005-panic-probe/candidate-failure.mjs';
/** A successful before baseline is not a completed installation stage. */
export async function finalizeDiagnostic(root, context, derived) {
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
  return sealProbeResult(root, result);
}

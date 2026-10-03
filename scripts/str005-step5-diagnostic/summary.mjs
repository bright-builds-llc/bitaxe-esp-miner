import { optionalProof } from '../str005-startup-probe/finish.mjs';
import { validateSafetyDiagnostic } from '../str005-v2-serial/safety-diagnostics.mjs';
import { fixtureFacts } from './fixture-close.mjs';

/** Closed step-5 conclusion; the judge's completion is unchanged and parity is never promoted. */
export function step5Summary(parts, result, clientFailure) {
  const rows = (parts.recovery?.diagnostics?.observations ?? []).filter(row => row.category !== 'boot').map(validateSafetyDiagnostic);
  const receipt = rows.find(row => row.category === 'worker_preparation_receipt' && row.origin === 'current_boot' && row.status === 'valid') ?? null;
  const detail = rows.find(row => row.category === 'worker_revocation_detail' && receipt && row.generation === receipt.generation) ?? null;
  const outcome = result.complete ? 'step5_passed' : detail ? 'revocation_diagnosed' : 'unverified';
  return { step5: { schema: 'str005-step5-summary-v1', outcome, revocation_detail: detail, preparation_receipt: receipt,
    worker_rejection: clientFailure?.rejection ?? null,
    control_failures: rows.filter(row => row.category === 'control_failure').map(row => row.error),
    cause_proven: false, parity_promotion: false } };
}

/** A malformed summary input leaves the sealed result truthful rather than blocking the seal. */
export async function extendResult(root, parts, result) {
  try {
    const summary = step5Summary(parts, result, await optionalProof(root, 'client-failure.json'));
    summary.step5.fixture = await fixtureFacts(root);
    return summary;
  }
  catch { return { step5: { schema: 'str005-step5-summary-v1', outcome: 'unverified', summary_error: 'step5_summary_invalid',
    cause_proven: false, parity_promotion: false } }; }
}

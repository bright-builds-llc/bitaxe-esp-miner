import { resolve } from 'node:path';
import { proof, canonical } from '../str005-noise-serial/files.mjs';
import { inspectInstallArtifacts } from '../str005-panic-probe/install.mjs';
import { validateCandidateState, baselineConclusion, currentProof, BASELINE_PARTS, validateFinished, validatePart } from '../str005-panic-probe/model.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { sealed } from './anchors.mjs';
/** Rejudge exact new installation artifacts without checking historical serial locators. */
export async function installationAnchor(root, repo) {
  const seal = await sealed(root, repo), context = (await proof(root, 'context.json')).value;
  const result = (await proof(root, 'result.json')).value;
  check(context.diagnosticSuccessor === true && context.stage === 'installation' && result.complete && result.installation_complete &&
    result.candidate_install_reviewed && result.baseline_complete && result.host_resources_released && result.blockers.length === 0,
  'diagnostic_installation_anchor');
  await inspectInstallArtifacts(root, context);
  check(Array.isArray(result.candidate_recoveries) && result.candidate_recoveries.length > 0 && result.candidate_recoveries.every(row => row.complete && row.blockers.length === 0), 'diagnostic_installation_recovery');
  const relative = result.candidate_recoveries.at(-1).round;
  check(/^candidate-recovery-[0-9]{3}$/u.test(relative), 'diagnostic_installation_recovery');
  const candidate = { ...context, retainedBaseline: undefined, before_source: context }, parts = {};
  for (const stage of [...BASELINE_PARTS, 'finished']) {
    const value = (await proof(root, `${relative}/${stage}.json`)).value;
    parts[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status :
      stage === 'finished' ? validateFinished(value, context) : ['state', 'closed'].includes(stage) ? validateCandidateState(value, context) : validatePart(stage, value, candidate);
  }
  const begin = (await proof(root, `${relative}/collection-begin.json`)).value;
  const recovered = (await proof(root, `${relative}/current-recovery.json`)).value;
  check(baselineConclusion(parts, candidate).complete && canonical(recovered) === canonical(currentProof(candidate, parts, begin.startedAtUnixMs)), 'diagnostic_installation_proof');
  const before = (await proof(root, 'current-recovery.json')).value;
  check(canonical(before.ledger) === canonical(recovered.ledger) && canonical(before.original_budget) === canonical(recovered.original_budget) &&
    (await proof(root, 'baseline-closed.json')).value.preservation.baseline_id === parts.closed.preservation.baseline_id, 'diagnostic_installation_preservation');
  return { root, seal, context, recovery: recovered };
}

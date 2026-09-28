import { isDeepStrictEqual as same } from 'node:util';
import { proof, privateRoot, verifyInventory } from '../str005-noise-serial/files.mjs';
import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { BASELINE_PARTS, baselineConclusion, currentProof, validatePart, validateFinished } from './model.mjs';
import { check } from '../str005-v2-serial/values.mjs';
/** Validate a successful no-effect successor producer and reconstruct its exact proof. */
export function validateIdleRecovery({ context, result, begin, parts, proof: saved }, identity, physical) {
  check(context.schema === 'str005-panic-probe-v1' && context.renewSuccessor === true && context.recoveryOnly === true &&
    context.ownerTask === 'task-str005-v2-accepted-share-probe' && context.installEnabled === false && context.selfTestEnabled === false &&
    !context.retainedBaseline && context.before_source.firmware_commit === identity.firmware_commit &&
    context.before_source.app_elf_sha256 === identity.app_elf_sha256 && context.gate_commit === identity.gate_commit &&
    context.detector.physical === physical && /^[a-f0-9]{40}$/u.test(context.commit), 'renew_idle_producer');
  check(result.schema === 'str005-panic-baseline-result-v1' && result.complete === true && result.baseline_complete === true &&
    result.host_resources_released === true && result.installation_complete === false && result.continuity_basis === 'current-session-only' &&
    Array.isArray(result.blockers) && result.blockers.length === 0, 'renew_idle_result');
  check(begin.schema === 'str005-renew-baseline-begin-v1' && Number.isSafeInteger(begin.startedAtUnixMs) && begin.startedAtUnixMs >= 0 &&
    saved.observed_at_unix_ms === begin.startedAtUnixMs && baselineConclusion(parts, context).complete &&
    same(saved, currentProof(context, parts, begin.startedAtUnixMs)), 'renew_idle_recomputed');
  check(parts.status.state === 'idle' && parts.status.record === null && Number.isSafeInteger(parts.status.observation.bootOrdinal), 'renew_idle_status');
  return { context, result, begin, parts, proof: saved, status: parts.status };
}
/** Sealed inventories do not make failed producers successful; all baseline inputs are revalidated. */
export async function verifiedIdleRecovery(root, identity, physical) {
  await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const context = (await proof(root, 'context.json')).value, parts = {};
  for (const stage of [...BASELINE_PARTS, 'finished']) {
    const value = (await proof(root, `baseline-${stage}.json`)).value;
    parts[stage] = stage === 'diagnostics' ? await validateDiagnosticExport(value, context.gate_root) :
      stage === 'finished' ? validateFinished(value, context) : stage === 'status' ? validateRecoveryParts({ status: value }, context).status : validatePart(stage, value, context);
  }
  return { ...validateIdleRecovery({ context, parts, result: (await proof(root, 'result.json')).value,
    begin: (await proof(root, 'baseline-begin.json')).value, proof: (await proof(root, 'current-recovery.json')).value }, identity, physical), sealSha256: seal.sha256 };
}

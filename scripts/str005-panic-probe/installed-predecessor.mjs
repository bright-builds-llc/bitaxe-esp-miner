/** A sealed successful update supplies identity lineage, never fresh effect authority. */
import { readdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { ignored } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory, canonical } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { inspectInstall, admitRecovery } from './install.mjs';
import { validateNativeAudit } from './audit.mjs';

export function validateInstalledAnchor(context, result, candidate, baseline, closed) {
  check(result.schema === 'str005-panic-baseline-result-v1' && result.complete === true &&
    result.installation_complete === true && result.candidate_install_reviewed === true &&
    result.host_resources_released === true && result.baseline_complete === true &&
    Array.isArray(result.blockers) && result.blockers.length === 0 &&
    Array.isArray(result.candidate_recoveries) && result.candidate_recoveries.length >= 1 &&
    result.candidate_recoveries.length <= 8 && result.candidate_recoveries.every(row => row.complete === true && row.blockers?.length === 0),
  'panic_installed_anchor_incomplete');
  const expected = { ...context, retainedBaseline: undefined, before_source: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 } };
  admitRecovery(candidate, expected, candidate.observed_at_unix_ms);
  check(canonical(candidate.ledger) === canonical(baseline.ledger) && canonical(candidate.original_budget) === canonical(baseline.original_budget) &&
    closed.status === 'closed' && closed.connected === false && closed.serialOwnershipReleased === true &&
    closed.deviceRestorationConfirmed === true && closed.deviceLeaseInactive === true &&
    closed.expectedFirmwareSourceCommit === context.firmware_commit && closed.expectedAppElfSha256 === context.app_elf_sha256 &&
    closed.preservation?.settings_match === true && closed.preservation.device_identity_match === true &&
    closed.preservation.authorization_high_water_match === true && closed.preservation.mine_on_boot === false,
  'panic_installed_anchor_candidate');
  return { schema: 'str005-installed-anchor-v1', predecessor_installation_verified: true,
    installed_identity: expected.before_source, current_effect_authority: false, historical_resource_proof: false, parity_promotion: false };
}

export async function installedPredecessor(root, firmwareRoot) {
  ignored(firmwareRoot, root); await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
  const saved = await proof(root, 'context.json'), context = saved.value;
  const result = (await proof(root, 'result.json')).value;
  await inspectInstall(root, context);
  const rounds = (await readdir(root)).filter(name => /^candidate-recovery-[0-9]{3}$/u.test(name)).sort();
  check(rounds.length >= 1 && rounds.length <= 8 && rounds.every((name,index) => name === `candidate-recovery-${String(index+1).padStart(3,'0')}`) &&
    JSON.stringify(rounds) === JSON.stringify(result.candidate_recoveries?.map(row => row.round)), 'panic_installed_anchor_rounds');
  const round = rounds.at(-1), candidate = await proof(root, `${round}/current-recovery.json`);
  const baseline = (await proof(root, 'current-recovery.json')).value, closed = (await proof(root, `${round}/closed.json`)).value;
  const beforeClosed = (await proof(root, 'baseline-closed.json')).value;
  check(beforeClosed.preservation?.baseline_id === closed.preservation?.baseline_id, 'panic_installed_anchor_preservation');
  const review = validateInstalledAnchor(context, result, candidate.value, baseline, closed);
  const audit = await proof(root, 'native-audit.json');
  check(audit.sha256 === context.nativeAuditSha256, 'panic_installed_anchor_audit');
  validateNativeAudit(audit.value, context.app_elf_sha256);
  return { kind: 'verified_installation', context, audit, review, root, seal_sha256: seal.sha256,
    context_sha256: saved.sha256, candidate_proof_sha256: candidate.sha256 };
}

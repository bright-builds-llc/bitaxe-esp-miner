import { resolve } from 'node:path';
import { isDeepStrictEqual as equal } from 'node:util';
import { proof } from '../str005-noise-serial/files.mjs';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { requireGone, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { conclusion, restartEvidence, STAGES, FRESH_MS } from '../str005-startup-preparation/model.mjs';
import { PARENT_SEALS } from '../str005-startup-preparation/main.mjs';
import { currentConclusion } from './recovery-main.mjs';
import { RECOVERY_STAGES } from './recovery-server.mjs';
import { sealed } from './capture.mjs';
import { verifyClear } from './clear.mjs';
async function readParts(root, names) {
  return Object.fromEntries(await Promise.all(names.map(async name => [name, (await proof(root, `${name}.json`)).value])));
}
export function validatePreparationTimeline(recoveryStarted, restartStarted, claimedAt) {
  check([recoveryStarted, restartStarted, claimedAt].every(value => Number.isSafeInteger(value) && value >= 0) &&
    restartStarted >= recoveryStarted && claimedAt >= restartStarted &&
    claimedAt - recoveryStarted <= FRESH_MS && claimedAt - restartStarted <= FRESH_MS, 'startup_preparation_stale');
}
/** Independently rejudge the actual preparation children and the erased-core boot lineage. */
export async function preparedEvidence(root) {
  const seal = await sealed(root), context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-startup-preparation-context-v1', 'startup_preparation_schema');
  for (const key of ['startup', 'recovery']) check(await sealed(context.parentRoots[key]) === PARENT_SEALS[key], 'startup_preparation_parent');
  const old = (await proof(context.parentRoots.startup, 'context.json')).value;
  check(context.firmware_commit === old.firmware_commit && context.app_elf_sha256 === old.app_elf_sha256 &&
    context.historical_gate_commit === old.gate_commit && context.physical === old.physical, 'startup_preparation_identity');
  const oldBefore = (await proof(context.parentRoots.startup, 'before.json')).value;
  const oldRecovery = (await proof(context.parentRoots.recovery, 'status.json')).value;
  const oldAdmitted = (await proof(context.parentRoots.startup, 'start-admitted.json')).value;
  check(context.attemptId === oldBefore.attempt.id && context.before_boot_ordinal === oldRecovery.observation.bootOrdinal &&
    context.before_boot_ordinal === oldAdmitted.status.observation.bootOrdinal, 'startup_preparation_boot_lineage');
  await verifyClear(context.parentRoots.startup, old);
  const recoveryRoot = resolve(root, 'recovery'), restartRoot = resolve(root, 'restart');
  const recoverySeal = await sealed(recoveryRoot); await sealed(restartRoot);
  for (const child of [recoveryRoot, restartRoot]) {
    const owner = (await proof(child, 'server-owner.json')).value;
    await requireGone([owner.owner]); requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  }
  const recovery = await readParts(recoveryRoot, [...RECOVERY_STAGES, 'finished']);
  check(currentConclusion(recovery, context, true).current_recovery_complete &&
    recovery.status.observation.bootOrdinal === context.before_boot_ordinal, 'startup_preparation_recovery');
  const restart = await readParts(restartRoot, [...STAGES, 'finished', 'evidence']);
  const claim = (await proof(restartRoot, 'restart-claim.json')).value;
  check(claim.request.expectedBootOrdinal === context.before_boot_ordinal, 'startup_preparation_restart_boot');
  const recoveryResult = (await proof(recoveryRoot, 'result.json')).value;
  const recoveryStarted = (await proof(recoveryRoot, 'collection-begin.json')).value.startedAtUnixMs;
  const restartStarted = (await proof(restartRoot, 'begin.json')).value.startedAtUnixMs;
  check(recoveryResult.current_recovery_complete === true && recoveryResult.startedAtUnixMs === recoveryStarted &&
    claim.beforeStartedAtUnixMs === restartStarted, 'startup_preparation_clock_binding');
  validatePreparationTimeline(recoveryStarted, restartStarted, claim.claimedAtUnixMs);
  restart.evidence = await restartEvidence(restart.evidence, context, claim.request); restart.evidenceVerified = true;
  const derived = conclusion(restart, context, true), retained = (await proof(root, 'result.json')).value;
  check(derived.complete && Object.entries(derived).every(([key, value]) => equal(retained[key], value)) &&
    retained.recoverySealSha256 === recoverySeal && retained.restartEvidenceSha256 === await fileDigest(resolve(restartRoot, 'evidence.json')) &&
    derived.reset_reason === 'software_cpu' && derived.after_boot_ordinal === context.before_boot_ordinal + 1,
  'startup_preparation_incomplete');
  return { seal, context, result: derived, historical: old, coreClearRoot: context.parentRoots.startup };
}

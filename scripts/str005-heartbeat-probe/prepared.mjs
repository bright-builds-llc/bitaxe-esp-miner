import { resolve } from 'node:path';
import { isDeepStrictEqual as equal } from 'node:util';
import { proof } from '../str005-noise-serial/files.mjs';
import { sealed } from '../str005-startup-probe/capture.mjs';
import { inspectPredecessor } from './predecessor.mjs';
import { currentConclusion } from '../str005-startup-probe/recovery-main.mjs';
import { RECOVERY_STAGES } from '../str005-startup-probe/recovery-server.mjs';
import { conclusion, restartEvidence, STAGES } from '../str005-startup-preparation/model.mjs';
import { validatePreparationTimeline } from '../str005-startup-probe/prepared-evidence.mjs';
import { requireGone, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { check } from '../str005-v2-serial/values.mjs';
async function parts(root, names) { return Object.fromEntries(await Promise.all(names.map(async name => [name, (await proof(root, `${name}.json`)).value]))); }
/** Freshness begins at the consumed collection trigger, never at server startup or UI handoff. */
export async function recoveryCollectionStarted(root) {
  const collection = (await proof(root, 'collection-begin.json')).value;
  const result = (await proof(root, 'result.json')).value;
  check(collection.schema === 'str005-recovery-collection-v1' && Number.isSafeInteger(collection.startedAtUnixMs) &&
    collection.startedAtUnixMs >= 0 && result.current_recovery_complete === true && result.startedAtUnixMs === collection.startedAtUnixMs,
  'heartbeat_preparation_collection');
  return collection.startedAtUnixMs;
}
/** No new Start while the previous share's boot-local retained record still occupies the slot. */
export async function inspectPreparation(root) {
  const seal = await sealed(root), context = (await proof(root, 'context.json')).value;
  check(context.schema === 'str005-heartbeat-preparation-context-v1', 'heartbeat_preparation_schema');
  const prior = await inspectPredecessor(context.parentRoots.share);
  check(prior.seal === context.parentSeals.share && prior.context.firmware_commit === context.firmware_commit &&
    prior.context.app_elf_sha256 === context.app_elf_sha256 && prior.context.gate_commit === context.gate_commit &&
    prior.context.physical === context.physical && prior.before.attempt.id === context.attemptId &&
    equal(prior.recovery.ledger, context.expectedLedger) && prior.recovery.status.observation.bootOrdinal === context.before_boot_ordinal,
  'heartbeat_preparation_lineage');
  const recoveredRoot = resolve(root, 'recovery'), restartRoot = resolve(root, 'restart');
  for (const child of [recoveredRoot, restartRoot]) {
    await sealed(child); const owner = (await proof(child, 'server-owner.json')).value;
    await requireGone([owner.owner]); requireLsofAbsent(['-nP', `-iTCP:${owner.port}`, '-sTCP:LISTEN', '-t']);
  }
  const recovered = await parts(recoveredRoot, [...RECOVERY_STAGES, 'finished']);
  check(currentConclusion(recovered, context, true).current_recovery_complete &&
    recovered.status.observation.bootOrdinal === context.before_boot_ordinal, 'heartbeat_preparation_recovery');
  const restarted = await parts(restartRoot, [...STAGES, 'finished', 'evidence']);
  const claim = (await proof(restartRoot, 'restart-claim.json')).value;
  const recoveryStarted = await recoveryCollectionStarted(recoveredRoot);
  const restartStarted = (await proof(restartRoot, 'begin.json')).value.startedAtUnixMs;
  validatePreparationTimeline(recoveryStarted, restartStarted, claim.claimedAtUnixMs);
  check(claim.beforeStartedAtUnixMs === restartStarted && claim.request.expectedBootOrdinal === context.before_boot_ordinal, 'heartbeat_preparation_clock');
  restarted.evidence = await restartEvidence(restarted.evidence, context, claim.request); restarted.evidenceVerified = true;
  const result = conclusion(restarted, context, true), saved = (await proof(root, 'result.json')).value;
  check(result.complete && Object.entries(result).every(([key, value]) => equal(saved[key], value)), 'heartbeat_preparation_incomplete');
  return { seal, context, result, prior };
}

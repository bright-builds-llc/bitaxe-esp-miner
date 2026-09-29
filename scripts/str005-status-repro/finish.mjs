import { readFile, readdir, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fixtureCompletionComplete, fixtureReleaseComplete } from '../str005-startup-probe/server-release.mjs';
import { optionalProof } from '../str005-startup-probe/finish.mjs';
import { normalStopVerified } from '../str005-startup-probe/normal-stop.mjs';
import { validateFailure } from '../str005-startup-probe/failure.mjs';
import { validateRecoveryParts } from '../str005-v2-serial/recovery-evidence.mjs';
import { requireGone, requireNoHolders, requireLsofAbsent, signerExitProofs } from '../str005-v2-serial/host-resources.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { proof, writeNew, inventory } from '../str005-noise-serial/files.mjs';
import { validateRun, LIMITS } from './policy.mjs';

export function judge(parts, context) {
  const blockers = []; let run = null, recoveryTyped = false;
  if (parts.run) try { run = validateRun(parts.run, context); } catch { blockers.push('status_repro_run_invalid'); }
  else blockers.push('status_repro_run_missing');
  const before = parts.before, recovery = parts.recovery ?? {};
  if (!before?.attempt || !before.ledger || !before.original_budget) blockers.push('status_repro_before_missing');
  if (before?.ledger && before.ledger.pending) blockers.push('status_repro_before_pending');
  if (run && !(run.startInvokedAt !== null && run.startRepliedAt !== null && run.stopRequestedAt !== null &&
      run.startRepliedAt >= run.startInvokedAt && run.startRepliedAt - run.startInvokedAt <= LIMITS.replyMs &&
      run.stopRequestedAt >= run.startRepliedAt && run.stopRequestedAt - run.startInvokedAt <= LIMITS.stopFromInvocationMs &&
      run.stopRequestedAt - run.startRepliedAt <= LIMITS.stopFromReplyMs)) blockers.push('status_repro_timing_unproved');
  if (!run?.observedStart || !run.proof || run.firstFailure !== null) blockers.push('status_repro_observation_unproved');
  const observed = parts.statusObservation;
  if (!observed?.status?.record || !before?.attempt?.id || observed.status.record.attemptId !== before.attempt.id ||
      observed.status.record.workerGeneration !== run?.proof?.generation || observed.running !== true ||
      observed.renewalsConfirmed !== 0 || observed.generation !== run?.proof?.generation ||
      observed.status.record.bootOrdinal !== context.expectedBootOrdinal) blockers.push('status_repro_status_unproved');
  try { validateRecoveryParts(recovery, { ...context, attemptId: before?.attempt?.id }); recoveryTyped = true; }
  catch { blockers.push('status_repro_recovery_invalid'); }
  const currentSafe = recoveryTyped && recovery.state?.deviceRestorationConfirmed === true &&
    recovery.state.deviceBaselineConfirmed === true && recovery.state.deviceLeaseInactive === true &&
    recovery.state.connected === true && recovery.state.running === false &&
    recovery.state.preservation?.settings_match === true && recovery.state.preservation?.device_identity_match === true &&
    recovery.state.preservation?.authorization_high_water_match === true &&
    recovery.closed?.status === 'closed' && recovery.closed.connected === false &&
    recovery.closed.deviceRestorationConfirmed === true && recovery.closed.serialOwnershipReleased === true &&
    recovery.closed.preservation?.baseline_id === recovery.state.preservation.baseline_id &&
    recovery.ledger?.pending === false && recovery.original_budget?.pending === false &&
    recovery.status?.scope === 'share' && recovery.diagnostics?.schema === 'str005-recovery-diagnostics-v1' &&
    recovery.diagnostics.observations.some(row => row.category === 'boot' && row.boot_ordinal === recovery.status.observation.bootOrdinal) &&
    recovery.finished?.failures?.length === 0;
  if (!currentSafe) blockers.push('status_repro_current_recovery_incomplete');
  const historicalRetained = Boolean(recoveryTyped && before?.attempt?.id && recovery.status?.record &&
    recovery.status.record.attemptId === before.attempt.id && recovery.status.record.state === 'terminal' &&
    recovery.status.record.resources?.socketClosed === true && recovery.status.record.resources?.workerQuiescent === true &&
    recovery.status.record.resources?.fenceRetained === false);
  if (!historicalRetained) blockers.push('status_repro_retained_attempt_unavailable');
  if (before?.ledger && recovery.ledger && !(recovery.ledger.next_ordinal === before.ledger.next_ordinal + 1 &&
      recovery.ledger.last_completed_ordinal === before.ledger.next_ordinal &&
      recovery.ledger.total_charged_ms === before.ledger.total_charged_ms + 180000 && recovery.ledger.pending === false))
    blockers.push('status_repro_charge_unproven');
  if (run?.proof && !normalStopVerified(recovery.status?.record, recovery.state, run.proof.generation))
    blockers.push('status_repro_normal_stop_unproven');
  if (parts.fixtureStarted && !parts.fixtureCompletion) blockers.push('status_repro_fixture_natural_completion_failed');
  if (!parts.hostReleased) blockers.push('status_repro_host_release_unproven');
  return { schema: 'str005-diagnostic-status-result-v1', complete: blockers.length === 0, blockers,
    current_safe_recovery: currentSafe, historical_retained_proof: historicalRetained,
    qualification_success: blockers.length === 0, observed_start: Boolean(run?.observedStart), status_observed: Boolean(observed),
    fixture_natural_completion: parts.fixtureCompletion, fixture_resources_released: parts.fixtureReleased,
    host_resources_released: parts.hostReleased, core_capture_verified: false, cause_proven: false, parity_promotion: false };
}

export function lateCompletionUnknown(run, receipt) {
  return run?.lateReplyPending === true && (!receipt || receipt.stopComplete !== true || receipt.closeComplete !== true);
}

/** Actual owner absence permits sealing an incomplete result; a live writer does not. */
export async function finalize(root, context, operations = {}) {
  const parts = { recovery: {}, hostReleased: false };
  for (const [key, file] of [['before','before.json'],['run','run.json'],['statusObservation','status-observation.json']])
    parts[key] = await optionalProof(root, file);
  parts.lateCompletion = await optionalProof(root, 'late-completion.json');
  const server = await optionalProof(root, 'server-owner.json'), claim = await optionalProof(root, 'serve-claim.json');
  const fixtureClaim = await optionalProof(root, 'fixture-start.claim.json'), fixtureOwner = await optionalProof(root, 'fixture-owner.json');
  const release = await optionalProof(root, 'fixture-release.json'), completion = await optionalProof(root, 'fixture-completion.json');
  parts.fixtureStarted = Boolean(fixtureClaim);
  parts.fixtureReleased = release ? fixtureReleaseComplete(release) : !fixtureClaim;
  parts.fixtureCompletion = completion ? fixtureCompletionComplete(completion) : false;
  const cleanupBlockers = []; let ownersGone = true, serialReleased = true;
  if (fixtureClaim && !fixtureOwner) { ownersGone = false; cleanupBlockers.push('status_repro_fixture_owner_unknown'); }
  try {
    const owners = [];
    if (server?.owner) owners.push(server.owner); else if (claim?.owner) owners.push(claim.owner);
    if (fixtureOwner?.owner) owners.push(fixtureOwner.owner);
    if (owners.length) await (operations.requireGone ?? requireGone)(owners);
    if (server) (operations.requireLsofAbsent ?? requireLsofAbsent)(['-nP', `-iTCP:${server.port}`, '-sTCP:LISTEN', '-t']);
    await signerExitProofs(root, context);
  } catch { ownersGone = false; cleanupBlockers.push('status_repro_owner_release_unproven'); }
  try {
    const path = resolve(dirname(root), 'final-detector.stdout.log');
    const detected = parseDetector(await readFile(path, 'utf8'), context.physical, Date.now() - (await stat(path)).mtimeMs);
    for (const port of new Set([context.detector.port, detected.port])) (operations.requireNoHolders ?? requireNoHolders)(port);
  } catch { serialReleased = false; cleanupBlockers.push('status_repro_serial_release_unproven'); }
  if (fixtureClaim && (!fixtureOwner || !await optionalProof(root, 'fixture-exit.json') ||
      !await optionalProof(root, 'fixture-reap.json') || !parts.fixtureReleased)) cleanupBlockers.push('status_repro_fixture_release_unproven');
  parts.hostReleased = ownersGone && serialReleased && (!fixtureClaim || parts.fixtureReleased && !cleanupBlockers.includes('status_repro_fixture_release_unproven'));
  const rounds = (await readdir(root)).filter(name => /^recovery-[1-4]-session\.json$/u.test(name)).sort();
  const round = rounds.at(-1)?.split('-')[1];
  if (round) for (const stage of ['state','ledger','original_budget','status','diagnostics','closed','finished']) {
    const value = await optionalProof(root, `recovery-${round}-${stage}.json`);
    if (value !== undefined) parts.recovery[stage] = value;
  }
  const result = judge(parts, context);
  result.late_completion = parts.lateCompletion ?? null;
  if (lateCompletionUnknown(parts.run, parts.lateCompletion)) {
    result.blockers.push('status_repro_late_completion_unknown'); result.complete = false;
  }
  result.blockers.push(...cleanupBlockers); result.complete &&= cleanupBlockers.length === 0;
  result.qualification_success = result.complete;
  const ownerFailure = await optionalProof(root, 'first-failure.json');
  result.owner_first_failure = ownerFailure ? validateFailure(ownerFailure) : null;
  const clientFailure = await optionalProof(root, 'client-failure.json');
  result.client_first_failure = clientFailure ? { phase: clientFailure.phase, category: clientFailure.category } : null;
  let ownerWasFirst = Boolean(ownerFailure);
  if (ownerFailure && clientFailure) {
    const [ownerFile, clientFile] = await Promise.all([stat(resolve(root, 'first-failure.json')), stat(resolve(root, 'client-failure.json'))]);
    ownerWasFirst = ownerFile.mtimeMs <= clientFile.mtimeMs;
  }
  result.first_failure = ownerWasFirst ? { phase: ownerFailure.phase, category: ownerFailure.category } : result.client_first_failure;
  if (ownerFailure) { result.blockers.push('status_repro_owner_failed'); result.complete = false; result.qualification_success = false; }
  result.cleanup_failures = cleanupBlockers;
  if (!ownersGone) { await writeNew(resolve(root, `finish-blocker-${Date.now()}.json`), result); return { ...result, sealed: false }; }
  await writeNew(resolve(root, 'result.json'), result);
  await writeNew(resolve(root, 'sealed-inventory.json'), { files: await inventory(root) });
  return { ...result, sealed: true };
}

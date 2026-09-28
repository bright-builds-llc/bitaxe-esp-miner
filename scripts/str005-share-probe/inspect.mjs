import { readdir } from 'node:fs/promises';
import { isDeepStrictEqual as same } from 'node:util';
import { proof } from '../str005-noise-serial/files.mjs';
import { inspectLiveSelection, completedShares } from '../str005-v2-serial/execution-selection.mjs';
import { validateRecordProgress } from '../str005-v2-serial/device.mjs';
import { judgeFixture } from '../str005-v2-serial/protocol-fixture.mjs';
import { judgeShares } from '../str005-v2-serial/protocol-shares.mjs';
import { decodeJob } from '../str005-v2-serial/job-proof.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
/** Recompute target and native/fixture ACK proof from retained bytes, independent of browser success. */
export async function inspectShare(root, context, parts) {
  const issued = (await proof(root, 'issued.json')).value;
  check(issued.attemptId === parts.before?.attempt?.id && issued.authorizationCount === 3 && issued.renewalCount === 2, 'share_issuance_bound');
  const saved = await proof(root, 'share-selection.json'), selection = saved.value;
  check(selection.schema === 'str005-share-selection-v1' && selection.contextSha256 === sha256(JSON.stringify(context)) &&
    saved.sha256 === parts.run?.proof?.selectionSha256, 'share_selection_binding');
  const names = (await readdir(root)).filter(name => /^share-observation-[0-9]{3}\.json$/u.test(name)).sort();
  check(names.length > 0 && names.length <= 400 && names.length === selection.sequence, 'share_observation_membership');
  let previous, previousAt = 0, renewals = 0;
  for (const [index, name] of names.entries()) {
    check(name === `share-observation-${String(index + 1).padStart(3, '0')}.json`, 'share_observation_gap');
    const row = (await proof(root, name)).value;
    check(row.atHostMs >= previousAt && row.state.running && !row.state.failure && !row.state.heartbeatSuppressed &&
      Number.isInteger(row.state.renewalsConfirmed) && row.state.renewalsConfirmed >= renewals && row.state.renewalsConfirmed <= 2 &&
      row.record.attemptId === parts.before.attempt.id && row.record.workerGeneration === parts.run.proof.generation &&
      row.state.qualification?.generation === row.record.workerGeneration, 'share_observation_state');
    if (previous) validateRecordProgress(previous, row.record);
    previous = row.record; previousAt = row.atHostMs; renewals = row.state.renewalsConfirmed;
  }
  check(same(previous, selection.record) && selection.atHostMs >= previousAt && renewals <= parts.run.proof.renewalsConfirmed, 'share_selection_origin');
  const selected = await inspectLiveSelection(root, { ...context, attemptId: parts.before.attempt.id }, selection.record);
  check(selected && same(selected.fact, selection.fact) && selected.fixtureShareSha256 === selection.fixtureShareSha256 &&
    selected.fixtureShareFile === selection.fixtureShareFile && selected.jobSha256 === selection.jobSha256, 'share_selection_recomputed');
  const terminal = parts.recovery.status?.record;
  check(terminal, 'share_terminal_missing'); validateRecordProgress(selection.record, terminal);
  const input = { scope: 'share' };
  for (const [key, name] of Object.entries({ job: 'job.json', fixtureEvents: 'fixture-events.json', fixtureTerminal: 'fixture-terminal.json',
    fixtureConnectionFacts: 'connection-facts.json', fixtureShares: 'shares.json' })) input[key] = (await proof(root, `fixture-run/${name}`)).value;
  input.connectionComparison = (await proof(root, 'share-connection.json')).value;
  check(input.connectionComparison.readinessSha256 === (await proof(root, 'fixture-ready.json')).sha256, 'share_ready_binding');
  const rows = await completedShares(root);
  check(rows.length === input.fixtureShares.shares.length && rows.every((row, index) => same(row.share, input.fixtureShares.shares[index])), 'share_incremental_join');
  const fixture = judgeFixture(input, terminal, decodeJob(input.job));
  const shares = judgeShares(terminal, input.job, fixture);
  check(shares.acknowledged > 0 && shares.selected.fact.submissionSequence === selected.fact.submissionSequence, 'share_first_ack_join');
  return { verified: true, acknowledged: shares.acknowledged, submitted: shares.submitted, selectionSha256: saved.sha256 };
}

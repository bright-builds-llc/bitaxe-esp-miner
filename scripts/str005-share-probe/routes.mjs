import { proof } from '../str005-noise-serial/files.mjs';
import { parseStatus, validateRecordProgress } from '../str005-v2-serial/device.mjs';
import { compareConnection } from '../str005-v2-serial/fixture.mjs';
import { inspectLiveSelection } from '../str005-v2-serial/execution-selection.mjs';
import { validateState } from '../fixed-usb-qualification/judge.mjs';
import { check, object, sha256 } from '../str005-v2-serial/values.mjs';
import { signWindow } from './signing.mjs';
import { validateRun } from './evidence.mjs';
/** Persist only redacted native records and safe tuple-comparison facts. */
export function routePolicy(root, now = Date.now) {
  let sequence = 0, previous, selected = false, compared = false, renewals = 0;
  return { issueWindow: signWindow, issuanceCounts: { authorizationCount: 3, renewalCount: 2 }, validateRun,
    async extraRoute({ path, input, save, before, fixture, observeReady, context }) {
      if (path !== '/share/observe') return;
      await observeReady(); object(input, ['status', 'state']); check(!selected && fixture && before && sequence < 400, 'share_observation_bound');
      const status = parseStatus(input.status), record = status.record, state = input.state;
      validateState(state, context);
      check(record && record.attemptId === before.attempt.id && record.state !== 'terminal' &&
        record.workerGeneration === state.qualification?.generation && record.bootOrdinal === context.expectedBootOrdinal &&
        state.running && !state.failure && !state.heartbeatSuppressed && Number.isInteger(state.renewalsConfirmed) &&
        state.renewalsConfirmed >= renewals && state.renewalsConfirmed <= 2, 'share_live_binding');
      if (previous) validateRecordProgress(previous, record);
      previous = record; renewals = state.renewalsConfirmed; sequence++;
      await save(`share-observation-${String(sequence).padStart(3, '0')}.json`, { atHostMs: now(), record, state });
      if (!compared && status.connection) {
        const started = now(), facts = compareConnection(status, await fixture.connection(), fixture.ready);
        const readiness = await proof(root, 'fixture-ready.json');
        const comparison = { attemptId: record.attemptId, ...facts,
          ...Object.fromEntries(['bootOrdinal', 'workerGeneration', 'serialTransportEpoch', 'poolSessionGeneration', 'poolTransportEpoch'].map(key => [key, record[key]])),
          expectedPeerMatch: true, expectedPeerCount: 1, unexpectedPeerCount: 0, candidateOverflow: false,
          readinessSha256: readiness.sha256, deviceObservationSequence: sequence, comparisonStartedAtMs: started, comparisonCompletedAtMs: now() };
        await save('share-connection.json', comparison); compared = true;
      }
      const selection = await inspectLiveSelection(root, { ...context, attemptId: before.attempt.id }, record);
      if (!selection) return { handled: true, value: { selected: false } };
      check(compared, 'share_connection_unproven');
      const receipt = { schema: 'str005-share-selection-v1', contextSha256: sha256(JSON.stringify(context)), sequence, atHostMs: now(),
        record: selection.record, fact: selection.fact, jobSha256: selection.jobSha256,
        fixtureShareFile: selection.fixtureShareFile, fixtureShareSha256: selection.fixtureShareSha256 };
      await save('share-selection.json', receipt); selected = true;
      return { handled: true, value: { selected: true, selectionSha256: sha256(`${JSON.stringify(receipt, null, 2)}\n`) } };
    } };
}

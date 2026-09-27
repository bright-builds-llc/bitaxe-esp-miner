import { validateDiagnosticExport } from '../fixed-usb-qualification/diagnostic-export.mjs';
import { check, object, sha256, uint } from '../str005-v2-serial/values.mjs';

/** Keep partial reset evidence, but never interpret a reset alone as core-capture success. */
export async function validateSelfTest(value, gateRoot, request, identity) {
  object(value, ['summary', 'ack', 'observations', 'lifecycle']);
  const s = validateSelfTestSummary(value.summary, request);
  if (value.ack !== null) {
    object(value.ack, ['schema', 'requestNonceSha256', 'bootOrdinal', 'nextBootOrdinal']);
    check(value.ack.schema === 'worker-qualification-core-dump-self-test-v1' && value.ack.requestNonceSha256 === sha256(request.requestNonce) &&
      value.ack.bootOrdinal === request.expectedBootOrdinal && value.ack.nextBootOrdinal === s.nextBootOrdinal, 'panic_self_test_ack');
  }
  check(Array.isArray(value.observations) && value.observations.length <= 512 && Array.isArray(value.lifecycle) && value.lifecycle.length <= 16,
    'panic_self_test_bounds');
  for (const row of value.lifecycle) {
    object(row, ['record', 'atMs', 'event']); uint(row.record, s.records); uint(row.atMs, s.durationMs);
    check(['prearmed', 'acknowledged', 'stream_interrupted', 'same_port_reopened', 'hello_started', 'complete', 'failed'].includes(row.event), 'panic_self_test_lifecycle');
  }
  const observations = [];
  for (let i = 0; i < value.observations.length; i += 40) {
    const rows = value.observations.slice(i, i + 40);
    for (const row of rows) { object(row, ['record', 'atMs', 'diagnostic']); uint(row.record, s.records); uint(row.atMs, s.durationMs); }
    const validated = await validateDiagnosticExport({ schema: 'worker-diagnostic-export-v1', observations: rows.map(row => row.diagnostic) }, gateRoot);
    observations.push(...rows.map((row, index) => ({ ...row, diagnostic: validated.observations[index] })));
  }
  const result = { summary: structuredClone(s), ack: structuredClone(value.ack), observations, lifecycle: structuredClone(value.lifecycle) };
  check((value.ack !== null) === s.ackMatched, 'panic_self_test_ack_presence');
  if (s.stage === 'complete') validateCompleteEvidence(result, request, identity);
  return result;
}

export function validateSelfTestSummary(summary, request) {
  const s = summary;
  object(s, ['schema', 'stage', 'ackMatched', 'expectedBootOrdinal', 'nextBootOrdinal', 'bootObserved', 'runtimeReadyObserved',
    'softwareResetObserved', 'panicResetObserved', 'identityObserved', 'identityMatched', 'records', 'bytes', 'durationMs', 'portReopens', 'streamInterrupted', 'continuity']);
  check(s.schema === 'worker-qualification-core-dump-self-test-observation-v1' && ['armed', 'acknowledged', 'reacquiring', 'complete', 'failed'].includes(s.stage) &&
    s.expectedBootOrdinal === request.expectedBootOrdinal && s.nextBootOrdinal === request.expectedBootOrdinal + 1 &&
    ['ackMatched', 'bootObserved', 'runtimeReadyObserved', 'softwareResetObserved', 'panicResetObserved', 'identityObserved', 'identityMatched', 'streamInterrupted']
      .every(key => typeof s[key] === 'boolean') && s.softwareResetObserved === false && [0, 1].includes(s.portReopens) &&
    ['uninterrupted', 'interrupted', 'same_port_reopened'].includes(s.continuity), 'panic_self_test_summary');
  uint(s.records); uint(s.bytes); uint(s.durationMs);
  if (s.stage === 'complete') check(['ackMatched', 'bootObserved', 'runtimeReadyObserved', 'identityObserved', 'identityMatched', 'panicResetObserved']
    .every(key => s[key] === true) && s.records > 0 && s.records <= 512 && s.bytes >= s.records && s.bytes <= 262144 && s.durationMs <= 30000 &&
    (!s.portReopens || s.streamInterrupted) && s.continuity === (s.portReopens ? 'same_port_reopened' : s.streamInterrupted ? 'interrupted' : 'uninterrupted'), 'panic_self_test_complete_flags');
  return structuredClone(s);
}

/** Reconstruct the claimed successful reset from validated diagnostics and ordered lifecycle. */
export function validateCompleteEvidence(value, request, identity) {
  const s = validateSelfTestSummary(value.summary, request), events = value.lifecycle, observations = value.observations;
  check(identity && /^[0-9a-f]{40}$/u.test(identity.firmware_commit) && /^[0-9a-f]{64}$/u.test(identity.app_elf_sha256), 'panic_self_test_expected_identity');
  check(value.ack !== null && value.ack.requestNonceSha256 === sha256(request.requestNonce) &&
    value.ack.bootOrdinal === request.expectedBootOrdinal && value.ack.nextBootOrdinal === request.expectedBootOrdinal + 1 &&
    value.ack.schema === 'worker-qualification-core-dump-self-test-v1', 'panic_self_test_complete_ack');
  let priorRecord = 0, priorTime = 0;
  for (const event of events) {
    check(event.record >= priorRecord && event.atMs >= priorTime && event.atMs <= s.durationMs, 'panic_self_test_event_order');
    priorRecord = event.record; priorTime = event.atMs;
  }
  const named = name => events.filter(row => row.event === name);
  const ack = named('acknowledged')[0], hello = named('hello_started').at(-1), reopen = named('same_port_reopened')[0];
  check(named('prearmed').length === 1 && events[0]?.event === 'prearmed' && events[0].record === 0 &&
    named('acknowledged').length === 1 && events[1] === ack && ack.record > 0 &&
    named('complete').length === 1 && events.at(-1)?.event === 'complete' && events.at(-1).record === s.records && events.at(-1).atMs === s.durationMs &&
    named('failed').length === 0 && named('stream_interrupted').length === Number(s.streamInterrupted) && named('same_port_reopened').length === s.portReopens &&
    named('hello_started').length >= 1 && named('hello_started').length <= s.portReopens + 1, 'panic_self_test_complete_lifecycle');
  const boundary = reopen ?? ack;
  check(events.indexOf(hello) > events.indexOf(boundary), 'panic_self_test_hello_order');
  if (reopen) check(events.indexOf(reopen) === events.indexOf(named('stream_interrupted')[0]) + 1 && events.indexOf(reopen) > 2,
    'panic_self_test_reopen_order');
  priorRecord = 0; priorTime = 0; let newBootSeen = false;
  for (const row of observations) {
    check(row.record > priorRecord && row.atMs >= priorTime && row.atMs <= s.durationMs, 'panic_self_test_observation_order');
    priorRecord = row.record; priorTime = row.atMs;
    const d = row.diagnostic;
    if (row.record > ack.record && d.category === 'runtime_identity') check(d.firmware_commit === identity.firmware_commit && d.app_elf_sha256 === identity.app_elf_sha256,
      'panic_self_test_identity');
    if (row.record > ack.record && d.category === 'startup') check(d.first_failure === 'none' && d.state !== 'failed', 'panic_self_test_startup_failure');
    if (row.record > ack.record && d.category === 'statistics_startup') check(!['spawn_failed', 'config_failed', 'cancelled'].includes(d.state), 'panic_self_test_statistics_failure');
    if (d.category === 'boot' && row.record > ack.record) {
      if (!newBootSeen && d.boot_ordinal === request.expectedBootOrdinal) continue;
      check(d.boot_ordinal === s.nextBootOrdinal && d.reset_reason === 'panic', 'panic_self_test_boot'); newBootSeen = true;
    }
  }
  const fresh = observations.filter(row => row.record > boundary.record && row.record <= hello.record && row.atMs >= boundary.atMs && row.atMs <= hello.atMs);
  const boot = fresh.find(row => row.diagnostic.category === 'boot' && row.diagnostic.boot_ordinal === s.nextBootOrdinal && row.diagnostic.reset_reason === 'panic');
  check(boot && fresh.some(row => row.diagnostic.category === 'runtime_identity'), 'panic_self_test_fresh_boot_identity');
  const ready = fresh.filter(row => row.record > boot.record && row.diagnostic.category === 'startup' && row.diagnostic.stage === 'runtime_ready' &&
    row.diagnostic.state === 'complete' && row.diagnostic.first_failure === 'none');
  check(ready.length >= 2 && ready.some((row, index) => index > 0 && row.diagnostic.uptime_ms > ready[index - 1].diagnostic.uptime_ms) &&
    ready.every((row, index) => index === 0 || row.diagnostic.uptime_ms >= ready[index - 1].diagnostic.uptime_ms), 'panic_self_test_fresh_readiness');
  if (observations.some(row => row.record > ack.record && row.diagnostic.category === 'statistics_startup' && row.diagnostic.state === 'prepared')) check(
    fresh.some(row => row.record > boot.record && row.diagnostic.category === 'statistics_startup' && row.diagnostic.state === 'active'),
    'panic_self_test_active_statistics');
}

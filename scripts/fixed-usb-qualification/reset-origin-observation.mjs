import { exactObject, hex, QualificationError, requireCondition as check } from './contract.mjs';

export const RESET_ORIGIN_CATEGORIES = Object.freeze(['boot', 'startup', 'runtime_identity', 'panic', 'allocation_failure', 'allocation_context', 'storage_http_status']);
const STAGES = ['early_identity', 'usb_install', 'nvs', 'hardware', 'worker_recovery', 'runtime_services', 'storage_http', 'network', 'worker_control', 'statistics', 'runtime_ready'];
const ALLOCATION_STAGES = ['early_identity', 'hardware', 'runtime_services', 'storage_http', 'network', 'usb_install', 'statistics', 'runtime_ready'];
const RESET_REASONS = ['power_on', 'software_cpu', 'watchdog', 'panic', 'brownout', 'other'];
const integer = (value, maximum = Number.MAX_SAFE_INTEGER) => Number.isSafeInteger(value) && value >= 0 && value <= maximum;
const fields = {
  boot: ['boot_ordinal', 'reset_reason', 'uptime_ms'], startup: ['stage', 'state', 'first_failure', 'uptime_ms'],
  runtime_identity: ['firmware_commit', 'app_elf_sha256'], panic: ['file_hash', 'line'],
  allocation_failure: ['requested_bytes', 'capabilities'], allocation_context: ['requested_bytes', 'capabilities', 'source_hash', 'stage'],
  storage_http_status: ['spiffs_available', 'http_ready'],
};
/** Validates one selected Gate diagnostic before any persistence boundary. */
export function parseResetOriginDiagnostic(value) {
  check(value && RESET_ORIGIN_CATEGORIES.includes(value.category), 'reset_origin_category');
  exactObject(value, ['category', 'authoritative', ...fields[value.category]]);
  check(value.authoritative === false, 'reset_origin_authority');
  if (value.category === 'boot') check(integer(value.boot_ordinal) && value.boot_ordinal > 0 && RESET_REASONS.includes(value.reset_reason) && integer(value.uptime_ms), 'reset_origin_boot');
  if (value.category === 'startup') check(STAGES.includes(value.stage) && ['entered', 'failed', 'complete'].includes(value.state) &&
    ['none', ...STAGES].includes(value.first_failure) && integer(value.uptime_ms) && !(value.state === 'failed' && value.first_failure === 'none'), 'reset_origin_startup');
  if (value.category === 'runtime_identity') check(hex(value.firmware_commit, 40) && hex(value.app_elf_sha256, 64), 'reset_origin_identity');
  if (value.category === 'panic') check(hex(value.file_hash, 8) && integer(value.line, 0xffffffff) && value.line > 0, 'reset_origin_panic');
  if (value.category.startsWith('allocation_')) {
    check(integer(value.requested_bytes, 0xffffffff) && value.requested_bytes > 0 && hex(value.capabilities, 8), 'reset_origin_allocation');
    if (value.category === 'allocation_context') check(hex(value.source_hash, 16) && ALLOCATION_STAGES.includes(value.stage), 'reset_origin_allocation');
  }
  // Gate's closed grammar exports these two values as strings, not JavaScript booleans.
  if (value.category === 'storage_http_status') check(['true', 'false'].includes(value.spiffs_available) && ['true', 'false'].includes(value.http_ready), 'reset_origin_storage');
  return { ...value };
}
function timeline() { return { records: 0, advances: 0, duplicates: 0, regressions: 0, firstUptimeMs: null, lastUptimeMs: null, spanMs: 0 }; }
function advance(target, uptime) {
  target.records++;
  if (target.firstUptimeMs === null) target.firstUptimeMs = uptime;
  if (target.lastUptimeMs !== null) {
    if (uptime > target.lastUptimeMs) target.advances++;
    else if (uptime === target.lastUptimeMs) target.duplicates++;
    else target.regressions++;
  }
  target.lastUptimeMs = uptime;
  target.spanMs = Math.max(0, uptime - target.firstUptimeMs);
}
function maximumGap(rows, start, end) {
  if (!rows.length) return null;
  let last = start, gap = 0;
  for (const row of rows) { gap = Math.max(gap, row.hostMonotonicMs - last); last = row.hostMonotonicMs; }
  return Math.max(gap, end - last);
}
const healthy = value => value.category === 'startup' && value.stage === 'runtime_ready' && value.state === 'complete' && value.first_failure === 'none';

function summarize(records, policy, end, inputIssues) {
  const issues = new Set(inputIssues), segments = [], transitions = [];
  const bootRows = records.filter(row => row.diagnostic.category === 'boot');
  let segment;
  for (const row of bootRows) {
    const d = row.diagnostic;
    if (!segment || segment.bootOrdinal !== d.boot_ordinal) {
      if (segment) {
        const step = d.boot_ordinal - segment.bootOrdinal;
        transitions.push({ fromOrdinal: segment.bootOrdinal, toOrdinal: d.boot_ordinal, ordinalStep: step, atSequence: row.sequence, resetReason: d.reset_reason });
        if (step < 1) issues.add('boot_ordinal_regression');
        if (step > 1) issues.add('missing_boot_ordinals');
      }
      segment = { bootOrdinal: d.boot_ordinal, resetReason: d.reset_reason, firstSequence: row.sequence,
        lastSequence: row.sequence, boot: timeline(), startup: timeline(), healthyStartup: timeline() };
      segments.push(segment);
    } else if (segment.resetReason !== d.reset_reason) issues.add('same_boot_reset_reason_conflict');
    segment.lastSequence = row.sequence; advance(segment.boot, d.uptime_ms);
  }
  let unboundStartupRecords = 0, startupFailures = 0, transitionAssociatedStartupRestarts = 0;
  for (const row of records.filter(row => row.diagnostic.category === 'startup')) {
    const d = row.diagnostic;
    if (d.state === 'failed' || d.first_failure !== 'none') startupFailures++;
    let index = segments.findLastIndex(value => value.firstSequence <= row.sequence);
    if (index < 0) {
      if (segments.length && d.uptime_ms <= segments[0].boot.firstUptimeMs) index = 0;
      else { unboundStartupRecords++; continue; }
    }
    const next = segments[index + 1], current = segments[index];
    // A low startup timestamp between old/new boot markers is compatible with
    // the independently observed transition; it does not attribute the reset cause.
    if (next && row.sequence > current.lastSequence && d.uptime_ms < current.boot.lastUptimeMs && d.uptime_ms <= next.boot.firstUptimeMs) {
      index++; transitionAssociatedStartupRestarts++;
    }
    advance(segments[index].startup, d.uptime_ms);
    if (healthy(d)) advance(segments[index].healthyStartup, d.uptime_ms);
  }
  if (segments.some(value => value.boot.regressions)) issues.add('boot_uptime_regression');
  if (segments.some(value => value.startup.regressions)) issues.add('startup_uptime_regression');
  if (startupFailures) issues.add('startup_failure_observed');
  const identities = records.filter(row => row.diagnostic.category === 'runtime_identity');
  const identityConflicts = identities.filter(({ diagnostic: d }) => d.firmware_commit !== policy.firmwareCommit || d.app_elf_sha256 !== policy.appElfSha256).length;
  if (!identities.length) issues.add('identity_missing');
  if (identityConflicts) issues.add('identity_conflict');
  const healthyRows = records.filter(row => healthy(row.diagnostic));
  const hostSpanMs = Math.max(0, end - policy.startedAtHostMonotonicMs);
  const gaps = { allRecordsMs: maximumGap(records, policy.startedAtHostMonotonicMs, end),
    bootRecordsMs: maximumGap(bootRows, policy.startedAtHostMonotonicMs, end),
    healthyStartupRecordsMs: maximumGap(healthyRows, policy.startedAtHostMonotonicMs, end) };
  if (hostSpanMs < policy.minimumSpanMs) issues.add('host_span_incomplete');
  if (!bootRows.length) issues.add('boot_observations_missing');
  if (!healthyRows.length) issues.add('healthy_startup_missing');
  if (Object.values(gaps).some(value => value === null || value > policy.maximumGapMs)) issues.add('observation_coverage_gap');
  const panicRecords = records.filter(row => row.diagnostic.category === 'panic').length;
  const allocationRecords = records.filter(row => row.diagnostic.category.startsWith('allocation_')).length;
  const storage = records.filter(row => row.diagnostic.category === 'storage_http_status');
  return { schema: 'reset-origin-observation-v1', authoritative: false, hardwareAuthority: false, priorResetAttribution: 'unknown',
    recordCount: records.length, hostSpanMs, minimumSpanMs: policy.minimumSpanMs, maximumGapMs: policy.maximumGapMs, gaps,
    coverageComplete: hostSpanMs >= policy.minimumSpanMs && Object.values(gaps).every(value => value !== null && value <= policy.maximumGapMs) &&
      !issues.has('record_sequence_gap') && !issues.has('host_clock_regression') && !issues.has('invalid_record') && !issues.has('record_bound') && !issues.has('boot_segment_bound'),
    initialBootOrdinal: segments[0]?.bootOrdinal ?? null, initialResetReason: segments[0]?.resetReason ?? null,
    observedTransitionCount: transitions.length, transitions, segments, unboundStartupRecords, transitionAssociatedStartupRestarts,
    identityRecords: identities.length, identityConflicts, startupFailureRecords: startupFailures,
    healthyStartupRecords: healthyRows.length, bootAdvances: segments.reduce((sum,value)=>sum+value.boot.advances,0),
    healthyStartupAdvances: segments.reduce((sum,value)=>sum+value.healthyStartup.advances,0),
    unattributedPanicReceiptRecords: panicRecords, unattributedAllocationReceiptRecords: allocationRecords,
    storageReadyRecords: storage.filter(row=>row.diagnostic.spiffs_available === 'true' && row.diagnostic.http_ready === 'true').length,
    storageUnavailableRecords: storage.filter(row=>row.diagnostic.spiffs_available === 'false' || row.diagnostic.http_ready === 'false').length,
    issues: [...issues] };
}

/** Bounded diagnostic reducer. Authentication, minimum advance counts and restart acceptance belong to the caller's separate contract. */
export function createResetOriginObservation(policy) {
  exactObject(policy, ['firmwareCommit', 'appElfSha256', 'minimumSpanMs', 'maximumGapMs', 'startedAtHostMonotonicMs']);
  check(hex(policy.firmwareCommit,40) && hex(policy.appElfSha256,64) && integer(policy.minimumSpanMs) && policy.minimumSpanMs > 0 &&
    integer(policy.maximumGapMs) && policy.maximumGapMs > 0 && integer(policy.startedAtHostMonotonicMs), 'reset_origin_policy');
  policy = { ...policy };
  const records = [], issues = new Set(); let lastHost = policy.startedAtHostMonotonicMs, lastOrdinal, segmentCount = 0, finished;
  function observe(input) {
    check(!finished, 'reset_origin_finished');
    try {
      exactObject(input, ['sequence', 'hostMonotonicMs', 'diagnostic']);
      check(integer(input.sequence) && input.sequence > 0 && integer(input.hostMonotonicMs), 'reset_origin_record');
      check(records.length < 4096, 'reset_origin_record_bound');
      const value = parseResetOriginDiagnostic(input.diagnostic);
      if (value.category === 'boot' && lastOrdinal !== value.boot_ordinal) {
        check(++segmentCount <= 8, 'reset_origin_boot_segment_bound'); lastOrdinal = value.boot_ordinal;
      }
      if (input.sequence !== records.length + 1) issues.add('record_sequence_gap');
      if (input.hostMonotonicMs < lastHost) issues.add('host_clock_regression');
      lastHost = input.hostMonotonicMs;
      records.push({ sequence: input.sequence, hostMonotonicMs: input.hostMonotonicMs, diagnostic: value });
    } catch (error) {
      issues.add(error.code === 'reset_origin_record_bound' ? 'record_bound' : error.code === 'reset_origin_boot_segment_bound' ? 'boot_segment_bound' : 'invalid_record');
      throw new QualificationError(error instanceof QualificationError ? error.code : 'reset_origin_record');
    }
  }
  function finish(input) {
    exactObject(input, ['hostMonotonicMs']);
    const { hostMonotonicMs } = input;
    check(integer(hostMonotonicMs), 'reset_origin_finish');
    if (finished) return structuredClone(finished);
    if (hostMonotonicMs < lastHost || hostMonotonicMs < policy.startedAtHostMonotonicMs) issues.add('host_clock_regression');
    finished = summarize(records, policy, hostMonotonicMs, issues);
    return structuredClone(finished);
  }
  return { observe, finish };
}

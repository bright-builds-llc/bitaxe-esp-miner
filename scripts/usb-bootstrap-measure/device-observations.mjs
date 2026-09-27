const categories = new Set(["bootstrap_diagnostic", "diagnostic", "diagnostic_replay", "hello", "heartbeat", "receive_credit", "control_reply", "resynchronization"]);
const kinds = new Set(["startup_progress", "statistics_startup", "boot_identity", "admission", "retained_diagnostic", "tx_observation", "protocol", "resynchronization"]);
const stages = new Set(["completed", "write", "write_timeout", "flush_timeout", "cancelled"]);
const counters = ["actual_failures", "replay_attempts", "replay_completed"];
const unsigned = ["start_ms", "end_ms", "record_bytes", "queued_bytes", "queue_calls", "queue_positive", "queue_zero", "drain_calls", "drain_timeout", "drain_success", "drain_other", "first_drain_ms", "last_drain_ms", "max_drain_ms", ...counters, "flags"];
const signed = ["last_queue_return", "last_drain_return"];
const fields = ["schema", "slot", "category", "record_kind", "outcome", "stage", ...unsigned, ...signed, "redacted"];
const invariant = (condition, code) => { if (!condition) throw Object.assign(Error(code), { code }); };
const number = (value, signedValue = false) => {
  invariant(typeof value === "string" && (signedValue ? /^(?:0|-?[1-9][0-9]*)$/u : /^(?:0|[1-9][0-9]*)$/u).test(value), "integer");
  const n = Number(value); invariant(Number.isSafeInteger(n) && (!signedValue || n >= -2147483648 && n <= 2147483647), "integer_range"); return n;
};
function tokens(line, prefix) {
  const result = {};
  for (const item of line.slice(prefix.length).trim().split(/\s+/u)) {
    const match = /^([a-z_][a-z_0-9]*)=([^=\s]+)$/u.exec(item);
    invariant(match && !Object.hasOwn(result, match[1]), "fields"); result[match[1]] = match[2];
  }
  return result;
}
function observation(line) {
  invariant(Buffer.byteLength(line) <= 1024, "length");
  const value = tokens(line, "usb_tx_observation");
  invariant(Object.keys(value).length === fields.length && fields.every(key => Object.hasOwn(value, key)), "fields");
  invariant(value.schema === "v1" && value.redacted === "true" && ["first_bootstrap", "first_failure"].includes(value.slot), "schema");
  invariant(categories.has(value.category) && kinds.has(value.record_kind) && stages.has(value.stage) && ["completed", "failed"].includes(value.outcome), "enum");
  for (const key of unsigned) value[key] = number(value[key]);
  for (const key of signed) value[key] = number(value[key], true);
  invariant(unsigned.filter(key => !key.endsWith("_ms")).every(key => value[key] <= 0xffffffff) && value.flags <= 7, "range");
  invariant(value.record_bytes > 0 && value.record_bytes <= 66560, "record_size");
  invariant(value.outcome === "completed" ? value.stage === "completed" : value.stage !== "completed", "outcome");
  invariant(value.outcome !== "failed" || value.actual_failures > 0, "failure_count");
  invariant(value.slot !== "first_bootstrap" || value.category === "bootstrap_diagnostic", "bootstrap_category");
  invariant(value.slot !== "first_failure" || value.outcome === "failed" && value.actual_failures > 0, "failure_slot");
  invariant(value.replay_completed <= value.replay_attempts, "replay_counts");
  if (value.flags === 0) {
    invariant(value.queued_bytes <= value.record_bytes && value.start_ms <= value.end_ms, "timing_or_queue");
    invariant(value.outcome !== "completed" || value.end_ms - value.start_ms <= 2000, "completed_after_deadline");
    invariant(value.queue_calls > 0 || ["cancelled", "write_timeout"].includes(value.stage) && value.queued_bytes === 0 && value.drain_calls === 0 && value.last_queue_return === 0, "empty_queue");
    invariant(value.queue_positive + value.queue_zero <= value.queue_calls && value.queue_calls - value.queue_positive - value.queue_zero <= 1, "queue_counts");
    invariant(value.drain_calls === value.drain_timeout + value.drain_success + value.drain_other, "drain_counts");
    if (value.drain_calls === 0) invariant(value.first_drain_ms === 0 && value.last_drain_ms === 0 && value.max_drain_ms === 0, "absent_drain");
    else invariant(value.start_ms <= value.first_drain_ms && value.first_drain_ms <= value.last_drain_ms && value.last_drain_ms <= value.end_ms && value.max_drain_ms <= value.end_ms - value.start_ms, "drain_timing");
    if (["completed", "flush_timeout"].includes(value.stage)) invariant(value.queued_bytes === value.record_bytes && value.drain_calls > 0, "full_queue");
    if (value.outcome === "completed") invariant(value.drain_success > 0 && value.last_drain_return === 0, "completion");
  }
  return value;
}
function stable(value) {
  return JSON.stringify(Object.fromEntries(fields.filter(key => ![...counters, "flags"].includes(key)).map(key => [key, value[key]])));
}
/** Parse only allowlisted measurements; raw log text never appears in returned diagnostics. */
export function parseDeviceObservations(text, { firmwareCommit, appElfSha256 }) {
  invariant(/^[a-f0-9]{40}$/u.test(firmwareCommit) && /^[a-f0-9]{64}$/u.test(appElfSha256), "expected_identity");
  invariant(typeof text === "string" && Buffer.byteLength(text) <= 16 * 1024 * 1024, "log_size");
  const issues = new Set(), slots = new Map(), bootIds = new Set();
  let observationCount = 0, identityCount = 0, bootCount = 0, lastBootUptime = -1, lastStartupUptime = -1, startupComplete = false, legacyFailureObserved = false;
  let totals = { actual_failures: 0, replay_attempts: 0, replay_completed: 0 };
  const truncatedTail = text.length > 0 && !text.endsWith("\n"), lines = text.split("\n");
  if (truncatedTail) {
    const tail = lines.pop().replace(/\x1b\[[0-9;]*m/gu, "").trim();
    if (/^usb_(?:tx_observation|tx_failure|runtime_identity|reboot_discriminator|startup)(?:\s|$)/u.test(tail)) issues.add("truncated_device_record");
  }
  for (const raw of lines) {
    const line = raw.replace(/\x1b\[[0-9;]*m/gu, "").trim();
    try {
      if (line.startsWith("usb_runtime_identity ")) {
        const v = tokens(line, "usb_runtime_identity");
        invariant(v.schema === "v1" && v.redacted === "true" && v.firmware_commit === firmwareCommit && v.app_elf_sha256 === appElfSha256, "identity"); identityCount++;
      } else if (line.startsWith("usb_reboot_discriminator ")) {
        const v = tokens(line, "usb_reboot_discriminator"), uptime = number(v.uptime_ms), boot = number(v.boot_ordinal);
        invariant(v.schema === "v1" && v.redacted === "true" && boot > 0 && uptime >= lastBootUptime, "boot");
        bootIds.add(boot); lastBootUptime = uptime; bootCount++;
      } else if (line.startsWith("usb_startup ")) {
        const v = tokens(line, "usb_startup"), uptime = number(v.uptime_ms);
        invariant(v.schema === "v1" && v.redacted === "true" && uptime >= lastStartupUptime, "startup");
        lastStartupUptime = uptime;
        if (v.stage === "runtime_ready" && v.state === "complete" && v.first_failure === "none") startupComplete = true;
      } else if (line.startsWith("usb_tx_failure")) legacyFailureObserved = true;
      else if (line.startsWith("usb_tx_observation")) {
        const value = observation(line), previous = slots.get(value.slot);
        invariant(!previous || stable(previous) === stable(value), "retained_observation_changed");
        invariant(counters.every(key => value[key] >= totals[key]), "counter_regression");
        totals = Object.fromEntries(counters.map(key => [key, value[key]]));
        if (value.flags) issues.add("measurement_flags");
        slots.set(value.slot, value); observationCount++;
      }
    } catch (error) { issues.add("device_" + (error.code ?? "record")); }
  }
  if (identityCount === 0) issues.add("identity_missing");
  if (bootCount < 2 || bootIds.size !== 1) issues.add("stable_boot_missing");
  if (lastStartupUptime < 0) issues.add("startup_missing");
  const firstBootstrap = slots.get("first_bootstrap") ?? null, firstFailure = slots.get("first_failure") ?? null;
  if (!firstBootstrap) issues.add("first_bootstrap_missing");
  if ((legacyFailureObserved || totals.actual_failures > 0) && !firstFailure) issues.add("first_failure_missing");
  for (const value of slots.values()) if (value.end_ms > Math.max(lastBootUptime, lastStartupUptime)) issues.add("observation_outside_boot_stream");
  return { schema: "usb-bootstrap-device-observations-v1", firmwareCommit, appElfSha256,
    bootOrdinal: bootIds.size === 1 ? [...bootIds][0] : null, firstBootstrap, firstFailure,
    counters: { actualFailures: totals.actual_failures, replayAttempts: totals.replay_attempts, replayCompleted: totals.replay_completed },
    observationCount, startupComplete, legacyFailureObserved, truncatedTail, complete: issues.size === 0, issues: [...issues].sort() };
}

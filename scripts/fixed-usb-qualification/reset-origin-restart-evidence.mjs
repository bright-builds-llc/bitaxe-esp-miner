import { digest, exactObject, requireCondition as check } from "./contract.mjs";
import { parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";
const integer = (value, maximum) => Number.isSafeInteger(value) && value >= 0 && value <= maximum;
const SUMMARY = [
  "schema",
  "stage",
  "ackMatched",
  "expectedBootOrdinal",
  "nextBootOrdinal",
  "bootObserved",
  "softwareResetObserved",
  "identityObserved",
  "identityMatched",
  "runtimeReadyObserved",
  "records",
  "bytes",
  "durationMs",
  "portReopens",
  "streamInterrupted",
  "continuity",
];
/** Validates and strips nonessential diagnostics before persistence; no raw bytes or nonce are retained. */
export function validateRestartEvidence(value, context, expectedBootOrdinal) {
  exactObject(value, ["summary", "ack", "observations", "lifecycle"]);
  const s = value.summary;
  exactObject(s, SUMMARY);
  check(
    s.schema === "worker-qualification-restart-observation-v1" &&
      s.stage === "complete" &&
      ["ackMatched", "bootObserved", "softwareResetObserved", "identityObserved", "identityMatched", "runtimeReadyObserved"].every(
        (key) => s[key] === true,
      ) &&
      Number.isSafeInteger(expectedBootOrdinal) &&
      expectedBootOrdinal > 0 &&
      expectedBootOrdinal < Number.MAX_SAFE_INTEGER &&
      s.expectedBootOrdinal === expectedBootOrdinal &&
      s.nextBootOrdinal === expectedBootOrdinal + 1 &&
      integer(s.records, 512) &&
      s.records > 0 &&
      integer(s.bytes, 262144) &&
      s.bytes >= s.records &&
      integer(s.durationMs, 30000) &&
      ((s.portReopens === 0 && s.streamInterrupted === false && s.continuity === "uninterrupted") ||
        (s.portReopens === 1 && s.streamInterrupted === true && s.continuity === "same_port_reopened")),
    "restart_observation_summary",
  );
  exactObject(value.ack, ["schema", "requestNonceSha256", "bootOrdinal", "nextBootOrdinal"]);
  check(
    value.ack.schema === "worker-qualification-restart-v1" &&
      value.ack.requestNonceSha256 === digest(context.request_nonce) &&
      value.ack.bootOrdinal === expectedBootOrdinal &&
      value.ack.nextBootOrdinal === expectedBootOrdinal + 1,
    "restart_ack_binding",
  );
  check(
    Array.isArray(value.lifecycle) &&
      value.lifecycle.length >= 4 &&
      value.lifecycle.length <= 8 &&
      Array.isArray(value.observations) &&
      value.observations.length > 0 &&
      value.observations.length <= 512,
    "restart_observation_bound",
  );
  let time = 0,
    record = 0;
  for (const row of value.lifecycle) {
    exactObject(row, ["record", "atMs", "event"]);
    check(
      integer(row.record, s.records) &&
        row.record >= record &&
        integer(row.atMs, s.durationMs) &&
        row.atMs >= time &&
        ["prearmed", "acknowledged", "stream_interrupted", "same_port_reopened", "hello_started", "complete"].includes(row.event),
      "restart_lifecycle",
    );
    record = row.record;
    time = row.atMs;
  }
  const events = value.lifecycle,
    named = (name) => events.filter((row) => row.event === name),
    ack = named("acknowledged")[0];
  check(
    named("prearmed").length === 1 &&
      events[0].event === "prearmed" &&
      events[0].record === 0 &&
      events[0].atMs === 0 &&
      named("acknowledged").length === 1 &&
      events.indexOf(ack) === 1 &&
      ack.record > 0 &&
      named("complete").length === 1 &&
      events.at(-1).event === "complete" &&
      events.at(-1).atMs === s.durationMs &&
      events.at(-1).record === s.records &&
      named("stream_interrupted").length === s.portReopens &&
      named("same_port_reopened").length === s.portReopens &&
      named("hello_started").length >= 1 &&
      named("hello_started").length <= 1 + s.portReopens,
    "restart_lifecycle_order",
  );
  const reopen = named("same_port_reopened")[0],
    interrupted = named("stream_interrupted")[0];
  if (reopen) check(events.indexOf(interrupted) > 1 && events.indexOf(reopen) === events.indexOf(interrupted) + 1, "restart_reopen_order");
  const boundary = reopen ?? ack,
    hello = named("hello_started").at(-1);
  check(events.indexOf(hello) > events.indexOf(boundary), "restart_hello_order");
  time = 0;
  record = 0;
  const observations = [];
  for (const row of value.observations) {
    exactObject(row, ["record", "atMs", "diagnostic"]);
    check(
      integer(row.record, s.records) && row.record > record && integer(row.atMs, s.durationMs) && row.atMs >= time,
      "restart_diagnostic_order",
    );
    record = row.record;
    time = row.atMs;
    const d = row.diagnostic;
    if (d?.category === "memory") {
      check(d.authoritative === false, "restart_diagnostic_authority");
      continue;
    }
    if (d?.category === "worker_admission") {
      check(d.authoritative === false && d.stage === "idle" && d.first_failure === "none", "restart_unexpected_work");
      continue;
    }
    const parsed = parseResetOriginDiagnostic(d);
    check(
      !["panic", "allocation_failure", "allocation_context"].includes(parsed.category) &&
        (parsed.category !== "startup" || (parsed.state !== "failed" && parsed.first_failure === "none")) &&
        (parsed.category !== "storage_http_status" || (parsed.spiffs_available === "true" && parsed.http_ready === "true")),
      "restart_failure_observed",
    );
    observations.push({ record: row.record, atMs: row.atMs, diagnostic: parsed });
  }
  let newBootSeen = false,
    lastBootUptime = -1;
  for (const row of observations.filter((row) => row.diagnostic.category === "boot")) {
    const d = row.diagnostic;
    if (row.record <= ack.record || (!newBootSeen && d.boot_ordinal === expectedBootOrdinal)) {
      check(d.boot_ordinal === expectedBootOrdinal, "restart_unexpected_boot");
      continue;
    }
    check(d.boot_ordinal === expectedBootOrdinal + 1 && d.reset_reason === "software_cpu", "restart_unexpected_boot");
    check(d.uptime_ms >= lastBootUptime, "restart_boot_uptime_regression");
    lastBootUptime = d.uptime_ms;
    newBootSeen = true;
  }
  const observed = observations.filter(
    (row) => row.record > boundary.record && row.record <= hello.record && row.atMs >= boundary.atMs && row.atMs <= hello.atMs,
  );
  const boot = observed.find((row) => row.diagnostic.category === "boot" && row.diagnostic.boot_ordinal === expectedBootOrdinal + 1);
  check(boot, "restart_fresh_boot_missing");
  const identities = observed.filter((row) => row.diagnostic.category === "runtime_identity");
  check(
    identities.length > 0 &&
      observations
        .filter((row) => row.diagnostic.category === "runtime_identity")
        .every(
          (row) => row.diagnostic.firmware_commit === context.firmware_commit && row.diagnostic.app_elf_sha256 === context.app_elf_sha256,
        ),
    "restart_identity_missing",
  );
  const startup = observed.filter(
    (row) =>
      row.record > boot.record &&
      row.diagnostic.category === "startup" &&
      row.diagnostic.stage === "runtime_ready" &&
      row.diagnostic.state === "complete",
  );
  check(
    startup.length >= 2 &&
      startup.every((row, index) => index === 0 || row.diagnostic.uptime_ms >= startup[index - 1].diagnostic.uptime_ms) &&
      startup.at(-1).diagnostic.uptime_ms > startup[0].diagnostic.uptime_ms,
    "restart_fresh_startup_missing",
  );
  return { summary: { ...s }, ack: { ...value.ack }, observations, lifecycle: events.map((row) => ({ ...row })) };
}

/** Preserve bounded failed-observer metadata without converting it into successful restart evidence. */
export function validateFailedRestartEvidence(value, context, expectedBootOrdinal) {
  if (value?.summary?.stage === "complete") return validateRestartEvidence(value, context, expectedBootOrdinal);
  exactObject(value, ["summary", "ack", "observations", "lifecycle"]);
  const s = value.summary;
  exactObject(s, SUMMARY);
  check(
    s.schema === "worker-qualification-restart-observation-v1" &&
      s.stage === "failed" &&
      s.expectedBootOrdinal === expectedBootOrdinal &&
      s.nextBootOrdinal === expectedBootOrdinal + 1 &&
      [
        "ackMatched",
        "bootObserved",
        "softwareResetObserved",
        "identityObserved",
        "identityMatched",
        "runtimeReadyObserved",
        "streamInterrupted",
      ].every((key) => typeof s[key] === "boolean") &&
      ["records", "bytes", "durationMs"].every((key) => Number.isSafeInteger(s[key]) && s[key] >= 0) &&
      [0, 1].includes(s.portReopens) &&
      ["uninterrupted", "interrupted", "same_port_reopened"].includes(s.continuity),
    "restart_failed_summary",
  );
  if (value.ack !== null) {
    exactObject(value.ack, ["schema", "requestNonceSha256", "bootOrdinal", "nextBootOrdinal"]);
    check(
      value.ack.schema === "worker-qualification-restart-v1" &&
        value.ack.requestNonceSha256 === digest(context.request_nonce) &&
        value.ack.bootOrdinal === expectedBootOrdinal &&
        value.ack.nextBootOrdinal === expectedBootOrdinal + 1,
      "restart_failed_ack",
    );
  }
  check(
    Array.isArray(value.observations) &&
      value.observations.length <= 512 &&
      Array.isArray(value.lifecycle) &&
      value.lifecycle.length > 0 &&
      value.lifecycle.length <= 8,
    "restart_failed_bound",
  );
  const observations = [];
  let record = 0,
    time = 0;
  for (const row of value.observations) {
    exactObject(row, ["record", "atMs", "diagnostic"]);
    check(
      integer(row.record, s.records) && row.record > record && integer(row.atMs, s.durationMs) && row.atMs >= time,
      "restart_failed_order",
    );
    record = row.record;
    time = row.atMs;
    if (["memory", "worker_admission"].includes(row.diagnostic?.category)) continue;
    observations.push({ record, atMs: time, diagnostic: parseResetOriginDiagnostic(row.diagnostic) });
  }
  record = 0;
  time = 0;
  for (const row of value.lifecycle) {
    exactObject(row, ["record", "atMs", "event"]);
    check(
      integer(row.record, s.records) &&
        row.record >= record &&
        integer(row.atMs, s.durationMs) &&
        row.atMs >= time &&
        ["prearmed", "acknowledged", "stream_interrupted", "same_port_reopened", "hello_started", "failed"].includes(row.event),
      "restart_failed_lifecycle",
    );
    record = row.record;
    time = row.atMs;
  }
  check(value.lifecycle.at(-1).event === "failed", "restart_failed_lifecycle");
  return {
    summary: { ...s },
    ack: value.ack === null ? null : { ...value.ack },
    observations,
    lifecycle: value.lifecycle.map((row) => ({ ...row })),
  };
}

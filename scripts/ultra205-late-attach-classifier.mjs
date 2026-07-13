#!/usr/bin/env node

import fs from "node:fs";
import { fileURLToPath } from "node:url";

const U64_MAX = (1n << 64n) - 1n;
const HEARTBEAT_KEYS = [
  "cadence_ms",
  "listener_armed",
  "redacted",
  "sequence",
  "session",
  "uptime_ms",
];
const ACCEPTED_STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function parseUnsignedU64(value) {
  if (!/^(?:0|[1-9][0-9]*)$/u.test(value ?? "")) return undefined;
  const parsed = BigInt(value);
  return parsed <= U64_MAX ? parsed : undefined;
}

function parseMarker(marker) {
  const tokens = marker.split(/\s+/u);
  if (tokens[0] !== "runtime_heartbeat" || tokens.length !== 7) {
    return undefined;
  }
  const entries = [];
  for (const token of tokens.slice(1)) {
    const separator = token.indexOf("=");
    if (separator <= 0 || separator === token.length - 1) return undefined;
    entries.push([token.slice(0, separator), token.slice(separator + 1)]);
  }
  const keys = entries.map(([key]) => key).sort();
  if (keys.some((key, index) => key !== [...HEARTBEAT_KEYS].sort()[index])) {
    return undefined;
  }
  return Object.fromEntries(entries);
}

export function parseHeartbeatStream(text) {
  const heartbeats = [];
  let malformed = false;
  let unexpectedLineCount = 0;
  for (const line of text.split(/\r?\n/u)) {
    if (line.trim() === "") continue;
    const maybeMarker = line.match(
      /(?:^|\s)(runtime_heartbeat(?:\s|$).*)$/u,
    )?.[1];
    if (maybeMarker === undefined) {
      unexpectedLineCount += 1;
      continue;
    }
    const values = parseMarker(maybeMarker);
    const sequence = parseUnsignedU64(values?.sequence);
    const uptimeMs = parseUnsignedU64(values?.uptime_ms);
    if (
      values === undefined ||
      !/^[0-9a-f]{32}$/u.test(values.session ?? "") ||
      sequence === undefined ||
      uptimeMs === undefined ||
      !["1000", "10000"].includes(values.cadence_ms) ||
      !["true", "false"].includes(values.listener_armed) ||
      values.redacted !== "true"
    ) {
      malformed = true;
      continue;
    }
    const expectedCadence = uptimeMs <= 120_000n ? "1000" : "10000";
    heartbeats.push({
      session: values.session,
      sequence,
      uptimeMs,
      cadenceValid: values.cadence_ms === expectedCadence,
      listenerArmed: values.listener_armed === "true",
    });
  }
  return { heartbeats, malformed, unexpectedLineCount };
}

export function validateOrderedHeartbeats(streams) {
  const heartbeats = streams.flatMap((stream) => stream.heartbeats);
  const sessions = new Set(heartbeats.map(({ session }) => session));
  let monotonic = true;
  let listenerArmed = false;
  for (let index = 0; index < heartbeats.length; index += 1) {
    const heartbeat = heartbeats[index];
    if (listenerArmed && !heartbeat.listenerArmed) monotonic = false;
    listenerArmed ||= heartbeat.listenerArmed;
    if (index === 0) continue;
    const previous = heartbeats[index - 1];
    if (
      heartbeat.sequence <= previous.sequence ||
      heartbeat.uptimeMs <= previous.uptimeMs
    ) {
      monotonic = false;
    }
  }
  return {
    heartbeatCount: heartbeats.length,
    sameSession: sessions.size <= 1,
    monotonic,
    cadenceValid: heartbeats.every(({ cadenceValid }) => cadenceValid),
    listenerArmed: heartbeats.some(({ listenerArmed }) => listenerArmed),
    session: sessions.size === 1 ? heartbeats[0]?.session : undefined,
  };
}

export function validateConnectedPreflight(espflashText, osNativeText) {
  const espflash = parseHeartbeatStream(espflashText);
  const osNative = parseHeartbeatStream(osNativeText);
  const osOrdered = validateOrderedHeartbeats([osNative]);
  const observed = validateOrderedHeartbeats([espflash, osNative]);
  const espflashValidWhenPresent =
    espflash.heartbeats.length === 0 ||
    (!espflash.malformed && observed.sameSession);
  return {
    passed:
      osNative.heartbeats.length > 0 &&
      !osNative.malformed &&
      osOrdered.sameSession &&
      osOrdered.monotonic &&
      osOrdered.cadenceValid &&
      osOrdered.listenerArmed &&
      espflashValidWhenPresent,
    espflashHeartbeatCount: espflash.heartbeats.length,
    osNativeHeartbeatCount: osNative.heartbeats.length,
    sameSession: observed.sameSession,
    osNativeMonotonic: osOrdered.monotonic,
    osNativeCadenceValid: osOrdered.cadenceValid,
    osNativeListenerArmed: osOrdered.listenerArmed,
    session: observed.session,
  };
}

function markerPayload(line, marker) {
  return line.match(new RegExp(`(?:^|\\s)(${marker}(?:\\s|$).*)$`, "u"))?.[1];
}

function parseBootEvidence(text) {
  const sessions = new Set();
  const states = new Set();
  let malformed = false;
  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes("plan13_boot_evidence")) continue;
    const marker = markerPayload(line, "plan13_boot_evidence");
    const match = marker?.match(
      /^plan13_boot_evidence session=([0-9a-f]{32}) state=(booted|listener_armed) redacted=true$/u,
    );
    if (match === undefined || match === null) {
      malformed = true;
      continue;
    }
    sessions.add(match[1]);
    states.add(match[2]);
  }
  return { sessions, states, malformed };
}

function parseAcceptedStages(text) {
  const observed = new Set();
  let malformed = false;
  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes("accepted_state_snapshot")) continue;
    const marker = markerPayload(line, "accepted_state_snapshot");
    const match = marker?.match(/^accepted_state_snapshot stage=([^\s]+)\s/u);
    if (match === undefined || match === null || !marker?.endsWith(" redacted=true")) {
      malformed = true;
      continue;
    }
    if (!ACCEPTED_STAGES.includes(match[1])) {
      malformed = true;
      continue;
    }
    observed.add(match[1]);
  }
  return { observed, malformed };
}

function hasRuntimeHazard(text) {
  return /stack overflow|stack canary|guru meditation|panic(?:ed)?|abort\(\)|SW_CPU_RESET|RTC_SW_(?:SYS|CPU)_RST|software reset/iu.test(
    text,
  );
}

export function qualifyOsNativeColdStream(osNativeText, preflightSession) {
  const stream = parseHeartbeatStream(osNativeText);
  const ordered = validateOrderedHeartbeats([stream]);
  const bootEvidence = parseBootEvidence(osNativeText);
  const acceptedStages = parseAcceptedStages(osNativeText);
  const heartbeatSession = ordered.session;
  const evidenceSession =
    bootEvidence.sessions.size === 1 ? [...bootEvidence.sessions][0] : undefined;
  const sessionConsistent =
    heartbeatSession !== undefined &&
    evidenceSession !== undefined &&
    heartbeatSession === evidenceSession;
  const newSession =
    /^[0-9a-f]{32}$/u.test(preflightSession ?? "") &&
    sessionConsistent &&
    heartbeatSession !== preflightSession;
  const bootEvidenceComplete =
    bootEvidence.states.has("booted") &&
    bootEvidence.states.has("listener_armed");
  const acceptedStateReplayComplete = ACCEPTED_STAGES.every((stage) =>
    acceptedStages.observed.has(stage),
  );
  const applicationByteCount = Buffer.byteLength(osNativeText);
  const runtimeHazard = hasRuntimeHazard(osNativeText);
  const passed =
    applicationByteCount > 0 &&
    stream.heartbeats.length >= 3 &&
    !stream.malformed &&
    ordered.sameSession &&
    ordered.monotonic &&
    ordered.cadenceValid &&
    ordered.listenerArmed &&
    !bootEvidence.malformed &&
    bootEvidence.sessions.size === 1 &&
    bootEvidenceComplete &&
    !acceptedStages.malformed &&
    acceptedStateReplayComplete &&
    !runtimeHazard &&
    sessionConsistent &&
    newSession;
  return {
    schema_version: "ultra205-os-native-cold-qualification-v2",
    category: passed ? "native_cold_delivers" : "cold_native_evidence_invalid",
    application_byte_count: applicationByteCount,
    heartbeat_count: stream.heartbeats.length,
    same_session: ordered.sameSession,
    monotonic: ordered.monotonic,
    cadence_valid: ordered.cadenceValid,
    listener_armed: ordered.listenerArmed,
    boot_evidence_replay_complete: bootEvidenceComplete,
    accepted_state_stage_count: acceptedStages.observed.size,
    accepted_state_replay_complete: acceptedStateReplayComplete,
    one_cold_session: sessionConsistent,
    new_cold_session: newSession,
    malformed:
      stream.malformed || bootEvidence.malformed || acceptedStages.malformed,
    runtime_hazard: runtimeHazard,
    session: heartbeatSession ?? null,
    unexpected_non_heartbeat_line_count: stream.unexpectedLineCount,
  };
}

export function classifyLateAttachStreams(espflashBeforeText, osNativeText, espflashAfterText) {
  const streams = [
    parseHeartbeatStream(espflashBeforeText),
    parseHeartbeatStream(osNativeText),
    parseHeartbeatStream(espflashAfterText),
  ];
  const [espflashBefore, osNative, espflashAfter] = streams;
  const ordered = validateOrderedHeartbeats(streams);
  const counts = streams.map(({ heartbeats }) => heartbeats.length);
  const delivered = counts.map((count) => count > 0);
  let category;
  if (
    streams.some(({ malformed }) => malformed) ||
    streams.some(({ unexpectedLineCount }) => unexpectedLineCount > 0)
  ) {
    category = "unexpected_non_heartbeat_bytes";
  } else if (!ordered.sameSession || !ordered.monotonic || !ordered.cadenceValid) {
    category = "inconclusive_mixed_delivery";
  } else if (delivered.every(Boolean)) {
    category = "all_readers_deliver";
  } else if (!delivered[0] && delivered[1] && delivered[2]) {
    category = "os_open_activates_transport";
  } else if (!delivered[0] && delivered[1] && !delivered[2]) {
    category = "espflash_reader_silent";
  } else if (!delivered[0] && !delivered[1] && !delivered[2]) {
    category = "late_attach_transport_silent";
  } else if (!delivered[1] && (delivered[0] || delivered[2])) {
    category = "os_reader_silent";
  } else {
    category = "inconclusive_mixed_delivery";
  }
  return {
    schema_version: "ultra205-late-attach-classification-v1",
    category,
    espflash_before_heartbeat_count: espflashBefore.heartbeats.length,
    os_native_heartbeat_count: osNative.heartbeats.length,
    espflash_after_heartbeat_count: espflashAfter.heartbeats.length,
    same_session: ordered.sameSession,
    monotonic: ordered.monotonic,
    cadence_valid: ordered.cadenceValid,
    unexpected_non_heartbeat_line_count: streams.reduce(
      (count, stream) => count + stream.unexpectedLineCount,
      0,
    ),
  };
}

function main() {
  const command = process.argv[2];
  if (command === "preflight" && process.argv.length === 5) {
    const result = validateConnectedPreflight(
      fs.readFileSync(process.argv[3], "utf8"),
      fs.readFileSync(process.argv[4], "utf8"),
    );
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(result.passed ? 0 : 1);
  }
  if (command === "classify" && process.argv.length === 6) {
    const result = classifyLateAttachStreams(
      fs.readFileSync(process.argv[3], "utf8"),
      fs.readFileSync(process.argv[4], "utf8"),
      fs.readFileSync(process.argv[5], "utf8"),
    );
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return;
  }
  if (command === "qualify-os-native" && process.argv.length === 5) {
    const result = qualifyOsNativeColdStream(
      fs.readFileSync(process.argv[3], "utf8"),
      process.argv[4],
    );
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(result.category === "native_cold_delivers" ? 0 : 1);
  }
  throw new Error(
    "usage: ultra205-late-attach-classifier.mjs preflight ESP OS | classify ESP1 OS ESP2 | qualify-os-native OS PREFLIGHT_SESSION",
  );
}

if (
  process.argv[1] !== undefined &&
  fs.realpathSync(fileURLToPath(import.meta.url)) === fs.realpathSync(process.argv[1])
) {
  main();
}

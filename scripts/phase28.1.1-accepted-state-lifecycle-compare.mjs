#!/usr/bin/env node

import fs from "node:fs";
import { fileURLToPath } from "node:url";

import { parseAcceptedStateLog } from "./phase28.1.1-accepted-state-compare.mjs";

export const ACCEPTED_STATE_LIFECYCLE_STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

const U64_MAX = (1n << 64n) - 1n;
const HEARTBEAT_KEYS = [
  "cadence_ms",
  "listener_armed",
  "redacted",
  "sequence",
  "session",
  "uptime_ms",
];
const HEARTBEAT_FAULT_PRECEDENCE = [
  "malformed",
  "session_conflict",
  "monotonicity_failed",
  "cadence_invalid",
  "absent",
  "listener_proof_absent",
];

export class AcceptedStateLifecycleError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "AcceptedStateLifecycleError";
    this.code = code;
  }
}

function memberPrefix(memberName) {
  return memberName === "reinit" ? "reinit" : "cold_start";
}

function lifecycleError(memberName, suffix, message) {
  return new AcceptedStateLifecycleError(
    `${memberPrefix(memberName)}_${suffix}`,
    `accepted_state_lifecycle_error: ${memberName} ${message}`,
  );
}

function field(line, name) {
  return line.match(new RegExp(`(?:^|\\s)${name}=([^\\s]+)`))?.[1];
}

function markerFromLine(line) {
  return line.match(/(?:^|\s)(accepted_state_snapshot(?:\s|$).*)$/u)?.[1];
}

function snapshotsEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function parseValues(marker, token, expectedKeys) {
  const tokens = marker.split(/\s+/u);
  if (tokens[0] !== token || tokens.length !== expectedKeys.length + 1) {
    return undefined;
  }
  const entries = [];
  for (const fieldToken of tokens.slice(1)) {
    const separator = fieldToken.indexOf("=");
    if (separator <= 0 || separator === fieldToken.length - 1) return undefined;
    entries.push([
      fieldToken.slice(0, separator),
      fieldToken.slice(separator + 1),
    ]);
  }
  const keys = entries.map(([name]) => name).sort();
  if (!snapshotsEqual(keys, [...expectedKeys].sort())) return undefined;
  return Object.fromEntries(entries);
}

function parseUnsignedU64(value) {
  if (!/^(?:0|[1-9][0-9]*)$/u.test(value ?? "")) return undefined;
  const parsed = BigInt(value);
  return parsed <= U64_MAX ? parsed : undefined;
}

function collectBootEvidence(text, memberName) {
  const sessions = new Map();
  let markerCount = 0;
  let malformed = false;
  let originalBootCount = 0;
  let originalListenerCount = 0;

  for (const line of text.split(/\r?\n/u)) {
    if (line.includes("bitaxe-rust boot")) originalBootCount += 1;
    if (line.includes("h4_continuous_result=listener_armed")) {
      originalListenerCount += 1;
    }
    if (!line.includes("plan13_boot_evidence")) continue;
    const maybeMarker = line.match(
      /(?:^|\s)(plan13_boot_evidence(?:\s|$).*)$/u,
    )?.[1];
    const values =
      maybeMarker === undefined
        ? undefined
        : parseValues(maybeMarker, "plan13_boot_evidence", [
            "redacted",
            "session",
            "state",
          ]);
    if (
      values === undefined ||
      !/^[0-9a-f]{32}$/u.test(values.session ?? "") ||
      !["booted", "listener_armed"].includes(values.state) ||
      values.redacted !== "true"
    ) {
      malformed = true;
      continue;
    }
    markerCount += 1;
    const states = sessions.get(values.session) ?? new Set();
    states.add(values.state);
    sessions.set(values.session, states);
  }

  return {
    memberName,
    sessions,
    markerCount,
    malformed,
    originalBootCount,
    originalListenerCount,
  };
}

function collectHeartbeats(text, memberName) {
  const heartbeats = [];
  let malformed = false;
  let cadenceInvalid = false;

  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes("runtime_heartbeat")) continue;
    const maybeMarker = line.match(
      /(?:^|\s)(runtime_heartbeat(?:\s|$).*)$/u,
    )?.[1];
    const values =
      maybeMarker === undefined
        ? undefined
        : parseValues(maybeMarker, "runtime_heartbeat", HEARTBEAT_KEYS);
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
    if (values.cadence_ms !== expectedCadence) cadenceInvalid = true;
    heartbeats.push({
      session: values.session,
      sequence,
      uptimeMs,
      listenerArmed: values.listener_armed === "true",
    });
  }

  let monotonicityFailed = false;
  for (let index = 1; index < heartbeats.length; index += 1) {
    const previous = heartbeats[index - 1];
    const current = heartbeats[index];
    if (
      current.sequence <= previous.sequence ||
      current.uptimeMs <= previous.uptimeMs ||
      (previous.listenerArmed && !current.listenerArmed)
    ) {
      monotonicityFailed = true;
    }
  }

  return {
    memberName,
    heartbeats,
    malformed,
    cadenceInvalid,
    monotonicityFailed,
  };
}

function heartbeatFaults(boot, heartbeat, { requireOriginalMarkers }) {
  const heartbeatSessions = new Set(
    heartbeat.heartbeats.map(({ session }) => session),
  );
  const unifiedSessions = new Set([...boot.sessions.keys(), ...heartbeatSessions]);
  const dedicatedStates = new Set(
    [...boot.sessions.values()].flatMap((states) => [...states]),
  );
  const heartbeatListener = heartbeat.heartbeats.some(
    ({ listenerArmed }) => listenerArmed,
  );
  const listenerProof = requireOriginalMarkers
    ? boot.originalListenerCount === 1 && dedicatedStates.has("listener_armed")
    : dedicatedStates.has("listener_armed") || heartbeatListener;
  return {
    malformed: heartbeat.malformed,
    session_conflict:
      heartbeatSessions.size > 1 ||
      (heartbeat.heartbeats.length > 0 && unifiedSessions.size > 1),
    monotonicity_failed: heartbeat.monotonicityFailed,
    cadence_invalid: heartbeat.cadenceInvalid,
    absent: heartbeat.heartbeats.length === 0,
    listener_proof_absent: !listenerProof,
  };
}

function selectHeartbeatFault(reinitCandidate, coldCandidate) {
  for (const kind of HEARTBEAT_FAULT_PRECEDENCE) {
    for (const candidate of [reinitCandidate, coldCandidate]) {
      if (!candidate.faults[kind]) continue;
      const suffix =
        kind === "listener_proof_absent"
          ? "listener_proof_absent"
          : `heartbeat_${kind}`;
      return lifecycleError(
        candidate.memberName,
        suffix,
        `failed ${kind.replaceAll("_", " ")} validation`,
      );
    }
  }
  return undefined;
}

function validateDedicatedBootEvidence(boot, { requireOriginalMarkers }) {
  const { memberName } = boot;
  if (boot.malformed) {
    throw lifecycleError(
      memberName,
      "boot_evidence_malformed",
      "boot evidence is malformed",
    );
  }
  if (requireOriginalMarkers && boot.originalBootCount !== 1) {
    throw lifecycleError(
      memberName,
      "boot_proof_absent",
      "original boot proof is absent or repeated",
    );
  }
  if (!requireOriginalMarkers && boot.originalBootCount > 1) {
    throw lifecycleError(
      memberName,
      "multiple_boot_sessions",
      "original runtime markers indicate multiple boots",
    );
  }
  if (requireOriginalMarkers && boot.originalListenerCount !== 1) {
    throw lifecycleError(
      memberName,
      "listener_proof_absent",
      "original listener proof is absent or repeated",
    );
  }
  if (!requireOriginalMarkers && boot.originalListenerCount > 1) {
    throw lifecycleError(
      memberName,
      "multiple_boot_sessions",
      "original runtime markers indicate multiple boots",
    );
  }
  if (boot.sessions.size === 0) {
    if (requireOriginalMarkers) {
      throw lifecycleError(
        memberName,
        "boot_proof_absent",
        "dedicated boot proof is absent",
      );
    }
    return;
  }
  if (boot.sessions.size !== 1) {
    throw lifecycleError(
      memberName,
      "multiple_boot_sessions",
      "has multiple boot sessions",
    );
  }
  const states = boot.sessions.values().next().value;
  if (requireOriginalMarkers && !states.has("booted")) {
    throw lifecycleError(
      memberName,
      "boot_proof_absent",
      "dedicated boot proof is absent",
    );
  }
  if (requireOriginalMarkers && !states.has("listener_armed")) {
    throw lifecycleError(
      memberName,
      "listener_proof_absent",
      "dedicated listener proof is absent",
    );
  }
}

function heartbeatCategory(heartbeats) {
  const early = heartbeats.some(({ uptimeMs }) => uptimeMs <= 120_000n);
  const steady = heartbeats.some(({ uptimeMs }) => uptimeMs > 120_000n);
  if (early && steady) return "early_and_steady";
  if (early) return "early_only";
  return "steady_only";
}

export function parsePlan13BootEvidenceMember(
  text,
  memberName,
  { requireOriginalMarkers },
) {
  const boot = collectBootEvidence(text, memberName);
  validateDedicatedBootEvidence(boot, { requireOriginalMarkers });
  const states = new Set(
    [...boot.sessions.values()].flatMap((sessionStates) => [...sessionStates]),
  );
  if (!states.has("booted")) {
    throw lifecycleError(memberName, "boot_proof_absent", "boot proof is absent");
  }
  if (!states.has("listener_armed")) {
    throw lifecycleError(
      memberName,
      "listener_proof_absent",
      "listener proof is absent",
    );
  }
  return {
    bootSessionCount: boot.sessions.size,
    bootEvidenceStateCount: states.size,
    bootEvidenceMarkerCount: boot.markerCount,
    equivalentDuplicates: boot.markerCount > states.size,
    originalBootCount: boot.originalBootCount,
    originalListenerCount: boot.originalListenerCount,
  };
}

export function parseRuntimeHeartbeatMember(text, memberName) {
  const heartbeat = collectHeartbeats(text, memberName);
  const boot = collectBootEvidence(text, memberName);
  const faults = heartbeatFaults(boot, heartbeat, {
    requireOriginalMarkers: memberName === "reinit",
  });
  const selected = selectHeartbeatFault(
    memberName === "reinit"
      ? { memberName, faults }
      : { memberName: "reinit", faults: {} },
    memberName === "reinit"
      ? { memberName: "cold-start", faults: {} }
      : { memberName, faults },
  );
  if (selected !== undefined) throw selected;
  return heartbeat;
}

export function parseAcceptedStateLifecycleMember(text, memberName) {
  const snapshots = new Map();
  let markerCount = 0;
  for (const line of text.split(/\r?\n/u)) {
    const marker = markerFromLine(line);
    if (marker === undefined) continue;
    markerCount += 1;
    if (field(marker, "redacted") !== "true") {
      throw lifecycleError(memberName, "lifecycle_invalid", "marker is not redacted");
    }
    if (field(marker, "observation") === "unavailable") {
      throw lifecycleError(
        memberName,
        "lifecycle_invalid",
        "contains unavailable observation",
      );
    }
    const parsed = parseAcceptedStateLog(marker);
    if (parsed.size !== 1) {
      throw lifecycleError(memberName, "lifecycle_invalid", "marker did not parse");
    }
    const [stage, snapshot] = parsed.entries().next().value;
    const maybeExisting = snapshots.get(stage);
    if (maybeExisting !== undefined && !snapshotsEqual(maybeExisting, snapshot)) {
      throw lifecycleError(
        memberName,
        "lifecycle_invalid",
        `has conflicting duplicate for ${stage}`,
      );
    }
    snapshots.set(stage, snapshot);
  }
  const actualStages = [...snapshots.keys()].sort();
  const expectedStages = [...ACCEPTED_STATE_LIFECYCLE_STAGES].sort();
  if (!snapshotsEqual(actualStages, expectedStages)) {
    throw lifecycleError(memberName, "lifecycle_invalid", "stage set is incomplete");
  }
  return {
    snapshots,
    markerCount,
    equivalentDuplicates: markerCount >= snapshots.size,
  };
}

export function compareAcceptedStateLifecycle(reinitText, coldStartText) {
  const reinitBootCandidate = collectBootEvidence(reinitText, "reinit");
  const coldBootCandidate = collectBootEvidence(coldStartText, "cold-start");
  const reinitHeartbeat = collectHeartbeats(reinitText, "reinit");
  const coldHeartbeat = collectHeartbeats(coldStartText, "cold-start");
  const reinitFaults = heartbeatFaults(reinitBootCandidate, reinitHeartbeat, {
    requireOriginalMarkers: true,
  });
  const coldFaults = heartbeatFaults(coldBootCandidate, coldHeartbeat, {
    requireOriginalMarkers: false,
  });
  const maybeHeartbeatFault = selectHeartbeatFault(
    { memberName: "reinit", faults: reinitFaults },
    { memberName: "cold-start", faults: coldFaults },
  );
  if (maybeHeartbeatFault !== undefined) throw maybeHeartbeatFault;

  validateDedicatedBootEvidence(reinitBootCandidate, {
    requireOriginalMarkers: true,
  });
  validateDedicatedBootEvidence(coldBootCandidate, {
    requireOriginalMarkers: false,
  });
  const reinitStates = new Set(
    [...reinitBootCandidate.sessions.values()].flatMap((states) => [...states]),
  );
  if (!reinitStates.has("booted")) {
    throw lifecycleError("reinit", "boot_proof_absent", "boot proof is absent");
  }
  if (!reinitStates.has("listener_armed")) {
    throw lifecycleError(
      "reinit",
      "listener_proof_absent",
      "listener proof is absent",
    );
  }

  const reinit = parseAcceptedStateLifecycleMember(reinitText, "reinit");
  const coldStart = parseAcceptedStateLifecycleMember(coldStartText, "cold-start");
  const stageStatus = Object.fromEntries(
    ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => [
      `stage_${stage}`,
      snapshotsEqual(reinit.snapshots.get(stage), coldStart.snapshots.get(stage))
        ? "match"
        : "mismatch",
    ]),
  );
  const lifecycleStatus = Object.values(stageStatus).every(
    (status) => status === "match",
  )
    ? "match"
    : "mismatch";
  const reinitDedicatedStates = new Set(
    [...reinitBootCandidate.sessions.values()].flatMap((states) => [...states]),
  );
  const coldDedicatedStates = new Set(
    [...coldBootCandidate.sessions.values()].flatMap((states) => [...states]),
  );

  return {
    lifecycle_status: lifecycleStatus,
    reinit_stage_count: reinit.snapshots.size,
    cold_start_stage_count: coldStart.snapshots.size,
    reinit_marker_count: reinit.markerCount,
    cold_start_marker_count: coldStart.markerCount,
    reinit_equivalent_duplicates: reinit.equivalentDuplicates,
    cold_start_equivalent_duplicates: coldStart.equivalentDuplicates,
    reinit_boot_session_count: 1,
    cold_start_boot_session_count: 1,
    reinit_boot_evidence_state_count: reinitDedicatedStates.size,
    cold_start_boot_evidence_state_count: coldDedicatedStates.size,
    reinit_boot_evidence_marker_count: reinitBootCandidate.markerCount,
    cold_start_boot_evidence_marker_count: coldBootCandidate.markerCount,
    reinit_boot_evidence_equivalent_duplicates:
      reinitBootCandidate.markerCount > reinitDedicatedStates.size,
    cold_start_boot_evidence_equivalent_duplicates:
      coldBootCandidate.markerCount > coldDedicatedStates.size,
    reinit_heartbeat_count: reinitHeartbeat.heartbeats.length,
    cold_start_heartbeat_count: coldHeartbeat.heartbeats.length,
    reinit_heartbeat_present: true,
    cold_start_heartbeat_present: true,
    reinit_heartbeat_uptime_category: heartbeatCategory(
      reinitHeartbeat.heartbeats,
    ),
    cold_start_heartbeat_uptime_category: heartbeatCategory(
      coldHeartbeat.heartbeats,
    ),
    reinit_heartbeat_cadence_category: "valid",
    cold_start_heartbeat_cadence_category: "valid",
    reinit_listener_fallback_used: false,
    cold_start_listener_fallback_used:
      !coldDedicatedStates.has("listener_armed") &&
      coldHeartbeat.heartbeats.some(({ listenerArmed }) => listenerArmed),
    ...stageStatus,
    redacted: true,
  };
}

export function unavailableAcceptedStateLifecycle() {
  return {
    lifecycle_status: "unavailable",
    reinit_stage_count: 0,
    cold_start_stage_count: 0,
    reinit_marker_count: 0,
    cold_start_marker_count: 0,
    reinit_equivalent_duplicates: false,
    cold_start_equivalent_duplicates: false,
    reinit_boot_session_count: 0,
    cold_start_boot_session_count: 0,
    reinit_boot_evidence_state_count: 0,
    cold_start_boot_evidence_state_count: 0,
    reinit_boot_evidence_marker_count: 0,
    cold_start_boot_evidence_marker_count: 0,
    reinit_boot_evidence_equivalent_duplicates: false,
    cold_start_boot_evidence_equivalent_duplicates: false,
    reinit_heartbeat_count: 0,
    cold_start_heartbeat_count: 0,
    reinit_heartbeat_present: false,
    cold_start_heartbeat_present: false,
    reinit_heartbeat_uptime_category: "unavailable",
    cold_start_heartbeat_uptime_category: "unavailable",
    reinit_heartbeat_cadence_category: "unavailable",
    cold_start_heartbeat_cadence_category: "unavailable",
    reinit_listener_fallback_used: false,
    cold_start_listener_fallback_used: false,
    ...Object.fromEntries(
      ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => [
        `stage_${stage}`,
        "unavailable",
      ]),
    ),
    redacted: true,
  };
}

export function renderAcceptedStateLifecycle(report) {
  return `${Object.entries(report)
    .map(([name, value]) => `${name}: ${String(value)}`)
    .join("\n")}\n`;
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const name = argv[index];
    if (name === "--unavailable") {
      args.set(name, true);
      continue;
    }
    const value = argv[index + 1];
    if (!name?.startsWith("--") || value === undefined) {
      throw new AcceptedStateLifecycleError(
        "validator_error",
        "accepted_state_lifecycle_error: invalid arguments",
      );
    }
    args.set(name, value);
    index += 1;
  }
  if (!args.has("--out")) {
    throw new AcceptedStateLifecycleError(
      "validator_error",
      "accepted_state_lifecycle_error: missing --out",
    );
  }
  if (args.has("--unavailable")) return args;
  for (const required of ["--reinit-log", "--cold-start-log"]) {
    if (!args.has(required)) {
      throw new AcceptedStateLifecycleError(
        "validator_error",
        `accepted_state_lifecycle_error: missing ${required}`,
      );
    }
  }
  return args;
}

function isMainModule() {
  if (process.argv[1] === undefined) return false;
  try {
    return (
      fs.realpathSync(fileURLToPath(import.meta.url)) ===
      fs.realpathSync(process.argv[1])
    );
  } catch {
    return false;
  }
}

if (isMainModule()) {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = args.has("--unavailable")
      ? unavailableAcceptedStateLifecycle()
      : compareAcceptedStateLifecycle(
          fs.readFileSync(args.get("--reinit-log"), "utf8"),
          fs.readFileSync(args.get("--cold-start-log"), "utf8"),
        );
    fs.writeFileSync(args.get("--out"), renderAcceptedStateLifecycle(report));
  } catch (error) {
    const code =
      error instanceof AcceptedStateLifecycleError
        ? error.code
        : "validator_error";
    process.stderr.write(`accepted_state_lifecycle_failure_code=${code}\n`);
    process.exitCode = 1;
  }
}

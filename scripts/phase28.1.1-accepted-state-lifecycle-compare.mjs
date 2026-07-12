#!/usr/bin/env node

import fs from "node:fs";

import { parseAcceptedStateLog } from "./phase28.1.1-accepted-state-compare.mjs";

export const ACCEPTED_STATE_LIFECYCLE_STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function field(line, name) {
  return line.match(new RegExp(`(?:^|\\s)${name}=([^\\s]+)`))?.[1];
}

function markerFromLine(line) {
  const match = line.match(/(?:^|\s)(accepted_state_snapshot(?:\s|$).*)$/u);
  return match?.[1];
}

function snapshotsEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function parsePlan13BootEvidenceMember(
  text,
  memberName,
  { requireOriginalMarkers },
) {
  const sessions = new Map();
  let markerCount = 0;
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
    if (maybeMarker === undefined) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} boot evidence is malformed`,
      );
    }
    const tokens = maybeMarker.split(/\s+/u);
    if (tokens.length !== 4) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} boot evidence is malformed`,
      );
    }
    const values = Object.fromEntries(
      tokens.slice(1).map((token) => {
        const separator = token.indexOf("=");
        return [token.slice(0, separator), token.slice(separator + 1)];
      }),
    );
    if (
      Object.keys(values).sort().join(",") !== "redacted,session,state" ||
      !/^[0-9a-f]{32}$/u.test(values.session ?? "") ||
      !["booted", "listener_armed"].includes(values.state) ||
      values.redacted !== "true"
    ) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} boot evidence is malformed`,
      );
    }
    markerCount += 1;
    const states = sessions.get(values.session) ?? new Set();
    states.add(values.state);
    sessions.set(values.session, states);
  }

  if (requireOriginalMarkers) {
    if (originalBootCount !== 1) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} original boot proof is absent or repeated`,
      );
    }
    if (originalListenerCount !== 1) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} original listener proof is absent or repeated`,
      );
    }
  } else if (originalBootCount > 1 || originalListenerCount > 1) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} original runtime markers indicate multiple boots`,
    );
  }
  if (sessions.size === 0) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} boot proof is absent`,
    );
  }
  if (sessions.size !== 1) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} has multiple boot sessions`,
    );
  }
  const states = sessions.values().next().value;
  if (!states.has("booted")) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} boot proof is absent`,
    );
  }
  if (!states.has("listener_armed")) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} listener proof is absent`,
    );
  }

  return {
    bootSessionCount: sessions.size,
    bootEvidenceStateCount: states.size,
    bootEvidenceMarkerCount: markerCount,
    equivalentDuplicates: markerCount > states.size,
    originalBootCount,
    originalListenerCount,
  };
}

export function parseAcceptedStateLifecycleMember(text, memberName) {
  const snapshots = new Map();
  let markerCount = 0;

  for (const line of text.split(/\r?\n/u)) {
    const marker = markerFromLine(line);
    if (marker === undefined) continue;

    markerCount += 1;
    if (field(marker, "redacted") !== "true") {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} marker is not redacted`,
      );
    }
    if (field(marker, "observation") === "unavailable") {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} contains unavailable observation`,
      );
    }

    const parsed = parseAcceptedStateLog(marker);
    if (parsed.size !== 1) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} marker did not parse`,
      );
    }
    const [stage, snapshot] = parsed.entries().next().value;
    const maybeExisting = snapshots.get(stage);
    if (maybeExisting !== undefined && !snapshotsEqual(maybeExisting, snapshot)) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} has conflicting duplicate for ${stage}`,
      );
    }
    snapshots.set(stage, snapshot);
  }

  const actualStages = [...snapshots.keys()].sort();
  const expectedStages = [...ACCEPTED_STATE_LIFECYCLE_STAGES].sort();
  if (!snapshotsEqual(actualStages, expectedStages)) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} stage set is incomplete`,
    );
  }

  return {
    snapshots,
    markerCount,
    equivalentDuplicates: markerCount >= snapshots.size,
  };
}

export function compareAcceptedStateLifecycle(reinitText, coldStartText) {
  const reinitBoot = parsePlan13BootEvidenceMember(reinitText, "reinit", {
    requireOriginalMarkers: true,
  });
  const coldStartBoot = parsePlan13BootEvidenceMember(
    coldStartText,
    "cold-start",
    { requireOriginalMarkers: false },
  );
  const reinit = parseAcceptedStateLifecycleMember(reinitText, "reinit");
  const coldStart = parseAcceptedStateLifecycleMember(
    coldStartText,
    "cold-start",
  );
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

  return {
    lifecycle_status: lifecycleStatus,
    reinit_stage_count: reinit.snapshots.size,
    cold_start_stage_count: coldStart.snapshots.size,
    reinit_marker_count: reinit.markerCount,
    cold_start_marker_count: coldStart.markerCount,
    reinit_equivalent_duplicates: reinit.equivalentDuplicates,
    cold_start_equivalent_duplicates: coldStart.equivalentDuplicates,
    reinit_boot_session_count: reinitBoot.bootSessionCount,
    cold_start_boot_session_count: coldStartBoot.bootSessionCount,
    reinit_boot_evidence_state_count: reinitBoot.bootEvidenceStateCount,
    cold_start_boot_evidence_state_count:
      coldStartBoot.bootEvidenceStateCount,
    reinit_boot_evidence_marker_count: reinitBoot.bootEvidenceMarkerCount,
    cold_start_boot_evidence_marker_count:
      coldStartBoot.bootEvidenceMarkerCount,
    reinit_boot_evidence_equivalent_duplicates:
      reinitBoot.equivalentDuplicates,
    cold_start_boot_evidence_equivalent_duplicates:
      coldStartBoot.equivalentDuplicates,
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
      throw new Error("accepted_state_lifecycle_error: invalid arguments");
    }
    args.set(name, value);
    index += 1;
  }
  if (!args.has("--out")) {
    throw new Error("accepted_state_lifecycle_error: missing --out");
  }
  if (args.has("--unavailable")) return args;
  for (const required of ["--reinit-log", "--cold-start-log"]) {
    if (!args.has(required)) {
      throw new Error(`accepted_state_lifecycle_error: missing ${required}`);
    }
  }
  return args;
}

if (import.meta.url === `file://${process.argv[1]}`) {
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
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
